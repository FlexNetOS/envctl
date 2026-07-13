//! Deterministic, fail-closed snapshots of installable directory trees.
//!
//! Hashing and copying deliberately share this representation.  A source is traversed once,
//! symlinks are resolved under a containment boundary, and the resulting immutable snapshot is
//! what is both hashed and materialized.  That prevents the two operations from disagreeing
//! about symlinks, empty directories, file modes, or source changes between verification and
//! installation.

use std::collections::HashSet;
use std::ffi::{OsStr, OsString};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use sha2::{Digest, Sha256};

use crate::{err, Result};

#[cfg(unix)]
const TREE_HASH_DOMAIN: &[u8] = b"envctl-agent-env-tree-v1-unix\0";
#[cfg(windows)]
const TREE_HASH_DOMAIN: &[u8] = b"envctl-agent-env-tree-v1-windows-wide\0";
#[cfg(not(any(unix, windows)))]
const TREE_HASH_DOMAIN: &[u8] = b"envctl-agent-env-tree-v1-platform-lossy\0";
static TEMP_SEQUENCE: AtomicU64 = AtomicU64::new(0);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum EntryKind {
    Directory,
    File,
}

#[derive(Clone, Copy)]
enum SymlinkPolicy {
    FollowContained,
    RejectAll,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct TreeEntry {
    /// Empty for the snapshot root; otherwise raw, platform-native relative components.
    path: Vec<Vec<u8>>,
    kind: EntryKind,
    mode: u32,
    bytes: Vec<u8>,
}

/// An immutable, deterministic view of an installable directory tree.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TreeSnapshot {
    entries: Vec<TreeEntry>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum MaterializeStageFault {
    None,
    AfterCreate,
    BeforeCapture,
}

impl TreeSnapshot {
    /// Capture one selected file while proving its canonical target remains in the materialized
    /// source. Internal symlinks are followed; escapes, cycles, directories, and special entries
    /// are rejected.
    pub fn capture_file_within(containment_root: &Path, path: &Path) -> Result<Vec<u8>> {
        let boundary = fs::canonicalize(containment_root).map_err(|error| {
            err(format!(
                "cannot canonicalize file containment root {}: {error}",
                containment_root.display()
            ))
        })?;
        let canonical = fs::canonicalize(path).map_err(|error| {
            err(format!(
                "symlink cycle or unresolved selected file {}: {error}",
                path.display()
            ))
        })?;
        if canonical != boundary && !canonical.starts_with(&boundary) {
            return Err(err(format!(
                "selected file escapes its materialized source: {} -> {}",
                path.display(),
                canonical.display()
            )));
        }
        let metadata = fs::metadata(&canonical)?;
        if !metadata.is_file() {
            return Err(err(format!(
                "selected source is not a regular file: {}",
                path.display()
            )));
        }
        Ok(fs::read(canonical)?)
    }

    /// Capture `root`, following only symlinks whose canonical targets remain under `root`.
    ///
    /// Symlinked files/directories become ordinary effective entries in the snapshot.  Cycles,
    /// external escapes and special filesystem entries are rejected. Native non-UTF-8 names are
    /// preserved losslessly in the platform-tagged hash domain.
    pub fn capture(root: &Path) -> Result<Self> {
        Self::capture_within(root, root)
    }

    /// Capture an installed destination without following a destination-root symlink.
    pub fn capture_destination(root: &Path) -> Result<Self> {
        let root_link_meta = fs::symlink_metadata(root).map_err(|e| {
            err(format!(
                "cannot inspect destination root {}: {e}",
                root.display()
            ))
        })?;
        if root_link_meta.file_type().is_symlink() {
            return Err(err(format!(
                "destination root must not be a symlink: {}",
                root.display()
            )));
        }
        if !root_link_meta.is_dir() {
            return Err(err(format!(
                "destination root is not a directory: {}",
                root.display()
            )));
        }
        Self::capture_with_policy(root, root, SymlinkPolicy::RejectAll)
    }

    /// Capture a selected tree while proving its root remains within `containment_root`.
    ///
    /// This variant exists for a selected skill that is itself an internal symlink in a
    /// materialized source. Nested links may reuse shared content elsewhere in that source pack,
    /// but may not escape the pack.
    pub fn capture_within(containment_root: &Path, root: &Path) -> Result<Self> {
        Self::capture_with_policy(containment_root, root, SymlinkPolicy::FollowContained)
    }

