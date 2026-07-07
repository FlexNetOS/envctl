#![forbid(unsafe_code)]

use anyhow::Result;
use codex_harness::halt_jobs;

fn main() -> Result<()> {
    let dry_run = std::env::args().any(|a| a == "--dry-run");
    let value = halt_jobs(dry_run)?;
    println!("{}", serde_json::to_string_pretty(&value)?);
    Ok(())
}
