//! The fail-closed guard engine — the Rust port of `ubuntu-boot-repair.sh`'s
//! `resolve_verified()` + refusal chain (L73-105). Evaluated before any
//! destructive phase. The cardinal rule: **when uncertain, REFUSE.** A guard that
//! cannot resolve, re-verify, or prove its precondition returns `Some(reason)`
//! (→ `OpStatus::Refused`) — it NEVER silently passes. If `blkid`/`findmnt` are
//! missing or error, that is treated as "cannot prove safe" → refuse.
use crate::component::{Guard, Hook, HookRunner, Phase};
use crate::error::RunContext;
use crate::event::EventSink;
use crate::layout::MetaLayout;
use crate::model::{OpResult, OpStatus};
use std::ffi::OsString;
use std::path::Path;
use std::process::{Command, Output};

/// Portable [`crate::model::DataPath::uuid`] sentinel: resolve the filesystem UUID carrying the
/// data path at purge time, then run the same UUID-resolves, live-root, and mount re-verification
/// chain as an explicitly declared UUID. It is intentionally a literal manifest value rather than
/// a workstation UUID so manifests remain portable without weakening the fail-closed guard.
pub const RUNTIME_FILESYSTEM_UUID: &str = "runtime";

/// `Some(reason)` if ANY guard refuses; `None` only if every guard affirmatively
/// passes.
pub fn check_guards(guards: &[Guard], runner: &dyn HookRunner, ctx: &RunContext) -> Option<String> {
    for g in guards {
        if let Some(reason) = check_one(g, runner, ctx) {
            return Some(reason);
        }
    }
    None
}

fn check_one(g: &Guard, runner: &dyn HookRunner, ctx: &RunContext) -> Option<String> {
    match g {
        Guard::PathExists { path } => {
            let p = expand_tilde(path);
            if Path::new(&p).exists() {
                None
            } else {
                Some(format!("refused: required path missing: {path}"))
            }
        }

        Guard::HookSucceeds { hook } => {
            let r = runner.run(
                "<guard>",
                Phase::Detect,
                hook,
                false,
                &crate::event::EventSink::null(),
            );
            if r.status == OpStatus::Ok {
                None
            } else {
                Some(format!(
                    "refused: guard hook did not succeed ({})",
                    r.message
                ))
            }
        }

        // Resolve a device by UUID and RE-VERIFY it carries that UUID. Any
        // failure to resolve or re-verify => refuse (fail-closed).
        Guard::UuidResolves { uuid } => resolve_verified_device(uuid).err(),

        // Refuse if this UUID/device IS the live/running root.
        Guard::NotLiveDevice { uuid } => {
            if let Err(reason) = resolve_verified_device(uuid) {
                return Some(reason);
            }
            let live_uuid = match ctx
                .live_root_uuid
                .as_deref()
                .filter(|value| valid_single_value(value))
                .map(str::to_owned)
                .or_else(resolve_live_root_uuid)
            {
                Some(live_uuid) => live_uuid,
                None => {
                    return Some(format!(
                        "refused: cannot prove the live root UUID while checking {uuid}"
                    ));
                }
            };
            if same_uuid(&live_uuid, uuid) {
                Some(format!("refused: {uuid} is the LIVE root filesystem"))
            } else {
                None
            }
        }

        // Refuse if the UUID is currently mounted anywhere (the "never umount
        // /home"). If we cannot run findmnt, we cannot prove it is unmounted → refuse.
        Guard::NotMounted { uuid } => {
            let dev = match resolve_verified_device(uuid) {
                Ok(dev) => dev,
                Err(reason) => return Some(reason),
            };
            let by_device = findmnt_mount_state(["-S", dev.as_str()]);
            let uuid_source = format!("UUID={uuid}");
            let by_uuid = findmnt_mount_state(["--source", uuid_source.as_str()]);
            match (by_device, by_uuid) {
                (MountState::NotMounted, MountState::NotMounted) => None,
                (MountState::Mounted, _) | (_, MountState::Mounted) => {
                    Some(format!("refused: {uuid} is currently mounted"))
                }
                _ => Some(format!(
                    "refused: findmnt could not prove {uuid} is unmounted"
                )),
            }
        }
    }
}

