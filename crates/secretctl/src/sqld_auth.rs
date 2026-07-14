//! One-shot, Rust-native bootstrap for sqld JWT authentication.
//!
//! sqld v0.24.32 accepts an Ed25519 SubjectPublicKeyInfo `PUBLIC KEY` PEM and verifies EdDSA JWTs
//! against it. The component needs an unattended way to create that pair without introducing
//! Python, Node, OpenSSL, or a static credential. This module is deliberately private to
//! `secretctl`; the hidden CLI command is an install-time seam owned by `manifest/sqld.toml`.

use std::collections::HashSet;
use std::ffi::OsStr;
use std::fs::{self, File, OpenOptions};
use std::io::{self, Read, Write};
use std::net::{Ipv4Addr, SocketAddrV4, TcpStream};
use std::os::unix::ffi::OsStrExt;
use std::os::unix::fs::{DirBuilderExt, MetadataExt, OpenOptionsExt, PermissionsExt};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use anyhow::{anyhow, bail, Context};
use envctl_secrets::sqld_auth::generate_sqld_auth_material;
use serde_json::Value;
use sha2::{Digest, Sha256};
use zeroize::Zeroizing;

pub(crate) fn pinned_sqld_payload_sha256() -> anyhow::Result<&'static str> {
    match std::env::consts::ARCH {
        "x86_64" => Ok("0863c3fbe68ac9714bca2cec1330def7a0ba5e4a29f199bf60ef46fa0c95b895"),
        "aarch64" => Ok("54039931c1088483706790e6cf73444ad88b843a9bb0ca8285b82fc309ad4810"),
        arch => bail!("unsupported architecture {arch} for pinned sqld v0.24.32"),
    }
}

/// Create sqld's verification key and matching bearer. Existing destinations are never replaced.
pub(crate) fn bootstrap(public_key: &Path, client_token: &Path) -> anyhow::Result<()> {
    if public_key == client_token {
        bail!("sqld public-key and client-token destinations must differ");
    }
    refuse_existing(public_key, "sqld public key")?;
    refuse_existing(client_token, "sqld client token")?;

    let public_parent = prepare_auth_parent(public_key, "sqld public key")?;
    let token_parent = prepare_auth_parent(client_token, "sqld client token")?;

    // Re-check after creating the parent directories. This closes the ordinary preflight race;
    // the hard-link commit below is the final fail-closed, no-replace operation.
    refuse_existing(public_key, "sqld public key")?;
    refuse_existing(client_token, "sqld client token")?;

    let material = generate_sqld_auth_material()?;
    let public_stage = StagedFile::write(public_parent.path(), "public", &material.public_key_pem)
        .context("staging the sqld public key")?;
    let token_stage = StagedFile::write(token_parent.path(), "token", &material.client_jwt)
        .context("staging the sqld client token")?;

    public_stage
        .commit_no_replace(public_key)
        .context("committing the sqld public key without overwrite")?;
    if let Err(error) = token_stage.commit_no_replace(client_token) {
        public_stage.rollback_link(public_key);
        return Err(error).context("committing the sqld client token without overwrite");
    }

    public_stage.remove_stage()?;
    token_stage.remove_stage()?;
    public_parent.revalidate_and_sync()?;
    if token_parent.path() != public_parent.path() {
        token_parent.revalidate_and_sync()?;
    }
    Ok(())
}

/// Block until systemd's exact sqld MainPID owns the loopback listener and enforces the managed
/// JWT pair. This is the `ExecStartPost=` barrier for `sqld.service`; systemd includes its successful
/// completion in `Before=`/`After=` ordering, so dependent secretd cannot start against a stale or
/// foreign listener.
///
/// The bearer is accepted only through `client_token`, whose ownership/mode are checked before it is
/// read. It is never placed in argv, the environment, or an error/log message.
pub(crate) struct ReadinessProbeRequest<'a> {
    pub(crate) pid: u32,
    pub(crate) expected_executable: &'a Path,
    pub(crate) expected_sha256: &'a [String],
    pub(crate) expected_mode: u32,
    pub(crate) port: u16,
    pub(crate) client_token: &'a Path,
    pub(crate) helper_digest: &'a Path,
    pub(crate) timeout: Duration,
}

pub(crate) fn readiness_probe(request: ReadinessProbeRequest<'_>) -> anyhow::Result<()> {
    let ReadinessProbeRequest {
        pid,
        expected_executable,
        expected_sha256,
        expected_mode,
        port,
        client_token,
        helper_digest,
        timeout,
    } = request;
    if pid == 0 {
        bail!("sqld readiness probe requires a nonzero MainPID");
    }
    if port == 0 {
        bail!("sqld readiness probe requires a nonzero loopback port");
    }
    if timeout.is_zero() {
        bail!("sqld readiness probe timeout must be nonzero");
    }
    if expected_sha256.is_empty() {
        bail!("sqld readiness probe requires at least one pinned payload SHA-256");
    }
    for digest in expected_sha256 {
        validate_sha256(digest, "expected sqld payload SHA-256")?;
    }

    // This MUST precede reading the bearer or opening a socket. The unit invokes no system-owned
    // checksum utility: the pure-Rust helper proves its own bytes against the component-owned 0600
    // record before the credential path crosses any auth/network code.
    verify_self_digest(helper_digest)?;

    let start_time = process_start_time(pid)
        .with_context(|| format!("capturing sqld MainPID {pid} identity"))?;
    let deadline = Instant::now()
        .checked_add(timeout)
        .ok_or_else(|| anyhow::anyhow!("sqld readiness timeout overflow"))?;
    // Type=exec may start ExecStartPost while the managed shell frontdoor is completing its final
    // exec. Do not touch the bearer or network until /proc/MainPID/exe is the exact managed inode
    // and the bytes read from that open proc handle match a pinned release digest.
    let executable_identity = loop {
        if process_start_time(pid).ok().as_deref() != Some(start_time.as_str()) {
            bail!("sqld MainPID {pid} exited or changed identity before payload validation");
        }
        match verify_process_executable(pid, expected_executable, expected_sha256, expected_mode)? {
            Some(identity) => break identity,
            None if Instant::now() < deadline => std::thread::sleep(Duration::from_millis(100)),
            None => bail!(
                "sqld readiness timed out before MainPID {pid} exec'd the pinned managed payload"
            ),
        }
    };
    let token = read_safe_token(client_token)?;
    let address = SocketAddrV4::new(Ipv4Addr::LOCALHOST, port);
    loop {
        if process_start_time(pid).ok().as_deref() != Some(start_time.as_str()) {
            bail!("sqld MainPID {pid} exited or changed identity before readiness");
        }
        let current_identity =
            verify_process_executable(pid, expected_executable, expected_sha256, expected_mode)?;
        let last_not_ready = if current_identity == Some(executable_identity) {
            match pid_owns_loopback_listener(pid, port) {
                Ok(true) => match authenticate_once(address, token.as_str()) {
                    Ok(()) => {
                        // Close the response/identity race: success belongs to the same process and
                        // listening socket that systemd named when ExecStartPost began.
                        let still_same = process_start_time(pid).ok().as_deref()
                            == Some(start_time.as_str())
                            && verify_process_executable(
                                pid,
                                expected_executable,
                                expected_sha256,
                                expected_mode,
                            )
                            .ok()
                            .flatten()
                                == Some(executable_identity)
                            && pid_owns_loopback_listener(pid, port).unwrap_or(false);
                        if still_same {
                            return Ok(());
                        }
                        bail!("sqld MainPID/listener identity changed during the auth proof");
                    }
                    Err(AuthAttemptError::NotReady(message)) => message,
                    Err(AuthAttemptError::Refused(error)) => return Err(error),
                },
                Ok(false) => format!("sqld MainPID {pid} does not own 127.0.0.1:{port}"),
                Err(error) => error.to_string(),
            }
        } else {
            format!("sqld MainPID {pid} executable identity changed after validation")
        };

        if Instant::now() >= deadline {
            bail!(
                "sqld readiness timed out after {}s: {last_not_ready}",
                timeout.as_secs()
            );
        }
        std::thread::sleep(Duration::from_millis(100));
    }
}

/// Write the SHA-256 record consumed by [`readiness_probe`]. Existing outputs are never replaced;
/// component install stages a fresh record beside a freshly built helper and atomically renames it.
pub(crate) fn write_self_digest(
    output: &Path,
    installed_path: Option<&Path>,
) -> anyhow::Result<()> {
    refuse_existing(output, "sqld helper digest")?;
    let parent = prepare_record_parent(output, "sqld helper digest")?;
    refuse_existing(output, "sqld helper digest")?;

    let (mut running_file, running) = open_running_executable()?;
    let recorded = match installed_path {
        Some(path) => canonical_destination(path)?,
        None => running.clone(),
    };
    let digest = sha256_reader(&mut running_file, "running sqld helper")?;
    let record = format!("{digest}  {}\n", recorded.display());
    let staged = StagedFile::write(parent.path(), "helper-digest", record.as_bytes())
        .context("staging the sqld helper digest")?;
    staged
        .commit_no_replace(output)
        .context("committing the sqld helper digest without overwrite")?;
    staged.remove_stage()?;
    parent.revalidate_and_sync()
}

pub(crate) fn verify_self_digest(record_path: &Path) -> anyhow::Result<()> {
    let mut record_file = open_safe_0600(record_path, "sqld helper-digest")?;
    let mut record = String::new();
    record_file
        .read_to_string(&mut record)
        .with_context(|| format!("reading sqld helper-digest file {}", record_path.display()))?;
    let record = record
        .strip_suffix('\n')
        .ok_or_else(|| anyhow::anyhow!("sqld helper-digest record is not newline terminated"))?;
    if record.contains('\n') {
        bail!("sqld helper-digest record must contain exactly one line");
    }
    let (expected_digest, expected_path) = record
        .split_once("  ")
        .ok_or_else(|| anyhow::anyhow!("sqld helper-digest record has an invalid shape"))?;
    if expected_digest.len() != 64
        || !expected_digest
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        bail!("sqld helper-digest record has an invalid SHA-256");
    }
    let (mut running_file, running) = open_running_executable()?;
    let recorded = Path::new(expected_path)
        .canonicalize()
        .context("canonicalizing the recorded sqld helper path")?;
    if recorded != running {
        bail!("sqld helper-digest record names a different executable");
    }
    let actual_digest = sha256_reader(&mut running_file, "running sqld helper")?;
    if actual_digest != expected_digest {
        bail!("sqld helper bytes differ from the owned SHA-256 record");
    }
    Ok(())
}

/// Verify an envctl-managed regular file using only Rust. The opened inode is required to be a
/// stable, non-symlink, current-user-owned file with the exact managed mode; its bytes are hashed
/// through that already-open handle, so a path replacement cannot change what is authenticated.
pub(crate) fn verify_sha256(
    path: &Path,
    expected_sha256: &str,
    expected_mode: &str,
) -> anyhow::Result<()> {
    validate_sha256(expected_sha256, "expected SHA-256")?;
    let mode = parse_octal_mode(expected_mode)?;
    let (mut file, _) = open_safe_owned_regular(path, "sqld pinned file", Some(mode))?;
    let actual = sha256_reader(&mut file, "sqld pinned file")?;
    if actual != expected_sha256 {
        bail!("sqld pinned file bytes differ from the expected SHA-256");
    }
    Ok(())
}

