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
# It intentionally does not move credentials or broad real-home application state by default.
# Shell dotfiles are only canonicalized with the explicit --apply-shell-dotfiles opt-in; default
# --apply remains limited to the proven-safe bridges above plus explicitly requested allow-listed
# --migrate-dot entries.
# History/backup dot entries are only archived+bridged into META_ROOT with the explicit
# --apply-history-archives opt-in; default --apply remains non-mutating for them.
# Portable app configs are only migrated when they are explicitly allow-listed here.
# Explicit --migrate-dot requests are allow-listed, require --apply for mutation, and preserve an
# existing canonical META_ROOT target by archiving the old real-home state under META_ROOT first.
# Live migrations also require lsof proof that no process has open file handles below the source
# tree before any --apply move/archive/link mutation is attempted.
set -euo pipefail

usage() {
  cat <<'USAGE'
usage: scripts/audit-meta-local-paths.sh [--apply] [--apply-shell-dotfiles] [--apply-history-archives] [--shell-dotfile-conflict-report PATH] [--app-config-conflict-report PATH] [--unknown-app-config-report PATH] [--sensitive-state-report PATH] [--migration-blockers-report PATH] [--migration-blockers-summary PATH] [--inventory PATH] [--inventory-summary PATH] [--deep-link-inventory PATH] [--deep-link-summary PATH] [--fail-real-home-deep-links] [--migrate-dot DOT]... [--meta-root PATH] [--real-home PATH] [--envctl-home-source PATH]

Audits $META_ROOT/.local, $META_ROOT/.toolchains, and every top-level real-home dot entry for path drift.
With --inventory, also writes a tab-separated relocation inventory:
dot_entry, type, state, target_class, canonical_target, action, apply_safe.
With --inventory-summary, writes a tab-separated per-class migration summary:
target_class, total, apply_safe_yes, apply_safe_no, apply_safe_na, actions.
With --deep-link-inventory, recursively inventories symlinks below $META_ROOT/.local and
$META_ROOT/.toolchains:
scan_root, symlink, link_text, resolved_target, target_class, action.
With --deep-link-summary, writes a per-class recursive symlink summary:
target_class, total, actions.
Recursive deep-link inventory is report-only by default because toolchains, venvs, and container
stores legitimately contain embedded absolute system links and missing internal links.  Add
--fail-real-home-deep-links to fail if any recursive link resolves back into the real home outside
META_ROOT.
With --migrate-dot, performs an explicit owner-requested migration for allow-listed entries only
(known toolchain state, known agent/app config state including portable app-config files
like .ideavimrc, portable app-config dirs like .gphoto/.vscode-shared/.archon/.n8n-mcp/.n8n/.n8n-claude-bridge/.pki/.forge/.ruvector/.hermes/.ai/.jetbrains/.meta/.java/.repowire,
portable cache dirs like .nv, or a managed dotfile present under --envctl-home-source).
Mutation still requires --apply; without --apply the script prints the planned move and changes nothing.
With --apply, migrations require lsof and refuse to mutate while any process has open file handles
below the source tree.
With --shell-dotfile-conflict-report, writes supervised shell-dotfile merge rows:
dot_entry, real_path, canonical_target, action, apply_safe, real_sha256, canonical_sha256, real_lines, canonical_lines, recommendation.
With --app-config-conflict-report, writes supervised app-config merge rows when a known real-home
app-config entry has an existing canonical META_ROOT target:
dot_entry, real_path, canonical_target, action, apply_safe, real_type, canonical_type,
real_digest, canonical_digest, real_entries, canonical_entries, recommendation.
With --unknown-app-config-report, writes read-only classification rows for app-config-state entries
that do not yet have a canonical META_ROOT target:
dot_entry, real_path, type, digest, entries, direct_files, direct_dirs, symlinks,
sensitive_hints, recommendation.
With --sensitive-state-report, writes read-only metadata rows for sensitive real-home entries:
dot_entry, real_path, type, digest, entries, direct_files, direct_dirs, symlinks,
sensitive_hints, action, apply_safe, recommendation.
With --migration-blockers-report, writes read-only residual blocker rows for real-home dot entries
that are not already bridged into META_ROOT:
dot_entry, real_path, type, target_class, action, apply_safe, canonical_target, blocker,
blocker_detail, open_handles, open_handle_sample, recommendation.
With --migration-blockers-summary, writes read-only per-blocker residual counts:
blocker, total, apply_safe_yes, apply_safe_no, open_handles, recommendations.
With --apply-history-archives and --apply, moves history/backup dot entries under
$META_ROOT/var/lib/envctl/real-home-dotfile-migration/history-or-backup/<dot-entry> and leaves
the original real-home path as a symlink bridge. Existing non-identical canonical archive targets
are left untouched for owner-supervised merge.
USAGE
}

APPLY=0
APPLY_SHELL_DOTFILES=0
APPLY_HISTORY_ARCHIVES=0
ROOT="$(git -C "$(dirname "${BASH_SOURCE[0]}")/.." rev-parse --show-toplevel 2>/dev/null || pwd)"
META_ROOT="${META_ROOT:-$(cd "$ROOT/.." && pwd)}"
REAL_HOME="${ENVCTL_REAL_HOME:-$HOME}"
ENVCTL_HOME_SOURCE="$ROOT/home"
INVENTORY_PATH=""
INVENTORY_SUMMARY_PATH=""
SHELL_DOTFILE_CONFLICT_REPORT_PATH=""
APP_CONFIG_CONFLICT_REPORT_PATH=""
UNKNOWN_APP_CONFIG_REPORT_PATH=""
SENSITIVE_STATE_REPORT_PATH=""
MIGRATION_BLOCKERS_REPORT_PATH=""
MIGRATION_BLOCKERS_SUMMARY_PATH=""
DEEP_LINK_INVENTORY_PATH=""
DEEP_LINK_SUMMARY_PATH=""
FAIL_REAL_HOME_DEEP_LINKS=0
MIGRATE_DOTS=()

while [ "$#" -gt 0 ]; do
  case "$1" in
    --apply) APPLY=1; shift ;;
    --apply-shell-dotfiles) APPLY_SHELL_DOTFILES=1; shift ;;
    --apply-history-archives) APPLY_HISTORY_ARCHIVES=1; shift ;;
    --inventory) INVENTORY_PATH="${2:?--inventory requires a path}"; shift 2 ;;
    --inventory-summary) INVENTORY_SUMMARY_PATH="${2:?--inventory-summary requires a path}"; shift 2 ;;
    --shell-dotfile-conflict-report) SHELL_DOTFILE_CONFLICT_REPORT_PATH="${2:?--shell-dotfile-conflict-report requires a path}"; shift 2 ;;
    --app-config-conflict-report) APP_CONFIG_CONFLICT_REPORT_PATH="${2:?--app-config-conflict-report requires a path}"; shift 2 ;;
    --unknown-app-config-report) UNKNOWN_APP_CONFIG_REPORT_PATH="${2:?--unknown-app-config-report requires a path}"; shift 2 ;;
    --sensitive-state-report) SENSITIVE_STATE_REPORT_PATH="${2:?--sensitive-state-report requires a path}"; shift 2 ;;
    --migration-blockers-report) MIGRATION_BLOCKERS_REPORT_PATH="${2:?--migration-blockers-report requires a path}"; shift 2 ;;
    --migration-blockers-summary) MIGRATION_BLOCKERS_SUMMARY_PATH="${2:?--migration-blockers-summary requires a path}"; shift 2 ;;
    --deep-link-inventory) DEEP_LINK_INVENTORY_PATH="${2:?--deep-link-inventory requires a path}"; shift 2 ;;
    --deep-link-summary) DEEP_LINK_SUMMARY_PATH="${2:?--deep-link-summary requires a path}"; shift 2 ;;
    --fail-real-home-deep-links) FAIL_REAL_HOME_DEEP_LINKS=1; shift ;;
    --migrate-dot) MIGRATE_DOTS+=("${2:?--migrate-dot requires a dot entry}"); shift 2 ;;
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
history_archive_root="$META_ROOT/var/lib/envctl/real-home-dotfile-migration/history-or-backup"