// ---- helpers (each treats a tool failure as the conservative/refusing branch) --

fn expand_tilde(p: &str) -> String {
    MetaLayout::from_env_or_default().expand_meta_path(p)
}

fn valid_single_value(value: &str) -> bool {
    !value.is_empty()
        && value.trim() == value
        && !value.chars().any(|ch| matches!(ch, '\n' | '\r' | '\0'))
}

fn same_uuid(left: &str, right: &str) -> bool {
    left.eq_ignore_ascii_case(right)
}

fn successful_single_value(output: Output) -> Option<String> {
    if !output.status.success() {
        return None;
    }
    let stderr = std::str::from_utf8(&output.stderr).ok()?;
    if !stderr.trim().is_empty() {
        return None;
    }
    let stdout = std::str::from_utf8(&output.stdout).ok()?;
    let value = stdout.trim();
    if valid_single_value(value) {
        Some(value.to_owned())
    } else {
        None
    }
}

/// `blkid -U <uuid>` → device path, or None.
fn resolve_dev(uuid: &str) -> Option<String> {
    if !valid_single_value(uuid) {
        return None;
    }
    let out = trusted_blkid_command().args(["-U", uuid]).output().ok()?;
    successful_single_value(out)
}

/// `blkid -s UUID -o value <dev>` → its UUID, or None.
fn uuid_of(dev: &str) -> Option<String> {
    if !valid_single_value(dev) {
        return None;
    }
    let blkid = trusted_blkid_command()
        .args(["-s", "UUID", "-o", "value", dev])
        .output()
        .ok()
        .and_then(successful_single_value);
    blkid.or_else(|| {
        trusted_lsblk_command()
            .args(["-dnro", "UUID", "--", dev])
            .output()
            .ok()
            .and_then(successful_single_value)
    })
}

fn resolve_verified_device(uuid: &str) -> Result<String, String> {
    let dev = resolve_dev(uuid).ok_or_else(|| {
        format!("refused: UUID {uuid} did not resolve (blkid unavailable or unknown)")
    })?;
    let observed = uuid_of(&dev).ok_or_else(|| {
        format!("refused: UUID {uuid} resolved to {dev} but could not be re-verified")
    })?;
    if same_uuid(&observed, uuid) {
        Ok(dev)
    } else {
        Err(format!(
            "refused: UUID {uuid} resolved to {dev} but re-verified as {observed}"
        ))
    }
}

/// findmnt's SOURCE column carries a btrfs-subvolume / bind-mount suffix, e.g.
/// `/dev/nvme0n1p2[/@]`. Strip it so the bare device can be fed to blkid and
/// compared against `resolve_dev` output. (AUDIT-FIX blocker: without this the
/// live-root guard failed OPEN on every btrfs/bind root — blkid errored on the
/// suffix → live_root_uuid None → both checks skipped.)
fn strip_subvol(src: &str) -> &str {
    match src.find('[') {
        Some(i) => src[..i].trim_end(),
        None => src,
    }
}

/// `findmnt -no SOURCE /` → the live root device (subvol suffix stripped).
fn live_root_source() -> Option<String> {
    let out = trusted_findmnt_command()
        .args(["-no", "SOURCE", "/"])
        .output()
        .ok()?;
    let source = successful_single_value(out)?;
    let source = strip_subvol(&source);
    if valid_single_value(source) {
        Some(source.to_owned())
    } else {
        None
    }
}

