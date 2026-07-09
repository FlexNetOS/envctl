#![forbid(unsafe_code)]

use anyhow::{anyhow, Result};
use codex_harness::{
    bad_behavior_counts, grant_full_access, policy_decision, record_policy_violation, DecisionKind,
};
use std::env;

fn split_after_double_dash(args: &[String]) -> Result<Vec<String>> {
    let pos = args
        .iter()
        .position(|a| a == "--")
        .ok_or_else(|| anyhow!("missing -- before command"))?;
    Ok(args[pos + 1..].to_vec())
}

fn main() -> Result<()> {
    let args = env::args().skip(1).collect::<Vec<_>>();
    let Some(cmd) = args.first().map(String::as_str) else {
        eprintln!("usage: codex-harness-policy <check|counters|grant-full-access> [--record] -- <argv...>");
        std::process::exit(2);
    };
    match cmd {
        "check" => {
            let record = args.iter().any(|a| a == "--record");
            let argv = split_after_double_dash(&args[1..])?;
            let decision = policy_decision(&argv);
            if record && decision.decision != DecisionKind::Allow {
                record_policy_violation(&decision)?;
            }
            println!("{}", serde_json::to_string_pretty(&decision)?);
            if decision.decision == DecisionKind::Deny {
                std::process::exit(1);
            }
        }
        "counters" => {
            println!("{}", serde_json::to_string_pretty(&bad_behavior_counts()?)?);
        }
        "grant-full-access" => {
            let reason = args
                .get(1)
                .map(String::as_str)
                .unwrap_or("operator requested full access");
            println!(
                "{}",
                serde_json::to_string_pretty(&grant_full_access(reason)?)?
            );
        }
        other => return Err(anyhow!("unknown policy command {other}")),
    }
    Ok(())
}
