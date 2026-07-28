//! REQ-061-ARCH18 — direct contract between the shipped `envctl db` CLI and
//! its operator documentation.

use std::path::{Path, PathBuf};
use std::process::Command;

const CLI_EXAMPLES: &[&str] = &[
    "envctl db roots --observed /home/u/meta --release /home/u/lifeos --json",
    "envctl db query --preset root-meta --json",
    "envctl db --repo-root /path/to/repo query --preset paths:legacy --json",
    "envctl db --repo-root /path/to/repo symbols --json",
    "envctl db --repo-root /path/to/repo impact --symbol LIFE_OS_ROOT --json",
    "envctl db refactor --from META_ROOT --to LIFE_OS_ROOT --json",
    "envctl db --repo-root /path/to/repo refactor --from META_ROOT --to LIFE_OS_ROOT --render-out /tmp/rendered",
    "envctl db --repo-root /path/to/repo refactor --from META_ROOT --to LIFE_OS_ROOT --apply --confirm --approve drdave --note 'REQ-055 migration'",
    "envctl db deploy --kind hooks --target /path/to/root --stage /tmp/rendered --json",
    "envctl db deploy --kind hooks --target /path/to/root --stage /tmp/rendered --apply --confirm --approve drdave",
];

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .expect("crates/cli must be nested below the workspace root")
        .to_path_buf()
}

fn read_doc(relative: &str) -> String {
    let path = workspace_root().join(relative);
    std::fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("required documentation {}: {error}", path.display()))
}

fn assert_contains(haystack: &str, needle: &str, document: &str) {
    assert!(
        haystack.contains(needle),
        "{document} must contain the exact contract text `{needle}`"
    );
}

fn split_shell_words(command: &str) -> Vec<String> {
    let mut words = Vec::new();
    let mut word = String::new();
    let mut quote = None;

    for character in command.chars() {
        match (quote, character) {
            (Some(expected), actual) if expected == actual => quote = None,
            (Some(_), actual) => word.push(actual),
            (None, '\'' | '"') => quote = Some(character),
            (None, actual) if actual.is_whitespace() => {
                if !word.is_empty() {
                    words.push(std::mem::take(&mut word));
                }
            }
            (None, actual) => word.push(actual),
        }
    }

    assert!(quote.is_none(), "unterminated quote in example: {command}");
    if !word.is_empty() {
        words.push(word);
    }
    words
}

#[test]
fn req_061_arch18_docs_cover_the_complete_database_automation_contract() {
    let readme = read_doc("README.md");
    let roadmap = read_doc("docs/ROADMAP.md");
    let automation = read_doc("docs/DB-AUTOMATION.md");

    for text in [
        "Meta control plane",
        "LifeOS product runtime",
        "META_ROOT",
        "LIFE_OS_ROOT",
        "LIFEOS_ROOT",
    ] {
        assert_contains(&automation, text, "docs/DB-AUTOMATION.md");
    }

    for preset in [
        "root:meta",
        "root:lifeos",
        "hooks:codex",
        "wrappers:broken",
        "mutable:unsafe",
        "paths:legacy",
    ] {
        assert_contains(&automation, preset, "docs/DB-AUTOMATION.md");
    }

    for text in [
        "`wrappers-broken` currently resolves to files whose `file_kind` is `shell`",
        "`mutable-unsafe` currently resolves to files whose `mutable_policy` is `never`",
        "`paths-legacy` currently matches the literal `legacy` substring in `absolute_path`",
        "The CLI is wired to these engine APIs today",
        "does not yet expose a database screen",
    ] {
        assert_contains(&automation, text, "docs/DB-AUTOMATION.md");
    }

    for text in [
        "envctl db symbols",
        "envctl db impact --symbol",
        "plan-only by default",
        "--render-out",
        "originals are never modified",
        "--apply --confirm --approve",
        "Queued",
        "Refused",
        "atomic",
        ".envctl-bak",
    ] {
        assert_contains(&automation, text, "docs/DB-AUTOMATION.md");
    }

    for example in CLI_EXAMPLES {
        assert_contains(&automation, example, "docs/DB-AUTOMATION.md");
    }

    for text in [
        "Meta and LifeOS database automation",
        "docs/DB-AUTOMATION.md",
        "envctl db roots --json",
        "envctl db query --preset root-meta --json",
    ] {
        assert_contains(&readme, text, "README.md");
    }

    for text in [
        "Database automation — implemented",
        "Meta control plane",
        "LifeOS product runtime",
        "DB-AUTOMATION.md",
        "root-alias refactor",
        "hook deployment",
    ] {
        assert_contains(&roadmap, text, "docs/ROADMAP.md");
    }
}

#[test]
fn every_documented_example_is_accepted_by_the_current_clap_surface() {
    for example in CLI_EXAMPLES {
        let mut words = split_shell_words(example);
        assert_eq!(words.first().map(String::as_str), Some("envctl"));
        words.remove(0);
        words.push("--help".to_string());

        let output = Command::new(env!("CARGO_BIN_EXE_envctl"))
            .args(&words)
            .output()
            .unwrap_or_else(|error| panic!("spawn `{example}` help validation: {error}"));
        assert!(
            output.status.success(),
            "documented example is not accepted by the current CLI: `{example}`\n{}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
}
