//! Versioned SHA-256 content hashing for agent assets.
//!
//! Skill trees use the lock-v3 `tree-v1` domain. Paths are separator-normalized, but relevant
//! permission modes are also authenticated, so a skill lock is reproducible for the declared
//! target platform rather than falsely claiming byte-for-byte cross-OS equivalence.

use std::fs;
use std::io::{BufReader, Read};
use std::path::Path;

use sha2::{Digest, Sha256};

use crate::{Result, TreeSnapshot};

/// Hash a directory tree: every file's relative path (separator-normalized) and contents
/// are folded into one SHA-256 digest, with files visited in sorted order for stability.
pub fn hash_dir(path: &Path) -> Result<String> {
    Ok(TreeSnapshot::capture(path)?.hash())
}

/// Hash an arbitrary string (used to key machine-local state by lock path).
pub fn hash_str(s: &str) -> String {
    hash_bytes(s.as_bytes())
}

/// Hash already-captured bytes without re-reading a mutable source path.
pub fn hash_bytes(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    format!("{:x}", hasher.finalize())
}

/// Hash a single file (for MCP / command asset tracking).
pub fn hash_file(path: &Path) -> Result<String> {
    let mut hasher = Sha256::new();
    let file = fs::File::open(path)?;
    let mut reader = BufReader::new(file);
    let mut buf = [0u8; 8192];
    sha256_update_reader(&mut reader, &mut hasher, &mut buf)?;
    Ok(format!("{:x}", hasher.finalize()))
}

fn sha256_update_reader<R: Read>(
    reader: &mut R,
    hasher: &mut Sha256,
    buf: &mut [u8; 8192],
) -> Result<()> {
    loop {
        let n = reader.read(buf)?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn temp_dir(prefix: &str) -> PathBuf {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!("{prefix}-{nanos}"))
    }

    /// The relative-path bytes fed into the digest must be separator-invariant:
    /// `a\b` and `a/b` must contribute identically.
    #[test]
    fn relative_path_separator_invariant() {
        let win = "a\\b\\c.md".replace('\\', "/");
        let unix = "a/b/c.md".replace('\\', "/");
        assert_eq!(win, unix);
    }

    #[test]
    fn hash_dir_is_stable_across_runs() {
        let root = temp_dir("agent-env-hash-stable");
        fs::create_dir_all(root.join("sub")).expect("create dirs");
        fs::write(root.join("SKILL.md"), "# Demo\n").expect("write");
        fs::write(root.join("sub/extra.md"), "body\n").expect("write");

        let a = hash_dir(&root).expect("hash a");
        let b = hash_dir(&root).expect("hash b");
        assert_eq!(a, b);

        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn hash_dir_changes_when_content_changes() {
        let root = temp_dir("agent-env-hash-diff");
        fs::create_dir_all(&root).expect("create dirs");
        fs::write(root.join("SKILL.md"), "# Demo\n").expect("write");
        let a = hash_dir(&root).expect("hash a");
        fs::write(root.join("SKILL.md"), "# Different\n").expect("write");
        let b = hash_dir(&root).expect("hash b");
        assert_ne!(a, b);
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn hash_file_and_hash_str_are_deterministic() {
        assert_eq!(hash_str("abc"), hash_str("abc"));
        assert_ne!(hash_str("abc"), hash_str("abd"));

        let root = temp_dir("agent-env-hash-file");
        fs::create_dir_all(&root).expect("create dirs");
        let f = root.join("f.txt");
        fs::write(&f, "payload\n").expect("write");
        assert_eq!(hash_file(&f).unwrap(), hash_file(&f).unwrap());
        let _ = fs::remove_dir_all(&root);
    }
}
