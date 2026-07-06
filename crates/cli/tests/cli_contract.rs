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
            .env("META_ROOT", &self.meta)
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
        "migrate",
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
fn catalog_repo_root_imports_yazelix_config_without_manifest() {
    let fx = Fixture::new();
    let yazelix = fx.root.join("yazelix");
    std::fs::create_dir_all(yazelix.join("config_metadata")).unwrap();
    std::fs::create_dir_all(yazelix.join("configs/zellij/layouts")).unwrap();
    std::fs::create_dir_all(yazelix.join("nushell/config")).unwrap();
    std::fs::write(
        yazelix.join("settings_default.jsonc"),
        r#"{"debug_mode":false}"#,
    )
    .unwrap();
    std::fs::write(
        yazelix.join("config_metadata/main_config_contract.toml"),
        r#"
[[field]]
key = "debug_mode"
default = false
"#,
    )
    .unwrap();
    std::fs::write(
        yazelix.join("configs/zellij/layouts/flexnetos_agent_workspace.kdl"),
        "layout {}\n",
    )
    .unwrap();
    std::fs::write(
        yazelix.join("nushell/config/config.nu"),
        "$env.config.show_banner = false\n",
    )
    .unwrap();

    let out = Command::new(bin())
        .current_dir(&fx.root)
        .env("ENVCTL_MANIFEST_DIR", fx.root.join("missing-manifest"))
        .args(["--json", "catalog", "--repo-root"])
        .arg(&yazelix)
        .args(["table", "config-files"])
        .output()
        .unwrap();
    assert!(out.status.success(), "stderr: {}", stderr(&out));
    let rows: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    let rows = rows.as_array().unwrap();
    assert!(rows.iter().any(|row| {
        row.get("path").and_then(|v| v.as_str()) == Some("settings_default.jsonc")
            && row.get("file_kind").and_then(|v| v.as_str()) == Some("yazelix_settings_default")
    }));
    assert!(rows.iter().any(|row| {
        row.get("path").and_then(|v| v.as_str()) == Some("nushell/config/config.nu")
            && row.get("owner_component").and_then(|v| v.as_str()) == Some("yazelix")
    }));
}

#[test]
fn catalog_manifest_dir_defaults_repo_root_to_parent() {
    let fx = Fixture::new();
    let yazelix = fx.root.join("yazelix");
    std::fs::create_dir_all(&yazelix).unwrap();
    std::fs::write(
        yazelix.join("settings_default.jsonc"),
        r#"{"debug_mode":false}"#,
    )
    .unwrap();

    let out = Command::new(bin())
        .current_dir(&fx.root)
        .args(["--json", "catalog", "--manifest-dir"])
        .arg(yazelix.join("missing-manifest"))
        .args(["table", "config-files"])
        .output()
        .unwrap();
    assert!(out.status.success(), "stderr: {}", stderr(&out));
    let rows: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    let rows = rows.as_array().unwrap();
    assert!(rows.iter().any(|row| {
        row.get("path").and_then(|v| v.as_str()) == Some("settings_default.jsonc")
            && row.get("file_kind").and_then(|v| v.as_str()) == Some("yazelix_settings_default")
    }));
}

