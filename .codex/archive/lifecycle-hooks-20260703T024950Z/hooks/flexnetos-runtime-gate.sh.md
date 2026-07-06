#!/usr/bin/env bash
# FlexNetOS Codex runtime gate.
#
# This is harness/runtime policy, not product code. It blocks the failure mode
# that caused prior drift: editing peer repos without a current snapshot, and
# mutating installed Yazelix/profile/desktop surfaces before source proof exists.
set -u

event="${1:-self-test}"
shift || true

root="${FLEXNETOS_ROOT:-/home/flexnetos/FlexNetOS}"
state="${FLEXNETOS_GATE_STATE:-$root/var/lib/codex-runtime-gate}"
log_dir="${FLEXNETOS_GATE_LOG_DIR:-$root/var/log/codex-runtime-gate}"
snapshot_dir="$state/repo-snapshots"
proof_dir="$state/proofs"
violation_dir="$state/violations/open"
archive_dir="$root/var/lib/codex-runtime-gate/archives"
event_log="$log_dir/events.jsonl"
br_bin="${FLEXNETOS_BR:-/nix/store/7k262am00v9lk31in8if4fjbck9jrfj8-beads_rust-0.2.11/bin/br}"
gitkb_bin="${FLEXNETOS_GIT_KB:-$root/usr/bin/git-kb}"
meta_bin="${FLEXNETOS_META:-$root/usr/bin/meta}"

mkdir -p "$snapshot_dir" "$proof_dir" "$violation_dir" "$archive_dir" "$log_dir" 2>/dev/null || true

read_stdin() {
  local payload
  payload="$(cat 2>/dev/null || true)"
  printf '%s' "$payload"
}

json_escape() {
  sed 's/\\/\\\\/g; s/"/\\"/g' | tr '\n' ' '
}

now_utc() {
  date -u +%Y-%m-%dT%H:%M:%SZ
}

log_event() {
  local kind="$1"
  local detail="$2"
  local escaped
  escaped="$(printf '%s' "$detail" | json_escape)"
  printf '{"time":"%s","event":"%s","detail":"%s"}\n' "$(now_utc)" "$kind" "$escaped" >>"$event_log" 2>/dev/null || true
}

repo_key() {
  basename "$1" | tr -c 'A-Za-z0-9_.-' '_'
}

repo_for_payload() {
  local payload="$1"
  local cwd
  cwd="$(pwd -P 2>/dev/null || pwd)"
  local matched_payload=0

  for name in yazelix meta envctl flexnetos_runner; do
    case "$payload" in
      *"$root/src/$name"*|*"/src/$name/"*|*"/src/$name "*|*"/src/$name"*)
        printf '%s\n' "$root/src/$name"
        matched_payload=1
        ;;
    esac
  done

  [ "$matched_payload" -eq 1 ] && return 0

  for name in yazelix meta envctl flexnetos_runner; do
    case "$cwd" in
      *"$root/src/$name"*|*"/src/$name/"*|*"/src/$name")
        printf '%s\n' "$root/src/$name"
        ;;
    esac
  done
}

current_head() {
  git -C "$1" rev-parse HEAD 2>/dev/null || printf 'UNKNOWN'
}

snapshot_file() {
  printf '%s/%s.snapshot' "$snapshot_dir" "$(repo_key "$1")"
}

snapshot_is_current() {
  local repo="$1"
  local file
  file="$(snapshot_file "$repo")"
  [ -f "$file" ] || return 1
  local snap_head curr_head
  snap_head="$(awk -F= '$1=="head"{print $2; exit}' "$file" 2>/dev/null)"
  curr_head="$(current_head "$repo")"
  [ -n "$snap_head" ] && [ "$snap_head" = "$curr_head" ]
}

write_violation() {
  local code="$1"
  local detail="$2"
  local file
  file="$violation_dir/$(date -u +%Y%m%dT%H%M%SZ)-$code.txt"
  {
    printf 'code=%s\n' "$code"
    printf 'time=%s\n' "$(now_utc)"
    printf 'detail=%s\n' "$detail"
  } >"$file" 2>/dev/null || true
  log_event "violation:$code" "$detail"
}

