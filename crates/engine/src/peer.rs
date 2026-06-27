//! PEER registration for `add-repo` — the meta-native path.
//!
//! meta is a meta-repo of co-equal **peer** repos declared in the meta-root
//! `.meta.yaml` (`projects:` block) and ignored in the root `.gitignore`, each
//! cloned as a sibling directory of the workspace root. This module registers an
//! added repo *that way* instead of as a private build-from-source managed
//! component (see [`crate::register`]).
//!
//! It is the counterpart to the component path: where `register::synth_dropin`
//! writes `manifest/components.d/<id>.toml`, this module performs a **grep-guarded,
//! idempotent** edit of the meta-root `.meta.yaml` + `.gitignore` and clones the
//! repo as a meta sibling. meta has no mutating `project add` verb by design
//! (KB `patterns.md`; ADR-0001 §6 "grep-guard file edits, never blind append"),
//! so envctl owns this seam — fail-closed and dry-run-by-default, exactly like the
//! rest of add-repo.

use crate::event::{Event, EventSink, Stream};
use crate::layout::MetaLayout;
use crate::model::{AddRepoSpec, RunSummary};
use std::path::{Path, PathBuf};

/// GitHub orgs whose repos auto-route to PEER registration. A repo under one of
/// these is a first-class workspace member, not a third-party tool.
const OWNED_ORGS: &[&str] = &["FlexNetOS"];

/// `(host, org, repo)` parsed from a GitHub remote in either `https://` or
/// `git@host:org/repo(.git)` form. Returns `None` for anything we don't recognize
/// (which then defaults to the component path under `Auto`).
pub fn parse_github_remote(url: &str) -> Option<(String, String, String)> {
    let u = url.trim();
    // git@github.com:Org/Repo(.git)
    if let Some(rest) = u.strip_prefix("git@") {
        let (host, path) = rest.split_once(':')?;
        let (org, repo) = path.split_once('/')?;
        return Some((host.to_string(), org.to_string(), strip_git(repo)));
    }
    // https://github.com/Org/Repo(.git) or http://, or ssh://git@host/Org/Repo
    for scheme in ["https://", "http://", "ssh://"] {
        if let Some(rest) = u.strip_prefix(scheme) {
            let rest = rest.strip_prefix("git@").unwrap_or(rest);
            let (host, path) = rest.split_once('/')?;
            let (org, repo) = path.split_once('/')?;
            // repo is the first path segment after org; drop any trailing path.
            let repo = repo.split('/').next().unwrap_or(repo);
            return Some((host.to_string(), org.to_string(), strip_git(repo)));
        }
    }
    None
}

fn strip_git(repo: &str) -> String {
    repo.strip_suffix(".git").unwrap_or(repo).to_string()
}

/// True when the remote belongs to an owned org (→ auto-route to PEER).
pub fn is_owned_remote(url: &str) -> bool {
    matches!(parse_github_remote(url), Some((host, org, _))
        if host.eq_ignore_ascii_case("github.com")
            && OWNED_ORGS.iter().any(|o| o.eq_ignore_ascii_case(&org)))
}

/// Canonical `git@github.com:Org/Repo.git` form for the `.meta.yaml` `repo:` field
/// (the convention every existing peer uses), regardless of the input scheme.
/// Falls back to the input URL unchanged for non-GitHub remotes.
pub fn canonical_repo_url(url: &str) -> String {
    match parse_github_remote(url) {
        Some((host, org, repo)) => format!("git@{host}:{org}/{repo}.git"),
        None => url.trim().to_string(),
    }
}

/// The `.meta.yaml` project entry block for a peer (2-space-indented, matching the
/// existing file style). `provides`/`tags` keys are omitted when empty.
pub fn render_meta_entry(id: &str, repo_url: &str, provides: &[String], tags: &[String]) -> String {
    let mut s = format!("  {id}:\n    repo: {repo_url}\n");
    if !provides.is_empty() {
        s.push_str(&format!("    provides: [{}]\n", provides.join(", ")));
    }
    if !tags.is_empty() {
        s.push_str(&format!("    tags: [{}]\n", tags.join(", ")));
    }
    s
}

