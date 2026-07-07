#![forbid(unsafe_code)]

use anyhow::{anyhow, Result};
use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

fn rust_files() -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    let lib = PathBuf::from("src/lib.rs");
    if lib.exists() {
        files.push(lib);
    }
    let bin_dir = PathBuf::from("src/bin");
    if bin_dir.exists() {
        for entry in fs::read_dir(bin_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("rs") {
                files.push(path);
            }
        }
    }
    files.sort();
    if files.is_empty() {
        return Err(anyhow!("no Rust files found under src"));
    }
    Ok(files)
}

fn main() -> Result<()> {
    let mut forwarded = Vec::new();
    for arg in env::args().skip(1) {
        if arg == "--all" || arg == "--all-features" || arg == "--all-targets" {
            continue;
        }
        forwarded.push(arg);
    }
    let files = rust_files()?;
    let status = Command::new("rustfmt")
        .args(["--edition", "2021"])
        .args(&forwarded)
        .args(files)
        .status()?;
    std::process::exit(status.code().unwrap_or(1));
}
