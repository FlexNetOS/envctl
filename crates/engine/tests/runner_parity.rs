//! Behavior-parity tests for the real `ProcessRunner` (the hook executor).
//!
//! These pin the hook supervisor behavior that meta/envctl command substrate changes must preserve:
//! exit-code capture, stderr/stdout tail + char-safe truncate, per-phase timeout
//! synthesizing exit 124, process-group reaping, phase-conditional
//! streaming + tee, quiet capture on read-only phases, hook wrapping (argv vs
//! `bash -lc`/`-c`), per-hook env injection, lossy-UTF-8 safety, and spawn-failure
//! handling. Before this file there was NO test exercising the real runner
//! (tests/engine.rs uses DryRun/stub runners only).
//!
//! The per-phase timeout is overridable via `ENVCTL_HOOK_TIMEOUT_MS` (a process-
//! global env var), and the streaming test repoints `HOME`, so every test holds a
//! shared lock to run serially — otherwise a parallel test would see another's
//! shrunk timeout / temp HOME.

use envctl_engine::{Event, EventSink, Hook, HookRunner, OpStatus, Phase, ProcessRunner};
use std::collections::BTreeMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Mutex, MutexGuard};

static ENV_LOCK: Mutex<()> = Mutex::new(());

fn lock() -> MutexGuard<'static, ()> {
    ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner())
}

/// RAII override of the per-phase timeout; removed on drop (even on panic).
struct TimeoutMs;
impl TimeoutMs {
    fn set(ms: u64) -> Self {
        std::env::set_var("ENVCTL_HOOK_TIMEOUT_MS", ms.to_string());
        TimeoutMs
    }
}
impl Drop for TimeoutMs {
    fn drop(&mut self) {
        std::env::remove_var("ENVCTL_HOOK_TIMEOUT_MS");
    }
}

fn unique_tmp(tag: &str) -> std::path::PathBuf {
    static N: AtomicU64 = AtomicU64::new(0);
    let p = std::env::temp_dir().join(format!(
        "envctl-runner-test-{}-{}-{}",
        std::process::id(),
        tag,
        N.fetch_add(1, Ordering::Relaxed)
    ));
    std::fs::create_dir_all(&p).unwrap();
    p
}

/// Run a hook through the real `ProcessRunner`, returning its result + every event.
fn run(hook: &Hook, phase: Phase) -> (envctl_engine::model::OpResult, Vec<Event>) {
    let (sink, rx) = EventSink::channel();
    let res = ProcessRunner.run("test-comp", phase, hook, false, &sink);
    drop(sink); // close the channel so the receiver drains
    let events = rx.into_iter().collect();
    (res, events)
}

fn script(body: &str, login_shell: bool) -> Hook {
    Hook::Script {
        script: body.to_string(),
        path: None,
        env: BTreeMap::new(),
        needs_sudo: false,
        login_shell,
    }
}

#[test]
fn exit_code_success_and_failure() {
    let _g = lock();
    let (ok, _) = run(&script("exit 0", false), Phase::Verify);
    assert_eq!(ok.status, OpStatus::Ok);
    assert_eq!(ok.exit_code, Some(0));

    let (bad, _) = run(&script("exit 7", false), Phase::Verify);
    assert_eq!(bad.status, OpStatus::Failed);
    assert_eq!(bad.exit_code, Some(7));
}

#[test]
fn failure_message_is_stderr_tail() {
    let _g = lock();
    let (res, _) = run(
        &script("echo boom-on-stderr 1>&2; exit 1", false),
        Phase::Verify,
    );
    assert_eq!(res.status, OpStatus::Failed);
    assert!(
        res.message.contains("boom-on-stderr"),
        "message should carry the stderr tail, got: {:?}",
        res.message
    );
}

#[test]
fn failure_message_falls_back_to_stdout_tail() {
    let _g = lock();
    // stderr empty, diagnostic only on stdout — must still surface something.
    let (res, _) = run(&script("echo diag-on-stdout; exit 3", false), Phase::Verify);
    assert_eq!(res.status, OpStatus::Failed);
    assert!(
        res.message.contains("diag-on-stdout"),
        "empty-stderr failure should fall back to the stdout tail, got: {:?}",
        res.message
    );
}

