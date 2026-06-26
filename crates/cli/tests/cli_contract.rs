//! End-to-end CLI contract tests for the autonomous TDD loop.
//!
//! These tests drive the real `envctl` binary through hermetic fixtures and pin the
//! machine-facing behavior the loop uses to decide whether a CLI gap is red/green:
//! root surface discovery, manifest-independent commands, fail-closed refusal exit
//! codes, JSON shape contracts, secret wrapper argv, dry-run no-write behavior, and lock-check JSON truth.
use std::path::{Path, PathBuf};
use std::process::Command;

fn bin() -> &'static str {
    env!("CARGO_BIN_EXE_envctl")
}

struct Fixture {
    root: PathBuf,
    manifest: PathBuf,
    meta: PathBuf,
    xdg_config: PathBuf,
    xdg_data: PathBuf,
    xdg_cache: PathBuf,
    home: PathBuf,
}

impl Fixture {
    fn new() -> Self {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        use std::sync::atomic::{AtomicU64, Ordering};
        static SEQ: AtomicU64 = AtomicU64::new(0);
        let seq = SEQ.fetch_add(1, Ordering::Relaxed);
        let root = std::env::temp_dir().join(format!("envctl-cli-contract-{nanos}-{seq}"));
        let manifest = root.join("manifest");
        let meta = root.join("meta");
        let xdg_config = root.join("xdg-config");
        let xdg_data = root.join("xdg-data");
        let xdg_cache = root.join("xdg-cache");
        let home = root.join("home");
        std::fs::create_dir_all(&manifest).unwrap();
        std::fs::create_dir_all(&meta).unwrap();
        std::fs::create_dir_all(&home).unwrap();
        std::fs::write(
            manifest.join("base.toml"),
            r#"
[[component]]
id = "stub"
name = "Stub Component"

[component.detect]
kind = "command"
command = "true"

[component.install]
kind = "command"
command = "sh"
args = ["-c", "printf install > would-write"]

[component.fix]
kind = "command"
command = "sh"
args = ["-c", "printf fix > would-write"]

[component.remove]
kind = "command"
command = "sh"
args = ["-c", "printf remove > would-write"]
"#,
        )
        .unwrap();
        let hub = root.join("mcp_hub");
        std::fs::create_dir_all(&hub).unwrap();
        std::fs::write(
            hub.join("registry.json"),
            r#"{
  "schema": "hub.registry.v1",
  "entries": [
    {
      "id": "stub-tool",
      "name": "Stub Tool",
      "description": "Hermetic registry fixture",
      "component": "stub",
      "status": "stable",
      "tier": 1
    }
  ]
}
"#,
        )
        .unwrap();
        std::fs::write(
            meta.join(".meta.yaml"),
            "projects:\n  envctl:\n    repo: git@github.com:FlexNetOS/envctl.git\n",
        )
        .unwrap();
        Self {
            root,
            manifest,
            meta,
            xdg_config,
            xdg_data,
            xdg_cache,
            home,
        }
    }

    fn cmd(&self) -> Command {
        let mut c = Command::new(bin());
        c.current_dir(&self.root)
            .env("ENVCTL_MANIFEST_DIR", &self.manifest)
            .env("XDG_CONFIG_HOME", &self.xdg_config)
            .env("XDG_DATA_HOME", &self.xdg_data)
            .env("XDG_CACHE_HOME", &self.xdg_cache)
            .env("HOME", &self.home);
        c
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        std::fs::remove_dir_all(&self.root).ok();
    }
}

fn stdout(out: &std::process::Output) -> String {
    String::from_utf8_lossy(&out.stdout).into_owned()
}

fn stderr(out: &std::process::Output) -> String {
    String::from_utf8_lossy(&out.stderr).into_owned()
}

#[test]
fn root_help_lists_the_full_cli_surface() {
    let out = Command::new(bin()).arg("--help").output().unwrap();
    assert!(out.status.success(), "stderr: {}", stderr(&out));
    let help = stdout(&out);
    for verb in [
        "auto-detect",
        "graph",
        "registry",
        "lock",
        "doctor",
        "install",
        "reset",
        "auto-fix",
        "add-repo",
        "dashboard",
        "env",
        "agent",
        "self",
        "completions",
        "secret",
    ] {
        assert!(help.contains(verb), "root --help missing `{verb}`:\n{help}");
    }
}

