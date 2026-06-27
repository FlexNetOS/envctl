#!/usr/bin/env bash
# audit-meta-local-paths.sh — verify the meta-local install surface stays inside META_ROOT.
#
# Read-only by default.  With --apply, performs only conservative, reversible migrations:
#   * create/fix the single real-home .local -> $META_ROOT/.local bridge when the existing entry is
#     missing or already a symlink;
#   * repoint $META_ROOT/.local/bin/<name> symlinks that resolve outside META_ROOT only when an
#     executable replacement already exists under $META_ROOT/usr/bin or $META_ROOT/.toolchains/cargo/bin;
#   * relink real-home .gitconfig through $META_ROOT/.gitconfig, archiving a non-symlink first.
#
# It intentionally does not move credentials or broad real-home application state (.ssh, .config,
# .cache, .codex, .claude, .cargo, ...).  Instead it walks every top-level real-home dot entry and
# reports the owner-supervised migration work still required.
set -euo pipefail

usage() {
  cat <<'USAGE'
usage: scripts/audit-meta-local-paths.sh [--apply] [--meta-root PATH] [--real-home PATH] [--envctl-home-source PATH]

Audits $META_ROOT/.local, $META_ROOT/.toolchains, and every top-level real-home dot entry for path drift.
USAGE
}

APPLY=0
ROOT="$(git -C "$(dirname "${BASH_SOURCE[0]}")/.." rev-parse --show-toplevel 2>/dev/null || pwd)"
META_ROOT="${META_ROOT:-$(cd "$ROOT/.." && pwd)}"
REAL_HOME="${ENVCTL_REAL_HOME:-$HOME}"
ENVCTL_HOME_SOURCE="$ROOT/home"

while [ "$#" -gt 0 ]; do
  case "$1" in
    --apply) APPLY=1; shift ;;
    --meta-root) META_ROOT="${2:?--meta-root requires a path}"; shift 2 ;;
    --real-home) REAL_HOME="${2:?--real-home requires a path}"; shift 2 ;;
    --envctl-home-source) ENVCTL_HOME_SOURCE="${2:?--envctl-home-source requires a path}"; shift 2 ;;
    -h|--help) usage; exit 0 ;;
    *) echo "unknown argument: $1" >&2; usage >&2; exit 2 ;;
  esac
done

META_ROOT="$(cd "$META_ROOT" && pwd -P)"
REAL_HOME="$(cd "$REAL_HOME" && pwd -P)"
if [ -d "$ENVCTL_HOME_SOURCE" ]; then
  ENVCTL_HOME_SOURCE="$(cd "$ENVCTL_HOME_SOURCE" && pwd -P)"
fi

failures=0
warnings=0
changed=0
dot_entries_seen=0
stamp="$(date -u +%Y%m%dT%H%M%SZ)"
archive_dir="$META_ROOT/var/lib/envctl/real-home-dotfile-migration/$stamp"

say() { printf '%s\n' "$*"; }
fail() { failures=$((failures + 1)); say "FAIL: $*" >&2; }
warn() { warnings=$((warnings + 1)); say "WARN: $*" >&2; }
ok() { say "OK: $*"; }
changed_msg() { changed=$((changed + 1)); say "APPLY: $*"; }