    fn capture_with_policy(
        containment_root: &Path,
        root: &Path,
        symlink_policy: SymlinkPolicy,
    ) -> Result<Self> {
        let root_meta = fs::metadata(root).map_err(|e| {
            err(format!(
                "cannot inspect snapshot root {}: {e}",
                root.display()
            ))
        })?;
        if !root_meta.is_dir() {
            return Err(err(format!(
                "snapshot root is not a directory: {}",
                root.display()
            )));
        }
        let canonical_root = fs::canonicalize(root).map_err(|e| {
            err(format!(
                "cannot canonicalize snapshot root {}: {e}",
                root.display()
            ))
        })?;
        let canonical_boundary = fs::canonicalize(containment_root).map_err(|e| {
            err(format!(
                "cannot canonicalize snapshot containment root {}: {e}",
                containment_root.display()
            ))
        })?;
        if canonical_root != canonical_boundary && !canonical_root.starts_with(&canonical_boundary)
        {
            return Err(err(format!(
                "selected snapshot root escapes its materialized source: {} -> {}",
                root.display(),
                canonical_root.display()
            )));
        }

        let mut entries = Vec::new();
        let mut active_directories = HashSet::new();
        capture_node(
            root,
            &[],
            &canonical_boundary,
            symlink_policy,
            &mut active_directories,
            &mut entries,
        )?;
        entries.sort_by(|a, b| a.path.cmp(&b.path));
        Ok(Self { entries })
    }

    /// SHA-256 over a versioned, length-framed encoding of every effective entry.
    pub fn hash(&self) -> String {
        let mut hasher = Sha256::new();
        hasher.update(TREE_HASH_DOMAIN);
        for entry in &self.entries {
            hasher.update([match entry.kind {
                EntryKind::Directory => b'd',
                EntryKind::File => b'f',
            }]);
            hasher.update((entry.path.len() as u64).to_le_bytes());
            for component in &entry.path {
                hash_frame(&mut hasher, component);
            }
            hasher.update(entry.mode.to_le_bytes());
            hash_frame(&mut hasher, &entry.bytes);
        }
        format!("{:x}", hasher.finalize())
    }

    /// Convert a captured source tree to the subset Git can reproduce across Unix clones.
    ///
    /// Git records file contents, names, and only the executable bit. It does not record empty
    /// directories or the remaining permission bits. Project locks are intended to survive a
    /// fresh clone, so their installed snapshot uses canonical directory/file modes and rejects
    /// empty directories instead of attesting bytes that Git cannot carry.
    pub fn into_git_portable(mut self) -> Result<Self> {
        for directory in self
            .entries
            .iter()
            .filter(|entry| entry.kind == EntryKind::Directory)
        {
            let has_descendant = self.entries.iter().any(|candidate| {
                candidate.path.len() > directory.path.len()
                    && candidate.path.starts_with(&directory.path)
            });
            if !has_descendant {
                return Err(err(format!(
                    "project skill contains an empty directory that Git cannot reproduce: {}",
                    path_from_components(&directory.path).display()
                )));
            }
        }

        #[cfg(unix)]
        for entry in &mut self.entries {
            entry.mode = match entry.kind {
                EntryKind::Directory => 0o755,
                EntryKind::File if entry.mode & 0o111 != 0 => 0o755,
                EntryKind::File => 0o644,
            };
        }

        Ok(self)
    }

    /// Materialize this snapshot at `destination`, replacing it with a verified sibling rename.
    ///
    /// The old destination is retained until the new tree has been fully written and re-captured;
    /// a failed final rename restores the old tree.
    pub fn install_atomic(&self, destination: &Path) -> Result<()> {
        if let Ok(metadata) = fs::symlink_metadata(destination) {
            if metadata.file_type().is_symlink() {
                return Err(err(format!(
                    "refusing symlink destination root without proven ownership: {}",
                    destination.display()
                )));
            }
            if !metadata.is_dir() {
                return Err(err(format!(
                    "refusing non-directory destination root: {}",
                    destination.display()
                )));
            }
            Self::capture_destination(destination)?;
        }
        let parent = destination.parent().unwrap_or_else(|| Path::new("."));
        fs::create_dir_all(parent)?;
        let stage = unique_sibling(destination, "stage");
        let backup = unique_sibling(destination, "backup");

        let result = (|| -> Result<()> {
            self.write_new_tree(&stage)?;
            let staged = Self::capture_destination(&stage)?;
            if staged != *self {
                return Err(err(format!(
                    "staged tree verification failed for {}",
                    destination.display()
                )));
            }

            let had_destination = fs::symlink_metadata(destination).is_ok();
            if had_destination {
                fs::rename(destination, &backup)?;
            }
            if let Err(e) = fs::rename(&stage, destination) {
                if had_destination {
                    let _ = fs::rename(&backup, destination);
                }
                return Err(e.into());
            }
            if had_destination {
                // The commit point is the successful stage→destination rename.  Cleanup must not
                // turn that committed success into an error; first make a read-only old tree
                // removable, then perform best-effort post-commit cleanup.
                let _ = make_tree_removable(&backup);
                let _ = remove_any(&backup);
            }
            Ok(())
        })();

        if fs::symlink_metadata(&stage).is_ok() {
            let _ = make_tree_removable(&stage);
            let _ = remove_any(&stage);
        }
        if result.is_err()
            && fs::symlink_metadata(destination).is_err()
            && fs::symlink_metadata(&backup).is_ok()
        {
            let _ = fs::rename(&backup, destination);
        }
        result
    }

