//! Typed entities mirroring the package DDL (sql/001) column-for-column.
//! Enum variants mirror the SQL CHECK constraints exactly; serde renames keep the
//! JSON wire shape identical to the DDL's TEXT values.

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Local helper: string-backed enums with exact DDL CHECK values.
macro_rules! str_enum {
    ($name:ident { $($variant:ident => $text:literal),+ $(,)? }) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
        pub enum $name {
            $(#[serde(rename = $text)] $variant,)+
        }
        impl $name {
            pub fn as_str(&self) -> &'static str {
                match self { $(Self::$variant => $text,)+ }
            }
            pub fn parse(s: &str) -> crate::migration_db::Result<Self> {
                match s {
                    $($text => Ok(Self::$variant),)+
                    other => Err(crate::migration_db::MigrationDbError::Validation(
                        format!(concat!("invalid ", stringify!($name), ": {} (expected one of: ", $($text, " ",)+ ")"), other),
                    )),
                }
            }
        }
        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.write_str(self.as_str())
            }
        }
    };
}

str_enum!(TargetType {
    Codebase => "codebase",
    Data => "data",
    Infrastructure => "infrastructure",
    Integration => "integration",
    Mixed => "mixed",
});

str_enum!(RunStatus {
    Created => "created",
    Planning => "planning",
    AwaitingApproval => "awaiting_approval",
    Running => "running",
    Paused => "paused",
    Validating => "validating",
    Completed => "completed",
    Failed => "failed",
    Blocked => "blocked",
    Cancelled => "cancelled",
    Denied => "denied",
});

str_enum!(HumanMode {
    Observer => "observer",
    ApprovalGated => "approval-gated",
    Operator => "operator",
    AgentOnly => "agent-only",
});

str_enum!(OpStatus {
    Queued => "queued",
    Ready => "ready",
    AwaitingApproval => "awaiting_approval",
    Running => "running",
    Succeeded => "succeeded",
    Failed => "failed",
    Blocked => "blocked",
    Denied => "denied",
    Cancelled => "cancelled",
});

str_enum!(Risk {
    R0 => "R0",
    R1 => "R1",
    R2 => "R2",
    R3 => "R3",
    R4 => "R4",
    R5 => "R5",
});

impl Risk {
    /// R3+ requires explicit approval (AGENT_CONTROL_PROTOCOL).
    pub fn requires_approval(&self) -> bool {
        matches!(self, Risk::R3 | Risk::R4 | Risk::R5)
    }
}

str_enum!(ActorType {
    System => "system",
    Agent => "agent",
    Human => "human",
    Plugin => "plugin",
    External => "external",
});

str_enum!(ApprovalStatus {
    Open => "open",
    Approved => "approved",
    Denied => "denied",
    Expired => "expired",
    Cancelled => "cancelled",
});

str_enum!(ValidationStatus {
    Pass => "pass",
    Fail => "fail",
    Warn => "warn",
    Blocked => "blocked",
    Unknown => "unknown",
});

str_enum!(ArtifactStatus {
    Complete => "complete",
    Partial => "partial",
    Unknown => "unknown",
    Blocked => "blocked",
});