if [ -n "$INVENTORY_PATH" ]; then
  mkdir -p "$(dirname "$INVENTORY_PATH")"
  printf 'dot_entry\ttype\tstate\ttarget_class\tcanonical_target\taction\tapply_safe\n' >"$INVENTORY_PATH"
fi
if [ -n "$INVENTORY_SUMMARY_PATH" ]; then
  mkdir -p "$(dirname "$INVENTORY_SUMMARY_PATH")"
fi
if [ -n "$SHELL_DOTFILE_CONFLICT_REPORT_PATH" ]; then
  mkdir -p "$(dirname "$SHELL_DOTFILE_CONFLICT_REPORT_PATH")"
  printf 'dot_entry\treal_path\tcanonical_target\taction\tapply_safe\treal_sha256\tcanonical_sha256\treal_lines\tcanonical_lines\trecommendation\n' >"$SHELL_DOTFILE_CONFLICT_REPORT_PATH"
fi
if [ -n "$APP_CONFIG_CONFLICT_REPORT_PATH" ]; then
  mkdir -p "$(dirname "$APP_CONFIG_CONFLICT_REPORT_PATH")"
  printf 'dot_entry\treal_path\tcanonical_target\taction\tapply_safe\treal_type\tcanonical_type\treal_digest\tcanonical_digest\treal_entries\tcanonical_entries\trecommendation\n' >"$APP_CONFIG_CONFLICT_REPORT_PATH"
fi
if [ -n "$UNKNOWN_APP_CONFIG_REPORT_PATH" ]; then
  mkdir -p "$(dirname "$UNKNOWN_APP_CONFIG_REPORT_PATH")"
  printf 'dot_entry\treal_path\ttype\tdigest\tentries\tdirect_files\tdirect_dirs\tsymlinks\tsensitive_hints\trecommendation\n' >"$UNKNOWN_APP_CONFIG_REPORT_PATH"
fi
if [ -n "$SENSITIVE_STATE_REPORT_PATH" ]; then
  mkdir -p "$(dirname "$SENSITIVE_STATE_REPORT_PATH")"
  printf 'dot_entry\treal_path\ttype\tdigest\tentries\tdirect_files\tdirect_dirs\tsymlinks\tsensitive_hints\taction\tapply_safe\trecommendation\n' >"$SENSITIVE_STATE_REPORT_PATH"
fi
if [ -n "$MIGRATION_BLOCKERS_REPORT_PATH" ]; then
  mkdir -p "$(dirname "$MIGRATION_BLOCKERS_REPORT_PATH")"
  printf 'dot_entry\treal_path\ttype\ttarget_class\taction\tapply_safe\tcanonical_target\tblocker\tblocker_detail\topen_handles\topen_handle_sample\trecommendation\n' >"$MIGRATION_BLOCKERS_REPORT_PATH"
fi
if [ -n "$MIGRATION_BLOCKERS_SUMMARY_PATH" ]; then
  mkdir -p "$(dirname "$MIGRATION_BLOCKERS_SUMMARY_PATH")"
fi
if [ -n "$DEEP_LINK_INVENTORY_PATH" ]; then
  mkdir -p "$(dirname "$DEEP_LINK_INVENTORY_PATH")"
  printf 'scan_root\tsymlink\tlink_text\tresolved_target\ttarget_class\taction\n' >"$DEEP_LINK_INVENTORY_PATH"
fi
if [ -n "$DEEP_LINK_SUMMARY_PATH" ]; then
  mkdir -p "$(dirname "$DEEP_LINK_SUMMARY_PATH")"
fi

declare -A summary_total=()
declare -A summary_apply_yes=()
declare -A summary_apply_no=()
declare -A summary_apply_na=()
declare -A summary_actions=()
declare -A deep_link_total=()
declare -A deep_link_actions=()
declare -A migration_blocker_total=()
declare -A migration_blocker_apply_yes=()
declare -A migration_blocker_apply_no=()
declare -A migration_blocker_open_handles=()
declare -A migration_blocker_recommendations=()

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

entry_type() {
  local path="$1"
  if [ -L "$path" ]; then
    printf 'symlink'
  elif [ -d "$path" ]; then
    printf 'directory'
  elif [ -f "$path" ]; then
    printf 'file'
  elif [ -e "$path" ]; then
    printf 'other'
  else
    printf 'missing'
  fi
}

require_no_open_handles_for_migration() {
  local dot="$1" source="$2" lsof_out count sample

  if ! command -v lsof >/dev/null 2>&1; then
    fail "--migrate-dot $dot: lsof is unavailable; refusing automatic migration without open-handle proof"
    return 1
  fi

  lsof_out="$(mktemp "${TMPDIR:-/tmp}/envctl-lsof.XXXXXX")"
  if [ -d "$source" ]; then
    if lsof +D "$source" >"$lsof_out" 2>/dev/null; then
      :
    else
      :
    fi
  else
    if lsof "$source" >"$lsof_out" 2>/dev/null; then
      :
    else
      :
    fi
  fi

  count="$(awk 'NR > 1 && NF > 0 { count++ } END { print count + 0 }' "$lsof_out")"
  if [ "$count" -gt 0 ]; then
    sample="$(awk 'NR == 2 && NF >= 2 { print $1 "/" $2; exit }' "$lsof_out")"
    fail "--migrate-dot $dot: $count open file handle(s) under $source${sample:+ ($sample)}; close owning processes before migration"
    sed 's/^/  /' "$lsof_out" >&2
    rm -f "$lsof_out"
    return 1
  fi

  rm -f "$lsof_out"
  return 0
}

open_handle_report_for_path() {
  local source="$1" lsof_out count sample

  if ! command -v lsof >/dev/null 2>&1; then
    printf 'unknown\tlsof-unavailable\n'
    return 0
  fi

  lsof_out="$(mktemp "${TMPDIR:-/tmp}/envctl-lsof-report.XXXXXX")"
  if [ -d "$source" ]; then
    if lsof +D "$source" >"$lsof_out" 2>/dev/null; then
      :
    else
      :
    fi
  else
    if lsof "$source" >"$lsof_out" 2>/dev/null; then
      :
    else
      :
    fi
  fi

  count="$(awk 'NR > 1 && NF > 0 { count++ } END { print count + 0 }' "$lsof_out")"
  sample="$(awk 'NR == 2 && NF >= 2 { print $1 "/" $2; exit }' "$lsof_out")"
  rm -f "$lsof_out"
  printf '%s\t%s\n' "$count" "$sample"
}

is_shell_dotfile() {
  case "$1" in
    .bashrc|.profile|.zshrc|.zshenv|.bash_profile|.bash_logout) return 0 ;;
    *) return 1 ;;
  esac
}


file_sha256() {
  local path="$1"
  if command -v sha256sum >/dev/null 2>&1; then
    sha256sum "$path" | awk '{print $1}'
  elif command -v shasum >/dev/null 2>&1; then
    shasum -a 256 "$path" | awk '{print $1}'
  else
    printf 'unknown'
  fi
}

sha256_stdin() {
  if command -v sha256sum >/dev/null 2>&1; then
    sha256sum | awk '{print $1}'
  elif command -v shasum >/dev/null 2>&1; then
    shasum -a 256 | awk '{print $1}'
  else
    cat >/dev/null
    printf 'unknown'
  fi
}

file_line_count() {
  local path="$1"
  wc -l <"$path" | tr -d '[:space:]'
}

