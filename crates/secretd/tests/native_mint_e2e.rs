//! G2 daemon e2e: native GitHub App installation-token minting wired end-to-end through the REAL
//! `secretd` stack against a MOCK GitHub endpoint (no live GitHub App, no network egress).
//!
//! Drives the production modules: `envctl_secretd::server::serve` (the five services behind the real
//! `OwnerGuard`), the real `grpc::unlock` handler (which late-binds the `GitHubAppMint` from the
//! unlocked vault via `rebuild_github_provider` + `DaemonHttpTransport`), the real `grpc::mint`
//! handler (which routes `mode=native` through the engine's `resolve_injection`), and the real
//! `conv` conversions.
//!
//! Coverage (the U6 contract):
//!   (a) `ResolvedInjection.env[GITHUB_TOKEN]` == the MOCK-minted token (NOT the relay bearer);
//!   (b) `expires_at` surfaces GitHub's authoritative value; the bearer field is the relay bearer;
//!   (c) a 404/500 from GitHub ⇒ REFUSE (no injection in the response);
//!   (d) a locked vault / no credential ⇒ NoMint ⇒ fall back to the proxy-swap shape.
#![cfg(feature = "provider-github")]

use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use envctl_secrets::keyslot::{Argon2Params, ARGON2_M_KIB_FLOOR, ARGON2_T_COST_FLOOR};
use envctl_secrets::paths::Paths;
use envctl_secrets::seam::{NoMint, SystemClock, UpstreamError, UsbProbe};
use envctl_secrets::vault::{InMemStore, Store};
use envctl_secrets::{EgressReq, EgressResp, Engine, EventSink, SecretMeta, Upstream};
use envctl_secrets_proto::v1;
use hyper_util::rt::TokioIo;
use tonic::transport::{Endpoint, Uri};
use tonic::Streaming;
use zeroize::Zeroizing;

/// A throwaway 1024-bit RSA key (PKCS#1) — weak BY DESIGN, never a real credential.
const TEST_PEM: &str = "-----BEGIN RSA PRIVATE KEY-----\nMIICXgIBAAKBgQDw1EvUY2q80CzzraBZxIBLq1xjF9Eu5PsEseAd2bD+oJo4QQkI\npGycm26vJalBiW/rdzcSPaxPUT7KgH1IeftkUL0pbDG6nN08MgJM0/LjVKx3fK5A\n2Lq+CCh+eHfRGxcX8haBzWcwi4tfb90/7Vi9CGh7IXyyMTWLNW/mBVoH8wIDAQAB\nAoGBAMSPYbzdz9Z/ytCwm7noyhX4rRUr8U3nEoIIdDWo4e9RQc48NpVZLlS8ACDw\nCi81b6WtzcMTlzm9xBQfvyGSff0S/cCPAWEfGNItWOg5jeLSNftDVh4yM06BPEOI\nf+FwkGPiQYtCnhSXLhQq0ClODymjHyW+M7MBf8iyqnd8bnUhAkEA/q8Z5C7YQSFq\nIbywMegUkmCykiX8oCrvykg8i5oOjZXhIp/hnxv6jYynZd0PV1oOtbVTuvEve8kr\nCj+84GCPKQJBAPIS3i9C1VaaecCoSlnSY6FHWXmbLsm4wqXGbcyS0m4tQclIXfsd\nuDO4AUTu6Xc893Xfa3M/4Jpl7Fs5TReVbbsCQQCUFIlQVDBmxh/oV8Z2bgMwDMsn\nELEvC2f6zD9vx/Y4OnH5aM6NbX4juSlHn92go3s0CacSZdN+/LtqrR6Ls3jpAkBC\n/DOdUlokf9SHGkqQtmY5X7wDqYx153l9U/5YKJywPjfBEhRng57QOO+o+o+CHk2/\nwVZDav6k2uVfjOinSQM3AkEApokk6NycDKY657zkXPtlhKBsvyxfVW+evW9XjoHi\nEnHNytN8c6NOpZMjmzxgSUoOpAI4OVMIH00OvKHIIpvN0w==\n-----END RSA PRIVATE KEY-----";

const MINTED_TOKEN: &str = "ghs_e2e_minted_token";
const APP_SECRET: &str = "github_app";

// ---- fakes / helpers -------------------------------------------------------------------------

#[derive(Clone)]
struct NullUpstream;
#[async_trait::async_trait]
impl Upstream for NullUpstream {
    async fn send(
        &self,
        _req: EgressReq,
        _real_key: &Zeroizing<Vec<u8>>,
    ) -> Result<EgressResp, UpstreamError> {
        Err(UpstreamError::Io("upstream not wired in e2e".into()))
    }
}

struct NoUsb;
impl UsbProbe for NoUsb {
    fn keyfile_for(&self, _uuid: &str) -> Option<Zeroizing<Vec<u8>>> {
        None
    }
}

/// Cheap-but-valid Argon2 params (at the downgrade floor) so the test's init/unlock derivations are
/// fast — the default 1 GiB params make a multi-unlock e2e take minutes.
fn cheap_argon2() -> Argon2Params {
    Argon2Params {
        m_kib: ARGON2_M_KIB_FLOOR,
        t_cost: ARGON2_T_COST_FLOOR,
        p_lanes: 1,
    }
}