deny() {
  local code="$1"
  local detail="$2"
  write_violation "$code" "$detail"
  printf 'FLEXNETOS CODEX GATE DENY [%s]\n%s\n' "$code" "$detail" >&2
  exit 2
}

looks_like_write_command() {
  local payload="$1"
  case "$payload" in
    *"apply_patch"*|*" sed -i "*|*" rm "*|*" rm -"*|*" mv "*|*" cp "*|*" install "*|*" touch "*|*" chmod "*|*" chown "*|*" git apply "*|*" tee "*|*"> "*) return 0 ;;
    *) return 1 ;;
  esac
}

looks_like_write_tool() {
  local payload="$1"
  case "$payload" in
    *"apply_patch"*|*"\"Edit\""*|*"\"Write\""*) return 0 ;;
    *) return 1 ;;
  esac
}

payload_touches_meta_org() {
  local payload="$1"
  case "$payload" in
    *"$root/src/meta"*|*"/src/meta"*|*"FlexNetOS/meta"*|*"git@github.com:FlexNetOS/meta"*|*"github.com:FlexNetOS/meta"*) return 0 ;;
    *) return 1 ;;
  esac
}

payload_touches_meta_original_fleet() {
  local payload="$1"
  case "$payload" in
    *"$root/src/meta/loop_lib"*|*"$root/src/meta/meta_plugin_protocol"*|*"$root/src/meta/meta_core"*|*"$root/src/meta/meta_git_lib"*|*"$root/src/meta/loop_cli"*|*"$root/src/meta/meta_cli"*|*"$root/src/meta/meta_git_cli"*|*"$root/src/meta/meta_project_cli"*|*"$root/src/meta/meta_rust_cli"*|*"$root/src/meta/meta_mcp"*) return 0 ;;
    *"/src/meta/loop_lib"*|*"/src/meta/meta_plugin_protocol"*|*"/src/meta/meta_core"*|*"/src/meta/meta_git_lib"*|*"/src/meta/loop_cli"*|*"/src/meta/meta_cli"*|*"/src/meta/meta_git_cli"*|*"/src/meta/meta_project_cli"*|*"/src/meta/meta_rust_cli"*|*"/src/meta/meta_mcp"*) return 0 ;;
    *"loop_lib"*|*"meta_plugin_protocol"*|*"meta_core"*|*"meta_git_lib"*|*"loop_cli"*|*"meta_cli"*|*"meta_git_cli"*|*"meta_project_cli"*|*"meta_rust_cli"*|*"meta_mcp"*) payload_touches_meta_org "$payload" && return 0; return 1 ;;
    *) return 1 ;;
  esac
}

looks_like_removal_or_delete() {
  local payload="$1"
  case "$payload" in
    *" rm "*|*" rm -"*|*"\"command\":\"rm "*|*"\"command\":\"rm -"*|*" rmdir "*|*"\"command\":\"rmdir "*|*" unlink "*|*"\"command\":\"unlink "*|*" truncate "*|*"\"command\":\"truncate "*|*" shred "*|*" wipe "*|*" delete "*|*"\"Delete\""*) return 0 ;;
    *" git clean "*|*" git reset --hard"*|*" git checkout -- "*|*" git restore "*|*" git rm "*|*" nix profile remove"*|*"codex plugin remove"*) return 0 ;;
    *) return 1 ;;
  esac
}

looks_like_downgrade_or_overwrite() {
  local payload="$1"
  case "$payload" in
    *" downgrade "*|*" rollback "*|*" revert "*|*" reset "*|*"\"command\":\"reset "*|*"--force"*|*" -f "*|*" --hard"*|*"checkout "*|*"\"command\":\"checkout "*|*"restore "*|*"\"command\":\"restore "*|*"restore-tree"*|*"replace "*|*"overwrite "*) return 0 ;;
    *"git reset"*|*"git checkout"*|*"git restore"*|*"git revert"*|*"nix flake lock --rollback"*|*"nix profile rollback"*) return 0 ;;
    *) return 1 ;;
  esac
}

