//! `secretd` as a library: the control-plane daemon's gRPC services, the SO_PEERCRED owner gate, the
//! proto<->engine conversions, the sync->async event bridge, and the server assembly.
//!
//! `main.rs` is a thin binary over this crate; the e2e integration test (`tests/e2e.rs`) consumes
//! these SAME modules so the REAL daemon code — `server::serve`, the `grpc` handlers, `conv`, and the
//! `peercred::OwnerGuard` interceptor — is under test, not an inline replica.
pub mod audit;
pub mod config;
pub mod conv;
// F2 / TASK-0031 PR-1: the remote relay-edge HTTPS plane (in-process TLS + RFC 9449 DPoP + EKM
// channel binding). Gated behind `relay-edge` (default-OFF); a default secretd build omits it.
#[cfg(feature = "relay-edge")]
pub mod edge;
pub mod grpc;
pub mod peercred;
pub mod proxy;
pub mod server;
#[cfg(feature = "provider-github")]
pub mod transport; // DaemonHttpTransport: the engine's HttpTransport seam (native GitHub App mint, G2)