fn make_engine(paths: &Paths) -> Engine {
    Engine::with_seams(
        paths.clone(),
        Box::new(InMemStore::new()) as Box<dyn Store>,
        Box::new(SystemClock),
        Box::new(NoUsb),
        Box::new(NoMint),
        Box::new(NullUpstream),
        #[cfg(feature = "provider-github")]
        Box::new(envctl_secrets::mint_github::NoopHttpTransport),
        Box::new(envctl_secrets::broker::UnprovenGate),
        Box::new(envctl_secrets::SystemClockTrustedTime),
        envctl_secrets::Topology::OnBox,
    )
    .expect("with_seams")
}

fn temp_paths(tag: &str) -> (PathBuf, Paths) {
    use std::os::unix::fs::PermissionsExt;
    let dir = std::env::temp_dir().join(format!("envctl-native-e2e-{tag}-{}", std::process::id()));
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

async fn drain(mut s: Streaming<v1::Event>, buf: &Arc<Mutex<Vec<u8>>>) -> Vec<v1::Event> {
    let mut out = Vec::new();
    while let Some(ev) = s.message().await.expect("stream message") {
        buf.lock()
            .unwrap()
            .extend_from_slice(format!("{ev:?}").as_bytes());
        out.push(ev);
    }
    out
}

fn contains(haystack: &[u8], needle: &[u8]) -> bool {
    if needle.is_empty() || haystack.len() < needle.len() {
        return false;
    }
    haystack.windows(needle.len()).any(|w| w == needle)
}

/// These tests set process-global env vars (`ENVCTL_GITHUB_API_BASE` / `_APP_SECRET`) that the
/// daemon's unlock rebuild reads, so they must NOT run concurrently even under the default
/// multi-thread test harness. A `tokio::sync::Mutex` serializes them — async-aware so the guard may
/// be held across `.await` (a std Mutex guard cannot). Held for the body of each test.
static SERIAL: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());
async fn serial_guard() -> tokio::sync::MutexGuard<'static, ()> {
    SERIAL.lock().await
}

/// A one-shot mock GitHub endpoint. Accepts ONE connection, reads the request, replies `(status,
/// body)`. Returns its `http://127.0.0.1:<port>` base (plain HTTP — reqwest speaks it; the daemon
/// points its `api_base` here via `ENVCTL_GITHUB_API_BASE`).
fn spawn_mock_github(status: u16, body: &'static str) -> (String, std::thread::JoinHandle<()>) {
    use std::io::{Read, Write};
    use std::net::TcpListener;
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = listener.local_addr().unwrap();
    let base = format!("http://{addr}");
    let handle = std::thread::spawn(move || {
        if let Ok((mut sock, _)) = listener.accept() {
            let mut acc: Vec<u8> = Vec::new();
            let mut chunk = [0u8; 1024];
            loop {
                match sock.read(&mut chunk) {
                    Ok(0) | Err(_) => break,
                    Ok(n) => {
                        acc.extend_from_slice(&chunk[..n]);
                        if acc.windows(4).any(|w| w == b"\r\n\r\n") {
                            break;
                        }
                    }
                }
            }
            let reason = if status == 201 { "Created" } else { "Error" };
            let resp = format!(
                "HTTP/1.1 {status} {reason}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                body.len()
            );
            let _ = sock.write_all(resp.as_bytes());
            let _ = sock.flush();
        }
    });
    (base, handle)
}

/// Init the vault (passphrase only), unlock in-process, seed the App PEM (broker_only) + app/install
/// meta, then lock again — so the App credential is enrolled BEFORE the daemon's unlock RPC rebuilds
/// the native minter from it.
fn seed_app_credential(engine: &Engine) {
    let sink = EventSink::null();
    engine
        .init_vault(
            Zeroizing::new("correct horse battery staple".to_string()),
            None,
            None,
            cheap_argon2(),
            &sink,
        )
        .expect("init_vault");
    engine
        .unlock(
            envctl_secrets::Unlock::Passphrase(Zeroizing::new(
                "correct horse battery staple".to_string(),
            )),
            &sink,
        )
        .expect("unlock");
    engine
        .secret_put(
            SecretMeta {
                name: APP_SECRET.to_string(),
                provider: envctl_secrets::broker::Provider::Github,
                note: "e2e app key".to_string(),
                broker_only: true,
            },
            Zeroizing::new(TEST_PEM.as_bytes().to_vec()),
            &sink,
        )
        .expect("secret_put app pem");
    engine
        .put_app_credential_meta(APP_SECRET, "42", 99)
        .expect("put app meta");
    engine.lock(&sink).expect("lock");
}

fn serve(engine: Engine, sock: std::path::PathBuf) {
    let listener = bind(&sock);
    let owner_uid = rustix::process::getuid().as_raw();
    tokio::spawn(async move {
        envctl_secretd::server::serve(engine, owner_uid, listener, std::future::pending())
            .await
            .expect("serve");
    });
}

async fn unlock_over_wire(sock: &std::path::Path, wire: &Arc<Mutex<Vec<u8>>>) {
    let mut lock = v1::lock_client::LockClient::new(connect(sock.to_path_buf()).await);
    let stream = lock
        .unlock(v1::UnlockReq {
            passphrase: Some("correct horse battery staple".to_string()),
        })
        .await
        .expect("unlock rpc")
        .into_inner();
    let evs = drain(stream, wire).await;
    assert!(
        evs.iter()
            .any(|e| matches!(&e.kind, Some(v1::event::Kind::VaultUnlocked(_)))),
        "unlock must emit VaultUnlocked"
    );
}