path_digest() {
  local path="$1" rel link_text

  if [ -L "$path" ]; then
    link_text="$(readlink "$path" 2>/dev/null || true)"
    printf 'L\t%s\n' "$link_text" | sha256_stdin
  elif [ -f "$path" ]; then
    file_sha256 "$path"
  elif [ -d "$path" ]; then
    (
      cd "$path"
      while IFS= read -r -d '' rel; do
        rel="${rel#./}"
        if [ -L "$rel" ]; then
          link_text="$(readlink "$rel" 2>/dev/null || true)"
          printf 'L\t%s\t%s\n' "$rel" "$link_text"
        elif [ -f "$rel" ]; then
          printf 'F\t%s\t%s\n' "$rel" "$(file_sha256 "$rel")"
        elif [ -d "$rel" ]; then
          printf 'D\t%s\n' "$rel"
        else
          printf 'O\t%s\n' "$rel"
        fi
      done < <(find . -mindepth 1 -print0 | LC_ALL=C sort -z)
    ) | sha256_stdin
  elif [ -e "$path" ]; then
    printf 'O\t%s\n' "$(entry_type "$path")" | sha256_stdin
  else
    printf 'missing'
  fi
}

path_entry_count() {
  local path="$1"
  if [ -L "$path" ] || [ -f "$path" ]; then
    printf '1'
  elif [ -d "$path" ]; then
    find "$path" -mindepth 1 -printf . 2>/dev/null | wc -c | tr -d '[:space:]'
  elif [ -e "$path" ]; then
    printf '1'
  else
    printf '0'
  fi
}

path_direct_file_count() {
  local path="$1"
  if [ -f "$path" ] && [ ! -L "$path" ]; then
    printf '1'
  elif [ -d "$path" ] && [ ! -L "$path" ]; then
    find "$path" -mindepth 1 -maxdepth 1 -type f -printf . 2>/dev/null | wc -c | tr -d '[:space:]'
  else
    printf '0'
  fi
}

path_direct_dir_count() {
  local path="$1"
  if [ -d "$path" ] && [ ! -L "$path" ]; then
    find "$path" -mindepth 1 -maxdepth 1 -type d -printf . 2>/dev/null | wc -c | tr -d '[:space:]'
  else
    printf '0'
  fi
}

path_symlink_count() {
  local path="$1"
  if [ -L "$path" ]; then
    printf '1'
  elif [ -d "$path" ]; then
    find "$path" -type l -printf . 2>/dev/null | wc -c | tr -d '[:space:]'
  else
    printf '0'
  fi
}

path_sensitive_hint_count() {
  local path="$1" name
  if [ ! -e "$path" ] && [ ! -L "$path" ]; then
    printf '0'
  elif [ -d "$path" ] && [ ! -L "$path" ]; then
    find "$path" -mindepth 1 \
      \( -iname '*token*' \
        -o -iname '*secret*' \
        -o -iname '*credential*' \
        -o -iname '*apikey*' \
        -o -iname '*api-key*' \
        -o -iname '*private-key*' \
        -o -iname '*.pem' \
        -o -iname '*.key' \
        -o -iname 'key4.db' \
        -o -iname 'cert9.db' \
        -o -iname 'pkcs11.txt' \
        -o -iname '*.p12' \
        -o -iname '*.pfx' \
        -o -iname 'id_rsa' \
        -o -iname 'id_ed25519' \) \
      -printf . 2>/dev/null | wc -c | tr -d '[:space:]'
  else
    name="$(basename "$path")"
    case "${name,,}" in
      *token*|*secret*|*credential*|*apikey*|*api-key*|*private-key*|*.pem|*.key|key4.db|cert9.db|pkcs11.txt|*.p12|*.pfx|id_rsa|id_ed25519) printf '1' ;;
      *) printf '0' ;;
    esac
  fi
}

record_shell_dotfile_conflict() {
  local dot="$1" path="$2" canonical_target="$3" action="$4" apply_safe="$5"
  [ -n "$SHELL_DOTFILE_CONFLICT_REPORT_PATH" ] || return 0
  [ -f "$path" ] || return 0
  [ -f "$canonical_target" ] || return 0

  printf '%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\n' \
    "$dot" \
    "$path" \
    "$canonical_target" \
    "$action" \
    "$apply_safe" \
    "$(file_sha256 "$path")" \
    "$(file_sha256 "$canonical_target")" \
    "$(file_line_count "$path")" \
    "$(file_line_count "$canonical_target")" \
    "merge-canonical-then-bridge" >>"$SHELL_DOTFILE_CONFLICT_REPORT_PATH"
}

record_app_config_conflict() {
  local dot="$1" path="$2" canonical_target="$3" action="$4" apply_safe="$5"
  [ -n "$APP_CONFIG_CONFLICT_REPORT_PATH" ] || return 0
  [ -n "$canonical_target" ] || return 0
  { [ -e "$path" ] || [ -L "$path" ]; } || return 0
  { [ -e "$canonical_target" ] || [ -L "$canonical_target" ]; } || return 0

  printf '%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\n' \
    "$dot" \
    "$path" \
    "$canonical_target" \
    "$action" \
    "$apply_safe" \
    "$(entry_type "$path")" \
    "$(entry_type "$canonical_target")" \
    "$(path_digest "$path")" \
    "$(path_digest "$canonical_target")" \
    "$(path_entry_count "$path")" \
    "$(path_entry_count "$canonical_target")" \
    "merge-canonical-then-bridge" >>"$APP_CONFIG_CONFLICT_REPORT_PATH"
}

record_unknown_app_config() {
  local dot="$1" path="$2" type="$3"
  [ -n "$UNKNOWN_APP_CONFIG_REPORT_PATH" ] || return 0
  { [ -e "$path" ] || [ -L "$path" ]; } || return 0

  printf '%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\n' \
    "$dot" \
    "$path" \
    "$type" \
    "$(path_digest "$path")" \
    "$(path_entry_count "$path")" \
    "$(path_direct_file_count "$path")" \
    "$(path_direct_dir_count "$path")" \
    "$(path_symlink_count "$path")" \
    "$(path_sensitive_hint_count "$path")" \
    "classify-canonical-target-before-migration" >>"$UNKNOWN_APP_CONFIG_REPORT_PATH"
}

record_sensitive_state() {
  local dot="$1" path="$2" type="$3" action="$4" apply_safe="$5"
  [ -n "$SENSITIVE_STATE_REPORT_PATH" ] || return 0
  { [ -e "$path" ] || [ -L "$path" ]; } || return 0

  printf '%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\n' \
    "$dot" \
    "$path" \
    "$type" \
    "$(path_digest "$path")" \
    "$(path_entry_count "$path")" \
    "$(path_direct_file_count "$path")" \
    "$(path_direct_dir_count "$path")" \
    "$(path_symlink_count "$path")" \
    "$(path_sensitive_hint_count "$path")" \
    "$action" \
    "$apply_safe" \
    "owner-supervised-vault-or-bridge-before-migration" >>"$SENSITIVE_STATE_REPORT_PATH"
}

