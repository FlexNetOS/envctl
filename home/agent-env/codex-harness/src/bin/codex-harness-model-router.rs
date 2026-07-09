#![forbid(unsafe_code)]

use anyhow::{anyhow, Result};
use codex_harness::{route_model_tasks, sample_model_tasks};
use std::env;

fn route_tasks_from_args(args: &[String]) -> Result<Vec<String>> {
    let mut tasks = Vec::new();
    let mut i = 0usize;
    while i < args.len() {
        match args[i].as_str() {
            "--" => {}
            "--task" => {
                i += 1;
                let task = args.get(i).ok_or_else(|| anyhow!("--task missing value"))?;
                tasks.push(task.clone());
            }
            "--write-scope" | "--profile" | "--provider" | "--model" => {
                i += 1;
                args.get(i)
                    .ok_or_else(|| anyhow!("{} missing value", args[i - 1]))?;
            }
            arg if arg.starts_with("--") => {
                return Err(anyhow!("unknown model-router option: {arg}"));
            }
            task => tasks.push(task.to_string()),
        }
        i += 1;
    }
    if tasks.is_empty() {
        return Err(anyhow!(
            "usage: codex-harness-model-router route [--task <task> | <task>] [task...]"
        ));
    }
    Ok(tasks)
}

fn main() -> Result<()> {
    let args = env::args().skip(1).collect::<Vec<_>>();
    let tasks = if args == ["sample", "tasks"] {
        sample_model_tasks()
    } else if args.first().map(String::as_str) == Some("route") {
        route_tasks_from_args(&args[1..])?
    } else if args.is_empty() {
        return Err(anyhow!(
            "usage: codex-harness-model-router sample tasks | route [--task <task> | <task>] [task...]"
        ));
    } else {
        vec![args.join(" ")]
    };
    let value = route_model_tasks(&tasks)?;
    println!("{}", serde_json::to_string_pretty(&value)?);
    Ok(())
}
