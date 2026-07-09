#![forbid(unsafe_code)]

use anyhow::{anyhow, Result};
use codex_harness::openrouter_probe_value;
use std::env;

fn main() -> Result<()> {
    let args = env::args().skip(1).collect::<Vec<_>>();
    let Some(cmd) = args.first().map(String::as_str) else {
        eprintln!("usage: codex-harness-openrouter-shim <probe> [--model MODEL] [--prompt PROMPT]");
        std::process::exit(2);
    };
    match cmd {
        "probe" => {
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
            let value = openrouter_probe_value(model.as_deref(), prompt.as_deref())?;
            println!("{}", serde_json::to_string_pretty(&value)?);
            if !value.get("ok").and_then(|v| v.as_bool()).unwrap_or(false) {
                std::process::exit(1);
            }
        }
        other => return Err(anyhow!("unknown openrouter-shim command {other}")),
    }
    Ok(())
}
