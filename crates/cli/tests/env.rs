//! Integration tests for `envctl env --toolchains`. Drives the real binary
//! against a fixture `.meta.yaml` and asserts the meta-hosted install layout:
//! canonical exposure/state under `.local/{bin,lib,share,state,cache,tmp,opt}`,
//! plus legacy manager stores under `.toolchains`. In particular RUSTUP_HOME
//! travels with CARGO_HOME so an `eval "$(envctl env --toolchains)"` shell points
//! rustup at the meta-owned compatibility store, not user-global ~/.rustup.
use std::path::{Path, PathBuf};
use std::process::Command;

fn bin() -> &'static str {
    env!("CARGO_BIN_EXE_envctl")
}

/// Write a fixture `.meta.yaml` into a unique temp dir; return the dir (= meta root).
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
    let mut c = Command::new(bin());
    c.arg("env")
        .arg("--toolchains")
        .arg("--meta-file")
        .arg(root.join(".meta.yaml"));
    if json {
        c.arg("--json");
    }
    let out = c.output().expect("run envctl env");
    assert!(
        out.status.success(),
        "envctl env failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8(out.stdout).unwrap()
}

/// Shell form exposes the canonical meta `.local` layout and pairs RUSTUP_HOME
/// with CARGO_HOME under the legacy `.toolchains/` manager store. Without
/// RUSTUP_HOME the eval-seam silently leaks to ~/.rustup.
#[test]
fn toolchains_shell_exports_meta_local_layout_and_rustup_home() {
    let root = fixture_dir();
    let r = root.to_string_lossy();
    let out = run(&root, false);
    assert!(
        out.contains(&format!("export ENVCTL_LOCAL='{r}/.local'")),
        "missing canonical meta .local prefix:\n{out}"
    );
    assert!(
        out.contains(&format!("export ENVCTL_BIN_DIR='{r}/.local/bin'")),
        "missing canonical meta .local/bin export:\n{out}"
    );
    assert!(
        out.contains(&format!(
            "export ENVCTL_REPO_STORE='{r}/.local/share/envctl/repos'"
        )),
        "missing canonical envctl repo store export:\n{out}"
    );
    assert!(
        out.contains(&format!(
            "export ENVCTL_LEGACY_TOOLCHAINS='{r}/.toolchains'"
        )),
        "missing legacy toolchains compatibility export:\n{out}"
    );
    assert!(
        out.contains(&format!("export CARGO_HOME='{r}/.toolchains/cargo'")),
        "missing meta CARGO_HOME export:\n{out}"
    );
    assert!(
        out.contains(&format!("export RUSTUP_HOME='{r}/.toolchains/rustup'")),
        "RUSTUP_HOME must travel with CARGO_HOME (meta-owned rustup store):\n{out}"
    );
    assert!(
        out.contains(&format!(
            "export OLLAMA_LIBRARY_PATH='{r}/.toolchains/ollama/lib/ollama'"
        )),
        "OLLAMA_LIBRARY_PATH must redirect ollama at the meta-owned GPU runner libs:\n{out}"
    );
    assert!(
        out.contains(&format!("export LIBCLANG_PATH='{r}/.toolchains/llvm/lib'")),
        "LIBCLANG_PATH must point at the meta-owned LLVM/clang lib dir:\n{out}"
    );
    assert!(
        out.contains(&format!("export GCC_PATH='{r}/.toolchains/libgccjit/lib'")),
        "GCC_PATH must point at the meta-owned libgccjit lib dir:\n{out}"
    );
    assert!(
        out.contains(&format!(
            "export HELIX_RUNTIME='{r}/.toolchains/helix/runtime'"
        )),
        "HELIX_RUNTIME must point at the meta-owned helix tree-sitter runtime dir:\n{out}"
    );
    assert!(
        out.contains(&format!(
            "export PATH=\"{r}/.local/bin:{r}/.toolchains/.bun/bin:{r}/.toolchains/cargo/bin:{r}/.toolchains/uv/tools/bin:$PATH\""
        )),
        "PATH must put canonical meta .local/bin ahead of legacy manager bins:\n{out}"
    );
}

/// JSON form carries the layout variables and RUSTUP_HOME too, so machine
/// consumers see the same seam.
#[test]
fn toolchains_json_carries_meta_local_layout_and_rustup_home() {
    let root = fixture_dir();
    let out = run(&root, true);
    let v: serde_json::Value = serde_json::from_str(&out).unwrap();
    let r = root.to_string_lossy();
    assert_eq!(
        v["ENVCTL_LOCAL"].as_str(),
        Some(format!("{r}/.local").as_str()),
        "json ENVCTL_LOCAL"
    );
    assert_eq!(
        v["ENVCTL_BIN_DIR"].as_str(),
        Some(format!("{r}/.local/bin").as_str()),
        "json ENVCTL_BIN_DIR"
    );
    assert_eq!(
        v["ENVCTL_REPO_STORE"].as_str(),
        Some(format!("{r}/.local/share/envctl/repos").as_str()),
        "json ENVCTL_REPO_STORE"
    );
    assert_eq!(
        v["ENVCTL_LEGACY_TOOLCHAINS"].as_str(),
        Some(format!("{r}/.toolchains").as_str()),
        "json ENVCTL_LEGACY_TOOLCHAINS"
    );
    assert_eq!(
        v["CARGO_HOME"].as_str(),
        Some(format!("{r}/.toolchains/cargo").as_str()),
        "json CARGO_HOME"
    );
    assert_eq!(
        v["RUSTUP_HOME"].as_str(),
        Some(format!("{r}/.toolchains/rustup").as_str()),
        "json RUSTUP_HOME must be present and meta-located"
    );
    assert_eq!(
        v["OLLAMA_LIBRARY_PATH"].as_str(),
        Some(format!("{r}/.toolchains/ollama/lib/ollama").as_str()),
        "json OLLAMA_LIBRARY_PATH must redirect ollama at the meta-owned GPU runner libs"
    );
    assert_eq!(
        v["LIBCLANG_PATH"].as_str(),
        Some(format!("{r}/.toolchains/llvm/lib").as_str()),
        "json LIBCLANG_PATH must point at the meta-owned LLVM/clang lib dir"
    );
    assert_eq!(
        v["GCC_PATH"].as_str(),
        Some(format!("{r}/.toolchains/libgccjit/lib").as_str()),
        "json GCC_PATH must point at the meta-owned libgccjit lib dir"
    );
    assert_eq!(
        v["HELIX_RUNTIME"].as_str(),
        Some(format!("{r}/.toolchains/helix/runtime").as_str()),
        "json HELIX_RUNTIME must point at the meta-owned helix tree-sitter runtime dir"
    );
}
