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
//!   - [`DbWatcher`] uses the platform's native recursive watcher and persists
//!     a delta after each notification.
//!
//! **Poll fallback:** if native watcher creation, recursive registration, or a
//! later event fails (including Linux inotify `ENOSPC` watch-limit failures),
//! [`DbWatcher`] switches to bounded polling. Polling needs no per-directory
//! watches, so large trees remain observable without changing sysctls.

use crate::db::Result;
use crate::db_index::{DbIndexStore, FileIndex, ScanScope};
use notify::{RecommendedWatcher, RecursiveMode, Watcher};
use serde::{Deserialize, Serialize};
use std::sync::mpsc::{self, Receiver, RecvTimeoutError};
use std::time::Duration;

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

/// Active filesystem event source. `reason` is populated when native watching
/// could not be used and is suitable for operator-facing diagnostics.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WatchBackend {
    Notify,
    Poll { reason: String },
}

/// A synchronous, non-printing native watcher with a poll safety net.
///
/// The native watcher is held solely to keep its OS registration alive. Events
/// are coalesced into one content-hash delta, so duplicate/noisy notifications
/// never invalidate unchanged rows.
pub struct DbWatcher {
    scope: ScanScope,
    store: DbIndexStore,
    interval: Duration,
    backend: WatchBackend,
    events: Receiver<notify::Result<notify::Event>>,
    _watcher: Option<RecommendedWatcher>,
}

impl DbWatcher {
    /// Persist an initial baseline, then try to register a recursive native
    /// watcher. Any registration failure becomes a documented poll fallback.
    pub fn start(scope: ScanScope, store: DbIndexStore, interval: Duration) -> Result<Self> {
        let initial = FileIndex::scan(&scope)?;
        store.save(&initial)?;

        let (sender, events) = mpsc::channel();
        let mut watcher = match notify::recommended_watcher(sender) {
            Ok(watcher) => watcher,
            Err(error) => {
                return Ok(Self::polling(
                    scope,
                    store,
                    interval,
                    events,
                    error.to_string(),
                ));
            }
        };
        let roots = std::iter::once(scope.root.as_str())
            .chain(scope.extra_roots.iter().map(String::as_str));
        let registration = roots
            .map(std::path::Path::new)
            .try_for_each(|root| watcher.watch(root, RecursiveMode::Recursive));
        match registration {
            Ok(()) => Ok(Self {
                scope,
                store,
                interval,
                backend: WatchBackend::Notify,
                events,
                _watcher: Some(watcher),
            }),
            Err(error) => Ok(Self::polling(
                scope,
                store,
                interval,
                events,
                classify_notify_error(&error),
            )),
        }
    }

    fn polling(
        scope: ScanScope,
        store: DbIndexStore,
        interval: Duration,
        events: Receiver<notify::Result<notify::Event>>,
        reason: String,
    ) -> Self {
        Self {
            scope,
            store,
            interval,
            backend: WatchBackend::Poll { reason },
            events,
            _watcher: None,
        }
    }

    pub fn backend(&self) -> &WatchBackend {
        &self.backend
    }

    /// Wait for the next native event or poll deadline, then return only rows
    /// whose content/path membership changed since the persisted baseline.
    pub fn next_delta(&mut self) -> Result<IndexDelta> {
        if matches!(self.backend, WatchBackend::Notify) {
            loop {
                match self.events.recv_timeout(self.interval) {
                    Ok(Ok(event)) if is_relevant_event(&event) => break,
                    // Persisting the baseline itself produces events below
                    // `.envctl`; ignore them to avoid a self-triggered loop.
                    Ok(Ok(_)) => continue,
                    Err(RecvTimeoutError::Timeout) => break,
                    Ok(Err(error)) => {
                        self.backend = WatchBackend::Poll {
                            reason: classify_notify_error(&error),
                        };
                        self._watcher = None;
                        break;
                    }
                    Err(RecvTimeoutError::Disconnected) => {
                        self.backend = WatchBackend::Poll {
                            reason: "native watcher event channel disconnected".into(),
                        };
                        self._watcher = None;
                        break;
                    }
                }
            }
        } else {
            std::thread::sleep(self.interval);
        }
        poll_persisted(&self.scope, &self.store)
    }
}

fn is_relevant_event(event: &notify::Event) -> bool {
    event.paths.iter().any(|path| {
        !path
            .components()
            .any(|part| part.as_os_str() == crate::db_index::ENVCTL_STATE_DIR)
    })
}

