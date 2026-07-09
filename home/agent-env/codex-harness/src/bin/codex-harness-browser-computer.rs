#![forbid(unsafe_code)]

use anyhow::{anyhow, Result};
use codex_harness::browser_computer_value;
use std::env;

fn main() -> Result<()> {
    let args = env::args().skip(1).collect::<Vec<_>>();
    let Some(cmd) = args.first().map(String::as_str) else {
        eprintln!("usage: codex-harness-browser-computer <verify>");
        std::process::exit(2);
    };
    match cmd {
        "verify" => {
            let value = browser_computer_value()?;
            println!("{}", serde_json::to_string_pretty(&value)?);
            if !value.get("ok").and_then(|v| v.as_bool()).unwrap_or(false) {
                std::process::exit(1);
            }
        }
        other => return Err(anyhow!("unknown browser-computer command {other}")),
    }
    Ok(())
}
