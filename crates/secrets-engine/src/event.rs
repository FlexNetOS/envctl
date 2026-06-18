//! The observability spine (exact envctl shape, std mpsc). The engine never prints; it emits
//! `SecretEvent`s. Security *outcomes* are committed to the durable, hash-chained audit log by
//! the engine BEFORE an RPC returns — this channel is cosmetic/best-effort (HF-14).
use serde::{Deserialize, Serialize};
use std::sync::mpsc::{Receiver, Sender};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum SecretEvent {
    VaultUnlocked {
        factor: crate::keyslot::Factor,
    },
    VaultLocked,
    Audit(AuditRecord),
    SecretWritten {
        name: String,
        version: u32,
    },
    SecretRead {
        name: String,
        by_uid: u32,
    },
    RelayMinted {
        relay: String,
        kind: crate::broker::RelayKind,
        expires_at: String,
    }, // bearer NEVER in payload
    RelayRotated {
        relay: String,
        expires_at: String,
    },
    RelayRevoked {
        relay: String,
        reason: String,
    },
    /// token_id + client identity for per-swap traceability (OI-11); bearer NEVER included.
    RelaySwapped {
        relay: String,
        host: String,
        method: String,
        allowed: bool,
        token_id: String,
        client_uid: u32,
        client_label: String,
    },
    /// A long-lived relay stream was actively TORN DOWN mid-flight because the periodic re-check
    /// lapsed (TASK-0032 / FS-S5: relay/bearer revoke, vault lock, USB-key pull, or the max-duration
    /// cap). METADATA ONLY — `reason` is the `DenyReason` discriminant (the max-duration cap maps to
    /// the policy/bearer-expiry reason), `token_id` is the public bearer id; the bearer, the real key,
    /// and the proxied body are NEVER included. Consumed identically by the CLI + GUI (no divergence).
    RelayStreamTornDown {
        relay: String,
        token_id: String,
        reason: String,
    },
    /// The F2 relay edge SHED a request at the anti-abuse stage (TASK-0031-PR2): a per-IP admission
    /// rate breach (CVE-2024-47609) or a missing/stale DPoP-Nonce challenge. METADATA ONLY — `reason`
    /// is a fixed discriminant label (e.g. `"rate_limited"`, `"nonce_challenge"`), `client_or_ip` is
    /// the source IP or public client label, `count` is a best-effort occurrence tally. NO bearer,
    /// proof, EKM, nonce, key, or body bytes are ever included. Consumed identically by the CLI + GUI.
    EdgeRequestShed {
        reason: String,
        client_or_ip: String,
        count: u64,
    },
    GuardRefused {
        subject: String,
        reason: String,
    },
    CaIssued {
        serial: String,
        cn: String,
        not_after: String,
    },
    LeafMinted {
        sni: String,
        relay: String,
        not_after: String,
    },
    Log {
        source: String,
        stream: Stream,
        line: String,
    },
    ChildExited {
        code: i32,
    },
    RunFinished {
        summary: RunSummary,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Stream {
    Stdout,
    Stderr,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AuditRecord {
    pub seq: i64,
    pub ts: String,
    pub actor_uid: Option<u32>,
    pub event_type: String,
    pub subject: Option<String>,
    pub detail: serde_json::Value,
    pub outcome: AuditOutcome,
    pub prev_hash: Vec<u8>,
    pub row_hash: Vec<u8>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuditOutcome {
    Ok,
    Refused,
    Failed,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct RunSummary {
    pub failed: Vec<String>,
    pub refused: Vec<String>,
}
impl RunSummary {
    pub fn ok(&self) -> bool {
        self.failed.is_empty() && self.refused.is_empty()
    }
}

/// Cosmetic, best-effort fan-out of `SecretEvent`s. Drop-on-closed is fine.
#[derive(Clone)]
pub struct EventSink(Sender<SecretEvent>);
impl EventSink {
    pub fn channel() -> (EventSink, Receiver<SecretEvent>) {
        let (tx, rx) = std::sync::mpsc::channel();
        (EventSink(tx), rx)
    }
    pub fn null() -> EventSink {
        let (tx, _rx) = std::sync::mpsc::channel();
        EventSink(tx)
    }
    pub fn emit(&self, ev: SecretEvent) {
        let _ = self.0.send(ev);
    }
}