/// Verify an incumbent installed helper before replacement. This is invoked by a freshly built,
/// source-owned helper, never by the incumbent itself.
pub(crate) fn verify_owned_digest(path: &Path, record_path: &Path) -> anyhow::Result<()> {
    verified_owned_digest(path, record_path).map(|_| ())
}

pub(crate) fn verify_current_helper(path: &Path, record_path: &Path) -> anyhow::Result<()> {
    let incumbent_digest = verified_owned_digest(path, record_path)?;
    let (mut running, _) = open_running_executable()?;
    let verifier_digest = sha256_reader(&mut running, "fresh sqld helper verifier")?;
    if incumbent_digest != verifier_digest {
        bail!("installed sqld readiness helper differs from the fresh current-source build");
    }
    Ok(())
}

pub(crate) fn commit_helper_generation(
    staged_dir: &Path,
    current_dir: &Path,
) -> anyhow::Result<()> {
    if !staged_dir.is_absolute() || !current_dir.is_absolute() {
        bail!("sqld helper generation paths must be absolute");
    }
    let staged_parent = staged_dir
        .parent()
        .ok_or_else(|| anyhow::anyhow!("staged helper generation has no parent"))?;
    if current_dir.parent() != Some(staged_parent)
        || current_dir.file_name() != Some(OsStr::new("current"))
        || !staged_dir
            .file_name()
            .and_then(OsStr::to_str)
            .is_some_and(|name| name.starts_with(".next."))
    {
        bail!("sqld helper generation exchange paths violate the managed layout");
    }
    let parent = PreparedParent::open(
        staged_parent.to_path_buf(),
        "sqld helper-generation",
        &[0o755],
    )?;
    validate_real_owned_directory(staged_dir, "staged sqld helper generation", &[0o755])?;
    if current_dir.exists() {
        validate_real_owned_directory(current_dir, "current sqld helper generation", &[0o755])?;
    } else if fs::symlink_metadata(current_dir).is_ok() {
        bail!("current sqld helper generation is not a real directory");
    }

    let staged_helper = staged_dir.join("secretctl");
    let staged_digest = staged_dir.join("secretctl.sha256");
    let staged_source = staged_dir.join("secretctl.source.sha256");
    let (expected_digest, recorded_path) = read_path_digest(&staged_digest)?;
    let expected_path = current_dir.join("secretctl");
    if Path::new(&recorded_path) != expected_path {
        bail!("staged helper digest does not name the active generation path");
    }
    let (mut helper, helper_canonical) =
        open_safe_owned_regular(&staged_helper, "staged sqld readiness helper", Some(0o755))?;
    if sha256_reader(&mut helper, "staged sqld readiness helper")? != expected_digest {
        bail!("staged sqld readiness helper differs from its digest record");
    }
    let _ = read_plain_digest(&staged_source, "sqld helper source-digest")?;
    let (_, running_canonical) = open_running_executable()?;
    if running_canonical != helper_canonical {
        bail!("only the staged sqld helper may commit its own generation");
    }

    // A generation is one real directory, never three independently replaced leaf files. Linux's
    // RENAME_EXCHANGE swaps the complete staged and current directories as one atomic namespace
    // operation, so a crash cannot expose an absent `current` or a mixed helper/digest/source
    // triple. We keep an open handle plus device/inode identity for the canonical parent
    // throughout. Adversarial rename by another process running as the same uid is outside this
    // component's threat model.
    parent.revalidate_and_sync()?;
    if current_dir.exists() {
        exchange_generation_dirs(&parent.directory, staged_dir, current_dir)
            .context("atomically exchanging sqld helper generations")?;
        if let Err(sync_error) = parent.revalidate_and_sync() {
            // The exchange succeeded but was not durably synced. Restore the incumbent before
            // reporting the ordinary failure. A reverse-exchange failure is still fail-closed:
            // both sides remain complete, but dependents must not proceed on unproven durability.
            match exchange_generation_dirs(&parent.directory, staged_dir, current_dir) {
                Ok(()) => {
                    let _ = parent.revalidate_and_sync();
                    return Err(sync_error)
                        .context("syncing helper-generation exchange; rollback succeeded");
                }
                Err(rollback_error) => {
                    bail!(
                        "syncing helper-generation exchange failed ({sync_error}); reverse \
                         exchange also failed ({rollback_error}); one complete generation remains \
                         active but durability is unproven"
                    );
                }
            }
        }
        // `staged_dir` now names the complete retired generation. Cleanup is hygiene, not part of
        // activation; a read-only/busy retired tree cannot turn a successful atomic exchange into
        // a false failed-install result.
        cleanup_retired_generation(staged_dir, |path| fs::remove_dir_all(path));
    } else {
        rename_generation_dir(&parent.directory, staged_dir, current_dir)
            .context("atomically activating the first sqld helper generation")?;
        if let Err(sync_error) = parent.revalidate_and_sync() {
            match rename_generation_dir(&parent.directory, current_dir, staged_dir) {
                Ok(()) => {
                    let _ = parent.revalidate_and_sync();
                    return Err(sync_error)
                        .context("syncing first helper-generation activation; rollback succeeded");
                }
                Err(rollback_error) => {
                    bail!(
                        "syncing first helper-generation activation failed ({sync_error}); \
                         rollback also failed ({rollback_error}); the complete generation may \
                         remain active but durability is unproven"
                    );
                }
            }
        }
    }
    Ok(())
}

fn generation_leaf(path: &Path) -> io::Result<&OsStr> {
    path.file_name()
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "generation path has no leaf"))
}

fn exchange_generation_dirs(parent: &File, left: &Path, right: &Path) -> io::Result<()> {
    rustix::fs::renameat_with(
        parent,
        generation_leaf(left)?,
        parent,
        generation_leaf(right)?,
        rustix::fs::RenameFlags::EXCHANGE,
    )
    .map_err(io::Error::from)
}

fn rename_generation_dir(parent: &File, from: &Path, to: &Path) -> io::Result<()> {
    rustix::fs::renameat(parent, generation_leaf(from)?, parent, generation_leaf(to)?)
        .map_err(io::Error::from)
}

fn cleanup_retired_generation(path: &Path, cleanup: impl FnOnce(&Path) -> io::Result<()>) {
    let _ = cleanup(path);
}

fn verified_owned_digest(path: &Path, record_path: &Path) -> anyhow::Result<String> {
    let (expected_digest, recorded_path) = read_path_digest(record_path)?;
    let (mut file, canonical) =
        open_safe_owned_regular(path, "installed sqld readiness helper", Some(0o755))?;
    let recorded = Path::new(&recorded_path)
        .canonicalize()
        .context("canonicalizing the recorded installed-helper path")?;
    if recorded != canonical {
        bail!("sqld helper-digest record names a different installed helper");
    }
    let actual = sha256_reader(&mut file, "installed sqld readiness helper")?;
    if actual != expected_digest {
        bail!("sqld readiness helper differs from its owned-byte digest");
    }
    Ok(actual)
}

pub(crate) fn write_source_digest(
    source_root: &Path,
    toolchain_files: &[PathBuf],
    toolchain_roots: &[PathBuf],
    crate_archives: &[PathBuf],
    output: &Path,
) -> anyhow::Result<()> {
    refuse_existing(output, "sqld helper source digest")?;
    let parent = prepare_record_parent(output, "sqld helper source digest")?;
    refuse_existing(output, "sqld helper source digest")?;
    let digest = compute_source_digest(
        source_root,
        toolchain_files,
        toolchain_roots,
        crate_archives,
    )?;
    let record = format!("{digest}\n");
    let staged = StagedFile::write(parent.path(), "helper-source-digest", record.as_bytes())
        .context("staging the sqld helper source digest")?;
    staged
        .commit_no_replace(output)
        .context("committing the sqld helper source digest without overwrite")?;
    staged.remove_stage()?;
    parent.revalidate_and_sync()
}

pub(crate) fn verify_source_digest(
    source_root: &Path,
    toolchain_files: &[PathBuf],
    toolchain_roots: &[PathBuf],
    crate_archives: &[PathBuf],
    record_path: &Path,
) -> anyhow::Result<()> {
    let expected = read_plain_digest(record_path, "sqld helper source-digest")?;
    let actual = compute_source_digest(
        source_root,
        toolchain_files,
        toolchain_roots,
        crate_archives,
    )?;
    if actual != expected {
        bail!("sqld readiness helper was built from stale Rust/build inputs");
    }
    Ok(())
}

fn canonical_destination(path: &Path) -> anyhow::Result<PathBuf> {
    if !path.is_absolute() {
        bail!("sqld helper installed path must be absolute");
    }
    if path.exists() {
        let canonical = path
            .canonicalize()
            .with_context(|| format!("canonicalizing sqld helper path {}", path.display()))?;
        if canonical != path {
            bail!("sqld helper installed path has a symlinked or non-canonical ancestor");
        }
        return Ok(canonical);
    }
    let parent = path
        .parent()
        .ok_or_else(|| anyhow::anyhow!("sqld helper installed path has no parent"))?;
    let name = path
        .file_name()
        .ok_or_else(|| anyhow::anyhow!("sqld helper installed path has no file name"))?;
    if parent.exists() {
        let canonical_parent = parent
            .canonicalize()
            .with_context(|| format!("canonicalizing sqld helper parent {}", parent.display()))?;
        if canonical_parent != parent {
            bail!("sqld helper installed path has a symlinked or non-canonical ancestor");
        }
        return Ok(canonical_parent.join(name));
    }
    let lexical_grandparent = parent
        .parent()
        .ok_or_else(|| anyhow::anyhow!("sqld helper installed parent has no parent"))?;
    let grandparent = lexical_grandparent.canonicalize().with_context(|| {
        format!(
            "canonicalizing sqld helper generation parent {}",
            lexical_grandparent.display()
        )
    })?;
    if grandparent != lexical_grandparent {
        bail!("sqld helper installed path has a symlinked or non-canonical ancestor");
    }
    let parent_name = parent
        .file_name()
        .ok_or_else(|| anyhow::anyhow!("sqld helper installed parent has no name"))?;
    Ok(grandparent.join(parent_name).join(name))
}