/// Mint over the REAL daemon with `mode=native, provider=github`.
async fn native_mint(sock: &std::path::Path) -> v1::MintResp {
    let mut relay = v1::relay_client::RelayClient::new(connect(sock.to_path_buf()).await);
    relay
        .mint(v1::MintReq {
            relay: APP_SECRET.to_string(),
            ephemeral: true,
            provider: v1::ProviderKind::Github as i32,
            ttl_secs: 3600,
            client_pid: 0,
            mode: v1::DataPlaneMode::NativeSubtoken as i32,
            repos: vec!["meta".to_string()],
            perms: vec!["checks:write".to_string()],
        })
        .await
        .expect("relay.mint")
        .into_inner()
}

// ============================================================================================

/// (a) + (b): a 201 from the mock ⇒ the minted token is injected as GITHUB_TOKEN (NOT the relay
/// bearer), GitHub's `expires_at` is surfaced, and the minted token never appears in the event-stream
/// wire (the injection is the owner-only delivery channel).
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn native_mint_injects_minted_token_and_event_never_leaks_it() {
    let _g = serial_guard().await;
    let body = r#"{"token":"ghs_e2e_minted_token","expires_at":"2026-06-12T23:00:00Z","permissions":{"checks":"write"}}"#;
    let (base, mock) = spawn_mock_github(201, body);
    std::env::set_var("ENVCTL_GITHUB_API_BASE", &base);
    std::env::set_var("ENVCTL_GITHUB_APP_SECRET", APP_SECRET);

    let event_wire: Arc<Mutex<Vec<u8>>> = Arc::new(Mutex::new(Vec::new()));
    let (_dir, paths) = temp_paths("ok");
    let engine = make_engine(&paths);
    seed_app_credential(&engine);

    let sock = paths.control_socket();
    serve(engine.clone(), sock.clone());
    tokio::time::sleep(std::time::Duration::from_millis(50)).await;

    // Unlock over the wire — triggers the daemon's rebuild, installing the GitHubAppMint at the mock.
    unlock_over_wire(&sock, &event_wire).await;

    let resp = native_mint(&sock).await;
    let _ = mock.join();

    let injection = resp.injection.expect("native mint must carry an injection");
    assert_eq!(
        injection.mode,
        v1::DataPlaneMode::NativeSubtoken as i32,
        "the resolved injection is the native plane"
    );
    // (a) the MINTED token is injected, NOT the relay bearer.
    assert_eq!(
        injection.env.get("GITHUB_TOKEN").map(String::as_str),
        Some(MINTED_TOKEN)
    );
    assert_eq!(
        injection.env.get("GH_TOKEN").map(String::as_str),
        Some(MINTED_TOKEN)
    );
    assert_ne!(
        injection.env.get("GITHUB_TOKEN").map(String::as_str),
        Some(resp.bearer.as_str()),
        "the relay bearer must NOT be the injected token"
    );
    // (b) the bearer field is the relay bearer (never the minted token); expires_at is present.
    assert_ne!(
        resp.bearer, MINTED_TOKEN,
        "bearer field is the relay bearer"
    );
    assert!(!resp.expires_at.is_empty(), "expires_at present");

    // The minted token NEVER appears in any byte the client received from the EVENT stream (the
    // RelayMinted event carries only relay + expires_at).
    let ew = event_wire.lock().unwrap();
    assert!(
        !contains(&ew, MINTED_TOKEN.as_bytes()),
        "minted token must never cross the event-stream wire"
    );
    drop(ew);

    std::env::remove_var("ENVCTL_GITHUB_API_BASE");
    std::env::remove_var("ENVCTL_GITHUB_APP_SECRET");
}

