# KB sync runbook — envctl ⟷ meta

envctl is meta's env-manager agent; its git-kb knowledge base **adopts meta policy** and stays
in sync with the parent `meta` KB. This runbook is the **local-box** procedure for cross-KB sync.
It cannot run in an ephemeral remote clone (there is no parent `meta` checkout and no `git-kb`
CLI there) — which is exactly why the durable artifacts (the `.kb/store/` documents) are
**git-tracked** and travel in-repo, independent of any live pull.

## Durability model (why this works)

- `.kb/store/` is **git-tracked TEXT** (documents, commits, refs, `manifest.json`) — the durable
  source of truth. It survives clone/reclaim and is reviewed in PRs.
- `.kb/.cache/` (the redb index) and `workspaces/`/`stashes/` are `.gitignore`d — rebuildable /
  ephemeral.
- After any change to the tracked store (a pull, a vendored doc, a teammate's commit), rebuild
  the local index:
  ```bash
  git-kb reindex      # rebuild .kb/.cache from the tracked store
  git-kb verify       # check file-store integrity
  git-kb list --path context/
  ```

## One-time: wire the meta remote

git-kb sync is git-like push/pull replication over a remote (`http(s)://`, `gitkb://`).
Local two-KB sync uses meta's `git-kb serve`:

```bash
# In the meta checkout (parent), serve its KB:
( cd "$META_ROOT" && git-kb serve )            # HTTP server on localhost:<port>

# In envctl, register the remote (file:// is unconfirmed — prefer the served HTTP endpoint):
git-kb remote add meta http://localhost:<port>
git-kb remote list
git-kb sync status meta                         # health check, no transfer
```
(Alternative for an offline one-shot transfer: `git-kb bundle` in meta → apply the `.kbbundle`
in envctl.)

## Pull meta's KB — the RIGHT way (store-before → full pull → diff-after → reconcile)

Do the **full** pull. Do **not** cherry-pick / path-scope it (the repeating fail loop). The
safety net is backup + diff + reconcile, using git-kb's native primitives:

```bash
# 1. STORE BEFORE — full KB backup (documents + commits + stashes); restore = rollback
git-kb backup            # writes a backup file; note its path

# 2. FULL PULL — everything, no pathspec
git-kb pull meta

# 3. CONFLICTS are first-class — resolve, don't blindly overwrite
git-kb conflict          # list/inspect; resolve each
git-kb rebase --continue # or --abort

# 4. DIFF AFTER — see exactly what meta brought in
git-kb diff --commit <before>..<after> --json
git-kb log

# 5. RECONCILE — keep envctl's env-manager identity where it must differ
#    (e.g. context/immutable/project-brief stays envctl-specific), then:
git-kb commit -a -m "Sync from meta KB: <summary>"

# Escape hatch if the pull went wrong:
git-kb restore <backup-file>
```

Because the store is tracked, the reconciled result is then committed to git (durable) — it does
not evaporate.

## Vendored inheritance (no live pull required)

The meta policy envctl must follow is also vendored as durable docs so the inheritance holds even
before any sync: `reference/meta-kb-policy`, `reference/meta-org-policy` (authoritative sources:
`meta/.kb/AGENTS.md`, `meta/META-ORG-POLICY.md`).