fn same_device_identity(left: &str, right: &str) -> bool {
    if left == right {
        return true;
    }
    if let (Ok(left), Ok(right)) = (std::fs::canonicalize(left), std::fs::canonicalize(right)) {
        if left == right {
            return true;
        }
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::{FileTypeExt, MetadataExt};
        if let (Ok(left), Ok(right)) = (std::fs::metadata(left), std::fs::metadata(right)) {
            return left.file_type().is_block_device()
                && right.file_type().is_block_device()
                && left.rdev() == right.rdev();
        }
    }
    false
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum MountState {
    Mounted,
    NotMounted,
    Indeterminate,
}

fn findmnt_mount_state<const N: usize>(args: [&str; N]) -> MountState {
    let Ok(output) = trusted_findmnt_command().args(args).output() else {
        return MountState::Indeterminate;
    };
    let (Ok(stdout), Ok(stderr)) = (
        std::str::from_utf8(&output.stdout),
        std::str::from_utf8(&output.stderr),
    ) else {
        return MountState::Indeterminate;
    };
    let stdout_empty = stdout.trim().is_empty();
    let stderr_empty = stderr.trim().is_empty();
    if output.status.success() {
        if !stdout_empty && stderr_empty {
            MountState::Mounted
        } else {
            MountState::Indeterminate
        }
    } else if output.status.code() == Some(1) && stdout_empty && stderr_empty {
        MountState::NotMounted
    } else {
        MountState::Indeterminate
    }
}

/// Resolve the live root UUID once, for `RunContext`. Prefer findmnt's own UUID
/// column (robust on btrfs/bind/LUKS where SOURCE is decorated or a mapper node);
/// fall back to SOURCE→blkid with the subvol suffix stripped. (AUDIT-FIX blocker:
/// the old SOURCE→blkid-only path returned None on btrfs, disabling the guard.)
pub fn resolve_live_root_uuid() -> Option<String> {
    let reported_uuid = successful_single_value(
        trusted_findmnt_command()
            .args(["-no", "UUID", "/"])
            .output()
            .ok()?,
    )?;
    let source = live_root_source()?;
    let source_uuid = uuid_of(&source)?;
    if !same_uuid(&reported_uuid, &source_uuid) {
        return None;
    }
    let resolved = resolve_verified_device(&reported_uuid).ok()?;
    if !same_device_identity(&source, &resolved) {
        return None;
    }
    Some(reported_uuid)
}

/// A no-op HookRunner (every hook → Failed). `verify_path_uuid` only exercises
/// UuidResolves/NotLiveDevice (which don't touch the runner), so this just
/// satisfies `check_one`'s signature.
struct NullRunner;
impl HookRunner for NullRunner {
    fn run(&self, comp: &str, phase: Phase, _h: &Hook, _d: bool, _s: &EventSink) -> OpResult {
        OpResult {
            component: comp.into(),
            phase,
            status: OpStatus::Failed,
            availability: None,
            exit_code: None,
            duration_ms: 0,
            message: "null runner".into(),
            dry_run: false,
        }
    }
}

/// Fail-closed UUID re-verify for a `--purge` target: the path must exist, its
/// UUID must resolve + re-verify, it must NOT be the live root, and the mount
/// carrying the path must actually report the declared UUID. Returns
/// `Some(reason)` to REFUSE (never deletes on uncertainty).
pub fn verify_path_uuid(path: &str, uuid: &str, ctx: &RunContext) -> Option<String> {
    let p = expand_tilde(path);
    if !Path::new(&p).exists() {
        return Some(format!("refused: purge target missing: {path}"));
    }
    let runtime_uuid;
    let uuid = if uuid == RUNTIME_FILESYSTEM_UUID {
        runtime_uuid = match mount_uuid_of(&p) {
            Some(uuid) => uuid,
            None => {
                return Some(format!(
                    "refused: cannot resolve runtime fs UUID carrying {path}"
                ));
            }
        };
        runtime_uuid.as_str()
    } else {
        uuid
    };
    if let Some(r) = check_one(&Guard::UuidResolves { uuid: uuid.into() }, &NullRunner, ctx) {
        return Some(r);
    }
    if let Some(r) = check_one(
        &Guard::NotLiveDevice { uuid: uuid.into() },
        &NullRunner,
        ctx,
    ) {
        return Some(r);
    }
    match mount_uuid_of(&p) {
        Some(f) if f == uuid => None,
        Some(f) => Some(format!(
            "refused: {path} is on UUID {f}, not the declared {uuid}"
        )),
        None => Some(format!(
            "refused: cannot determine the fs UUID carrying {path}"
        )),
    }
}

/// The fs UUID carrying `path`. Prefer findmnt's UUID column (robust on
/// btrfs/bind/LUKS); fall back to SOURCE→blkid with the subvol suffix stripped.
/// (AUDIT-FIX major: SOURCE→blkid-only returned None on btrfs because the column
/// carries a `[/subvol]` suffix that blkid rejects → purge wrongly refused.)
fn mount_uuid_of(path: &str) -> Option<String> {
    let reported_uuid = successful_single_value(
        trusted_findmnt_command()
            .args(["-no", "UUID", "--target", path])
            .output()
            .ok()?,
    )?;
    let source_output = trusted_findmnt_command()
        .args(["-no", "SOURCE", "--target", path])
        .output()
        .ok()?;
    let source = successful_single_value(source_output)?;
    let source = strip_subvol(&source);
    if !valid_single_value(source) {
        return None;
    }
    let source_uuid = uuid_of(source)?;
    if !same_uuid(&source_uuid, &reported_uuid) {
        return None;
    }
    let resolved = resolve_verified_device(&reported_uuid).ok()?;
    if !same_device_identity(source, &resolved) {
        return None;
    }
    Some(reported_uuid)
}

fn trusted_blkid_program() -> OsString {
    #[cfg(test)]
    if let Some(program) = std::env::var_os("ENVCTL_TEST_BLKID_BIN") {
        return program;
    }
    OsString::from("/usr/sbin/blkid")
}

fn trusted_findmnt_program() -> OsString {
    #[cfg(test)]
    if let Some(program) = std::env::var_os("ENVCTL_TEST_FINDMNT_BIN") {
        return program;
    }
    OsString::from("/usr/bin/findmnt")
}

fn trusted_lsblk_program() -> OsString {
    #[cfg(test)]
    if let Some(program) = std::env::var_os("ENVCTL_TEST_LSBLK_BIN") {
        return program;
    }
    OsString::from("/usr/bin/lsblk")
}

fn trusted_guard_command(program: OsString) -> Command {
    let mut command = Command::new(program);
    command
        .env_clear()
        .env("PATH", "/usr/bin:/usr/sbin:/bin:/sbin")
        .env("LC_ALL", "C");
    command
}

fn trusted_blkid_command() -> Command {
    trusted_guard_command(trusted_blkid_program())
}

fn trusted_findmnt_command() -> Command {
    trusted_guard_command(trusted_findmnt_program())
}

fn trusted_lsblk_command() -> Command {
    trusted_guard_command(trusted_lsblk_program())
}

#[cfg(test)]
mod tests {
    use super::{strip_subvol, RUNTIME_FILESYSTEM_UUID};

    #[cfg(unix)]
    struct FakeGuardTools {
        root: std::path::PathBuf,
        prior_blkid: Option<std::ffi::OsString>,
        prior_findmnt: Option<std::ffi::OsString>,
        prior_lsblk: Option<std::ffi::OsString>,
    }

    #[cfg(unix)]
    impl FakeGuardTools {
        fn new(blkid_body: &str, findmnt_body: &str) -> Self {
            use std::os::unix::fs::PermissionsExt;

            let stamp = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos();
            let root = std::env::temp_dir()
                .join(format!("envctl-guard-tools-{}-{stamp}", std::process::id()));
            std::fs::create_dir(&root).unwrap();
            let write_tool = |name: &str, body: &str| {
                let path = root.join(name);
                std::fs::write(&path, format!("#!/bin/sh\n{body}\n")).unwrap();
                std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755)).unwrap();
                path
            };
            let blkid = write_tool("blkid", blkid_body);
            let findmnt = write_tool("findmnt", findmnt_body);
            let lsblk = write_tool("lsblk", "exit 1");
            let tools = Self {
                root,
                prior_blkid: std::env::var_os("ENVCTL_TEST_BLKID_BIN"),
                prior_findmnt: std::env::var_os("ENVCTL_TEST_FINDMNT_BIN"),
                prior_lsblk: std::env::var_os("ENVCTL_TEST_LSBLK_BIN"),
            };
            std::env::set_var("ENVCTL_TEST_BLKID_BIN", blkid);
            std::env::set_var("ENVCTL_TEST_FINDMNT_BIN", findmnt);
            std::env::set_var("ENVCTL_TEST_LSBLK_BIN", lsblk);
            tools
        }

        fn missing_findmnt(&self) {
            std::env::set_var("ENVCTL_TEST_FINDMNT_BIN", self.root.join("missing-findmnt"));
        }

        fn set_lsblk(&self, body: &str) {
            use std::os::unix::fs::PermissionsExt;

            let path = self.root.join("lsblk");
            std::fs::write(&path, format!("#!/bin/sh\n{body}\n")).unwrap();
            std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o755)).unwrap();
        }
    }

    #[cfg(unix)]
    impl Drop for FakeGuardTools {
        fn drop(&mut self) {
            for (key, prior) in [
                ("ENVCTL_TEST_BLKID_BIN", self.prior_blkid.take()),
                ("ENVCTL_TEST_FINDMNT_BIN", self.prior_findmnt.take()),
                ("ENVCTL_TEST_LSBLK_BIN", self.prior_lsblk.take()),
            ] {
                match prior {
                    Some(value) => std::env::set_var(key, value),
                    None => std::env::remove_var(key),
                }
            }
            let _ = std::fs::remove_dir_all(&self.root);
        }
    }

    #[cfg(unix)]
    const VERIFIED_DATA_BLKID: &str = r#"
