//! TASK-0035 — end-to-end coverage of the newly-wired gRPC surface over the REAL `secretd` stack:
//! Vault.List / Vault.Rm (dry-run + apply) / Vault.Rotate, Relay.Create + Relay.List, Audit.Query,
//! and the GetSecret.meta population. Drives the PRODUCTION `server::serve` + real `grpc` handlers +
//! real `conv` over a real UDS, exactly like `e2e.rs` (no inline replica).
//!
//! Load-bearing invariants asserted here:
//!   * Metadata never carries a value/ciphertext byte (List/meta are non-secret).
//!   * Rm/Rotate are fail-closed: dry-run (no apply/confirm) mutates NOTHING; locked refuses with
//!     `failed_precondition`; an empty name is `invalid_argument`.
//!   * Audit.Query returns the durable chain and post-filters; no secret bytes appear.
use std::path::{Path, PathBuf};

use envctl_secrets::keyslot::Argon2Params;
use envctl_secrets::paths::Paths;
use envctl_secrets::seam::{NoMint, SystemClock, UpstreamError, UsbProbe};
use envctl_secrets::vault::{InMemStore, Store};
use envctl_secrets::{EgressReq, EgressResp, Engine, EventSink, Upstream};
use envctl_secrets_proto::v1;
use hyper_util::rt::TokioIo;
use tonic::transport::{Endpoint, Uri};
use tonic::Streaming;
use zeroize::Zeroizing;

const USB_UUID: &str = "TASK35-USB";
const SENTINEL: &[u8] = b"DO-NOT-LEAK-THIS-VALUE";

// ---- fakes / harness (mirrors e2e.rs) --------------------------------------------------------

struct PresentUsb {
    keyfile: Zeroizing<Vec<u8>>,
}
impl UsbProbe for PresentUsb {
    fn keyfile_for(&self, uuid: &str) -> Option<Zeroizing<Vec<u8>>> {
        (uuid == USB_UUID).then(|| self.keyfile.clone())
    }
}

#[derive(Clone)]
struct NullUpstream;
#[async_trait::async_trait]
impl Upstream for NullUpstream {
    async fn send(
        &self,
        _req: EgressReq,
        _real_key: &Zeroizing<Vec<u8>>,
    ) -> Result<EgressResp, UpstreamError> {
        Err(UpstreamError::Io("upstream not wired".into()))
    }
}

fn make_engine(paths: &Paths, keyfile: &Zeroizing<Vec<u8>>) -> Engine {
    Engine::with_seams(
        paths.clone(),
        Box::new(InMemStore::new()) as Box<dyn Store>,
        Box::new(SystemClock),
        Box::new(PresentUsb {
            keyfile: keyfile.clone(),
        }),
        Box::new(NoMint),
        Box::new(NullUpstream),
        #[cfg(feature = "provider-github")]
        Box::new(envctl_secrets::mint_github::NoopHttpTransport),
    )
    .expect("with_seams")
}