#[test]
fn catalog_repo_root_imports_yazelix_codedb_file_inventory() {
    let fx = Fixture::new();
    let yazelix = fx.root.join("yazelix");
    std::fs::create_dir_all(yazelix.join("docs/generated")).unwrap();
    std::fs::create_dir_all(yazelix.join(".local/share/yazelix")).unwrap();
    std::fs::write(
        yazelix.join("settings_default.jsonc"),
        r#"{"debug_mode":false}"#,
    )
    .unwrap();
    std::fs::write(yazelix.join(".local/share/yazelix/state.json"), "{}\n").unwrap();
    std::fs::write(
        yazelix.join("docs/generated/yazelix_file_target_inventory.json"),
        format!(
            r#"[
  {{
    "target_id": "repo_settings_default",
    "absolute_path": "{}",
    "normalized_logical_path": "repo_source:settings_default.jsonc",
    "owner": "yazelix",
    "source_of_truth_class": "repo_source",
    "exists": true,
    "file_kind": "regular_file",
    "parser_hint": "jsonc",
    "mutability": "source_controlled",
    "reproduction_policy": "git_checkout",
    "safety_policy": "source_content_import_allowed",
    "import_mode": "content_blob"
  }},
  {{
    "target_id": "nix_store_runtime",
    "absolute_path": "/nix/store/example-yazelix-runtime",
    "normalized_logical_path": "nix_store:/nix/store/example-yazelix-runtime",
    "owner": "nix",
    "source_of_truth_class": "nix_store_package_output",
    "exists": true,
    "file_kind": "package_output",
    "parser_hint": "nix_store_path",
    "mutability": "immutable_store",
    "reproduction_policy": "nix_realise",
    "safety_policy": "nix_store_metadata_only",
    "import_mode": "metadata_only"
  }},
  {{
    "target_id": "local_state",
    "absolute_path": "{}",
    "normalized_logical_path": "real_home_runtime_state:.local/share/yazelix/state.json",
    "owner": "user",
    "source_of_truth_class": "real_home_runtime_state",
    "exists": true,
    "file_kind": "regular_file",
    "parser_hint": "json",
    "mutability": "user_state",
    "reproduction_policy": "runtime_observed",
    "safety_policy": "real_home_metadata_only",
    "import_mode": "metadata_only"
  }}
]
"#,
            yazelix.join("settings_default.jsonc").display(),
            yazelix.join(".local/share/yazelix/state.json").display(),
        ),
    )
    .unwrap();

    let out = Command::new(bin())
        .current_dir(&fx.root)
        .env("ENVCTL_MANIFEST_DIR", fx.root.join("missing-manifest"))
        .args(["--json", "catalog", "--repo-root"])
        .arg(&yazelix)
        .args(["table", "codedb-file-imports"])
        .output()
        .unwrap();
    assert!(out.status.success(), "stderr: {}", stderr(&out));
    let rows: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    let rows = rows.as_array().unwrap();

    assert_eq!(rows.len(), 3, "{rows:#?}");
    let repo_row = rows
        .iter()
        .find(|row| row.get("target_id").and_then(|v| v.as_str()) == Some("repo_settings_default"))
        .expect("repo content row");
    assert_eq!(
        repo_row.get("import_status").and_then(|v| v.as_str()),
        Some("blob_metadata_ready")
    );
    assert!(repo_row
        .get("content_hash")
        .and_then(|v| v.as_str())
        .is_some_and(|hash| hash.len() == 64));
    assert!(repo_row
        .get("blob_ref")
        .and_then(|v| v.as_str())
        .is_some_and(|blob| blob.starts_with("sha256:")));
    assert!(repo_row
        .get("last_observed")
        .and_then(|v| v.as_str())
        .is_some_and(|value| value.contains('T')));
    assert_eq!(
        repo_row.get("structured_status").and_then(|v| v.as_str()),
        Some("structured_rows_ready")
    );
    assert!(repo_row
        .get("structured_row_count")
        .and_then(|v| v.as_u64())
        .is_some_and(|count| count > 0));
    assert!(repo_row
        .get("structured_rows")
        .and_then(|v| v.as_array())
        .is_some_and(|rows| rows.iter().any(|row| {
            row.get("key").and_then(|v| v.as_str()) == Some("debug_mode")
                && row.get("value").and_then(|v| v.as_str()) == Some("false")
        })));

    for target_id in ["nix_store_runtime", "local_state"] {
        let row = rows
            .iter()
            .find(|row| row.get("target_id").and_then(|v| v.as_str()) == Some(target_id))
            .unwrap_or_else(|| panic!("missing {target_id} row"));
        assert_eq!(
            row.get("import_status").and_then(|v| v.as_str()),
            Some("metadata_only")
        );
        assert!(row.get("content_hash").is_some_and(|v| v.is_null()));
        assert!(row.get("blob_ref").is_some_and(|v| v.is_null()));
        assert!(row
            .get("skip_reason")
            .and_then(|v| v.as_str())
            .is_some_and(|reason| reason.ends_with("metadata_only")));
        assert_eq!(
            row.get("structured_status").and_then(|v| v.as_str()),
            Some("metadata_only")
        );
        assert_eq!(
            row.get("structured_row_count").and_then(|v| v.as_u64()),
            Some(0)
        );
    }
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
    let bin_dir = fx.meta.join("usr/bin");
    std::fs::create_dir_all(&bin_dir).unwrap();
    let log = fx.root.join("secretctl-argv.log");
    let fake = bin_dir.join("secretctl");
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
        vec!["migrate", "apply"],
        vec!["migrate", "purge"],
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
fn migration_scan_json_reports_meta_layout_and_legacy_manifest_debt() {
    let fx = Fixture::new();
    let components = fx.manifest.join("components.d");
    std::fs::create_dir_all(&components).unwrap();
    let legacy_home_local = ["~", ".local/bin/foo"].join("/");
    let legacy_usr_local = ["/usr", "local/bin/bar"].join("/");
    std::fs::write(
        components.join("legacy.toml"),
        format!(
            r#"
[[component]]
id = "legacy-paths"
name = "Legacy Paths"

[component.detect]
kind = "command"
command = "true"

[component.install]
kind = "command"
command = "sh"
args = ["-c", "echo $META_ROOT/.toolchains/legacy && echo {legacy_home_local} && echo {legacy_usr_local}"]
"#
        ),
    )
    .unwrap();

    let out = fx
        .cmd()
        .args(["--json", "migrate", "scan"])
        .output()
        .unwrap();
    assert!(out.status.success(), "stderr: {}", stderr(&out));
    let v: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(v["schema"], "envctl.migration.report.v1");
    assert_eq!(v["meta_root"], fx.meta.display().to_string());
    assert!(
        v["layout"].as_array().unwrap().iter().any(|entry| {
            entry["key"] == "bin" && entry["path"] == fx.meta.join("usr/bin").display().to_string()
        }),
        "layout missing canonical meta usr/bin: {v}"
    );
    let items = v["items"].as_array().unwrap();
    assert!(
        items
            .iter()
            .any(|item| item["kind"] == "manifest_legacy_token"
                && item["status"] == "needs_migration"),
        "missing legacy token item: {v}"
    );
    assert!(
        items
            .iter()
            .any(|item| item["kind"] == "user_global_path"
                && item["action"] == "adopt_into_meta_local"),
        "missing user-global adoption item: {v}"
    );
    assert!(
        items.iter().any(|item| {
            item["kind"] == "legacy_compatibility_root" && item["protected"] == true
        }),
        "missing protected legacy compatibility item: {v}"
    );
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
