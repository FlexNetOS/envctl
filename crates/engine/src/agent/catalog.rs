//! Engine-owned skill-catalog commands.  The catalog changes only canonical
//! owner state and its generated active projection; installation remains the
//! normal lock → sync lifecycle.

use serde::Serialize;

use envctl_agent_env::catalog::{Catalog, CatalogPack, CatalogSkill, CORE_PACK};

use crate::agent::{AgentCatalogSpec, AgentLockMode, AgentLockSpec, AgentSyncSpec};
use crate::event::EventSink;
use crate::Engine;

#[derive(Clone, Debug, Serialize)]
pub struct AgentCatalogReport {
    pub action: String,
    pub active_packs: Vec<String>,
    pub packs: Vec<CatalogPack>,
    pub skills: Vec<CatalogSkill>,
    pub changed: bool,
    pub lifecycle_verified: bool,
    pub projection: String,
}

impl Engine {
    /// Read or change the project-owned capability catalog. `apply=false` is a
    /// strict preview: no catalog source or generated projection is written.
    pub fn agent_catalog(
        &self,
        spec: AgentCatalogSpec,
        _sink: &EventSink,
    ) -> anyhow::Result<AgentCatalogReport> {
        let root = std::env::current_dir()?;
        let mut catalog = Catalog::load(&root)?;
        validate_selector_count(&spec)?;
        let mut action = "list".to_string();
        let mut changed = false;

        if let Some(query) = spec.search.as_deref() {
            let skills = catalog.search(query)?;
            return Ok(AgentCatalogReport {
                action: "search".into(),
                active_packs: catalog.active_packs.clone(),
                packs: catalog.pack_inventory(),
                skills,
                changed: false,
                lifecycle_verified: false,
                projection: envctl_agent_env::CATALOG_ACTIVE_RELATIVE.into(),
            });
        }
        if let Some(name) = spec.show.as_deref() {
            let name = catalog.resolve_skill(name)?;
            let skills = catalog
                .inventory()?
                .into_iter()
                .filter(|skill| skill.name == name)
                .collect();
            return Ok(AgentCatalogReport {
                action: "show".into(),
                active_packs: catalog.active_packs.clone(),
                packs: catalog.pack_inventory(),
                skills,
                changed: false,
                lifecycle_verified: false,
                projection: envctl_agent_env::CATALOG_ACTIVE_RELATIVE.into(),
            });
        }
        if let Some(pack) = spec.activate_pack.as_deref() {
            if !catalog.packs.contains_key(pack) {
                anyhow::bail!("unknown capability pack `{pack}`");
            }
            if !catalog.active_packs.contains(&pack.to_string()) {
                catalog.active_packs.push(pack.to_string());
                changed = true;
            }
            action = format!("activate-pack:{pack}");
        }
        if let Some(skill) = spec.activate_skill.as_deref() {
            let skill = catalog.resolve_skill(skill)?;
            let explicit = format!("skill-{skill}");
            if !catalog.packs.contains_key(&explicit) {
                catalog.packs.insert(
                    explicit.clone(),
                    envctl_agent_env::CapabilityPack {
                        description: format!("Explicit owner activation for `{skill}`"),
                        skills: vec![skill],
                    },
                );
            }
            if !catalog.active_packs.contains(&explicit) {
                catalog.active_packs.push(explicit);
                changed = true;
            }
            action = "activate-skill".into();
        }
        if let Some(intent) = spec.activate_intent.as_deref() {
            let skill = catalog.resolve_intent(intent)?;
            let explicit = format!("skill-{skill}");
            if !catalog.packs.contains_key(&explicit) {
                catalog.packs.insert(
                    explicit.clone(),
                    envctl_agent_env::CapabilityPack {
                        description: format!("Deterministic intent activation `{intent}`"),
                        skills: vec![skill],
                    },
                );
            }
            if !catalog.active_packs.contains(&explicit) {
                catalog.active_packs.push(explicit);
                changed = true;
            }
            action = "activate-intent".into();
        }
        if let Some(pack) = spec.deactivate_pack.as_deref() {
            if pack == CORE_PACK {
                anyhow::bail!("cannot deactivate mandatory core capability pack");
            }
            let before = catalog.active_packs.len();
            catalog.active_packs.retain(|active| active != pack);
            changed = before != catalog.active_packs.len();
            if pack.starts_with("skill-")
                && !catalog.active_packs.iter().any(|active| active == pack)
                && catalog.packs.remove(pack).is_some()
            {
                changed = true;
            }
            action = format!("deactivate-pack:{pack}");
        }
        catalog.validate(&root)?;
        let skills = catalog.inventory()?;
        if spec.sync && !spec.apply {
            anyhow::bail!("agent catalog --sync requires --apply");
        }
        let mut lifecycle_verified = false;
        // `--apply --sync` is also the owner-approved repair path for an
        // already-selected catalog whose generated active projection was
        // removed or drifted. It writes only the projection derived from the
        // validated canonical catalog, never an unmanaged copy.
        if spec.apply && (changed || spec.sync) {
            catalog.save(&root)?;
            catalog.write_active_projection(&root)?;
        }
        if spec.sync {
            self.agent_lock(
                AgentLockSpec {
                    config_path: None,
                    scope_override: None,
                    check: false,
                    upgrade_only: Vec::new(),
                    lock_mode: AgentLockMode::Plain,
                },
                _sink,
            )?;
            let report = self.agent_sync(
                AgentSyncSpec {
                    config_path: None,
                    scope_override: None,
                    apply: true,
                    lock_mode: AgentLockMode::Plain,
                },
                _sink,
            )?;
            if report.summary.failed > 0 || report.summary.broken > 0 {
                anyhow::bail!(
                    "agent catalog owner sync failed; active projection was not accepted"
                );
            }
            let check = self.agent_lock(
                AgentLockSpec {
                    config_path: None,
                    scope_override: None,
                    check: true,
                    upgrade_only: Vec::new(),
                    lock_mode: AgentLockMode::Locked,
                },
                _sink,
            )?;
            if !check.drift.is_empty() {
                anyhow::bail!("agent catalog owner lock check found drift");
            }
            lifecycle_verified = true;
        }
        Ok(AgentCatalogReport {
            action,
            active_packs: catalog.active_packs.clone(),
            packs: catalog.pack_inventory(),
            skills,
            changed,
            lifecycle_verified,
            projection: envctl_agent_env::CATALOG_ACTIVE_RELATIVE.into(),
        })
    }
}

fn validate_selector_count(spec: &AgentCatalogSpec) -> anyhow::Result<()> {
    let count = [
        spec.search.is_some(),
        spec.show.is_some(),
        spec.activate_pack.is_some(),
        spec.activate_skill.is_some(),
        spec.activate_intent.is_some(),
        spec.deactivate_pack.is_some(),
    ]
    .into_iter()
    .filter(|selected| *selected)
    .count();
    if count > 1 {
        anyhow::bail!("agent catalog accepts at most one selector");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::validate_selector_count;
    use crate::agent::AgentCatalogSpec;

    #[test]
    fn shared_engine_rejects_conflicting_catalog_selectors() {
        let spec = AgentCatalogSpec {
            search: Some("rust".into()),
            activate_pack: Some("rust".into()),
            ..AgentCatalogSpec::default()
        };
        assert!(validate_selector_count(&spec)
            .unwrap_err()
            .to_string()
            .contains("at most one selector"));
    }
}
