#![forbid(unsafe_code)]

use std::fs;
use std::path::PathBuf;

fn envctl_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(3)
        .expect("harness crate path has envctl ancestor")
        .to_path_buf()
}

#[test]
fn ignored_runtime_receipts_are_not_checkout_completion_authority() {
    let root = envctl_root();
    let ignore = fs::read_to_string(root.join(".gitignore")).expect("repo ignore exists");
    assert!(ignore.contains("/home/agent-env/codex-harness/state/"));
    assert!(ignore.contains("/home/agent-env/codex-harness/ledger/*.jsonl"));
    assert!(ignore.contains("must never be"));
    assert!(ignore.contains("sole permission authority or completion proof"));

    let verifier = fs::read_to_string(
        root.join("home/agent-env/codex-harness/src/bin/codex-harness-final-verify.rs"),
    )
    .expect("tracked final verifier exists");
    assert!(verifier.contains("tracked operator-intent record"));
    assert!(verifier.contains("execpolicy_allows_full_access"));
    assert!(verifier.contains("operator full-access launch"));
}