    /// Materialize and verify a new staging tree without touching a live destination.
    pub(crate) fn materialize_staged(&self, stage: &Path) -> Result<()> {
        self.materialize_staged_inner(stage, MaterializeStageFault::None)
    }

    pub(crate) fn materialize_staged_inner(
        &self,
        stage: &Path,
        fault: MaterializeStageFault,
    ) -> Result<()> {
        self.write_new_tree_inner(stage, fault == MaterializeStageFault::AfterCreate)?;
        let result = if fault == MaterializeStageFault::BeforeCapture {
            Err(err(format!(
                "injected staged tree capture failure: {}",
                stage.display()
            )))
        } else {
            Self::capture_destination(stage).and_then(|staged| {
                if staged == *self {
                    Ok(())
                } else {
                    Err(err(format!(
                        "staged tree verification failed: {}",
                        stage.display()
                    )))
                }
            })
        };
        if result.is_err() {
            let _ = make_tree_removable(stage);
            let _ = remove_any(stage);
        }
        result
    }

    fn write_new_tree(&self, destination: &Path) -> Result<()> {
        self.write_new_tree_inner(destination, false)
    }

    fn write_new_tree_inner(&self, destination: &Path, fail_after_create: bool) -> Result<()> {
        if fs::symlink_metadata(destination).is_ok() {
            return Err(err(format!(
                "snapshot staging path already exists: {}",
                destination.display()
            )));
        }
        fs::create_dir(destination)?;
        let result = (|| -> Result<()> {
            if fail_after_create {
                return Err(err(format!(
                    "injected staged tree write failure: {}",
                    destination.display()
                )));
            }
            for entry in self.entries.iter().filter(|entry| !entry.path.is_empty()) {
                let path = destination.join(path_from_components(&entry.path));
                match entry.kind {
                    EntryKind::Directory => fs::create_dir(&path)?,
                    EntryKind::File => {
                        let parent = path.parent().ok_or_else(|| {
                            err(format!(
                                "snapshot entry has no parent: {}",
                                path_from_components(&entry.path).display()
                            ))
                        })?;
                        fs::create_dir_all(parent)?;
                        fs::write(&path, &entry.bytes)?;
                        set_mode(&path, entry.mode)?;
                    }
                }
            }

            // Set directory permissions after populating children so read-only source directories
            // remain reproducible without preventing snapshot construction.
            let mut directories: Vec<&TreeEntry> = self
                .entries
                .iter()
                .filter(|entry| entry.kind == EntryKind::Directory)
                .collect();
            directories.sort_by_key(|entry| std::cmp::Reverse(entry.path.len()));
            for entry in directories {
                let path = if entry.path.is_empty() {
                    destination.to_path_buf()
                } else {
                    destination.join(path_from_components(&entry.path))
                };
                set_mode(&path, entry.mode)?;
            }
            Ok(())
        })();
        if result.is_err() {
            let _ = make_tree_removable(destination);
            let _ = remove_any(destination);
        }
        result
    }
}

fn capture_node(
    physical_path: &Path,
    logical_path: &[Vec<u8>],
    canonical_root: &Path,
    symlink_policy: SymlinkPolicy,
    active_directories: &mut HashSet<PathBuf>,
    entries: &mut Vec<TreeEntry>,
) -> Result<()> {
    let link_meta = fs::symlink_metadata(physical_path)?;
    let is_symlink = link_meta.file_type().is_symlink();
    if is_symlink && matches!(symlink_policy, SymlinkPolicy::RejectAll) {
        return Err(err(format!(
            "destination tree must not contain symlinks: {}",
            physical_path.display()
        )));
    }
    if matches!(symlink_policy, SymlinkPolicy::RejectAll) {
        ensure_current_user_owns(&link_meta, physical_path)?;
    }
    let canonical = fs::canonicalize(physical_path).map_err(|e| {
        err(format!(
            "symlink cycle or unresolved entry at {}: {e}",
            physical_path.display()
        ))
    })?;
    if canonical != canonical_root && !canonical.starts_with(canonical_root) {
        return Err(err(format!(
            "symlink escapes snapshot root: {} -> {}",
            physical_path.display(),
            canonical.display()
        )));
    }

    let metadata = if is_symlink {
        fs::metadata(physical_path)?
    } else {
        link_meta
    };
    if metadata.is_dir() {
        if !active_directories.insert(canonical.clone()) {
            return Err(err(format!(
                "symlink cycle detected at {}",
                physical_path.display()
            )));
        }
        entries.push(TreeEntry {
            path: logical_path.to_vec(),
            kind: EntryKind::Directory,
            mode: relevant_mode(&metadata),
            bytes: Vec::new(),
        });

        let mut children: Vec<(Vec<u8>, OsString)> = Vec::new();
        for child in fs::read_dir(&canonical)? {
            let child = child?;
            let name = child.file_name();
            let encoded = encode_component(&name);
            if encoded.is_empty() || name == OsStr::new(".") || name == OsStr::new("..") {
                return Err(err(format!("invalid snapshot entry name: {name:?}")));
            }
            children.push((encoded, name));
        }
        children.sort_by(|a, b| a.0.cmp(&b.0));
        for (encoded, name) in children {
            let mut child_logical = logical_path.to_vec();
            child_logical.push(encoded);
            capture_node(
                &canonical.join(&name),
                &child_logical,
                canonical_root,
                symlink_policy,
                active_directories,
                entries,
            )?;
        }
        active_directories.remove(&canonical);
        return Ok(());
    }

    if metadata.is_file() {
        entries.push(TreeEntry {
            path: logical_path.to_vec(),
            kind: EntryKind::File,
            mode: relevant_mode(&metadata),
            bytes: fs::read(&canonical)?,
        });
        return Ok(());
    }

    Err(err(format!(
        "unsupported special filesystem entry: {}",
        physical_path.display()
    )))
}

fn hash_frame(hasher: &mut Sha256, bytes: &[u8]) {
    hasher.update((bytes.len() as u64).to_le_bytes());
    hasher.update(bytes);
}

#[cfg(unix)]
fn relevant_mode(metadata: &fs::Metadata) -> u32 {
    use std::os::unix::fs::PermissionsExt;
    metadata.permissions().mode() & 0o7777
}

#[cfg(unix)]
fn ensure_current_user_owns(metadata: &fs::Metadata, path: &Path) -> Result<()> {
    use std::os::unix::fs::MetadataExt;

    let status = fs::read_to_string("/proc/self/status")?;
    let uid = status
        .lines()
        .find_map(|line| line.strip_prefix("Uid:"))
        .and_then(|fields| fields.split_whitespace().next())
        .and_then(|value| value.parse::<u32>().ok())
        .ok_or_else(|| err("cannot determine current uid from /proc/self/status"))?;
    if metadata.uid() != uid {
        return Err(err(format!(
            "destination entry is not owned by the current user: {}",
            path.display()
        )));
    }
    Ok(())
}

#[cfg(not(unix))]
fn ensure_current_user_owns(_metadata: &fs::Metadata, _path: &Path) -> Result<()> {
    Ok(())
}

#[cfg(not(unix))]
fn relevant_mode(_metadata: &fs::Metadata) -> u32 {
    0
}

#[cfg(unix)]
fn set_mode(path: &Path, mode: u32) -> Result<()> {
    use std::os::unix::fs::PermissionsExt;
    fs::set_permissions(path, fs::Permissions::from_mode(mode))?;
    Ok(())
}

#[cfg(not(unix))]
fn set_mode(_path: &Path, _mode: u32) -> Result<()> {
    Ok(())
}

fn path_from_components(path: &[Vec<u8>]) -> PathBuf {
    path.iter()
        .map(|component| decode_component(component))
        .collect()
}

#[cfg(unix)]
fn encode_component(name: &OsStr) -> Vec<u8> {
    use std::os::unix::ffi::OsStrExt;
    name.as_bytes().to_vec()
}

#[cfg(unix)]
fn decode_component(bytes: &[u8]) -> OsString {
    use std::os::unix::ffi::OsStringExt;
    OsString::from_vec(bytes.to_vec())
}

#[cfg(windows)]
fn encode_component(name: &OsStr) -> Vec<u8> {
    use std::os::windows::ffi::OsStrExt;
    name.encode_wide()
        .flat_map(u16::to_le_bytes)
        .collect::<Vec<_>>()
}

#[cfg(windows)]
fn decode_component(bytes: &[u8]) -> OsString {
    use std::os::windows::ffi::OsStringExt;
    let wide = bytes
        .chunks_exact(2)
        .map(|pair| u16::from_le_bytes([pair[0], pair[1]]))
        .collect::<Vec<_>>();
    OsString::from_wide(&wide)
}

#[cfg(not(any(unix, windows)))]
fn encode_component(name: &OsStr) -> Vec<u8> {
    name.to_string_lossy().as_bytes().to_vec()
}

#[cfg(not(any(unix, windows)))]
fn decode_component(bytes: &[u8]) -> OsString {
    OsString::from(String::from_utf8_lossy(bytes).into_owned())
}

fn unique_sibling(path: &Path, role: &str) -> PathBuf {
    let parent = path.parent().unwrap_or_else(|| Path::new("."));
    let name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("agent-env-tree");
    loop {
        let sequence = TEMP_SEQUENCE.fetch_add(1, Ordering::Relaxed);
        let candidate = parent.join(format!(
            ".{name}.envctl-{role}-{}-{sequence}",
            std::process::id()
        ));
        if fs::symlink_metadata(&candidate).is_err() {
            return candidate;
        }
    }
}

fn remove_any(path: &Path) -> Result<()> {
    let metadata = fs::symlink_metadata(path)?;
    if metadata.is_dir() && !metadata.file_type().is_symlink() {
        fs::remove_dir_all(path)?;
    } else {
        fs::remove_file(path)?;
    }
    Ok(())
}

fn make_tree_removable(path: &Path) -> Result<()> {
    let metadata = fs::symlink_metadata(path)?;
    if metadata.is_dir() && !metadata.file_type().is_symlink() {
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(path, fs::Permissions::from_mode(0o700))?;
        }
        for entry in fs::read_dir(path)? {
            make_tree_removable(&entry?.path())?;
        }
    } else {
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            if !metadata.file_type().is_symlink() {
                fs::set_permissions(path, fs::Permissions::from_mode(0o600))?;
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_dir(prefix: &str) -> PathBuf {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("clock")
            .as_nanos();
        std::env::temp_dir().join(format!("{prefix}-{}-{nanos}", std::process::id()))
    }

    #[test]
    fn empty_directory_changes_snapshot_hash_and_is_materialized() {
        let root = temp_dir("agent-env-tree-empty");
        fs::create_dir_all(&root).expect("root");
        let before = TreeSnapshot::capture(&root).expect("before").hash();
        fs::create_dir(root.join("empty")).expect("empty");
        let snapshot = TreeSnapshot::capture(&root).expect("after");
        assert_ne!(before, snapshot.hash());

        let destination = temp_dir("agent-env-tree-empty-dst");
        snapshot.install_atomic(&destination).expect("install");
        assert!(destination.join("empty").is_dir());
        assert_eq!(snapshot, TreeSnapshot::capture(&destination).unwrap());

        let _ = fs::remove_dir_all(&root);
        let _ = fs::remove_dir_all(&destination);
    }

    #[test]
    fn git_portable_snapshot_rejects_empty_directories() {
        let root = temp_dir("agent-env-tree-git-empty");
        fs::create_dir_all(root.join("empty")).expect("empty");
        fs::write(root.join("SKILL.md"), "portable").expect("skill");
        let message = TreeSnapshot::capture(&root)
            .unwrap()
            .into_git_portable()
            .unwrap_err()
            .to_string();
        assert!(message.contains("empty directory"), "{message}");
        let _ = fs::remove_dir_all(root);
    }

    #[cfg(unix)]
    #[test]
    fn real_git_clones_under_different_umasks_share_portable_tree_hash() {
        use std::os::unix::fs::PermissionsExt;
        use std::process::Command;

        let root = temp_dir("agent-env-tree-git-clone");
        let repo = root.join("source");
        fs::create_dir_all(repo.join("skill")).expect("repo");
        fs::write(repo.join("skill/SKILL.md"), "# Skill\n").expect("skill");
        fs::write(repo.join("skill/run.sh"), "#!/bin/sh\nexit 0\n").expect("script");
        fs::set_permissions(repo.join("skill/run.sh"), fs::Permissions::from_mode(0o755))
            .expect("executable");
        for args in [
            vec!["init", "--quiet"],
            vec!["config", "user.email", "envctl@example.invalid"],
            vec!["config", "user.name", "envctl test"],
            vec!["add", "."],
            vec!["commit", "--quiet", "-m", "fixture"],
        ] {
            let status = Command::new("git")
                .args(args)
                .current_dir(&repo)
                .status()
                .expect("git available");
            assert!(status.success());
        }

        let clone_with_umask = |mask: &str, destination: &Path| {
            let status = Command::new("sh")
                .args([
                    "-c",
                    "umask \"$1\"; exec git clone --quiet \"$2\" \"$3\"",
                    "envctl-git-clone",
                    mask,
                ])
                .arg(&repo)
                .arg(destination)
                .status()
                .expect("clone");
            assert!(status.success());
        };
        let clone_a = root.join("clone-a");
        let clone_b = root.join("clone-b");
        clone_with_umask("0022", &clone_a);
        clone_with_umask("0002", &clone_b);

        let a = TreeSnapshot::capture(&clone_a.join("skill"))
            .unwrap()
            .into_git_portable()
            .unwrap();
        let b = TreeSnapshot::capture(&clone_b.join("skill"))
            .unwrap()
            .into_git_portable()
            .unwrap();
        assert_eq!(a.hash(), b.hash());

        let destination = root.join("installed");
        a.install_atomic(&destination).unwrap();
        assert_eq!(
            fs::metadata(destination.join("SKILL.md"))
                .unwrap()
                .permissions()
                .mode()
                & 0o777,
            0o644
        );
        assert_eq!(
            fs::metadata(destination.join("run.sh"))
                .unwrap()
                .permissions()
                .mode()
                & 0o777,
            0o755
        );
        let _ = fs::remove_dir_all(root);
    }

    #[cfg(unix)]
    #[test]
    fn internal_file_and_directory_symlinks_are_followed() {
        use std::os::unix::fs::symlink;

        let root = temp_dir("agent-env-tree-links");
        fs::create_dir_all(root.join("refs")).expect("refs");
        fs::write(root.join("plain.txt"), "plain").expect("plain");
        fs::write(root.join("refs/guide.md"), "guide").expect("guide");
        symlink("plain.txt", root.join("linked.txt")).expect("file link");
        symlink("refs", root.join("linked-refs")).expect("dir link");

        let snapshot = TreeSnapshot::capture(&root).expect("capture");
        let destination = temp_dir("agent-env-tree-links-dst");
        snapshot.install_atomic(&destination).expect("install");
        assert_eq!(
            fs::read_to_string(destination.join("linked.txt")).unwrap(),
            "plain"
        );
        assert_eq!(
            fs::read_to_string(destination.join("linked-refs/guide.md")).unwrap(),
            "guide"
        );
        assert!(!fs::symlink_metadata(destination.join("linked.txt"))
            .unwrap()
            .file_type()
            .is_symlink());

        let _ = fs::remove_dir_all(&root);
        let _ = fs::remove_dir_all(&destination);
    }

    #[cfg(unix)]
    #[test]
    fn external_symlink_escape_is_rejected() {
        use std::os::unix::fs::symlink;

        let root = temp_dir("agent-env-tree-escape");
        let outside = temp_dir("agent-env-tree-outside");
        fs::create_dir_all(&root).expect("root");
        fs::create_dir_all(&outside).expect("outside");
        fs::write(outside.join("secret"), "nope").expect("secret");
        symlink(outside.join("secret"), root.join("file-link")).expect("file link");
        assert!(TreeSnapshot::capture(&root)
            .unwrap_err()
            .to_string()
            .contains("escapes snapshot root"));
        fs::remove_file(root.join("file-link")).expect("remove file link");
        symlink(&outside, root.join("dir-link")).expect("dir link");
        assert!(TreeSnapshot::capture(&root)
            .unwrap_err()
            .to_string()
            .contains("escapes snapshot root"));

        let _ = fs::remove_dir_all(&root);
        let _ = fs::remove_dir_all(&outside);
    }

    #[cfg(unix)]
    #[test]
    fn selected_root_symlink_must_remain_inside_materialized_source() {
        use std::os::unix::fs::symlink;

        let source = temp_dir("agent-env-tree-source-boundary");
        let outside = temp_dir("agent-env-tree-root-outside");
        fs::create_dir_all(&source).expect("source");
        fs::create_dir_all(&outside).expect("outside");
        fs::write(outside.join("SKILL.md"), "outside").expect("skill");
        symlink(&outside, source.join("escaped-skill")).expect("link");
        let message = TreeSnapshot::capture_within(&source, &source.join("escaped-skill"))
            .unwrap_err()
            .to_string();
        assert!(
            message.contains("escapes its materialized source"),
            "{message}"
        );

        fs::create_dir_all(source.join("inside")).expect("inside");
        fs::write(source.join("inside/SKILL.md"), "inside").expect("skill");
        symlink("inside", source.join("internal-skill")).expect("internal link");
        TreeSnapshot::capture_within(&source, &source.join("internal-skill"))
            .expect("internal root link");

        let _ = fs::remove_dir_all(&source);
        let _ = fs::remove_dir_all(&outside);
    }

    #[cfg(unix)]
    #[test]
    fn configured_source_root_symlink_is_supported() {
        use std::os::unix::fs::symlink;

        let real = temp_dir("agent-env-tree-configured-real");
        let link_parent = temp_dir("agent-env-tree-configured-link-parent");
        fs::create_dir_all(&real).expect("real");
        fs::create_dir_all(&link_parent).expect("link parent");
        fs::write(real.join("SKILL.md"), "linked root").expect("skill");
        let link = link_parent.join("source");
        symlink(&real, &link).expect("link");
        assert_eq!(
            TreeSnapshot::capture(&link).expect("capture linked root"),
            TreeSnapshot::capture(&real).expect("capture real root")
        );

        let _ = fs::remove_dir_all(&real);
        let _ = fs::remove_dir_all(&link_parent);
    }

    #[cfg(unix)]
    #[test]
    fn nested_link_may_reuse_shared_sibling_inside_materialized_source() {
        use std::os::unix::fs::symlink;

        let pack = temp_dir("agent-env-tree-shared-pack");
        fs::create_dir_all(pack.join("skill")).expect("skill");
        fs::create_dir_all(pack.join("shared")).expect("shared");
        fs::write(pack.join("skill/SKILL.md"), "skill").expect("skill file");
        fs::write(pack.join("shared/guide.md"), "guide").expect("guide");
        symlink("../shared", pack.join("skill/references")).expect("shared link");

        let snapshot = TreeSnapshot::capture_within(&pack, &pack.join("skill"))
            .expect("shared sibling is contained by pack");
        let destination = temp_dir("agent-env-tree-shared-dst");
        snapshot.install_atomic(&destination).expect("install");
        assert_eq!(
            fs::read_to_string(destination.join("references/guide.md")).unwrap(),
            "guide"
        );

        let _ = fs::remove_dir_all(&pack);
        let _ = fs::remove_dir_all(&destination);
    }

    #[cfg(unix)]
    #[test]
    fn atomic_install_refuses_destination_root_symlink() {
        use std::os::unix::fs::symlink;

        let source = temp_dir("agent-env-tree-destination-link-src");
        let outside = temp_dir("agent-env-tree-destination-link-outside");
        let parent = temp_dir("agent-env-tree-destination-link-parent");
        fs::create_dir_all(&source).expect("source");
        fs::create_dir_all(&outside).expect("outside");
        fs::create_dir_all(&parent).expect("parent");
        fs::write(source.join("SKILL.md"), "source").expect("source skill");
        fs::write(outside.join("untouched"), "foreign").expect("foreign");
        let destination = parent.join("skill");
        symlink(&outside, &destination).expect("destination link");

        let message = TreeSnapshot::capture(&source)
            .unwrap()
            .install_atomic(&destination)
            .unwrap_err()
            .to_string();
        assert!(message.contains("symlink destination root"), "{message}");
        assert_eq!(
            fs::read_to_string(outside.join("untouched")).unwrap(),
            "foreign"
        );
        assert!(fs::symlink_metadata(&destination)
            .unwrap()
            .file_type()
            .is_symlink());

        let _ = fs::remove_dir_all(&source);
        let _ = fs::remove_dir_all(&outside);
        let _ = fs::remove_dir_all(&parent);
    }

    #[cfg(unix)]
    #[test]
    fn destination_capture_rejects_nested_file_and_directory_symlinks() {
        use std::os::unix::fs::symlink;

        let destination = temp_dir("agent-env-tree-nested-destination-link");
        fs::create_dir_all(destination.join("real-dir")).expect("destination");
        fs::write(destination.join("real-file"), "data").expect("file");
        symlink("real-file", destination.join("linked-file")).expect("file link");
        let message = TreeSnapshot::capture_destination(&destination)
            .unwrap_err()
            .to_string();
        assert!(message.contains("must not contain symlinks"), "{message}");
        fs::remove_file(destination.join("linked-file")).expect("remove link");
        symlink("real-dir", destination.join("linked-dir")).expect("dir link");
        let message = TreeSnapshot::capture_destination(&destination)
            .unwrap_err()
            .to_string();
        assert!(message.contains("must not contain symlinks"), "{message}");
        let _ = fs::remove_dir_all(destination);
    }

    #[test]
    fn atomic_install_refuses_existing_regular_file_destination() {
        let source = temp_dir("agent-env-tree-file-dst-src");
        let parent = temp_dir("agent-env-tree-file-dst-parent");
        fs::create_dir_all(&source).expect("source");
        fs::create_dir_all(&parent).expect("parent");
        fs::write(source.join("SKILL.md"), "source").expect("skill");
        let destination = parent.join("skill");
        fs::write(&destination, "foreign file").expect("foreign");

        let message = TreeSnapshot::capture(&source)
            .unwrap()
            .install_atomic(&destination)
            .unwrap_err()
            .to_string();
        assert!(
            message.contains("non-directory destination root"),
            "{message}"
        );
        assert_eq!(fs::read_to_string(&destination).unwrap(), "foreign file");

        let _ = fs::remove_dir_all(&source);
        let _ = fs::remove_dir_all(&parent);
    }

    #[cfg(unix)]
    #[test]
    fn replacing_read_only_tree_reports_success_and_cleans_backup() {
        use std::os::unix::fs::PermissionsExt;

        let source = temp_dir("agent-env-tree-readonly-src");
        let parent = temp_dir("agent-env-tree-readonly-parent");
        let destination = parent.join("skill");
        fs::create_dir_all(&source).expect("source");
        fs::create_dir_all(destination.join("locked")).expect("old tree");
        fs::write(source.join("SKILL.md"), "new").expect("new");
        fs::write(destination.join("locked/old"), "old").expect("old");
        fs::set_permissions(
            destination.join("locked"),
            fs::Permissions::from_mode(0o555),
        )
        .expect("lock nested dir");
        fs::set_permissions(&destination, fs::Permissions::from_mode(0o555))
            .expect("lock root dir");

        TreeSnapshot::capture(&source)
            .unwrap()
            .install_atomic(&destination)
            .expect("committed replacement");
        assert_eq!(
            fs::read_to_string(destination.join("SKILL.md")).unwrap(),
            "new"
        );
        let leftovers = fs::read_dir(&parent)
            .unwrap()
            .map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned())
            .filter(|name| name.contains("envctl-backup"))
            .collect::<Vec<_>>();
        assert!(leftovers.is_empty(), "backup residue: {leftovers:?}");

        let _ = fs::remove_dir_all(&source);
        let _ = fs::remove_dir_all(&parent);
    }

    #[cfg(unix)]
    #[test]
    fn symlink_cycle_is_rejected() {
        use std::os::unix::fs::symlink;

        let root = temp_dir("agent-env-tree-cycle");
        fs::create_dir_all(root.join("cycle")).expect("cycle");
        symlink("..", root.join("cycle/loop")).expect("link");
        let message = TreeSnapshot::capture(&root).unwrap_err().to_string();
        assert!(message.contains("symlink cycle"), "{message}");
        let _ = fs::remove_dir_all(&root);
    }

    #[cfg(unix)]
    #[test]
    fn mode_only_change_changes_hash_and_copy_preserves_mode() {
        use std::os::unix::fs::PermissionsExt;

        let root = temp_dir("agent-env-tree-mode");
        fs::create_dir_all(&root).expect("root");
        let script = root.join("run.sh");
        fs::write(&script, "#!/bin/sh\n").expect("script");
        fs::set_permissions(&script, fs::Permissions::from_mode(0o644)).expect("mode");
        let before = TreeSnapshot::capture(&root).expect("before").hash();
        fs::set_permissions(&script, fs::Permissions::from_mode(0o755)).expect("mode");
        let snapshot = TreeSnapshot::capture(&root).expect("after");
        assert_ne!(before, snapshot.hash());

        let destination = temp_dir("agent-env-tree-mode-dst");
        snapshot.install_atomic(&destination).expect("install");
        let mode = fs::metadata(destination.join("run.sh"))
            .unwrap()
            .permissions()
            .mode();
        assert_eq!(mode & 0o777, 0o755);

        let _ = fs::remove_dir_all(&root);
        let _ = fs::remove_dir_all(&destination);
    }

    #[cfg(unix)]
    #[test]
    fn non_utf8_name_round_trips_and_participates_in_hash() {
        use std::os::unix::ffi::OsStringExt;

        let root = temp_dir("agent-env-tree-non-utf8");
        fs::create_dir_all(&root).expect("root");
        let raw_name = OsString::from_vec(vec![b'n', b'a', b'm', b'e', 0xff]);
        let source_file = root.join(&raw_name);
        fs::write(&source_file, "one").expect("write");
        let before = TreeSnapshot::capture(&root).expect("before");

        let destination = temp_dir("agent-env-tree-non-utf8-dst");
        before.install_atomic(&destination).expect("install");
        assert_eq!(fs::read(destination.join(&raw_name)).unwrap(), b"one");
        assert_eq!(before, TreeSnapshot::capture(&destination).unwrap());

        fs::write(&source_file, "two").expect("mutate");
        let after = TreeSnapshot::capture(&root).expect("after");
        assert_ne!(before.hash(), after.hash());

        let _ = fs::remove_dir_all(&root);
        let _ = fs::remove_dir_all(&destination);
    }

    #[cfg(unix)]
    #[test]
    fn special_entry_is_rejected() {
        use std::os::unix::net::UnixListener;

        let root = temp_dir("agent-env-tree-special");
        fs::create_dir_all(&root).expect("root");
        let _socket = UnixListener::bind(root.join("socket")).expect("socket");
        assert!(TreeSnapshot::capture(&root)
            .unwrap_err()
            .to_string()
            .contains("unsupported special"));
        let _ = fs::remove_dir_all(&root);
    }
}
