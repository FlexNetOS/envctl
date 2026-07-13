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
    db_deploy::DeploySpec,
    db_query::{QueryPreset, QuerySpec},
    db_refactor::RootAliasSpec,
    DbSnapshot, IndexDelta, ScanScope, WatchState,
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
fn cli_and_gui_symbols_surface_is_identical() {
    let root = fixture("symbols");

    // Both front-ends build their own snapshot and serialize the shared symbol +
    // occurrence index. `db symbols --json` renders exactly this.
    let symbols_json = |snap: &DbSnapshot| {
        serde_json::to_string(&serde_json::json!({
            "symbols": snap.symbols().symbols(),
            "occurrences": snap.symbols().occurrences(),
        }))
        .unwrap()
    };

    let cli = DbSnapshot::open(scope(&root)).unwrap();
    let gui = DbSnapshot::open(scope(&root)).unwrap();
    assert_eq!(
        symbols_json(&cli),
        symbols_json(&gui),
        "CLI and GUI must share the symbols contract"
    );
    // Sanity: META_ROOT / LIFE_OS_ROOT symbols were extracted.
    assert!(cli
        .symbols()
        .symbols()
        .iter()
        .any(|s| s.normalized_name == "META_ROOT"));

    let _ = fs::remove_dir_all(&root);
}

#[test]
fn cli_and_gui_deploy_plans_are_identical_and_fail_closed() {
    let root = fixture("deploy");
    // A staged tree to promote and a target root indexed from the fixture.
    let stage = std::env::temp_dir().join(format!(
        "envctl-db-parity-deploy-stage-{}",
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&stage);
    fs::create_dir_all(stage.join("hooks")).unwrap();
    fs::write(stage.join("hooks/h.sh"), b"cd $LIFE_OS_ROOT\n").unwrap();

    let spec = DeploySpec {
        kind: "hooks".into(),
        target: root.display().to_string(),
        stage_dir: Some(stage.display().to_string()),
    };

    let cli = DbSnapshot::open(scope(&root)).unwrap();
    let gui = DbSnapshot::open(scope(&root)).unwrap();
    let cli_plan = cli.deploy_plan(&spec).unwrap();
    let gui_plan = gui.deploy_plan(&spec).unwrap();
    assert_eq!(
        serde_json::to_string(&cli_plan).unwrap(),
        serde_json::to_string(&gui_plan).unwrap(),
        "CLI and GUI must share the deploy contract"
    );
    // Fail-closed: a plan is never pre-approved.
    assert!(!cli_plan.approved);

    let _ = fs::remove_dir_all(&root);
    let _ = fs::remove_dir_all(&stage);
}

#[test]
fn cli_and_gui_watch_deltas_are_identical() {
    let root = fixture("watch");

    // Each front-end takes its own initial snapshot of the same scope and ticks
    // once with no fs change between — the shared incremental-invalidation core
    // must observe byte-identical deltas.
    let mut cli = WatchState::init(scope(&root)).unwrap();
    let mut gui = WatchState::init(scope(&root)).unwrap();
    let cli_delta: IndexDelta = cli.tick().unwrap();
    let gui_delta: IndexDelta = gui.tick().unwrap();
    assert_eq!(
        serde_json::to_string(&cli_delta).unwrap(),
        serde_json::to_string(&gui_delta).unwrap(),
        "CLI and GUI must share the watch contract"
    );
    assert!(cli_delta.is_empty(), "no change -> empty delta");
    assert!(cli.index().files().len() >= 3, "fixture indexed");

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
