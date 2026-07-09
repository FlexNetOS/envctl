#![forbid(unsafe_code)]

use anyhow::{anyhow, Result};
use codex_harness::{
    append_ledger, archive_path, browser_computer_value, claude_bridge_value_with_auth,
    grant_full_access, openrouter_probe_value, policy_decision, restore_archive, run_codex_exec,
    run_foreground, run_ollama, spawn_codex_exec, spawn_ollama_run, spawn_supervised,
};
use serde_json::json;
use std::env;
use std::path::PathBuf;

fn split_after_double_dash(args: &[String]) -> Result<(Vec<String>, Vec<String>)> {
    let pos = args
        .iter()
        .position(|a| a == "--")
        .ok_or_else(|| anyhow!("missing -- before command"))?;
    Ok((args[..pos].to_vec(), args[pos + 1..].to_vec()))
}

fn main() -> Result<()> {
    let args = env::args().skip(1).collect::<Vec<_>>();
    let Some(cmd) = args.first().map(String::as_str) else {
        eprintln!("usage: codex-harness-runner <policy-check|run|spawn|archive|restore|ledger-append|grant-full-access|openrouter-probe|claude-bridge|browser-computer|codex-exec|spawn-codex-exec|ollama-run|spawn-ollama-run> [opts] -- <argv...>");
        std::process::exit(2);
    };
    match cmd {
        "grant-full-access" => {
            let reason = if args.len() > 1 {
                args[1..].join(" ")
            } else {
                "operator requested full access for all phases".to_string()
            };
            println!(
                "{}",
                serde_json::to_string_pretty(&grant_full_access(&reason)?)?
            );
        }
        "openrouter-probe" => {
            let mut model = None::<String>;
            let mut prompt = None::<String>;
            let mut i = 1usize;
            while i < args.len() {
                match args[i].as_str() {
                    "--model" => {
                        i += 1;
                        model = Some(
                            args.get(i)
                                .ok_or_else(|| anyhow!("--model missing value"))?
                                .clone(),
                        );
                    }
                    "--prompt" => {
                        i += 1;
                        prompt = Some(
                            args.get(i)
                                .ok_or_else(|| anyhow!("--prompt missing value"))?
                                .clone(),
                        );
                    }
                    _ => {}
                }
                i += 1;
            }
            println!(
                "{}",
                serde_json::to_string_pretty(&openrouter_probe_value(
                    model.as_deref(),
                    prompt.as_deref()
                )?)?
            );
        }
        "claude-bridge" => {
            let execute = args.iter().any(|a| a == "--execute");
            let allow_default_auth = args.iter().any(|a| a == "--allow-default-auth");
            let prompt = args
                .windows(2)
                .find(|pair| pair[0] == "--prompt")
                .map(|pair| pair[1].clone());
            println!(
                "{}",
                serde_json::to_string_pretty(&claude_bridge_value_with_auth(
                    prompt.as_deref(),
                    execute,
                    allow_default_auth
                )?)?
            );
        }
        "browser-computer" => {
            println!(
                "{}",
                serde_json::to_string_pretty(&browser_computer_value()?)?
            );
        }
        "policy-check" => {
            let (_, argv) = split_after_double_dash(&args[1..])?;
            println!("{}", serde_json::to_string_pretty(&policy_decision(&argv))?);
        }
        "run" | "spawn" => {
            let (opts, argv) = split_after_double_dash(&args[1..])?;
            if argv.is_empty() {
                return Err(anyhow!("empty command"));
            }
            let mut cwd = env::current_dir()?;
            let mut i = 0usize;
            while i < opts.len() {
                if opts[i] == "--cwd" {
                    i += 1;
                    cwd = PathBuf::from(opts.get(i).ok_or_else(|| anyhow!("--cwd missing value"))?);
                }
                i += 1;
            }
            if cmd == "run" {
                let code = run_foreground(&cwd, &argv)?;
                println!("{}", json!({"exit_code": code}));
                std::process::exit(code);
            } else {
                let job = spawn_supervised(&cwd, &argv)?;
                println!("{}", serde_json::to_string_pretty(&job)?);
            }
        }
        "archive" => {
            let (opts, argv) = split_after_double_dash(&args[1..])?;
            if argv.len() != 1 {
                return Err(anyhow!("archive expects exactly one path after --"));
            }
            let mut reason = "codex-harness-runner archive".to_string();
            let mut i = 0usize;
            while i < opts.len() {
                if opts[i] == "--reason" {
                    i += 1;
                    reason = opts
                        .get(i)
                        .ok_or_else(|| anyhow!("--reason missing value"))?
                        .clone();
                }
                i += 1;
            }
            println!(
                "{}",
                serde_json::to_string_pretty(&archive_path(&PathBuf::from(&argv[0]), &reason)?)?
            );
        }
        "restore" => {
            let (opts, argv) = split_after_double_dash(&args[1..])?;
            if !argv.is_empty() {
                return Err(anyhow!("restore expects --from and --to options, no argv"));
            }
            let mut from = None;
            let mut to = None;
            let mut i = 0usize;
            while i < opts.len() {
                match opts[i].as_str() {
                    "--from" => {
                        i += 1;
                        from = Some(PathBuf::from(
                            opts.get(i).ok_or_else(|| anyhow!("--from missing value"))?,
                        ));
                    }
                    "--to" => {
                        i += 1;
                        to = Some(PathBuf::from(
                            opts.get(i).ok_or_else(|| anyhow!("--to missing value"))?,
                        ));
                    }
                    _ => {}
                }
                i += 1;
            }
            println!(
                "{}",
                serde_json::to_string_pretty(&restore_archive(
                    &from.ok_or_else(|| anyhow!("--from required"))?,
                    &to.ok_or_else(|| anyhow!("--to required"))?,
                )?)?
            );
        }
        "ledger-append" => {
            let (opts, json_args) = split_after_double_dash(&args[1..])?;
            let mut ledger = None;
            let mut i = 0usize;
            while i < opts.len() {
                if opts[i] == "--ledger" {
                    i += 1;
                    ledger = Some(
                        opts.get(i)
                            .ok_or_else(|| anyhow!("--ledger missing value"))?
                            .clone(),
                    );
                }
                i += 1;
            }
            let ledger = ledger.ok_or_else(|| anyhow!("--ledger required"))?;
            let allowed = [
                "harness.jsonl",
                "processes.jsonl",
                "archive.jsonl",
                "budget.jsonl",
                "decisions.jsonl",
                "research.jsonl",
                "rules.jsonl",
                "policy.jsonl",
                "soul.jsonl",
                "subagents.jsonl",
                "counters.jsonl",
                "model_router.jsonl",
                "model-routing.jsonl",
                "network.jsonl",
                "browser-computer.jsonl",
                "plugins.jsonl",
                "mcp.jsonl",
                "bad-behavior.jsonl",
                "index.jsonl",
                "github.jsonl",
                "memory.jsonl",
            ];
            if !allowed.contains(&ledger.as_str()) {
                return Err(anyhow!("unsupported ledger name"));
            }
            let text = json_args.join(" ");
            let value: serde_json::Value = serde_json::from_str(&text)?;
            append_ledger(&ledger, value)?;
            println!("{}", serde_json::json!({"ledger":ledger,"appended":true}));
        }
        "codex-exec" | "spawn-codex-exec" | "ollama-run" | "spawn-ollama-run" => {
            let (opts, prompt_args) = split_after_double_dash(&args[1..])?;
            let prompt = prompt_args.join(" ");
            if prompt.trim().is_empty() {
                return Err(anyhow!("{cmd} requires a prompt after --"));
            }
            let mut cwd = env::current_dir()?;
            let mut profile = "envctl-harness".to_string();
            let mut model = "gemma4:latest".to_string();
            let mut i = 0usize;
            while i < opts.len() {
                match opts[i].as_str() {
                    "--cwd" => {
                        i += 1;
                        cwd = PathBuf::from(
                            opts.get(i).ok_or_else(|| anyhow!("--cwd missing value"))?,
                        );
                    }
                    "--profile" => {
                        i += 1;
                        profile = opts
                            .get(i)
                            .ok_or_else(|| anyhow!("--profile missing value"))?
                            .clone();
                    }
                    "--model" => {
                        i += 1;
                        model = opts
                            .get(i)
                            .ok_or_else(|| anyhow!("--model missing value"))?
                            .clone();
                    }
                    _ => {}
                }
                i += 1;
            }
            match cmd {
                "codex-exec" => {
                    let code = run_codex_exec(&cwd, &profile, &prompt)?;
                    std::process::exit(code);
                }
                "spawn-codex-exec" => {
                    let job = spawn_codex_exec(&cwd, &profile, &prompt)?;
                    println!("{}", serde_json::to_string_pretty(&job)?);
                }
                "ollama-run" => {
                    let code = run_ollama(&cwd, &model, &prompt)?;
                    std::process::exit(code);
                }
                "spawn-ollama-run" => {
                    let job = spawn_ollama_run(&cwd, &model, &prompt)?;
                    println!("{}", serde_json::to_string_pretty(&job)?);
                }
                _ => unreachable!(),
            }
        }
        other => return Err(anyhow!("unknown runner command {other}")),
    }
    Ok(())
}
