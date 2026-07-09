#![forbid(unsafe_code)]

use anyhow::{anyhow, Result};
use codex_harness::{route_model_tasks, sample_model_tasks};
use std::env;

fn main() -> Result<()> {
    let args = env::args().skip(1).collect::<Vec<_>>();
    let tasks = if args == ["sample", "tasks"] {
        sample_model_tasks()
    } else if args.first().map(String::as_str) == Some("route") {
        let mut tasks = args.into_iter().skip(1).collect::<Vec<_>>();
        if tasks.is_empty() {
            return Err(anyhow!(
                "usage: codex-harness-model-router route <task> [task...]"
            ));
        }
        if tasks.len() > 1 && tasks.iter().any(|t| t == "--") {
            tasks.retain(|t| t != "--");
        }
        tasks
    } else if args.is_empty() {
        return Err(anyhow!(
            "usage: codex-harness-model-router sample tasks | route <task> [task...]"
        ));
    } else {
        vec![args.join(" ")]
    };
    let value = route_model_tasks(&tasks)?;
    println!("{}", serde_json::to_string_pretty(&value)?);
    Ok(())
}
