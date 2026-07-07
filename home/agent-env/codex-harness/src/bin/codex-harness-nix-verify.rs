#![forbid(unsafe_code)]

use anyhow::Result;
use codex_harness::nix_verify_value;

fn main() -> Result<()> {
    let value = nix_verify_value();
    println!("{}", serde_json::to_string_pretty(&value)?);
    if !value.get("ok").and_then(|v| v.as_bool()).unwrap_or(false) {
        std::process::exit(1);
    }
    Ok(())
}
