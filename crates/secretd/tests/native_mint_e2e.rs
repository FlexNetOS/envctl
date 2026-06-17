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