/// Is `id` already declared as a project in this `.meta.yaml` text? Grep-guard on
/// the exact 2-space-indented `  <id>:` key so we never double-register.
pub fn meta_has_project(meta_yaml: &str, id: &str) -> bool {
    let needle = format!("  {id}:");
    meta_yaml.lines().any(|l| l.trim_end() == needle)
}

/// Is `<id>/` already present (uncommented) in this `.gitignore` text?
pub fn gitignore_has(gitignore: &str, id: &str) -> bool {
    let needle = format!("{id}/");
    gitignore
        .lines()
        .map(str::trim)
        .any(|l| l == needle || l == format!("/{id}/"))
}

/// What a peer registration WOULD/DID do — used for both the dry-run preview and
/// the apply log so the two paths describe identical intent.
pub struct PeerPlan {
    pub id: String,
    pub repo_url: String,
    pub meta_file: PathBuf,
    pub gitignore: PathBuf,
    pub clone_target: PathBuf,
    pub meta_entry: String,
    pub need_meta_edit: bool,
    pub need_gitignore_edit: bool,
    pub need_clone: bool,
}

/// Resolve the meta-root and build the (idempotent) plan. Fail-closed: refuses if
/// the meta-root `.meta.yaml` marker can't be found (no blind workspace creation).
pub fn plan_peer(spec: &AddRepoSpec) -> anyhow::Result<PeerPlan> {
    let id = spec.id.trim().to_string();
    if spec.git_url.trim().is_empty() {
        anyhow::bail!(
            "peer mode needs a git remote URL — a local-only working tree can only be a managed component (use --mode component)"
        );
    }
    let meta_root = MetaLayout::from_env_or_default().meta_root().to_path_buf();
    let meta_file = meta_root.join(".meta.yaml");
    if !meta_file.is_file() {
        anyhow::bail!(
            "cannot register a peer: no .meta.yaml at meta-root '{}' (set META_ROOT to the workspace that holds .meta.yaml)",
            meta_root.display()
        );
    }
    let gitignore = meta_root.join(".gitignore");
    let clone_target = meta_root.join(&id);
    let repo_url = canonical_repo_url(&spec.git_url);
    let meta_entry = render_meta_entry(&id, &repo_url, &spec.provides, &spec.tags);

    let meta_txt = std::fs::read_to_string(&meta_file).unwrap_or_default();
    let gi_txt = std::fs::read_to_string(&gitignore).unwrap_or_default();

    Ok(PeerPlan {
        need_meta_edit: !meta_has_project(&meta_txt, &id),
        need_gitignore_edit: !gitignore_has(&gi_txt, &id),
        need_clone: !clone_target.exists(),
        id,
        repo_url,
        meta_file,
        gitignore,
        clone_target,
        meta_entry,
    })
}

