#![forbid(unsafe_code)]

use anyhow::Result;
use codex_harness::memory_audit_value;

fn main() -> Result<()> {
    let value = memory_audit_value()?;
    println!("{}", serde_json::to_string_pretty(&value)?);
    Ok(())
}