fn open_running_executable() -> anyhow::Result<(File, PathBuf)> {
    let proc_exe = Path::new("/proc/self/exe");
    let linked = fs::read_link(proc_exe).context("reading /proc/self/exe")?;
    if linked.to_string_lossy().ends_with(" (deleted)") {
        bail!("running sqld helper executable has been deleted/replaced");
    }
    let file = File::open(proc_exe).context("opening the running sqld helper by /proc/self/exe")?;
    let handle_metadata = file
        .metadata()
        .context("inspecting the open running sqld helper")?;
    if !handle_metadata.is_file() {
        bail!("running sqld helper is not a regular executable file");
    }
    let linked_metadata = fs::metadata(&linked)
        .with_context(|| format!("inspecting running sqld helper path {}", linked.display()))?;
    if handle_metadata.dev() != linked_metadata.dev()
        || handle_metadata.ino() != linked_metadata.ino()
    {
        bail!("running sqld helper path changed during identity validation");
    }
    let canonical = linked
        .canonicalize()
        .context("canonicalizing the running sqld helper path")?;
    Ok((file, canonical))
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct ProcessExecutableIdentity {
    device: u64,
    inode: u64,
}

/// Return `None` only for the bounded frontdoor-interpreter transition. Once `/proc/PID/exe`
/// resolves to the managed path, every ownership/mode/inode/digest mismatch is an integrity error.
fn verify_process_executable(
    pid: u32,
    expected_executable: &Path,
    expected_sha256: &[String],
    expected_mode: u32,
) -> anyhow::Result<Option<ProcessExecutableIdentity>> {
    let (expected_file, expected_canonical) = open_safe_owned_regular(
        expected_executable,
        "managed sqld payload",
        Some(expected_mode),
    )?;
    let expected_metadata = expected_file
        .metadata()
        .context("inspecting the open managed sqld payload")?;
    let proc_path = PathBuf::from(format!("/proc/{pid}/exe"));
    let linked = fs::read_link(&proc_path)
        .with_context(|| format!("reading sqld MainPID {pid} executable link"))?;
    if linked.to_string_lossy().ends_with(" (deleted)") {
        bail!("sqld MainPID {pid} executable was replaced after start");
    }
    let linked_canonical = linked
        .canonicalize()
        .with_context(|| format!("canonicalizing sqld MainPID {pid} executable"))?;
    if linked_canonical != expected_canonical {
        return Ok(None);
    }

    let mut running = File::open(&proc_path)
        .with_context(|| format!("opening sqld MainPID {pid} executable handle"))?;
    let running_metadata = running
        .metadata()
        .with_context(|| format!("inspecting sqld MainPID {pid} executable handle"))?;
    if !running_metadata.is_file()
        || running_metadata.dev() != expected_metadata.dev()
        || running_metadata.ino() != expected_metadata.ino()
    {
        bail!("sqld MainPID {pid} executable inode differs from the managed payload path");
    }
    let actual = sha256_reader(&mut running, "running sqld payload")?;
    if !expected_sha256.iter().any(|expected| expected == &actual) {
        bail!("running sqld payload bytes differ from every pinned SHA-256");
    }
    Ok(Some(ProcessExecutableIdentity {
        device: running_metadata.dev(),
        inode: running_metadata.ino(),
    }))
}

fn sha256_reader(file: &mut File, label: &str) -> anyhow::Result<String> {
    let mut hasher = Sha256::new();
    let mut buffer = [0_u8; 64 * 1024];
    loop {
        let count = file
            .read(&mut buffer)
            .with_context(|| format!("hashing {label} bytes"))?;
        if count == 0 {
            break;
        }
        hasher.update(&buffer[..count]);
    }
    Ok(format!("{:x}", hasher.finalize()))
}

fn validate_sha256(value: &str, label: &str) -> anyhow::Result<()> {
    if value.len() != 64
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        bail!("{label} is not a lowercase 64-character SHA-256");
    }
    Ok(())
}

fn parse_octal_mode(value: &str) -> anyhow::Result<u32> {
    if value.len() != 4
        || !value.starts_with('0')
        || !value.bytes().all(|byte| (b'0'..=b'7').contains(&byte))
    {
        bail!("expected mode must be four octal digits such as 0755");
    }
    u32::from_str_radix(value, 8).context("parsing expected file mode")
}

fn read_plain_digest(path: &Path, label: &str) -> anyhow::Result<String> {
    let mut file = open_safe_0600(path, label)?;
    let mut record = String::new();
    file.read_to_string(&mut record)
        .with_context(|| format!("reading {label} file {}", path.display()))?;
    let digest = record
        .strip_suffix('\n')
        .ok_or_else(|| anyhow::anyhow!("{label} record is not newline terminated"))?;
    if digest.contains('\n') {
        bail!("{label} record must contain exactly one line");
    }
    validate_sha256(digest, label)?;
    Ok(digest.to_owned())
}

fn read_path_digest(path: &Path) -> anyhow::Result<(String, String)> {
    let mut file = open_safe_0600(path, "sqld helper-digest")?;
    let mut record = String::new();
    file.read_to_string(&mut record)
        .with_context(|| format!("reading sqld helper-digest file {}", path.display()))?;
    let record = record
        .strip_suffix('\n')
        .ok_or_else(|| anyhow::anyhow!("sqld helper-digest record is not newline terminated"))?;
    if record.contains('\n') {
        bail!("sqld helper-digest record must contain exactly one line");
    }
    let (digest, recorded_path) = record
        .split_once("  ")
        .ok_or_else(|| anyhow::anyhow!("sqld helper-digest record has an invalid shape"))?;
    validate_sha256(digest, "sqld helper-digest SHA-256")?;
    if recorded_path.is_empty() {
        bail!("sqld helper-digest record has an empty path");
    }
    Ok((digest.to_owned(), recorded_path.to_owned()))
}

fn open_safe_owned_regular(
    path: &Path,
    label: &str,
    expected_mode: Option<u32>,
) -> anyhow::Result<(File, PathBuf)> {
    let before = fs::symlink_metadata(path)
        .with_context(|| format!("inspecting {label} {}", path.display()))?;
    if before.file_type().is_symlink() || !before.is_file() {
        bail!(
            "{label} must be a non-symlink regular file: {}",
            path.display()
        );
    }
    let file = File::open(path).with_context(|| format!("opening {label} {}", path.display()))?;
    let handle = file
        .metadata()
        .with_context(|| format!("inspecting open {label} {}", path.display()))?;
    let after = fs::symlink_metadata(path)
        .with_context(|| format!("rechecking {label} {}", path.display()))?;
    let current_uid = current_uid()?;
    let stable = before.dev() == handle.dev()
        && before.ino() == handle.ino()
        && after.dev() == handle.dev()
        && after.ino() == handle.ino();
    if !stable
        || after.file_type().is_symlink()
        || !handle.is_file()
        || !after.is_file()
        || handle.uid() != current_uid
        || after.uid() != current_uid
    {
        bail!(
            "{label} must be a stable current-user-owned non-symlink regular file: {}",
            path.display()
        );
    }
    if expected_mode.is_some_and(|mode| {
        handle.permissions().mode() & 0o7777 != mode || after.permissions().mode() & 0o7777 != mode
    }) {
        bail!("{label} has the wrong managed mode: {}", path.display());
    }
    let canonical = path
        .canonicalize()
        .with_context(|| format!("canonicalizing {label} {}", path.display()))?;
    if canonical != path {
        bail!(
            "{label} path has a symlinked or non-canonical ancestor: {}",
            path.display()
        );
    }
    let final_metadata = fs::metadata(&canonical)
        .with_context(|| format!("rechecking canonical {label} {}", canonical.display()))?;
    if final_metadata.dev() != handle.dev() || final_metadata.ino() != handle.ino() {
        bail!("{label} path changed during validation: {}", path.display());
    }
    Ok((file, canonical))
}

fn compute_source_digest(
    source_root: &Path,
    toolchain_files: &[PathBuf],
    toolchain_roots: &[PathBuf],
    crate_archives: &[PathBuf],
) -> anyhow::Result<String> {
    let root_metadata = fs::symlink_metadata(source_root)
        .with_context(|| format!("inspecting envctl source root {}", source_root.display()))?;
    if root_metadata.file_type().is_symlink()
        || !root_metadata.is_dir()
        || root_metadata.uid() != current_uid()?
    {
        bail!(
            "envctl source root must be a current-user-owned real directory: {}",
            source_root.display()
        );
    }
    let root = source_root
        .canonicalize()
        .context("canonicalizing the envctl source root")?;
    if root != source_root {
        bail!("envctl source root has a symlinked or non-canonical ancestor");
    }
    let mut files = Vec::new();
    for relative in [
        "Cargo.toml",
        "Cargo.lock",
        "rust-toolchain.toml",
        "assets/scripts/envctl-sqld-hermetic-cargo.sh",
        "crates/secretctl",
        "crates/secrets-engine",
        "crates/secrets-proto",
    ] {
        collect_source_files(&root, &root.join(relative), &mut files)?;
    }
    for relative in [".cargo/config.toml", ".cargo/config"] {
        let cargo_config = root.join(relative);
        if cargo_config.exists() || fs::symlink_metadata(&cargo_config).is_ok() {
            collect_source_files(&root, &cargo_config, &mut files)?;
        }
    }
    files.sort_by(|left, right| {
        left.as_os_str()
            .as_bytes()
            .cmp(right.as_os_str().as_bytes())
    });
    files.dedup();
    if files.is_empty() {
        bail!("envctl sqld-helper source set is empty");
    }

    let mut hasher = Sha256::new();
    hasher.update(b"envctl-sqld-source-v1\0");
    for relative in files {
        let absolute = root.join(&relative);
        let (mut file, _) = open_safe_owned_regular(&absolute, "sqld helper source file", None)?;
        let size = file
            .metadata()
            .context("inspecting open sqld helper source file")?
            .len();
        let path_bytes = relative.as_os_str().as_bytes();
        hasher.update((path_bytes.len() as u64).to_le_bytes());
        hasher.update(path_bytes);
        hasher.update(size.to_le_bytes());
        let mut buffer = [0_u8; 64 * 1024];
        loop {
            let count = file
                .read(&mut buffer)
                .context("hashing sqld helper source file")?;
            if count == 0 {
                break;
            }
            hasher.update(&buffer[..count]);
        }
    }
    let mut toolchain_inputs = toolchain_files.to_vec();
    toolchain_inputs.sort_by(|left, right| {
        left.file_name()
            .map(OsStr::as_bytes)
            .cmp(&right.file_name().map(OsStr::as_bytes))
    });
    for pair in toolchain_inputs.windows(2) {
        if pair[0].file_name() == pair[1].file_name() {
            bail!(
                "sqld helper toolchain inputs contain a duplicate leaf name: {}",
                pair[0].display()
            );
        }
    }
    hasher.update(b"envctl-sqld-toolchain-v1\0");
    for toolchain_path in toolchain_inputs {
        let leaf = toolchain_path.file_name().ok_or_else(|| {
            anyhow!(
                "sqld helper toolchain input has no file name: {}",
                toolchain_path.display()
            )
        })?;
        let (mut file, _) =
            open_safe_owned_regular(&toolchain_path, "sqld helper toolchain input", Some(0o755))?;
        let size = file
            .metadata()
            .context("inspecting open sqld helper toolchain input")?
            .len();
        let leaf_bytes = leaf.as_bytes();
        hasher.update((leaf_bytes.len() as u64).to_le_bytes());
        hasher.update(leaf_bytes);
        hasher.update(size.to_le_bytes());
        let mut buffer = [0_u8; 64 * 1024];
        loop {
            let count = file
                .read(&mut buffer)
                .context("hashing sqld helper toolchain input")?;
            if count == 0 {
                break;
            }
            hasher.update(&buffer[..count]);
        }
    }
    let mut roots = toolchain_roots.to_vec();
    roots.sort_by(|left, right| {
        left.as_os_str()
            .as_bytes()
            .cmp(right.as_os_str().as_bytes())
    });
    roots.dedup();
    if roots.is_empty() {
        bail!("sqld helper toolchain-root set is empty");
    }
    for pair in roots.windows(2) {
        if pair[0].file_name() == pair[1].file_name() {
            bail!(
                "sqld helper toolchain roots contain a duplicate leaf name: {}",
                pair[0].display()
            );
        }
    }
    hasher.update(b"envctl-sqld-toolchain-roots-v2\0");
    for root_path in roots {
        hash_owned_toolchain_root(&mut hasher, &root_path)?;
    }
    let mut registry_inputs = crate_archives.to_vec();
    registry_inputs.sort_by(|left, right| {
        left.file_name()
            .map(OsStr::as_bytes)
            .cmp(&right.file_name().map(OsStr::as_bytes))
    });
    if registry_inputs.is_empty() {
        bail!("sqld helper registry input set is empty");
    }
    for pair in registry_inputs.windows(2) {
        if pair[0].file_name() == pair[1].file_name() {
            bail!(
                "sqld helper registry inputs contain a duplicate leaf name: {}",
                pair[0].display()
            );
        }
    }
    hasher.update(b"envctl-sqld-registry-v1\0");
    for archive_path in registry_inputs {
        let leaf = archive_path.file_name().ok_or_else(|| {
            anyhow!(
                "sqld helper crate archive has no file name: {}",
                archive_path.display()
            )
        })?;
        if archive_path.extension() != Some(OsStr::new("crate")) {
            bail!(
                "sqld helper registry input is not a .crate archive: {}",
                archive_path.display()
            );
        }
        let (mut file, _) =
            open_safe_owned_regular(&archive_path, "sqld helper crate archive", Some(0o444))?;
        let size = file
            .metadata()
            .context("inspecting open sqld helper crate archive")?
            .len();
        let leaf_bytes = leaf.as_bytes();
        hasher.update((leaf_bytes.len() as u64).to_le_bytes());
        hasher.update(leaf_bytes);
        hasher.update(size.to_le_bytes());
        let mut buffer = [0_u8; 64 * 1024];
        loop {
            let count = file
                .read(&mut buffer)
                .context("hashing sqld helper crate archive")?;
            if count == 0 {
                break;
            }
            hasher.update(&buffer[..count]);
        }
    }
    Ok(format!("{:x}", hasher.finalize()))
}

