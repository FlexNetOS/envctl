#!/usr/bin/env bash
# Proves scripts/meta-fleet-sync.py only mutates safe clean repos:
#   * clean behind-only repos are fast-forward pulled
#   * clean ahead-only repos are pushed
#   * dirty, diverged, no-upstream, and missing repos are only reported/skipped
set -euo pipefail

REPO_ROOT="$(git -C "$(dirname "${BASH_SOURCE[0]}")" rev-parse --show-toplevel)"
SYNC="$REPO_ROOT/scripts/meta-fleet-sync.py"
[ -x "$SYNC" ] || { echo "FAIL: $SYNC missing/not executable" >&2; exit 1; }
fail() { echo "FAIL: $*" >&2; exit 1; }

tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT

gitc() { git -c user.email=t@example.com -c user.name=test -c commit.gpgsign=false "$@"; }

make_remote() {
  local name="$1"
  git init -q --bare "$tmp/$name.git"
  gitc clone -q "$tmp/$name.git" "$tmp/$name-seed" 2>/dev/null
  (
    cd "$tmp/$name-seed"
    echo seed > f
    gitc add f
    gitc commit -qm seed
    gitc branch -M main
    gitc push -qu origin main
  )
  gitc -C "$tmp/$name.git" symbolic-ref HEAD refs/heads/main
}

clone_project() {
  local name="$1"
  gitc clone -q "$tmp/$name.git" "$tmp/meta/$name"
  gitc -C "$tmp/meta/$name" checkout -q main
}

advance_remote() {
  local name="$1" text="$2"
  gitc clone -q "$tmp/$name.git" "$tmp/$name-advance" 2>/dev/null
  (
    cd "$tmp/$name-advance"
    gitc checkout -q main
    echo "$text" >> f
    gitc commit -qam "$text"
    gitc push -q origin main
  )
  gitc -C "$tmp/meta/$name" fetch -q origin
}

advance_remote_without_local_fetch() {
  local name="$1" text="$2"
  gitc clone -q "$tmp/$name.git" "$tmp/$name-advance" 2>/dev/null
  (
    cd "$tmp/$name-advance"
    gitc checkout -q main
    echo "$text" >> f
    gitc commit -qam "$text"
    gitc push -q origin main
  )
}

mkdir -p "$tmp/meta"
gitc init -q "$tmp/meta"
(
  cd "$tmp/meta"
  echo root > README.md
  gitc add README.md
  gitc commit -qm root
)

for name in behind ahead dirty diverged noupstream; do
  make_remote "$name"
  clone_project "$name"
done

make_remote worktree
gitc clone -q "$tmp/worktree.git" "$tmp/worktree-owner"
gitc -C "$tmp/worktree-owner" checkout -q main
gitc -C "$tmp/worktree-owner" worktree add -q "$tmp/meta/worktree" -b worktree --track origin/main

# safe pull: local is clean and behind origin/main
advance_remote behind remote-advance

# safe push: local is clean and ahead origin/main
echo local >> "$tmp/meta/ahead/f"
gitc -C "$tmp/meta/ahead" commit -qam local-ahead

# dirty skip: local is behind and dirty; must not pull into dirty worktree
advance_remote dirty remote-dirty
echo dirty-local >> "$tmp/meta/dirty/untracked.txt"

# diverged skip: both local and remote have unique commits
advance_remote diverged remote-diverged
echo local-diverged >> "$tmp/meta/diverged/local.txt"
gitc -C "$tmp/meta/diverged" add local.txt
gitc -C "$tmp/meta/diverged" commit -qm local-diverged

# no-upstream skip
gitc -C "$tmp/meta/noupstream" branch --unset-upstream

# worktree fetch: a linked worktree must be fetched even though its .git is a file
advance_remote_without_local_fetch worktree remote-worktree

cat > "$tmp/projects.json" <<JSON
{
  "path": ".",
  "repo": null,
  "root": "$tmp/meta",
  "cwd": "$tmp/meta",
  "projects": [
    {"name": "behind", "path": "behind", "repo": "$tmp/behind.git"},
    {"name": "ahead", "path": "ahead", "repo": "$tmp/ahead.git"},
    {"name": "dirty", "path": "dirty", "repo": "$tmp/dirty.git"},
    {"name": "diverged", "path": "diverged", "repo": "$tmp/diverged.git"},
    {"name": "noupstream", "path": "noupstream", "repo": "$tmp/noupstream.git"},
    {"name": "worktree", "path": "worktree", "repo": "$tmp/worktree.git"},
    {"name": "missing", "path": "missing", "repo": "$tmp/missing.git"}
  ]
}
JSON