fn classify_notify_error(error: &notify::Error) -> String {
    let message = error.to_string();
    let lower = message.to_ascii_lowercase();
    if lower.contains("no space left")
        || lower.contains("watch limit")
        || lower.contains("max files")
        || lower.contains("too many open files")
    {
        format!("native watcher limit reached; using poll fallback: {message}")
    } else {
        format!("native watcher unavailable; using poll fallback: {message}")
    }
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

/// One poll step against a **persisted** baseline: scan `scope`, diff it against
/// the index stored in `store` (by content hash, via [`FileIndex::diff_paths`]),
/// persist the fresh scan as the new baseline, and return the [`IndexDelta`].
///
/// This is the incremental-invalidation contract the CLI `envctl db watch` and
/// the GUI watch view both drive: only `added`/`changed`/`removed` rows are
/// invalidated; a missing baseline makes everything `added`. Because the fresh
/// scan is written back, successive polls report only what changed *since the
/// last poll* — the persisted index is the durable source of truth (NFR03/REQ-057).
pub fn poll_persisted(scope: &ScanScope, store: &DbIndexStore) -> Result<IndexDelta> {
    let current = FileIndex::scan(scope)?;
    let previous = if store.exists() {
        // A corrupt baseline degrades to "everything added" rather than failing
        // the poll — the fresh scan repairs it below.
        store.load().unwrap_or_default()
    } else {
        FileIndex::new()
    };
    let (added, changed, removed) = current.diff_paths(&previous);
    let unchanged = current
        .files()
        .len()
        .saturating_sub(added.len() + changed.len());
    store.save(&current)?;
    Ok(IndexDelta {
        added,
        changed,
        removed,
        unchanged,
    })
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
    fn poll_persisted_uses_dbindexstore_baseline_and_invalidates_only_changes() {
        let root = tmp("persist");
        fs::write(root.join("a.sh"), b"cd $META_ROOT\n").unwrap();
        fs::write(root.join("b.sh"), b"cd $META_ROOT/b\n").unwrap();
        let store = DbIndexStore::for_root(&root);

        // First poll: no baseline on disk yet -> everything is "added", and the
        // fresh scan is persisted (the durable index is now real — NFR03).
        assert!(!store.exists());
        let d0 = poll_persisted(&scope(&root), &store).unwrap();
        assert_eq!(d0.added.len(), 2, "no baseline -> all added: {d0:?}");
        assert!(d0.changed.is_empty() && d0.removed.is_empty());
        assert!(store.exists(), "poll must persist the fresh scan");

        // The persisted index round-trips (load == the scan just written).
        let reloaded = store.load().unwrap();
        assert_eq!(
            reloaded.files(),
            FileIndex::scan(&scope(&root)).unwrap().files()
        );

        // Second poll with NO change -> empty delta against the persisted baseline.
        let d1 = poll_persisted(&scope(&root), &store).unwrap();
        assert!(d1.is_empty(), "no fs change -> empty delta, got {d1:?}");

        // Mutate one, add one -> only those invalidate (baseline came from disk).
        fs::write(root.join("a.sh"), b"cd $LIFE_OS_ROOT\n").unwrap();
        fs::write(root.join("c.sh"), b"cd $META_ROOT/c\n").unwrap();
        let d2 = poll_persisted(&scope(&root), &store).unwrap();
        let ends = |v: &[String], name: &str| v.iter().any(|p| p.ends_with(name));
        assert!(ends(&d2.changed, "a.sh"), "a.sh changed: {d2:?}");
        assert!(ends(&d2.added, "c.sh"), "c.sh added: {d2:?}");
        assert_eq!(d2.invalidated(), 2);

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

    #[test]
    fn db_watcher_uses_native_notifications_and_filters_its_state_file() {
        let root = tmp("native");
        fs::write(root.join("a.sh"), b"before\n").unwrap();
        let store = DbIndexStore::for_root(&root);
        let mut watcher =
            DbWatcher::start(scope(&root), store, Duration::from_millis(500)).unwrap();
        assert_eq!(watcher.backend(), &WatchBackend::Notify);

        fs::write(root.join("a.sh"), b"after\n").unwrap();
        let delta = watcher.next_delta().unwrap();
        assert_eq!(delta.changed.len(), 1, "{delta:?}");
        assert!(delta.changed[0].ends_with("a.sh"));

        // The baseline save emits native events under `.envctl`. They are
        // ignored; the safety deadline returns an empty delta instead of a
        // self-triggered invalidation loop.
        assert!(watcher.next_delta().unwrap().is_empty());
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn inotify_limit_errors_are_operator_actionable() {
        let error = notify::Error::generic("No space left on device (os error 28)");
        let reason = classify_notify_error(&error);
        assert!(reason.contains("limit reached"));
        assert!(reason.contains("poll fallback"));
    }
}