fn hash_owned_toolchain_root(hasher: &mut Sha256, lexical_root: &Path) -> anyhow::Result<()> {
    let metadata = fs::symlink_metadata(lexical_root).with_context(|| {
        format!(
            "inspecting sqld helper toolchain root {}",
            lexical_root.display()
        )
    })?;
    if metadata.file_type().is_symlink() || !metadata.is_dir() || metadata.uid() != current_uid()? {
        bail!(
            "sqld helper toolchain root must be a current-user-owned real directory: {}",
            lexical_root.display()
        );
    }
    let root = lexical_root.canonicalize().with_context(|| {
        format!(
            "canonicalizing sqld helper toolchain root {}",
            lexical_root.display()
        )
    })?;
    if root != lexical_root {
        bail!("sqld helper toolchain root has a symlinked or non-canonical ancestor");
    }
    let mut nodes = Vec::new();
    collect_toolchain_root_nodes(&root, &root, &mut nodes)?;
    nodes.sort_by(|left, right| {
        left.as_os_str()
            .as_bytes()
            .cmp(right.as_os_str().as_bytes())
    });
    if nodes.is_empty() {
        bail!("sqld helper toolchain root is empty: {}", root.display());
    }
    let root_leaf = root.file_name().ok_or_else(|| {
        anyhow!(
            "sqld helper toolchain root has no leaf name: {}",
            root.display()
        )
    })?;
    let root_leaf_bytes = root_leaf.as_bytes();
    hasher.update((root_leaf_bytes.len() as u64).to_le_bytes());
    hasher.update(root_leaf_bytes);
    hasher.update((metadata.permissions().mode() & 0o7777).to_le_bytes());
    for relative in nodes {
        let absolute = root.join(&relative);
        let node_metadata = fs::symlink_metadata(&absolute).with_context(|| {
            format!(
                "rechecking sqld helper toolchain-root node {}",
                absolute.display()
            )
        })?;
        if node_metadata.uid() != current_uid()? {
            bail!(
                "sqld helper toolchain-root node is not current-user-owned: {}",
                absolute.display()
            );
        }
        let path_bytes = relative.as_os_str().as_bytes();
        hasher.update((path_bytes.len() as u64).to_le_bytes());
        hasher.update(path_bytes);
        hasher.update((node_metadata.permissions().mode() & 0o7777).to_le_bytes());
        if node_metadata.file_type().is_symlink() {
            let target = validate_owned_contained_toolchain_symlink(&root, &absolute)?;
            let target_bytes = target.as_os_str().as_bytes();
            hasher.update(b"l");
            hasher.update((target_bytes.len() as u64).to_le_bytes());
            hasher.update(target_bytes);
        } else if node_metadata.is_dir() {
            hasher.update(b"d");
        } else if node_metadata.is_file() {
            let (mut file, _) =
                open_safe_owned_regular(&absolute, "sqld helper toolchain-root file", None)?;
            let open_metadata = file
                .metadata()
                .context("inspecting open sqld helper toolchain-root file")?;
            hasher.update(b"f");
            hasher.update(open_metadata.len().to_le_bytes());
            let mut buffer = [0_u8; 64 * 1024];
            loop {
                let count = file
                    .read(&mut buffer)
                    .context("hashing sqld helper toolchain-root file")?;
                if count == 0 {
                    break;
                }
                hasher.update(&buffer[..count]);
            }
        } else {
            bail!(
                "sqld helper toolchain-root node is not a file, directory, or symlink: {}",
                absolute.display()
            );
        }
    }
    Ok(())
}

fn collect_toolchain_root_nodes(
    root: &Path,
    path: &Path,
    nodes: &mut Vec<PathBuf>,
) -> anyhow::Result<()> {
    let metadata = fs::symlink_metadata(path).with_context(|| {
        format!(
            "inspecting sqld helper toolchain-root path {}",
            path.display()
        )
    })?;
    if metadata.uid() != current_uid()? {
        bail!(
            "sqld helper toolchain-root path must be current-user-owned: {}",
            path.display()
        );
    }
    if path != root {
        nodes.push(
            path.strip_prefix(root)
                .context("sqld helper toolchain-root node escaped its root")?
                .to_path_buf(),
        );
    }
    if metadata.file_type().is_symlink() {
        validate_owned_contained_toolchain_symlink(root, path)?;
        return Ok(());
    }
    if metadata.is_dir() {
        for entry in fs::read_dir(path).with_context(|| {
            format!(
                "reading sqld helper toolchain-root directory {}",
                path.display()
            )
        })? {
            collect_toolchain_root_nodes(root, &entry?.path(), nodes)?;
        }
        return Ok(());
    }
    if !metadata.is_file() {
        bail!(
            "sqld helper toolchain-root path is not regular: {}",
            path.display()
        );
    }
    Ok(())
}

fn validate_owned_contained_toolchain_symlink(root: &Path, path: &Path) -> anyhow::Result<PathBuf> {
    let before = fs::symlink_metadata(path).with_context(|| {
        format!(
            "inspecting sqld helper toolchain-root symlink {}",
            path.display()
        )
    })?;
    if !before.file_type().is_symlink() || before.uid() != current_uid()? {
        bail!(
            "sqld helper toolchain-root symlink is not current-user-owned: {}",
            path.display()
        );
    }
    let target = fs::read_link(path).with_context(|| {
        format!(
            "reading sqld helper toolchain-root symlink {}",
            path.display()
        )
    })?;
    if target.as_os_str().is_empty() || target.is_absolute() {
        bail!(
            "sqld helper toolchain-root symlink target must be relative: {}",
            path.display()
        );
    }
    let resolved = path.canonicalize().with_context(|| {
        format!(
            "resolving non-dangling sqld helper toolchain-root symlink {}",
            path.display()
        )
    })?;
    if resolved != root && !resolved.starts_with(root) {
        bail!(
            "sqld helper toolchain-root symlink escapes its root: {}",
            path.display()
        );
    }
    let resolved_metadata = fs::metadata(&resolved).with_context(|| {
        format!(
            "inspecting sqld helper toolchain-root symlink target {}",
            resolved.display()
        )
    })?;
    if !resolved_metadata.is_file() && !resolved_metadata.is_dir() {
        bail!(
            "sqld helper toolchain-root symlink targets a special node: {}",
            path.display()
        );
    }
    let after = fs::symlink_metadata(path).with_context(|| {
        format!(
            "rechecking sqld helper toolchain-root symlink {}",
            path.display()
        )
    })?;
    let after_target = fs::read_link(path).with_context(|| {
        format!(
            "re-reading sqld helper toolchain-root symlink {}",
            path.display()
        )
    })?;
    if !after.file_type().is_symlink()
        || after.uid() != before.uid()
        || after.dev() != before.dev()
        || after.ino() != before.ino()
        || after_target != target
    {
        bail!(
            "sqld helper toolchain-root symlink changed during validation: {}",
            path.display()
        );
    }
    Ok(target)
}

fn collect_source_files(root: &Path, path: &Path, files: &mut Vec<PathBuf>) -> anyhow::Result<()> {
    let metadata = fs::symlink_metadata(path)
        .with_context(|| format!("inspecting sqld helper source path {}", path.display()))?;
    if metadata.file_type().is_symlink() || metadata.uid() != current_uid()? {
        bail!(
            "sqld helper source path must be current-user-owned and non-symlink: {}",
            path.display()
        );
    }
    if metadata.is_dir() {
        for entry in fs::read_dir(path)
            .with_context(|| format!("reading sqld helper source directory {}", path.display()))?
        {
            collect_source_files(root, &entry?.path(), files)?;
        }
        return Ok(());
    }
    if !metadata.is_file() {
        bail!(
            "sqld helper source path is not a regular file: {}",
            path.display()
        );
    }
    let name = path.file_name().and_then(OsStr::to_str).unwrap_or_default();
    let included = matches!(name, "Cargo.toml" | "Cargo.lock" | "rust-toolchain.toml")
        || path
            .extension()
            .and_then(OsStr::to_str)
            .is_some_and(|extension| matches!(extension, "rs" | "toml" | "proto" | "sh"));
    if included {
        files.push(
            path.strip_prefix(root)
                .context("sqld helper source escaped its root")?
                .to_path_buf(),
        );
    }
    Ok(())
}

