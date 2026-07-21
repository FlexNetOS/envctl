//! Locked, owner-managed capability-pack catalog for Codex skill materialization.
//!
//! The catalog deliberately reads only its repository-local owner file.  It never
//! falls back to HOME, caches, archived paths, Git hooks, or legacy worktrees.

use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::fs;
use std::path::{Component, Path};

use serde::{Deserialize, Serialize};

use crate::{err, Result};

pub const CATALOG_RELATIVE: &str = "agent-skills/skill-catalog/catalog.yaml";
pub const ACTIVE_RELATIVE: &str = "agent-env.active.yaml";
pub const CORE_PACK: &str = "core";

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Catalog {
    pub version: u8,
    #[serde(default)]
    pub active_packs: Vec<String>,
    pub packs: BTreeMap<String, Pack>,
    pub skills: BTreeMap<String, Skill>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Pack {
    pub description: String,
    pub skills: Vec<String>,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct Skill {
    /// Explicit project-local canonical owner for immutable system skills.
    /// Omitted entries use the Envctl capability-pack owner convention.
    #[serde(default)]
    pub source: Option<String>,
    /// A remote owner is admitted only when it is explicitly pinned. This
    /// preserves pre-existing external owner provenance without a fallback.
    #[serde(default)]
    pub source_ref: Option<String>,
    #[serde(default)]
    pub sub_dir: Option<String>,
    #[serde(default)]
    pub aliases: Vec<String>,
    #[serde(default)]
    pub intents: Vec<String>,
    /// Compact, owner-curated terms used by catalog search.  Descriptions stay
    /// in the canonical SKILL.md files and are deliberately not copied into
    /// the always-active catalog.
    #[serde(default)]
    pub keywords: Vec<String>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct CatalogSkill {
    pub name: String,
    pub owner: String,
    pub packs: Vec<String>,
    pub active: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct CatalogPack {
    pub name: String,
    pub description: String,
    pub skills: Vec<String>,
    pub active: bool,
}

impl Catalog {
    pub fn load(project_root: &Path) -> Result<Self> {
        let path = project_root.join(CATALOG_RELATIVE);
        let bytes = fs::read_to_string(&path).map_err(|e| {
            err(format!(
                "skill catalog owner is unavailable at {}: {e}",
                path.display()
            ))
        })?;
        let catalog: Catalog = serde_yaml::from_str(&bytes)
            .map_err(|e| err(format!("invalid skill catalog {}: {e}", path.display())))?;
        catalog.validate(project_root)?;
        Ok(catalog)
    }

    pub fn save(&self, project_root: &Path) -> Result<()> {
        self.validate(project_root)?;
        let rendered = serde_yaml::to_string(self)
            .map_err(|e| err(format!("failed to render skill catalog: {e}")))?;
        crate::write_config_atomic(&project_root.join(CATALOG_RELATIVE), rendered.as_bytes())
    }

    pub fn validate(&self, project_root: &Path) -> Result<()> {
        if self.version != 1 {
            return Err(err(format!(
                "unsupported skill catalog version {}",
                self.version
            )));
        }
        if !self.packs.contains_key(CORE_PACK) {
            return Err(err("skill catalog must define mandatory core pack"));
        }
        if !self.active_packs.iter().any(|p| p == CORE_PACK) {
            return Err(err("skill catalog must keep mandatory core pack active"));
        }
        if !self
            .packs
            .get(CORE_PACK)
            .expect("core pack existence checked above")
            .skills
            .iter()
            .any(|skill| skill == "skill-catalog")
        {
            return Err(err(
                "mandatory core capability pack must materialize the skill-catalog helper",
            ));
        }
        let mut aliases = HashMap::new();
        let mut intents = HashMap::new();
        for (name, skill) in &self.skills {
            validate_name("skill", name)?;
            validate_owner_source(project_root, name, skill)?;
            for alias in &skill.aliases {
                claim_unique(&mut aliases, alias, name, "alias")?;
            }
            for intent in &skill.intents {
                claim_unique(&mut intents, intent, name, "task intent")?;
            }
            for keyword in &skill.keywords {
                validate_name("catalog keyword", keyword)?;
            }
        }
        for (pack, definition) in &self.packs {
            validate_name("pack", pack)?;
            if definition.skills.is_empty() {
                return Err(err(format!("capability pack `{pack}` must not be empty")));
            }
            for skill in &definition.skills {
                if !self.skills.contains_key(skill) {
                    return Err(err(format!(
                        "capability pack `{pack}` references unknown skill `{skill}`"
                    )));
                }
            }
        }
        for pack in &self.active_packs {
            if !self.packs.contains_key(pack) {
                return Err(err(format!(
                    "active capability pack `{pack}` is not declared"
                )));
            }
        }
        Ok(())
    }

    pub fn active_skills(&self) -> Result<BTreeSet<String>> {
        self.skills_for_packs(&self.active_packs)
    }

    pub fn skills_for_packs(&self, packs: &[String]) -> Result<BTreeSet<String>> {
        let mut selected = BTreeSet::new();
        for pack in packs {
            let definition = self
                .packs
                .get(pack)
                .ok_or_else(|| err(format!("unknown capability pack `{pack}`")))?;
            selected.extend(definition.skills.iter().cloned());
        }
        Ok(selected)
    }

    pub fn resolve_skill(&self, query: &str) -> Result<String> {
        if self.skills.contains_key(query) {
            return Ok(query.to_string());
        }
        let matches: Vec<_> = self
            .skills
            .iter()
            .filter(|(_, skill)| skill.aliases.iter().any(|alias| alias == query))
            .map(|(name, _)| name.clone())
            .collect();
        exactly_one("skill or alias", query, matches)
    }

    pub fn resolve_intent(&self, intent: &str) -> Result<String> {
        let matches: Vec<_> = self
            .skills
            .iter()
            .filter(|(_, skill)| skill.intents.iter().any(|candidate| candidate == intent))
            .map(|(name, _)| name.clone())
            .collect();
        exactly_one("task intent", intent, matches)
    }

    /// Search compact catalog metadata without reading the complete skill
    /// descriptions into the active discovery payload.
    pub fn search(&self, query: &str) -> Result<Vec<CatalogSkill>> {
        let query = query.to_ascii_lowercase();
        Ok(self
            .inventory()?
            .into_iter()
            .filter(|entry| {
                entry.name.to_ascii_lowercase().contains(&query)
                    || entry
                        .packs
                        .iter()
                        .any(|pack| pack.to_ascii_lowercase().contains(&query))
                    || self.skills.get(&entry.name).is_some_and(|skill| {
                        skill
                            .aliases
                            .iter()
                            .any(|value| value.to_ascii_lowercase().contains(&query))
                            || skill
                                .intents
                                .iter()
                                .any(|value| value.to_ascii_lowercase().contains(&query))
                            || skill
                                .keywords
                                .iter()
                                .any(|value| value.to_ascii_lowercase().contains(&query))
                    })
            })
            .collect())
    }

    pub fn inventory(&self) -> Result<Vec<CatalogSkill>> {
        let active = self.active_skills()?;
        self.skills
            .keys()
            .map(|name| {
                Ok(CatalogSkill {
                    name: name.clone(),
                    owner: source_for(name, self.skills.get(name).expect("inventory key exists"))?,
                    packs: self
                        .packs
                        .iter()
                        .filter(|(_, pack)| pack.skills.contains(name))
                        .map(|(pack, _)| pack.clone())
                        .collect(),
                    active: active.contains(name),
                })
            })
            .collect()
    }

    pub fn pack_inventory(&self) -> Vec<CatalogPack> {
        self.packs
            .iter()
            .map(|(name, pack)| CatalogPack {
                name: name.clone(),
                description: pack.description.clone(),
                skills: pack.skills.clone(),
                active: self.active_packs.contains(name),
            })
            .collect()
    }

    pub fn render_active_projection(&self) -> Result<String> {
        let skills = self.active_skills()?;
        if !skills.contains("skill-catalog") {
            return Err(err("core pack must materialize the skill-catalog helper"));
        }
        let mut out =
            String::from("# Generated by `envctl agent catalog`; do not hand-edit.\nskills:\n");
        for name in skills {
            let skill = self.skills.get(&name).expect("active key exists");
            let source = source_for(&name, skill)?;
            out.push_str(&format!("  - source: {source}\n"));
            if source.starts_with("https://") {
                out.push_str(&format!(
                    "    ref: {}\n    sub-dir: {}\n    skills:\n      - {name}\n",
                    skill.source_ref.as_deref().expect("validated remote ref"),
                    skill.sub_dir.as_deref().expect("validated remote sub-dir"),
                ));
            } else {
                out.push_str("    skills: \"*\"\n");
            }
        }
        Ok(out)
    }

    pub fn write_active_projection(&self, project_root: &Path) -> Result<()> {
        crate::write_config_atomic(
            &project_root.join(ACTIVE_RELATIVE),
            self.render_active_projection()?.as_bytes(),
        )
    }
}

/// Resolve owner sources before accepting them. Lexical path checks alone are
/// insufficient: a tracked symlink could otherwise turn a trusted-looking
/// catalog entry into a HOME, cache, archive, or legacy-hook fallback.
fn validate_owner_source(project_root: &Path, name: &str, skill: &Skill) -> Result<()> {
    let source = source_for(name, skill)?;
    if source.starts_with("https://") {
        if source != "https://github.com/FlexNetOS/meta"
            || skill.source_ref.as_deref() != Some("fb7273a7c8d05dce0bac649ded940a86ad41e107")
            || skill.sub_dir.as_deref() != Some("agent-env/skills")
        {
            return Err(err(format!(
                "untrusted or unpinned remote catalog source for `{name}`: {source}"
            )));
        }
        return Ok(());
    }
    if Path::new(source.trim_start_matches("./"))
        .file_name()
        .is_none_or(|terminal| terminal != name)
    {
        return Err(err(format!(
            "catalog source for `{name}` must end in the declared skill name: {source}"
        )));
    }
    let root = project_root.join(source.trim_start_matches("./"));
    let allowed_relative = if source == "./agent-skills/skill-catalog" {
        "agent-skills/skill-catalog"
    } else if source.starts_with("./agent-skills/") {
        "agent-skills"
    } else {
        ".kb/skills"
    };
    let allowed = project_root
        .join(allowed_relative)
        .canonicalize()
        .map_err(|e| {
            err(format!(
                "trusted catalog root {allowed_relative} is unavailable: {e}"
            ))
        })?;
    let canonical_root = root.canonicalize().map_err(|e| {
        err(format!(
            "catalog skill `{name}` has no canonical owner directory at {}: {e}",
            root.display()
        ))
    })?;
    if !canonical_root.starts_with(&allowed) {
        return Err(err(format!(
            "catalog skill `{name}` resolves outside trusted owner root {allowed_relative}"
        )));
    }
    let skill_file = root.join("SKILL.md");
    let canonical_skill = skill_file.canonicalize().map_err(|e| {
        err(format!(
            "catalog skill `{name}` has no canonical SKILL.md at {}: {e}",
            skill_file.display()
        ))
    })?;
    if !canonical_skill.starts_with(&canonical_root) {
        return Err(err(format!(
            "catalog skill `{name}` SKILL.md resolves outside its canonical owner directory"
        )));
    }
    Ok(())
}

fn source_for(name: &str, skill: &Skill) -> Result<String> {
    let source = skill.source.clone().unwrap_or_else(|| {
        if name == "skill-catalog" {
            "./agent-skills/skill-catalog".into()
        } else {
            format!("./agent-skills/{name}")
        }
    });
    if source.starts_with("https://") {
        return Ok(source);
    }
    let normalized = source.strip_prefix("./").ok_or_else(|| {
        err(format!(
            "catalog source for `{name}` must be project-relative: {source}"
        ))
    })?;
    let path = Path::new(normalized);
    let trusted = normalized == "agent-skills/skill-catalog"
        || normalized.starts_with("agent-skills/")
        || normalized.starts_with(".kb/skills/");
    if !trusted
        || path
            .components()
            .any(|component| !matches!(component, Component::Normal(_)))
    {
        return Err(err(format!(
            "untrusted catalog source for `{name}`: {source}"
        )));
    }
    Ok(source)
}

fn validate_name(kind: &str, value: &str) -> Result<()> {
    let path = Path::new(value);
    if value.is_empty()
        || path.is_absolute()
        || value.contains('\\')
        || value.contains('\0')
        || path
            .components()
            .any(|c| !matches!(c, Component::Normal(_)))
    {
        return Err(err(format!("invalid {kind} `{value}`")));
    }
    Ok(())
}

fn claim_unique(
    seen: &mut HashMap<String, String>,
    value: &str,
    owner: &str,
    kind: &str,
) -> Result<()> {
    validate_name(kind, value)?;
    if let Some(existing) = seen.insert(value.to_string(), owner.to_string()) {
        return Err(err(format!(
            "ambiguous {kind} `{value}` belongs to both `{existing}` and `{owner}`"
        )));
    }
    Ok(())
}

fn exactly_one(kind: &str, value: &str, matches: Vec<String>) -> Result<String> {
    match matches.as_slice() {
        [one] => Ok(one.clone()),
        [] => Err(err(format!("no catalog {kind} matches `{value}`"))),
        _ => Err(err(format!(
            "ambiguous catalog {kind} `{value}`: {}",
            matches.join(", ")
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicU64, Ordering};

    static TEST_ROOT_SEQUENCE: AtomicU64 = AtomicU64::new(0);

    struct TestRoot(PathBuf);

    impl TestRoot {
        fn new() -> Self {
            let path = std::env::temp_dir().join(format!(
                "envctl-skill-catalog-test-{}-{}",
                std::process::id(),
                TEST_ROOT_SEQUENCE.fetch_add(1, Ordering::Relaxed),
            ));
            fs::create_dir_all(&path).unwrap();
            Self(path)
        }

        fn path(&self) -> &Path {
            &self.0
        }
    }

    impl Drop for TestRoot {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    fn catalog_with_sources() -> (TestRoot, Catalog) {
        let root = TestRoot::new();
        for name in ["a", "b", "skill-catalog"] {
            let dir = root.path().join("agent-skills").join(name);
            fs::create_dir_all(&dir).unwrap();
            fs::write(dir.join("SKILL.md"), "---\nname: test\n---\n").unwrap();
        }
        (
            root,
            Catalog {
                version: 1,
                active_packs: vec!["core".into()],
                packs: BTreeMap::from([(
                    "core".into(),
                    Pack {
                        description: "x".into(),
                        skills: vec!["a".into(), "skill-catalog".into()],
                    },
                )]),
                skills: BTreeMap::from([
                    (
                        "a".into(),
                        Skill {
                            source: None,
                            source_ref: None,
                            sub_dir: None,
                            aliases: vec!["alpha".into()],
                            intents: vec!["implement-a".into()],
                            keywords: vec!["rust".into()],
                        },
                    ),
                    (
                        "b".into(),
                        Skill {
                            source: None,
                            source_ref: None,
                            sub_dir: None,
                            aliases: vec!["beta".into()],
                            intents: vec!["implement-b".into()],
                            keywords: vec!["planning".into()],
                        },
                    ),
                    (
                        "skill-catalog".into(),
                        Skill {
                            source: None,
                            source_ref: None,
                            sub_dir: None,
                            aliases: Vec::new(),
                            intents: Vec::new(),
                            keywords: Vec::new(),
                        },
                    ),
                ]),
            },
        )
    }

    #[test]
    fn catalog_validates_owner_sources_and_resolves_compact_metadata() {
        let (root, mut catalog) = catalog_with_sources();
        catalog.validate(root.path()).unwrap();
        assert_eq!(catalog.resolve_skill("alpha").unwrap(), "a");
        assert_eq!(catalog.resolve_intent("implement-a").unwrap(), "a");
        assert_eq!(catalog.search("rust").unwrap()[0].name, "a");
        assert_eq!(catalog.pack_inventory()[0].name, "core");
        assert_eq!(
            catalog.active_skills().unwrap(),
            BTreeSet::from(["a".into(), "skill-catalog".into()])
        );

        catalog.skills.get_mut("b").unwrap().aliases = vec!["alpha".into()];
        assert!(catalog
            .validate(root.path())
            .unwrap_err()
            .to_string()
            .contains("ambiguous alias"));
    }

    #[test]
    fn catalog_rejects_missing_owner_and_unsafe_keyword() {
        let (root, mut catalog) = catalog_with_sources();
        fs::remove_file(root.path().join("agent-skills/a/SKILL.md")).unwrap();
        assert!(catalog
            .validate(root.path())
            .unwrap_err()
            .to_string()
            .contains("no canonical SKILL.md"));

        fs::write(
            root.path().join("agent-skills/a/SKILL.md"),
            "---\nname: test\n---\n",
        )
        .unwrap();
        catalog.skills.get_mut("a").unwrap().keywords = vec!["../unsafe".into()];
        assert!(catalog
            .validate(root.path())
            .unwrap_err()
            .to_string()
            .contains("invalid catalog keyword"));
    }

    #[test]
    fn catalog_rejects_core_without_catalog_helper_and_symlink_escapes() {
        let (root, mut catalog) = catalog_with_sources();
        catalog.packs.get_mut(CORE_PACK).unwrap().skills = vec!["a".into()];
        assert!(catalog
            .validate(root.path())
            .unwrap_err()
            .to_string()
            .contains("must materialize the skill-catalog helper"));

        catalog
            .packs
            .get_mut(CORE_PACK)
            .unwrap()
            .skills
            .push("skill-catalog".into());
        let outside = root.path().join("outside");
        fs::create_dir_all(&outside).unwrap();
        fs::write(outside.join("SKILL.md"), "---\nname: outside\n---\n").unwrap();
        let source = root.path().join("agent-skills/a");
        fs::remove_dir_all(&source).unwrap();
        #[cfg(unix)]
        std::os::unix::fs::symlink(&outside, &source).unwrap();
        #[cfg(unix)]
        assert!(catalog
            .validate(root.path())
            .unwrap_err()
            .to_string()
            .contains("outside trusted owner root"));
    }

    #[test]
    fn catalog_rejects_source_for_a_different_skill_name() {
        let (root, mut catalog) = catalog_with_sources();
        catalog.skills.get_mut("a").unwrap().source = Some("./agent-skills/b".into());
        assert!(catalog
            .validate(root.path())
            .unwrap_err()
            .to_string()
            .contains("must end in the declared skill name"));
    }
}
