//! Streaming-revocation tear-down for the F2 remote relay edge (TASK-0032 / FS-S5).
//!
//! A long-lived HTTP/2 response stream (e.g. an SSE/token stream from the upstream provider) is
//! authorized ONCE at open by the swap's `decide()`. Without an in-stream re-check, an authorization
//! that lapses MID-STREAM — a relay/bearer revoke, a vault lock, or a USB-key pull — would not stop
//! the already-flowing bytes. This module wraps the engine's per-request upstream chunk receiver in a
//! supervised middle task that, alongside forwarding each chunk downstream, periodically re-runs the
//! engine's `decide()` (via [`Engine::relay_stream_authorized`]) and ACTIVELY tears the in-flight
//! stream down the instant authorization lapses.
//!
//! ## Invariants (this module is I/O plumbing ONLY — the engine is the sole policy authority)
//! - ALL re-check policy lives in [`Engine::relay_stream_authorized`] → `decide()`. This module only
//!   `select!`s, forwards, and drops; it makes NO independent judgment about whether a stream may
//!   live. It can only ever tear DOWN, never keep alive past a `decide()` deny.
//! - FAIL-CLOSED: every uncertainty tears the stream down. A `decide()` Deny → drop the sender; the
//!   re-check's own `StreamAuthz::TearDown` already folds locked-vault / poisoned-lock / store-err /
//!   vanished-bearer / USB-absent into a tear-down (the engine maps those internal `Err`s closed).
//!   The hard max-duration deadline tears down unconditionally. There is NO `unwrap`/`expect`/panic
//!   on the periodic path.
//! - NO secret bytes in logs/audit: the tear-down event carries only {reason, relay, token_id}; the
//!   real key and the proxied body NEVER appear. The body bytes flow through opaquely.
//! - Backpressure preserved: the wrapped (downstream) channel stays bounded at `BODY_CHANNEL_CAP`, so
//!   a slow client still applies backpressure to the upstream pump — no unbounded buffering.
//! - Default-OFF behind the `relay-edge` cargo feature (same as the listener).

use std::time::Duration;

use envctl_secrets::broker::decide::RemotePeer;
use envctl_secrets::{DenyReason, EgressReq, Engine, EventSink, Method, SecretEvent, StreamAuthz};

use crate::proxy::{BodyChunk, BodyRx, BODY_CHANNEL_CAP};

/// Build the ZERO-byte re-check `EgressReq` for a streaming re-check from the open-time context. It
/// mirrors the host/method/path/`remote` the swap's `decide()` saw at open, but carries `bytes_out:
/// 0` so the engine's `peek` consumes NO byte budget (and `peer_uid/pid: None` — the remote plane has
/// no local peer; identity rides the `RemotePeer`). The SAME `remote` is re-asserted by clause 11a.
pub fn recheck_egress_req(
    method: Method,
    host: String,
    path: String,
    remote: Option<RemotePeer>,
) -> EgressReq {
    EgressReq {
        method,
        host,
        path,
        headers: Vec::new(),
        bytes_out: 0,
        peer_uid: None,
        peer_pid: None,
        observed_sni: None,
        remote,
    }
}

/// How often the supervised forwarder re-runs `decide()` against the live clock / revocation / USB
/// gate. The worst-case detection latency for a revoke/lock/USB-pull is one interval. 2s is "prompt"
/// (not instant — a `tokio::sync::watch` push for ~0 latency is the documented PR-4 follow-up).
pub const RECHECK_INTERVAL: Duration = Duration::from_secs(2);

/// Hard upper bound on a single relay stream's lifetime. A stream that has run this long is torn down
/// unconditionally (defense-in-depth: caps a wedged/abusive long-poll even if `decide()` keeps
/// allowing it). ~5 min covers a generous provider streaming response.
pub const MAX_STREAM_SECS: u64 = 300;

/// Re-check cadence + lifetime cap for one supervised stream. Production uses [`Timing::production`];
/// the e2e test injects small values via [`Timing::new`] so it never sleeps for the production
/// interval/cap (a test-only override, NOT a production sleep).
#[derive(Clone, Copy, Debug)]
pub struct Timing {
    pub interval: Duration,
    pub max_duration: Duration,
}

impl Timing {
    pub fn production() -> Self {
        Timing {
            interval: RECHECK_INTERVAL,
            max_duration: Duration::from_secs(MAX_STREAM_SECS),
        }
    }

    /// Test-only override of the cadence/cap (so the e2e closes a stream within seconds, not the
    /// production 2s/300s). Inert in production (the edge always uses [`Timing::production`]).
    pub fn new(interval: Duration, max_duration: Duration) -> Self {
        Timing {
            interval,
            max_duration,
        }
    }
}

