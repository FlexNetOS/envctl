#![forbid(unsafe_code)]

use anyhow::{anyhow, Result};
use codex_harness::github_guard_check;
use std::env;

fn main() -> Result<()> {
    let args = env::args().skip(1).collect::<Vec<_>>();
    let Some(cmd) = args.first().map(String::as_str) else {
        eprintln!(
            "usage: codex-harness-github-guard <check|run> [--decision-id ID] -- <gh|git> ..."
        );
        std::process::exit(2);
    };
    let execute = match cmd {
        "check" => false,
        "run" => true,
        other => return Err(anyhow!("unknown github-guard command {other}")),
    };
    let mut decision_id = None::<String>;
    let mut sep = None;
    let mut i = 1usize;
    while i < args.len() {
        match args[i].as_str() {
            "--decision-id" => {
                i += 1;
                decision_id = Some(
                    args.get(i)
                        .ok_or_else(|| anyhow!("--decision-id missing value"))?
                        .clone(),
                );
            }
            "--" => {
                sep = Some(i);
                break;
            }
            _ => {}
        }
        i += 1;
    }
    let sep = sep.ok_or_else(|| anyhow!("missing -- before gh/git command"))?;
    let argv = args[sep + 1..].to_vec();
    let value = github_guard_check(&argv, decision_id.as_deref(), execute)?;
    println!("{}", serde_json::to_string_pretty(&value)?);
    if !value.get("ok").and_then(|v| v.as_bool()).unwrap_or(false) {
        std::process::exit(1);
    }
    Ok(())
}
