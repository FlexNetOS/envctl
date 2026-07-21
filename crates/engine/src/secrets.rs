//! `secrets`: the engine-owned subprocess seam that drives the installed `secretctl`
//! binary on behalf of the GUI (TASK-0028, Architecture B). The engine is the single
//! sync, non-printing authority — the GUI builds an argv `Vec<String>` (the IDENTICAL
//! `secretctl` clap surface the CLI drives), and the engine spawns the subprocess,
//! pipes the (optional) secret stdin buffer, captures stdout/stderr/exit, and emits a
//! single `Event::SecretsResult`. It parses NOTHING secret and holds no token after the
//! child exits.
//!
//! Why a subprocess and not an embedded gRPC client: the secrets verbs require a tonic
//! `VaultClient` over the daemon UDS in its own async runtime. Grafting that into the
//! pure-sync egui app would add tokio + tonic + prost + secrets-proto and force a fresh
//! `ci/gates/no-c.sh` re-proof. Driving the identical clap surface makes CLI↔GUI
//! divergence structurally impossible and adds ZERO crate deps.
use crate::{Event, EventSink};
use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use zeroize::Zeroizing;

/// Resolve the `secretctl` binary through the sole installed-runtime owner.
/// The returned path always remains profile-addressed; store provenance is a
/// profile-health concern and must never replace the profile frontdoor here.
fn resolve_secretctl() -> Option<PathBuf> {
    let home = std::env::var_os("ENVCTL_REAL_HOME")
        .filter(|value| !value.is_empty())
        .or_else(|| std::env::var_os("HOME").filter(|value| !value.is_empty()))?;
    let profile = PathBuf::from(home).join(".nix-profile");
    [
        profile.join("toolbin/secretctl"),
        profile.join("bin/secretctl"),
    ]
    .into_iter()
    .find(|path| path.is_file())
}

