#![forbid(unsafe_code)]

use anyhow::Result;
use codex_harness::prompt_review::{assert_prompt_review_ok, review_full_access_prompt_path};
use std::env;
use std::path::{Path, PathBuf};

fn default_prompt_path() -> PathBuf {
    let cwd = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    for ancestor in cwd.ancestors() {
        let candidate = ancestor
            .join(".codex/prompts/prompt:codex-gpt-harness-v3-full-access-no-sandbox.prompt.md");
        if candidate.exists() {
            return candidate;
        }
    }
    Path::new("/home/flexnetos/meta/src/envctl/.codex/prompts/prompt:codex-gpt-harness-v3-full-access-no-sandbox.prompt.md").to_path_buf()
}

fn main() -> Result<()> {
    let prompt = env::args_os()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(default_prompt_path);
    let report = review_full_access_prompt_path(&prompt)?;
    println!("{}", serde_json::to_string_pretty(&report)?);
    assert_prompt_review_ok(&report)?;
    Ok(())
}
