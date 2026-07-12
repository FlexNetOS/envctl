//! db_atomic — the parse-plan-temp-fsync-rename-reread-hash atomic edit primitive
//! with a `.bak` rollback point (REQ ARCH08 / MISS07 / NFR08).
//!
//! Every in-place mutation the db surface performs (the `db refactor --apply`
//! and, transitively, safe promotions) goes through [`atomic_backup_write`] so a
//! crashed or racing write can never leave a target half-written or silently
//! corrupt:
//!
//!   1. **backup** — an existing target is copied to `<dest>.bak` first, so the
//!      prior content is always recoverable (git is the outer backstop; this is
//!      the inner one).
//!   2. **temp + fsync** — the new bytes are written to a sibling temp file which
//!      is `fsync`'d (and the parent dir `fsync`'d) so the data is durable before
//!      the rename is observable.
//!   3. **rename** — the temp is renamed over the destination, which is atomic on
//!      POSIX: a reader sees either the whole old file or the whole new one.
//!   4. **reread + hash-verify** — the destination is read back and its SHA-256
//!      compared to the intended bytes. On mismatch the target is restored from
//!      the `.bak` and a typed error is returned (fail-closed: never leave a
//!      target whose content does not match what was asked for).
//!
//! Pure `std::fs` + `sha2` (already engine deps) — no new C in the trust boundary.

use crate::db::{DbError, Result};
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};

/// Suffix appended to a target to hold its pre-write content for rollback.
pub const BAK_SUFFIX: &str = ".bak";

/// SHA-256 hex digest of `bytes`.
fn sha256_hex(bytes: &[u8]) -> String {
    let mut h = Sha256::new();
    h.update(bytes);
    format!("{:x}", h.finalize())
}

/// The `.bak` path for a destination (`<dest>.bak`).
fn bak_path(dest: &Path) -> PathBuf {
    let mut s = dest.as_os_str().to_owned();
    s.push(BAK_SUFFIX);
    PathBuf::from(s)
}

/// Best-effort fsync of a path (file or directory). A platform that cannot fsync
/// a directory handle is tolerated — the rename is still atomic; durability of
/// the *directory entry* is the only thing weakened, which git backstops.
fn fsync(path: &Path) -> std::io::Result<()> {
    match std::fs::File::open(path) {
        Ok(f) => f.sync_all(),
        // Opening a directory for sync is not portable everywhere; don't fail the
        // write over a missing durability guarantee.
        Err(_) => Ok(()),
    }
}

/// Atomically replace `dest` with `bytes`, keeping a `.bak` of any prior content
/// and verifying the result by re-reading and hashing. Creates parent dirs.
///
/// Returns the `.bak` path when an existing target was backed up (`None` for a
/// brand-new file). On a post-write hash mismatch the target is restored from the
/// backup and [`DbError::Io`] is returned.
pub fn atomic_backup_write(dest: &Path, bytes: &[u8]) -> Result<Option<PathBuf>> {
    let parent = dest.parent();
    if let Some(p) = parent {
        std::fs::create_dir_all(p).map_err(|e| DbError::Io(format!("{}: {e}", p.display())))?;
    }

    // 1. Back up an existing target for rollback.
    let backup = if dest.is_file() {
        let bak = bak_path(dest);
        std::fs::copy(dest, &bak).map_err(|e| DbError::Io(format!("{}: {e}", bak.display())))?;
        Some(bak)
    } else {
        None
    };

    // 2. Write to a temp sibling and fsync it.
    let want_hash = sha256_hex(bytes);
    let tmp = dest.with_extension(format!(
        "{}.envctl-atomic-tmp",
        dest.extension().and_then(|e| e.to_str()).unwrap_or("out")
    ));
    std::fs::write(&tmp, bytes).map_err(|e| DbError::Io(format!("{}: {e}", tmp.display())))?;
    fsync(&tmp).map_err(|e| DbError::Io(format!("fsync {}: {e}", tmp.display())))?;

    // 3. Atomic rename over the destination, then fsync the parent dir entry.
    std::fs::rename(&tmp, dest).map_err(|e| {
        let _ = std::fs::remove_file(&tmp);
        DbError::Io(format!(
            "rename {} -> {}: {e}",
            tmp.display(),
            dest.display()
        ))
    })?;
    if let Some(p) = parent {
        let _ = fsync(p);
    }

    // 4. Re-read and hash-verify; roll back on mismatch.
    let readback =
        std::fs::read(dest).map_err(|e| DbError::Io(format!("reread {}: {e}", dest.display())))?;
    if sha256_hex(&readback) != want_hash {
        // Restore the prior content if we have it; otherwise remove the corrupt file.
        match &backup {
            Some(bak) => {
                let _ = std::fs::copy(bak, dest);
            }
            None => {
                let _ = std::fs::remove_file(dest);
            }
        }
        return Err(DbError::Io(format!(
            "post-write verification failed for {} (hash mismatch); restored from backup",
            dest.display()
        )));
    }

    Ok(backup)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn tmp(tag: &str) -> PathBuf {
        let d = std::env::temp_dir().join(format!("envctl-db-atomic-{tag}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&d);
        fs::create_dir_all(&d).unwrap();
        d
    }

    #[test]
    fn new_file_write_makes_no_backup_and_verifies() {
        let d = tmp("new");
        let dest = d.join("sub/created.sh");
        let bak = atomic_backup_write(&dest, b"cd $LIFE_OS_ROOT\n").unwrap();
        assert!(bak.is_none(), "no prior content -> no .bak");
        assert_eq!(fs::read_to_string(&dest).unwrap(), "cd $LIFE_OS_ROOT\n");
        assert!(!bak_path(&dest).exists());
        let _ = fs::remove_dir_all(&d);
    }

    #[test]
    fn existing_file_is_backed_up_then_replaced_and_verified() {
        let d = tmp("replace");
        let dest = d.join("wrapper.sh");
        fs::write(&dest, b"cd $META_ROOT\n").unwrap();

        let bak = atomic_backup_write(&dest, b"cd $LIFE_OS_ROOT\n")
            .unwrap()
            .expect("existing target -> .bak");
        // New content in place; old content preserved in the backup.
        assert_eq!(fs::read_to_string(&dest).unwrap(), "cd $LIFE_OS_ROOT\n");
        assert_eq!(fs::read_to_string(&bak).unwrap(), "cd $META_ROOT\n");
        assert_eq!(bak, bak_path(&dest));
        let _ = fs::remove_dir_all(&d);
    }

    #[test]
    fn no_temp_file_is_left_behind() {
        let d = tmp("clean");
        let dest = d.join("f.toml");
        atomic_backup_write(&dest, b"key = \"v\"\n").unwrap();
        let leftovers: Vec<_> = fs::read_dir(&d)
            .unwrap()
            .filter_map(|e| e.ok())
            .map(|e| e.file_name().to_string_lossy().into_owned())
            .filter(|n| n.contains("envctl-atomic-tmp"))
            .collect();
        assert!(leftovers.is_empty(), "temp left behind: {leftovers:?}");
        let _ = fs::remove_dir_all(&d);
    }
}
