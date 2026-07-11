#![forbid(unsafe_code)]

use anyhow::{Result, anyhow};
use codex_harness::{append_ledger, codex_harness_dir, redact, state_dir, utc_now};
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

const ACTIVE_CATALOG: &str = "/home/flexnetos/.codex/model-catalog.json";

const REQUIRED_CATALOG_MODELS: &[&str] = &[
    "gpt-5.4",
    "gpt-5.4-mini",
    "gpt-5.4-nano",
    "gpt-5.3-codex",
    "gpt-5.3-codex-spark",
    "gpt-5.6-sol",
    "gpt-5.6-terra",
    "gpt-5.6-luna",
    "o3",
    "o3-pro",
    "o3-mini",
    "o4-mini",
    "gpt-4.1",
    "gpt-4.1-mini",
    "gpt-4.1-nano",
    "gpt-4o",
    "gpt-4o-mini",
];

const DEFAULT_PROBE_MODELS: &[&str] = &[
    "gpt-5.6-sol",
    "gpt-5.6-terra",
    "gpt-5.6-luna",
    "gpt-5.4",
    "gpt-5.4-mini",
    "gpt-5.4-nano",
    "gpt-5.3-codex",
    "gpt-5.3-codex-spark",
    "o3",
    "o3-pro",
    "o3-mini",
    "o4-mini",
    "gpt-4.1",
    "gpt-4.1-mini",
    "gpt-4.1-nano",
    "gpt-4o",
    "gpt-4o-mini",
];

const MUST_PASS_MODELS: &[&str] = &["gpt-5.4", "gpt-5.4-mini", "gpt-5.3-codex-spark"];

const ACCOUNT_GATED_MODELS: &[&str] = &[
    "gpt-5.6-sol",
    "gpt-5.6-terra",
    "gpt-5.6-luna",
    "gpt-5.4-nano",
    "gpt-5.3-codex",
    "o3",
    "o3-pro",
    "o3-mini",
    "o4-mini",
    "gpt-4.1",
    "gpt-4.1-mini",
    "gpt-4.1-nano",
    "gpt-4o",
    "gpt-4o-mini",
];

fn read_catalog(path: &Path) -> Result<BTreeSet<String>> {
    let value: Value = serde_json::from_str(&fs::read_to_string(path)?)?;
    let models = value
        .get("models")
        .and_then(Value::as_array)
        .ok_or_else(|| anyhow!("catalog lacks models array: {}", path.display()))?;
    Ok(models
        .iter()
        .filter_map(|model| model.get("slug").and_then(Value::as_str))
        .map(str::to_owned)
        .collect())
}

fn catalog_inventory() -> Result<Value> {
    let active = read_catalog(Path::new(ACTIVE_CATALOG))?;
    let harness_path = codex_harness_dir().join("model-catalog/model-catalog.json");
    let harness = if harness_path.exists() {
        read_catalog(&harness_path)?
    } else {
        BTreeSet::new()
    };
    let required = REQUIRED_CATALOG_MODELS
        .iter()
        .map(|model| {
            json!({
                "model": model,
                "active_catalog": active.contains(*model),
                "harness_catalog": harness.contains(*model),
            })
        })
        .collect::<Vec<_>>();
    let missing_active = REQUIRED_CATALOG_MODELS
        .iter()
        .filter(|model| !active.contains(**model))
        .copied()
        .collect::<Vec<_>>();
    let missing_harness = REQUIRED_CATALOG_MODELS
        .iter()
        .filter(|model| !harness.contains(**model))
        .copied()
        .collect::<Vec<_>>();
    Ok(json!({
        "ok": missing_active.is_empty() && missing_harness.is_empty(),
        "ts_utc": utc_now(),
        "active_catalog": ACTIVE_CATALOG,
        "active_catalog_count": active.len(),
        "harness_catalog": harness_path,
        "harness_catalog_count": harness.len(),
        "required": required,
        "missing_active": missing_active,
        "missing_harness": missing_harness,
    }))
}

fn remove_secret_env(command: &mut Command) {
    for key in [
        "OPENROUTER_API_KEY",
        "OPENAI_API_KEY",
        "ANTHROPIC_API_KEY",
        "CLAUDE_API_KEY",
        "GITHUB_TOKEN",
        "GH_TOKEN",
    ] {
        command.env_remove(key);
    }
}