record_migration_blocker() {
  local dot="$1" type="$2" state="$3" target_class="$4" canonical_target="$5" action="$6" apply_safe="$7"
  local path blocker blocker_detail open_handles open_handle_sample recommendation

  [ -n "$MIGRATION_BLOCKERS_REPORT_PATH" ] || [ -n "$MIGRATION_BLOCKERS_SUMMARY_PATH" ] || return 0
  [ "$state" = "real-home-state" ] || [ "$state" = "external-symlink" ] || return 0

  path="$REAL_HOME/$dot"
  { [ -e "$path" ] || [ -L "$path" ]; } || return 0

  blocker="owner-supervised"
  blocker_detail="$action"
  open_handles="n/a"
  open_handle_sample=""
  recommendation="$action"

  if [ "$apply_safe" = "yes" ]; then
    IFS=$'\t' read -r open_handles open_handle_sample < <(open_handle_report_for_path "$path")
    if [ "$open_handles" = "unknown" ]; then
      blocker="needs-open-handle-proof"
      blocker_detail="lsof-unavailable"
      recommendation="install-lsof-or-run-with-lsof-before-apply"
    elif [ "$open_handles" -gt 0 ]; then
      blocker="open-handles"
      blocker_detail="open-handles-present"
      recommendation="close-processes-then-run-apply-migrate-dot"
    else
      blocker="ready-for-explicit-migration"
      blocker_detail="no-open-handles-observed"
      recommendation="run-apply-migrate-dot"
    fi
  else
    case "$target_class" in
      sensitive)
        blocker="owner-supervised-sensitive"
        blocker_detail="credential-or-private-state"
        recommendation="owner-supervised-vault-or-bridge"
        ;;
      cache)
        blocker="owner-supervised-cache"
        blocker_detail="component-managed-cache-migration"
        recommendation="use-component-managed-cache-migration"
        ;;
      managed-dotfile)
        blocker="owner-supervised-managed-dotfile"
        blocker_detail="$action"
        recommendation="owner-review-before-bridge"
        ;;
      shell-dotfile)
        blocker="owner-supervised-shell-dotfile"
        blocker_detail="$action"
        recommendation="merge-canonical-then-bridge"
        ;;
      app-config-state)
        blocker="owner-supervised-app-config"
        blocker_detail="$action"
        recommendation="classify-or-migrate-via-explicit-migrate-dot"
        ;;
      toolchain-state)
        blocker="owner-supervised-toolchain-state"
        blocker_detail="$action"
        recommendation="use-component-managed-toolchain-migration"
        ;;
      external-symlink)
        blocker="owner-supervised-external-symlink"
        blocker_detail="$action"
        recommendation="relink-to-meta-local-target"
        ;;
    esac
  fi

  migration_blocker_observe "$blocker" "$apply_safe" "$open_handles" "$recommendation"

  if [ -n "$MIGRATION_BLOCKERS_REPORT_PATH" ]; then
    printf '%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\n' \
      "$dot" \
      "$path" \
      "$type" \
      "$target_class" \
      "$action" \
      "$apply_safe" \
      "$canonical_target" \
      "$blocker" \
      "$blocker_detail" \
      "$open_handles" \
      "$open_handle_sample" \
      "$recommendation" >>"$MIGRATION_BLOCKERS_REPORT_PATH"
  fi
}

shell_dotfile_action() {
  local path="$1" canonical_target="$2"
  if [ -e "$canonical_target" ]; then
    if [ -f "$path" ] && [ -f "$canonical_target" ] && cmp -s "$path" "$canonical_target"; then
      printf 'bridge-canonical\tyes\n'
    else
      printf 'owner-supervised-merge-and-bridge\tno\n'
    fi
  else
    printf 'move-to-canonical-and-bridge\tyes\n'
  fi
}

is_history_or_backup_dot() {
  case "$1" in
    .bash_history|.zsh_history|.*_history|*.bak|*.bak.*|*.backup|*.backup.*) return 0 ;;
    *) return 1 ;;
  esac
}


history_archive_target_for_dot() {
  local dot="$1"
  printf '%s/%s\n' "$history_archive_root" "$dot"
}

apply_shell_dotfile_bridge() {
  local dot="$1" path="$2" canonical_target="$3"
  local action apply_safe

  [ "$APPLY" -eq 1 ] || return 0
  [ "$APPLY_SHELL_DOTFILES" -eq 1 ] || return 0
  [ -e "$path" ] || return 0
  [ ! -L "$path" ] || return 0

  IFS=$'\t' read -r action apply_safe < <(shell_dotfile_action "$path" "$canonical_target")
  [ "$apply_safe" = "yes" ] || {
    warn "$path differs from canonical $canonical_target; owner-supervised merge required"
    return 0
  }

  if [ "$action" = "move-to-canonical-and-bridge" ]; then
    mkdir -p "$(dirname "$canonical_target")"
    mv "$path" "$canonical_target"
    ln -sfn "$canonical_target" "$path"
    changed_msg "moved $path to $canonical_target and linked $path -> $canonical_target"
  elif [ "$action" = "bridge-canonical" ]; then
    mkdir -p "$archive_dir"
    mv "$path" "$archive_dir/$dot"
    ln -sfn "$canonical_target" "$path"
    changed_msg "archived duplicate $path to $archive_dir/$dot and linked $path -> $canonical_target"
  fi
}

apply_history_archive_bridge() {
  local dot="$1" path="$2" target duplicate_archive

  [ "$APPLY" -eq 1 ] || return 0
  [ "$APPLY_HISTORY_ARCHIVES" -eq 1 ] || return 0
  [ -e "$path" ] || [ -L "$path" ] || return 0
  [ ! -L "$path" ] || return 0

  target="$(history_archive_target_for_dot "$dot")"

  if [ -e "$target" ] || [ -L "$target" ]; then
    if { [ -f "$path" ] && [ -f "$target" ] && cmp -s "$path" "$target"; } ||
      { [ -d "$path" ] && [ -d "$target" ] && diff -qr "$path" "$target" >/dev/null 2>&1; }; then
      duplicate_archive="$archive_dir/history-or-backup"
      mkdir -p "$duplicate_archive"
      mv "$path" "$duplicate_archive/$dot"
      ln -sfn "$target" "$path"
      changed_msg "archived duplicate $path to $duplicate_archive/$dot and linked $path -> $target"
    else
      warn "$path cannot be archived automatically: $target already exists and differs; owner-supervised merge required"
    fi
    return 0
  fi

  mkdir -p "$(dirname "$target")"
  mv "$path" "$target"
  ln -sfn "$target" "$path"
  changed_msg "archived $path to $target and linked $path -> $target"
}


inventory_row() {
  local dot="$1" type="$2" state="$3" target_class="$4" canonical_target="$5" action="$6" apply_safe="$7"
  if [ -n "$INVENTORY_PATH" ]; then
    printf '%s\t%s\t%s\t%s\t%s\t%s\t%s\n' "$dot" "$type" "$state" "$target_class" "$canonical_target" "$action" "$apply_safe" >>"$INVENTORY_PATH"
  fi
  record_migration_blocker "$dot" "$type" "$state" "$target_class" "$canonical_target" "$action" "$apply_safe"
  summary_observe "$target_class" "$action" "$apply_safe"
}

summary_observe() {
  local target_class="$1" action="$2" apply_safe="$3" existing_actions
  [ -n "$INVENTORY_SUMMARY_PATH" ] || return 0

  summary_total["$target_class"]=$(( ${summary_total["$target_class"]:-0} + 1 ))
  case "$apply_safe" in
    yes) summary_apply_yes["$target_class"]=$(( ${summary_apply_yes["$target_class"]:-0} + 1 )) ;;
    no) summary_apply_no["$target_class"]=$(( ${summary_apply_no["$target_class"]:-0} + 1 )) ;;
    n/a) summary_apply_na["$target_class"]=$(( ${summary_apply_na["$target_class"]:-0} + 1 )) ;;
  esac

  existing_actions="${summary_actions["$target_class"]:-}"
  if [ -z "$existing_actions" ]; then
    summary_actions["$target_class"]="$action"
  else
    case ",$existing_actions," in
      *,"$action",*) ;;
      *) summary_actions["$target_class"]="$existing_actions,$action" ;;
    esac
  fi
}

emit_inventory_summary() {
  [ -n "$INVENTORY_SUMMARY_PATH" ] || return 0

  {
    printf 'target_class\ttotal\tapply_safe_yes\tapply_safe_no\tapply_safe_na\tactions\n'
    if [ "${#summary_total[@]}" -gt 0 ]; then
      printf '%s\n' "${!summary_total[@]}" | sort | while IFS= read -r target_class; do
        printf '%s\t%s\t%s\t%s\t%s\t%s\n' \
          "$target_class" \
          "${summary_total["$target_class"]:-0}" \
          "${summary_apply_yes["$target_class"]:-0}" \
          "${summary_apply_no["$target_class"]:-0}" \
          "${summary_apply_na["$target_class"]:-0}" \
          "${summary_actions["$target_class"]:-}"
      done
    fi
  } >"$INVENTORY_SUMMARY_PATH"
}