looks_like_archive_and_compress() {
  local payload="$1"
  case "$payload" in
    *" tar "*".tar.gz"*|*" tar "*".tgz"*|*" tar "*".tar.zst"*|*" zstd "*|*" gzip "*|*" xz "*|*" zip "*|*" 7z "*) ;;
    *) return 1 ;;
  esac

  case "$payload" in
    *"$archive_dir"*|*"$root/var/lib/codex-runtime-gate/archives"*|*"$root/var/log/raw"*|*"$root/artifacts"*|*"$root/snapshots"*) return 0 ;;
    *) return 1 ;;
  esac
}

check_meta_additive_only_policy() {
  local payload="$1"

  payload_touches_meta_org "$payload" || return 0

  if looks_like_removal_or_delete "$payload" || looks_like_downgrade_or_overwrite "$payload"; then
    if looks_like_archive_and_compress "$payload"; then
      log_event "meta-additive-archive-gate" "destructive Meta-org operation includes compressed archive path"
      return 0
    fi

    if payload_touches_meta_original_fleet "$payload"; then
      deny "meta-original-additive-only" "Meta original repos are additive-only upgrade targets. Before any remove/delete/downgrade/reset/overwrite operation, archive the affected state first and write a compressed artifact under $archive_dir, $root/artifacts, $root/snapshots, or $root/var/log/raw."
    fi

    deny "meta-org-archive-required" "FlexNetOS Meta org repos require archive-first discipline for destructive or downgrade-like operations. Create a compressed archive artifact before removal/delete/reset/overwrite."
  fi
}

is_install_check() {
  local payload="$1"
  case "$payload" in
    *"raw.githubusercontent.com/luccahuguet/yazelix/main/shells/posix/install_check.sh"*|*"install_check.sh"*"luccahuguet/yazelix"*) return 0 ;;
    *) return 1 ;;
  esac
}

is_installed_surface_mutation() {
  local payload="$1"
  case "$payload" in
    *"nix profile add"*|*"nix profile install"*|*"nix profile upgrade"*|*"nix profile remove"*|*"home-manager switch"*|*"home-manager boot"*) return 0 ;;
    *"yzx desktop install"*|*"desktop-file-install"*|*"update-desktop-database"*) return 0 ;;
    *"/home/flexnetos/.local/share/applications"*|*"~/.local/share/applications"*|*"/home/flexnetos/.nix-profile"*|*"~/.nix-profile"*)
      looks_like_write_command "$payload" && return 0
      return 1
      ;;
    *) return 1 ;;
  esac
}

installed_surface_is_unlocked() {
  [ -f "$proof_dir/yazelix_install_check.ok" ] \
    && [ -f "$proof_dir/yazelix_source_validated.ok" ] \
    && [ -f "$state/allow_installed_surface_mutation" ]
}

require_executable() {
  local code="$1"
  local path="$2"
  [ -x "$path" ] || deny "$code" "required executable is missing or not executable: $path"
}

run_foundation_check() {
  local code="$1"
  local detail="$2"
  local cwd="$3"
  shift 3
  if ! (cd "$cwd" && "$@" >/dev/null 2>&1); then
    deny "$code" "$detail"
  fi
}