case "$*" in
  '-U data-uuid') printf '%s\n' /dev/data ;;
  '-s UUID -o value /dev/data') printf '%s\n' data-uuid ;;
  *) exit 1 ;;
esac
"#;

    #[test]
    fn destructive_guard_tools_are_absolute_and_environment_scrubbed() {
        let _lock = crate::test_env_lock();
        let prior_path = std::env::var_os("PATH");
        let prior_preload = std::env::var_os("LD_PRELOAD");
        let prior_blkid = std::env::var_os("ENVCTL_TEST_BLKID_BIN");
        let prior_findmnt = std::env::var_os("ENVCTL_TEST_FINDMNT_BIN");
        let prior_lsblk = std::env::var_os("ENVCTL_TEST_LSBLK_BIN");
        std::env::set_var("PATH", "/must/not/use");
        std::env::set_var("LD_PRELOAD", "/must/not/load.so");
        std::env::remove_var("ENVCTL_TEST_BLKID_BIN");
        std::env::remove_var("ENVCTL_TEST_FINDMNT_BIN");
        std::env::remove_var("ENVCTL_TEST_LSBLK_BIN");

        let blkid = super::trusted_blkid_command();
        let findmnt = super::trusted_findmnt_command();
        let lsblk = super::trusted_lsblk_command();

        for (key, value) in [
            ("PATH", prior_path),
            ("LD_PRELOAD", prior_preload),
            ("ENVCTL_TEST_BLKID_BIN", prior_blkid),
            ("ENVCTL_TEST_FINDMNT_BIN", prior_findmnt),
            ("ENVCTL_TEST_LSBLK_BIN", prior_lsblk),
        ] {
            match value {
                Some(value) => std::env::set_var(key, value),
                None => std::env::remove_var(key),
            }
        }

        assert_eq!(blkid.get_program(), "/usr/sbin/blkid");
        assert_eq!(findmnt.get_program(), "/usr/bin/findmnt");
        assert_eq!(lsblk.get_program(), "/usr/bin/lsblk");
        for command in [blkid, findmnt, lsblk] {
            assert!(command
                .get_envs()
                .all(|(key, _)| key != std::ffi::OsStr::new("LD_PRELOAD")));
            assert_eq!(
                command
                    .get_envs()
                    .find(|(key, _)| *key == std::ffi::OsStr::new("LC_ALL"))
                    .and_then(|(_, value)| value),
                Some(std::ffi::OsStr::new("C"))
            );
        }
    }

    #[test]
    fn strip_subvol_handles_btrfs_and_bind_suffixes() {
        assert_eq!(strip_subvol("/dev/nvme0n1p2"), "/dev/nvme0n1p2");
        assert_eq!(strip_subvol("/dev/nvme0n1p2[/@]"), "/dev/nvme0n1p2");
        assert_eq!(strip_subvol("/dev/sda1[/@home]"), "/dev/sda1");
        // a trailing space before the bracket is trimmed
        assert_eq!(strip_subvol("/dev/sda1 [/@]"), "/dev/sda1");
    }

    #[cfg(unix)]
    #[test]
    fn not_live_refuses_unverified_candidate_and_unknown_live_identity() {
        use crate::component::Guard;
        use crate::error::RunContext;

        let _env = crate::test_env_lock();
        {
            let _tools = FakeGuardTools::new(
                r#"
case "$*" in
  '-U data-uuid') printf '%s\n' /dev/data ;;
  '-s UUID -o value /dev/data') printf '%s\n' forged-uuid ;;
  *) exit 1 ;;
