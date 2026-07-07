#![forbid(unsafe_code)]

use anyhow::Result;
use codex_harness::jsonl_parse_stdin;

fn main() -> Result<()> {
    let value = jsonl_parse_stdin()?;
    println!("{}", serde_json::to_string_pretty(&value)?);
    Ok(())
}