fn marker_for(model: &str) -> String {
    let safe = model
        .chars()
        .map(|ch| if ch.is_ascii_alphanumeric() { ch } else { '_' })
        .collect::<String>()
        .to_ascii_uppercase();
    format!("MODEL_ACCESS_OK_{safe}")
}

fn probe_model(model: &str) -> Result<Value> {
    let marker = marker_for(model);
    let prompt = format!("Do not use tools. Reply exactly: {marker}");
    let mut command = Command::new("codex");
    command.args([
        "exec",
        "--ephemeral",
        "--json",
        "--color",
        "never",
        "-m",
        model,
        &prompt,
    ]);
    remove_secret_env(&mut command);
    let output = command.output()?;
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    let mut agent_message = String::new();
    let mut error_message = String::new();
    let mut completed = false;
    for line in stdout.lines() {
        if line.trim().is_empty() {
            continue;
        }
        let Ok(value) = serde_json::from_str::<Value>(line) else {
            continue;
        };
        if value.get("type").and_then(Value::as_str) == Some("turn.completed") {
            completed = true;
        }
        if value.get("type").and_then(Value::as_str) == Some("error") {
            error_message = value
                .get("message")
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_string();
        }
        if let Some(item) = value.get("item") {
            if item.get("type").and_then(Value::as_str) == Some("agent_message") {
                agent_message = item
                    .get("text")
                    .and_then(Value::as_str)
                    .unwrap_or_default()
                    .to_string();
            }
            if item.get("type").and_then(Value::as_str) == Some("error") && error_message.is_empty()
            {
                error_message = item
                    .get("message")
                    .and_then(Value::as_str)
                    .unwrap_or_default()
                    .to_string();
            }
        }
    }
    let exit_code = output.status.code().unwrap_or(1);
    let ok = exit_code == 0 && completed && agent_message.trim() == marker;
    let account_gated =
        !ok && error_message.contains("not supported when using Codex with a ChatGPT account");
    let value = json!({
        "model": model,
        "marker": marker,
        "ok": ok,
        "account_gated": account_gated,
        "exit_code": exit_code,
        "turn_completed": completed,
        "agent_message_redacted": redact(&agent_message),
        "error_redacted": redact(&error_message),
        "stderr_redacted": redact(&stderr),
        "stdout_line_count": stdout.lines().count(),
    });
    append_ledger(
        "model-routing.jsonl",
        json!({
            "event": "model_access_probe",
            "decision": if ok || account_gated { "record" } else { "deny" },
            "model": model,
            "ok": ok,
            "account_gated": account_gated,
            "exit_code": exit_code,
            "error_redacted": redact(&error_message),
        }),
    )?;
    Ok(value)
}

fn proof_path() -> PathBuf {
    state_dir().join("model-access-proof.json")
}

fn evaluate_proof(probes: &[Value]) -> Value {
    let mut by_model: BTreeMap<String, Value> = BTreeMap::new();
    for probe in probes {
        if let Some(model) = probe.get("model").and_then(Value::as_str) {
            by_model.insert(model.to_string(), probe.clone());
        }
    }
    let must_pass = MUST_PASS_MODELS
        .iter()
        .map(|model| {
            let ok = by_model
                .get(*model)
                .and_then(|probe| probe.get("ok"))
                .and_then(Value::as_bool)
                .unwrap_or(false);
            json!({"model": model, "ok": ok})
        })
        .collect::<Vec<_>>();
    let account_gated = ACCOUNT_GATED_MODELS
        .iter()
        .map(|model| {
            let gated = by_model
                .get(*model)
                .and_then(|probe| probe.get("account_gated"))
                .and_then(Value::as_bool)
                .unwrap_or(false);
            let ok = by_model
                .get(*model)
                .and_then(|probe| probe.get("ok"))
                .and_then(Value::as_bool)
                .unwrap_or(false);
            json!({"model": model, "ok": ok, "account_gated": gated})
        })
        .collect::<Vec<_>>();
    let must_pass_ok = must_pass
        .iter()
        .all(|row| row.get("ok").and_then(Value::as_bool).unwrap_or(false));
    let account_gated_documented = account_gated.iter().all(|row| {
        row.get("ok").and_then(Value::as_bool).unwrap_or(false)
            || row
                .get("account_gated")
                .and_then(Value::as_bool)
                .unwrap_or(false)
    });
    json!({
        "must_pass": must_pass,
        "account_gated": account_gated,
        "must_pass_ok": must_pass_ok,
        "account_gated_documented": account_gated_documented,
        "ok": must_pass_ok && account_gated_documented,
    })
}