fn read_safe_token(path: &Path) -> anyhow::Result<Zeroizing<String>> {
    let mut file = open_safe_0600(path, "sqld client-token")?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)
        .with_context(|| format!("reading sqld client-token file {}", path.display()))?;
    let token = Zeroizing::new(contents);
    if token.is_empty()
        || !token
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || b"._~-".contains(&byte))
    {
        bail!("sqld client-token file has an invalid JWT shape");
    }
    Ok(token)
}

fn open_safe_0600(path: &Path, label: &str) -> anyhow::Result<File> {
    let canonical = path
        .canonicalize()
        .with_context(|| format!("canonicalizing {label} file {}", path.display()))?;
    if canonical != path {
        bail!(
            "{label} file has a symlinked or non-canonical ancestor: {}",
            path.display()
        );
    }
    let before = fs::symlink_metadata(path)
        .with_context(|| format!("inspecting {label} file {}", path.display()))?;
    if before.file_type().is_symlink() || !before.is_file() {
        bail!(
            "{label} file must be a current-user-owned 0600 regular file: {}",
            path.display()
        );
    }
    let file =
        File::open(path).with_context(|| format!("opening {label} file {}", path.display()))?;
    let handle = file
        .metadata()
        .with_context(|| format!("inspecting open {label} file {}", path.display()))?;
    let after = fs::symlink_metadata(path)
        .with_context(|| format!("rechecking {label} file {}", path.display()))?;
    let current_uid = current_uid()?;
    let stable_inode = before.dev() == handle.dev()
        && before.ino() == handle.ino()
        && after.dev() == handle.dev()
        && after.ino() == handle.ino();
    if !stable_inode
        || after.file_type().is_symlink()
        || !handle.is_file()
        || !after.is_file()
        || handle.uid() != current_uid
        || after.uid() != current_uid
        || handle.permissions().mode() & 0o7777 != 0o600
        || after.permissions().mode() & 0o7777 != 0o600
    {
        bail!(
            "{label} file must be a stable current-user-owned 0600 regular file: {}",
            path.display()
        );
    }
    Ok(file)
}

fn current_uid() -> anyhow::Result<u32> {
    Ok(fs::metadata("/proc/self")
        .context("resolving current process ownership")?
        .uid())
}

fn process_start_time(pid: u32) -> anyhow::Result<String> {
    let stat = fs::read_to_string(format!("/proc/{pid}/stat"))
        .with_context(|| format!("reading /proc/{pid}/stat"))?;
    let after_comm = stat
        .rfind(") ")
        .map(|index| &stat[index + 2..])
        .ok_or_else(|| anyhow::anyhow!("invalid /proc/{pid}/stat format"))?;
    // The suffix starts at field 3 (`state`); starttime is field 22, hence index 19.
    after_comm
        .split_whitespace()
        .nth(19)
        .map(str::to_owned)
        .ok_or_else(|| anyhow::anyhow!("/proc/{pid}/stat has no starttime field"))
}

fn pid_owns_loopback_listener(pid: u32, port: u16) -> anyhow::Result<bool> {
    let mut socket_inodes = HashSet::new();
    for entry in fs::read_dir(format!("/proc/{pid}/fd"))
        .with_context(|| format!("reading sqld MainPID {pid} file descriptors"))?
    {
        let Ok(entry) = entry else { continue };
        let Ok(target) = fs::read_link(entry.path()) else {
            continue;
        };
        let target = target.to_string_lossy();
        if let Some(inode) = target
            .strip_prefix("socket:[")
            .and_then(|value| value.strip_suffix(']'))
            .and_then(|value| value.parse::<u64>().ok())
        {
            socket_inodes.insert(inode);
        }
    }
    if socket_inodes.is_empty() {
        return Ok(false);
    }

    let tcp = fs::read_to_string(format!("/proc/{pid}/net/tcp"))
        .with_context(|| format!("reading sqld MainPID {pid} TCP table"))?;
    let expected_local = format!("0100007F:{port:04X}");
    for line in tcp.lines().skip(1) {
        let fields: Vec<_> = line.split_whitespace().collect();
        if fields.len() > 9
            && fields[1] == expected_local
            && fields[3] == "0A"
            && fields[9]
                .parse::<u64>()
                .is_ok_and(|inode| socket_inodes.contains(&inode))
        {
            return Ok(true);
        }
    }
    Ok(false)
}

enum AuthAttemptError {
    /// The owned socket exists but did not yet complete an HTTP exchange; retry until the bound.
    NotReady(String),
    /// A complete response violated the fail-closed auth/protocol contract; retrying cannot repair
    /// an open-auth listener or mismatched credential, so fail the service start immediately.
    Refused(anyhow::Error),
}

fn authenticate_once(address: SocketAddrV4, token: &str) -> Result<(), AuthAttemptError> {
    let unauth = http_pipeline(address, None)
        .map_err(|error| AuthAttemptError::NotReady(error.to_string()))?;
    if unauth.status != 401 {
        return Err(AuthAttemptError::Refused(anyhow::anyhow!(
            "sqld auth is not enforced: unauthenticated SQL returned HTTP {}",
            unauth.status
        )));
    }

    let authenticated = http_pipeline(address, Some(token))
        .map_err(|error| AuthAttemptError::NotReady(error.to_string()))?;
    if !(200..300).contains(&authenticated.status) {
        return Err(AuthAttemptError::Refused(anyhow::anyhow!(
            "sqld rejected the managed client JWT with HTTP {}",
            authenticated.status
        )));
    }
    let value: Value = serde_json::from_slice(&authenticated.body).map_err(|error| {
        AuthAttemptError::Refused(anyhow::anyhow!(
            "sqld authenticated SQL response was not JSON: {error}"
        ))
    })?;
    if !contains_integer_one(&value) {
        return Err(AuthAttemptError::Refused(anyhow::anyhow!(
            "sqld authenticated SELECT 1 returned an unexpected response"
        )));
    }
    Ok(())
}

struct HttpResponse {
    status: u16,
    body: Vec<u8>,
}

fn http_pipeline(address: SocketAddrV4, token: Option<&str>) -> anyhow::Result<HttpResponse> {
    const BODY: &str = r#"{"requests":[{"type":"execute","stmt":{"sql":"SELECT 1"}}]}"#;
    const MAX_RESPONSE: u64 = 1024 * 1024;

    let mut stream = TcpStream::connect_timeout(&address.into(), Duration::from_secs(1))
        .context("connecting to the owned sqld loopback listener")?;
    stream
        .set_read_timeout(Some(Duration::from_secs(2)))
        .context("setting sqld probe read timeout")?;
    stream
        .set_write_timeout(Some(Duration::from_secs(2)))
        .context("setting sqld probe write timeout")?;

    let auth_header = token.map(|_| "Authorization: Bearer ").unwrap_or_default();
    let auth_value = token.unwrap_or_default();
    let request = Zeroizing::new(format!(
        "POST /v2/pipeline HTTP/1.1\r\nHost: 127.0.0.1:{}\r\nConnection: close\r\nContent-Type: application/json\r\nContent-Length: {}\r\n{auth_header}{auth_value}{}\r\n{BODY}",
        address.port(),
        BODY.len(),
        if token.is_some() { "\r\n" } else { "" },
    ));
    stream
        .write_all(request.as_bytes())
        .context("writing the sqld readiness request")?;
    stream
        .flush()
        .context("flushing the sqld readiness request")?;

    let mut raw = Vec::new();
    stream
        .take(MAX_RESPONSE + 1)
        .read_to_end(&mut raw)
        .context("reading the sqld readiness response")?;
    if raw.len() as u64 > MAX_RESPONSE {
        bail!("sqld readiness response exceeded 1 MiB");
    }
    parse_http_response(&raw)
}

fn parse_http_response(raw: &[u8]) -> anyhow::Result<HttpResponse> {
    let split = raw
        .windows(4)
        .position(|window| window == b"\r\n\r\n")
        .ok_or_else(|| anyhow::anyhow!("sqld returned an incomplete HTTP response"))?;
    let headers =
        std::str::from_utf8(&raw[..split]).context("sqld returned invalid HTTP headers")?;
    let status = headers
        .lines()
        .next()
        .and_then(|line| line.split_whitespace().nth(1))
        .and_then(|value| value.parse::<u16>().ok())
        .ok_or_else(|| anyhow::anyhow!("sqld returned an invalid HTTP status line"))?;
    let encoded_body = &raw[split + 4..];
    let chunked = headers.lines().skip(1).any(|line| {
        let lower = line.to_ascii_lowercase();
        lower.starts_with("transfer-encoding:") && lower.contains("chunked")
    });
    let body = if chunked {
        decode_chunked(encoded_body)?
    } else {
        encoded_body.to_vec()
    };
    Ok(HttpResponse { status, body })
}

fn decode_chunked(mut input: &[u8]) -> anyhow::Result<Vec<u8>> {
    let mut decoded = Vec::new();
    loop {
        let line_end = input
            .windows(2)
            .position(|window| window == b"\r\n")
            .ok_or_else(|| anyhow::anyhow!("invalid chunked sqld response"))?;
        let size_text = std::str::from_utf8(&input[..line_end])
            .context("invalid chunk size from sqld")?
            .split(';')
            .next()
            .unwrap_or_default();
        let size = usize::from_str_radix(size_text, 16).context("invalid chunk size from sqld")?;
        input = &input[line_end + 2..];
        if size == 0 {
            return Ok(decoded);
        }
        if input.len() < size + 2 || &input[size..size + 2] != b"\r\n" {
            bail!("truncated chunked sqld response");
        }
        decoded.extend_from_slice(&input[..size]);
        if decoded.len() > 1024 * 1024 {
            bail!("decoded sqld response exceeded 1 MiB");
        }
        input = &input[size + 2..];
    }
}

fn contains_integer_one(value: &Value) -> bool {
    match value {
        Value::Object(object) => {
            (object.get("type").and_then(Value::as_str) == Some("integer")
                && object.get("value").and_then(Value::as_str) == Some("1"))
                || object.values().any(contains_integer_one)
        }
        Value::Array(values) => values.iter().any(contains_integer_one),
        _ => false,
    }
}

fn refuse_existing(path: &Path, label: &str) -> anyhow::Result<()> {
    match fs::symlink_metadata(path) {
        Ok(_) => bail!(
            "refusing to overwrite existing {label} at {}",
            path.display()
        ),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(()),
        Err(error) => {
            Err(error).with_context(|| format!("inspecting {label} at {}", path.display()))
        }
    }
}

fn destination_parent(path: &Path, label: &str) -> anyhow::Result<PathBuf> {
    path.parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .map(Path::to_path_buf)
        .ok_or_else(|| anyhow::anyhow!("{label} destination has no parent directory"))
}