str_enum!(RollbackStatus {
    Planned => "planned",
    AwaitingApproval => "awaiting_approval",
    Running => "running",
    Succeeded => "succeeded",
    Failed => "failed",
    Blocked => "blocked",
    Cancelled => "cancelled",
});

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Target {
    pub id: String,
    pub target_id: String,
    pub target_type: TargetType,
    pub primary_root: String,
    pub compare_root: Option<String>,
    pub descriptor_json: Value,
    pub descriptor_hash: String,
    pub safety_mode: String,
    pub max_auto_risk: Risk,
    pub created_at_utc: String,
    pub updated_at_utc: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Package {
    pub id: String,
    pub package_name: String,
    pub package_path: String,
    pub package_hash: String,
    pub manifest_json: Value,
    pub imported_at_utc: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArtifactContract {
    pub id: String,
    pub contract_name: String,
    pub contract_version: String,
    pub source_package_id: Option<String>,
    pub contract_hash: String,
    pub contract_json: Value,
    pub created_at_utc: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Recipe {
    pub id: String,
    pub recipe_name: String,
    pub recipe_version: String,
    pub artifact_contract_id: String,
    pub recipe_hash: String,
    pub recipe_json: Value,
    pub created_at_utc: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Run {
    pub id: String,
    pub target_id: String,
    pub recipe_id: String,
    pub artifact_contract_id: String,
    pub status: RunStatus,
    pub human_mode: HumanMode,
    pub initiated_by: Option<String>,
    pub sandbox_policy: Option<String>,
    pub approval_policy: Option<String>,
    pub tool_versions_json: Option<Value>,
    pub reproducibility_hash: Option<String>,
    pub started_at_utc: Option<String>,
    pub completed_at_utc: Option<String>,
    pub created_at_utc: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Operation {
    pub id: String,
    pub run_id: String,
    pub parent_operation_id: Option<String>,
    pub operation_type: String,
    pub phase: Option<String>,
    pub status: OpStatus,
    pub risk: Risk,
    pub idempotency_key: String,
    pub command_hash: Option<String>,
    pub command_redacted: Option<String>,
    pub input_json: Option<Value>,
    pub output_ref: Option<String>,
    pub error_json: Option<Value>,
    pub started_at_utc: Option<String>,
    pub completed_at_utc: Option<String>,
    pub created_at_utc: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunEvent {
    pub id: String,
    pub run_id: String,
    pub event_seq: u64,
    pub event_type: String,
    pub phase: Option<String>,
    pub actor_type: ActorType,
    pub actor_id: Option<String>,
    pub operation_id: Option<String>,
    pub payload_json: Value,
    pub evidence_refs_json: Option<Value>,
    pub previous_event_hash: Option<String>,
    pub event_hash: Option<String>,
    pub created_at_utc: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Evidence {
    pub id: String,
    pub run_id: String,
    pub operation_id: Option<String>,
    pub uri: String,
    pub evidence_kind: String,
    pub sha256: Option<String>,
    pub redacted: bool,
    pub metadata_json: Option<Value>,
    pub created_at_utc: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Artifact {
    pub id: String,
    pub run_id: String,
    pub artifact_id: String,
    pub title: String,
    pub artifact_type: Option<String>,
    pub status: ArtifactStatus,
    pub path: Option<String>,
    pub content_hash: Option<String>,
    pub generated_by_operation_id: Option<String>,
    pub evidence_json: Option<Value>,
    pub links_json: Option<Value>,
    pub created_at_utc: String,
    pub updated_at_utc: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphEdge {
    pub id: String,
    pub run_id: String,
    pub from_node: String,
    pub to_node: String,
    pub edge_type: String,
    pub source_artifact_id: Option<String>,
    pub confidence: Option<String>,
    pub evidence_json: Option<Value>,
    pub created_at_utc: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Approval {
    pub id: String,
    pub run_id: String,
    pub operation_id: String,
    pub risk: Risk,
    pub status: ApprovalStatus,
    pub requested_by: Option<String>,
    pub decided_by: Option<String>,
    pub reason: Option<String>,
    pub requested_at_utc: String,
    pub decided_at_utc: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Validation {
    pub id: String,
    pub run_id: String,
    pub artifact_id: Option<String>,
    pub operation_id: Option<String>,
    pub validator: String,
    pub status: ValidationStatus,
    pub details_json: Option<Value>,
    pub evidence_json: Option<Value>,
    pub created_at_utc: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Checkpoint {
    pub id: String,
    pub run_id: String,
    pub operation_id: Option<String>,
    pub checkpoint_kind: String,
    pub checkpoint_ref: String,
    pub checkpoint_hash: Option<String>,
    pub metadata_json: Option<Value>,
    pub created_at_utc: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rollback {
    pub id: String,
    pub run_id: String,
    pub operation_id: Option<String>,
    pub rollback_type: String,
    pub status: RollbackStatus,
    pub plan_json: Value,
    pub result_json: Option<Value>,
    pub created_at_utc: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentSession {
    pub id: String,
    pub run_id: Option<String>,
    pub agent_name: String,
    pub model_label: Option<String>,
    pub authority_level: Option<String>,
    pub session_json: Option<Value>,
    pub created_at_utc: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginSession {
    pub id: String,
    pub run_id: Option<String>,
    pub plugin_name: String,
    pub plugin_version: Option<String>,
    pub nu_version: Option<String>,
    pub human_mode: Option<HumanMode>,
    pub session_json: Option<Value>,
    pub created_at_utc: String,
}
