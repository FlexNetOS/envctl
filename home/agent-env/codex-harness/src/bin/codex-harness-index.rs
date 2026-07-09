#![forbid(unsafe_code)]

use anyhow::{anyhow, Result};
use codex_harness::index_integrity_check;
use std::env;

fn main() -> Result<()> {
    let args = env::args().skip(1).collect::<Vec<_>>();
    if args == ["integrity", "check"] {
        let value = index_integrity_check()?;
        println!("{}", serde_json::to_string_pretty(&value)?);
        if !value.get("ok").and_then(|v| v.as_bool()).unwrap_or(false) {
            std::process::exit(1);
        }
        return Ok(());
    }
    Err(anyhow!("usage: codex-harness-index integrity check"))
}
