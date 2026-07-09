#![forbid(unsafe_code)]

use anyhow::{anyhow, Result};
use codex_harness::memory_export_redacted;
use std::env;

fn main() -> Result<()> {
    let args = env::args().skip(1).collect::<Vec<_>>();
    let mut limit = Some(200usize);
    let mut i = 0usize;
    while i < args.len() {
        match args[i].as_str() {
            "--all" => limit = None,
            "--limit" => {
                i += 1;
                limit = Some(
                    args.get(i)
                        .ok_or_else(|| anyhow!("--limit missing value"))?
                        .parse()?,
                );
            }
            _ => {}
        }
        i += 1;
    }
    let summary = memory_export_redacted(limit)?;
    eprintln!("{}", serde_json::to_string_pretty(&summary)?);
    Ok(())
}