#[test]
fn manifest_independent_commands_work_without_a_manifest() {
    let fx = Fixture::new();
    let missing_manifest = fx.root.join("missing-manifest");

    let env_out = Command::new(bin())
        .current_dir(&fx.root)
        .env("ENVCTL_MANIFEST_DIR", &missing_manifest)
        .args(["env", "--meta-file"])
        .arg(fx.meta.join(".meta.yaml"))
        .output()
        .unwrap();
    assert!(env_out.status.success(), "stderr: {}", stderr(&env_out));
    assert!(stdout(&env_out).contains("META_ROOT"));

    let dashboard_out = Command::new(bin())
        .current_dir(&fx.root)
        .env("ENVCTL_MANIFEST_DIR", &missing_manifest)
        .args(["dashboard", "--meta-file"])
        .arg(fx.meta.join(".meta.yaml"))
        .output()
        .unwrap();
    assert!(
        dashboard_out.status.success(),
        "stderr: {}",
        stderr(&dashboard_out)
    );
    assert!(stdout(&dashboard_out).contains("Generated by envctl dashboard"));
}

#[test]
fn reset_whole_roster_refuses_before_worker_and_writes_nothing() {
    let fx = Fixture::new();
    let before = tree_snapshot(&fx.root);
    let out = fx.cmd().arg("reset").output().unwrap();
    assert!(
        !out.status.success(),
        "reset without --all --confirm must fail"
    );
    assert_eq!(out.status.code(), Some(2));
    assert!(stderr(&out).contains("refusing whole-roster reset"));
    assert_eq!(
        before,
        tree_snapshot(&fx.root),
        "reset refusal mutated fixture"
    );
}

#[test]
fn lock_check_json_reports_clean_lock_as_locked_and_drift_empty() {
    let fx = Fixture::new();
    let lock = fx.cmd().arg("lock").output().unwrap();
    assert!(lock.status.success(), "stderr: {}", stderr(&lock));

    let check = fx
        .cmd()
        .args(["lock", "--check", "--json"])
        .output()
        .unwrap();
    assert!(check.status.success(), "stderr: {}", stderr(&check));
    let v: serde_json::Value = serde_json::from_slice(&check.stdout).unwrap();
    assert_eq!(
        v["locked"], true,
        "clean lock should report locked=true: {v}"
    );
    assert_eq!(v["drift"].as_array().unwrap().len(), 0, "json: {v}");
}

fn tree_snapshot(root: &Path) -> Vec<String> {
    fn walk(base: &Path, cur: &Path, out: &mut Vec<String>) {
        let mut entries: Vec<_> = std::fs::read_dir(cur)
            .unwrap()
            .map(|e| e.unwrap().path())
            .collect();
        entries.sort();
        for path in entries {
            let rel = path.strip_prefix(base).unwrap().display().to_string();
            if path.is_dir() {
                out.push(format!("dir:{rel}"));
                walk(base, &path, out);
            } else {
                let bytes = std::fs::read(&path).unwrap();
                out.push(format!("file:{rel}:{}", bytes.len()));
            }
        }
    }
    let mut out = Vec::new();
    walk(root, root, &mut out);
    out
}

#[test]
fn json_shapes_cover_detect_doctor_graph_and_registry() {
    let fx = Fixture::new();

    let detect = fx.cmd().args(["--json", "auto-detect"]).output().unwrap();
    assert!(detect.status.success(), "stderr: {}", stderr(&detect));
    let detect_json: serde_json::Value = serde_json::from_slice(&detect.stdout).unwrap();
    assert!(
        detect_json["generated_at"].is_string(),
        "detect json: {detect_json}"
    );
    assert!(
        detect_json["components"].as_array().is_some(),
        "detect json: {detect_json}"
    );
    assert_eq!(
        detect_json["components"][0]["id"], "stub",
        "detect json: {detect_json}"
    );

    let doctor = fx.cmd().args(["--json", "doctor"]).output().unwrap();
    assert!(doctor.status.success(), "stderr: {}", stderr(&doctor));
    let doctor_json: serde_json::Value = serde_json::from_slice(&doctor.stdout).unwrap();
    assert!(
        doctor_json["tools"].as_array().is_some(),
        "doctor json: {doctor_json}"
    );
    assert!(
        doctor_json["writable"].as_array().is_some(),
        "doctor json: {doctor_json}"
    );

    let graph = fx.cmd().args(["--json", "graph"]).output().unwrap();
    assert!(graph.status.success(), "stderr: {}", stderr(&graph));
    let graph_json: serde_json::Value = serde_json::from_slice(&graph.stdout).unwrap();
    assert!(
        graph_json["nodes"].as_array().is_some(),
        "graph json: {graph_json}"
    );
    assert!(
        graph_json["edges"].as_array().is_some(),
        "graph json: {graph_json}"
    );

    let registry = fx
        .cmd()
        .args(["--json", "registry", "--check"])
        .output()
        .unwrap();
    assert!(registry.status.success(), "stderr: {}", stderr(&registry));
    let registry_json: serde_json::Value = serde_json::from_slice(&registry.stdout).unwrap();
    assert_eq!(
        registry_json["sources"].as_array().unwrap().len(),
        1,
        "registry json: {registry_json}"
    );
    assert_eq!(
        registry_json["entries"][0]["id"], "stub-tool",
        "registry json: {registry_json}"
    );
    assert_eq!(
        registry_json["drift"].as_array().unwrap().len(),
        0,
        "registry json: {registry_json}"
    );
}

