//! Concrete `HookRunner` impls. `ProcessRunner` spawns the wrapped bash via
//! `std::process`; `DryRunRunner` returns `DryRun` without executing. Both are
//! `Send + Sync` (no interior mutability).
//!
//! Phase 2: action phases (Install/Fix/Remove) now LINE-STREAM stdout/stderr as
//! `Event::Log` (so the CLI/GUI show progress live during a long apt/nix/CUDA run)
//! AND tee every line to `$META_ROOT/var/lib/envctl/envctl.log` (the analogue of
//! `$META_ROOT/var/lib/envctl/yazelix-setup.log`, survives a crash). Read-only phases (Detect/Verify)
//! capture quietly — only the exit code matters, and leaking their output would
//! corrupt the CLI table / `--json`. Every hook is bounded by a per-phase timeout
//! (the process is killed on expiry) so a stuck installer can't wedge the worker.
use crate::component::{Hook, HookRunner, Phase};
use crate::event::{Event, EventSink, Stream};
use crate::layout::MetaLayout;
use crate::model::{OpResult, OpStatus};
use loop_lib::{build_command as loop_build_command, SpawnSpec};
use std::collections::BTreeMap;
use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, Read, Write};
use std::os::unix::process::CommandExt; // for pre_exec (audit fix #20)
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

#[derive(Default)]
pub struct ProcessRunner;

impl HookRunner for ProcessRunner {
    fn run(
        &self,
        comp: &str,
        phase: Phase,
        hook: &Hook,
        dry_run: bool,
        sink: &EventSink,
    ) -> OpResult {
        if dry_run {
            return mk(comp, phase, OpStatus::DryRun, None, "dry-run");
        }

        // Action phases stream + tee; read-only probes capture quietly.
        let streaming = matches!(phase, Phase::Install | Phase::Fix | Phase::Remove);

        let mut cmd = build_command(hook);
        cmd.stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        // Audit fix (#20): own a process group (setsid) so a per-phase timeout can
        // reap the whole child tree. Without this we only kill the immediate
        // bash/sudo and the real grandchild workload survives. Mirrors addrepo.rs.
        unsafe {
            cmd.pre_exec(|| {
                let _ = rustix::process::setsid();
                Ok(())
            });
        }

        let mut child = match cmd.spawn() {
            Ok(c) => c,
            Err(e) => {
                return mk(
                    comp,
                    phase,
                    OpStatus::Failed,
                    None,
                    &format!("spawn failed: {e}"),
                )
            }
        };
        let pid = child.id();

        let log = Arc::new(Mutex::new(if streaming { open_run_log() } else { None }));
        let tail = Arc::new(Mutex::new(Vec::<String>::new())); // last stderr lines for the message
                                                               // Audit fix (minor #27): also keep a stdout tail so probes that echo their
                                                               // diagnostic to stdout then exit 1 don't yield an empty failure message.
        let out_tail = Arc::new(Mutex::new(Vec::<String>::new()));

        let h_out = child.stdout.take().map(|r| {
            pump(
                r,
                comp.to_string(),
                Stream::Stdout,
                streaming,
                sink.clone(),
                log.clone(),
                Some(out_tail.clone()),
            )
        });
        let h_err = child.stderr.take().map(|r| {
            pump(
                r,
                comp.to_string(),
                Stream::Stderr,
                streaming,
                sink.clone(),
                log.clone(),
                Some(tail.clone()),
            )
        });

        let (code, success, timed_out) = wait_timeout(&mut child, timeout_for(phase), pid);
        if let Some(h) = h_out {
            let _ = h.join();
        }
        if let Some(h) = h_err {
            let _ = h.join();
        }

        if timed_out {
            // Audit fix (minor #24): after SIGKILL the wait code is None, which is
            // indistinguishable from a spawn/internal error. Synthesize the
            // conventional timeout exit code (124) so JSON consumers can tell a
            // timeout from a did-not-run, regardless of what `code` carries.
            let _ = code;
            return mk(
                comp,
                phase,
                OpStatus::Failed,
                Some(124),
                &format!("timed out after {}s", timeout_for(phase).as_secs()),
            );
        }
        if success {
            mk(comp, phase, OpStatus::Ok, code, "")
        } else {
            let mut msg = tail.lock().map(|v| v.join("\n")).unwrap_or_default();
            // Audit fix (minor #27): fall back to the stdout tail when stderr was
            // empty (probe wrote its diagnostic to stdout) so the CLI never shows a
            // failure with no explanation.
            if msg.is_empty() {
                msg = out_tail.lock().map(|v| v.join("\n")).unwrap_or_default();
            }
            mk(comp, phase, OpStatus::Failed, code, truncate(&msg, 4000))
        }
    }
}

