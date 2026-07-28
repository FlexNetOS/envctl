use serde_json::Value;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

fn temp_dir() -> PathBuf {
    let path = std::env::temp_dir().join(format!(
        "envctl-migration-target-cli-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir_all(&path).unwrap();
    path
}

fn run(root: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_envctl"))
        .args(args)
        .env("ENVCTL_MIGRATION_DB", root.join("migration.redb"))
        .output()
        .expect("run envctl")
}

fn stdout_json(output: &Output) -> Value {
    assert!(
        output.status.success(),
        "status: {}\nstderr: {}",
        output.status,
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&output.stdout).expect("JSON stdout")
}

#[test]
fn target_validate_add_list_show_and_assertions() {
    let root = temp_dir();
    let descriptor = root.join("target.yaml");
    std::fs::write(
        &descriptor,
        r#"schema_version: 1
target_id: cli-target
target_type: codebase
primary_root: /tmp/repo
output_root: migration-artifacts
include: ["**/*"]
exclude: [target]
collectors:
  filesystem: true
safety:
  default_mode: approval-gated
  max_auto_risk: R2
  allow_network: false
  allow_destructive: false
artifact_contract:
  name: contract
  version: 1
recipe:
  name: recipe
  version: 1
metadata: {}
"#,
    )
    .unwrap();
    let descriptor_arg = descriptor.to_str().unwrap();

    let validated = stdout_json(&run(
        &root,
        &["--json", "migration", "target", "validate", descriptor_arg],
    ));
    assert_eq!(validated["valid"], true);
    assert_eq!(validated["descriptor"]["target_id"], "cli-target");
    assert_eq!(validated["descriptor_hash"].as_str().unwrap().len(), 64);

    let added = stdout_json(&run(
        &root,
        &[
            "--json",
            "migration",
            "target",
            "add",
            "--descriptor",
            descriptor_arg,
        ],
    ));
    assert_eq!(added["target_id"], "cli-target");
    assert_eq!(added["safety_mode"], "approval-gated");
    assert_eq!(added["allow_network"], false);
    assert_eq!(added["descriptor_hash"], validated["descriptor_hash"]);

    let listed = stdout_json(&run(&root, &["--json", "migration", "target", "list"]));
    assert_eq!(listed.as_array().unwrap().len(), 1);

    let shown = stdout_json(&run(
        &root,
        &["--json", "migration", "target", "show", "cli-target"],
    ));
    assert_eq!(shown["descriptor_json"]["recipe"]["name"], "recipe");

    let contradiction = run(
        &root,
        &[
            "--json",
            "migration",
            "target",
            "add",
            "wrong-target",
            "--descriptor",
            descriptor_arg,
        ],
    );
    assert!(!contradiction.status.success());
    assert!(String::from_utf8_lossy(&contradiction.stderr)
        .contains("target_id assertion contradicts descriptor"));
}
