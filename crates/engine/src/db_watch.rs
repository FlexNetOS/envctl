//! db_watch — incremental index invalidation (REQ-057).
//!
//! `envctl db watch --repo-root` keeps a [`FileIndex`] fresh as files change,
//! invalidating **only** the changed rows rather than rebuilding the whole
//! index. The engine layer stays non-printing/sync (doctrine), so it exposes a
//! [`WatchState`] the CLI drives:
//!
//!   - [`WatchState::init`] takes the first snapshot.
//!   - [`WatchState::tick`] re-scans the scope, diffs it against the held
//!     snapshot by content hash (via [`FileIndex::diff_paths`]), swaps in the
//!     new snapshot, and returns an [`IndexDelta`] of what changed.
//!
//! **Poll fallback (documented, per the REQ-057 gate):** `tick()` *is* the poll
//! step — the CLI calls it on an interval. A `notify`/inotify watcher is an
//! optional acceleration layered on top in the CLI (REQ-059/060-gated dependency
//! decision); when the OS inotify watch limit is hit on a large tree, the CLI
//! degrades to calling `tick()` on a timer, which needs no per-file watches. The
//! engine core therefore has no OS-watch dependency and the no-C boundary holds.

use crate::db::Result;
use crate::db_index::{FileIndex, ScanScope};
use serde::{Deserialize, Serialize};

/// What changed between two index snapshots. Only these rows need re-derived
/// symbols; everything else is reused (incremental invalidation).
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct IndexDelta {
    pub added: Vec<String>,
    pub changed: Vec<String>,
    pub removed: Vec<String>,
    /// Rows carried over untouched (index_size - added - changed).
    pub unchanged: usize,
}

impl IndexDelta {
    /// True when nothing changed since the last tick (no re-index work).
    pub fn is_empty(&self) -> bool {
        self.added.is_empty() && self.changed.is_empty() && self.removed.is_empty()
    }

    /// Total number of invalidated (added + changed + removed) rows.
    pub fn invalidated(&self) -> usize {
        self.added.len() + self.changed.len() + self.removed.len()
    }
}

/// Holds the current scope + last index snapshot for incremental re-scans.
#[derive(Debug, Clone)]
pub struct WatchState {
    scope: ScanScope,
    index: FileIndex,
}

impl WatchState {
    /// Take the first snapshot of `scope`.
    pub fn init(scope: ScanScope) -> Result<Self> {
        let index = FileIndex::scan(&scope)?;
        Ok(Self { scope, index })
    }

    /// The current (most recent) index snapshot.
    pub fn index(&self) -> &FileIndex {
        &self.index
    }

    /// Re-scan the scope, compute the delta against the held snapshot, then
    /// adopt the new snapshot. Returns only the changed rows (the gate:
    /// "watch invalidates only changed file rows"). A no-op tick returns an
    /// empty [`IndexDelta`].
    pub fn tick(&mut self) -> Result<IndexDelta> {
        let next = FileIndex::scan(&self.scope)?;
        let (added, changed, removed) = next.diff_paths(&self.index);
        let unchanged = next
            .files()
            .len()
            .saturating_sub(added.len() + changed.len());
        self.index = next;
        Ok(IndexDelta {
            added,
            changed,
            removed,
            unchanged,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn tmp(tag: &str) -> std::path::PathBuf {
        let d = std::env::temp_dir().join(format!("envctl-db-watch-{tag}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&d);
        fs::create_dir_all(&d).unwrap();
        d
    }

    fn scope(root: &std::path::Path) -> ScanScope {
        ScanScope {
            root: root.display().to_string(),
            ..Default::default()
        }
    }

    #[test]
    fn tick_invalidates_only_changed_added_removed_rows() {
        let root = tmp("delta");
        fs::write(root.join("a.sh"), b"cd $META_ROOT\n").unwrap();
        fs::write(root.join("b.sh"), b"cd $META_ROOT/b\n").unwrap();

        let mut w = WatchState::init(scope(&root)).unwrap();
        assert_eq!(w.index().files().len(), 2);

        // No change -> empty delta, everything unchanged.
        let d0 = w.tick().unwrap();
        assert!(d0.is_empty(), "no fs change -> empty delta, got {d0:?}");
        assert_eq!(d0.unchanged, 2);

        // Modify a.sh, add c.sh, remove b.sh.
        fs::write(root.join("a.sh"), b"cd $LIFE_OS_ROOT\n").unwrap();
        fs::write(root.join("c.sh"), b"cd $META_ROOT/c\n").unwrap();
        fs::remove_file(root.join("b.sh")).unwrap();

        let d1 = w.tick().unwrap();
        let ends = |v: &[String], name: &str| v.iter().any(|p| p.ends_with(name));
        assert!(ends(&d1.changed, "a.sh"), "a.sh changed: {d1:?}");
        assert!(ends(&d1.added, "c.sh"), "c.sh added: {d1:?}");
        assert!(ends(&d1.removed, "b.sh"), "b.sh removed: {d1:?}");
        // Only a.sh + c.sh + b.sh are invalidated — nothing else churns.
        assert_eq!(d1.invalidated(), 3);
        // c.sh is the only carried-over unchanged row would be none here since
        // a changed and c added; unchanged counts rows neither added nor changed.
        assert_eq!(d1.unchanged, 0);

        // Idempotent: a second tick with no change is empty again.
        assert!(w.tick().unwrap().is_empty());

        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn diff_paths_is_deterministic_and_hash_based() {
        let root = tmp("hash");
        fs::write(root.join("x.sh"), b"same\n").unwrap();
        let a = FileIndex::scan(&scope(&root)).unwrap();
        // Rewrite identical content -> same hash -> not "changed".
        fs::write(root.join("x.sh"), b"same\n").unwrap();
        let b = FileIndex::scan(&scope(&root)).unwrap();
        let (added, changed, removed) = b.diff_paths(&a);
        assert!(added.is_empty() && changed.is_empty() && removed.is_empty());
        let _ = fs::remove_dir_all(&root);
    }
}
