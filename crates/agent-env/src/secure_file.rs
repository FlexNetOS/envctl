//! No-follow, current-user-owned file primitives for lock/runtime roots of trust.

use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};

use crate::{err, Result};

static STAGE_SEQUENCE: AtomicU64 = AtomicU64::new(0);

#[cfg(unix)]
fn current_ids() -> Result<(u32, u32)> {
    use std::os::unix::fs::MetadataExt;
    let metadata = fs::metadata("/proc/self")?;
    Ok((metadata.uid(), metadata.gid()))
}

#[cfg(unix)]
fn ensure_owned(metadata: &fs::Metadata, path: &Path) -> Result<()> {
    use std::os::unix::fs::MetadataExt;
    let (uid, _) = current_ids()?;
    if metadata.uid() != uid {
        return Err(err(format!(
            "refusing non-current-user-owned path: {}",
            path.display()
        )));
    }
    ensure_not_cross_user_writable(metadata, path, false)?;
    Ok(())
}

#[cfg(not(unix))]
fn ensure_owned(_metadata: &fs::Metadata, _path: &Path) -> Result<()> {
    Ok(())
}

#[cfg(unix)]
fn primary_group_is_private(uid: u32, gid: u32) -> bool {
    let Ok(passwd) = fs::read_to_string("/etc/passwd") else {
        return false;
    };
    let mut current_name = None;
    for line in passwd.lines() {
        let fields = line.split(':').collect::<Vec<_>>();
        if fields.len() < 4 {
            return false;
        }
        let Ok(entry_uid) = fields[2].parse::<u32>() else {
            return false;
        };
        let Ok(entry_gid) = fields[3].parse::<u32>() else {
            return false;
        };
        if entry_uid == uid {
            current_name = Some(fields[0]);
            if entry_gid != gid {
                return false;
            }
        } else if entry_gid == gid {
            return false;
        }
    }
    let Some(current_name) = current_name else {
        return false;
    };
    let Ok(groups) = fs::read_to_string("/etc/group") else {
        return false;
    };
    let mut found = false;
    for line in groups.lines() {
        let fields = line.split(':').collect::<Vec<_>>();
        if fields.len() < 4 {
            return false;
        }
        let Ok(entry_gid) = fields[2].parse::<u32>() else {
            return false;
        };
        if entry_gid != gid {
            continue;
        }
        if found {
            return false;
        }
        found = true;
        if fields[3]
            .split(',')
            .filter(|member| !member.is_empty())
            .any(|member| member != current_name)
        {
            return false;
        }
    }
    found
}

#[cfg(unix)]
fn ensure_not_cross_user_writable(
    metadata: &fs::Metadata,
    path: &Path,
    allow_sticky_system_ancestor: bool,
) -> Result<()> {
    use std::os::unix::fs::{MetadataExt, PermissionsExt};
    let mode = metadata.permissions().mode() & 0o7777;
    let sticky_exception = allow_sticky_system_ancestor && mode & 0o1000 != 0;
    if mode & 0o002 != 0 && !sticky_exception {
        return Err(err(format!(
            "refusing other-writable state authority: {}",
            path.display()
        )));
    }
    if mode & 0o020 != 0 && !sticky_exception {
        let (uid, primary_gid) = current_ids()?;
        if metadata.gid() != primary_gid || !primary_group_is_private(uid, primary_gid) {
            return Err(err(format!(
                "refusing non-private-group-writable state authority: {}",
                path.display()
            )));
        }
    }
    Ok(())
}

