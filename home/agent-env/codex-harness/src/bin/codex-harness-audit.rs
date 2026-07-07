#![forbid(unsafe_code)]

use anyhow::Result;
use codex_harness::audit_value;

fn main() -> Result<()> {
    let value = audit_value();
    println!("{}", serde_json::to_string_pretty(&value)?);
    if !value.get("ok").and_then(|v| v.as_bool()).unwrap_or(false) {
        std::process::exit(1);
    }
    Ok(())
}