esac
"#,
                "exit 1",
            );
            let reason = super::check_one(
                &Guard::NotLiveDevice {
                    uuid: "data-uuid".into(),
                },
                &super::NullRunner,
                &RunContext {
                    live_root_uuid: Some("live-uuid".into()),
                    ..RunContext::default()
                },
            );
            assert!(reason.is_some(), "candidate UUID must be re-verified");
        }
        {
            let _tools = FakeGuardTools::new(VERIFIED_DATA_BLKID, "exit 2");
            let reason = super::check_one(
                &Guard::NotLiveDevice {
                    uuid: "data-uuid".into(),
                },
                &super::NullRunner,
                &RunContext::default(),
            );
            assert!(
                reason.is_some(),
                "an unavailable live-root identity must refuse"
            );
        }
    }

    #[cfg(unix)]
    #[test]
    fn not_live_compares_verified_uuid_identity_not_device_spelling() {
        use crate::component::Guard;
        use crate::error::RunContext;

        let _env = crate::test_env_lock();
        let _tools = FakeGuardTools::new(VERIFIED_DATA_BLKID, "exit 1");
        let reason = super::check_one(
            &Guard::NotLiveDevice {
                uuid: "data-uuid".into(),
            },
            &super::NullRunner,
            &RunContext {
                live_root_uuid: Some("DATA-UUID".into()),
                ..RunContext::default()
            },
        );
        assert!(
            reason.is_some(),
            "UUID identity comparison must be canonical"
        );
    }

    #[cfg(unix)]
    #[test]
    fn unprivileged_lsblk_fallback_preserves_verified_guard_capability() {
        use crate::component::Guard;
        use crate::error::RunContext;

        let _env = crate::test_env_lock();
        let tools = FakeGuardTools::new(
            r#"
case "$*" in
  '-U data-uuid') printf '%s\n' /dev/data ;;
  '-U live-uuid') printf '%s\n' /dev/live ;;
  '-s UUID -o value /dev/data'|'-s UUID -o value /dev/live') exit 0 ;;
  *) exit 1 ;;