fn run_probe(models: &[String]) -> Result<Value> {
    let inventory = catalog_inventory()?;
    let mut probes = Vec::new();
    for model in models {
        probes.push(probe_model(model)?);
    }
    let evaluation = evaluate_proof(&probes);
    let value = json!({
        "ts_utc": utc_now(),
        "catalog": inventory,
        "probes": probes,
        "evaluation": evaluation,
    });
    fs::create_dir_all(state_dir())?;
    fs::write(proof_path(), serde_json::to_vec_pretty(&value)?)?;
    Ok(value)
}

fn print_usage() {
    eprintln!("usage: codex-harness-model-access inventory | probe [model ...] | summary");
}

fn main() -> Result<()> {
    let mut args = env::args().skip(1).collect::<Vec<_>>();
    if args.is_empty() {
        print_usage();
        return Err(anyhow!("missing command"));
    }
    let command = args.remove(0);
    let value = match command.as_str() {
        "inventory" => catalog_inventory()?,
        "probe" => {
            let models = if args.is_empty() {
                DEFAULT_PROBE_MODELS
                    .iter()
                    .map(|model| model.to_string())
                    .collect()
            } else {
                args
            };
            run_probe(&models)?
        }
        "summary" => {
            let text = fs::read_to_string(proof_path())?;
            let value: Value = serde_json::from_str(&text)?;
            json!({
                "proof_path": proof_path(),
                "evaluation": value.get("evaluation").cloned().unwrap_or(Value::Null),
                "catalog_ok": value.pointer("/catalog/ok").and_then(Value::as_bool).unwrap_or(false),
            })
        }
        _ => {
            print_usage();
            return Err(anyhow!("unknown command: {command}"));
        }
    };
    println!("{}", serde_json::to_string_pretty(&value)?);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn probe(model: &str, ok: bool, account_gated: bool) -> Value {
        json!({
            "model": model,
            "ok": ok,
            "account_gated": account_gated,
        })
    }

    #[test]
    fn marker_normalizes_model_slugs() {
        assert_eq!(marker_for("gpt-5.6-sol"), "MODEL_ACCESS_OK_GPT_5_6_SOL");
        assert_eq!(marker_for("o4-mini"), "MODEL_ACCESS_OK_O4_MINI");
    }

    #[test]
    fn evaluation_accepts_required_passes_and_documented_gates() {
        let probes = MUST_PASS_MODELS
            .iter()
            .map(|model| probe(model, true, false))
            .chain(
                ACCOUNT_GATED_MODELS
                    .iter()
                    .map(|model| probe(model, false, true)),
            )
            .collect::<Vec<_>>();
        let evaluation = evaluate_proof(&probes);
        assert!(
            evaluation
                .get("must_pass_ok")
                .and_then(Value::as_bool)
                .unwrap_or(false)
        );
        assert!(
            evaluation
                .get("account_gated_documented")
                .and_then(Value::as_bool)
                .unwrap_or(false)
        );
        assert!(
            evaluation
                .get("ok")
                .and_then(Value::as_bool)
                .unwrap_or(false)
        );
    }

    #[test]
    fn evaluation_rejects_unexplained_gated_model_failure() {
        let probes = MUST_PASS_MODELS
            .iter()
            .map(|model| probe(model, true, false))
            .chain(ACCOUNT_GATED_MODELS.iter().map(|model| {
                if *model == "o3" {
                    probe(model, false, false)
                } else {
                    probe(model, false, true)
                }
            }))
            .collect::<Vec<_>>();
        let evaluation = evaluate_proof(&probes);
        assert!(
            !evaluation
                .get("account_gated_documented")
                .and_then(Value::as_bool)
                .unwrap_or(true)
        );
        assert!(
            !evaluation
                .get("ok")
                .and_then(Value::as_bool)
                .unwrap_or(true)
        );
    }
}