#[test]
fn secret_wrapper_forwards_frozen_argv_without_live_daemon() {
    let fx = Fixture::new();
    let local_bin = fx.home.join(".local/bin");
    std::fs::create_dir_all(&local_bin).unwrap();
    let log = fx.root.join("secretctl-argv.log");
    let fake = local_bin.join("secretctl");
    std::fs::write(
        &fake,
        format!(
            r#"#!/bin/sh
printf '%s\n' "$@" > '{}'
printf '{{"ok":true,"argv":["%s"]}}\n' "$*"
exit 0
"#,
            log.display()
        ),
    )
    .unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = std::fs::metadata(&fake).unwrap().permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(&fake, perms).unwrap();
    }

    let out = fx
        .cmd()
        .args([
            "--json",
            "secret",
            "ca",
            "trust",
            "node",
            "python",
            "--system-bundle",
            "--apply",
            "--confirm",
        ])
        .output()
        .unwrap();
    assert!(out.status.success(), "stderr: {}", stderr(&out));
    let stdout_json: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(stdout_json["ok"], true, "stdout: {}", stdout(&out));
    let argv = std::fs::read_to_string(&log).unwrap();
    assert_eq!(
        argv.lines().collect::<Vec<_>>(),
        vec![
            "ca",
            "trust-apply",
            "--target=node",
            "--target=python",
            "--system-bundle",
            "--apply",
            "--confirm",
        ]
    );
}

#[test]
fn mutating_verbs_preview_without_writing_fixture_state() {
    let fx = Fixture::new();
    let local_repo = fx.root.join("local-source");
    std::fs::create_dir_all(local_repo.join("src")).unwrap();
    std::fs::write(
        local_repo.join("Cargo.toml"),
        "[package]\nname='local-source'\nversion='0.1.0'\nedition='2021'\n",
    )
    .unwrap();
    std::fs::write(local_repo.join("src/main.rs"), "fn main() {}\n").unwrap();

    let cases: Vec<Vec<String>> = vec![
        vec!["install", "--dry-run", "stub"],
        vec!["auto-fix", "stub"],
        vec!["reset", "stub"],
        vec![
            "add-repo",
            "file://fixture",
            "--id",
            "local-source",
            "--local",
        ],
        vec!["self", "uninstall"],
    ]
    .into_iter()
    .map(|v| v.into_iter().map(String::from).collect())
    .collect();

    for args in cases {
        let before = tree_snapshot(&fx.root);
        let mut cmd = fx.cmd();
        if args[0] == "add-repo" {
            cmd.arg("add-repo")
                .arg("file://fixture")
                .args(["--id", "local-source", "--local"])
                .arg(&local_repo);
        } else {
            cmd.args(args.iter().map(String::as_str));
        }
        let out = cmd.output().unwrap();
        assert!(
            out.status.success(),
            "args={args:?}\nstderr: {}\nstdout: {}",
            stderr(&out),
            stdout(&out)
        );
        assert_eq!(
            before,
            tree_snapshot(&fx.root),
            "preview mutated fixture for args={args:?}"
        );
        assert!(
            !fx.root.join("would-write").exists(),
            "hook ran during preview for args={args:?}"
        );
    }
}

#[test]
fn exit_code_matrix_covers_usage_and_guard_refusals() {
    let fx = Fixture::new();
    let cases: Vec<(Vec<&str>, i32, &str)> = vec![
        (vec!["install", "missing-component"], 1, "unknown component"),
        (
            vec!["reset", "--purge", "stub"],
            2,
            "refusing --purge without --confirm",
        ),
        (vec!["--color", "bogus", "doctor"], 2, "invalid value"),
        (
            vec!["agent", "lock", "--locked", "--update"],
            2,
            "unexpected argument",
        ),
    ];

    for (args, code, needle) in cases {
        let out = fx.cmd().args(args.clone()).output().unwrap();
        assert_eq!(
            out.status.code(),
            Some(code),
            "args={args:?}\nstdout: {}\nstderr: {}",
            stdout(&out),
            stderr(&out)
        );
        let combined = format!("{}{}", stdout(&out), stderr(&out));
        assert!(
            combined.contains(needle),
            "args={args:?} missing `{needle}` in:\n{combined}"
        );
    }
}
