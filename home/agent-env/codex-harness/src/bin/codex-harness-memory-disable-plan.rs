#![forbid(unsafe_code)]

use anyhow::Result;
use codex_harness::memory_disable_plan_value;

fn main() -> Result<()> {
    let value = memory_disable_plan_value()?;
    println!("{}", serde_json::to_string_pretty(&value)?);
    Ok(())
}
