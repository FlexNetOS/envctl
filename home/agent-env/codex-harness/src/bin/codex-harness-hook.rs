#![forbid(unsafe_code)]

use anyhow::Result;
use codex_harness::{hook_response, read_stdin_string};
use serde_json::Value;

fn main() -> Result<()> {
    let input = read_stdin_string()?;
    let value: Value = if input.trim().is_empty() {
        serde_json::json!({})
    } else {
        serde_json::from_str(&input)?
    };
    let response = hook_response(&value)?;
    println!("{}", serde_json::to_string(&response)?);
    Ok(())
}