migration_blocker_observe() {
  local blocker="$1" apply_safe="$2" open_handles="$3" recommendation="$4" existing_recommendations
  [ -n "$MIGRATION_BLOCKERS_SUMMARY_PATH" ] || return 0

  migration_blocker_total["$blocker"]=$(( ${migration_blocker_total["$blocker"]:-0} + 1 ))
  case "$apply_safe" in
    yes) migration_blocker_apply_yes["$blocker"]=$(( ${migration_blocker_apply_yes["$blocker"]:-0} + 1 )) ;;
    no) migration_blocker_apply_no["$blocker"]=$(( ${migration_blocker_apply_no["$blocker"]:-0} + 1 )) ;;
  esac

  if [[ "$open_handles" =~ ^[0-9]+$ ]]; then
    migration_blocker_open_handles["$blocker"]=$(( ${migration_blocker_open_handles["$blocker"]:-0} + open_handles ))
  fi

  existing_recommendations="${migration_blocker_recommendations["$blocker"]:-}"
  if [ -z "$existing_recommendations" ]; then
    migration_blocker_recommendations["$blocker"]="$recommendation"
  else
    case ",$existing_recommendations," in
      *,"$recommendation",*) ;;
      *) migration_blocker_recommendations["$blocker"]="$existing_recommendations,$recommendation" ;;
    esac
  fi
}

emit_migration_blockers_summary() {
  [ -n "$MIGRATION_BLOCKERS_SUMMARY_PATH" ] || return 0

  {
    printf 'blocker\ttotal\tapply_safe_yes\tapply_safe_no\topen_handles\trecommendations\n'
    if [ "${#migration_blocker_total[@]}" -gt 0 ]; then
      printf '%s\n' "${!migration_blocker_total[@]}" | sort | while IFS= read -r blocker; do
        printf '%s\t%s\t%s\t%s\t%s\t%s\n' \
          "$blocker" \
          "${migration_blocker_total["$blocker"]:-0}" \
          "${migration_blocker_apply_yes["$blocker"]:-0}" \
          "${migration_blocker_apply_no["$blocker"]:-0}" \
          "${migration_blocker_open_handles["$blocker"]:-0}" \
          "${migration_blocker_recommendations["$blocker"]:-}"
      done
    fi
  } >"$MIGRATION_BLOCKERS_SUMMARY_PATH"
}

deep_link_observe() {
  local target_class="$1" action="$2" existing_actions
  [ -n "$DEEP_LINK_SUMMARY_PATH" ] || return 0

  deep_link_total["$target_class"]=$(( ${deep_link_total["$target_class"]:-0} + 1 ))
  existing_actions="${deep_link_actions["$target_class"]:-}"
  if [ -z "$existing_actions" ]; then
    deep_link_actions["$target_class"]="$action"
  else
    case ",$existing_actions," in
      *,"$action",*) ;;
      *) deep_link_actions["$target_class"]="$existing_actions,$action" ;;
    esac
  fi
}

emit_deep_link_summary() {
  [ -n "$DEEP_LINK_SUMMARY_PATH" ] || return 0

  {
    printf 'target_class\ttotal\tactions\n'
    if [ "${#deep_link_total[@]}" -gt 0 ]; then
      printf '%s\n' "${!deep_link_total[@]}" | sort | while IFS= read -r target_class; do
        printf '%s\t%s\t%s\n' \
          "$target_class" \
          "${deep_link_total["$target_class"]:-0}" \
          "${deep_link_actions["$target_class"]:-}"
      done
    fi
  } >"$DEEP_LINK_SUMMARY_PATH"
}

deep_link_row() {
  local scan_root="$1" symlink="$2" link_text="$3" resolved_target="$4" target_class="$5" action="$6"
  if [ -n "$DEEP_LINK_INVENTORY_PATH" ]; then
    printf '%s\t%s\t%s\t%s\t%s\t%s\n' "$scan_root" "$symlink" "$link_text" "$resolved_target" "$target_class" "$action" >>"$DEEP_LINK_INVENTORY_PATH"
  fi
  deep_link_observe "$target_class" "$action"
}

classify_deep_link() {
  local scan_root="$1" symlink="$2" link_text resolved target_class action
  link_text="$(readlink "$symlink" 2>/dev/null || true)"
  resolved="$(readlink -f "$symlink" 2>/dev/null || true)"

  if [ -z "$resolved" ] || { [ ! -e "$resolved" ] && [ ! -L "$resolved" ]; }; then
    target_class="missing-target"
    action="owner-supervised-repair-or-ignore-embedded-toolchain-link"
  elif is_under_meta "$resolved"; then
    target_class="inside-meta"
    action="none"
  else
    case "$resolved" in
      "$REAL_HOME"|"$REAL_HOME"/*)
        target_class="real-home-leak"
        action="migrate-or-relink-to-meta"
        if [ "$FAIL_REAL_HOME_DEEP_LINKS" -eq 1 ]; then
          fail "$symlink resolves into real home outside META_ROOT ($resolved)"
        else
          warn "$symlink resolves into real home outside META_ROOT ($resolved); owner-supervised migration required"
        fi
        ;;
      *)
        target_class="external-system"
        action="embedded-toolchain-or-system-reference"
        ;;
    esac
  fi

  deep_link_row "$scan_root" "$symlink" "$link_text" "$resolved" "$target_class" "$action"
}

scan_deep_links() {
  local scan_root
  [ -n "$DEEP_LINK_INVENTORY_PATH" ] || [ -n "$DEEP_LINK_SUMMARY_PATH" ] || [ "$FAIL_REAL_HOME_DEEP_LINKS" -eq 1 ] || return 0

  for scan_root in "$META_ROOT/.local" "$META_ROOT/.toolchains"; do
    [ -d "$scan_root" ] || continue
    while IFS= read -r -d '' symlink; do
      classify_deep_link "$scan_root" "$symlink"
    done < <(find "$scan_root" -type l -print0 2>/dev/null | sort -z)
  done
}

app_config_target_for_dot() {
  local dot="$1"
  case "$dot" in
    .ideavimrc)
      printf '%s\n' "$META_ROOT/.ideavimrc"
      ;;
    .ollama)
      printf '%s\n' "$META_ROOT/var/lib/ollama"
      ;;
    .claude.json)
      printf '%s\n' "$META_ROOT/.local/share/claude/claude.json"
      ;;
    .gphoto)
      printf '%s\n' "$META_ROOT/.config/gphoto"
      ;;
    .vscode-shared)
      printf '%s\n' "$META_ROOT/.local/share/vscode-shared"
      ;;
    .repomix)
      printf '%s\n' "$META_ROOT/.local/share/repomix"
      ;;
    .ai)
      printf '%s\n' "$META_ROOT/.local/share/ai"
      ;;
    .jetbrains)
      printf '%s\n' "$META_ROOT/.local/share/jetbrains"
      ;;
    .meta)
      printf '%s\n' "$META_ROOT/.local/share/meta"
      ;;
    .java)
      printf '%s\n' "$META_ROOT/.local/share/java"
      ;;
    .pi)
      printf '%s\n' "$META_ROOT/.local/share/pi"
      ;;
    .n8n)
      printf '%s\n' "$META_ROOT/.local/share/n8n"
      ;;
    .n8n-claude-bridge)
      printf '%s\n' "$META_ROOT/.local/share/n8n-claude-bridge"
      ;;
    .pki)
      printf '%s\n' "$META_ROOT/.local/share/pki"
      ;;
    .forge)
      printf '%s\n' "$META_ROOT/.local/share/forge"
      ;;
    .ruvector)
      printf '%s\n' "$META_ROOT/.local/share/ruvector"
      ;;
    .repowire)
      printf '%s\n' "$META_ROOT/.local/state/repowire"
      ;;
    .archon)
      printf '%s\n' "$META_ROOT/.local/share/archon"
      ;;
    .hermes)
      printf '%s\n' "$META_ROOT/.local/share/hermes"
      ;;
    .n8n-mcp)
      printf '%s\n' "$META_ROOT/.local/share/n8n-mcp"
      ;;
    .agents|.ampcode|.claude|.codex|.codeium|.copilot|.cursor|.gemini|.goose_recipes|.junie|.kimi|.kimi-code|.roo|.vscode|.windsurf|.mozilla|.thunderbird)
      printf '%s\n' "$META_ROOT/.local/share/${dot#.}"
      ;;
    *)
      return 1
      ;;
  esac
}

is_app_config_dot() {
  app_config_target_for_dot "$1" >/dev/null 2>&1
}

cache_target_for_dot() {
  local dot="$1"
  case "$dot" in
    .nv)
      printf '%s\n' "$META_ROOT/.local/cache/nvidia"
      ;;
    *)
      return 1
      ;;
  esac
}

is_portable_app_config_file_dot() {
  case "$1" in
    .ideavimrc) return 0 ;;
    *) return 1 ;;
  esac
}

is_portable_app_config_dir_dot() {
  case "$1" in
    .gphoto|.vscode-shared|.repomix|.ai|.jetbrains|.meta|.java|.pi|.n8n|.n8n-claude-bridge|.pki|.forge|.ruvector|.repowire|.archon|.hermes|.n8n-mcp) return 0 ;;
    *) return 1 ;;
  esac
}

is_portable_cache_dir_dot() {
  case "$1" in
    .nv) return 0 ;;
    *) return 1 ;;
  esac
}

is_merge_existing_app_config_dir_dot() {
  case "$1" in
    .junie) return 0 ;;
    *) return 1 ;;
  esac
}

canonical_target_for_dot() {
  local dot="$1"
  case "$dot" in
    .cargo|.rustup|.bun|.npm|.wasmer|.dotnet|.pgrx|.venvs|.go|.gradle|.nix-*)
      printf '%s\n' "$META_ROOT/.toolchains/${dot#.}"
      ;;
    .agents|.ai|.ampcode|.archon|.claude|.claude.json|.codex|.codeium|.copilot|.cursor|.gemini|.goose_recipes|.gphoto|.vscode-shared|.repomix|.hermes|.jetbrains|.meta|.java|.pi|.n8n|.n8n-claude-bridge|.pki|.forge|.ruvector|.repowire|.junie|.kimi|.kimi-code|.n8n-mcp|.ollama|.roo|.vscode|.windsurf|.mozilla|.thunderbird|.ideavimrc)
      app_config_target_for_dot "$dot"
      ;;
    .nv)
      cache_target_for_dot "$dot"
      ;;
    *) printf '%s\n' "$ENVCTL_HOME_SOURCE/$dot" ;;
  esac
}

is_migratable_dot() {
  local dot="$1"

  case "$dot" in
    .*/*|.|..|.local|.config|.cache|.ssh|.aws|.gnupg|.mcp-auth|.docker|.kube|.password-store|.lane|.fxapp-gh-profile)
      return 1
      ;;
    .cargo|.rustup|.bun|.npm|.wasmer|.dotnet|.pgrx|.venvs|.go|.gradle|.nix-*)
      return 0
      ;;
    .*)
      if is_portable_app_config_file_dot "$dot"; then
        return 0
      fi
      if is_portable_app_config_dir_dot "$dot"; then
        return 0
      fi
      if is_portable_cache_dir_dot "$dot"; then
        return 0
      fi
      if is_merge_existing_app_config_dir_dot "$dot"; then
        return 0
      fi
      is_app_config_dot "$dot" && return 0
      [ -e "$ENVCTL_HOME_SOURCE/$dot" ] || [ -L "$ENVCTL_HOME_SOURCE/$dot" ]
      ;;
    *)
      return 1
      ;;
  esac
}