python3 "$SYNC" --meta-root "$tmp/meta" --project-list-json "$tmp/projects.json" --no-fetch --json > "$tmp/dry.json"
python3 - "$tmp/dry.json" <<'PY'
import json, sys
data=json.load(open(sys.argv[1]))
summary=data["summary"]
assert summary.get("safe_pull_ff") == 1, summary
assert summary.get("safe_push") == 1, summary
assert summary.get("dirty_skip") == 1, summary
assert summary.get("diverged_skip") == 1, summary
assert summary.get("no_upstream_skip") == 2, summary
assert summary.get("clean_synced") == 1, summary  # linked worktree stays invisible until fetched
assert summary.get("missing_skip") == 1, summary
root = next(repo for repo in data["repos"] if repo["path"] == ".")
assert root["ignored_managed_untracked_count"] == 6, root
assert root["dirty_count"] == 0, root
PY

python3 "$SYNC" --meta-root "$tmp/meta" --project-list-json "$tmp/projects.json" --fetch --json > "$tmp/fetched.json"
python3 - "$tmp/fetched.json" <<'PY'
import json, sys
data=json.load(open(sys.argv[1]))
summary=data["summary"]
assert summary.get("safe_pull_ff") == 2, summary  # behind + worktree
assert summary.get("safe_push") == 1, summary
assert "clean_synced" not in summary, summary
PY

make_remote brokenfetch
clone_project brokenfetch
gitc -C "$tmp/meta/brokenfetch" remote set-url origin "$tmp/does-not-exist.git"

cat > "$tmp/broken-projects.json" <<JSON
{
  "path": ".",
  "repo": null,
  "root": "$tmp/meta",
  "cwd": "$tmp/meta",
  "projects": [
    {"name": "brokenfetch", "path": "brokenfetch", "repo": "$tmp/brokenfetch.git"}
  ]
}
JSON

if python3 "$SYNC" --meta-root "$tmp/meta" --project-list-json "$tmp/broken-projects.json" --fetch --json > "$tmp/broken.json" 2> "$tmp/broken.err"; then
  fail "fetch failure unexpectedly succeeded"
fi
grep -q "fetch failed; refusing to classify or apply" "$tmp/broken.err" || fail "fetch failure did not fail closed"

if python3 "$SYNC" --meta-root "$tmp/meta" --project-list-json "$tmp/broken-projects.json" --apply --json > "$tmp/broken-apply.json" 2> "$tmp/broken-apply.err"; then
  fail "apply with fetch failure unexpectedly succeeded"
fi
grep -q "fetch failed; refusing to classify or apply" "$tmp/broken-apply.err" || fail "apply fetch failure did not fail closed"

behind_before="$(gitc -C "$tmp/meta/behind" rev-parse HEAD)"
python3 "$SYNC" --meta-root "$tmp/meta" --project-list-json "$tmp/projects.json" --no-fetch --apply --json > "$tmp/apply.json"
behind_after="$(gitc -C "$tmp/meta/behind" rev-parse HEAD)"
[ "$behind_before" != "$behind_after" ] || fail "behind repo was not fast-forward pulled"
[ "$behind_after" = "$(gitc -C "$tmp/meta/behind" rev-parse origin/main)" ] || fail "behind repo did not match origin/main"
[ "$(gitc -C "$tmp/ahead.git" rev-parse main)" = "$(gitc -C "$tmp/meta/ahead" rev-parse HEAD)" ] || fail "ahead repo was not pushed"
[ "$(gitc -C "$tmp/meta/worktree" rev-parse HEAD)" = "$(gitc -C "$tmp/meta/worktree" rev-parse origin/main)" ] || fail "worktree repo was not fetched/pulled"
[ -f "$tmp/meta/dirty/untracked.txt" ] || fail "dirty untracked file was lost"
[ "$(gitc -C "$tmp/meta/dirty" rev-parse HEAD)" != "$(gitc -C "$tmp/meta/dirty" rev-parse origin/main)" ] || fail "dirty repo was pulled despite dirty state"

echo "PASS: meta-fleet-sync safely pulled/pushed only clean eligible repos, including linked worktrees, and skipped risky states"
