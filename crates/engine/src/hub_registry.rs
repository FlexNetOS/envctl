//! Federated hub registry view.
//!
//! envctl owns the component manifest. The hub registries are a read-only layer
//! above that: each `<name>_hub/registry.json` records a tool surface and the
//! envctl component that provisions or exposes it. The loader stays sync and
//! pure-Rust so the CLI and GUI can share the same report.

use crate::model::Registry;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

const REGISTRY_SCHEMA: &str = "hub.registry.v1";

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
struct HubRegistryFile {
    schema: String,
    #[serde(default)]
    entries: Vec<HubRegistryEntry>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HubRegistryEntry {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub description: String,
    pub component: String,
    #[serde(default)]
    pub status: HubRegistryStatus,
    #[serde(default)]
    pub tier: u8,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HubRegistryStatus {
    #[default]
    Stable,
    Experimental,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HubRegistrySource {
    pub hub: String,
    pub path: String,
    pub entries: usize,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HubRegistryDrift {
    pub hub: String,
    pub id: String,
    pub component: String,
    pub detail: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HubRegistryEntryView {
    pub hub: String,
    pub path: String,
    #[serde(flatten)]
    pub entry: HubRegistryEntry,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HubRegistryReport {
    pub root: String,
    pub sources: Vec<HubRegistrySource>,
    pub entries: Vec<HubRegistryEntryView>,
    pub drift: Vec<HubRegistryDrift>,
}

impl HubRegistryReport {
    pub fn clean(&self) -> bool {
        self.drift.is_empty()
    }
}

pub fn load(root: &Path, manifest: &Registry) -> anyhow::Result<HubRegistryReport> {
    let mut sources = Vec::new();
    let mut entries = Vec::new();
    let mut drift = Vec::new();

    if !root.is_dir() {
        return Ok(HubRegistryReport {
            root: root.display().to_string(),
            sources,
            entries,
            drift,
        });
    }

    let mut seen = BTreeSet::new();
    let mut hubs: Vec<(String, PathBuf)> = Vec::new();
    for entry in std::fs::read_dir(root)? {
        let entry = entry?;
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let Some(name) = path.file_name().and_then(|s| s.to_str()) else {
            continue;
        };
        if !name.ends_with("_hub") {
            continue;
        }
        let registry = path.join("registry.json");
        if registry.is_file() {
            hubs.push((name.to_string(), registry));
        }
    }
    hubs.sort_by(|a, b| a.0.cmp(&b.0));

    for (hub, path) in hubs {
        let text = std::fs::read_to_string(&path)?;
        let file: HubRegistryFile = serde_json::from_str(&text)
            .map_err(|e| anyhow::anyhow!("failed to parse {}: {e}", path.display()))?;
        if file.schema != REGISTRY_SCHEMA {
            return Err(anyhow::anyhow!(
                "{}: unsupported schema {}, expected {}",
                path.display(),
                file.schema,
                REGISTRY_SCHEMA
            ));
        }

        sources.push(HubRegistrySource {
            hub: hub.clone(),
            path: path.display().to_string(),
            entries: file.entries.len(),
        });

        for entry in file.entries {
            if !seen.insert((hub.clone(), entry.id.clone())) {
                return Err(anyhow::anyhow!(
                    "{}: duplicate entry id {}",
                    path.display(),
                    entry.id
                ));
            }
            if manifest.get(&entry.component).is_none() {
                drift.push(HubRegistryDrift {
                    hub: hub.clone(),
                    id: entry.id.clone(),
                    component: entry.component.clone(),
                    detail: format!(
                        "entry binds to missing manifest component '{}'",
                        entry.component
                    ),
                });
            }
            entries.push(HubRegistryEntryView {
                hub: hub.clone(),
                path: path.display().to_string(),
                entry,
            });
        }
    }

    entries.sort_by(|a, b| a.entry.id.cmp(&b.entry.id).then(a.hub.cmp(&b.hub)));

    Ok(HubRegistryReport {
        root: root.display().to_string(),
        sources,
        entries,
        drift,
    })
}