migrate_real_home_dot() {
  local dot="$1" source target resolved

  if ! is_migratable_dot "$dot"; then
    fail "--migrate-dot $dot is not in the supervised migration allowlist; refusing automatic move"
    return 0
  fi

  source="$REAL_HOME/$dot"
  target="$(canonical_target_for_dot "$dot")"

  if [ ! -e "$source" ] && [ ! -L "$source" ]; then
    ok "--migrate-dot $dot: $source is missing; nothing to migrate"
    return 0
  fi

  if is_portable_app_config_file_dot "$dot" && [ ! -f "$source" ]; then
    fail "--migrate-dot $dot expects a regular file; refusing automatic move"
    return 0
  fi

  if is_portable_app_config_dir_dot "$dot" && [ ! -d "$source" ]; then
    fail "--migrate-dot $dot: $source is not a directory; refusing automatic app-config directory migration"
    return 0
  fi

  if is_portable_cache_dir_dot "$dot" && [ ! -d "$source" ]; then
    fail "--migrate-dot $dot: $source is not a directory; refusing automatic cache directory migration"
    return 0
  fi

  if is_merge_existing_app_config_dir_dot "$dot" && [ ! -d "$source" ]; then
    fail "--migrate-dot $dot: $source is not a directory; refusing automatic merge app-config directory migration"
    return 0
  fi

  if [ -L "$source" ]; then
    resolved="$(readlink -f "$source" 2>/dev/null || true)"
    if [ -n "$resolved" ] && is_under_meta "$resolved"; then
      ok "--migrate-dot $dot: $source already resolves inside META_ROOT ($resolved)"
    else
      fail "--migrate-dot $dot: $source is an external symlink (${resolved:-missing target}); refusing automatic relink"
    fi
    return 0
  fi

  if [ "$APPLY" -ne 1 ]; then
    if is_merge_existing_app_config_dir_dot "$dot" && [ -d "$target" ]; then
      say "DRY-RUN: would merge source-only entries from $source into existing $target, archive original under $archive_dir/$dot, and link $source -> $target"
    elif [ -e "$target" ] || [ -L "$target" ]; then
      say "DRY-RUN: would archive $source under $archive_dir/$dot and link it to existing $target"
    else
      say "DRY-RUN: would move $source to $target and link $source -> $target"
    fi
    return 0
  fi

  require_no_open_handles_for_migration "$dot" "$source" || return 0

  if [ -e "$target" ] || [ -L "$target" ]; then
    resolved="$(readlink -f "$target" 2>/dev/null || true)"
    if [ -z "$resolved" ] || ! is_under_meta "$resolved"; then
      fail "--migrate-dot $dot: existing target $target resolves outside META_ROOT (${resolved:-missing target}); refusing automatic migration"
      return 0
    fi
    if is_merge_existing_app_config_dir_dot "$dot"; then
      if [ ! -d "$target" ]; then
        fail "--migrate-dot $dot: existing target $target is not a directory; refusing merge app-config directory migration"
        return 0
      fi
      if ! SOURCE_DIR="$source" TARGET_DIR="$target" python3 - <<'PY'
import filecmp
import os
import shutil
import sys
from pathlib import Path

source = Path(os.environ["SOURCE_DIR"])
target = Path(os.environ["TARGET_DIR"])
mismatches = []
blocked_links = []

for path in source.rglob("*"):
    rel = path.relative_to(source)
    dest = target / rel
    if path.is_symlink():
        blocked_links.append(str(rel))
        continue
    if path.is_dir():
        if dest.exists() and not dest.is_dir():
            mismatches.append(f"{rel}: source directory collides with non-directory target")
        continue
    if path.is_file():
        if not dest.exists():
            continue
        if not dest.is_file():
            mismatches.append(f"{rel}: source file collides with non-file target")
        elif not filecmp.cmp(path, dest, shallow=False):
            mismatches.append(f"{rel}: source file differs from existing target")
        continue
    mismatches.append(f"{rel}: unsupported source entry type")

if blocked_links:
    print("source symlinks are not safe for merge-copy:", ", ".join(blocked_links[:20]), file=sys.stderr)
if mismatches:
    print("merge collision(s):", file=sys.stderr)
    for item in mismatches[:20]:
        print(item, file=sys.stderr)
if blocked_links or mismatches:
    sys.exit(1)

for path in sorted(source.rglob("*"), key=lambda p: len(p.relative_to(source).parts)):
    rel = path.relative_to(source)
    dest = target / rel
    if path.is_dir():
        if not dest.exists():
            dest.mkdir()
            shutil.copystat(path, dest, follow_symlinks=False)
    elif path.is_file() and not dest.exists():
        dest.parent.mkdir(parents=True, exist_ok=True)
        shutil.copy2(path, dest, follow_symlinks=False)
PY
      then
        fail "--migrate-dot $dot: existing target $target has conflicting entries or unsafe links; refusing automatic merge"
        return 0
      fi
      mkdir -p "$archive_dir"
      mv "$source" "$archive_dir/$dot"
      ln -sfn "$target" "$source"
      changed_msg "merged $source into existing $target, archived original to $archive_dir/$dot, and linked $source -> $target"
      return 0
    fi
    mkdir -p "$archive_dir"
    mv "$source" "$archive_dir/$dot"
    ln -sfn "$target" "$source"
    changed_msg "archived $source to $archive_dir/$dot and linked $source -> $target"
  else
    mkdir -p "$(dirname "$target")"
    mv "$source" "$target"
    ln -sfn "$target" "$source"
    changed_msg "moved $source to $target and linked $source -> $target"
  fi
}