/// Validate every existing parent component with `symlink_metadata`, then require the deepest
/// existing parent (the authority that can create/rename the leaf) to be current-user-owned.
pub(crate) fn validate_parent_chain(parent: &Path) -> Result<()> {
    let mut chain = parent
        .ancestors()
        .map(Path::to_path_buf)
        .collect::<Vec<_>>();
    chain.reverse();
    let mut deepest = None;
    #[cfg(unix)]
    let (uid, _) = current_ids()?;
    #[cfg(unix)]
    let mut entered_user_subtree = false;
    for path in chain {
        match fs::symlink_metadata(&path) {
            Ok(metadata) => {
                if metadata.file_type().is_symlink() || !metadata.is_dir() {
                    return Err(err(format!(
                        "path traverses a symlink or non-directory parent: {}",
                        path.display()
                    )));
                }
                #[cfg(unix)]
                {
                    use std::os::unix::fs::MetadataExt;
                    if metadata.uid() == uid {
                        entered_user_subtree = true;
                        ensure_owned(&metadata, &path)?;
                    } else {
                        if entered_user_subtree {
                            return Err(err(format!(
                                "state authority leaves the current-user-owned subtree: {}",
                                path.display()
                            )));
                        }
                        ensure_not_cross_user_writable(&metadata, &path, true)?;
                    }
                }
                deepest = Some((path, metadata));
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => return Err(error.into()),
        }
    }
    let (path, metadata) = deepest.ok_or_else(|| err("path has no existing parent ancestor"))?;
    ensure_owned(&metadata, &path)
}

fn create_parents_no_follow(parent: &Path) -> Result<Vec<std::path::PathBuf>> {
    validate_parent_chain(parent)?;
    let mut missing = Vec::new();
    let mut cursor = parent;
    loop {
        match fs::symlink_metadata(cursor) {
            Ok(metadata) => {
                if metadata.file_type().is_symlink() || !metadata.is_dir() {
                    return Err(err(format!(
                        "path parent must be a real directory: {}",
                        cursor.display()
                    )));
                }
                ensure_owned(&metadata, cursor)?;
                break;
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                missing.push(cursor.to_path_buf());
                cursor = cursor
                    .parent()
                    .ok_or_else(|| err("path has no existing parent ancestor"))?;
            }
            Err(error) => return Err(error.into()),
        }
    }
    let mut created = Vec::new();
    for path in missing.into_iter().rev() {
        if let Err(error) = create_private_directory(&path) {
            cleanup_created_parents(&created);
            return Err(error);
        }
        created.push(path);
    }
    if let Err(error) = validate_parent_chain(parent) {
        cleanup_created_parents(&created);
        return Err(error);
    }
    Ok(created)
}

pub(crate) fn create_private_directory(path: &Path) -> Result<()> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::{DirBuilderExt, PermissionsExt};
        let mut builder = fs::DirBuilder::new();
        builder.mode(0o700).create(path)?;
        if let Err(error) = fs::set_permissions(path, fs::Permissions::from_mode(0o700)) {
            let _ = fs::remove_dir(path);
            return Err(error.into());
        }
    }
    #[cfg(not(unix))]
    fs::create_dir(path)?;
    Ok(())
}

fn cleanup_created_parents(created: &[std::path::PathBuf]) {
    for path in created.iter().rev() {
        let _ = fs::remove_dir(path);
    }
}

fn validate_leaf(path: &Path) -> Result<Option<fs::Metadata>> {
    match fs::symlink_metadata(path) {
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(error.into()),
        Ok(metadata) => {
            if metadata.file_type().is_symlink() || !metadata.is_file() {
                return Err(err(format!(
                    "managed state leaf must be a real regular file: {}",
                    path.display()
                )));
            }
            ensure_owned(&metadata, path)?;
            Ok(Some(metadata))
        }
    }
}

#[cfg(unix)]
fn open_read_no_follow(path: &Path) -> Result<File> {
    use rustix::fs::{open, Mode, OFlags};
    let fd = open(
        path,
        OFlags::RDONLY | OFlags::CLOEXEC | OFlags::NOFOLLOW,
        Mode::empty(),
    )
    .map_err(std::io::Error::from)?;
    Ok(fd.into())
}

#[cfg(not(unix))]
fn open_read_no_follow(path: &Path) -> Result<File> {
    Ok(File::open(path)?)
}

/// Read an optional regular file without following the leaf or any parent symlink.
pub(crate) fn read_optional(path: &Path) -> Result<Option<Vec<u8>>> {
    let parent = path
        .parent()
        .ok_or_else(|| err("managed state path has no parent"))?;
    validate_parent_chain(parent)?;
    if validate_leaf(path)?.is_none() {
        return Ok(None);
    }
    let mut file = open_read_no_follow(path)?;
    let metadata = file.metadata()?;
    if !metadata.is_file() {
        return Err(err(format!(
            "managed state leaf changed type while opening: {}",
            path.display()
        )));
    }
    ensure_owned(&metadata, path)?;
    let mut bytes = Vec::new();
    file.read_to_end(&mut bytes)?;
    Ok(Some(bytes))
}

fn stage_suffix() -> String {
    let sequence = STAGE_SEQUENCE.fetch_add(1, Ordering::Relaxed);
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or_default();
    format!("{}-{sequence}-{nanos}", std::process::id())
}

/// Atomically replace a state file through a current-user-owned, create-new sibling stage.
pub(crate) fn write_atomic(path: &Path, bytes: &[u8], new_mode: u32) -> Result<()> {
    write_atomic_inner(path, bytes, new_mode, true).map(|_| ())
}

/// Atomically initialize a state file. Without `force`, the final hard-link commit is an atomic
/// create-if-absent operation and can never clobber an object raced into the destination.
pub(crate) fn initialize_atomic(
    path: &Path,
    bytes: &[u8],
    new_mode: u32,
    force: bool,
) -> Result<bool> {
    write_atomic_inner(path, bytes, new_mode, force)
}