esac
"#,
            r#"
case "$*" in
  '-no UUID /') printf '%s\n' live-uuid ;;
  '-no SOURCE /') printf '%s\n' /dev/live ;;
  *) exit 1 ;;
esac
"#,
        );
        tools.set_lsblk(
            r#"
case "$*" in
  '-dnro UUID -- /dev/data') printf '%s\n' data-uuid ;;
  '-dnro UUID -- /dev/live') printf '%s\n' live-uuid ;;
  *) exit 1 ;;
esac
"#,
        );

        assert_eq!(
            super::resolve_live_root_uuid().as_deref(),
            Some("live-uuid")
        );
        assert!(super::check_one(
            &Guard::NotLiveDevice {
                uuid: "data-uuid".into(),
            },
            &super::NullRunner,
            &RunContext::default(),
        )
        .is_none());
    }

    #[cfg(unix)]
    #[test]
    fn not_mounted_only_passes_two_explicit_no_match_results() {
        use crate::component::Guard;
        use crate::error::RunContext;

        let _env = crate::test_env_lock();
        let cases = [
            ("exit 2", true, "findmnt exit 2"),
            ("exit 0", true, "successful empty output"),
            ("exit 1", false, "explicit empty no-match"),
            (
                "printf '%s\\n' findmnt-error >&2; exit 1",
                true,
                "exit 1 with stderr",
            ),
            (
                "printf '%s\\n' malformed; exit 1",
                true,
                "exit 1 with stdout",
            ),
            ("printf '%s\\n' mounted", true, "positive mounted row"),
        ];
        for (findmnt_body, must_refuse, label) in cases {
            let tools = FakeGuardTools::new(VERIFIED_DATA_BLKID, findmnt_body);
            let reason = super::check_one(
                &Guard::NotMounted {
                    uuid: "data-uuid".into(),
                },
                &super::NullRunner,
                &RunContext::default(),
            );
            assert_eq!(reason.is_some(), must_refuse, "{label}");
            drop(tools);
        }

        let tools = FakeGuardTools::new(VERIFIED_DATA_BLKID, "exit 1");
        tools.missing_findmnt();
        let reason = super::check_one(
            &Guard::NotMounted {
                uuid: "data-uuid".into(),
            },
            &super::NullRunner,
            &RunContext::default(),
        );
        assert!(reason.is_some(), "findmnt spawn failure must refuse");
    }

    #[cfg(unix)]
    #[test]
    fn live_and_target_uuid_proofs_reject_malformed_or_inconsistent_findmnt() {
        let _env = crate::test_env_lock();
        {
            let _tools = FakeGuardTools::new(
                r#"
case "$*" in
  '-U garbage') printf '%s\n' /dev/other ;;
  '-s UUID -o value /dev/live') printf '%s\n' live-uuid ;;
  '-s UUID -o value /dev/other') printf '%s\n' garbage ;;
  *) exit 1 ;;