fn prepare_auth_parent(path: &Path, label: &str) -> anyhow::Result<PreparedParent> {
    let parent = destination_parent(path, label)?;
    match fs::symlink_metadata(&parent) {
        Ok(_) => {}
        Err(error) if error.kind() == io::ErrorKind::NotFound => {
            let mut builder = fs::DirBuilder::new();
            builder.mode(0o700);
            builder.create(&parent).with_context(|| {
                format!("creating the {label} parent directory {}", parent.display())
            })?;
        }
        Err(error) => {
            return Err(error).with_context(|| {
                format!(
                    "inspecting the {label} parent directory {}",
                    parent.display()
                )
            })
        }
    }
    PreparedParent::open(parent, label, &[0o700])
}

fn prepare_record_parent(path: &Path, label: &str) -> anyhow::Result<PreparedParent> {
    let parent = destination_parent(path, label)?;
    PreparedParent::open(parent, label, &[0o700, 0o755])
}

fn validate_real_owned_directory(path: &Path, label: &str, modes: &[u32]) -> anyhow::Result<()> {
    let metadata = fs::symlink_metadata(path)
        .with_context(|| format!("inspecting {label} {}", path.display()))?;
    let canonical = path
        .canonicalize()
        .with_context(|| format!("canonicalizing {label} {}", path.display()))?;
    if canonical != path
        || metadata.file_type().is_symlink()
        || !metadata.is_dir()
        || metadata.uid() != current_uid()?
        || !modes.contains(&(metadata.permissions().mode() & 0o7777))
    {
        bail!(
            "{label} must be a canonical current-user-owned real directory with managed mode: {}",
            path.display()
        );
    }
    Ok(())
}

struct PreparedParent {
    path: PathBuf,
    directory: File,
    device: u64,
    inode: u64,
    mode: u32,
    label: String,
}

impl PreparedParent {
    fn open(path: PathBuf, label: &str, allowed_modes: &[u32]) -> anyhow::Result<Self> {
        let before = fs::symlink_metadata(&path).with_context(|| {
            format!("inspecting the {label} parent directory {}", path.display())
        })?;
        let mode = before.permissions().mode() & 0o7777;
        let canonical = path.canonicalize().with_context(|| {
            format!(
                "canonicalizing the {label} parent directory {}",
                path.display()
            )
        })?;
        if before.file_type().is_symlink()
            || !before.is_dir()
            || before.uid() != current_uid()?
            || !allowed_modes.contains(&mode)
            || canonical != path
        {
            bail!(
                "{label} parent must be a current-user-owned real directory with managed mode: {}",
                path.display()
            );
        }
        let directory = File::open(&path)
            .with_context(|| format!("opening the {label} parent directory {}", path.display()))?;
        let handle = directory.metadata().with_context(|| {
            format!(
                "inspecting the open {label} parent directory {}",
                path.display()
            )
        })?;
        let after = fs::symlink_metadata(&path).with_context(|| {
            format!("rechecking the {label} parent directory {}", path.display())
        })?;
        if !handle.is_dir()
            || after.file_type().is_symlink()
            || !after.is_dir()
            || before.dev() != handle.dev()
            || before.ino() != handle.ino()
            || after.dev() != handle.dev()
            || after.ino() != handle.ino()
            || handle.uid() != current_uid()?
            || after.uid() != current_uid()?
            || handle.permissions().mode() & 0o7777 != mode
            || after.permissions().mode() & 0o7777 != mode
        {
            bail!(
                "{label} parent changed during validation: {}",
                path.display()
            );
        }
        Ok(Self {
            path,
            directory,
            device: handle.dev(),
            inode: handle.ino(),
            mode,
            label: label.to_owned(),
        })
    }

    fn path(&self) -> &Path {
        &self.path
    }

    fn revalidate_and_sync(&self) -> anyhow::Result<()> {
        let current = fs::symlink_metadata(&self.path).with_context(|| {
            format!(
                "rechecking the {} parent directory {}",
                self.label,
                self.path.display()
            )
        })?;
        if current.file_type().is_symlink()
            || !current.is_dir()
            || current.dev() != self.device
            || current.ino() != self.inode
            || current.uid() != current_uid()?
            || current.permissions().mode() & 0o7777 != self.mode
        {
            bail!(
                "{} parent changed before sync: {}",
                self.label,
                self.path.display()
            );
        }
        self.directory.sync_all().with_context(|| {
            format!(
                "syncing {} parent directory {}",
                self.label,
                self.path.display()
            )
        })
    }
}

struct StagedFile {
    path: PathBuf,
    device: u64,
    inode: u64,
}

impl StagedFile {
    fn write(parent: &Path, label: &str, bytes: &[u8]) -> anyhow::Result<Self> {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        for attempt in 0..128_u32 {
            let path = parent.join(format!(
                ".sqld-auth.{}.{}.{}.stage",
                std::process::id(),
                stamp,
                attempt
            ));
            let mut file = match OpenOptions::new()
                .write(true)
                .create_new(true)
                .mode(0o600)
                .open(&path)
            {
                Ok(file) => file,
                Err(error) if error.kind() == io::ErrorKind::AlreadyExists => continue,
                Err(error) => {
                    return Err(error).with_context(|| {
                        format!("creating the {label} staging file in {}", parent.display())
                    })
                }
            };
            if let Err(error) = file.write_all(bytes).and_then(|()| file.sync_all()) {
                let _ = fs::remove_file(&path);
                return Err(error).context("writing and syncing sqld auth material");
            }
            let metadata = file
                .metadata()
                .context("inspecting staged sqld auth material")?;
            return Ok(Self {
                path,
                device: metadata.dev(),
                inode: metadata.ino(),
            });
        }
        bail!("could not allocate a unique sqld auth staging file")
    }

    fn commit_no_replace(&self, destination: &Path) -> anyhow::Result<()> {
        fs::hard_link(&self.path, destination).with_context(|| {
            format!(
                "linking staged sqld auth material to {} (destination may already exist)",
                destination.display()
            )
        })
    }

    fn rollback_link(&self, destination: &Path) {
        let same_file = fs::symlink_metadata(destination)
            .map(|metadata| metadata.dev() == self.device && metadata.ino() == self.inode)
            .unwrap_or(false);
        if same_file {
            let _ = fs::remove_file(destination);
        }
    }

    fn remove_stage(&self) -> anyhow::Result<()> {
        fs::remove_file(&self.path)
            .with_context(|| format!("removing sqld auth staging file {}", self.path.display()))
    }
}

