//! Schema-faithful target descriptors and their JSON/YAML loader.

use super::{MigrationDbError, Result, Risk, TargetSpec, TargetType};
use serde::Deserialize;
use serde_json::{Map, Value};
use std::collections::BTreeMap;
use std::path::Path;

#[derive(Debug, Clone, Deserialize)]
pub struct TargetDescriptor {
    pub schema_version: u64,
    pub target_id: String,
    pub target_type: TargetType,
    pub primary_root: String,
    #[serde(default)]
    pub compare_root: Option<String>,
    #[serde(default = "default_output_root")]
    pub output_root: String,
    #[serde(default)]
    pub include: Vec<String>,
    #[serde(default)]
    pub exclude: Vec<String>,
    pub safety: TargetSafety,
    #[serde(default)]
    pub collectors: BTreeMap<String, bool>,
    pub artifact_contract: NamedVersion,
    pub recipe: NamedVersion,
    #[serde(default = "default_metadata")]
    pub metadata: Value,
    #[serde(default)]
    #[serde(flatten)]
    pub extensions: BTreeMap<String, Value>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TargetSafety {
    pub default_mode: TargetSafetyMode,
    pub max_auto_risk: Risk,
    pub allow_network: bool,
    pub allow_destructive: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct NamedVersion {
    pub name: String,
    pub version: Value,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum TargetSafetyMode {
    Observer,
    ApprovalGated,
    Operator,
    AgentOnly,
}

impl TargetSafetyMode {
    fn as_str(&self) -> &'static str {
        match self {
            Self::Observer => "observer",
            Self::ApprovalGated => "approval-gated",
            Self::Operator => "operator",
            Self::AgentOnly => "agent-only",
        }
    }
}

impl TargetDescriptor {
    /// Validate the fields which form the target-registry contract and derive
    /// the storage row entirely from the descriptor.
    pub fn into_spec(self, raw: Value) -> Result<TargetSpec> {
        if self.schema_version == 0 {
            return Err(MigrationDbError::Validation(
                "descriptor.schema_version must be at least 1".into(),
            ));
        }
        if self.target_id.trim().is_empty() || self.primary_root.trim().is_empty() {
            return Err(MigrationDbError::Validation(
                "descriptor target_id and primary_root must be non-empty".into(),
            ));
        }
        if self.output_root.trim().is_empty() {
            return Err(MigrationDbError::Validation(
                "descriptor output_root must be non-empty".into(),
            ));
        }
        for (field, reference) in [
            ("artifact_contract", &self.artifact_contract),
            ("recipe", &self.recipe),
        ] {
            if reference.name.trim().is_empty()
                || !(reference.version.is_string()
                    || reference.version.is_u64()
                    || reference.version.is_i64())
            {
                return Err(MigrationDbError::Validation(format!(
                    "descriptor.{field} requires a non-empty name and string or integer version"
                )));
            }
        }
        Ok(TargetSpec {
            target_id: self.target_id,
            target_type: self.target_type,
            primary_root: self.primary_root,
            compare_root: self.compare_root,
            descriptor: raw,
            safety_mode: self.safety.default_mode.as_str().into(),
            max_auto_risk: self.safety.max_auto_risk,
        })
    }
}

fn default_output_root() -> String {
    "migration-artifacts".to_string()
}

fn default_metadata() -> Value {
    Value::Object(Map::new())
}

pub fn parse_target_descriptor(
    text: &str,
    extension: Option<&str>,
) -> Result<(TargetDescriptor, Value)> {
    let raw: Value = match extension.map(str::to_ascii_lowercase).as_deref() {
        Some("json") => serde_json::from_str(text)?,
        Some("yaml") | Some("yml") => serde_yaml::from_str(text)
            .map_err(|e| MigrationDbError::Validation(format!("invalid YAML descriptor: {e}")))?,
        _ => serde_json::from_str(text).or_else(|json_error| {
            serde_yaml::from_str(text).map_err(|yaml_error| {
                MigrationDbError::Validation(format!(
                    "invalid descriptor (JSON: {json_error}; YAML: {yaml_error})"
                ))
            })
        })?,
    };
    if !raw.is_object() {
        return Err(MigrationDbError::Validation(
            "descriptor must be an object".into(),
        ));
    }
    let typed: TargetDescriptor = serde_json::from_value(raw.clone()).map_err(|e| {
        MigrationDbError::Validation(format!("descriptor does not match target schema: {e}"))
    })?;
    Ok((typed, raw))
}

pub fn load_target_descriptor(path: &Path) -> Result<(TargetDescriptor, Value)> {
    let text = std::fs::read_to_string(path)?;
    let extension = path.extension().and_then(|value| value.to_str());
    parse_target_descriptor(&text, extension)
}
