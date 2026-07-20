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

        let mut cmd = match build_command(hook) {
            Ok(command) => command,
            Err(error) => {
                return mk(
                    comp,
                    phase,
                    OpStatus::Failed,
                    None,
                    &format!("command construction failed: {error}"),
                )
            }
        };
        cmd.stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        // Own a process group so a per-phase timeout can reap the whole child
        // tree. `setpgid` deliberately retains the invoking terminal session:
        // detached `setsid` children cannot use a sudo ticket tied to that TTY,
        // even after Envctl pre-warms it. A distinct process group preserves the
        // same timeout reaping guarantee without severing that authorization path.
        unsafe {
            cmd.pre_exec(|| {
                rustix::process::setpgid(None, None).map_err(std::io::Error::from)?;
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
        availability: None,
        exit_code,
        duration_ms: 0,
        message: message.into(),
        dry_run: status == OpStatus::DryRun,
    }
}

fn timeout_for(phase: Phase) -> Duration {
    // Test/debug seam: `ENVCTL_HOOK_TIMEOUT_MS` overrides the per-phase timeout so the
    // timeout/process-group reaper path is exercisable in tests without a 60s wait. Unset in
    // normal operation (the production per-phase defaults below apply).
    if let Ok(ms) = std::env::var("ENVCTL_HOOK_TIMEOUT_MS") {
        if let Ok(n) = ms.parse::<u64>() {
            return Duration::from_millis(n);
        }
    }
    match phase {
        // Read-only probes include complete-generation integrity checks (for example LLVM's
        // path/mode/symlink/content digest over an 11 GiB tree). A cold page cache can make that
        // legitimate proof exceed one minute; keep it bounded, but do not misclassify a healthy
        // generation and needlessly enter its mutation path.
        Phase::Detect | Phase::Verify => Duration::from_secs(300),
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

/// Kill the child's whole process group (it is the group leader via setpgid).
fn kill_group(pid: u32) {
    if let Some(p) = rustix::process::Pid::from_raw(pid as i32) {
        // If the child never became a group leader (for example, if pre_exec
        // failed before setpgid() took effect), fall back to killing the child
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
/// `bash --noprofile --norc -lc` for login `Script` hooks (or `-c` for non-login hooks);
/// `bash <path>` for `ShippedScript`). needs_sudo uses
/// `sudo -n` (non-interactive): with a pre-warmed credential it runs silently;
/// without one it fails fast instead of hanging on a TTY-less password prompt.
///
/// envctl owns the wrapping *policy* (which program + args + env each hook shape
/// resolves to); the actual `std::process::Command` *construction* is delegated to
/// the shared meta substrate `loop_lib::build_command` (so meta and envctl assemble
/// subprocess commands the same way). Supervision — process groups, piped stdio, the pump
/// threads, the per-phase timeout — stays in `ProcessRunner::run`, because loop_lib
/// is a batch fan-out runner with no equivalent for those.
fn build_command(hook: &Hook) -> Result<Command, String> {
    let (program, args, hook_env, needs_sudo): (String, Vec<String>, Vec<(String, String)>, bool) =
        match hook {
            Hook::Command {
                command,
                args,
                env,
                needs_sudo,
            } => {
                let command = trusted_hook_entry(command);
                let mut args = args.clone();
                if command == "/usr/bin/bash" {
                    args.splice(0..0, ["--noprofile".to_string(), "--norc".to_string()]);
                }
                (command, args, env_pairs(env), *needs_sudo)
            }
            Hook::Script {
                script,
                path,
                env,
                needs_sudo,
                login_shell,
            } => {
                let shell_flag = if *login_shell { "-lc" } else { "-c" };
                // The bash command string is the inline script, or — when a
                // `path` is given — the path itself (bash executes it).
                let body = match path {
                    Some(p) => p.clone(),
                    None => script.clone(),
                };
                let program = "/usr/bin/bash".to_string();
                let mut argv = vec![
                    "--noprofile".to_string(),
                    "--norc".to_string(),
                    shell_flag.to_string(),
                ];
                argv.push(body);
                (program, argv, env_pairs(env), *needs_sudo)
            }
            Hook::ShippedScript {
                path,
                args,
                needs_sudo,
            } => {
                let path = resolve_shipped_script_path(path)?;
                let program = "/usr/bin/bash".to_string();
                let mut argv = vec![path];
                argv.extend(args.iter().cloned());
                (program, argv, Vec::new(), *needs_sudo)
            }
        };
    let mut env = enforced_meta_env(hook_env);
    if needs_sudo {
        enforce_privileged_path(&mut env);
    }
    let (program, args) = sudo_wrap(program, args, needs_sudo, &env)?;
    Ok(loop_build_command(&SpawnSpec {
        program: &program,
        args: &args,
        current_dir: None,
        env: &env,
        clear_env: true,
    }))
}

fn resolve_shipped_script_path(declared: &str) -> Result<String, String> {
    let layout = MetaLayout::from_env_or_default();
    let resolved = if let Some(relative) = declared.strip_prefix("$ENVCTL_SOURCE_ROOT/") {
        let root = std::env::var_os("ENVCTL_SOURCE_ROOT")
            .map(std::path::PathBuf::from)
            .unwrap_or_else(|| layout.meta_root().join("src/envctl"));
        if !root.is_absolute() {
            return Err("ENVCTL_SOURCE_ROOT must be absolute for a shipped script".to_string());
        }
        root.join(relative)
    } else {
        std::path::PathBuf::from(layout.expand_meta_path(declared))
    };
    if !resolved.is_absolute() || !envctl_agent_env::managed_path_authority_is_safe(&resolved) {
        return Err(format!(
            "shipped script has an unsafe authority path: {}",
            resolved.display()
        ));
    }
    let metadata = std::fs::symlink_metadata(&resolved).map_err(|error| {
        format!(
            "cannot inspect shipped script {}: {error}",
            resolved.display()
        )
    })?;
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        return Err(format!(
            "shipped script must be a real regular file: {}",
            resolved.display()
        ));
    }
    let canonical = std::fs::canonicalize(&resolved).map_err(|error| {
        format!(
            "cannot canonicalize shipped script {}: {error}",
            resolved.display()
        )
    })?;
    if canonical != resolved {
        return Err(format!(
            "shipped script path is not canonical: {}",
            resolved.display()
        ));
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        if metadata.uid() != rustix::process::geteuid().as_raw() {
            return Err(format!(
                "shipped script is not current-user-owned: {}",
                resolved.display()
            ));
        }
    }
    Ok(resolved.to_string_lossy().into_owned())
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
    let mut env = sanitized_inherited_env();

    // A number of clients prefer lowercase proxy aliases even when an uppercase key is also set.
    // If a trusted hook declares either spelling, remove both inherited spellings first so a
    // caller alias cannot silently outrank the hook's selected route.
    suppress_shadowed_proxy_aliases(&mut env, &hook_env);
    // Keep the trusted manifest's hook-specific non-layout values, then append enforced layout
    // values so the meta target wins even if an old manifest tried to override it. The command
    // itself receives a cleared environment; `sanitized_inherited_env` is the only compatibility
    // bridge for caller state and excludes shell/dynamic-loader execution controls.
    env.append(&mut hook_env);
    if !real_home.is_empty() {
        env.push(("ENVCTL_REAL_HOME".to_string(), real_home.clone()));
    }
    env.push((
        "META_ROOT".to_string(),
        layout.meta_root().display().to_string(),
    ));
    env.push(("HOME".to_string(), layout.meta_root().display().to_string()));
    // Rustup proxies derive both their payload lookup and Cargo state from these homes. They are
    // layout outputs, not ambient caller preferences: without explicit values a cleared hook with
    // HOME=$META_ROOT silently falls back to `$META_ROOT/.rustup` + `$META_ROOT/.cargo` instead of
    // envctl's declared `.toolchains/{rustup,cargo}` generation.
    env.push((
        "CARGO_HOME".to_string(),
        layout
            .legacy_toolchains()
            .join("cargo")
            .display()
            .to_string(),
    ));
    env.push((
        "RUSTUP_HOME".to_string(),
        layout
            .legacy_toolchains()
            .join("rustup")
            .display()
            .to_string(),
    ));
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
    // User-systemd is host session IPC, not an install destination. Derive its canonical address
    // from the invoking uid instead of inheriting caller-selected bus/runtime paths; components
    // that manage user units continue to reach this user's manager after env_clear.
    let host_runtime = format!("/run/user/{}", rustix::process::geteuid().as_raw());
    env.push(("XDG_RUNTIME_DIR".to_string(), host_runtime.clone()));
    env.push((
        "DBUS_SESSION_BUS_ADDRESS".to_string(),
        format!("unix:path={host_runtime}/bus"),
    ));

    let meta_bin = layout.bin().display().to_string();
    let compat_local_bin = layout.local_bin().display().to_string();
    let legacy_cargo = layout
        .legacy_toolchains()
        .join("cargo/bin")
        .display()
        .to_string();
    let path = [
        meta_bin,
        compat_local_bin,
        legacy_cargo,
        "/usr/local/bin".to_string(),
        "/usr/bin".to_string(),
        "/bin".to_string(),
        "/usr/local/sbin".to_string(),
        "/usr/sbin".to_string(),
        "/sbin".to_string(),
    ];
    env.push(("PATH".to_string(), path.join(":")));
    env
}

fn suppress_shadowed_proxy_aliases(
    inherited: &mut Vec<(String, String)>,
    hook_env: &[(String, String)],
) {
    const ALIASES: [(&str, &str); 5] = [
        ("HTTP_PROXY", "http_proxy"),
        ("HTTPS_PROXY", "https_proxy"),
        ("ALL_PROXY", "all_proxy"),
        ("NO_PROXY", "no_proxy"),
        ("FTP_PROXY", "ftp_proxy"),
    ];
    for (upper, lower) in ALIASES {
        if hook_env.iter().any(|(key, _)| key == upper || key == lower) {
            inherited.retain(|(key, _)| key != upper && key != lower);
        }
    }
}

/// The complete caller-environment compatibility bridge for component hooks.
///
/// This is deliberately an allowlist, not a denylist: language runtimes, compilers, package
/// managers, build systems, Git, SSH askpass helpers, and dynamic loaders all have environment
/// variables that can execute caller-selected code before (or while) a trusted manifest hook runs.
/// A denylist can never enumerate those surfaces safely. Locale/terminal presentation, network
/// proxy and CA selection, plus explicitly audited workflow selectors are the only ambient inputs
/// hooks retain. A trusted manifest may still declare any non-layout variable explicitly; exact
/// keys win, and proxy aliases are suppressed as a family. Enforced layout and host-session
/// outputs are appended last.
const SAFE_INHERITED_ENV_KEYS: &[&str] = &[
    // Locale. Do not include LOCPATH or GCONV_PATH: both select caller-owned runtime data/code.
    "LANG",
    "LANGUAGE",
    "LC_ALL",
    "LC_ADDRESS",
    "LC_COLLATE",
    "LC_CTYPE",
    "LC_IDENTIFICATION",
    "LC_MEASUREMENT",
    "LC_MESSAGES",
    "LC_MONETARY",
    "LC_NAME",
    "LC_NUMERIC",
    "LC_PAPER",
    "LC_TELEPHONE",
    "LC_TIME",
    // Terminal/color presentation. PAGER, EDITOR, TERMINFO, and similar executable/path controls
    // are intentionally absent.
    "TERM",
    "COLORTERM",
    "NO_COLOR",
    "CLICOLOR",
    "CLICOLOR_FORCE",
    "FORCE_COLOR",
    // Operator network routing. Preserve both conventional cases because curl and other clients
    // do not treat every proxy spelling identically.
    "HTTP_PROXY",
    "HTTPS_PROXY",
    "ALL_PROXY",
    "NO_PROXY",
    "FTP_PROXY",
    "http_proxy",
    "https_proxy",
    "all_proxy",
    "no_proxy",
    "ftp_proxy",
    // Enterprise/private CA inputs. These select trust data only; verification-disabling knobs
    // (for example GIT_SSL_NO_VERIFY) are intentionally absent.
    "SSL_CERT_FILE",
    "SSL_CERT_DIR",
    "CURL_CA_BUNDLE",
    "REQUESTS_CA_BUNDLE",
    "NODE_EXTRA_CA_CERTS",
    "GIT_SSL_CAINFO",
    "NIX_SSL_CERT_FILE",
    // Documented envctl selectors needed when the engine is run from an isolated worktree or a
    // non-default manifest. Other ENVCTL_* values are layout outputs or must be manifest-declared.
    "ENVCTL_SOURCE_ROOT",
    "ENVCTL_MANIFEST_DIR",
    // Documented Codex release selectors. Every consuming manifest hook validates these as a
    // non-empty, traversal-free single release-version segment before using them in a path or URL.
    "CODEX_VERSION",
    "CODEX_ALPHA_VERSION",
    // Non-secret decimal inputs for the shared GitHub App token resolver. The helper validates
    // both against their CLI integer domains before invoking secretctl.
    "ENVCTL_GH_INSTALLATION_ID",
    "ENVCTL_GH_TTL_SECS",
    // sqld's documented paired adoption inputs. These carry paths, never secret bytes; the sqld
    // hook requires both together and validates canonical current-user-owned 0600 regular files
    // before reading or committing either one.
    "SQLD_AUTH_JWT_KEY_SOURCE",
    "SQLD_CLIENT_JWT_SOURCE",
];

/// Preserve only explicitly audited caller configuration inputs. PATH and layout variables are
/// replaced later; manifest-declared hook env remains authoritative over this compatibility layer.
fn sanitized_inherited_env() -> Vec<(String, String)> {
    SAFE_INHERITED_ENV_KEYS
        .iter()
        .filter_map(|key| {
            std::env::var(key)
                .ok()
                .map(|value| ((*key).to_string(), value))
        })
        .collect()
}

/// Host control programs that form the hook entry boundary must never resolve through the mutable
/// meta prefix or caller PATH. Other command hooks intentionally resolve in envctl's deterministic
/// PATH so meta-owned component frontdoors (for example `weave`) remain usable.
fn trusted_hook_entry(command: &str) -> String {
    match command {
        "bash" | "/bin/bash" | "/usr/bin/bash" => "/usr/bin/bash".to_string(),
        "sh" | "/bin/sh" | "/usr/bin/sh" => "/usr/bin/sh".to_string(),
        "apt-get" | "/usr/bin/apt-get" => "/usr/bin/apt-get".to_string(),
        "nvidia-ctk" | "/usr/bin/nvidia-ctk" => "/usr/bin/nvidia-ctk".to_string(),
        _ => command.to_string(),
    }
}

/// Privileged hooks must not search the user-writable meta prefix. `sudo --preserve-env` keeps the
/// exact PATH we pass it under SETENV policy, so leaving the ordinary meta-first hook PATH in place
/// would let a `$META_ROOT/usr/bin/install` (or any other shadow) execute as root from an inline or
/// shipped script. Every current privileged hook depends only on Ubuntu host utilities; meta-owned
/// executables needed by generated workers are addressed by absolute `$META_ROOT/...` paths.
fn enforce_privileged_path(env: &mut Vec<(String, String)>) {
    const PRIVILEGED_HOST_PATH: &str = "/usr/sbin:/usr/bin:/sbin:/bin";
    env.retain(|(key, _)| key != "PATH");
    env.push(("PATH".to_string(), PRIVILEGED_HOST_PATH.to_string()));
}

/// Wrap a hook in non-interactive sudo while requesting preservation of the exact final
/// sanitized/enforced key set. Only names enter argv; values remain in the cleared Command
/// environment. sudo-rs either preserves the values under SETENV policy or refuses explicitly —
/// it must never silently run a privileged hook without its META/XDG inputs.
fn sudo_wrap(
    command: String,
    command_args: Vec<String>,
    needs_sudo: bool,
    env: &[(String, String)],
) -> Result<(String, Vec<String>), String> {
    if needs_sudo {
        if let Some((invalid, _)) = env.iter().find(|(key, _)| !valid_env_name(key)) {
            return Err(format!(
                "sudo hook environment key is not a portable name: {invalid:?}"
            ));
        }
        let names = env
            .iter()
            .map(|(key, _)| key.as_str())
            .collect::<std::collections::BTreeSet<_>>()
            .into_iter()
            .collect::<Vec<_>>()
            .join(",");
        let mut args = vec![
            "-n".to_string(),
            format!("--preserve-env={names}"),
            "--".to_string(),
            command,
        ];
        args.extend(command_args);
        Ok(("/usr/bin/sudo".to_string(), args))
    } else {
        Ok((command, command_args))
    }
}

fn valid_env_name(key: &str) -> bool {
    let mut bytes = key.bytes();
    matches!(bytes.next(), Some(b'A'..=b'Z' | b'a'..=b'z' | b'_'))
        && bytes.all(|byte| matches!(byte, b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'_'))
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

    struct EnvRestore(Vec<(String, Option<std::ffi::OsString>)>);

    impl EnvRestore {
        fn set(values: &[(&str, &std::ffi::OsStr)]) -> Self {
            let prior = values
                .iter()
                .map(|(key, _)| ((*key).to_string(), std::env::var_os(key)))
                .collect();
            for (key, value) in values {
                std::env::set_var(key, value);
            }
            Self(prior)
        }
    }

    impl Drop for EnvRestore {
        fn drop(&mut self) {
            for (key, value) in self.0.drain(..) {
                match value {
                    Some(value) => std::env::set_var(key, value),
                    None => std::env::remove_var(key),
                }
            }
        }
    }

    #[cfg(unix)]
    fn runner_test_dir(label: &str) -> std::path::PathBuf {
        let path = std::env::temp_dir().join(format!(
            "envctl-runner-{label}-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("system clock after epoch")
                .as_nanos()
        ));
        std::fs::create_dir(&path).expect("create runner test directory");
        path
    }

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

    #[test]
    fn read_only_timeout_covers_full_generation_integrity_proofs() {
        let _lock = crate::test_env_lock();
        let previous = std::env::var_os("ENVCTL_HOOK_TIMEOUT_MS");
        std::env::remove_var("ENVCTL_HOOK_TIMEOUT_MS");
        assert_eq!(timeout_for(Phase::Detect), Duration::from_secs(300));
        assert_eq!(timeout_for(Phase::Verify), Duration::from_secs(300));
        match previous {
            Some(value) => std::env::set_var("ENVCTL_HOOK_TIMEOUT_MS", value),
            None => std::env::remove_var("ENVCTL_HOOK_TIMEOUT_MS"),
        }
    }

    #[cfg(unix)]
    #[test]
    fn script_entrypoint_ignores_meta_and_caller_path_bash_shadows() {
        use std::os::unix::fs::PermissionsExt;

        let _lock = crate::test_env_lock();
        let root = runner_test_dir("path-shadow");
        let meta_bin = root.join("usr/bin");
        let marker = root.join("shadow-ran");
        std::fs::create_dir_all(&meta_bin).unwrap();
        let fake_bash = meta_bin.join("bash");
        std::fs::write(
            &fake_bash,
            format!("#!/bin/sh\n/usr/bin/touch '{}'\n", marker.display()),
        )
        .unwrap();
        std::fs::set_permissions(&fake_bash, std::fs::Permissions::from_mode(0o755)).unwrap();
        let caller_bin = root.join("caller-bin");
        std::fs::create_dir(&caller_bin).unwrap();
        std::fs::copy(&fake_bash, caller_bin.join("bash")).unwrap();

        let _restore = EnvRestore::set(&[
            ("META_ROOT", root.as_os_str()),
            ("HOME", root.as_os_str()),
            ("PATH", caller_bin.as_os_str()),
        ]);
        let hook = Hook::Script {
            script: ":".into(),
            path: None,
            env: BTreeMap::new(),
            needs_sudo: false,
            login_shell: false,
        };
        let status = build_command(&hook)
            .expect("construct trusted bash")
            .status()
            .expect("run trusted bash");
        assert!(status.success());
        assert!(!marker.exists(), "PATH-resolved bash shadow executed");
        assert_eq!(build_command(&hook).unwrap().get_program(), "/usr/bin/bash");
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn sudo_wrapper_preserves_sanitized_names_without_putting_values_in_argv() {
        let _lock = crate::test_env_lock();
        let _restore = EnvRestore::set(&[("META_ROOT", std::ffi::OsStr::new("/trusted/meta"))]);
        let hook = Hook::Command {
            command: "/usr/bin/printf".into(),
            args: vec!["ok".into()],
            env: BTreeMap::from([(
                "MANIFEST_PRIVATE_VALUE".to_string(),
                "must-remain-out-of-argv".to_string(),
            )]),
            needs_sudo: true,
        };
        let command = build_command(&hook).unwrap();
        assert_eq!(command.get_program(), "/usr/bin/sudo");
        let args = command
            .get_args()
            .map(|arg| arg.to_string_lossy().into_owned())
            .collect::<Vec<_>>();
        assert_eq!(args[0], "-n");
        assert!(args[1].starts_with("--preserve-env="));
        assert!(args[1].split_once('=').is_some_and(|(_, names)| names
            .split(',')
            .any(|name| name == "META_ROOT")
            && names
                .split(',')
                .any(|name| name == "MANIFEST_PRIVATE_VALUE")));
        assert_eq!(&args[2..], &["--", "/usr/bin/printf", "ok"]);
        let command_env = command
            .get_envs()
            .filter_map(|(key, value)| {
                value.map(|value| {
                    (
                        key.to_string_lossy().into_owned(),
                        value.to_string_lossy().into_owned(),
                    )
                })
            })
            .collect::<BTreeMap<_, _>>();
        assert_eq!(
            command_env.get("PATH").map(String::as_str),
            Some("/usr/sbin:/usr/bin:/sbin:/bin")
        );
        assert!(
            !command_env["PATH"].contains("/trusted/meta"),
            "a privileged hook must never search the user-writable meta prefix"
        );
        assert!(
            args.iter()
                .all(|arg| !arg.contains("must-remain-out-of-argv")),
            "environment values must remain in Command env, never sudo argv"
        );

        let invalid = Hook::Command {
            command: "/usr/bin/true".into(),
            args: Vec::new(),
            env: BTreeMap::from([("GOOD,BAD".to_string(), "value".to_string())]),
            needs_sudo: true,
        };
        assert!(build_command(&invalid)
            .unwrap_err()
            .contains("not a portable name"));

        let nvidia = Hook::Command {
            command: "nvidia-ctk".into(),
            args: vec!["cdi".into(), "generate".into()],
            env: BTreeMap::new(),
            needs_sudo: true,
        };
        let nvidia_args = build_command(&nvidia)
            .unwrap()
            .get_args()
            .map(|arg| arg.to_string_lossy().into_owned())
            .collect::<Vec<_>>();
        assert_eq!(nvidia_args[3], "/usr/bin/nvidia-ctk");
    }

    #[cfg(unix)]
    #[test]
    fn shipped_script_uses_validated_envctl_source_root() {
        use std::os::unix::fs::{symlink, PermissionsExt};

        let _lock = crate::test_env_lock();
        let meta = runner_test_dir("shipped-meta");
        let source = runner_test_dir("shipped-source");
        let assets = source.join("assets/scripts");
        std::fs::create_dir_all(&assets).unwrap();
        let observed = source.join("observed");
        let script = assets.join("probe.sh");
        std::fs::write(
            &script,
            format!("#!/bin/sh\nprintf shipped > '{}'\n", observed.display()),
        )
        .unwrap();
        std::fs::set_permissions(&script, std::fs::Permissions::from_mode(0o755)).unwrap();
        let _restore = EnvRestore::set(&[
            ("META_ROOT", meta.as_os_str()),
            ("HOME", meta.as_os_str()),
            ("ENVCTL_SOURCE_ROOT", source.as_os_str()),
        ]);
        let hook = Hook::ShippedScript {
            path: "$ENVCTL_SOURCE_ROOT/assets/scripts/probe.sh".into(),
            args: Vec::new(),
            needs_sudo: false,
        };
        let (sink, _rx) = EventSink::channel();
        let result = ProcessRunner.run("shipped-source-test", Phase::Verify, &hook, false, &sink);
        assert_eq!(result.status, OpStatus::Ok, "{}", result.message);
        assert_eq!(std::fs::read_to_string(&observed).unwrap(), "shipped");

        let linked = source.with_extension("link");
        symlink(&source, &linked).unwrap();
        std::env::set_var("ENVCTL_SOURCE_ROOT", &linked);
        assert!(build_command(&hook)
            .unwrap_err()
            .contains("unsafe authority path"));
        let _ = std::fs::remove_file(linked);
        let _ = std::fs::remove_dir_all(meta);
        let _ = std::fs::remove_dir_all(source);
    }

    #[cfg(unix)]
    #[test]
    fn script_entrypoint_drops_bash_startup_and_dynamic_loader_injection() {
        let _lock = crate::test_env_lock();
        let root = runner_test_dir("startup-env");
        let bash_env = root.join("bash-env");
        let marker = root.join("bash-env-ran");
        std::fs::write(
            &bash_env,
            format!("/usr/bin/touch '{}'\n", marker.display()),
        )
        .unwrap();
        let _restore = EnvRestore::set(&[
            ("META_ROOT", root.as_os_str()),
            ("HOME", root.as_os_str()),
            ("PATH", std::ffi::OsStr::new("/usr/bin:/bin")),
            ("BASH_ENV", bash_env.as_os_str()),
            ("LD_PRELOAD", std::ffi::OsStr::new("/must/not/load.so")),
        ]);
        let hook = Hook::Script {
            script: ":".into(),
            path: None,
            env: BTreeMap::new(),
            needs_sudo: false,
            login_shell: false,
        };
        let output = build_command(&hook)
            .expect("construct sanitized bash")
            .output()
            .expect("run sanitized bash");
        assert!(
            output.status.success(),
            "trusted shell failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(!marker.exists(), "BASH_ENV startup injection executed");
        assert!(
            !String::from_utf8_lossy(&output.stderr).contains("LD_PRELOAD"),
            "dynamic-loader injection reached trusted bash"
        );
        let _ = std::fs::remove_dir_all(root);
    }

    #[cfg(unix)]
    #[test]
    fn process_runner_drops_python_startup_injection_from_caller_environment() {
        let _lock = crate::test_env_lock();
        let root = runner_test_dir("python-startup-env");
        let python_path = root.join("python-path");
        let marker = root.join("sitecustomize-ran");
        let observed = root.join("observed-pythonpath");
        std::fs::create_dir(&python_path).unwrap();
        std::fs::write(
            python_path.join("sitecustomize.py"),
            format!(
                "open({:?}, 'w').write('ran')\n",
                marker.display().to_string()
            ),
        )
        .unwrap();

        let _restore = EnvRestore::set(&[
            ("META_ROOT", root.as_os_str()),
            ("HOME", root.as_os_str()),
            ("PYTHONPATH", python_path.as_os_str()),
        ]);
        let hook = Hook::Script {
            script: format!(
                "printf '%s' \"${{PYTHONPATH-unset}}\" > '{}'; /usr/bin/python3 -c 'pass'",
                observed.display()
            ),
            path: None,
            env: BTreeMap::new(),
            needs_sudo: false,
            login_shell: false,
        };
        let (sink, _rx) = EventSink::channel();
        let result = ProcessRunner.run("python-env-test", Phase::Verify, &hook, false, &sink);

        assert_eq!(result.status, OpStatus::Ok, "{}", result.message);
        assert_eq!(std::fs::read_to_string(&observed).unwrap(), "unset");
        assert!(
            !marker.exists(),
            "caller PYTHONPATH loaded sitecustomize before the trusted hook body"
        );
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn inherited_environment_is_an_explicit_allowlist_and_manifest_env_wins() {
        let _lock = crate::test_env_lock();
        let _restore = EnvRestore::set(&[
            ("META_ROOT", std::ffi::OsStr::new("/trusted/meta")),
            ("LANG", std::ffi::OsStr::new("C.UTF-8")),
            (
                "HTTPS_PROXY",
                std::ffi::OsStr::new("http://caller-proxy.invalid"),
            ),
            (
                "https_proxy",
                std::ffi::OsStr::new("http://caller-lowercase-proxy.invalid"),
            ),
            (
                "SSL_CERT_FILE",
                std::ffi::OsStr::new("/caller/certificates.pem"),
            ),
            (
                "ENVCTL_SOURCE_ROOT",
                std::ffi::OsStr::new("/caller/envctl-source"),
            ),
            (
                "ENVCTL_MANIFEST_DIR",
                std::ffi::OsStr::new("/caller/envctl-manifest"),
            ),
            ("CODEX_VERSION", std::ffi::OsStr::new("0.142.3+meta_1")),
            (
                "CODEX_ALPHA_VERSION",
                std::ffi::OsStr::new("0.143.0-alpha.29_meta+1"),
            ),
            (
                "ENVCTL_GH_INSTALLATION_ID",
                std::ffi::OsStr::new("140063898"),
            ),
            ("ENVCTL_GH_TTL_SECS", std::ffi::OsStr::new("3600")),
            (
                "SQLD_AUTH_JWT_KEY_SOURCE",
                std::ffi::OsStr::new("/caller/sqld-key.pem"),
            ),
            (
                "SQLD_CLIENT_JWT_SOURCE",
                std::ffi::OsStr::new("/caller/sqld-client.jwt"),
            ),
            ("RUSTUP_HOME", std::ffi::OsStr::new("/hostile/rustup")),
            ("XDG_RUNTIME_DIR", std::ffi::OsStr::new("/hostile/runtime")),
            (
                "DBUS_SESSION_BUS_ADDRESS",
                std::ffi::OsStr::new("unix:path=/hostile/bus"),
            ),
            ("PYTHONHOME", std::ffi::OsStr::new("/hostile/python")),
            (
                "NODE_OPTIONS",
                std::ffi::OsStr::new("--require=/hostile/startup.js"),
            ),
            ("RUBYOPT", std::ffi::OsStr::new("-r/hostile/startup.rb")),
            ("PERL5OPT", std::ffi::OsStr::new("-MHostile")),
            (
                "JAVA_TOOL_OPTIONS",
                std::ffi::OsStr::new("-javaagent:/hostile/agent.jar"),
            ),
            ("RUSTC_WRAPPER", std::ffi::OsStr::new("/hostile/rustc")),
            ("CARGO_HOME", std::ffi::OsStr::new("/hostile/cargo")),
            ("CC", std::ffi::OsStr::new("/hostile/cc")),
            ("MAKEFLAGS", std::ffi::OsStr::new("--eval=hostile")),
            (
                "GIT_CONFIG_GLOBAL",
                std::ffi::OsStr::new("/hostile/gitconfig"),
            ),
            ("GIT_SSH_COMMAND", std::ffi::OsStr::new("/hostile/git-ssh")),
            (
                "NIX_CONFIG",
                std::ffi::OsStr::new("plugin-files = /hostile"),
            ),
            (
                "DOTNET_STARTUP_HOOKS",
                std::ffi::OsStr::new("/hostile/dotnet.dll"),
            ),
            ("GOENV", std::ffi::OsStr::new("/hostile/goenv")),
            (
                "CMAKE_PROJECT_INCLUDE",
                std::ffi::OsStr::new("/hostile/project.cmake"),
            ),
            ("GIT_ASKPASS", std::ffi::OsStr::new("/hostile/askpass")),
        ]);
        let env = enforced_meta_env(vec![
            (
                "HTTPS_PROXY".to_string(),
                "http://manifest-proxy.invalid".to_string(),
            ),
            ("MANIFEST_ONLY".to_string(), "trusted".to_string()),
            ("META_ROOT".to_string(), "/manifest/escape".to_string()),
            ("CARGO_HOME".to_string(), "/manifest/cargo".to_string()),
            ("RUSTUP_HOME".to_string(), "/manifest/rustup".to_string()),
            ("XDG_CACHE_HOME".to_string(), "/manifest/cache".to_string()),
            ("PATH".to_string(), "/manifest/bin".to_string()),
        ]);

        assert_eq!(env_value(&env, "LANG"), "C.UTF-8");
        assert_eq!(
            env_value(&env, "HTTPS_PROXY"),
            "http://manifest-proxy.invalid"
        );
        assert!(
            !env.iter().any(|(key, _)| key == "https_proxy"),
            "caller lowercase proxy alias must not shadow manifest HTTPS_PROXY"
        );
        assert_eq!(env_value(&env, "SSL_CERT_FILE"), "/caller/certificates.pem");
        assert_eq!(
            env_value(&env, "ENVCTL_SOURCE_ROOT"),
            "/caller/envctl-source"
        );
        assert_eq!(
            env_value(&env, "ENVCTL_MANIFEST_DIR"),
            "/caller/envctl-manifest"
        );
        assert_eq!(env_value(&env, "CODEX_VERSION"), "0.142.3+meta_1");
        assert_eq!(
            env_value(&env, "CODEX_ALPHA_VERSION"),
            "0.143.0-alpha.29_meta+1"
        );
        assert_eq!(env_value(&env, "ENVCTL_GH_INSTALLATION_ID"), "140063898");
        assert_eq!(env_value(&env, "ENVCTL_GH_TTL_SECS"), "3600");
        assert_eq!(
            env_value(&env, "SQLD_AUTH_JWT_KEY_SOURCE"),
            "/caller/sqld-key.pem"
        );
        assert_eq!(
            env_value(&env, "SQLD_CLIENT_JWT_SOURCE"),
            "/caller/sqld-client.jwt"
        );
        assert_eq!(env_value(&env, "MANIFEST_ONLY"), "trusted");
        assert_eq!(env_value(&env, "META_ROOT"), "/trusted/meta");
        assert_eq!(
            env_value(&env, "CARGO_HOME"),
            "/trusted/meta/.toolchains/cargo"
        );
        assert_eq!(
            env_value(&env, "RUSTUP_HOME"),
            "/trusted/meta/.toolchains/rustup"
        );
        assert_eq!(env_value(&env, "XDG_CACHE_HOME"), "/trusted/meta/.cache");
        let expected_runtime = format!("/run/user/{}", rustix::process::geteuid().as_raw());
        assert_eq!(env_value(&env, "XDG_RUNTIME_DIR"), expected_runtime);
        assert_eq!(
            env_value(&env, "DBUS_SESSION_BUS_ADDRESS"),
            format!("unix:path={expected_runtime}/bus")
        );
        assert_eq!(
            env_value(&env, "PATH").split(':').next(),
            Some("/trusted/meta/usr/bin")
        );

        for key in [
            "PYTHONHOME",
            "NODE_OPTIONS",
            "RUBYOPT",
            "PERL5OPT",
            "JAVA_TOOL_OPTIONS",
            "RUSTC_WRAPPER",
            "CC",
            "MAKEFLAGS",
            "GIT_CONFIG_GLOBAL",
            "GIT_SSH_COMMAND",
            "NIX_CONFIG",
            "DOTNET_STARTUP_HOOKS",
            "GOENV",
            "CMAKE_PROJECT_INCLUDE",
            "GIT_ASKPASS",
        ] {
            assert!(
                !env.iter().any(|(candidate, _)| candidate == key),
                "dangerous inherited variable crossed the hook boundary: {key}"
            );
        }
    }

    #[cfg(unix)]
    #[test]
    fn login_script_ignores_home_profile_language_startup_injection() {
        let _lock = crate::test_env_lock();
        let root = runner_test_dir("login-profile-env");
        let python_path = root.join("python-path");
        let marker = root.join("login-profile-sitecustomize-ran");
        std::fs::create_dir(&python_path).unwrap();
        std::fs::write(
            python_path.join("sitecustomize.py"),
            format!(
                "open({:?}, 'w').write('ran')\n",
                marker.display().to_string()
            ),
        )
        .unwrap();
        std::fs::write(
            root.join(".bash_profile"),
            format!("export PYTHONPATH='{}'\n", python_path.display()),
        )
        .unwrap();
        let _restore =
            EnvRestore::set(&[("META_ROOT", root.as_os_str()), ("HOME", root.as_os_str())]);
        let hook = Hook::Script {
            script: "test \"${PYTHONPATH-unset}\" = unset; /usr/bin/python3 -c 'pass'".into(),
            path: None,
            env: BTreeMap::new(),
            needs_sudo: false,
            login_shell: true,
        };
        let (sink, _rx) = EventSink::channel();
        let result = ProcessRunner.run("login-profile-test", Phase::Verify, &hook, false, &sink);

        assert_eq!(result.status, OpStatus::Ok, "{}", result.message);
        assert!(
            !marker.exists(),
            "login startup profile reintroduced PYTHONPATH/sitecustomize"
        );
        let _ = std::fs::remove_dir_all(root);
    }

    #[cfg(unix)]
    #[test]
    fn command_bash_login_flag_also_ignores_home_profile() {
        let _lock = crate::test_env_lock();
        let root = runner_test_dir("command-login-profile");
        let marker = root.join("command-profile-ran");
        std::fs::write(
            root.join(".bash_profile"),
            format!("/usr/bin/touch '{}'\n", marker.display()),
        )
        .unwrap();
        let _restore =
            EnvRestore::set(&[("META_ROOT", root.as_os_str()), ("HOME", root.as_os_str())]);
        let hook = Hook::Command {
            command: "bash".into(),
            args: vec!["-lc".into(), ":".into()],
            env: BTreeMap::new(),
            needs_sudo: false,
        };
        let (sink, _rx) = EventSink::channel();
        let result = ProcessRunner.run("command-profile-test", Phase::Verify, &hook, false, &sink);

        assert_eq!(result.status, OpStatus::Ok, "{}", result.message);
        assert!(!marker.exists(), "command=bash sourced the home profile");
        let _ = std::fs::remove_dir_all(root);
    }

    #[cfg(unix)]
    #[test]
    fn login_shell_uses_meta_owned_cargo_and_rustup_homes() {
        use std::os::unix::fs::{symlink, PermissionsExt};

        let _lock = crate::test_env_lock();
        let root = runner_test_dir("rust-homes");
        let bin = root.join(".toolchains/cargo/bin");
        std::fs::create_dir_all(&bin).unwrap();
        let rustup = bin.join("rustup");
        std::fs::write(
            &rustup,
            "#!/bin/sh\nprintf '%s\\n%s\\n' \"$CARGO_HOME\" \"$RUSTUP_HOME\"\n",
        )
        .unwrap();
        std::fs::set_permissions(&rustup, std::fs::Permissions::from_mode(0o755)).unwrap();
        symlink("rustup", bin.join("cargo")).unwrap();
        let observed = root.join("observed-rust-homes");
        let _restore = EnvRestore::set(&[
            ("META_ROOT", root.as_os_str()),
            ("HOME", root.as_os_str()),
            ("CARGO_HOME", std::ffi::OsStr::new("/caller/cargo")),
            ("RUSTUP_HOME", std::ffi::OsStr::new("/caller/rustup")),
        ]);
        let hook = Hook::Script {
            script: format!("cargo > '{}'", observed.display()),
            path: None,
            env: BTreeMap::from([
                ("CARGO_HOME".to_string(), "/manifest/cargo".to_string()),
                ("RUSTUP_HOME".to_string(), "/manifest/rustup".to_string()),
            ]),
            needs_sudo: false,
            login_shell: true,
        };
        let (sink, _rx) = EventSink::channel();
        let result = ProcessRunner.run("rust-home-test", Phase::Verify, &hook, false, &sink);

        assert_eq!(result.status, OpStatus::Ok, "{}", result.message);
        assert_eq!(
            std::fs::read_to_string(&observed).unwrap(),
            format!(
                "{}\n{}\n",
                root.join(".toolchains/cargo").display(),
                root.join(".toolchains/rustup").display()
            )
        );
        let _ = std::fs::remove_dir_all(root);
    }
}
