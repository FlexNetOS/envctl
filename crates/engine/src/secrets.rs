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

/// Resolve the `secretctl` binary, fail-closed (returns `None` if it cannot be found):
/// (a) alongside the current executable (the GUI ships next to it in `~/.cargo/bin`),
/// (b) `$HOME/.cargo/bin/secretctl` (the manifest install location, `env-ctl.toml:66`),
/// (c) on `PATH`. The first existing path wins.
fn resolve_secretctl() -> Option<PathBuf> {
    // (a) alongside current_exe
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            let cand = dir.join("secretctl");
            if cand.is_file() {
                return Some(cand);
            }
        }
    }
    // (b) $HOME/.cargo/bin/secretctl
    if let Ok(home) = std::env::var("HOME") {
        let cand = PathBuf::from(home).join(".cargo/bin/secretctl");
        if cand.is_file() {
            return Some(cand);
        }
    }
    // (c) PATH
    which::which("secretctl").ok()
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
            stderr: "secretctl not installed (looked alongside the binary, in \
                     $HOME/.cargo/bin, and on PATH)"
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

    #[test]
    fn missing_binary_emits_failclosed_result_not_panic() {
        // Force resolution to miss: empty PATH + a HOME with no .cargo/bin/secretctl, and
        // current_exe's dir won't have a `secretctl` in the test harness.
        let _g = crate::test_env_lock();
        let prev_path = std::env::var("PATH").ok();
        let prev_home = std::env::var("HOME").ok();
        let tmp = std::env::temp_dir().join("envctl-secrets-test-nohome");
        std::env::set_var("PATH", "");
        std::env::set_var("HOME", &tmp);

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
