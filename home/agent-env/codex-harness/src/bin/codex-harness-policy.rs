#![forbid(unsafe_code)]

use anyhow::{anyhow, Result};
use codex_harness::{
    bad_behavior_counts, policy_decision, record_full_access_receipt, record_policy_violation,
    session_capability_status, set_session_capability, set_session_capability_preset, DecisionKind,
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
        eprintln!(
            "usage: codex-harness-policy <check|counters|session|record-full-access-receipt> ..."
        );
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
        "session" => match args.get(1).map(String::as_str) {
            Some("status") => {
                println!("{}", serde_json::to_string_pretty(&session_capability_status())?);
            }
            Some("preset") => {
                let preset = args
                    .get(2)
                    .ok_or_else(|| anyhow!("session preset requires full or restricted"))?;
                println!(
                    "{}",
                    serde_json::to_string_pretty(&set_session_capability_preset(preset)?)?
                );
            }
            Some("set") => {
                let capability = args
                    .get(2)
                    .ok_or_else(|| anyhow!("session set requires a capability"))?;
                let enabled = match args.get(3).map(String::as_str) {
                    Some("on" | "true" | "enable" | "enabled") => true,
                    Some("off" | "false" | "disable" | "disabled") => false,
                    _ => return Err(anyhow!("session set requires on or off")),
                };
                println!(
                    "{}",
                    serde_json::to_string_pretty(&set_session_capability(capability, enabled)?)?
                );
            }
            _ => {
                return Err(anyhow!(
                    "usage: codex-harness-policy session <status|preset full|preset restricted|set CAPABILITY on|off>"
                ))
            }
        },
        "record-full-access-receipt" => {
            let reason = args
                .get(1)
                .map(String::as_str)
                .unwrap_or("operator recorded current full-access selection");
            println!(
                "{}",
                serde_json::to_string_pretty(&record_full_access_receipt(reason)?)?
            );
        }
        "grant-full-access" => {
            return Err(anyhow!(
                "grant-full-access was retired: use the built-in /permissions command, then run `codex-harness-policy session status`"
            ));
        }
        other => return Err(anyhow!("unknown policy command {other}")),
    }
    Ok(())
}
