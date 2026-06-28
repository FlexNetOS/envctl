//! Integration tests for `envctl env --toolchains`. Drives the real binary
//! against a fixture `.meta.yaml` and asserts the meta-hosted install layout:
//! canonical exposure/state under `$META_ROOT/usr`, `$META_ROOT/var`, and
//! `$META_ROOT/opt`, meta-home XDG roots under `$META_ROOT`, plus legacy manager
//! stores under `.toolchains`. In particular RUSTUP_HOME travels with CARGO_HOME
//! so an `eval "$(envctl env --toolchains)"` shell points rustup at the
//! meta-owned compatibility store, not user-global ~/.rustup.
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

/// Shell form exposes the canonical meta FHS/XDG layout and pairs RUSTUP_HOME
/// with CARGO_HOME under the legacy `.toolchains/` manager store. Without
/// RUSTUP_HOME the eval-seam silently leaks to ~/.rustup.
#[test]
fn toolchains_shell_exports_meta_root_layout_and_rustup_home() {
    let root = fixture_dir();
    let r = root.to_string_lossy();
    let out = run(&root, false);
    assert!(
        out.contains(&format!("export ENVCTL_LOCAL='{r}/.local'")),
        "missing meta-home .local compatibility prefix:\n{out}"
    );
    assert!(
        out.contains(&format!("export ENVCTL_BIN_DIR='{r}/usr/bin'")),
        "missing canonical meta usr/bin export:\n{out}"
    );
    assert!(
        out.contains(&format!("export ENVCTL_LOCAL_BIN='{r}/.local/bin'")),
        "missing compatibility meta .local/bin export:\n{out}"
    );
    assert!(
        out.contains(&format!(
            "export ENVCTL_REPO_STORE='{r}/var/lib/envctl/repos'"
        )),
        "missing canonical envctl repo store export:\n{out}"
    );
    assert!(
        out.contains(&format!("export ENVCTL_XDG_CONFIG_HOME='{r}/.config'")),
        "missing meta XDG config export:\n{out}"
    );
    assert!(
        out.contains(&format!("export ENVCTL_XDG_DATA_HOME='{r}/.local/share'")),
        "missing meta XDG data export:\n{out}"
    );
    assert!(
        out.contains(&format!("export ENVCTL_XDG_STATE_HOME='{r}/.local/state'")),
        "missing meta XDG state export:\n{out}"
    );
    assert!(
        out.contains(&format!("export ENVCTL_XDG_CACHE_HOME='{r}/.cache'")),
        "missing meta XDG cache export:\n{out}"
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
        out.contains(&format!("export XDG_CACHE_HOME='{r}/.cache'")),
        "XDG_CACHE_HOME must make kache and other cache-heavy toolchains meta-owned:\n{out}"
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
            "export PATH=\"{r}/usr/bin:{r}/usr/sbin:{r}/usr/local/bin:{r}/usr/local/sbin:{r}/.local/bin:{r}/.toolchains/.bun/bin:{r}/.toolchains/cargo/bin:{r}/.toolchains/uv/tools/bin:$PATH\""
        )),
        "PATH must put the canonical meta usr bin tree ahead of compatibility bins:\n{out}"
    );
    // The rest of the /usr mirror lands on its search paths, prepend-with-fallback.
    assert!(
        out.contains(&format!(
            "export LD_LIBRARY_PATH=\"{r}/usr/lib:{r}/usr/lib64:{r}/usr/local/lib:{r}/usr/local/lib64:${{LD_LIBRARY_PATH:-}}\""
        )),
        "LD_LIBRARY_PATH must carry the meta usr lib tree without clobbering inherited values:\n{out}"
    );
    assert!(
        out.contains(&format!(
            "export CPATH=\"{r}/usr/include:{r}/usr/local/include:${{CPATH:-}}\""
        )),
        "CPATH must carry the meta usr include tree:\n{out}"
    );
    assert!(
        out.contains(&format!(
            "export PKG_CONFIG_PATH=\"{r}/usr/lib/pkgconfig:{r}/usr/share/pkgconfig:${{PKG_CONFIG_PATH:-}}\""
        )),
        "PKG_CONFIG_PATH must carry the meta usr pkgconfig dirs:\n{out}"
    );
    assert!(
        out.contains(&format!(
            "export MANPATH=\"{r}/usr/share/man:{r}/usr/local/share/man${{MANPATH:+:$MANPATH}}\""
        )),
        "MANPATH must carry the meta usr man dirs:\n{out}"
    );
    // The new FHS-mirror layout exports are present.
    for (key, sub) in [
        ("ENVCTL_USR_SBIN", "usr/sbin"),
        ("ENVCTL_USR_LIB64", "usr/lib64"),
        ("ENVCTL_USR_INCLUDE", "usr/include"),
        ("ENVCTL_USR_LOCAL", "usr/local"),
        ("ENVCTL_USR_LOCAL_BIN", "usr/local/bin"),
        ("ENVCTL_USR_LOCAL_LIB", "usr/local/lib"),
    ] {
        assert!(
            out.contains(&format!("export {key}='{r}/{sub}'")),
            "missing {key} usr-mirror export:\n{out}"
        );
    }
}

/// JSON form carries the layout variables and RUSTUP_HOME too, so machine
/// consumers see the same seam.
#[test]
fn toolchains_json_carries_meta_root_layout_and_rustup_home() {
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
        Some(format!("{r}/usr/bin").as_str()),
        "json ENVCTL_BIN_DIR"
    );
    assert_eq!(
        v["ENVCTL_LOCAL_BIN"].as_str(),
        Some(format!("{r}/.local/bin").as_str()),
        "json ENVCTL_LOCAL_BIN"
    );
    assert_eq!(
        v["ENVCTL_REPO_STORE"].as_str(),
        Some(format!("{r}/var/lib/envctl/repos").as_str()),
        "json ENVCTL_REPO_STORE"
    );
    assert_eq!(
        v["ENVCTL_XDG_CONFIG_HOME"].as_str(),
        Some(format!("{r}/.config").as_str()),
        "json ENVCTL_XDG_CONFIG_HOME"
    );
    assert_eq!(
        v["ENVCTL_XDG_DATA_HOME"].as_str(),
        Some(format!("{r}/.local/share").as_str()),
        "json ENVCTL_XDG_DATA_HOME"
    );
    assert_eq!(
        v["ENVCTL_XDG_STATE_HOME"].as_str(),
        Some(format!("{r}/.local/state").as_str()),
        "json ENVCTL_XDG_STATE_HOME"
    );
    assert_eq!(
        v["ENVCTL_XDG_CACHE_HOME"].as_str(),
        Some(format!("{r}/.cache").as_str()),
        "json ENVCTL_XDG_CACHE_HOME"
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
        v["XDG_CACHE_HOME"].as_str(),
        Some(format!("{r}/.cache").as_str()),
        "json XDG_CACHE_HOME must be present and meta-located for kache"
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
    // The FHS /usr mirror dirs are carried as discrete JSON vars too, so nushell
    // (and other machine consumers) can build their own search paths from them.
    for (key, sub) in [
        ("ENVCTL_USR_SBIN", "usr/sbin"),
        ("ENVCTL_USR_LIB64", "usr/lib64"),
        ("ENVCTL_USR_INCLUDE", "usr/include"),
        ("ENVCTL_USR_LOCAL_BIN", "usr/local/bin"),
        ("ENVCTL_USR_LOCAL_LIB", "usr/local/lib"),
    ] {
        assert_eq!(
            v[key].as_str(),
            Some(format!("{r}/{sub}").as_str()),
            "json {key} usr-mirror export"
        );
    }
}