/// The non-secret context the tear-down audit event carries (metadata ONLY): the relay/upstream label
/// and the PUBLIC bearer `token_id` (the same id `RelaySwapped` already audits — never the bearer
/// secret, never the key, never the body).
#[derive(Clone)]
pub struct StreamAudit {
    pub relay: String,
    pub token_id: String,
}

/// Wrap the engine's per-request upstream-chunk receiver `upstream_rx` in a supervised forwarding task
/// and return a NEW bounded receiver that becomes the response body. The task `tokio::select!`s over:
///   (a) the next upstream chunk → forward it downstream (awaited send = backpressure),
///   (b) a `RECHECK_INTERVAL` tick → `engine.relay_stream_authorized(...)`; on `TearDown` stop+drop,
///   (c) a hard `MAX_STREAM_SECS` deadline → tear down unconditionally.
/// On ANY tear-down it simply DROPS the downstream sender — the `StreamBody` ends cleanly and the
/// client sees an HTTP/2 stream close + a truncated body — and emits a metadata-only audit event.
///
/// `recheck_req` MUST be a ZERO-byte `EgressReq` carrying the SAME `RemotePeer` captured at the
/// stream's open (so `decide()` clause 11a re-asserts dpop_verified + the client_id/jkt binding each
/// tick) and the same host/method/path the swap saw. `bearer` is the relay bearer presented at open.
#[allow(clippy::too_many_arguments)]
pub fn relay_stream_response(
    upstream_rx: BodyRx,
    engine: Engine,
    bearer: zeroize::Zeroizing<String>,
    recheck_req: EgressReq,
    audit: StreamAudit,
    sink: EventSink,
    timing: Timing,
) -> BodyRx {
    let (down_tx, down_rx) = tokio::sync::mpsc::channel::<BodyChunk>(BODY_CHANNEL_CAP);

    tokio::spawn(async move {
        let mut upstream_rx = upstream_rx;
        let mut ticker = tokio::time::interval(timing.interval);
        // The first `tick()` completes immediately; consume it so the first REAL re-check happens one
        // full interval in (the swap already authorized the open — no need to re-check at t=0).
        ticker.tick().await;
        let deadline = tokio::time::sleep(timing.max_duration);
        tokio::pin!(deadline);

        loop {
            tokio::select! {
                // Bias is irrelevant for correctness (every branch is a tear-down except the forward),
                // but we keep the default random bias so a hot upstream cannot starve the re-check.
                chunk = upstream_rx.recv() => {
                    match chunk {
                        // Forward the opaque body bytes downstream. An awaited send preserves the
                        // bounded backpressure; a send error means the client hung up → stop (no
                        // tear-down event: the client, not policy, ended it).
                        Some(c) => {
                            if down_tx.send(c).await.is_err() {
                                break;
                            }
                        }
                        // Upstream EOF/error: the stream completed naturally. Drop the sender (clean
                        // end); not a revocation tear-down.
                        None => break,
                    }
                }
                _ = ticker.tick() => {
                    // The ONLY authority: re-run decide() with fresh clock/revocation/USB reads. Any
                    // lapse (incl. internal Err mapped closed inside the engine) → tear down.
                    if let StreamAuthz::TearDown(reason) =
                        engine.relay_stream_authorized(&bearer, &recheck_req, &sink)
                    {
                        emit_teardown(&sink, &audit, reason);
                        break; // drop down_tx → StreamBody ends → client sees the close.
                    }
                }
                _ = &mut deadline => {
                    // Hard lifetime cap: tear down unconditionally. Reported as PolicyExpired (the
                    // closest decision reason for "this stream has outlived its allowed lifetime").
                    emit_teardown(&sink, &audit, DenyReason::PolicyExpired);
                    break;
                }
            }
        }
        // down_tx drops here → the ReceiverStream ends → the hyper response body completes/aborts.
    });

    down_rx
}

/// Emit the metadata-only tear-down event. Carries ONLY {relay, token_id, reason} — never the bearer,
/// the real key, or the proxied body. Cosmetic/best-effort (the engine already durably audited the
/// open swap); the daemon never prints here.
fn emit_teardown(sink: &EventSink, audit: &StreamAudit, reason: DenyReason) {
    let reason_str = serde_json::to_value(reason)
        .ok()
        .and_then(|v| v.as_str().map(str::to_string))
        .unwrap_or_else(|| format!("{reason:?}"));
    sink.emit(SecretEvent::RelayStreamTornDown {
        relay: audit.relay.clone(),
        token_id: audit.token_id.clone(),
        reason: reason_str,
    });
}