classify_real_home_dot() {
  local dot="$1" path="$2" type state target_class canonical_target action apply_safe resolved
  type="$(entry_type "$path")"
  state="real-home-state"
  target_class="app-config-state"
  canonical_target=""
  action="owner-supervised-migration"
  apply_safe="no"

  if [ -L "$path" ]; then
    resolved="$(readlink -f "$path" 2>/dev/null || true)"
    canonical_target="$resolved"
    if [ -n "$resolved" ] && is_under_meta "$resolved"; then
      state="already-meta"
      target_class="already-meta"
      action="none"
      apply_safe="n/a"
    else
      state="external-symlink"
      target_class="external-symlink"
      action="owner-supervised-relink"
      apply_safe="no"
    fi
  elif [ -e "$ENVCTL_HOME_SOURCE/$dot" ]; then
    target_class="managed-dotfile"
    canonical_target="$ENVCTL_HOME_SOURCE/$dot"
    action="owner-supervised-bridge"
  else
    case "$dot" in
      .ssh|.aws|.gnupg|.mcp-auth|.docker|.kube|.password-store|.lane|.fxapp-gh-profile)
        target_class="sensitive"
        action="owner-supervised-vault-or-bridge"
        ;;
      .cache)
        target_class="cache"
        canonical_target="$META_ROOT/.local/cache"
        action="component-managed-cache-migration"
        ;;
      .nv)
        target_class="cache"
        canonical_target="$(cache_target_for_dot "$dot")"
        if [ "$type" = "directory" ]; then
          action="migrate-dir-to-meta-cache-and-bridge"
          apply_safe="yes"
        else
          action="owner-supervised-type-repair"
          apply_safe="no"
        fi
        ;;
      .cargo)
        target_class="toolchain-state"
        canonical_target="$META_ROOT/.toolchains/cargo"
        action="component-managed-toolchain-migration"
        ;;
      .rustup)
        target_class="toolchain-state"
        canonical_target="$META_ROOT/.toolchains/rustup"
        action="component-managed-toolchain-migration"
        ;;
      .bun|.npm|.wasmer|.dotnet|.pgrx|.venvs|.nix-*|.go|.gradle)
        target_class="toolchain-state"
        canonical_target="$META_ROOT/.toolchains/${dot#.}"
        action="component-managed-toolchain-migration"
        ;;
      .bashrc|.profile|.zshrc|.zshenv|.bash_profile|.bash_logout)
        local shell_action shell_apply_safe
        target_class="shell-dotfile"
        canonical_target="$META_ROOT/$dot"
        IFS=$'\t' read -r shell_action shell_apply_safe < <(shell_dotfile_action "$path" "$canonical_target")
        action="$shell_action"
        apply_safe="$shell_apply_safe"
        if [ "$action" = "owner-supervised-merge-and-bridge" ]; then
          record_shell_dotfile_conflict "$dot" "$path" "$canonical_target" "$action" "$apply_safe"
        fi
        ;;
      .bash_history|.zsh_history|.*_history|*.bak|*.bak.*|*.backup|*.backup.*)
        target_class="history-or-backup"
        canonical_target="$(history_archive_target_for_dot "$dot")"
        action="archive-and-bridge"
        apply_safe="yes"
        ;;
      .ideavimrc)
        if [ "$type" = "file" ]; then
          target_class="app-config-state"
          canonical_target="$META_ROOT/.ideavimrc"
          action="migrate-file-to-meta-root-and-bridge"
          apply_safe="yes"
        fi
        ;;
      .config)
        target_class="app-config-state"
        canonical_target="$META_ROOT/.config"
        action="component-managed-config-migration"
        ;;
      .gphoto)
        target_class="app-config-state"
        canonical_target="$META_ROOT/.config/gphoto"
        if [ "$type" = "directory" ]; then
          action="migrate-dir-to-meta-config-and-bridge"
          apply_safe="yes"
        else
          action="owner-supervised-type-repair"
          apply_safe="no"
        fi
        ;;
      .vscode-shared)
        target_class="app-config-state"
        canonical_target="$META_ROOT/.local/share/vscode-shared"
        if [ "$type" = "directory" ]; then
          action="migrate-dir-to-meta-share-and-bridge"
          apply_safe="yes"
        else
          action="owner-supervised-type-repair"
          apply_safe="no"
        fi
        ;;
      .repomix)
        target_class="app-config-state"
        canonical_target="$META_ROOT/.local/share/repomix"
        if [ "$type" = "directory" ]; then
          action="migrate-dir-to-meta-share-and-bridge"
          apply_safe="yes"
        else
          action="owner-supervised-type-repair"
          apply_safe="no"
        fi
        ;;
      .ai)
        target_class="app-config-state"
        canonical_target="$META_ROOT/.local/share/ai"
        if [ "$type" = "directory" ]; then
          action="migrate-dir-to-meta-share-and-bridge"
          apply_safe="yes"
        else
          action="owner-supervised-type-repair"
          apply_safe="no"
        fi
        ;;
      .jetbrains)
        target_class="app-config-state"
        canonical_target="$META_ROOT/.local/share/jetbrains"
        if [ "$type" = "directory" ]; then
          action="migrate-dir-to-meta-share-and-bridge"
          apply_safe="yes"
        else
          action="owner-supervised-type-repair"
          apply_safe="no"
        fi
        ;;
      .meta)
        target_class="app-config-state"
        canonical_target="$META_ROOT/.local/share/meta"
        if [ "$type" = "directory" ]; then
          action="migrate-dir-to-meta-share-and-bridge"
          apply_safe="yes"
        else
          action="owner-supervised-type-repair"
          apply_safe="no"
        fi
        ;;
      .java)
        target_class="app-config-state"
        canonical_target="$META_ROOT/.local/share/java"
        if [ "$type" = "directory" ]; then
          action="migrate-dir-to-meta-share-and-bridge"
          apply_safe="yes"
        else
          action="owner-supervised-type-repair"
          apply_safe="no"
        fi
        ;;
      .pi)
        target_class="app-config-state"
        canonical_target="$META_ROOT/.local/share/pi"
        if [ "$type" = "directory" ]; then
          action="migrate-dir-to-meta-share-and-bridge"
          apply_safe="yes"
        else
          action="owner-supervised-type-repair"
          apply_safe="no"
        fi
        ;;
      .n8n)
        target_class="app-config-state"
        canonical_target="$META_ROOT/.local/share/n8n"
        if [ "$type" = "directory" ]; then
          action="migrate-dir-to-meta-share-and-bridge"
          apply_safe="yes"
        else
          action="owner-supervised-type-repair"
          apply_safe="no"
        fi
        ;;
      .n8n-claude-bridge)
        target_class="app-config-state"
        canonical_target="$META_ROOT/.local/share/n8n-claude-bridge"
        if [ "$type" = "directory" ]; then
          action="migrate-dir-to-meta-share-and-bridge"
          apply_safe="yes"
        else
          action="owner-supervised-type-repair"
          apply_safe="no"
        fi
        ;;
      .pki)
        target_class="app-config-state"
        canonical_target="$META_ROOT/.local/share/pki"
        if [ "$type" = "directory" ]; then
          action="migrate-dir-to-meta-share-and-bridge"
          apply_safe="yes"
        else
          action="owner-supervised-type-repair"
          apply_safe="no"
        fi
        ;;
      .forge)
        target_class="app-config-state"
        canonical_target="$META_ROOT/.local/share/forge"
        if [ "$type" = "directory" ]; then
          action="migrate-dir-to-meta-share-and-bridge"
          apply_safe="yes"
        else
          action="owner-supervised-type-repair"
          apply_safe="no"
        fi
        ;;
      .ruvector)
        target_class="app-config-state"
        canonical_target="$META_ROOT/.local/share/ruvector"
        if [ "$type" = "directory" ]; then
          action="migrate-dir-to-meta-share-and-bridge"
          apply_safe="yes"
        else
          action="owner-supervised-type-repair"
          apply_safe="no"
        fi
        ;;
      .repowire)
        target_class="app-config-state"
        canonical_target="$META_ROOT/.local/state/repowire"
        if [ "$type" = "directory" ]; then
          action="migrate-dir-to-meta-state-and-bridge"
          apply_safe="yes"
        else
          action="owner-supervised-type-repair"
          apply_safe="no"
        fi
        ;;
      .archon)
        target_class="app-config-state"
        canonical_target="$META_ROOT/.local/share/archon"
        if [ "$type" = "directory" ]; then
          action="migrate-dir-to-meta-share-and-bridge"
          apply_safe="yes"
        else
          action="owner-supervised-type-repair"
          apply_safe="no"
        fi
        ;;
      .hermes)
        target_class="app-config-state"
        canonical_target="$META_ROOT/.local/share/hermes"
        if [ "$type" = "directory" ]; then
          action="migrate-dir-to-meta-share-and-bridge"
          apply_safe="yes"
        else
          action="owner-supervised-type-repair"
          apply_safe="no"
        fi
        ;;
      .n8n-mcp)
        target_class="app-config-state"
        canonical_target="$META_ROOT/.local/share/n8n-mcp"
        if [ "$type" = "directory" ]; then
          action="migrate-dir-to-meta-share-and-bridge"
          apply_safe="yes"
        else
          action="owner-supervised-type-repair"
          apply_safe="no"
        fi
        ;;
      .junie)
        target_class="app-config-state"
        canonical_target="$META_ROOT/.local/share/junie"
        if [ "$type" = "directory" ]; then
          action="merge-dir-to-existing-meta-share-and-bridge"
          apply_safe="yes"
        else
          action="owner-supervised-type-repair"
          apply_safe="no"
        fi
        ;;
      .agents|.ampcode|.claude|.claude.json|.codex|.codeium|.copilot|.cursor|.gemini|.goose_recipes|.kimi|.kimi-code|.ollama|.roo|.vscode|.windsurf|.mozilla|.thunderbird)
        target_class="app-config-state"
        canonical_target="$(app_config_target_for_dot "$dot")"
        action="owner-supervised-config-migration"
        ;;
    esac
  fi

  if [ "$target_class" = "sensitive" ]; then
    record_sensitive_state "$dot" "$path" "$type" "$action" "$apply_safe"
  fi
  if [ "$target_class" = "app-config-state" ] && [ "$action" = "owner-supervised-config-migration" ]; then
    record_app_config_conflict "$dot" "$path" "$canonical_target" "$action" "$apply_safe"
  fi
  if [ "$target_class" = "app-config-state" ] && [ -z "$canonical_target" ] && [ "$action" = "owner-supervised-migration" ]; then
    record_unknown_app_config "$dot" "$path" "$type"
  fi
  inventory_row "$dot" "$type" "$state" "$target_class" "$canonical_target" "$action" "$apply_safe"
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
if [ -L "$local_link" ]; then
  local_resolved="$(readlink -f "$local_link" 2>/dev/null || true)"
  if [ "$local_resolved" = "$META_ROOT/.local" ]; then
    inventory_row ".local" "symlink" "meta-bridge" "bridge" "$META_ROOT/.local" "ensure-symlink" "yes"
  else
    inventory_row ".local" "symlink" "bridge-drift" "bridge" "$META_ROOT/.local" "ensure-symlink" "yes"
  fi