foundation_context_gate() {
  require_executable "beads-missing" "$br_bin"
  require_executable "gitkb-missing" "$gitkb_bin"
  require_executable "meta-missing" "$meta_bin"

  run_foundation_check \
    "beads-unready" \
    "Beads is not initialized or readable in $root/src/yazelix; expected br ready to succeed." \
    "$root/src/yazelix" \
    "$br_bin" ready

  run_foundation_check \
    "yazelix-gitkb-unready" \
    "GitKB is not initialized or verified in $root/src/yazelix; expected git-kb verify to succeed." \
    "$root/src/yazelix" \
    "$gitkb_bin" verify
  run_foundation_check \
    "yazelix-gitkb-status-unready" \
    "GitKB status is not readable in $root/src/yazelix; expected git-kb status --json to succeed." \
    "$root/src/yazelix" \
    "$gitkb_bin" status --json

  run_foundation_check \
    "meta-gitkb-unready" \
    "GitKB is not initialized or verified in $root/src/meta; expected git-kb verify to succeed." \
    "$root/src/meta" \
    "$gitkb_bin" verify
  run_foundation_check \
    "meta-gitkb-status-unready" \
    "GitKB status is not readable in $root/src/meta; expected git-kb status --json to succeed." \
    "$root/src/meta" \
    "$gitkb_bin" status --json

  run_foundation_check \
    "meta-project-unready" \
    "Meta project initialization is not healthy in $root/src/meta; expected meta project check to succeed." \
    "$root/src/meta" \
    env "PATH=$root/usr/bin:$PATH" "$meta_bin" project check

  run_foundation_check \
    "meta-exec-gitkb-unready" \
    "Meta exec GitKB initialization is not healthy across the project set; expected meta exec -- git-kb verify to succeed." \
    "$root/src/meta" \
    env "PATH=$root/usr/bin:$PATH" "$meta_bin" exec -- git-kb verify
  run_foundation_check \
    "meta-exec-gitkb-status-unready" \
    "Meta exec GitKB status is not healthy across the project set; expected meta exec -- git-kb status --json to succeed." \
    "$root/src/meta" \
    env "PATH=$root/usr/bin:$PATH" "$meta_bin" exec -- git-kb status --json

  log_event "foundation-context-gate" "beads, git-kb verify/status, meta project, and meta exec git-kb verify/status passed"
}

check_repo_snapshots() {
  local payload="$1"
  local repo found
  found=0
  while IFS= read -r repo; do
    [ -n "$repo" ] || continue
    found=1
    if ! snapshot_is_current "$repo"; then
      deny "missing-repo-snapshot" "Before editing $repo, run: bash /home/flexnetos/FlexNetOS/src/envctl/.codex/hooks/flexnetos-runtime-gate.sh snapshot $repo"
    fi
  done <<EOF
$(repo_for_payload "$payload")
EOF
  [ "$found" -eq 0 ] && return 0
  return 0
}

pre_tool_use() {
  local payload="$1"
  log_event "$event" "$payload"

  if is_install_check "$payload"; then
    return 0
  fi

  if is_installed_surface_mutation "$payload" && ! installed_surface_is_unlocked; then
    deny "installed-surface-locked" "Profile, Home Manager, or desktop launcher mutation is locked. Required proofs: install_check, source validation, and explicit allow_installed_surface_mutation marker."
  fi

  check_meta_additive_only_policy "$payload"

  if looks_like_write_command "$payload" || looks_like_write_tool "$payload"; then
    check_repo_snapshots "$payload"
  fi
}

post_tool_use() {
  local payload="$1"
  log_event "$event" "$payload"
  if is_install_check "$payload" && printf '%s' "$payload" | grep -Eq '"(exit_code|exitCode|status)"[[:space:]]*:[[:space:]]*0'; then
    printf 'time=%s\nsource=luccahuguet/yazelix install_check\n' "$(now_utc)" >"$proof_dir/yazelix_install_check.ok"
    log_event "proof:yazelix_install_check" "recorded from successful post-tool-use payload"
  fi
}

session_start() {
  log_event "session-start" "cwd=$(pwd -P 2>/dev/null || pwd); root=$root; state=$state"
  foundation_context_gate
  printf 'FlexNetOS Codex runtime gate active: %s\n' "$state" >&2
}