fn mk(
    comp: &str,
    phase: Phase,
    status: OpStatus,
    exit_code: Option<i32>,
    message: &str,
) -> OpResult {
    OpResult {
        component: comp.into(),
        phase,
        status,
        exit_code,
        duration_ms: 0,
        message: message.into(),
        dry_run: status == OpStatus::DryRun,
    }
}

fn timeout_for(phase: Phase) -> Duration {
    // Test/debug seam: `ENVCTL_HOOK_TIMEOUT_MS` overrides the per-phase timeout so the
    // timeout/setsid-reaper path is exercisable in tests without a 60s wait. Unset in
    // normal operation (the production per-phase defaults below apply).
    if let Ok(ms) = std::env::var("ENVCTL_HOOK_TIMEOUT_MS") {
        if let Ok(n) = ms.parse::<u64>() {
            return Duration::from_millis(n);
        }
    }
    match phase {
        Phase::Detect | Phase::Verify => Duration::from_secs(60),
        Phase::Install => Duration::from_secs(1800), // big apt/nix/CUDA builds
        Phase::Fix | Phase::Remove => Duration::from_secs(900),
    }
}

/// Reader thread: line-stream a child stream. Emits `Event::Log` + tees to the
/// run log for action phases; always keeps a tail of stderr for the failure msg.
/// Uses lossy UTF-8 so non-UTF-8 build output can't kill the thread.
fn pump<R: Read + Send + 'static>(
    reader: R,
    comp: String,
    stream: Stream,
    streaming: bool,
    sink: EventSink,
    log: Arc<Mutex<Option<File>>>,
    tail: Option<Arc<Mutex<Vec<String>>>>,
) -> JoinHandle<()> {
    std::thread::spawn(move || {
        let mut br = BufReader::new(reader);
        let mut buf = Vec::new();
        loop {
            buf.clear();
            match br.read_until(b'\n', &mut buf) {
                Ok(0) | Err(_) => break,
                Ok(_) => {}
            }
            let line = String::from_utf8_lossy(&buf)
                .trim_end_matches(['\n', '\r'])
                .to_string();
            if streaming {
                sink.emit(Event::Log {
                    component: comp.clone(),
                    stream,
                    line: line.clone(),
                });
                if let Ok(mut g) = log.lock() {
                    if let Some(f) = g.as_mut() {
                        // Audit fix (minor #26): on the first tee write error, emit one
                        // diagnostic and drop the fd so we don't silently retry a dead
                        // fd per line for the rest of a long failing run. UI streaming
                        // above is unaffected.
                        if writeln!(f, "[{comp}] {line}").is_err() {
                            sink.emit(Event::Log {
                                component: comp.clone(),
                                stream: Stream::Stderr,
                                line: "envctl: run-log write failed; stopping log tee".to_string(),
                            });
                            *g = None;
                        }
                    }
                }
            }
            if let Some(t) = &tail {
                if let Ok(mut v) = t.lock() {
                    v.push(line);
                    if v.len() > 40 {
                        v.remove(0);
                    }
                }
            }
        }
    })
}

/// Poll the child to completion or kill it past the deadline.
fn wait_timeout(child: &mut Child, dur: Duration, pid: u32) -> (Option<i32>, bool, bool) {
    let deadline = Instant::now() + dur;
    loop {
        match child.try_wait() {
            Ok(Some(st)) => return (st.code(), st.success(), false),
            Ok(None) => {
                if Instant::now() >= deadline {
                    // Audit fix (#20): kill the whole process group (grandchildren
                    // too), not just the immediate bash/sudo, before reaping.
                    kill_group(pid);
                    let _ = child.kill();
                    let st = child.wait().ok();
                    return (st.and_then(|s| s.code()), false, true);
                }
                std::thread::sleep(Duration::from_millis(100));
            }
            Err(_) => return (None, false, false),
        }
    }
}

/// Kill the child's whole process group (it is the group leader via setsid).
fn kill_group(pid: u32) {
    if let Some(p) = rustix::process::Pid::from_raw(pid as i32) {
        // If the child never became a group leader (for example, if pre_exec
        // failed before setsid() took effect), fall back to killing the child
        // PID directly so a wedged probe cannot linger.
        if rustix::process::kill_process_group(p, rustix::process::Signal::Kill).is_err() {
            let _ = rustix::process::kill_process(p, rustix::process::Signal::Kill);
        }
    }
}

fn open_run_log() -> Option<File> {
    let dir = crate::layout::MetaLayout::from_env_or_default().state();
    std::fs::create_dir_all(&dir).ok()?;
    OpenOptions::new()
        .create(true)
        .append(true)
        .open(dir.join("envctl.log"))
        .ok()
}

