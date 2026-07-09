#![forbid(unsafe_code)]

use anyhow::{anyhow, Result};
use codex_harness::claude_bridge_value_with_auth;
use std::env;

fn main() -> Result<()> {
    let args = env::args().skip(1).collect::<Vec<_>>();
    let Some(cmd) = args.first().map(String::as_str) else {
        eprintln!("usage: codex-harness-claude-bridge <inventory|run> [--allow-default-auth] [--prompt PROMPT]");
        std::process::exit(2);
    };
    let allow_default_auth = args.iter().any(|arg| arg == "--allow-default-auth");
    let prompt = args
        .windows(2)
        .find(|pair| pair[0] == "--prompt")
        .map(|pair| pair[1].clone());
    let value = match cmd {
        "inventory" => claude_bridge_value_with_auth(prompt.as_deref(), false, allow_default_auth)?,
        "run" => claude_bridge_value_with_auth(prompt.as_deref(), true, allow_default_auth)?,
        other => return Err(anyhow!("unknown claude-bridge command {other}")),
    };
    println!("{}", serde_json::to_string_pretty(&value)?);
    if !value.get("ok").and_then(|v| v.as_bool()).unwrap_or(false) {
        std::process::exit(1);
    }
    Ok(())
}