fn write_atomic_inner(
    path: &Path,
    bytes: &[u8],
    new_mode: u32,
    replace_existing: bool,
) -> Result<bool> {
    let parent = path
        .parent()
        .ok_or_else(|| err("managed state path has no parent"))?;
    let created_parents = create_parents_no_follow(parent)?;
    let name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("state");
    let stage = parent.join(format!(".{name}.envctl-stage-{}", stage_suffix()));
    let result = (|| -> Result<bool> {
        let existing = validate_leaf(path)?;
        if existing.is_some() && !replace_existing {
            return Err(err(format!(
                "managed state leaf already exists: {}",
                path.display()
            )));
        }
        #[cfg(unix)]
        let final_mode = {
            use std::os::unix::fs::PermissionsExt;
            existing
                .as_ref()
                .map(|metadata| (metadata.permissions().mode() & 0o7777) & new_mode)
                .unwrap_or(new_mode)
        };
        #[cfg(not(unix))]
        let final_mode = new_mode;
        let mut options = fs::OpenOptions::new();
        options.write(true).create_new(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            options.mode(0o600);
        }
        let mut file = options.open(&stage)?;
        file.write_all(bytes)?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            file.set_permissions(fs::Permissions::from_mode(final_mode))?;
        }
        file.sync_all()?;
        drop(file);
        validate_parent_chain(parent)?;
        let before_commit = validate_leaf(path)?;
        if replace_existing {
            fs::rename(&stage, path)?;
        } else {
            if before_commit.is_some() {
                return Err(err(format!(
                    "managed state leaf already exists: {}",
                    path.display()
                )));
            }
            // A sibling hard link is the portable atomic no-clobber primitive: unlike rename it
            // fails with AlreadyExists if a concurrent actor creates any destination object.
            fs::hard_link(&stage, path)?;
            fs::remove_file(&stage)?;
        }
        #[cfg(unix)]
        File::open(parent)?.sync_all()?;
        Ok(existing.is_some())
    })();
    if result.is_err() {
        let _ = fs::remove_file(&stage);
        cleanup_created_parents(&created_parents);
    }
    result
}

/// Remove only a no-follow, current-user-owned regular state file.
pub(crate) fn remove_file(path: &Path) -> Result<()> {
    let parent = path
        .parent()
        .ok_or_else(|| err("managed state path has no parent"))?;
    validate_parent_chain(parent)?;
    if validate_leaf(path)?.is_some() {
        fs::remove_file(path)?;
    }
    Ok(())
}

#[cfg(all(test, unix))]
mod tests {
    use super::*;
    use std::os::unix::fs::PermissionsExt;

    fn root(name: &str) -> std::path::PathBuf {
        let path = std::env::temp_dir().join(format!(
            "{name}-{}-{}",
            std::process::id(),
            STAGE_SEQUENCE.fetch_add(1, Ordering::Relaxed)
        ));
        let _ = fs::remove_dir_all(&path);
        fs::create_dir_all(&path).unwrap();
        fs::set_permissions(&path, fs::Permissions::from_mode(0o700)).unwrap();
        path
    }

    #[test]
    fn rejects_other_writable_state_leaf() {
        let root = root("agent-env-insecure-leaf");
        let path = root.join("state.json");
        fs::write(&path, b"{}").unwrap();
        fs::set_permissions(&path, fs::Permissions::from_mode(0o666)).unwrap();
        assert!(read_optional(&path)
            .unwrap_err()
            .to_string()
            .contains("other-writable"));
        assert!(write_atomic(&path, b"{\"safe\":true}", 0o600).is_err());
        fs::set_permissions(&path, fs::Permissions::from_mode(0o600)).unwrap();
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn rejects_other_writable_owned_parent() {
        let root = root("agent-env-insecure-parent");
        let authority = root.join("authority");
        fs::create_dir(&authority).unwrap();
        let path = authority.join("state.json");
        fs::write(&path, b"{}").unwrap();
        fs::set_permissions(&authority, fs::Permissions::from_mode(0o777)).unwrap();
        assert!(read_optional(&path)
            .unwrap_err()
            .to_string()
            .contains("other-writable"));
        assert!(write_atomic(&path, b"{\"safe\":true}", 0o600).is_err());
        fs::set_permissions(&authority, fs::Permissions::from_mode(0o700)).unwrap();
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn atomic_write_creates_private_parents_independent_of_umask() {
        struct UmaskRestore(rustix::fs::Mode);
        impl Drop for UmaskRestore {
            fn drop(&mut self) {
                rustix::process::umask(self.0);
            }
        }

        let _lock = crate::dirs::test_env_lock();
        let root = root("agent-env-private-created-parents");
        let previous_umask = rustix::process::umask(rustix::fs::Mode::empty());
        let _restore = UmaskRestore(previous_umask);
        let path = root.join("nested/deep/state.json");
        write_atomic(&path, b"{}", 0o600).unwrap();
        for directory in [root.join("nested"), root.join("nested/deep")] {
            assert_eq!(
                fs::metadata(directory).unwrap().permissions().mode() & 0o777,
                0o700
            );
        }
        assert_eq!(
            fs::metadata(&path).unwrap().permissions().mode() & 0o777,
            0o600
        );
        fs::remove_dir_all(root).unwrap();
    }
}