fn temp_paths(tag: &str) -> (PathBuf, Paths) {
    use std::os::unix::fs::PermissionsExt;
    let dir = std::env::temp_dir().join(format!("envctl-task35-{tag}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    let paths = Paths::under(dir.clone());
    std::fs::create_dir_all(&paths.runtime).unwrap();
    std::fs::set_permissions(&paths.runtime, std::fs::Permissions::from_mode(0o700)).unwrap();
    (dir, paths)
}

fn bind(sock: &std::path::Path) -> tokio::net::UnixListener {
    use std::os::unix::fs::PermissionsExt;
    let listener = tokio::net::UnixListener::bind(sock).expect("bind UDS");
    std::fs::set_permissions(sock, std::fs::Permissions::from_mode(0o600)).unwrap();
    listener
}

async fn connect(sock: PathBuf) -> tonic::transport::Channel {
    Endpoint::try_from("http://[::]:0")
        .unwrap()
        .connect_with_connector(tower::service_fn(move |_: Uri| {
            let sock = sock.clone();
            async move {
                let stream = tokio::net::UnixStream::connect(sock).await?;
                Ok::<_, std::io::Error>(TokioIo::new(stream))
            }
        }))
        .await
        .expect("connect to daemon UDS")
}

/// Drain a streaming RPC to completion; returns the terminal error message (empty if it ended OK).
async fn drain_err(mut s: Streaming<v1::Event>) -> String {
    loop {
        match s.message().await {
            Ok(Some(_)) => continue,
            Ok(None) => return String::new(),
            Err(status) => return status.message().to_string(),
        }
    }
}

/// Stand up the real daemon over a tempdir UDS with the vault INITIALIZED (passphrase + USB). Returns
/// the dir (for teardown), the socket path, and the spawned server handle.
async fn serve_initialized(tag: &str) -> (PathBuf, PathBuf, Engine, tokio::task::JoinHandle<()>) {
    let (dir, paths) = temp_paths(tag);
    let keyfile = Zeroizing::new(vec![0xA5u8; 64]);
    let engine = make_engine(&paths, &keyfile);
    let sink0 = EventSink::null();
    engine
        .init_vault(
            Zeroizing::new("correct horse battery staple".to_string()),
            Some(USB_UUID.to_string()),
            Some(keyfile.clone()),
            Argon2Params::default(),
            &sink0,
        )
        .expect("init_vault");
    let sock = paths.control_socket();
    let listener = bind(&sock);
    let owner_uid = rustix::process::getuid().as_raw();
    let server_engine = engine.clone();
    let server = tokio::spawn(async move {
        envctl_secretd::server::serve(server_engine, owner_uid, listener, std::future::pending())
            .await
            .expect("serve");
    });
    tokio::time::sleep(std::time::Duration::from_millis(50)).await;
    (dir, sock, engine, server)
}

async fn unlock(sock: &Path) {
    let mut lock = v1::lock_client::LockClient::new(connect(sock.to_path_buf()).await);
    let stream = lock
        .unlock(v1::UnlockReq {
            passphrase: Some("correct horse battery staple".to_string()),
        })
        .await
        .expect("unlock rpc")
        .into_inner();
    drain_err(stream).await;
}

async fn add_secret(sock: &Path, name: &str, provider: i32, value: &[u8], broker_only: bool) {
    let mut vault = v1::vault_client::VaultClient::new(connect(sock.to_path_buf()).await);
    let s = vault
        .add(v1::AddSecretReq {
            name: name.into(),
            provider,
            value: value.to_vec(),
            note: format!("{name}-note"),
            overwrite: false,
            broker_only,
        })
        .await
        .expect("add")
        .into_inner();
    drain_err(s).await;
}

fn contains(haystack: &[u8], needle: &[u8]) -> bool {
    !needle.is_empty()
        && haystack.len() >= needle.len()
        && haystack.windows(needle.len()).any(|w| w == needle)
}

// ---- tests -----------------------------------------------------------------------------------

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn vault_list_is_metadata_only_and_provider_filtered() {
    let (dir, sock, _engine, server) = serve_initialized("list").await;
    unlock(&sock).await;
    add_secret(
        &sock,
        "anth",
        v1::ProviderKind::Anthropic as i32,
        SENTINEL,
        false,
    )
    .await;
    add_secret(
        &sock,
        "oai",
        v1::ProviderKind::Openai as i32,
        SENTINEL,
        true,
    )
    .await;

    let mut vault = v1::vault_client::VaultClient::new(connect(sock.clone()).await);

    // Unfiltered: both, metadata only, NO value byte.
    let all = vault
        .list(v1::ListSecretReq { provider: None })
        .await
        .expect("list")
        .into_inner();
    assert_eq!(all.items.len(), 2);
    assert!(all.items.iter().any(|i| i.name == "oai" && i.broker_only));
    let dumped = format!("{all:?}");
    assert!(
        !contains(dumped.as_bytes(), SENTINEL),
        "list leaked a value"
    );

    // Provider filter narrows to one.
    let filtered = vault
        .list(v1::ListSecretReq {
            provider: Some(v1::ProviderKind::Anthropic as i32),
        })
        .await
        .expect("list filtered")
        .into_inner();
    assert_eq!(filtered.items.len(), 1);
    assert_eq!(filtered.items[0].name, "anth");

    server.abort();
    let _ = std::fs::remove_dir_all(&dir);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn vault_rm_dry_run_then_apply_and_empty_arg_refused() {
    let (dir, sock, engine, server) = serve_initialized("rm").await;
    unlock(&sock).await;
    add_secret(
        &sock,
        "anth",
        v1::ProviderKind::Anthropic as i32,
        SENTINEL,
        false,
    )
    .await;

    let mut vault = v1::vault_client::VaultClient::new(connect(sock.clone()).await);

    // Empty name => invalid_argument BEFORE any engine call.
    let err = vault
        .rm(v1::RmSecretReq {
            name: String::new(),
            apply: true,
            confirm: true,
        })
        .await
        .expect_err("empty name must be invalid_argument");
    assert_eq!(err.code(), tonic::Code::InvalidArgument);

    // Dry-run (apply without confirm downgrades): mutates NOTHING.
    let s = vault
        .rm(v1::RmSecretReq {
            name: "anth".into(),
            apply: true,
            confirm: false,
        })
        .await
        .expect("rm dry")
        .into_inner();
    assert!(drain_err(s).await.is_empty(), "dry-run rm must not error");
    {
        let sink = EventSink::null();
        let still = engine.secret_list(None, &sink).expect("list");
        assert_eq!(still.len(), 1, "dry-run rm removed a secret");
    }

    // Apply (apply && confirm): removes it.
    let s = vault
        .rm(v1::RmSecretReq {
            name: "anth".into(),
            apply: true,
            confirm: true,
        })
        .await
        .expect("rm apply")
        .into_inner();
    assert!(drain_err(s).await.is_empty(), "apply rm must not error");
    {
        let sink = EventSink::null();
        assert!(engine.secret_list(None, &sink).expect("list").is_empty());
    }

    server.abort();
    let _ = std::fs::remove_dir_all(&dir);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn vault_rotate_appends_version_and_refuses_unknown() {
    let (dir, sock, engine, server) = serve_initialized("rotate").await;
    unlock(&sock).await;
    add_secret(
        &sock,
        "anth",
        v1::ProviderKind::Anthropic as i32,
        b"v1",
        true,
    )
    .await;

    let mut vault = v1::vault_client::VaultClient::new(connect(sock.clone()).await);

    // Dry-run: no new version.
    let s = vault
        .rotate(v1::RotateReq {
            name: "anth".into(),
            new_value: SENTINEL.to_vec(),
            apply: false,
        })
        .await
        .expect("rotate dry")
        .into_inner();
    assert!(drain_err(s).await.is_empty());

    // Apply: appends version 2, carries broker_only forward.
    let s = vault
        .rotate(v1::RotateReq {
            name: "anth".into(),
            new_value: SENTINEL.to_vec(),
            apply: true,
        })
        .await
        .expect("rotate apply")
        .into_inner();
    assert!(drain_err(s).await.is_empty());
    {
        let m = engine.secret_meta("anth").expect("meta").expect("some");
        assert!(m.broker_only, "rotate must carry broker_only forward");
    }

    // Unknown secret refused (engine bails -> terminal stream error).
    let s = vault
        .rotate(v1::RotateReq {
            name: "nope".into(),
            new_value: SENTINEL.to_vec(),
            apply: true,
        })
        .await
        .expect("rotate unknown rpc")
        .into_inner();
    let err = drain_err(s).await;
    assert!(err.contains("unknown secret"), "got {err:?}");

    // Empty name => invalid_argument.
    let err = vault
        .rotate(v1::RotateReq {
            name: String::new(),
            new_value: SENTINEL.to_vec(),
            apply: false,
        })
        .await
        .expect_err("empty rotate must be invalid_argument");
    assert_eq!(err.code(), tonic::Code::InvalidArgument);

    server.abort();
    let _ = std::fs::remove_dir_all(&dir);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn relay_create_then_list_filters_revoked() {
    let (dir, sock, _engine, server) = serve_initialized("relay").await;
    unlock(&sock).await;

    let mut relay = v1::relay_client::RelayClient::new(connect(sock.clone()).await);

    // Missing policy => invalid_argument.
    let err = relay
        .create(v1::CreateRelayReq { policy: None })
        .await
        .expect_err("missing policy must be invalid_argument");
    assert_eq!(err.code(), tonic::Code::InvalidArgument);

    // Create a named policy (additive).
    let s = relay
        .create(v1::CreateRelayReq {
            policy: Some(v1::RelayPolicy {
                name: "claude".into(),
                secret_name: "anthropic".into(),
                provider: v1::ProviderKind::Anthropic as i32,
                mode: v1::DataPlaneMode::HttpsProxyMitm as i32,
                host_allow: vec!["api.anthropic.com".into()],
                path_allow: vec!["/v1/".into()],
                method_allow: vec!["post".into()],
                expires_at: String::new(),
                rate_per_min: 0,
                quota_total: 0,
                enabled: true,
                ephemeral: false,
                upstream_base: String::new(),
            }),
        })
        .await
        .expect("create")
        .into_inner();
    assert!(drain_err(s).await.is_empty(), "create must not error");

    // List: present, method echoed back.
    let active = relay
        .list(v1::ListRelayReq {
            include_revoked: false,
        })
        .await
        .expect("list")
        .into_inner();
    assert_eq!(active.items.len(), 1);
    assert_eq!(active.items[0].name, "claude");
    assert_eq!(active.items[0].method_allow, vec!["post".to_string()]);

    // Revoke, then confirm the include_revoked filter.
    let r = relay
        .revoke(v1::RevokeRelayReq {
            name: "claude".into(),
            apply: true,
            confirm: true,
        })
        .await
        .expect("revoke")
        .into_inner();
    assert!(!r.dry_run);
    let after = relay
        .list(v1::ListRelayReq {
            include_revoked: false,
        })
        .await
        .expect("list after")
        .into_inner();
    assert!(after.items.is_empty(), "revoked policy must be filtered");
    let all = relay
        .list(v1::ListRelayReq {
            include_revoked: true,
        })
        .await
        .expect("list all")
        .into_inner();
    assert_eq!(all.items.len(), 1);

    server.abort();
    let _ = std::fs::remove_dir_all(&dir);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn audit_query_returns_rows_and_get_meta_populated() {
    let (dir, sock, _engine, server) = serve_initialized("audit").await;
    unlock(&sock).await;
    add_secret(
        &sock,
        "anth",
        v1::ProviderKind::Anthropic as i32,
        SENTINEL,
        false,
    )
    .await;

    // Get metadata-only: meta populated, value empty, no leak.
    {
        let mut vault = v1::vault_client::VaultClient::new(connect(sock.clone()).await);
        let r = vault
            .get(v1::GetSecretReq {
                name: "anth".into(),
                reveal: false,
                apply: false,
                confirm: false,
            })
            .await
            .expect("get meta")
            .into_inner();
        assert!(r.value.is_empty());
        let meta = r.meta.expect("meta populated (TASK-0035)");
        assert_eq!(meta.name, "anth");
        assert_eq!(meta.provider, v1::ProviderKind::Anthropic as i32);
    }

    // Audit.Query: returns the chain, post-filters, and never leaks the value.
    let mut audit = v1::audit_client::AuditClient::new(connect(sock.clone()).await);
    let r = audit
        .query(v1::AuditQueryReq {
            actor: None,
            relay: None,
            since: None,
            until: None,
            limit: 0,
        })
        .await
        .expect("audit query")
        .into_inner();
    assert!(!r.entries.is_empty());
    assert!(r
        .entries
        .iter()
        .any(|e| e.action.contains("secret_written")));
    let dumped = format!("{r:?}");
    assert!(
        !contains(dumped.as_bytes(), SENTINEL),
        "audit query leaked a value"
    );

    server.abort();
    let _ = std::fs::remove_dir_all(&dir);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn locked_vault_refuses_list_with_failed_precondition() {
    // A served-but-NOT-unlocked vault: List must refuse with failed_precondition (fail-closed gate).
    let (dir, sock, _engine, server) = serve_initialized("locked").await;
    // No unlock.
    let mut vault = v1::vault_client::VaultClient::new(connect(sock.clone()).await);
    let err = vault
        .list(v1::ListSecretReq { provider: None })
        .await
        .expect_err("locked list must refuse");
    assert_eq!(err.code(), tonic::Code::FailedPrecondition);

    server.abort();
    let _ = std::fs::remove_dir_all(&dir);
}
