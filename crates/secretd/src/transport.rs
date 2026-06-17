//! `DaemonHttpTransport` — the daemon's realization of the engine's
//! [`HttpTransport`](envctl_secrets::mint_github::HttpTransport) seam for native GitHub App
//! installation-token minting (G2).
//!
//! ## Why it lives here (and not in the engine)
//! The engine LIB is pure-Rust, non-printing, and pushes all network I/O to a `Send + Sync` seam —
//! exactly as the egress path does via [`crate::proxy::DaemonUpstream`]. The mint path's
//! request/response HTTP call is the [`HttpTransport`] trait; the daemon supplies the real
//! reqwest/rustls-on-ring impl that pins the FROZEN webpki roots (FS-S7) by **reusing
//! [`crate::proxy::build_upstream_client`] verbatim** — so this adds NO new dependency, no new TLS
//! config, and the no-C trust-boundary gate is unaffected.
//!
//! ## Sync → async bridge (load-bearing)
//! [`HttpTransport::execute`] is **synchronous**, but it is only ever called from inside a
//! `spawn_blocking` closure (the unlock RPC handler / the `Mint` blocking task — see
//! `grpc.rs`), so it runs on a **blocking** thread, NOT a reactor thread. We capture the runtime
//! [`Handle`](tokio::runtime::Handle) at construction (`DaemonHttpTransport::new` is called from
//! async code, where `Handle::current()` is valid) and `Handle::block_on` the reqwest future inside
//! `execute`. `Handle::block_on` is sound off the reactor (it would panic on a reactor thread) —
//! the same off-reactor `block_on` discipline the libSQL store uses (lib.rs:184-186).
//!
//! ## No secret / no error text on the wire (CF-6, mirrors `DaemonUpstream`)
//! Any reqwest failure is mapped to a FIXED, key-free [`TransportError::Io`] string — never the
//! error's own text (a hostile/buggy adapter could echo a header) and never the URL. The minted
//! token lives only in the success body, which the engine's `mint_github` parser moves straight into
//! `Zeroizing`; this module never logs the body.

use envctl_secrets::mint_github::{HttpRequest, HttpResponse, HttpTransport, TransportError};
use tokio::runtime::Handle;

/// Reuses [`crate::proxy::build_upstream_client`] (frozen webpki-roots/ring TLS + `.no_proxy()`),
/// bridging the engine's SYNC mint seam onto the daemon's tokio runtime via a captured
/// [`Handle`]. Constructed once per provider rebuild (on vault unlock) and held inside the
/// `GitHubAppMint`.
pub struct DaemonHttpTransport {
    client: reqwest::Client,
    rt: Handle,
}

impl DaemonHttpTransport {
    /// Build the transport. MUST be called from within the tokio runtime (async context) so
    /// `Handle::current()` resolves — the unlock RPC handler is async, satisfying this.
    pub fn new() -> Self {
        DaemonHttpTransport {
            client: crate::proxy::build_upstream_client(),
            rt: Handle::current(),
        }
    }
}

impl HttpTransport for DaemonHttpTransport {
    fn execute(&self, req: &HttpRequest) -> Result<HttpResponse, TransportError> {
        // Build the reqwest request from the engine's transport-agnostic shape. A build failure is a
        // misuse (bad method/url/header) — return a fixed key-free string.
        let method = reqwest::Method::from_bytes(req.method.as_bytes())
            .map_err(|_| TransportError::Io("mint request: bad method".to_string()))?;
        let mut builder = self.client.request(method, &req.url);
        for (k, v) in &req.headers {
            builder = builder.header(k, v);
        }
        let built = builder
            .body(req.body.clone())
            .build()
            .map_err(|_| TransportError::Io("mint request build failed".to_string()))?;

        // Off-reactor block_on: `execute` runs on a spawn_blocking thread, so `Handle::block_on` is
        // sound (it would panic on a reactor thread). On ANY transport error, surface a FIXED,
        // key-free string — never the error's own text (a buggy adapter could echo the auth header)
        // and never the URL. Mirrors `DaemonUpstream::send`'s "never echo error text".
        let client = self.client.clone();
        let resp = self
            .rt
            .block_on(async move { client.execute(built).await })
            .map_err(|_| TransportError::Io("mint transport send failed".to_string()))?;

        let status = resp.status().as_u16();
        let body = self
            .rt
            .block_on(async move { resp.bytes().await })
            .map_err(|_| TransportError::Io("mint transport body read failed".to_string()))?
            .to_vec();

        Ok(HttpResponse { status, body })
    }
}

impl Default for DaemonHttpTransport {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The transport shapes the reqwest request from the engine's `HttpRequest` and bridges the
    /// blocking call onto a captured runtime `Handle`. We exercise the request-shaping + the
    /// sync→async bridge against a loopback mock server (no real GitHub), driven from a
    /// `spawn_blocking` thread exactly as the production call site does.
    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn execute_round_trips_against_a_loopback_server() {
        use std::io::{Read, Write};
        use std::net::TcpListener;

        // A one-shot HTTP/1.1 server: read the request, assert the method + a header, reply 201.
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();
        let server = std::thread::spawn(move || {
            let (mut sock, _) = listener.accept().unwrap();
            // Read until the end of the request headers (CRLFCRLF). A single `read` can return a
            // short packet, splitting the header block — loop until the terminator is seen.
            let mut acc: Vec<u8> = Vec::new();
            let mut chunk = [0u8; 1024];
            loop {
                let n = sock.read(&mut chunk).unwrap();
                if n == 0 {
                    break;
                }
                acc.extend_from_slice(&chunk[..n]);
                if acc.windows(4).any(|w| w == b"\r\n\r\n") {
                    break;
                }
            }
            let req_text = String::from_utf8_lossy(&acc).into_owned();
            let body = r#"{"token":"ghs_loopback","expires_at":"2026-06-12T23:00:00Z"}"#;
            let resp = format!(
                "HTTP/1.1 201 Created\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                body.len(),
                body
            );
            sock.write_all(resp.as_bytes()).unwrap();
            sock.flush().unwrap();
            req_text
        });

        // Construct the transport in the async context (captures Handle::current()), then run the
        // SYNC `execute` on a blocking thread — the production sync→async bridge.
        let transport = DaemonHttpTransport::new();
        let url = format!("http://{addr}/app/installations/99/access_tokens");
        let req = HttpRequest {
            method: "POST",
            url,
            headers: vec![
                ("Authorization".into(), "Bearer jwt".into()),
                ("Content-Type".into(), "application/json".into()),
            ],
            body: br#"{"permissions":{"checks":"write"}}"#.to_vec(),
        };
        let resp = tokio::task::spawn_blocking(move || transport.execute(&req))
            .await
            .unwrap()
            .expect("transport executes");

        assert_eq!(resp.status, 201);
        let parsed: serde_json::Value = serde_json::from_slice(&resp.body).unwrap();
        assert_eq!(parsed["token"], "ghs_loopback");

        let req_text = server.join().unwrap();
        assert!(req_text.starts_with("POST "), "method shaped onto the wire");
        assert!(
            req_text
                .to_ascii_lowercase()
                .contains("authorization: bearer jwt"),
            "auth header forwarded; got request:\n{req_text}"
        );
    }
}