esac
"#,
                r#"
case "$*" in
  '-no UUID /') printf '%s\n' garbage ;;
  '-no SOURCE /') printf '%s\n' /dev/live ;;
  *) exit 1 ;;
esac
"#,
            );
            assert_eq!(super::resolve_live_root_uuid(), None);
        }
        {
            let _tools = FakeGuardTools::new(VERIFIED_DATA_BLKID, "exit 0");
            assert_eq!(super::resolve_live_root_uuid(), None);
        }
        {
            let root =
                std::env::temp_dir().join(format!("envctl-guard-target-{}", std::process::id()));
            std::fs::write(&root, b"target").unwrap();
            let findmnt = format!(
                r#"
case "$*" in
  '-no UUID --target {target}') printf '%s\n' data-uuid ;;
  '-no SOURCE --target {target}') printf '%s\n' /dev/other ;;
  *) exit 1 ;;
esac
"#,
                target = root.display()
            );
            let _tools = FakeGuardTools::new(
                r#"
case "$*" in
  '-U data-uuid') printf '%s\n' /dev/data ;;
  '-s UUID -o value /dev/data') printf '%s\n' data-uuid ;;
  '-s UUID -o value /dev/other') printf '%s\n' data-uuid ;;
  *) exit 1 ;;
esac
"#,
                &findmnt,
            );
            assert_eq!(super::mount_uuid_of(root.to_str().unwrap()), None);
            std::fs::remove_file(root).unwrap();
        }
    }

    #[cfg(unix)]
    #[test]
    fn runtime_filesystem_uuid_purge_preserves_by_default_and_reverifies_before_rename() {
        use crate::error::RunContext;
        use crate::model::{DataPath, ResetGates, Wiring};
        use std::ffi::OsString;
        use std::os::unix::fs::PermissionsExt;
        use std::path::Path;

        let _env = crate::test_env_lock();
        let stamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root = std::env::temp_dir().join(format!(
            "envctl-runtime-purge-{}-{stamp}",
            std::process::id()
        ));
        let bin = root.join("bin");
        let data = root.join("data");
        std::fs::create_dir_all(&bin).unwrap();
        std::fs::create_dir_all(&data).unwrap();
        std::fs::write(data.join("marker"), "preserve then trash\n").unwrap();

        fn executable(path: &Path, body: &str) {
            std::fs::write(path, body).unwrap();
            std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o755)).unwrap();
        }
        executable(
            &bin.join("findmnt"),
            "#!/bin/sh\ncase \"$*\" in\n  '-no UUID --target '*) echo data-uuid ;;\n  '-no SOURCE --target '*) echo /dev/data ;;\n  '-no SOURCE /') echo /dev/live ;;\n  *) exit 1 ;;\nesac\n",
        );
        executable(
            &bin.join("blkid"),
            "#!/bin/sh\ncase \"$*\" in\n  '-U data-uuid') echo /dev/data ;;\n  '-s UUID -o value /dev/data') echo data-uuid ;;\n  *) exit 1 ;;\nesac\n",
        );

        struct ToolRestore(Vec<(&'static str, Option<OsString>)>);
        impl Drop for ToolRestore {
            fn drop(&mut self) {
                for (key, value) in self.0.drain(..) {
                    match value {
                        Some(value) => std::env::set_var(key, value),
                        None => std::env::remove_var(key),
                    }
                }
            }
        }
        let _restore = ToolRestore(vec![
            (
                "ENVCTL_TEST_BLKID_BIN",
                std::env::var_os("ENVCTL_TEST_BLKID_BIN"),
            ),
            (
                "ENVCTL_TEST_FINDMNT_BIN",
                std::env::var_os("ENVCTL_TEST_FINDMNT_BIN"),
            ),
        ]);
        std::env::set_var("ENVCTL_TEST_BLKID_BIN", bin.join("blkid"));
        std::env::set_var("ENVCTL_TEST_FINDMNT_BIN", bin.join("findmnt"));

        let wiring = Wiring {
            data_paths: vec![DataPath {
                path: data.display().to_string(),
                uuid: Some(RUNTIME_FILESYSTEM_UUID.into()),
            }],
            ..Wiring::default()
        };
        let ctx = RunContext {
            live_root_uuid: Some("live-uuid".into()),
            ..RunContext::default()
        };

        let preview = crate::wiring::revert(&wiring, &ResetGates::default(), &ctx);
        assert!(preview.failures.is_empty(), "{preview:?}");
        assert!(data.join("marker").exists());

        let purged = crate::wiring::revert(
            &wiring,
            &ResetGates {
                purge: true,
                ..ResetGates::default()
            },
            &ctx,
        );
        assert!(purged.failures.is_empty(), "{purged:?}");
        assert!(!data.exists());
        let trashed = std::fs::read_dir(&root)
            .unwrap()
            .filter_map(Result::ok)
            .map(|entry| entry.path())
            .find(|path| {
                path.file_name()
                    .is_some_and(|name| name.to_string_lossy().starts_with("data.envctl-trash."))
            })
            .expect("purge must rename the data directory to recoverable trash");
        assert_eq!(
            std::fs::read_to_string(trashed.join("marker")).unwrap(),
            "preserve then trash\n"
        );
        std::fs::remove_dir_all(root).unwrap();
    }

    #[cfg(unix)]
    #[test]
    fn runtime_filesystem_uuid_purge_refuses_the_live_root_filesystem() {
        use crate::error::RunContext;

        // `/` is guaranteed to exist. If its UUID cannot be established, the runtime sentinel
        // refuses earlier; if it can, declaring that resolved identity live must still refuse.
        let ctx = RunContext {
            live_root_uuid: super::mount_uuid_of("/"),
            ..RunContext::default()
        };
        assert!(super::verify_path_uuid("/", RUNTIME_FILESYSTEM_UUID, &ctx).is_some());
    }
}