/// (c): a 404 from the mock ⇒ the engine REFUSES the native mint (durable Refused row) ⇒ the
/// response carries NO injection (the client refuses to spawn; no token emitted).
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn native_mint_http_error_refuses_with_no_injection() {
    let _g = serial_guard().await;
    let (base, mock) = spawn_mock_github(404, r#"{"message":"Not Found"}"#);
    std::env::set_var("ENVCTL_GITHUB_API_BASE", &base);
    std::env::set_var("ENVCTL_GITHUB_APP_SECRET", APP_SECRET);

    let event_wire: Arc<Mutex<Vec<u8>>> = Arc::new(Mutex::new(Vec::new()));
    let (_dir, paths) = temp_paths("refuse");
    let engine = make_engine(&paths);
    seed_app_credential(&engine);

    let sock = paths.control_socket();
    serve(engine.clone(), sock.clone());
    tokio::time::sleep(std::time::Duration::from_millis(50)).await;
    unlock_over_wire(&sock, &event_wire).await;

    let resp = native_mint(&sock).await;
    let _ = mock.join();
    assert!(
        resp.injection.is_none(),
        "a GitHub HTTP error must REFUSE the mint (no injection emitted)"
    );

    std::env::remove_var("ENVCTL_GITHUB_API_BASE");
    std::env::remove_var("ENVCTL_GITHUB_APP_SECRET");
}

/// (d): no App credential enrolled ⇒ the unlock rebuild keeps NoMint ⇒ a native mint falls back to
/// the proxy-swap shape (the relay bearer is injected), never refusing.
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn native_mint_without_credential_falls_back_to_proxy_swap() {
    let _g = serial_guard().await;
    std::env::set_var("ENVCTL_GITHUB_APP_SECRET", "github_app_absent");
    std::env::remove_var("ENVCTL_GITHUB_API_BASE");

    let event_wire: Arc<Mutex<Vec<u8>>> = Arc::new(Mutex::new(Vec::new()));
    let (_dir, paths) = temp_paths("fallback");
    let engine = make_engine(&paths);
    // Init + unlock + lock WITHOUT seeding any github_app secret (so the rebuild finds nothing).
    {
        let sink = EventSink::null();
        engine
            .init_vault(
                Zeroizing::new("correct horse battery staple".to_string()),
                None,
                None,
                cheap_argon2(),
                &sink,
            )
            .expect("init_vault");
        engine
            .unlock(
                envctl_secrets::Unlock::Passphrase(Zeroizing::new(
                    "correct horse battery staple".to_string(),
                )),
                &sink,
            )
            .expect("unlock");
        engine.lock(&sink).expect("lock");
    }

    let sock = paths.control_socket();
    serve(engine.clone(), sock.clone());
    tokio::time::sleep(std::time::Duration::from_millis(50)).await;
    unlock_over_wire(&sock, &event_wire).await;

    let resp = native_mint(&sock).await;
    let injection = resp
        .injection
        .expect("fallback still produces a proxy-swap injection");
    // NoMint ⇒ Unsupported ⇒ proxy-swap fallback (HTTPS_PROXY_MITM); the relay bearer is injected.
    assert_eq!(
        injection.mode,
        v1::DataPlaneMode::HttpsProxyMitm as i32,
        "no credential ⇒ proxy-swap fallback"
    );
    assert!(
        !contains(resp.bearer.as_bytes(), MINTED_TOKEN.as_bytes()),
        "no minted token exists on the fallback path"
    );

    std::env::remove_var("ENVCTL_GITHUB_APP_SECRET");
}

// ============================================================================================
// TASK-0020: the FROZEN `Vault.MintGithub` per-call surface, end-to-end through `secretd`.
// ============================================================================================

const MINT_GITHUB_KEY: &str = "github-app-private-key";
const FROZEN_TOKEN: &str = "ghs_frozen_e2e_token";

/// Build an engine whose `github_transport` is the REAL `DaemonHttpTransport` (so the per-call mint
/// actually reaches the mock). Must run inside the tokio runtime (`Handle::current()`).
fn make_engine_with_daemon_transport(paths: &Paths) -> Engine {
    Engine::with_seams(
        paths.clone(),
        Box::new(InMemStore::new()) as Box<dyn Store>,
        Box::new(SystemClock),
        Box::new(NoUsb),
        Box::new(NoMint),
        Box::new(NullUpstream),
        Box::new(envctl_secretd::transport::DaemonHttpTransport::new()),
        Box::new(envctl_secrets::broker::UnprovenGate),
        Box::new(envctl_secrets::SystemClockTrustedTime),
        envctl_secrets::Topology::OnBox,
    )
    .expect("with_seams")
}

/// Init + unlock + seed the FLAT-convention App key (`github-app-private-key` broker_only) + id
/// (`github-app-id` meta), then lock. The per-call `mint_github_token` opens these post-unlock.
fn seed_flat_app_credential(engine: &Engine) {
    let sink = EventSink::null();
    engine
        .init_vault(
            Zeroizing::new("correct horse battery staple".to_string()),
            None,
            None,
            cheap_argon2(),
            &sink,
        )
        .expect("init_vault");
    engine
        .unlock(
            envctl_secrets::Unlock::Passphrase(Zeroizing::new(
                "correct horse battery staple".to_string(),
            )),
            &sink,
        )
        .expect("unlock");
    engine
        .secret_put(
            SecretMeta {
                name: MINT_GITHUB_KEY.to_string(),
                provider: envctl_secrets::broker::Provider::Github,
                note: "e2e flat app key".to_string(),
                broker_only: true,
            },
            Zeroizing::new(TEST_PEM.as_bytes().to_vec()),
            &sink,
        )
        .expect("secret_put flat app pem");
    // The App id is a non-secret plaintext meta value (integrity-covered by the header MAC).
    engine
        .put_github_app_id("4044997")
        .expect("put github app id");
    engine.lock(&sink).expect("lock");
}

async fn mint_github_over_wire(
    sock: &std::path::Path,
    repository_ids: Vec<&str>,
    permissions: Vec<&str>,
) -> Result<v1::MintGithubResp, tonic::Status> {
    let mut c = v1::vault_client::VaultClient::new(connect(sock.to_path_buf()).await);
    c.mint_github(v1::MintGithubReq {
        installation_id: 12345,
        repository_ids: repository_ids.into_iter().map(str::to_string).collect(),
        permissions: permissions.into_iter().map(str::to_string).collect(),
        ttl_secs: 3600,
    })
    .await
    .map(|r| r.into_inner())
}

/// Happy path: the daemon mints over the per-call path and returns the FROZEN `{token,
/// expires_at_unix}`. The token is the mock-minted token; expires_at_unix is GitHub's epoch.
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn mint_github_returns_frozen_two_field_response() {
    let _g = serial_guard().await;
    let body = r#"{"token":"ghs_frozen_e2e_token","expires_at":"2026-06-12T23:00:00Z"}"#;
    let (base, mock) = spawn_mock_github(201, body);
    std::env::set_var("ENVCTL_GITHUB_API_BASE", &base);

    let event_wire: Arc<Mutex<Vec<u8>>> = Arc::new(Mutex::new(Vec::new()));
    let (_dir, paths) = temp_paths("mintgh-ok");
    let engine = make_engine_with_daemon_transport(&paths);
    seed_flat_app_credential(&engine);

    let sock = paths.control_socket();
    serve(engine.clone(), sock.clone());
    tokio::time::sleep(std::time::Duration::from_millis(50)).await;
    unlock_over_wire(&sock, &event_wire).await;

    let resp = mint_github_over_wire(&sock, vec!["10", "4044997"], vec!["checks:write"])
        .await
        .expect("mint_github ok");
    let _ = mock.join();

    assert_eq!(
        resp.token, FROZEN_TOKEN,
        "the mock-minted token is returned"
    );
    let expected = chrono::DateTime::parse_from_rfc3339("2026-06-12T23:00:00Z")
        .unwrap()
        .timestamp();
    assert_eq!(
        resp.expires_at_unix, expected,
        "expires_at_unix is GitHub's authoritative epoch (i64)"
    );
    assert!(resp.expires_at_unix > 0, "positive epoch");

    // The minted token must NEVER appear in the unlock event-stream wire (metadata-only events).
    let ew = event_wire.lock().unwrap();
    assert!(
        !contains(&ew, FROZEN_TOKEN.as_bytes()),
        "minted token must never cross the event-stream wire"
    );
    drop(ew);

    std::env::remove_var("ENVCTL_GITHUB_API_BASE");
}

/// A non-numeric `repository_ids` entry is rejected at the daemon boundary (invalid_argument) —
/// never forwarded to GitHub.
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn mint_github_rejects_non_numeric_repository_id() {
    let _g = serial_guard().await;
    std::env::remove_var("ENVCTL_GITHUB_API_BASE");
    let event_wire: Arc<Mutex<Vec<u8>>> = Arc::new(Mutex::new(Vec::new()));
    let (_dir, paths) = temp_paths("mintgh-badid");
    let engine = make_engine_with_daemon_transport(&paths);
    seed_flat_app_credential(&engine);

    let sock = paths.control_socket();
    serve(engine.clone(), sock.clone());
    tokio::time::sleep(std::time::Duration::from_millis(50)).await;
    unlock_over_wire(&sock, &event_wire).await;

    let err = mint_github_over_wire(&sock, vec!["not-a-number"], vec![])
        .await
        .expect_err("non-numeric repository id must be rejected");
    assert_eq!(
        err.code(),
        tonic::Code::InvalidArgument,
        "non-numeric repository_ids ⇒ invalid_argument, got: {err:?}"
    );
}

/// A locked vault ⇒ the mint fails closed with `failed_precondition` (no key ⇒ no token).
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn mint_github_locked_vault_fails_precondition() {
    let _g = serial_guard().await;
    std::env::remove_var("ENVCTL_GITHUB_API_BASE");
    let (_dir, paths) = temp_paths("mintgh-locked");
    let engine = make_engine_with_daemon_transport(&paths);
    seed_flat_app_credential(&engine); // leaves the vault LOCKED

    let sock = paths.control_socket();
    serve(engine.clone(), sock.clone());
    tokio::time::sleep(std::time::Duration::from_millis(50)).await;
    // Deliberately do NOT unlock — the vault stays locked.

    let err = mint_github_over_wire(&sock, vec![], vec![])
        .await
        .expect_err("locked vault must refuse the mint");
    assert_eq!(
        err.code(),
        tonic::Code::FailedPrecondition,
        "locked vault ⇒ failed_precondition, got: {err:?}"
    );
}

// ============================================================================================
// TASK-0027: `secretctl github-app revoke-token` — the `Vault.RevokeGithubToken` RPC drives a
// `DELETE /installation/token` over the REAL DaemonHttpTransport against the mock. The revoke
// authenticates with the token ITSELF (no App credential needed) — only an unlocked vault.
// ============================================================================================

const REVOKE_E2E_TOKEN: &[u8] = b"ghs_e2e_revoke_me";

/// Drive `Vault.RevokeGithubToken` over the wire.
async fn revoke_github_over_wire(
    sock: &std::path::Path,
    token: &[u8],
    apply: bool,
) -> Result<v1::RevokeResp, tonic::Status> {
    let mut c = v1::vault_client::VaultClient::new(connect(sock.to_path_buf()).await);
    c.revoke_github_token(v1::RevokeGithubTokenReq {
        token: token.to_vec(),
        apply,
        installation_id: 0,
    })
    .await
    .map(|r| r.into_inner())
}

/// Init the vault (passphrase only) + leave it UNLOCKED in-process — the revoke RPC requires only an
/// unlocked vault (no App credential), and the explicit-token verb supplies its own bearer.
fn init_and_unlock_only(engine: &Engine) {
    let sink = EventSink::null();
    engine
        .init_vault(
            Zeroizing::new("correct horse battery staple".to_string()),
            None,
            None,
            cheap_argon2(),
            &sink,
        )
        .expect("init_vault");
    engine
        .unlock(
            envctl_secrets::Unlock::Passphrase(Zeroizing::new(
                "correct horse battery staple".to_string(),
            )),
            &sink,
        )
        .expect("unlock");
}

/// Apply path: the daemon fires the DELETE against the 204 mock ⇒ `{count_revoked:1, dry_run:false}`.
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn revoke_github_token_over_wire_204_succeeds() {
    let _g = serial_guard().await;
    let (base, mock) = spawn_mock_github(204, "");
    std::env::set_var("ENVCTL_GITHUB_API_BASE", &base);

    let event_wire: Arc<Mutex<Vec<u8>>> = Arc::new(Mutex::new(Vec::new()));
    let (_dir, paths) = temp_paths("revokegh-ok");
    let engine = make_engine_with_daemon_transport(&paths);
    init_and_unlock_only(&engine);

    let sock = paths.control_socket();
    serve(engine.clone(), sock.clone());
    tokio::time::sleep(std::time::Duration::from_millis(50)).await;

    let resp = revoke_github_over_wire(&sock, REVOKE_E2E_TOKEN, true)
        .await
        .expect("revoke ok");
    let _ = mock.join();
    assert_eq!(resp.count_revoked, 1, "204 ⇒ one token revoked");
    assert!(!resp.dry_run, "apply=true ⇒ not a dry-run");

    // The token must NEVER appear on the (unlock) event-stream wire.
    let ew = event_wire.lock().unwrap();
    assert!(
        !contains(&ew, REVOKE_E2E_TOKEN),
        "token must never cross the event-stream wire"
    );
    drop(ew);

    std::env::remove_var("ENVCTL_GITHUB_API_BASE");
}

/// Dry-run (apply=false) contacts NOTHING — no DELETE on the wire — and reports `dry_run:true`. The
/// mock is NOT spawned, so any egress attempt would hang/fail the test; it returns quickly instead.
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn revoke_github_token_dry_run_contacts_nothing() {
    let _g = serial_guard().await;
    // Point at an UNROUTED base: if the daemon egressed, the call would error — it must not egress.
    std::env::set_var("ENVCTL_GITHUB_API_BASE", "http://127.0.0.1:1");

    let event_wire: Arc<Mutex<Vec<u8>>> = Arc::new(Mutex::new(Vec::new()));
    let (_dir, paths) = temp_paths("revokegh-dry");
    let engine = make_engine_with_daemon_transport(&paths);
    init_and_unlock_only(&engine);

    let sock = paths.control_socket();
    serve(engine.clone(), sock.clone());
    tokio::time::sleep(std::time::Duration::from_millis(50)).await;
    let _ = &event_wire;

    let resp = revoke_github_over_wire(&sock, REVOKE_E2E_TOKEN, false)
        .await
        .expect("dry-run ok");
    assert_eq!(resp.count_revoked, 0, "dry-run reports no revoke");
    assert!(resp.dry_run, "apply=false ⇒ dry_run");

    std::env::remove_var("ENVCTL_GITHUB_API_BASE");
}

/// A locked vault ⇒ the revoke fails closed with `failed_precondition` (mint's auth floor).
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn revoke_github_token_locked_vault_fails_precondition() {
    let _g = serial_guard().await;
    std::env::remove_var("ENVCTL_GITHUB_API_BASE");
    let (_dir, paths) = temp_paths("revokegh-locked");
    let engine = make_engine_with_daemon_transport(&paths);
    // Init the vault but DO NOT unlock — it stays locked.
    {
        let sink = EventSink::null();
        engine
            .init_vault(
                Zeroizing::new("correct horse battery staple".to_string()),
                None,
                None,
                cheap_argon2(),
                &sink,
            )
            .expect("init_vault");
    }

    let sock = paths.control_socket();
    serve(engine.clone(), sock.clone());
    tokio::time::sleep(std::time::Duration::from_millis(50)).await;

    let err = revoke_github_over_wire(&sock, REVOKE_E2E_TOKEN, true)
        .await
        .expect_err("locked vault must refuse the revoke");
    assert_eq!(
        err.code(),
        tonic::Code::FailedPrecondition,
        "locked vault ⇒ failed_precondition, got: {err:?}"
    );
}

// ============================================================================================
// TASK-0026: `secretctl github-app enroll` — the enroll RPC pair (Vault.Add broker_only +
// Vault.SetGithubAppId) writes EXACTLY what the TASK-0020 per-call mint reads. The round-trip
// (enroll over the wire → MintGithub succeeds against the mock) is the load-bearing gate proving
// no name-drift between the writer and the reader.
// ============================================================================================

const ENROLL_TOKEN: &str = "ghs_enroll_roundtrip_token";

/// Init the vault (passphrase only) + leave it UNLOCKED in-process — the enroll RPCs require an
/// unlocked vault, and enrolling over the wire (not via the engine seed helper) is the whole point.
fn init_and_unlock(engine: &Engine) {
    let sink = EventSink::null();
    engine
        .init_vault(
            Zeroizing::new("correct horse battery staple".to_string()),
            None,
            None,
            cheap_argon2(),
            &sink,
        )
        .expect("init_vault");
    engine
        .unlock(
            envctl_secrets::Unlock::Passphrase(Zeroizing::new(
                "correct horse battery staple".to_string(),
            )),
            &sink,
        )
        .expect("unlock");
}

/// Enroll the App credential ENTIRELY OVER THE WIRE, exactly as `secretctl github-app enroll --apply`
/// does: `Vault.Add{ broker_only=true }` (the PEM, under the engine's `GITHUB_APP_KEY_NAME`) THEN
/// `Vault.SetGithubAppId{ apply=true }` (the App id). Returns the drained event count so the caller
/// can assert the writes happened.
async fn enroll_over_wire(sock: &std::path::Path, app_id: &str, wire: &Arc<Mutex<Vec<u8>>>) {
    let mut c = v1::vault_client::VaultClient::new(connect(sock.to_path_buf()).await);
    // 1. Seal the PEM broker-only under the mint reader's name (verbatim engine const).
    let add = c
        .add(v1::AddSecretReq {
            name: envctl_secrets::GITHUB_APP_KEY_NAME.to_string(),
            provider: v1::ProviderKind::Github as i32,
            value: TEST_PEM.as_bytes().to_vec(),
            note: "e2e enroll pem".to_string(),
            overwrite: false,
            broker_only: true,
        })
        .await
        .expect("vault.add app pem")
        .into_inner();
    let _ = drain(add, wire).await;
    // 2. Persist the non-secret App id.
    let set = c
        .set_github_app_id(v1::SetGithubAppIdReq {
            app_id: app_id.to_string(),
            apply: true,
        })
        .await
        .expect("vault.set_github_app_id")
        .into_inner();
    let evs = drain(set, wire).await;
    assert!(
        evs.iter().any(|e| matches!(
            &e.kind, Some(v1::event::Kind::Log(l)) if l.line.contains("enrolled GitHub App id")
        )),
        "applied SetGithubAppId must emit a confirming Log, got {evs:?}"
    );
}

/// (ROUND-TRIP, load-bearing): init+unlock → enroll over the wire (Add broker_only PEM +
/// SetGithubAppId "4044997") → `Vault.MintGithub` against the mock SUCCEEDS, reading EXACTLY what
/// enroll wrote. Proves the enroll writer and the TASK-0020 mint reader share the same names.
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn enroll_then_mint_github_round_trips() {
    let _g = serial_guard().await;
    let body = r#"{"token":"ghs_enroll_roundtrip_token","expires_at":"2026-06-12T23:00:00Z"}"#;
    let (base, mock) = spawn_mock_github(201, body);
    std::env::set_var("ENVCTL_GITHUB_API_BASE", &base);

    let event_wire: Arc<Mutex<Vec<u8>>> = Arc::new(Mutex::new(Vec::new()));
    let (_dir, paths) = temp_paths("enroll-roundtrip");
    let engine = make_engine_with_daemon_transport(&paths);
    init_and_unlock(&engine); // vault is UNLOCKED; nothing enrolled yet

    let sock = paths.control_socket();
    serve(engine.clone(), sock.clone());
    tokio::time::sleep(std::time::Duration::from_millis(50)).await;

    // Enroll the credential over the wire (no engine-side seed), then mint reads it back.
    enroll_over_wire(&sock, "4044997", &event_wire).await;

    let resp = mint_github_over_wire(&sock, vec!["10"], vec!["checks:write"])
        .await
        .expect("mint_github after wire-enroll must succeed");
    let _ = mock.join();
    assert_eq!(
        resp.token, ENROLL_TOKEN,
        "the mint read EXACTLY the enrolled credential and minted against the mock"
    );
    assert!(resp.expires_at_unix > 0, "positive epoch");

    std::env::remove_var("ENVCTL_GITHUB_API_BASE");
}

/// (BROKER-ONLY REFUSAL): after enroll, `Vault.Get{reveal,apply,confirm}` on the App PEM is REFUSED
/// (`permission_denied`, empty value) — the PEM was sealed `broker_only=true`, so the operator
/// surface can never read it out; only the internal mint path opens it.
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn enrolled_pem_is_broker_only_and_reveal_is_refused() {
    let _g = serial_guard().await;
    std::env::remove_var("ENVCTL_GITHUB_API_BASE");
    let event_wire: Arc<Mutex<Vec<u8>>> = Arc::new(Mutex::new(Vec::new()));
    let (_dir, paths) = temp_paths("enroll-brokeronly");
    let engine = make_engine_with_daemon_transport(&paths);
    init_and_unlock(&engine);

    let sock = paths.control_socket();
    serve(engine.clone(), sock.clone());
    tokio::time::sleep(std::time::Duration::from_millis(50)).await;
    enroll_over_wire(&sock, "4044997", &event_wire).await;

    let mut c = v1::vault_client::VaultClient::new(connect(sock.clone()).await);
    let err = c
        .get(v1::GetSecretReq {
            name: envctl_secrets::GITHUB_APP_KEY_NAME.to_string(),
            reveal: true,
            apply: true,
            confirm: true,
        })
        .await
        .expect_err("a broker_only reveal must be refused");
    assert_eq!(
        err.code(),
        tonic::Code::PermissionDenied,
        "broker_only reveal ⇒ permission_denied, got: {err:?}"
    );
    // The PEM bytes must NEVER appear on the wire for a refused reveal.
    assert!(
        !contains(err.message().as_bytes(), b"BEGIN RSA PRIVATE KEY"),
        "the refused reveal must not echo the PEM"
    );
}

/// (NEGATIVE): an empty `app_id` is rejected at the boundary (`invalid_argument`) and writes nothing.
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn set_github_app_id_empty_is_invalid_argument() {
    let _g = serial_guard().await;
    std::env::remove_var("ENVCTL_GITHUB_API_BASE");
    let (_dir, paths) = temp_paths("setid-empty");
    let engine = make_engine_with_daemon_transport(&paths);
    init_and_unlock(&engine);

    let sock = paths.control_socket();
    serve(engine.clone(), sock.clone());
    tokio::time::sleep(std::time::Duration::from_millis(50)).await;

    let mut c = v1::vault_client::VaultClient::new(connect(sock.clone()).await);
    let err = c
        .set_github_app_id(v1::SetGithubAppIdReq {
            app_id: "   ".to_string(), // whitespace-only ⇒ empty after trim
            apply: true,
        })
        .await
        .expect_err("empty app_id must be rejected");
    assert_eq!(
        err.code(),
        tonic::Code::InvalidArgument,
        "empty app_id ⇒ invalid_argument, got: {err:?}"
    );
}

/// (NEGATIVE): a DRY-RUN `SetGithubAppId{apply=false}` emits a preview Log and mutates NOTHING — a
/// subsequent MintGithub still fails closed (no App id enrolled ⇒ permission_denied).
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn set_github_app_id_dry_run_mutates_nothing() {
    let _g = serial_guard().await;
    std::env::remove_var("ENVCTL_GITHUB_API_BASE");
    let event_wire: Arc<Mutex<Vec<u8>>> = Arc::new(Mutex::new(Vec::new()));
    let (_dir, paths) = temp_paths("setid-dryrun");
    let engine = make_engine_with_daemon_transport(&paths);
    init_and_unlock(&engine);

    let sock = paths.control_socket();
    serve(engine.clone(), sock.clone());
    tokio::time::sleep(std::time::Duration::from_millis(50)).await;

    // Seal the PEM so the ONLY missing piece is the App id (isolates that the dry-run didn't write it).
    {
        let mut c = v1::vault_client::VaultClient::new(connect(sock.clone()).await);
        let add = c
            .add(v1::AddSecretReq {
                name: envctl_secrets::GITHUB_APP_KEY_NAME.to_string(),
                provider: v1::ProviderKind::Github as i32,
                value: TEST_PEM.as_bytes().to_vec(),
                note: "dry-run pem".to_string(),
                overwrite: false,
                broker_only: true,
            })
            .await
            .expect("vault.add")
            .into_inner();
        let _ = drain(add, &event_wire).await;
    }

    // DRY-RUN the id enrollment: a preview Log, no write.
    {
        let mut c = v1::vault_client::VaultClient::new(connect(sock.clone()).await);
        let evs = drain(
            c.set_github_app_id(v1::SetGithubAppIdReq {
                app_id: "4044997".to_string(),
                apply: false,
            })
            .await
            .expect("dry-run set rpc")
            .into_inner(),
            &event_wire,
        )
        .await;
        assert!(
            evs.iter().any(|e| matches!(
                &e.kind, Some(v1::event::Kind::Log(l)) if l.line.contains("DRY-RUN")
            )),
            "dry-run SetGithubAppId must emit a DRY-RUN preview Log, got {evs:?}"
        );
    }

    // PROOF the id was not written: the per-call mint fails closed (App id not enrolled).
    let err = mint_github_over_wire(&sock, vec![], vec![])
        .await
        .expect_err("mint must fail: dry-run wrote no App id");
    assert_eq!(
        err.code(),
        tonic::Code::PermissionDenied,
        "no App id ⇒ permission_denied (the mint's absent-id remediation), got: {err:?}"
    );
}

/// (NEGATIVE): an `apply=true` SetGithubAppId on a LOCKED vault fails closed with
/// `failed_precondition` and writes nothing (the engine refuses without a DEK).
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn set_github_app_id_locked_vault_fails_precondition() {
    let _g = serial_guard().await;
    std::env::remove_var("ENVCTL_GITHUB_API_BASE");
    let (_dir, paths) = temp_paths("setid-locked");
    let engine = make_engine_with_daemon_transport(&paths);
    // Init but DO NOT unlock — the vault stays locked.
    {
        let sink = EventSink::null();
        engine
            .init_vault(
                Zeroizing::new("correct horse battery staple".to_string()),
                None,
                None,
                cheap_argon2(),
                &sink,
            )
            .expect("init_vault");
    }

    let sock = paths.control_socket();
    serve(engine.clone(), sock.clone());
    tokio::time::sleep(std::time::Duration::from_millis(50)).await;

    let mut c = v1::vault_client::VaultClient::new(connect(sock.clone()).await);
    let stream = c
        .set_github_app_id(v1::SetGithubAppIdReq {
            app_id: "4044997".to_string(),
            apply: true,
        })
        .await;
    // The error may surface at the call (unary-style status) — assert failed_precondition.
    let err = stream.expect_err("locked vault must refuse the id write");
    assert_eq!(
        err.code(),
        tonic::Code::FailedPrecondition,
        "locked vault ⇒ failed_precondition, got: {err:?}"
    );
}
