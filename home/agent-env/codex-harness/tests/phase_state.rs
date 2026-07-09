#![forbid(unsafe_code)]

use serde_json::Value;
use sha2::{Digest, Sha256};
use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

fn harness_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn envctl_root(harness: &Path) -> PathBuf {
    harness
        .ancestors()
        .nth(3)
        .expect("harness crate path has envctl ancestor")
        .to_path_buf()
}

#[test]
fn phase_state_files_cover_current_prompt_and_all_phases() {
    let harness = harness_root();
    let prompt_path = envctl_root(&harness)
        .join(".codex/prompts/prompt:codex-gpt-harness-v3-full-access-no-sandbox.prompt.md");
    let prompt_bytes = fs::read(&prompt_path).expect("current harness prompt exists");
    let prompt_sha256 = hex::encode(Sha256::digest(&prompt_bytes));

    let checklist_path = harness.join("state/phase-execution-checklist.json");
    let checklist: Value = serde_json::from_str(
        &fs::read_to_string(&checklist_path).expect("phase checklist exists"),
    )
    .expect("phase checklist is JSON");

    assert_eq!(
        checklist.get("schema").and_then(Value::as_str),
        Some("codex-harness.phase-execution-checklist.v1")
    );
    assert_eq!(
        checklist.pointer("/prompt/sha256").and_then(Value::as_str),
        Some(prompt_sha256.as_str())
    );

    let phases = checklist
        .get("phases")
        .and_then(Value::as_array)
        .expect("checklist has phases");
    let mut seen = BTreeSet::new();
    for phase in phases {
        let number = phase
            .get("phase")
            .and_then(Value::as_u64)
            .expect("phase has number");
        assert!(number <= 11, "unexpected phase {number}");
        seen.insert(number);
        assert_eq!(
            phase.get("result").and_then(Value::as_str),
            Some("pass"),
            "phase {number} must be pass"
        );
        let items = phase
            .get("items")
            .and_then(Value::as_array)
            .expect("phase has items");
        assert!(!items.is_empty(), "phase {number} has no items");
        for item in items {
            let status = item
                .get("status")
                .and_then(Value::as_str)
                .expect("item has status");
            assert!(
                status == "pass" || status == "unsupported",
                "unexpected status {status} in phase {number}"
            );
            let mandatory = item
                .get("mandatory")
                .and_then(Value::as_bool)
                .unwrap_or(true);
            if mandatory {
                assert_eq!(status, "pass", "mandatory phase {number} item not pass");
            }
            for key in ["id", "requirement", "proof_command", "evidence"] {
                assert!(
                    item.get(key)
                        .and_then(Value::as_str)
                        .map(|text| !text.trim().is_empty())
                        .unwrap_or(false),
                    "phase {number} item missing {key}"
                );
            }
        }
    }
    assert_eq!(seen, (0_u64..=11).collect::<BTreeSet<_>>());

    let continuation = fs::read_to_string(harness.join("state/compact-continuation.md"))
        .expect("compact continuation exists");
    assert!(continuation.contains(&prompt_sha256));
    assert!(continuation.contains("state/phase-execution-checklist.json"));
    assert!(continuation.contains("ledger/harness.jsonl"));
    assert!(continuation.contains("next exact command:"));
}