is_under_meta() {
  case "$1" in
    "$META_ROOT"|"$META_ROOT"/*) return 0 ;;
    *) return 1 ;;
  esac
}

relocate_symlink() {
  local link="$1" target="$2"
  local dir rel
  dir="$(dirname "$link")"
  if command -v python3 >/dev/null 2>&1; then
    rel="$(python3 - <<PY
import os
print(os.path.relpath(${target@Q}, ${dir@Q}))
PY
)"
    ln -sfn "$rel" "$link"
  else
    ln -sfn "$target" "$link"
  fi
}

# 1. The only intentional real-home bridge is ~/.local -> $META_ROOT/.local.
mkdir -p "$META_ROOT/.local"
local_link="$REAL_HOME/.local"
if [ -L "$local_link" ]; then
  local_resolved="$(readlink -f "$local_link" 2>/dev/null || true)"
  if [ "$local_resolved" = "$META_ROOT/.local" ]; then
    ok "$local_link -> $META_ROOT/.local"
  elif [ "$APPLY" -eq 1 ]; then
    ln -sfn "$META_ROOT/.local" "$local_link"
    changed_msg "relinked $local_link -> $META_ROOT/.local"
  else
    fail "$local_link resolves to ${local_resolved:-<missing>}, expected $META_ROOT/.local"
  fi
elif [ -e "$local_link" ]; then
  fail "$local_link is not a symlink; refusing to move a real directory automatically"
elif [ "$APPLY" -eq 1 ]; then
  ln -sfn "$META_ROOT/.local" "$local_link"
  changed_msg "created $local_link -> $META_ROOT/.local"
else
  fail "$local_link missing; expected symlink to $META_ROOT/.local"
fi

# 2. No symlink under $META_ROOT/.local/bin may point outside META_ROOT.
if [ -d "$META_ROOT/.local/bin" ]; then
  while IFS= read -r -d '' link; do
    name="$(basename "$link")"
    resolved="$(readlink -f "$link" 2>/dev/null || true)"
    if [ -n "$resolved" ] && is_under_meta "$resolved"; then
      ok "$link resolves inside META_ROOT"
      continue
    fi

    replacement=""
    for candidate in "$META_ROOT/usr/bin/$name" "$META_ROOT/.toolchains/cargo/bin/$name"; do
      if [ -x "$candidate" ]; then
        replacement="$candidate"
        break
      fi
    done

    if [ "$APPLY" -eq 1 ] && [ -n "$replacement" ]; then
      relocate_symlink "$link" "$replacement"
      changed_msg "repointed $link -> $replacement"
    else
      fail "$link resolves outside META_ROOT (${resolved:-missing target}); no safe meta replacement was applied"
    fi
  done < <(find "$META_ROOT/.local/bin" -maxdepth 1 -type l -print0 | sort -z)
fi

# 3. Top-level .toolchains symlinks, if any, must also stay inside META_ROOT.
if [ -d "$META_ROOT/.toolchains" ]; then
  while IFS= read -r -d '' link; do
    resolved="$(readlink -f "$link" 2>/dev/null || true)"
    if [ -n "$resolved" ] && is_under_meta "$resolved"; then
      ok "$link resolves inside META_ROOT"
    else
      fail "$link resolves outside META_ROOT (${resolved:-missing target})"
    fi
  done < <(find "$META_ROOT/.toolchains" -maxdepth 1 -type l -print0 | sort -z)
fi

# 4. Managed real-home .gitconfig must bridge through $META_ROOT/.gitconfig, not directly to a checkout.
gitconfig_source="$ENVCTL_HOME_SOURCE/.gitconfig"
canonical_gitconfig="$META_ROOT/.gitconfig"
real_gitconfig="$REAL_HOME/.gitconfig"
if [ "$APPLY" -eq 1 ] && [ ! -e "$canonical_gitconfig" ] && [ -e "$gitconfig_source" ]; then
  relocate_symlink "$canonical_gitconfig" "$gitconfig_source"
  changed_msg "created canonical $canonical_gitconfig -> $gitconfig_source"
fi

if [ -L "$real_gitconfig" ]; then
  real_gitconfig_resolved="$(readlink -f "$real_gitconfig" 2>/dev/null || true)"
  canonical_resolved="$(readlink -f "$canonical_gitconfig" 2>/dev/null || true)"
  if [ -n "$canonical_resolved" ] && [ "$real_gitconfig_resolved" = "$canonical_resolved" ] && [ "$(readlink "$real_gitconfig")" = "$canonical_gitconfig" ]; then
    ok "$real_gitconfig bridges through $canonical_gitconfig"
  elif [ "$APPLY" -eq 1 ] && [ -e "$canonical_gitconfig" ]; then
    ln -sfn "$canonical_gitconfig" "$real_gitconfig"
    changed_msg "relinked $real_gitconfig -> $canonical_gitconfig"
  else
    fail "$real_gitconfig resolves to ${real_gitconfig_resolved:-<missing>}; expected symlink to $canonical_gitconfig"
  fi
elif [ -e "$real_gitconfig" ]; then
  if [ "$APPLY" -eq 1 ] && [ -e "$canonical_gitconfig" ]; then
    mkdir -p "$archive_dir"
    mv "$real_gitconfig" "$archive_dir/.gitconfig"
    ln -sfn "$canonical_gitconfig" "$real_gitconfig"
    changed_msg "archived real-home .gitconfig to $archive_dir/.gitconfig and linked $real_gitconfig -> $canonical_gitconfig"
  else
    fail "$real_gitconfig is a real file; use --apply to archive it and link through $canonical_gitconfig"
  fi
elif [ "$APPLY" -eq 1 ] && [ -e "$canonical_gitconfig" ]; then
  ln -sfn "$canonical_gitconfig" "$real_gitconfig"
  changed_msg "created $real_gitconfig -> $canonical_gitconfig"
else
  warn "$real_gitconfig missing; no credential helper bridge installed"
fi

# 5. Walk every top-level real-home dot entry.  The two entries that this script may safely mutate
# (.local and .gitconfig) are handled above; everything else is inventory-only here unless already
# bridged into META_ROOT.  This keeps the loop honest ("every dot file/folder was observed") without
# auto-moving credentials, caches, shell histories, language toolchains, or app state.
if [ -d "$REAL_HOME" ]; then
  while IFS= read -r -d '' path; do
    dot_entries_seen=$((dot_entries_seen + 1))
    dot="$(basename "$path")"
    case "$dot" in
      .|..|.local|.gitconfig) continue ;;
    esac

    if [ -L "$path" ]; then
      resolved="$(readlink -f "$path" 2>/dev/null || true)"
      if [ -n "$resolved" ] && is_under_meta "$resolved"; then
        ok "$path resolves inside META_ROOT"
      else
        warn "$path symlink resolves outside META_ROOT (${resolved:-missing target}); owner-supervised migration required"
      fi
    else
      if [ -e "$ENVCTL_HOME_SOURCE/$dot" ]; then
        warn "$path is real-home state outside META_ROOT with managed source $ENVCTL_HOME_SOURCE/$dot; skipped automatic move (owner-supervised bridge required)"
      else
        warn "$path is real-home state outside META_ROOT; skipped automatic move (credentials/cache/toolchain/app state require owner-supervised migration)"
      fi
    fi
  done < <(find "$REAL_HOME" -mindepth 1 -maxdepth 1 -name '.*' ! -name '.' ! -name '..' -print0 | sort -z)
fi

if [ "$failures" -gt 0 ]; then
  say "meta-local audit: FAIL failures=$failures warnings=$warnings changed=$changed dot_entries=$dot_entries_seen" >&2
  exit 1
fi
say "meta-local audit: PASS warnings=$warnings changed=$changed dot_entries=$dot_entries_seen"
