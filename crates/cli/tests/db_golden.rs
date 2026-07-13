//! Golden determinism for the agent-facing db `--json` surface (REQ MISS12 /
//! NFR: deterministic machine output). Two identical invocations of the SAME
//! command must produce byte-identical stdout — an agent that diffs output
//! between runs must never see spurious churn.
//!
//! Drives the real `envctl` binary (not the engine API) so it also pins that the
//! CLI renderer itself is deterministic (pretty-printer, ordering, no timestamps
//! leaking into the machine contract).

use std::path::{Path, PathBuf};
use std::process::Command;

fn bin() -> &'static str {
    env!("CARGO_BIN_EXE_envctl")
}

fn fixture(tag: &str) -> PathBuf {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let d = std::env::temp_dir().join(format!("envctl-db-golden-{tag}-{nanos}"));
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(&d).unwrap();
    std::fs::write(d.join("cli.rs"), b"const R: &str = \"x\";\n").unwrap();
    std::fs::write(
        d.join("wrapper.sh"),
        b"cd $META_ROOT/bin\nexport A=${LIFEOS_ROOT}\n",
    )
    .unwrap();
    std::fs::write(d.join(".env"), b"SECRET=$META_ROOT/s\n").unwrap();
    d
}

fn run_json(args: &[&str], cwd: Option<&Path>) -> String {
    let mut cmd = Command::new(bin());
    cmd.args(args);
    if let Some(dir) = cwd {
        cmd.current_dir(dir);
    }
    let out = cmd.output().expect("spawn envctl");
    assert!(
        out.status.success(),
        "envctl {:?} failed: {}",
        args,
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8(out.stdout).expect("utf8 stdout")
}

#[test]
fn db_roots_json_is_byte_identical_across_runs() {
    let a = run_json(
        &[
            "db",
            "roots",
            "--observed",
            "/o",
            "--release",
            "/r",
            "--json",
        ],
        None,
    );
    let b = run_json(
        &[
            "db",
            "roots",
            "--observed",
            "/o",
            "--release",
            "/r",
            "--json",
        ],
        None,
    );
    assert_eq!(a, b, "db roots --json must be deterministic across runs");
    assert!(a.contains("release_target") || a.contains("observed_current"));
}

#[test]
fn db_query_json_is_byte_identical_across_runs() {
    let fx = fixture("query");
    let args = &["db", "query", "--preset", "root-meta", "--json"];
    let a = run_json(args, Some(&fx));
    let b = run_json(args, Some(&fx));
    assert_eq!(a, b, "db query --json must be deterministic across runs");
    assert!(a.contains("\"row_count\""));
    let _ = std::fs::remove_dir_all(&fx);
}

#[test]
fn db_symbols_json_is_byte_identical_across_runs() {
    let fx = fixture("symbols");
    let args = &["db", "symbols", "--json"];
    let a = run_json(args, Some(&fx));
    let b = run_json(args, Some(&fx));
    assert_eq!(a, b, "db symbols --json must be deterministic across runs");
    let _ = std::fs::remove_dir_all(&fx);
}
