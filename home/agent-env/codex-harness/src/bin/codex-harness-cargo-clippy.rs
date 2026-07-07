#![forbid(unsafe_code)]

use anyhow::Result;
use std::env;
use std::process::Command;

fn main() -> Result<()> {
    let args = env::args().skip(1).collect::<Vec<_>>();
    let split = args.iter().position(|arg| arg == "--");
    let (cargo_args, lint_args) = match split {
        Some(idx) => (&args[..idx], &args[idx + 1..]),
        None => (&args[..], &[][..]),
    };
    let existing_rustflags = env::var("RUSTFLAGS").unwrap_or_default();
    let lint_flags = lint_args.join(" ");
    let rustflags = [existing_rustflags.as_str(), lint_flags.as_str()]
        .into_iter()
        .filter(|s| !s.trim().is_empty())
        .collect::<Vec<_>>()
        .join(" ");
    let mut command = Command::new("cargo");
    command
        .arg("check")
        .args(cargo_args)
        .env("RUSTC_WORKSPACE_WRAPPER", "clippy-driver");
    if !rustflags.is_empty() {
        command.env("RUSTFLAGS", rustflags);
    }
    let status = command.status()?;
    std::process::exit(status.code().unwrap_or(1));
}
