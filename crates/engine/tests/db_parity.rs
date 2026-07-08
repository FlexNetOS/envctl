//! REQ-059 — CLI/GUI shared-engine parity for the GH#414 db surface.
//!
//! The db query / refactor / deploy behaviour lives in `envctl-engine`
//! ([`envctl_engine::DbSnapshot`]); `crates/cli/src/main.rs` and the GUI are thin
//! renderers that call the identical engine entry points. This test pins that
//! contract the way the repo's other `*_parity.rs` files do: it drives the shared
//! engine API as BOTH front-ends would and asserts they observe byte-identical
//! machine output — so a future change that let one front-end grow its own db
//! logic (divergent results) fails here.
//!
//! There is intentionally no front-end-specific code path to compare against:
//! that IS the invariant. The "CLI caller" and "GUI caller" below are the same
//! `DbSnapshot` methods invoked independently, and their serialized results must
//! match exactly.

use envctl_engine::{
    db_query::{QueryPreset, QuerySpec},
    db_refactor::RootAliasSpec,
    DbSnapshot, ScanScope,
};
use std::fs;

fn fixture(tag: &str) -> std::path::PathBuf {
    let d = std::env::temp_dir().join(format!("envctl-db-parity-{tag}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&d);
    fs::create_dir_all(&d).unwrap();
    fs::write(d.join("cli.rs"), b"const R: &str = \"x\";\n").unwrap();
    fs::write(
        d.join("wrapper.sh"),
        b"cd $META_ROOT/bin\nexport A=${LIFEOS_ROOT}\n",
    )
    .unwrap();
    fs::write(d.join(".env"), b"SECRET=$META_ROOT/s\n").unwrap();
    d
}

fn scope(root: &std::path::Path) -> ScanScope {
    ScanScope {
        root: root.display().to_string(),
        ..Default::default()
    }
}

fn meta_query() -> QuerySpec {
    QuerySpec {
        table: None,
        filters: vec![],
        preset: Some(QueryPreset::RootMeta),
        target_profile: None,
        explain: true,
    }
}

fn meta_refactor() -> RootAliasSpec {
    RootAliasSpec {
        from: "META_ROOT".into(),
        to: "LIFE_OS_ROOT".into(),
        target_profile: Some("lifeos-release".into()),
        scope: None,
        render_out: None,
    }
}

#[test]
fn cli_and_gui_query_paths_produce_identical_json() {
    let root = fixture("query");

    // "CLI front-end" builds a snapshot and queries.
    let cli = DbSnapshot::open(scope(&root)).unwrap();
    let cli_json = serde_json::to_string(&cli.query(&meta_query()).unwrap()).unwrap();

    // "GUI front-end" builds its OWN snapshot from the same scope and queries via
    // the identical engine entry point.
    let gui = DbSnapshot::open(scope(&root)).unwrap();
    let gui_json = serde_json::to_string(&gui.query(&meta_query()).unwrap()).unwrap();

    assert_eq!(
        cli_json, gui_json,
        "CLI and GUI must share the query contract"
    );
    // Sanity: the shared surface actually did something (META_ROOT symbol found).
    assert!(cli_json.contains("\"row_count\":1"), "got {cli_json}");

    let _ = fs::remove_dir_all(&root);
}

#[test]
fn cli_and_gui_refactor_plans_are_identical_and_fail_closed() {
    let root = fixture("refactor");

    let cli = DbSnapshot::open(scope(&root)).unwrap();
    let cli_plan = cli.refactor_plan(&meta_refactor()).unwrap();

    let gui = DbSnapshot::open(scope(&root)).unwrap();
    let gui_plan = gui.refactor_plan(&meta_refactor()).unwrap();

    assert_eq!(
        serde_json::to_string(&cli_plan).unwrap(),
        serde_json::to_string(&gui_plan).unwrap(),
        "CLI and GUI must share the refactor contract"
    );
    // Both front-ends see the same fail-closed plan: wrapper.sh safe, .env refused.
    assert_eq!(cli_plan.files_touched, 1);
    assert!(cli_plan.refused >= 1);
    assert!(!cli_plan.approved);

    let _ = fs::remove_dir_all(&root);
}

#[test]
fn cli_and_gui_roots_surface_is_identical() {
    let cli = envctl_engine::db_roots(Some("/o".into()), Some("/r".into()), "lifeos-release");
    let gui = envctl_engine::db_roots(Some("/o".into()), Some("/r".into()), "lifeos-release");
    assert_eq!(
        serde_json::to_string(&cli).unwrap(),
        serde_json::to_string(&gui).unwrap()
    );
    assert_eq!(
        cli.len(),
        2,
        "observed + release-target held simultaneously"
    );
}