stop_gate() {
  log_event "stop" "cwd=$(pwd -P 2>/dev/null || pwd)"
  foundation_context_gate
  if find "$violation_dir" -type f -print -quit 2>/dev/null | grep -q .; then
    printf 'FLEXNETOS CODEX GATE STOP: unresolved runtime gate violations exist under %s\n' "$violation_dir" >&2
    find "$violation_dir" -type f -maxdepth 1 -print 2>/dev/null | sort | tail -n 10 >&2
    exit 2
  fi
}

snapshot_repo() {
  local repo="${1:-}"
  [ -n "$repo" ] || { printf 'usage: %s snapshot /path/to/repo\n' "$0" >&2; exit 64; }
  [ -d "$repo/.git" ] || git -C "$repo" rev-parse --git-dir >/dev/null 2>&1 || {
    printf 'not a git repo: %s\n' "$repo" >&2
    exit 65
  }
  local file
  file="$(snapshot_file "$repo")"
  {
    printf 'created=%s\n' "$(now_utc)"
    printf 'repo=%s\n' "$repo"
    printf 'branch=%s\n' "$(git -C "$repo" rev-parse --abbrev-ref HEAD 2>/dev/null || true)"
    printf 'head=%s\n' "$(current_head "$repo")"
    printf 'remote=%s\n' "$(git -C "$repo" remote get-url origin 2>/dev/null || true)"
    printf 'dirty_begin\n'
    git -C "$repo" status --short --branch 2>/dev/null || true
    printf 'dirty_end\n'
  } >"$file"
  log_event "snapshot" "$repo -> $file"
  printf '%s\n' "$file"
}

record_yazelix_source_proof() {
  local proof="${1:-}"
  [ -n "$proof" ] && [ -s "$proof" ] || {
    printf 'usage: %s record-yazelix-source-proof /path/to/nonempty-proof-log\n' "$0" >&2
    exit 64
  }
  if ! grep -Eq 'PASS|passed|HARNESS-SCRIPTS GATE PASS|yzx dev test --sweep|cargo nextest|yzx_repo_validator' "$proof"; then
    printf 'proof log does not contain recognized pass markers: %s\n' "$proof" >&2
    exit 65
  fi
  {
    printf 'time=%s\n' "$(now_utc)"
    printf 'proof_log=%s\n' "$proof"
  } >"$proof_dir/yazelix_source_validated.ok"
  log_event "proof:yazelix_source_validated" "$proof"
}

allow_installed_surface_mutation() {
  local reason="${1:-}"
  [ -n "$reason" ] || {
    printf 'usage: %s allow-installed-surface-mutation REASON\n' "$0" >&2
    exit 64
  }
  printf 'time=%s\nreason=%s\n' "$(now_utc)" "$reason" >"$state/allow_installed_surface_mutation"
  log_event "allow-installed-surface-mutation" "$reason"
}

clear_violations() {
  local reason="${1:-}"
  [ -n "$reason" ] || {
    printf 'usage: %s clear-violations REASON\n' "$0" >&2
    exit 64
  }
  find "$violation_dir" -type f -maxdepth 1 -exec rm -f {} + 2>/dev/null || true
  log_event "clear-violations" "$reason"
}

payload="$(read_stdin)"

case "$event" in
  session-start|SessionStart) session_start ;;
  pre-tool-use|PreToolUse|permission-request|PermissionRequest) pre_tool_use "$payload" ;;
  post-tool-use|PostToolUse) post_tool_use "$payload" ;;
  pre-compact|PreCompact) stop_gate ;;
  stop|Stop) stop_gate ;;
  snapshot) snapshot_repo "${1:-}" ;;
  record-yazelix-source-proof) record_yazelix_source_proof "${1:-}" ;;
  allow-installed-surface-mutation) allow_installed_surface_mutation "$*" ;;
  clear-violations) clear_violations "$*" ;;
  self-test) printf 'FlexNetOS Codex runtime gate loaded\n' ;;
  *) printf 'unknown gate event: %s\n' "$event" >&2; exit 64 ;;
esac