fn truncate(s: &str, max: usize) -> &str {
    if s.len() <= max {
        s
    } else {
        // Audit fix (#21): walk forward to the next char boundary so slicing a
        // multibyte UTF-8 tail (real installer failure output) can't panic.
        let mut start = s.len().saturating_sub(max);
        while !s.is_char_boundary(start) {
            start += 1;
        }
        &s[start..]
    }
}

/// Translate a Hook into a ready-to-spawn `Command` (no shell for `Command`;
/// `bash -lc` for `Script`; `bash <path>` for `ShippedScript`). needs_sudo uses
/// `sudo -n` (non-interactive): with a pre-warmed credential it runs silently;
/// without one it fails fast instead of hanging on a TTY-less password prompt.
///
/// envctl owns the wrapping *policy* (which program + args + env each hook shape
/// resolves to); the actual `std::process::Command` *construction* is delegated to
/// the shared meta substrate `loop_lib::build_command` (so meta and envctl assemble
/// subprocess commands the same way). Supervision — setsid, piped stdio, the pump
/// threads, the per-phase timeout — stays in `ProcessRunner::run`, because loop_lib
/// is a batch fan-out runner with no equivalent for those.
fn build_command(hook: &Hook) -> Command {
    let (program, args, hook_env): (String, Vec<String>, Vec<(String, String)>) = match hook {
        Hook::Command {
            command,
            args,
            env,
            needs_sudo,
        } => {
            let (program, mut argv) = sudo_wrap(command.clone(), *needs_sudo);
            argv.extend(args.iter().cloned());
            (program, argv, env_pairs(env))
        }
        Hook::Script {
            script,
            path,
            env,
            needs_sudo,
            login_shell,
        } => {
            let shell_flag = if *login_shell { "-lc" } else { "-c" };
            // The `bash -lc` command string is the inline script, or — when a
            // `path` is given — the path itself (bash executes it).
            let body = match path {
                Some(p) => p.clone(),
                None => script.clone(),
            };
            let (program, mut argv) = if *needs_sudo {
                (
                    "sudo".to_string(),
                    vec!["-n".to_string(), "bash".to_string(), shell_flag.to_string()],
                )
            } else {
                ("bash".to_string(), vec![shell_flag.to_string()])
            };
            argv.push(body);
            (program, argv, env_pairs(env))
        }
        Hook::ShippedScript {
            path,
            args,
            needs_sudo,
        } => {
            let path = MetaLayout::from_env_or_default().expand_meta_path(path);
            let (program, mut argv) = if *needs_sudo {
                (
                    "sudo".to_string(),
                    vec!["-n".to_string(), "bash".to_string(), path],
                )
            } else {
                ("bash".to_string(), vec![path])
            };
            argv.extend(args.iter().cloned());
            (program, argv, Vec::new())
        }
    };
    let env = enforced_meta_env(hook_env);
    loop_build_command(&SpawnSpec {
        clear_env: false,
        program: &program,
        args: &args,
        current_dir: None,
        env: &env,
    })
}

/// Every component hook runs inside the meta-owned install prefix.
///
/// A large amount of legacy shell still spells exposure paths as
/// a user-local prefix; envctl's contract is stricter than that: installs belong
/// under `$META_ROOT`'s FHS/XDG layout (`usr`, `etc`, `var`, `opt`, and meta-XDG
/// roots), never the operator's user-global real user-home local tree. Legacy
/// scripts that use HOME land in `$META_ROOT`, with `$META_ROOT/.local` reserved
/// for XDG compatibility and the real home exposed only as an explicit escape hatch
/// for non-install host integration.
fn enforced_meta_env(mut hook_env: Vec<(String, String)>) -> Vec<(String, String)> {
    let layout = MetaLayout::from_env_or_default();
    let real_home = std::env::var("HOME").unwrap_or_default();
    let mut env = Vec::new();

    // Keep caller-specified values, then append enforced layout values so the
    // meta target wins even if an old manifest tried to override it.
    env.append(&mut hook_env);
    if !real_home.is_empty() {
        env.push(("ENVCTL_REAL_HOME".to_string(), real_home.clone()));
    }
    env.push((
        "META_ROOT".to_string(),
        layout.meta_root().display().to_string(),
    ));
    env.push(("HOME".to_string(), layout.meta_root().display().to_string()));
    for (key, path) in layout.env_exports() {
        env.push((key.to_string(), path.display().to_string()));
    }
    env.push((
        "XDG_CONFIG_HOME".to_string(),
        layout.xdg_config_home().display().to_string(),
    ));
    env.push((
        "XDG_DATA_HOME".to_string(),
        layout.xdg_data_home().display().to_string(),
    ));
    env.push((
        "XDG_STATE_HOME".to_string(),
        layout.xdg_state_home().display().to_string(),
    ));
    env.push((
        "XDG_CACHE_HOME".to_string(),
        layout.xdg_cache_home().display().to_string(),
    ));

    let meta_bin = layout.bin().display().to_string();
    let compat_local_bin = layout.local_bin().display().to_string();
    let legacy_cargo = layout
        .legacy_toolchains()
        .join("cargo/bin")
        .display()
        .to_string();
    let forbidden_real_home_entries = [".local/bin", ".cargo/bin", ".nix-profile/bin"]
        .into_iter()
        .map(|rel| PathBuf::from(&real_home).join(rel).display().to_string())
        .collect::<Vec<_>>();
    let filtered = std::env::var("PATH")
        .unwrap_or_default()
        .split(':')
        .filter(|entry| {
            !entry.is_empty()
                && (real_home.is_empty()
                    || !forbidden_real_home_entries
                        .iter()
                        .any(|forbidden| entry == forbidden))
        })
        .map(str::to_string)
        .collect::<Vec<_>>();
    let mut path = vec![meta_bin, compat_local_bin, legacy_cargo];
    path.extend(filtered);
    env.push(("PATH".to_string(), path.join(":")));
    env
}

