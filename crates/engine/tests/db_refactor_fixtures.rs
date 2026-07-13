//! Fixture-matrix proof for the `db refactor --apply` path over the atomic
//! backup-write primitive (ARCH08 / MISS07 / NFR08 / ARCH17).
//!
//! The checked-in `fixtures/refactor_matrix/` tree is copied into a temp
//! workspace, then driven through the *shared engine* entry points
//! (`DbSnapshot::open` → `refactor_plan` → `refactor_apply`) exactly as the CLI
//! and GUI do. We assert the fail-closed disposition per file kind and that every
//! mutated file leaves a `.bak` holding its original content.

use envctl_engine::{
    db_refactor::{Approval, RootAliasSpec},
    DbSnapshot, ScanScope,
};
use std::fs;
use std::path::{Path, PathBuf};

/// Repo-root-relative path to the checked-in fixture tree.
fn fixture_src() -> PathBuf {
    // CARGO_MANIFEST_DIR = crates/engine; the fixtures live at the repo root.
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/refactor_matrix")
        .canonicalize()
        .expect("fixture tree present")
}

/// Recursively copy `src` into `dst`.
fn copy_tree(src: &Path, dst: &Path) {
    fs::create_dir_all(dst).unwrap();
    for entry in fs::read_dir(src).unwrap() {
        let entry = entry.unwrap();
        let from = entry.path();
        let to = dst.join(entry.file_name());
        if from.is_dir() {
            copy_tree(&from, &to);
        } else {
            fs::copy(&from, &to).unwrap();
        }
    }
}

fn workspace(tag: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("envctl-refactor-fx-{tag}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&d);
    copy_tree(&fixture_src(), &d);
    d
}

#[test]
fn apply_over_fixture_matrix_rewrites_safe_files_backs_them_up_and_refuses_protected() {
    let ws = workspace("apply");
    let scope = ScanScope {
        root: ws.display().to_string(),
        ..Default::default()
    };
    let spec = RootAliasSpec {
        from: "META_ROOT".into(),
        to: "LIFE_OS_ROOT".into(),
        target_profile: Some("lifeos-release".into()),
        scope: None,
        render_out: None,
    };

    let snap = DbSnapshot::open(scope).unwrap();
    let plan = snap.refactor_plan(&spec).unwrap();

    // Safe rewrite targets: the two shell wrappers + the toml. The .rs has no
    // `$`-token, .env is Never (refused), README.md is ManualReview (refused).
    assert_eq!(plan.files_touched, 3, "wrapper.sh, alias.sh, paths.toml");
    assert!(plan.refused >= 1, "protected/.env + prose refused");
    assert!(!plan.approved, "a plan is never pre-approved");

    // Fail-closed: no confirm/approval mutates nothing.
    assert!(snap.refactor_apply(&plan, &spec, false, None).is_err());
    let unchanged = fs::read_to_string(ws.join("wrappers/wrapper.sh")).unwrap();
    assert!(
        unchanged.contains("$META_ROOT"),
        "refused apply left original"
    );

    // Gated apply: confirm + approval.
    let ok = Approval {
        approver: "drdave".into(),
        approved: true,
        note: Some("REQ-055 fixture".into()),
    };
    let mutated = snap.refactor_apply(&plan, &spec, true, Some(&ok)).unwrap();
    assert_eq!(mutated.len(), 3);

    // wrapper.sh rewritten; original preserved in .bak.
    let w = ws.join("wrappers/wrapper.sh");
    let w_new = fs::read_to_string(&w).unwrap();
    assert!(w_new.contains("$LIFE_OS_ROOT"), "rewritten: {w_new}");
    assert!(!w_new.contains("$META_ROOT"));
    let w_bak = fs::read_to_string(bak(&w)).expect("wrapper.sh .bak kept");
    assert!(w_bak.contains("$META_ROOT"), "backup holds original");

    // alias.sh: the $META_ROOT token rewrites, but the ${LIFEOS_ROOT} alias (a
    // DIFFERENT root that already normalizes to the `to` side) is left untouched.
    let a = ws.join("wrappers/alias.sh");
    let a_new = fs::read_to_string(&a).unwrap();
    assert!(
        a_new.contains("$LIFE_OS_ROOT/base"),
        "META_ROOT rewrote: {a_new}"
    );
    assert!(
        a_new.contains("${LIFEOS_ROOT}/legacy"),
        "alias untouched: {a_new}"
    );
    assert!(bak(&a).exists(), "alias.sh .bak kept");

    // paths.toml rewritten with a backup.
    let t = ws.join("config/paths.toml");
    assert!(fs::read_to_string(&t).unwrap().contains("$LIFE_OS_ROOT"));
    assert!(bak(&t).exists(), "toml .bak kept");

    // .env is Never: never modified, no backup.
    let env = ws.join("secrets/.env");
    assert!(fs::read_to_string(&env).unwrap().contains("$META_ROOT"));
    assert!(!bak(&env).exists(), ".env must never be written/backed up");

    // README.md is ManualReview: refused, unmodified, no backup.
    let readme = ws.join("README.md");
    assert!(fs::read_to_string(&readme).unwrap().contains("$META_ROOT"));
    assert!(!bak(&readme).exists());

    let _ = fs::remove_dir_all(&ws);
}

/// The `.bak` sibling for a path.
fn bak(p: &Path) -> PathBuf {
    let mut s = p.as_os_str().to_owned();
    s.push(".bak");
    PathBuf::from(s)
}
