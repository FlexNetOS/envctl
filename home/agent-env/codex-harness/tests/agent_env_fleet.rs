#![forbid(unsafe_code)]

use serde_json::{json, Value};
use std::fs;
#[cfg(unix)]
use std::os::unix::fs::symlink;
use std::path::{Path, PathBuf};
use std::process::Command;

fn envctl_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(3)
        .expect("harness crate path has envctl ancestor")
        .to_path_buf()
}

fn fixture() -> (tempfile::TempDir, PathBuf, PathBuf, PathBuf, PathBuf) {
    let temp = tempfile::tempdir().unwrap();
    let meta = temp.path().join("meta");
    fs::create_dir_all(meta.join("src/independent")).unwrap();
    fs::create_dir_all(meta.join("src/inherited")).unwrap();
    fs::write(
        meta.join("src/independent/agent-env.yaml"),
        "scope: project\n",
    )
    .unwrap();
    fs::write(meta.join("src/independent/agent-env.lock"), "version: 1\n").unwrap();
    let projects = temp.path().join("projects.json");
    fs::write(
        &projects,
        serde_json::to_vec(&json!({
            "repo": "meta",
            "projects": [
                {"name": "independent", "path": "src/independent"},
                {"name": "inherited", "path": "src/inherited"}
            ]
        }))
        .unwrap(),
    )
    .unwrap();

    let home = temp.path().join("home");
    let active_codex = home.join(".codex");
    fs::create_dir_all(&active_codex).unwrap();
    fs::write(
        active_codex.join("model-catalog.json"),
        serde_json::to_vec(&json!({
            "models": [{
                "slug": "tencent/hy3:free",
                "visibility": "list",
                "provider": "openrouter",
                "context_window": 262144,
                "free_route_expires_on": "2026-07-21"
            }]
        }))
        .unwrap(),
    )
    .unwrap();
    let profile = r#"model = "tencent/hy3:free"
model_provider = "openrouter"

[model_providers.openrouter]
base_url = "https://openrouter.ai/api/v1"
env_key = "OPENROUTER_API_KEY"
wire_api = "responses"
"#;
    fs::write(active_codex.join("envctl-openrouter.config.toml"), profile).unwrap();
    fs::write(
        active_codex.join("envctl-openrouter-gpt.config.toml"),
        profile,
    )
    .unwrap();
    let codex_bin = home.join(".nix-profile/bin/codex");
    fs::create_dir_all(codex_bin.parent().unwrap()).unwrap();
    #[cfg(unix)]
    symlink("/home/flexnetos/.nix-profile/bin/codex", &codex_bin).unwrap();

    (temp, meta, projects, active_codex, codex_bin)
}

fn verifier(meta: &Path, projects: &Path, active_codex: &Path, codex_bin: &Path) -> Command {
    let mut command = Command::new("/home/flexnetos/.nix-profile/toolbin/nu");
    command
        .arg("--no-config-file")
        .arg(envctl_root().join("scripts/verify-agent-env-fleet.nu"))
        .args(["--meta-root", meta.to_str().unwrap()])
        .args(["--project-list-json", projects.to_str().unwrap()])
        .args(["--active-codex-root", active_codex.to_str().unwrap()])
        .args(["--codex-bin", codex_bin.to_str().unwrap()])
        .arg("--json");
    command
}

#[test]
fn nushell_fleet_verifier_defaults_to_clean_nu_home_and_profile() {
    let (_temp, meta, projects, active_codex, _codex_bin) = fixture();
    let home = active_codex.parent().unwrap();
    let output = Command::new("/home/flexnetos/.nix-profile/toolbin/nu")
        .arg("--no-config-file")
        .arg(envctl_root().join("scripts/verify-agent-env-fleet.nu"))
        .args(["--meta-root", meta.to_str().unwrap()])
        .args(["--project-list-json", projects.to_str().unwrap()])
        .arg("--json")
        .env("HOME", home)
        .env_remove("ENVCTL_REAL_HOME")
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "stdout={}\nstderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let report: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(report["central_runtime"]["status"], "ok");
    assert_eq!(
        report["central_runtime"]["codex"]["profile_frontdoor"],
        true
    );
}