/// Spawn `secretctl <argv...>`, optionally piping `stdin` (a `Zeroizing` secret buffer —
/// e.g. the revoke token for `--token -`) to the child's stdin, capture stdout/stderr/exit,
/// and emit a single `Event::SecretsResult`. `verb` is a stable label (`"mint-github"` /
/// `"relay-mint"` / `"revoke"`) the GUI keys its result rendering off of.
///
/// Fail-closed: if `secretctl` is not resolvable, emits a `SecretsResult` carrying a
/// "secretctl not installed" error and a non-zero `code` — it never panics and never
/// synthesizes success. The `Zeroizing` stdin buffer is dropped (zeroized) when this
/// function returns; nothing secret is parsed or retained.
pub fn run_secretctl(
    verb: String,
    argv: Vec<String>,
    stdin: Option<Zeroizing<Vec<u8>>>,
    sink: &EventSink,
) {
    let Some(bin) = resolve_secretctl() else {
        sink.emit(Event::SecretsResult {
            verb,
            json_stdout: String::new(),
            stderr: "secretctl is not installed through the active Nix profile \
                     (checked profile toolbin and bin frontdoors)"
                .to_string(),
            code: None,
        });
        return;
    };

    let mut cmd = Command::new(&bin);
    cmd.args(&argv)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    // Only open a stdin pipe when we have a buffer to write; otherwise inherit-null so the
    // child never blocks reading a stdin that will never arrive.
    if stdin.is_some() {
        cmd.stdin(Stdio::piped());
    } else {
        cmd.stdin(Stdio::null());
    }

    let mut child = match cmd.spawn() {
        Ok(c) => c,
        Err(e) => {
            sink.emit(Event::SecretsResult {
                verb,
                json_stdout: String::new(),
                stderr: format!("failed to spawn secretctl: {e}"),
                code: None,
            });
            return;
        }
    };

    // Write the secret stdin buffer to the child, then drop our handle so the child sees EOF.
    // The `Zeroizing` buffer is moved in and zeroized on drop at the end of this block.
    if let Some(buf) = stdin {
        if let Some(mut sink_in) = child.stdin.take() {
            let _ = sink_in.write_all(&buf);
            // explicit drop closes the pipe (EOF) before we wait, and drops `buf` (zeroized).
            drop(sink_in);
        }
    }

    match child.wait_with_output() {
        Ok(out) => {
            sink.emit(Event::SecretsResult {
                verb,
                json_stdout: String::from_utf8_lossy(&out.stdout).into_owned(),
                stderr: String::from_utf8_lossy(&out.stderr).into_owned(),
                code: out.status.code(),
            });
        }
        Err(e) => {
            sink.emit(Event::SecretsResult {
                verb,
                json_stdout: String::new(),
                stderr: format!("secretctl wait failed: {e}"),
                code: None,
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::os::unix::fs::PermissionsExt;
    use std::path::Path;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_root(name: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!(
            "envctl-secrets-{name}-{}-{nanos}",
            std::process::id()
        ))
    }

    fn write_executable(path: &Path) {
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, b"#!/bin/sh\n").unwrap();
        let mut perms = std::fs::metadata(path).unwrap().permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(path, perms).unwrap();
    }

    #[test]
    fn resolve_secretctl_rejects_host_global_home_and_path_entries() {
        let _g = crate::test_env_lock();
        let root = temp_root("reject-host-global");
        let home = root.join("home");
        let meta = root.join("meta");
        let path_dir = root.join("foreign-bin");
        write_executable(&home.join("bin/secretctl"));
        write_executable(&home.join(".cargo/bin/secretctl"));
        write_executable(&path_dir.join("secretctl"));
        let prev_path = std::env::var("PATH").ok();
        let prev_home = std::env::var("HOME").ok();
        let prev_meta = std::env::var("META_ROOT").ok();
        std::env::set_var("PATH", &path_dir);
        std::env::set_var("HOME", &home);
        std::env::set_var("META_ROOT", &meta);

        let resolved = resolve_secretctl();

        match prev_path {
            Some(p) => std::env::set_var("PATH", p),
            None => std::env::remove_var("PATH"),
        }
        match prev_home {
            Some(h) => std::env::set_var("HOME", h),
            None => std::env::remove_var("HOME"),
        }
        match prev_meta {
            Some(m) => std::env::set_var("META_ROOT", m),
            None => std::env::remove_var("META_ROOT"),
        }
        let _ = std::fs::remove_dir_all(root);

        assert!(
            resolved.is_none(),
            "host-global secretctl must not resolve: {resolved:?}"
        );
    }

    #[test]
    fn resolve_secretctl_rejects_workspace_usr_bin_target() {
        let _g = crate::test_env_lock();
        let root = temp_root("accept-meta-usr-bin");
        let home = root.join("home");
        let meta = root.join("meta");
        let secretctl = meta.join("usr/bin/secretctl");
        write_executable(&secretctl);
        let prev_path = std::env::var("PATH").ok();
        let prev_home = std::env::var("HOME").ok();
        let prev_meta = std::env::var("META_ROOT").ok();
        std::env::set_var("PATH", "");
        std::env::set_var("HOME", &home);
        std::env::set_var("META_ROOT", &meta);

        let resolved = resolve_secretctl();

        match prev_path {
            Some(p) => std::env::set_var("PATH", p),
            None => std::env::remove_var("PATH"),
        }
        match prev_home {
            Some(h) => std::env::set_var("HOME", h),
            None => std::env::remove_var("HOME"),
        }
        match prev_meta {
            Some(m) => std::env::set_var("META_ROOT", m),
            None => std::env::remove_var("META_ROOT"),
        }
        let _ = std::fs::remove_dir_all(root);

        assert!(
            resolved.is_none(),
            "workspace binary must not own the frontdoor"
        );
    }

    #[test]
    fn missing_binary_emits_failclosed_result_not_panic() {
        // Force resolution to miss: empty PATH and a HOME with no profile secretctl.
        let _g = crate::test_env_lock();
        let prev_path = std::env::var("PATH").ok();
        let prev_home = std::env::var("HOME").ok();
        let prev_meta = std::env::var("META_ROOT").ok();
        let tmp = std::env::temp_dir().join("envctl-secrets-test-nohome");
        std::env::set_var("PATH", "");
        std::env::set_var("HOME", &tmp);
        std::env::set_var("META_ROOT", tmp.join("meta"));

        let (sink, rx) = EventSink::channel();
        run_secretctl(
            "revoke".to_string(),
            vec!["github-app".into(), "revoke-token".into()],
            None,
            &sink,
        );
        // restore env before assertions so a failure doesn't leak the mutation
        match prev_path {
            Some(p) => std::env::set_var("PATH", p),
            None => std::env::remove_var("PATH"),
        }
        match prev_home {
            Some(h) => std::env::set_var("HOME", h),
            None => std::env::remove_var("HOME"),
        }
        match prev_meta {
            Some(m) => std::env::set_var("META_ROOT", m),
            None => std::env::remove_var("META_ROOT"),
        }

        let ev = rx.recv().expect("a SecretsResult was emitted");
        match ev {
            Event::SecretsResult {
                verb,
                code,
                stderr,
                json_stdout,
            } => {
                assert_eq!(verb, "revoke");
                assert!(code.is_none(), "unresolved binary ⇒ no exit code");
                assert!(json_stdout.is_empty(), "no stdout on the not-found path");
                assert!(
                    stderr.contains("not installed"),
                    "fail-closed message, got: {stderr}"
                );
            }
            other => panic!("expected SecretsResult, got {other:?}"),
        }
    }
}
