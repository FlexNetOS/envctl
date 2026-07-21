//! Integration tests for the stable `envctl env --toolchains` surface.
//! The flag now projects the sole installed-runtime owner: the invoking user's
//! Nix profile. Envctl layout variables remain available for data/config, but
//! no workspace executable tree or language-manager prefix enters PATH.
use std::path::{Path, PathBuf};
use std::process::Command;

fn bin() -> &'static str {
    env!("CARGO_BIN_EXE_envctl")
}

fn fixture_dir() -> PathBuf {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let dir = std::env::temp_dir().join(format!("envctl-env-it-{nanos}"));
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(
        dir.join(".meta.yaml"),
        "projects:\n  envctl:\n    repo: git@github.com:FlexNetOS/envctl.git\n",
    )
    .unwrap();
    dir
}

fn run(root: &Path, json: bool) -> String {
    let mut command = Command::new(bin());
    command
        .arg("env")
        .arg("--toolchains")
        .arg("--meta-file")
        .arg(root.join(".meta.yaml"))
        .env("HOME", root.join("real-home"))
        .env_remove("ENVCTL_REAL_HOME");
    if json {
        command.arg("--json");
    }
    let output = command.output().expect("run envctl env");
    assert!(
        output.status.success(),
        "envctl env failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).unwrap()
}

#[test]
fn toolchains_shell_exports_profile_only_path_and_canonical_state() {
    let root = fixture_dir();
    let r = root.to_string_lossy();
    let profile = root.join("real-home/.nix-profile");
    let expected_path = format!(
        "{}:{}",
        profile.join("toolbin").display(),
        profile.join("bin").display()
    );
    let output = run(&root, false);

    for expected in [
        format!("export ENVCTL_PROFILE_ROOT='{}'", profile.display()),
        format!("export PATH='{expected_path}'"),
        format!("export ENVCTL_BIN_DIR='{r}/usr/bin'"),
        format!("export ENVCTL_REPO_STORE='{r}/var/lib/envctl/repos'"),
        format!("export ENVCTL_XDG_DATA_HOME='{r}/var/lib'"),
        format!("export ENVCTL_XDG_STATE_HOME='{r}/var/lib'"),
        format!("export ENVCTL_XDG_CACHE_HOME='{r}/var/cache'"),
        format!("export XDG_DATA_HOME='{r}/var/lib'"),
        format!("export XDG_STATE_HOME='{r}/var/lib'"),
        format!("export XDG_CACHE_HOME='{r}/var/cache'"),
        format!("export OLLAMA_MODELS='{r}/var/lib/ollama/models'"),
    ] {
        assert!(output.contains(&expected), "missing {expected}:\n{output}");
    }

    for forbidden in [
        "ENVCTL_LOCAL=",
        "ENVCTL_LOCAL_BIN=",
        "ENVCTL_LEGACY_TOOLCHAINS=",
        "BUN_INSTALL=",
        "MISE_DATA_DIR=",
        "CARGO_HOME=",
        "RUSTUP_HOME=",
        "UV_TOOL_DIR=",
        "OLLAMA_LIBRARY_PATH=",
        "LIBCLANG_PATH=",
        "GCC_PATH=",
        "HELIX_RUNTIME=",
        "LD_LIBRARY_PATH=",
    ] {
        assert!(
            !output.contains(forbidden),
            "second runtime owner leaked through {forbidden}:\n{output}"
        );
    }
}

#[test]
fn toolchains_json_matches_profile_only_shell_contract() {
    let root = fixture_dir();
    let r = root.to_string_lossy();
    let profile = root.join("real-home/.nix-profile");
    let expected_path = format!(
        "{}:{}",
        profile.join("toolbin").display(),
        profile.join("bin").display()
    );
    let output = run(&root, true);
    let value: serde_json::Value = serde_json::from_str(&output).unwrap();

    assert_eq!(
        value["ENVCTL_PROFILE_ROOT"],
        profile.to_string_lossy().as_ref()
    );
    assert_eq!(value["PATH"], expected_path);
    assert_eq!(value["ENVCTL_BIN_DIR"], format!("{r}/usr/bin"));
    assert_eq!(value["ENVCTL_XDG_DATA_HOME"], format!("{r}/var/lib"));
    assert_eq!(value["ENVCTL_XDG_STATE_HOME"], format!("{r}/var/lib"));
    assert_eq!(value["ENVCTL_XDG_CACHE_HOME"], format!("{r}/var/cache"));
    assert_eq!(value["XDG_DATA_HOME"], format!("{r}/var/lib"));
    assert_eq!(value["XDG_STATE_HOME"], format!("{r}/var/lib"));
    assert_eq!(value["XDG_CACHE_HOME"], format!("{r}/var/cache"));

    for forbidden in [
        "ENVCTL_LOCAL",
        "ENVCTL_LOCAL_BIN",
        "ENVCTL_LEGACY_TOOLCHAINS",
        "BUN_INSTALL",
        "MISE_DATA_DIR",
        "CARGO_HOME",
        "RUSTUP_HOME",
        "UV_TOOL_DIR",
        "OLLAMA_LIBRARY_PATH",
        "LIBCLANG_PATH",
        "GCC_PATH",
        "HELIX_RUNTIME",
        "LD_LIBRARY_PATH",
    ] {
        assert!(
            value.get(forbidden).is_none(),
            "second runtime owner leaked through {forbidden}: {value}"
        );
    }
}