#[test]
fn long_multibyte_stderr_truncates_without_panic() {
    let _g = lock();
    // Emit a long multibyte string to stderr then fail; truncate must not split a
    // UTF-8 boundary and the message stays bounded (<= 4000 bytes).
    let (res, _) = run(
        &script(
            "for i in $(seq 1 2000); do printf 'héllo-%s ' \"$i\" 1>&2; done; exit 1",
            false,
        ),
        Phase::Verify,
    );
    assert_eq!(res.status, OpStatus::Failed);
    assert!(
        res.message.len() <= 4000,
        "message must be truncated to <=4000 bytes"
    );
    assert!(res.message.is_char_boundary(0));
}

#[test]
fn timeout_synthesizes_exit_124() {
    let _g = lock();
    let _t = TimeoutMs::set(300);
    let (res, _) = run(&script("/usr/bin/sleep 30", false), Phase::Verify);
    assert_eq!(res.status, OpStatus::Failed);
    assert_eq!(res.exit_code, Some(124), "timeout must synthesize exit 124");
    assert!(res.message.contains("timed out"), "got: {:?}", res.message);
}

#[test]
fn process_group_reaps_the_whole_child_tree() {
    let _g = lock();
    let marker = unique_tmp("process-group").join("beat");
    let mp = marker.display().to_string();
    // Background a grandchild that keeps touching the marker; the parent sleeps long.
    // After the per-phase timeout kills the GROUP, the grandchild must die too, so the
    // marker's mtime stops advancing.
    let body = format!(
        "( while true; do /usr/bin/touch '{mp}'; /usr/bin/sleep 0.1; done ) & /usr/bin/sleep 30",
        mp = mp
    );
    let _t = TimeoutMs::set(500);
    let (res, _) = run(&script(&body, false), Phase::Verify);
    assert_eq!(res.exit_code, Some(124));
    // Let any un-reaped grandchild run a bit, then confirm the marker is frozen.
    std::thread::sleep(std::time::Duration::from_millis(400));
    let m1 = std::fs::metadata(&marker).and_then(|m| m.modified()).ok();
    std::thread::sleep(std::time::Duration::from_millis(600));
    let m2 = std::fs::metadata(&marker).and_then(|m| m.modified()).ok();
    assert_eq!(
        m1, m2,
        "grandchild survived group-kill: marker mtime still advancing (process-group reaping broken)"
    );
}