#[test]
fn nushell_fleet_verifier_classifies_independent_and_central_repos_read_only() {
    let (_temp, meta, projects, active_codex, codex_bin) = fixture();

    let output = verifier(&meta, &projects, &active_codex, &codex_bin)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "stdout={}\nstderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let report: Value = serde_json::from_slice(&output.stdout).unwrap();
    let repos = report["repos"].as_array().unwrap();
    assert_eq!(repos.len(), 3, "Meta root plus every declared project");
    let ownership = |name: &str| {
        repos.iter().find(|repo| repo["name"] == name).unwrap()["ownership"]
            .as_str()
            .unwrap()
    };
    assert_eq!(ownership("independent"), "independent");
    assert_eq!(ownership("inherited"), "central-inherited");
    assert_eq!(report["execution_requested"], false);
    assert_eq!(report["central_runtime"]["status"], "ok");
    assert_eq!(report["sync_verified"], false);
    assert!(repos
        .iter()
        .all(|repo| repo["preview"]["status"] == "not-requested"));
    assert!(repos
        .iter()
        .all(|repo| repo["audit"]["status"] == "not-requested"));
}

#[test]
fn nushell_fleet_verifier_executes_only_explicit_read_only_actions() {
    let (_temp, meta, projects, active_codex, codex_bin) = fixture();
    let output = verifier(&meta, &projects, &active_codex, &codex_bin)
        .args(["--envctl-bin", "/usr/bin/true"])
        .args(["--execute-preview", "--execute-audit"])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "stdout={}\nstderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let report: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(report["execution_requested"], true);
    assert_eq!(report["sync_verified"], true);
    let independent = report["repos"]
        .as_array()
        .unwrap()
        .iter()
        .find(|repo| repo["name"] == "independent")
        .unwrap();
    assert_eq!(independent["preview"]["status"], "ok");
    assert_eq!(independent["audit"]["status"], "ok");
    assert!(report["repos"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|repo| repo["ownership"] == "central-inherited")
        .all(|repo| repo["preview"]["status"] == "not-requested"
            && repo["audit"]["status"] == "not-requested"));
}

#[test]
fn nushell_fleet_verifier_fails_closed_without_canonical_envctl_engine() {
    let (_temp, meta, projects, active_codex, codex_bin) = fixture();
    let output = verifier(&meta, &projects, &active_codex, &codex_bin)
        .arg("--execute-preview")
        .output()
        .unwrap();
    assert!(!output.status.success());
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("canonical envctl engine is unavailable")
    );
}

#[test]
fn nushell_fleet_verifier_rejects_unproven_central_runtime() {
    let (_temp, meta, projects, active_codex, codex_bin) = fixture();
    fs::remove_file(active_codex.join("model-catalog.json")).unwrap();

    let output = verifier(&meta, &projects, &active_codex, &codex_bin)
        .output()
        .unwrap();
    assert!(!output.status.success());
    let report: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(report["ok"], false);
    assert_eq!(report["central_runtime"]["status"], "failed");
    assert_eq!(report["central_runtime"]["catalog"]["exists"], false);
    assert_eq!(report["sync_verified"], false);
}

#[test]
fn nushell_fleet_verifier_has_no_mutating_or_fallback_shell_surface() {
    let script = fs::read_to_string(envctl_root().join("scripts/verify-agent-env-fleet.nu"))
        .expect("tracked fleet verifier");
    for forbidden in ["--apply", "python", "node", "bash", "sh -c"] {
        assert!(
            !script.to_ascii_lowercase().contains(forbidden),
            "fleet verifier contains forbidden surface: {forbidden}"
        );
    }
    assert!(script.contains("--execute-preview"));
    assert!(script.contains("--execute-audit"));
    assert!(script.contains("--active-codex-root"));
    assert!(script.contains("--codex-bin"));
    assert!(script.contains("sync_verified"));
    assert!(script.contains("^$envctl_bin agent sync --config $config --scope project --json"));
    assert!(script.contains("^$envctl_bin agent audit --config $config --scope project --json"));
    assert!(script.contains("usr/libexec/envctl/cli/bin/envctl"));
    assert!(!script.contains("agent lock"));
}