/// Register an added repo as a first-class meta peer.
///
/// Preview (dry-run OR no `--build`): emit the exact `.meta.yaml`/`.gitignore`
/// edits + clone target, mutate nothing. Apply (`--build`): grep-guarded-insert the
/// project entry after the `projects:` line, grep-guarded-append `<id>/` to
/// `.gitignore`, then clone the repo as a meta sibling. The declaration is the
/// source of truth — a clone failure is reported but leaves the peer declared (a
/// later `meta git update` materializes it).
pub fn register_peer(
    spec: &AddRepoSpec,
    dry_run: bool,
    sink: &EventSink,
) -> anyhow::Result<RunSummary> {
    let plan = plan_peer(spec)?;
    let mut summary = RunSummary::default();
    let id = plan.id.clone();

    let preview = dry_run || !spec.allow_build;
    if preview {
        let mut msg = format!(
            "[preview] would register peer '{id}' (meta-native, NOT a managed component):\n"
        );
        if plan.need_meta_edit {
            msg.push_str(&format!(
                "  + {} (insert after `projects:`):\n{}",
                plan.meta_file.display(),
                indent(&plan.meta_entry, "      ")
            ));
        } else {
            msg.push_str(&format!(
                "  = {} already declares '{id}' (no change)\n",
                plan.meta_file.display()
            ));
        }
        if plan.need_gitignore_edit {
            msg.push_str(&format!("  + {}: {id}/\n", plan.gitignore.display()));
        } else {
            msg.push_str(&format!(
                "  = {} already ignores '{id}/' (no change)\n",
                plan.gitignore.display()
            ));
        }
        if plan.need_clone {
            msg.push_str(&format!(
                "  + clone {} -> {} (or run `meta git update`)\n",
                plan.repo_url,
                plan.clone_target.display()
            ));
        } else {
            msg.push_str(&format!(
                "  = {} already cloned (no change)\n",
                plan.clone_target.display()
            ));
        }
        sink.emit(Event::Log {
            component: id,
            stream: Stream::Stdout,
            line: msg,
        });
        return Ok(summary);
    }

    // APPLY (idempotent, grep-guarded).
    if plan.need_meta_edit {
        insert_meta_project(&plan.meta_file, &plan.meta_entry)?;
        log(
            sink,
            &id,
            format!("declared peer '{id}' in {}", plan.meta_file.display()),
        );
    }
    if plan.need_gitignore_edit {
        append_gitignore(&plan.gitignore, &id)?;
        log(
            sink,
            &id,
            format!("ignored '{id}/' in {}", plan.gitignore.display()),
        );
    }
    if plan.need_clone {
        match clone_sibling(&plan.repo_url, spec.git_ref.as_deref(), &plan.clone_target) {
            Ok(()) => log(
                sink,
                &id,
                format!("cloned -> {}", plan.clone_target.display()),
            ),
            Err(e) => {
                summary.failed.push(format!("{id}/clone"));
                sink.emit(Event::Log {
                    component: id.clone(),
                    stream: Stream::Stderr,
                    line: format!(
                        "peer '{id}' is declared in .meta.yaml but the sibling clone failed ({e}); run `meta git update` to materialize it"
                    ),
                });
            }
        }
    }

    log(
        sink,
        &id,
        format!("registered peer '{id}'. Verify: `meta project list -r` · sync: `meta git update`"),
    );
    sink.emit(Event::RunFinished {
        summary: summary.clone(),
    });
    Ok(summary)
}

/// Insert `entry` immediately after the first top-level `projects:` line. This is
/// deterministic regardless of what follows the block (no dependence on EOF state).
fn insert_meta_project(meta_file: &Path, entry: &str) -> anyhow::Result<()> {
    let txt = std::fs::read_to_string(meta_file)?;
    let mut out = String::with_capacity(txt.len() + entry.len());
    let mut inserted = false;
    for line in txt.lines() {
        out.push_str(line);
        out.push('\n');
        if !inserted && line.trim_end() == "projects:" {
            out.push_str(entry);
            inserted = true;
        }
    }
    if !inserted {
        anyhow::bail!(
            "no top-level `projects:` block in {} — refusing to blind-append (ADR-0001 §6)",
            meta_file.display()
        );
    }
    atomic_write(meta_file, &out)
}

fn append_gitignore(gitignore: &Path, id: &str) -> anyhow::Result<()> {
    let mut txt = std::fs::read_to_string(gitignore).unwrap_or_default();
    if !txt.is_empty() && !txt.ends_with('\n') {
        txt.push('\n');
    }
    txt.push_str(&format!("{id}/\n"));
    atomic_write(gitignore, &txt)
}

/// Clone the repo as a meta sibling. Uses `--` to defuse option-injection (the URL
/// is already leading-dash-guarded by `validate_add_repo_spec`); honors an optional
/// branch/tag. No shell.
fn clone_sibling(url: &str, git_ref: Option<&str>, target: &Path) -> anyhow::Result<()> {
    let mut cmd = std::process::Command::new("git");
    cmd.arg("clone");
    if let Some(r) = git_ref.filter(|r| !r.is_empty()) {
        cmd.args(["--branch", r]);
    }
    cmd.arg("--").arg(url).arg(target);
    let status = cmd.status()?;
    if !status.success() {
        anyhow::bail!("git clone exited with {status}");
    }
    Ok(())
}

fn atomic_write(path: &Path, contents: &str) -> anyhow::Result<()> {
    let tmp = path.with_extension(format!(
        "{}.envctl-tmp",
        path.extension().and_then(|e| e.to_str()).unwrap_or("")
    ));
    std::fs::write(&tmp, contents)?;
    std::fs::rename(&tmp, path)?;
    Ok(())
}