elif [ -e "$local_link" ]; then
  inventory_row ".local" "$(entry_type "$local_link")" "real-home-state" "bridge" "$META_ROOT/.local" "owner-supervised-archive-and-bridge" "no"
else
  inventory_row ".local" "missing" "missing" "bridge" "$META_ROOT/.local" "ensure-symlink" "yes"
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
if [ -L "$real_gitconfig" ]; then
  real_gitconfig_resolved="$(readlink -f "$real_gitconfig" 2>/dev/null || true)"
  canonical_resolved="$(readlink -f "$canonical_gitconfig" 2>/dev/null || true)"
  if [ -n "$canonical_resolved" ] && [ "$real_gitconfig_resolved" = "$canonical_resolved" ] && [ "$(readlink "$real_gitconfig")" = "$canonical_gitconfig" ]; then
    inventory_row ".gitconfig" "symlink" "managed-bridge" "managed-dotfile" "$canonical_gitconfig" "bridge-canonical" "yes"
  else
    inventory_row ".gitconfig" "symlink" "bridge-drift" "managed-dotfile" "$canonical_gitconfig" "bridge-canonical" "yes"
  fi
elif [ -e "$real_gitconfig" ]; then
  inventory_row ".gitconfig" "$(entry_type "$real_gitconfig")" "real-home-state" "managed-dotfile" "$canonical_gitconfig" "archive-and-bridge-canonical" "yes"
else
  inventory_row ".gitconfig" "missing" "missing" "managed-dotfile" "$canonical_gitconfig" "bridge-canonical" "yes"
fi

# 5. Apply only explicitly requested allow-listed real-home migrations.  This phase is deliberately
# opt-in so the default audit remains read-only outside the conservative .local/.gitconfig repairs.
for dot in "${MIGRATE_DOTS[@]}"; do
  migrate_real_home_dot "$dot"
done

# 6. Walk every top-level real-home dot entry.  The default audit only mutates .local/.gitconfig;
# requested --migrate-dot entries above are reflected here after they have been bridged into
# META_ROOT.  This keeps the loop honest ("every dot file/folder was observed") without auto-moving
# credentials, caches, shell histories, broad app state, or unrequested toolchains.
if [ -d "$REAL_HOME" ]; then
  while IFS= read -r -d '' path; do
    dot_entries_seen=$((dot_entries_seen + 1))
    dot="$(basename "$path")"
    case "$dot" in
      .|..|.local|.gitconfig) continue ;;
    esac
    if is_shell_dotfile "$dot"; then
      apply_shell_dotfile_bridge "$dot" "$path" "$META_ROOT/$dot"
    fi
    if is_history_or_backup_dot "$dot"; then
      apply_history_archive_bridge "$dot" "$path"
    fi
    classify_real_home_dot "$dot" "$path"
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

# 7. Optional recursive symlink inventory for the actual meta-local/toolchain stores.  This is the
# "walk the folders" proof surface: report every link below META_ROOT/.local and
# META_ROOT/.toolchains, classify whether it stays in META_ROOT, points back into the real home,
# points at system/container internals, or is a missing embedded toolchain link.
scan_deep_links

emit_inventory_summary
emit_migration_blockers_summary
emit_deep_link_summary

if [ "$failures" -gt 0 ]; then
  say "meta-local audit: FAIL failures=$failures warnings=$warnings changed=$changed dot_entries=$dot_entries_seen" >&2
  exit 1
fi
say "meta-local audit: PASS warnings=$warnings changed=$changed dot_entries=$dot_entries_seen"