impl Drop for StagedFile {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.path);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::{TcpListener, TcpStream};
    use std::thread;

    fn test_dir(label: &str) -> PathBuf {
        std::env::temp_dir().join(format!(
            "envctl-secretctl-sqld-{label}-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos()
        ))
    }

    #[test]
    fn bootstrap_writes_current_pair_as_0600_and_refuses_overwrite() {
        let dir = test_dir("files");
        let public_key = dir.join("auth-jwt-key.pem");
        let client_token = dir.join("client.jwt");
        bootstrap(&public_key, &client_token).expect("bootstrap");

        for path in [&public_key, &client_token] {
            let metadata = fs::symlink_metadata(path).expect("metadata");
            assert!(metadata.is_file());
            assert!(!metadata.file_type().is_symlink());
            assert_eq!(metadata.permissions().mode() & 0o7777, 0o600);
        }
        assert_eq!(
            fs::symlink_metadata(&dir).unwrap().permissions().mode() & 0o7777,
            0o700
        );

        let public_before = fs::read(&public_key).unwrap();
        let token_before = fs::read(&client_token).unwrap();
        let error = bootstrap(&public_key, &client_token).unwrap_err();
        assert!(error.to_string().contains("refusing to overwrite"));
        assert_eq!(fs::read(&public_key).unwrap(), public_before);
        assert_eq!(fs::read(&client_token).unwrap(), token_before);
        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn partial_existing_pair_is_preserved_fail_closed() {
        let dir = test_dir("partial");
        fs::create_dir_all(&dir).unwrap();
        let public_key = dir.join("auth-jwt-key.pem");
        let client_token = dir.join("client.jwt");
        fs::write(&public_key, b"operator-owned").unwrap();

        assert!(bootstrap(&public_key, &client_token).is_err());
        assert_eq!(fs::read(&public_key).unwrap(), b"operator-owned");
        assert!(!client_token.exists());
        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn bootstrap_secures_only_the_exact_auth_parent_and_preserves_ancestors() {
        let root = test_dir("parent-scope");
        let config = root.join("config");
        fs::create_dir_all(&config).unwrap();
        fs::set_permissions(&root, fs::Permissions::from_mode(0o755)).unwrap();
        fs::set_permissions(&config, fs::Permissions::from_mode(0o755)).unwrap();
        let auth = config.join("sqld");
        bootstrap(&auth.join("key.pem"), &auth.join("client.jwt")).unwrap();

        assert_eq!(
            fs::metadata(&root).unwrap().permissions().mode() & 0o7777,
            0o755
        );
        assert_eq!(
            fs::metadata(&config).unwrap().permissions().mode() & 0o7777,
            0o755
        );
        assert_eq!(
            fs::metadata(&auth).unwrap().permissions().mode() & 0o7777,
            0o700
        );
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn bootstrap_refuses_existing_nonprivate_auth_parent_without_chmod() {
        let dir = test_dir("unsafe-existing-parent");
        fs::create_dir_all(&dir).unwrap();
        fs::set_permissions(&dir, fs::Permissions::from_mode(0o755)).unwrap();
        let error = bootstrap(&dir.join("key.pem"), &dir.join("client.jwt")).unwrap_err();
        assert!(error.to_string().contains("managed mode"));
        assert_eq!(
            fs::metadata(&dir).unwrap().permissions().mode() & 0o7777,
            0o755
        );
        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn helper_digest_preserves_existing_0755_record_parent() {
        let dir = test_dir("record-parent-mode");
        fs::create_dir_all(&dir).unwrap();
        fs::set_permissions(&dir, fs::Permissions::from_mode(0o755)).unwrap();
        write_self_digest(&dir.join("secretctl.sha256"), None).unwrap();
        assert_eq!(
            fs::metadata(&dir).unwrap().permissions().mode() & 0o7777,
            0o755
        );
        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn toolchain_root_digest_accepts_contained_links_and_rejects_unsafe_links() {
        use std::os::unix::fs::symlink;

        let workspace = test_dir("toolchain-root-links");
        let root = workspace.join("llvm");
        fs::create_dir_all(root.join("lib")).unwrap();
        fs::write(root.join("lib/clang-21"), b"clang target bytes\n").unwrap();
        fs::write(root.join("lib/clang-22"), b"alternate target bytes\n").unwrap();
        let versioned_link = root.join("lib/libclang.so.1");
        symlink("clang-21", &versioned_link).unwrap();
        let link = root.join("lib/libclang.so");
        symlink("libclang.so.1", &link).unwrap();

        let mut first = Sha256::new();
        hash_owned_toolchain_root(&mut first, &root).unwrap();
        let first = format!("{:x}", first.finalize());

        // Both spellings resolve to the same contained bytes. The digest must still bind the raw
        // official-style symlink topology, not merely its final canonical target.
        fs::remove_file(&link).unwrap();
        symlink("clang-21", &link).unwrap();
        let mut changed = Sha256::new();
        hash_owned_toolchain_root(&mut changed, &root).unwrap();
        let changed = format!("{:x}", changed.finalize());
        assert_ne!(first, changed, "raw symlink target must be identity-bound");

        fs::write(workspace.join("outside"), b"outside bytes\n").unwrap();
        fs::remove_file(&link).unwrap();
        symlink("../../outside", &link).unwrap();
        let escaping_error = hash_owned_toolchain_root(&mut Sha256::new(), &root).unwrap_err();
        assert!(escaping_error.to_string().contains("escapes its root"));

        fs::remove_file(&link).unwrap();
        symlink(root.join("lib/clang-21"), &link).unwrap();
        let absolute_error = hash_owned_toolchain_root(&mut Sha256::new(), &root).unwrap_err();
        assert!(absolute_error.to_string().contains("must be relative"));

        fs::remove_file(&link).unwrap();
        symlink("missing", &link).unwrap();
        let dangling_error = hash_owned_toolchain_root(&mut Sha256::new(), &root).unwrap_err();
        assert!(dangling_error
            .to_string()
            .contains("resolving non-dangling"));
        fs::remove_dir_all(workspace).unwrap();
    }

    #[test]
    fn pinned_file_verifier_rejects_symlink_special_mode_and_never_executes() {
        use std::os::unix::fs::symlink;

        let dir = test_dir("pinned-file-shape");
        fs::create_dir_all(&dir).unwrap();
        let marker = dir.join("executed");
        let payload = dir.join("sqld");
        fs::write(
            &payload,
            format!("#!/bin/sh\ntouch '{}'\n", marker.display()),
        )
        .unwrap();
        fs::set_permissions(&payload, fs::Permissions::from_mode(0o755)).unwrap();
        let mut file = File::open(&payload).unwrap();
        let digest = sha256_reader(&mut file, "fixture payload").unwrap();

        verify_sha256(&payload, &digest, "0755").unwrap();
        assert!(!marker.exists());
        fs::set_permissions(&payload, fs::Permissions::from_mode(0o4755)).unwrap();
        assert!(verify_sha256(&payload, &digest, "0755")
            .unwrap_err()
            .to_string()
            .contains("wrong managed mode"));
        fs::set_permissions(&payload, fs::Permissions::from_mode(0o755)).unwrap();
        let link = dir.join("sqld-link");
        symlink(&payload, &link).unwrap();
        assert!(verify_sha256(&link, &digest, "0755")
            .unwrap_err()
            .to_string()
            .contains("non-symlink"));
        assert!(!marker.exists());
        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn source_digest_detects_checked_in_input_drift() {
        let root = test_dir("source-digest");
        for directory in [
            ".cargo",
            "assets/scripts",
            "crates/secretctl/src",
            "crates/secrets-engine/src",
            "crates/secrets-proto/proto",
            "records",
        ] {
            fs::create_dir_all(root.join(directory)).unwrap();
        }
        fs::set_permissions(root.join("records"), fs::Permissions::from_mode(0o700)).unwrap();
        for file in ["Cargo.toml", "Cargo.lock", "rust-toolchain.toml"] {
            fs::write(root.join(file), format!("{file}\n")).unwrap();
        }
        fs::write(root.join(".cargo/config.toml"), "[net]\noffline = true\n").unwrap();
        fs::write(
            root.join("assets/scripts/envctl-sqld-hermetic-cargo.sh"),
            "# pinned build helper\n",
        )
        .unwrap();
        fs::write(root.join("crates/secretctl/Cargo.toml"), "[package]\n").unwrap();
        fs::write(root.join("crates/secretctl/src/main.rs"), "fn main() {}\n").unwrap();
        fs::write(root.join("crates/secrets-engine/Cargo.toml"), "[package]\n").unwrap();
        fs::write(
            root.join("crates/secrets-engine/src/lib.rs"),
            "pub fn ok() {}\n",
        )
        .unwrap();
        fs::write(root.join("crates/secrets-proto/Cargo.toml"), "[package]\n").unwrap();
        fs::write(
            root.join("crates/secrets-proto/proto/vault.proto"),
            "syntax = \"proto3\";\n",
        )
        .unwrap();
        let toolchain_dir = root.join("toolchain");
        fs::create_dir(&toolchain_dir).unwrap();
        let clang = toolchain_dir.join("clang");
        fs::write(&clang, b"pinned clang bytes\n").unwrap();
        fs::set_permissions(&clang, fs::Permissions::from_mode(0o755)).unwrap();
        let toolchain_files = vec![clang];
        let resource_root = toolchain_dir.join("21");
        fs::create_dir(&resource_root).unwrap();
        fs::write(
            resource_root.join("stddef.h"),
            b"pinned clang resource bytes\n",
        )
        .unwrap();
        let toolchain_roots = vec![resource_root];
        let archive = toolchain_dir.join("sha2-0.10.9.crate");
        fs::write(&archive, b"Cargo.lock-checked crate archive bytes\n").unwrap();
        fs::set_permissions(&archive, fs::Permissions::from_mode(0o444)).unwrap();
        let crate_archives = vec![archive];
        let record = root.join("records/source.sha256");
        write_source_digest(
            &root,
            &toolchain_files,
            &toolchain_roots,
            &crate_archives,
            &record,
        )
        .unwrap();
        verify_source_digest(
            &root,
            &toolchain_files,
            &toolchain_roots,
            &crate_archives,
            &record,
        )
        .unwrap();
        fs::write(
            root.join(".cargo/config.toml"),
            "[build]\nrustflags = ['-Cpanic=abort']\n",
        )
        .unwrap();
        assert!(verify_source_digest(
            &root,
            &toolchain_files,
            &toolchain_roots,
            &crate_archives,
            &record
        )
        .unwrap_err()
        .to_string()
        .contains("stale Rust/build inputs"));
        fs::write(root.join(".cargo/config.toml"), "[net]\noffline = true\n").unwrap();
        fs::write(
            toolchain_roots[0].join("stddef.h"),
            b"mutated compiler resource bytes\n",
        )
        .unwrap();
        assert!(verify_source_digest(
            &root,
            &toolchain_files,
            &toolchain_roots,
            &crate_archives,
            &record
        )
        .unwrap_err()
        .to_string()
        .contains("stale Rust/build inputs"));
        fs::write(
            toolchain_roots[0].join("stddef.h"),
            b"pinned clang resource bytes\n",
        )
        .unwrap();
        fs::write(&toolchain_files[0], b"different clang bytes\n").unwrap();
        fs::set_permissions(&toolchain_files[0], fs::Permissions::from_mode(0o755)).unwrap();
        assert!(verify_source_digest(
            &root,
            &toolchain_files,
            &toolchain_roots,
            &crate_archives,
            &record
        )
        .unwrap_err()
        .to_string()
        .contains("stale Rust/build inputs"));
        fs::set_permissions(&crate_archives[0], fs::Permissions::from_mode(0o644)).unwrap();
        fs::write(&crate_archives[0], b"different crate archive bytes\n").unwrap();
        fs::set_permissions(&crate_archives[0], fs::Permissions::from_mode(0o444)).unwrap();
        assert!(verify_source_digest(
            &root,
            &toolchain_files,
            &toolchain_roots,
            &crate_archives,
            &record
        )
        .unwrap_err()
        .to_string()
        .contains("stale Rust/build inputs"));
        fs::remove_dir_all(root).unwrap();
    }

    fn read_request(stream: &mut TcpStream) -> String {
        stream
            .set_read_timeout(Some(Duration::from_secs(2)))
            .unwrap();
        let mut request = Vec::new();
        let mut buf = [0_u8; 2048];
        loop {
            let count = stream.read(&mut buf).unwrap();
            if count == 0 {
                break;
            }
            request.extend_from_slice(&buf[..count]);
            let Some(header_end) = request.windows(4).position(|window| window == b"\r\n\r\n")
            else {
                continue;
            };
            let headers = String::from_utf8_lossy(&request[..header_end]);
            let content_length = headers
                .lines()
                .find_map(|line| {
                    line.to_ascii_lowercase()
                        .strip_prefix("content-length:")
                        .and_then(|value| value.trim().parse::<usize>().ok())
                })
                .unwrap_or(0);
            if request.len() >= header_end + 4 + content_length {
                break;
            }
        }
        String::from_utf8(request).unwrap()
    }

    fn write_response(stream: &mut TcpStream, status: u16, body: &str) {
        let reason = if status == 401 { "Unauthorized" } else { "OK" };
        write!(
            stream,
            "HTTP/1.1 {status} {reason}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
            body.len()
        )
        .unwrap();
        stream.flush().unwrap();
    }

    fn spawn_sqld_fixture(
        open_auth: bool,
        token: &'static str,
    ) -> (u16, std::sync::mpsc::Sender<()>, thread::JoinHandle<()>) {
        let listener = TcpListener::bind((Ipv4Addr::LOCALHOST, 0)).unwrap();
        let port = listener.local_addr().unwrap().port();
        let request_count = if open_auth { 1 } else { 2 };
        let (release, hold) = std::sync::mpsc::channel();
        let handle = thread::spawn(move || {
            for _ in 0..request_count {
                let (mut stream, _) = listener.accept().unwrap();
                let request = read_request(&mut stream);
                assert!(request.contains(r#"{"requests":[{"type":"execute""#));
                let authenticated = request.contains("Authorization: Bearer ");
                if !authenticated && !open_auth {
                    write_response(&mut stream, 401, r#"{"error":"unauthorized"}"#);
                    continue;
                }
                if authenticated {
                    assert!(request.contains(&format!("Authorization: Bearer {token}\r\n")));
                }
                write_response(
                    &mut stream,
                    200,
                    r#"{"results":[{"response":{"result":{"rows":[[{"type":"integer","value":"1"}]]}}}]}"#,
                );
            }
            // Keep the listening FD owned through the probe's post-response identity re-check.
            hold.recv_timeout(Duration::from_secs(10)).unwrap();
        });
        (port, release, handle)
    }

    fn readiness_token(label: &str, value: &str) -> (PathBuf, PathBuf) {
        let dir = test_dir(label);
        fs::create_dir_all(&dir).unwrap();
        let path = dir.join("client.jwt");
        fs::write(&path, value).unwrap();
        fs::set_permissions(&path, fs::Permissions::from_mode(0o600)).unwrap();
        (dir, path)
    }

    fn readiness_digest(dir: &Path) -> PathBuf {
        fs::set_permissions(dir, fs::Permissions::from_mode(0o700)).unwrap();
        let path = dir.join("secretctl.sha256");
        write_self_digest(&path, None).unwrap();
        path
    }

    fn current_executable_sha256() -> String {
        let mut file = File::open(std::env::current_exe().unwrap()).unwrap();
        sha256_reader(&mut file, "test executable").unwrap()
    }

    fn current_executable_mode() -> u32 {
        fs::metadata(std::env::current_exe().unwrap())
            .unwrap()
            .permissions()
            .mode()
            & 0o7777
    }

    #[test]
    fn readiness_probe_binds_auth_proof_to_the_expected_pid_listener() {
        const TOKEN: &str = "fixture.header.payload.signature";
        let (dir, token_path) = readiness_token("ready", TOKEN);
        let digest_path = readiness_digest(&dir);
        let (port, release, server) = spawn_sqld_fixture(false, TOKEN);
        let pid = std::process::id();
        assert!(pid_owns_loopback_listener(pid, port).unwrap());

        readiness_probe(ReadinessProbeRequest {
            pid,
            expected_executable: &std::env::current_exe().unwrap(),
            expected_sha256: &[current_executable_sha256()],
            expected_mode: current_executable_mode(),
            port,
            client_token: &token_path,
            helper_digest: &digest_path,
            timeout: Duration::from_secs(2),
        })
        .expect("owned authenticated listener should become ready");

        release.send(()).unwrap();
        server.join().unwrap();
        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn readiness_probe_rejects_open_auth_even_when_bearers_would_work() {
        const TOKEN: &str = "fixture.header.payload.signature";
        let (dir, token_path) = readiness_token("open-auth", TOKEN);
        let digest_path = readiness_digest(&dir);
        let (port, release, server) = spawn_sqld_fixture(true, TOKEN);

        let error = readiness_probe(ReadinessProbeRequest {
            pid: std::process::id(),
            expected_executable: &std::env::current_exe().unwrap(),
            expected_sha256: &[current_executable_sha256()],
            expected_mode: current_executable_mode(),
            port,
            client_token: &token_path,
            helper_digest: &digest_path,
            timeout: Duration::from_secs(2),
        })
        .unwrap_err();
        assert!(error.to_string().contains("auth is not enforced"));

        release.send(()).unwrap();
        server.join().unwrap();
        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn readiness_probe_refuses_group_readable_token_before_network_use() {
        let (dir, token_path) = readiness_token("unsafe-token", "fixture.jwt.token");
        let digest_path = readiness_digest(&dir);
        fs::set_permissions(&token_path, fs::Permissions::from_mode(0o640)).unwrap();
        let error = readiness_probe(ReadinessProbeRequest {
            pid: std::process::id(),
            expected_executable: &std::env::current_exe().unwrap(),
            expected_sha256: &[current_executable_sha256()],
            expected_mode: current_executable_mode(),
            port: 9,
            client_token: &token_path,
            helper_digest: &digest_path,
            timeout: Duration::from_millis(1),
        })
        .unwrap_err();
        assert!(error.to_string().contains("current-user-owned 0600"));
        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn readiness_probe_rejects_digest_mismatch_before_token_or_network() {
        let dir = test_dir("digest-first");
        fs::create_dir_all(&dir).unwrap();
        let digest_path = readiness_digest(&dir);
        let running = std::env::current_exe().unwrap().canonicalize().unwrap();
        fs::write(
            &digest_path,
            format!("{}  {}\n", "0".repeat(64), running.display()),
        )
        .unwrap();
        fs::set_permissions(&digest_path, fs::Permissions::from_mode(0o600)).unwrap();
        let missing_token = dir.join("must-not-be-read.jwt");

        let error = readiness_probe(ReadinessProbeRequest {
            pid: std::process::id(),
            expected_executable: &running,
            expected_sha256: &[current_executable_sha256()],
            expected_mode: current_executable_mode(),
            port: 9,
            client_token: &missing_token,
            helper_digest: &digest_path,
            timeout: Duration::from_millis(1),
        })
        .unwrap_err();
        assert!(error.to_string().contains("helper bytes differ"));
        assert!(!missing_token.exists());
        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn readiness_rejects_unpinned_running_bytes_before_token_or_network() {
        let dir = test_dir("running-digest-first");
        fs::create_dir_all(&dir).unwrap();
        let digest_path = readiness_digest(&dir);
        let missing_token = dir.join("must-not-be-read.jwt");
        let error = readiness_probe(ReadinessProbeRequest {
            pid: std::process::id(),
            expected_executable: &std::env::current_exe().unwrap(),
            expected_sha256: &["0".repeat(64)],
            expected_mode: current_executable_mode(),
            port: 9,
            client_token: &missing_token,
            helper_digest: &digest_path,
            timeout: Duration::from_millis(1),
        })
        .unwrap_err();
        assert!(error
            .to_string()
            .contains("running sqld payload bytes differ"));
        assert!(!missing_token.exists());
        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn readiness_rejects_same_bytes_at_swapped_expected_inode_before_token() {
        let dir = test_dir("running-inode-first");
        fs::create_dir_all(&dir).unwrap();
        let digest_path = readiness_digest(&dir);
        let copied = dir.join("sqld-copy");
        fs::copy(std::env::current_exe().unwrap(), &copied).unwrap();
        fs::set_permissions(
            &copied,
            fs::Permissions::from_mode(current_executable_mode()),
        )
        .unwrap();
        let missing_token = dir.join("must-not-be-read.jwt");
        let error = readiness_probe(ReadinessProbeRequest {
            pid: std::process::id(),
            expected_executable: &copied,
            expected_sha256: &[current_executable_sha256()],
            expected_mode: current_executable_mode(),
            port: 9,
            client_token: &missing_token,
            helper_digest: &digest_path,
            timeout: Duration::from_millis(1),
        })
        .unwrap_err();
        assert!(error.to_string().contains("timed out before MainPID"));
        assert!(!missing_token.exists());
        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn readiness_rejects_special_bits_on_digest_record_before_token() {
        let dir = test_dir("digest-special-mode");
        fs::create_dir_all(&dir).unwrap();
        let digest_path = readiness_digest(&dir);
        fs::set_permissions(&digest_path, fs::Permissions::from_mode(0o2600)).unwrap();
        let missing_token = dir.join("must-not-be-read.jwt");
        let error = readiness_probe(ReadinessProbeRequest {
            pid: std::process::id(),
            expected_executable: &std::env::current_exe().unwrap(),
            expected_sha256: &[current_executable_sha256()],
            expected_mode: current_executable_mode(),
            port: 9,
            client_token: &missing_token,
            helper_digest: &digest_path,
            timeout: Duration::from_millis(1),
        })
        .unwrap_err();
        assert!(error.to_string().contains("current-user-owned 0600"));
        assert!(!missing_token.exists());
        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn open_handle_hash_does_not_follow_atomic_path_replacement() {
        let dir = test_dir("open-handle-hash");
        fs::create_dir_all(&dir).unwrap();
        let path = dir.join("helper");
        let replacement = dir.join("replacement");
        fs::write(&path, b"owned-old-inode").unwrap();
        fs::write(&replacement, b"foreign-new-inode").unwrap();
        let mut open = File::open(&path).unwrap();
        fs::rename(&replacement, &path).unwrap();

        let open_digest = sha256_reader(&mut open, "fixture").unwrap();
        let mut expected = Sha256::new();
        expected.update(b"owned-old-inode");
        assert_eq!(open_digest, format!("{:x}", expected.finalize()));
        let mut replacement_digest = Sha256::new();
        replacement_digest.update(b"foreign-new-inode");
        assert_ne!(open_digest, format!("{:x}", replacement_digest.finalize()));
        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn token_reader_refuses_symlink_without_reading_its_target() {
        use std::os::unix::fs::symlink;

        let dir = test_dir("token-symlink");
        fs::create_dir_all(&dir).unwrap();
        let target = dir.join("target.jwt");
        let link = dir.join("client.jwt");
        fs::write(&target, b"target.must.not.be.read").unwrap();
        fs::set_permissions(&target, fs::Permissions::from_mode(0o600)).unwrap();
        symlink(&target, &link).unwrap();
        let error = read_safe_token(&link).unwrap_err();
        assert!(error.to_string().contains("symlinked or non-canonical"));
        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn managed_leaves_refuse_a_symlinked_parent_without_target_mutation() {
        use std::os::unix::fs::symlink;

        let root = test_dir("symlinked-parent");
        let real = root.join("outside");
        let linked = root.join("managed");
        fs::create_dir_all(&real).unwrap();
        fs::set_permissions(&real, fs::Permissions::from_mode(0o700)).unwrap();
        symlink(&real, &linked).unwrap();

        let token = real.join("client.jwt");
        fs::write(&token, b"fixture.jwt.token").unwrap();
        fs::set_permissions(&token, fs::Permissions::from_mode(0o600)).unwrap();
        let error = read_safe_token(&linked.join("client.jwt")).unwrap_err();
        assert!(error.to_string().contains("symlinked or non-canonical"));

        let marker_key = real.join("must-not-create.pem");
        let marker_token = real.join("must-not-create.jwt");
        let error = bootstrap(
            &linked.join("must-not-create.pem"),
            &linked.join("must-not-create.jwt"),
        )
        .unwrap_err();
        assert!(error.to_string().contains("real directory"));
        assert!(!marker_key.exists());
        assert!(!marker_token.exists());

        let record_parent = root.join("records");
        fs::create_dir_all(&record_parent).unwrap();
        fs::set_permissions(&record_parent, fs::Permissions::from_mode(0o700)).unwrap();
        let record = record_parent.join("secretctl.sha256");
        let error = write_self_digest(&record, Some(&linked.join("secretctl"))).unwrap_err();
        assert!(error.to_string().contains("symlinked or non-canonical"));
        assert!(!record.exists());

        let error = compute_source_digest(&linked, &[], &[], &[]).unwrap_err();
        assert!(error.to_string().contains("real directory"));
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn helper_generation_exchange_never_exposes_a_mixed_triple() {
        let root = test_dir("generation-exchange");
        let staged = root.join(".next.fixture");
        let current = root.join("current");
        fs::create_dir_all(&staged).unwrap();
        fs::create_dir_all(&current).unwrap();
        for (directory, generation) in [(&staged, "new"), (&current, "old")] {
            for leaf in ["secretctl", "secretctl.sha256", "secretctl.source.sha256"] {
                fs::write(directory.join(leaf), format!("{generation}:{leaf}")).unwrap();
            }
        }

        let parent = File::open(&root).unwrap();
        exchange_generation_dirs(&parent, &staged, &current).unwrap();
        for leaf in ["secretctl", "secretctl.sha256", "secretctl.source.sha256"] {
            assert_eq!(
                fs::read_to_string(current.join(leaf)).unwrap(),
                format!("new:{leaf}")
            );
            assert_eq!(
                fs::read_to_string(staged.join(leaf)).unwrap(),
                format!("old:{leaf}")
            );
        }

        // Retired-tree cleanup is explicitly post-commit hygiene. An injected permission failure
        // leaves the complete old generation beside the complete active one and does not turn the
        // successful exchange into an error.
        cleanup_retired_generation(&staged, |_| {
            Err(io::Error::new(
                io::ErrorKind::PermissionDenied,
                "injected cleanup refusal",
            ))
        });
        assert!(staged.join("secretctl").is_file());
        assert!(current.join("secretctl").is_file());

        exchange_generation_dirs(&parent, &staged, &current).unwrap();
        assert_eq!(
            fs::read_to_string(current.join("secretctl")).unwrap(),
            "old:secretctl"
        );
        fs::remove_dir_all(root).unwrap();
    }
}