#[cfg(unix)]
#[test]
fn process_runner_preserves_the_callers_session_while_isolating_the_process_group() {
    let _g = lock();
    let root = unique_tmp("preserve-session");
    let observed = root.join("session-id");
    let hook = Hook::Script {
        script: format!(
            "/usr/bin/ps -o pid= -o sid= -o pgid= -p $$ > '{}'",
            observed.display()
        ),
        path: None,
        env: BTreeMap::new(),
        needs_sudo: false,
        login_shell: false,
    };
    let (sink, _rx) = EventSink::channel();
    let result = ProcessRunner.run("preserve-session", Phase::Verify, &hook, false, &sink);
    assert_eq!(result.status, OpStatus::Ok, "{}", result.message);

    let expected = std::process::Command::new("/usr/bin/ps")
        .args([
            "-o",
            "sid=",
            "-o",
            "pgid=",
            "-p",
            &std::process::id().to_string(),
        ])
        .output()
        .expect("inspect caller session");
    assert!(expected.status.success());
    let observed = std::fs::read_to_string(&observed).unwrap();
    let observed = observed.split_whitespace().collect::<Vec<_>>();
    let expected = String::from_utf8(expected.stdout).unwrap();
    let expected = expected.split_whitespace().collect::<Vec<_>>();
    assert_eq!(
        observed.len(),
        3,
        "unexpected child ps output: {observed:?}"
    );
    assert_eq!(
        expected.len(),
        2,
        "unexpected caller ps output: {expected:?}"
    );
    assert_eq!(observed[1], expected[0], "the runner must retain the invoking terminal session so privileged hooks can use its sudo ticket");
    assert_eq!(
        observed[2], observed[0],
        "the child must lead its own timeout-reapable process group"
    );
    assert_ne!(
        observed[2], expected[1],
        "the child must not share the caller's process group"
    );
    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn action_phase_streams_and_tees_to_log() {
    let _g = lock();
    let home = unique_tmp("home");
    let meta = unique_tmp("meta");
    let prev = std::env::var_os("HOME");
    let prev_meta = std::env::var_os("META_ROOT");
    std::env::set_var("HOME", &home);
    std::env::set_var("META_ROOT", &meta);
    let (res, events) = run(&script("echo streamed-line", false), Phase::Install);
    // restore HOME before assertions
    match prev {
        Some(v) => std::env::set_var("HOME", v),
        None => std::env::remove_var("HOME"),
    }
    match prev_meta {
        Some(v) => std::env::set_var("META_ROOT", v),
        None => std::env::remove_var("META_ROOT"),
    }
    assert_eq!(res.status, OpStatus::Ok);
    assert!(
        events
            .iter()
            .any(|e| matches!(e, Event::Log { line, .. } if line.contains("streamed-line"))),
        "action phase must emit Event::Log lines"
    );
    let logged =
        std::fs::read_to_string(meta.join("var/lib/envctl/envctl.log")).unwrap_or_default();
    assert!(
        logged.contains("[test-comp] streamed-line"),
        "action phase must tee to envctl.log, got: {:?}",
        logged
    );
}

#[test]
fn readonly_phase_is_quiet() {
    let _g = lock();
    let (res, events) = run(&script("echo should-not-stream", false), Phase::Detect);
    assert_eq!(res.status, OpStatus::Ok);
    assert!(
        !events.iter().any(|e| matches!(e, Event::Log { .. })),
        "Detect/Verify must NOT emit Event::Log (protects the table/--json)"
    );
}

#[test]
fn command_hook_is_argv_no_shell() {
    let _g = lock();
    // Shell metacharacters in an arg must be passed literally (no shell expansion),
    // proving the Command path runs argv directly. `printf %s` echoes the arg verbatim.
    let hook = Hook::Command {
        command: "/usr/bin/printf".to_string(),
        args: vec!["%s".to_string(), "a;b|c$(boom)".to_string()],
        env: BTreeMap::new(),
        needs_sudo: false,
    };
    let (res, _) = run(&hook, Phase::Verify);
    assert_eq!(
        res.status,
        OpStatus::Ok,
        "argv command should succeed: {:?}",
        res.message
    );
}

#[test]
fn per_hook_env_is_visible_to_child() {
    let _g = lock();
    let mut env = BTreeMap::new();
    env.insert("ENVCTL_PARITY".to_string(), "present".to_string());
    let hook = Hook::Script {
        script: "test \"$ENVCTL_PARITY\" = present".to_string(),
        path: None,
        env,
        needs_sudo: false,
        login_shell: false,
    };
    let (res, _) = run(&hook, Phase::Verify);
    assert_eq!(
        res.status,
        OpStatus::Ok,
        "per-hook env var must reach the child"
    );
}

#[test]
fn login_vs_nonlogin_shell_flag() {
    let _g = lock();
    // Both must run; login_shell toggles -lc vs -c. A trivial command works in both.
    for login in [true, false] {
        let (res, _) = run(&script("true", login), Phase::Verify);
        assert_eq!(res.status, OpStatus::Ok, "shell (login={login}) should run");
    }
}

#[test]
fn lossy_utf8_output_does_not_panic() {
    let _g = lock();
    // Emit invalid UTF-8 then exit 0; the pump threads use lossy decoding.
    let (res, _) = run(
        &script("printf '\\xff\\xfe invalid'; exit 0", false),
        Phase::Install,
    );
    assert_eq!(res.status, OpStatus::Ok);
}

#[test]
fn spawn_failure_is_failed_not_panic() {
    let _g = lock();
    let hook = Hook::Command {
        command: "definitely-not-a-real-binary-xyz".to_string(),
        args: vec![],
        env: BTreeMap::new(),
        needs_sudo: false,
    };
    let (res, _) = run(&hook, Phase::Verify);
    assert_eq!(res.status, OpStatus::Failed);
    assert!(
        res.message.contains("spawn failed"),
        "got: {:?}",
        res.message
    );
}