/// Resolve `(program, leading-args)` for an optional non-interactive `sudo -n`
/// prefix. Without sudo the program is the command itself and there are no leading
/// args; with sudo the program is `sudo` and the command becomes its first arg.
fn sudo_wrap(command: String, needs_sudo: bool) -> (String, Vec<String>) {
    if needs_sudo {
        ("sudo".to_string(), vec!["-n".to_string(), command])
    } else {
        (command, Vec::new())
    }
}

fn env_pairs(env: &BTreeMap<String, String>) -> Vec<(String, String)> {
    env.iter().map(|(k, v)| (k.clone(), v.clone())).collect()
}

/// Never executes; reports every hook as `DryRun`. Used by tests + previews.
pub struct DryRunRunner;

impl HookRunner for DryRunRunner {
    fn run(&self, comp: &str, phase: Phase, _h: &Hook, _d: bool, _sink: &EventSink) -> OpResult {
        mk(comp, phase, OpStatus::DryRun, None, "dry-run")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn env_value(env: &[(String, String)], key: &str) -> String {
        env.iter()
            .rev()
            .find(|(k, _)| k == key)
            .map(|(_, v)| v.clone())
            .unwrap_or_else(|| panic!("missing env key {key}"))
    }

    #[test]
    fn enforced_meta_env_retargets_home_and_strips_host_global_path_entries() {
        let _g = crate::test_env_lock();
        let prev_home = std::env::var("HOME").ok();
        let prev_meta = std::env::var("META_ROOT").ok();
        let prev_path = std::env::var("PATH").ok();
        std::env::set_var("HOME", "/home/alice");
        std::env::set_var("META_ROOT", "/workspace/meta");
        std::env::set_var(
            "PATH",
            "/home/alice/.local/bin:/home/alice/.cargo/bin:/home/alice/.nix-profile/bin:/usr/bin",
        );

        let env = enforced_meta_env(Vec::new());

        match prev_home {
            Some(v) => std::env::set_var("HOME", v),
            None => std::env::remove_var("HOME"),
        }
        match prev_meta {
            Some(v) => std::env::set_var("META_ROOT", v),
            None => std::env::remove_var("META_ROOT"),
        }
        match prev_path {
            Some(v) => std::env::set_var("PATH", v),
            None => std::env::remove_var("PATH"),
        }

        assert_eq!(env_value(&env, "ENVCTL_REAL_HOME"), "/home/alice");
        assert_eq!(env_value(&env, "HOME"), "/workspace/meta");
        let path = env_value(&env, "PATH");
        let entries = path.split(':').collect::<Vec<_>>();
        assert_eq!(entries[0], "/workspace/meta/usr/bin");
        assert_eq!(entries[1], "/workspace/meta/.local/bin");
        assert_eq!(entries[2], "/workspace/meta/.toolchains/cargo/bin");
        assert_eq!(
            env_value(&env, "XDG_CONFIG_HOME"),
            "/workspace/meta/.config"
        );
        assert_eq!(
            env_value(&env, "XDG_DATA_HOME"),
            "/workspace/meta/.local/share"
        );
        assert_eq!(
            env_value(&env, "XDG_STATE_HOME"),
            "/workspace/meta/.local/state"
        );
        assert_eq!(env_value(&env, "XDG_CACHE_HOME"), "/workspace/meta/.cache");
        assert!(entries.contains(&"/usr/bin"));
        assert!(!entries.contains(&"/home/alice/.local/bin"));
        assert!(!entries.contains(&"/home/alice/.cargo/bin"));
        assert!(!entries.contains(&"/home/alice/.nix-profile/bin"));
    }
}