fn indent(s: &str, pad: &str) -> String {
    s.lines().map(|l| format!("{pad}{l}\n")).collect::<String>()
}

fn log(sink: &EventSink, id: &str, line: String) {
    sink.emit(Event::Log {
        component: id.to_string(),
        stream: Stream::Stdout,
        line,
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_https_and_ssh() {
        assert_eq!(
            parse_github_remote("https://github.com/FlexNetOS/beads_rust"),
            Some(("github.com".into(), "FlexNetOS".into(), "beads_rust".into()))
        );
        assert_eq!(
            parse_github_remote("git@github.com:FlexNetOS/beads_rust.git"),
            Some(("github.com".into(), "FlexNetOS".into(), "beads_rust".into()))
        );
        assert_eq!(parse_github_remote("not a url"), None);
    }

    #[test]
    fn owned_routing() {
        assert!(is_owned_remote("https://github.com/FlexNetOS/beads_rust"));
        assert!(is_owned_remote("git@github.com:FlexNetOS/x.git"));
        assert!(!is_owned_remote("https://github.com/someone-else/tool"));
        assert!(!is_owned_remote("https://gitlab.com/FlexNetOS/x"));
    }

    #[test]
    fn canonicalizes_to_ssh() {
        assert_eq!(
            canonical_repo_url("https://github.com/FlexNetOS/beads_rust"),
            "git@github.com:FlexNetOS/beads_rust.git"
        );
        // non-github passes through unchanged
        assert_eq!(
            canonical_repo_url("https://example.com/x"),
            "https://example.com/x"
        );
    }

    #[test]
    fn meta_entry_omits_empty_keys() {
        let e = render_meta_entry(
            "beads_rust",
            "git@github.com:FlexNetOS/beads_rust.git",
            &[],
            &[],
        );
        assert_eq!(
            e,
            "  beads_rust:\n    repo: git@github.com:FlexNetOS/beads_rust.git\n"
        );
        let e2 = render_meta_entry(
            "x",
            "git@github.com:FlexNetOS/x.git",
            &["x".to_string()],
            &["tools".to_string(), "env".to_string()],
        );
        assert!(e2.contains("    provides: [x]\n"));
        assert!(e2.contains("    tags: [tools, env]\n"));
    }

    #[test]
    fn grep_guards_detect_existing() {
        let yaml = "defaults:\n  parallel: true\nprojects:\n  envctl:\n    repo: x\n  agent:\n    repo: y\n";
        assert!(meta_has_project(yaml, "envctl"));
        assert!(meta_has_project(yaml, "agent"));
        assert!(!meta_has_project(yaml, "beads_rust"));
        // substring must not false-match
        assert!(!meta_has_project(yaml, "env"));

        let gi = "target/\nenvctl/\n# comment\n/dist/\n";
        assert!(gitignore_has(gi, "envctl"));
        assert!(!gitignore_has(gi, "beads_rust"));
        assert!(!gitignore_has(gi, "env"));
    }

    #[test]
    fn insert_after_projects_is_idempotent_by_caller() {
        let dir = std::env::temp_dir().join(format!("envctl-peer-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let mf = dir.join(".meta.yaml");
        std::fs::write(
            &mf,
            "defaults:\n  parallel: true\nprojects:\n  envctl:\n    repo: x\n",
        )
        .unwrap();
        let entry = render_meta_entry(
            "beads_rust",
            "git@github.com:FlexNetOS/beads_rust.git",
            &[],
            &[],
        );
        insert_meta_project(&mf, &entry).unwrap();
        let after = std::fs::read_to_string(&mf).unwrap();
        assert!(meta_has_project(&after, "beads_rust"));
        assert!(meta_has_project(&after, "envctl"));
        // entry landed right after the projects: line
        let pos_projects = after.find("projects:\n").unwrap();
        let pos_beads = after.find("  beads_rust:").unwrap();
        let pos_envctl = after.find("  envctl:").unwrap();
        assert!(pos_projects < pos_beads && pos_beads < pos_envctl);
        std::fs::remove_dir_all(&dir).ok();
    }
}
