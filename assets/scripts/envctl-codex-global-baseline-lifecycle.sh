#!/usr/bin/env bash
# Validate the editable active-home Codex config and retire only runtime/cache
# shadows.  Binary ownership stays with the canonical Yazelix Nix profile.
set -euo pipefail
export PATH=/usr/bin:/bin
umask 077

codex_global_die() {
  printf 'codex-global: %s\n' "$*" >&2
  exit 1
}

codex_global_is_owned_real_dir() {
  local path="$1" uid="$2"
  [ -d "$path" ] && [ ! -L "$path" ] \
    && [ "$(/usr/bin/readlink -f -- "$path" 2>/dev/null)" = "$path" ] \
    && [ "$(/usr/bin/stat -c '%u' -- "$path")" = "$uid" ]
}

codex_global_require_safe_dir() {
  local path="$1" uid="$2" mode
  codex_global_is_owned_real_dir "$path" "$uid" \
    || codex_global_die "expected a canonical current-user-owned real directory: $path"
  mode="$(/usr/bin/stat -c '%a' -- "$path")"
  (( (8#$mode & 0002) == 0 )) \
    || codex_global_die "world-writable directory is unsafe: $path"
}

codex_global_require_safe_existing_chain() {
  local root="$1" target="$2" uid="$3" relative current part
  case "$target" in
    "$root"|"$root"/*) ;;
    *) codex_global_die "managed path escapes its root: $target" ;;
  esac
  codex_global_require_safe_dir "$root" "$uid"
  relative="${target#"$root"}"
  relative="${relative#/}"
  current="$root"
  IFS='/' read -r -a parts <<<"$relative"
  for part in "${parts[@]}"; do
    [ -n "$part" ] || continue
    current="$current/$part"
    if [ -e "$current" ] || [ -L "$current" ]; then
      codex_global_require_safe_dir "$current" "$uid"
    else
      break
    fi
  done
}

codex_global_prepare_archive_base() {
  local uid="$1" directory
  codex_global_require_safe_existing_chain "$CODEX_GLOBAL_META_ROOT" \
    "$CODEX_GLOBAL_META_ROOT/var/lib/envctl/legacy-archives" "$uid"
  for directory in \
    "$CODEX_GLOBAL_META_ROOT/var" \
    "$CODEX_GLOBAL_META_ROOT/var/lib" \
    "$CODEX_GLOBAL_META_ROOT/var/lib/envctl" \
    "$CODEX_GLOBAL_META_ROOT/var/lib/envctl/legacy-archives"; do
    if [ ! -e "$directory" ] && [ ! -L "$directory" ]; then
      /usr/bin/install -d -m 755 -- "$directory"
    fi
    codex_global_require_safe_dir "$directory" "$uid"
  done
}

codex_global_setup() {
  CODEX_GLOBAL_META_ROOT="${META_ROOT:?META_ROOT required}"
  CODEX_GLOBAL_REAL_HOME="${ENVCTL_REAL_HOME:?ENVCTL_REAL_HOME required}"
  CODEX_GLOBAL_SOURCE_ROOT="${ENVCTL_SOURCE_ROOT:-$CODEX_GLOBAL_META_ROOT/src/envctl}"
  CODEX_GLOBAL_CONFIG_ROOT="$CODEX_GLOBAL_REAL_HOME/.codex"
  CODEX_GLOBAL_CONFIG="$CODEX_GLOBAL_CONFIG_ROOT/config.toml"
  CODEX_GLOBAL_HOOKS="$CODEX_GLOBAL_CONFIG_ROOT/hooks.json"
  CODEX_GLOBAL_PROFILE_LIFECYCLE="$CODEX_GLOBAL_SOURCE_ROOT/assets/scripts/envctl-codex-profile-lifecycle.sh"
  CODEX_GLOBAL_RTK="$CODEX_GLOBAL_REAL_HOME/.nix-profile/bin/rtk"
  CODEX_GLOBAL_POLICY_SOURCES=(
    "$CODEX_GLOBAL_SOURCE_ROOT/home/.codex/AGENTS.md"
    "$CODEX_GLOBAL_SOURCE_ROOT/home/.codex/RTK.md"
    "$CODEX_GLOBAL_SOURCE_ROOT/home/.codex/AGENTS.rtk.md"
    "$CODEX_GLOBAL_SOURCE_ROOT/home/.codex/RULES.md"
    "$CODEX_GLOBAL_SOURCE_ROOT/home/.codex/model-catalog.json"
    "$CODEX_GLOBAL_SOURCE_ROOT/home/AGENTS.md"
    "$CODEX_GLOBAL_SOURCE_ROOT/home/AGENTS.rtk.md"
  )
  CODEX_GLOBAL_POLICY_DESTINATIONS=(
    "$CODEX_GLOBAL_CONFIG_ROOT/AGENTS.md"
    "$CODEX_GLOBAL_CONFIG_ROOT/RTK.md"
    "$CODEX_GLOBAL_CONFIG_ROOT/AGENTS.rtk.md"
    "$CODEX_GLOBAL_CONFIG_ROOT/RULES.md"
    "$CODEX_GLOBAL_CONFIG_ROOT/model-catalog.json"
    "$CODEX_GLOBAL_REAL_HOME/AGENTS.md"
    "$CODEX_GLOBAL_REAL_HOME/AGENTS.rtk.md"
  )
  local profile
  for profile in "$CODEX_GLOBAL_SOURCE_ROOT"/home/.codex/envctl-*.config.toml; do
    [ -e "$profile" ] \
      || codex_global_die "missing tracked Codex profile sources"
    CODEX_GLOBAL_POLICY_SOURCES+=("$profile")
    CODEX_GLOBAL_POLICY_DESTINATIONS+=(
      "$CODEX_GLOBAL_CONFIG_ROOT/$(/usr/bin/basename -- "$profile")"
    )
  done

  [ -f "$CODEX_GLOBAL_PROFILE_LIFECYCLE" ] \
    && [ ! -L "$CODEX_GLOBAL_PROFILE_LIFECYCLE" ] \
    && [ -x "$CODEX_GLOBAL_PROFILE_LIFECYCLE" ] \
    || codex_global_die \
      "missing canonical profile lifecycle: $CODEX_GLOBAL_PROFILE_LIFECYCLE"

  export CODEX_GLOBAL_META_ROOT CODEX_GLOBAL_REAL_HOME CODEX_GLOBAL_SOURCE_ROOT
  export CODEX_GLOBAL_CONFIG_ROOT CODEX_GLOBAL_CONFIG CODEX_GLOBAL_HOOKS CODEX_GLOBAL_PROFILE_LIFECYCLE
  export CODEX_GLOBAL_RTK
}

codex_global_require_hook_dispatcher() {
  local uid expected actual
  uid="$(/usr/bin/id -u)"
  [ -f "$CODEX_GLOBAL_HOOKS" ] && [ ! -L "$CODEX_GLOBAL_HOOKS" ] \
    || codex_global_die "missing canonical Codex hook configuration: $CODEX_GLOBAL_HOOKS"
  [ "$(/usr/bin/stat -c '%u' -- "$CODEX_GLOBAL_HOOKS")" = "$uid" ] \
    || codex_global_die "refusing foreign-owned Codex hook configuration"
  expected='{"hooks":{"PreToolUse":[{"hooks":[{"command":"/home/flexnetos/.nix-profile/bin/rtk hook claude","type":"command"}],"matcher":"Bash"}]}}'
  actual="$(/usr/bin/jq -ceS . "$CODEX_GLOBAL_HOOKS")" \
    || codex_global_die "malformed canonical Codex hook configuration"
  [ "$actual" = "$expected" ] \
    || codex_global_die "Codex hooks must contain only the RTK Bash PreToolUse dispatcher"
}

codex_global_sync_hook_dispatcher() (
  local uid staged
  uid="$(/usr/bin/id -u)"
  codex_global_require_safe_dir "$CODEX_GLOBAL_REAL_HOME" "$uid"
  codex_global_require_safe_dir "$CODEX_GLOBAL_CONFIG_ROOT" "$uid"
  if [ -e "$CODEX_GLOBAL_HOOKS" ] || [ -L "$CODEX_GLOBAL_HOOKS" ]; then
    [ ! -L "$CODEX_GLOBAL_HOOKS" ] \
      || codex_global_die "refusing symlinked Codex hook configuration"
    [ "$(/usr/bin/stat -c '%u' -- "$CODEX_GLOBAL_HOOKS")" = "$uid" ] \
      || codex_global_die "refusing foreign-owned Codex hook configuration"
  fi
  staged="$(/usr/bin/mktemp "$CODEX_GLOBAL_CONFIG_ROOT/.hooks.json.envctl.XXXXXXXX")"
  trap '/usr/bin/rm -f -- "$staged"' EXIT HUP INT TERM
  /usr/bin/install -m 600 /dev/null "$staged"
  /usr/bin/printf '%s\n' \
    '{' \
    '  "hooks": {' \
    '    "PreToolUse": [' \
    '      {' \
    '        "matcher": "Bash",' \
    '        "hooks": [' \
    '          {' \
    '            "type": "command",' \
    '            "command": "/home/flexnetos/.nix-profile/bin/rtk hook claude"' \
    '          }' \
    '        ]' \
    '      }' \
    '    ]' \
    '  }' \
    '}' >"$staged"
  /usr/bin/mv -Tf -- "$staged" "$CODEX_GLOBAL_HOOKS"
  trap - EXIT HUP INT TERM
  codex_global_require_hook_dispatcher
)

codex_global_require_policy_sources() {
  local source
  for source in "${CODEX_GLOBAL_POLICY_SOURCES[@]}"; do
    [ -f "$source" ] && [ ! -L "$source" ] && [ -r "$source" ] \
      || codex_global_die "missing tracked Codex policy/profile source: $source"
    case "$source" in "$CODEX_GLOBAL_SOURCE_ROOT/home"/*) ;; *)
      codex_global_die "tracked RTK policy source escaped the home projection: $source"
      ;;
    esac
  done
  /usr/bin/grep -Fqx \
    "@$CODEX_GLOBAL_REAL_HOME/.codex/RTK.md" \
    "$CODEX_GLOBAL_SOURCE_ROOT/home/.codex/AGENTS.md" \
    || codex_global_die \
      "tracked Codex AGENTS.md does not include the active RTK.md path"
  /usr/bin/grep -Fqx \
    "@$CODEX_GLOBAL_REAL_HOME/.codex/AGENTS.rtk.md" \
    "$CODEX_GLOBAL_SOURCE_ROOT/home/.codex/AGENTS.md" \
    || codex_global_die \
      "tracked Codex AGENTS.md does not include the active AGENTS.rtk.md path"
}

codex_global_require_policy_projection() {
  local uid index source destination mode
  uid="$(/usr/bin/id -u)"
  codex_global_require_policy_sources
  for index in "${!CODEX_GLOBAL_POLICY_SOURCES[@]}"; do
    source="${CODEX_GLOBAL_POLICY_SOURCES[$index]}"
    destination="${CODEX_GLOBAL_POLICY_DESTINATIONS[$index]}"
    if [ ! -f "$destination" ] || [ -L "$destination" ] \
        || [ "$(/usr/bin/stat -c '%u' -- "$destination" 2>/dev/null || true)" != "$uid" ] \
        || ! /usr/bin/cmp -s -- "$source" "$destination"; then
      codex_global_die \
        "missing or drifted active-home Codex policy/profile projection: $destination"
    fi
    mode="$(/usr/bin/stat -c '%a' -- "$destination")"
    [ "$mode" = 600 ] \
      || codex_global_die \
        "active-home Codex policy/profile projection must have mode 600: $destination"
  done
}

codex_global_require_rtk_policy_acceptance() {
  local output
  [ -x "$CODEX_GLOBAL_RTK" ] \
    || codex_global_die "profile-owned RTK is unavailable for Codex policy acceptance"
  output="$(/usr/bin/env -i \
    HOME="$CODEX_GLOBAL_REAL_HOME" \
    NO_COLOR=1 \
    TERM=dumb \
    PATH="$CODEX_GLOBAL_REAL_HOME/.nix-profile/bin:$CODEX_GLOBAL_REAL_HOME/.nix-profile/toolbin:/usr/bin:/bin" \
    "$CODEX_GLOBAL_RTK" init --global --codex --show)" \
    || codex_global_die "profile RTK could not inspect global Codex policy"
  printf '%s\n' "$output" \
    | /usr/bin/grep -Eq '^\[ok\] Global RTK\.md:' \
    || codex_global_die "profile RTK does not recognize global RTK.md"
  printf '%s\n' "$output" \
    | /usr/bin/grep -Eq '^\[ok\] Global AGENTS\.md:' \
    || codex_global_die "profile RTK does not recognize global AGENTS.md"
}

codex_global_sync_policy_projection() (
  local uid index source destination parent staged_path archive_root=""
  local relative archive_destination committed=0 need_archive=0 position
  local -a indexes=() staged=() destinations=() had_original=()
  local -a archive_destinations=() original_moved=() installed=()

  uid="$(/usr/bin/id -u)"
  codex_global_require_policy_sources
  codex_global_require_safe_dir "$CODEX_GLOBAL_REAL_HOME" "$uid"
  codex_global_require_safe_dir "$CODEX_GLOBAL_CONFIG_ROOT" "$uid"

  for index in "${!CODEX_GLOBAL_POLICY_SOURCES[@]}"; do
    source="${CODEX_GLOBAL_POLICY_SOURCES[$index]}"
    destination="${CODEX_GLOBAL_POLICY_DESTINATIONS[$index]}"
    parent="$(/usr/bin/dirname -- "$destination")"
    codex_global_require_safe_existing_chain \
      "$CODEX_GLOBAL_REAL_HOME" "$parent" "$uid"

    if [ -f "$destination" ] && [ ! -L "$destination" ] \
        && [ "$(/usr/bin/stat -c '%u' -- "$destination")" = "$uid" ] \
        && [ "$(/usr/bin/stat -c '%a' -- "$destination")" = 600 ] \
        && /usr/bin/cmp -s -- "$source" "$destination"; then
      continue
    fi

    if [ -e "$destination" ] || [ -L "$destination" ]; then
      [ "$(/usr/bin/stat -c '%u' -- "$destination")" = "$uid" ] \
        || codex_global_die "refusing foreign Codex policy/profile projection: $destination"
      need_archive=1
      had_original+=(1)
    else
      had_original+=(0)
    fi

    staged_path="$(/usr/bin/mktemp \
      "$parent/.envctl-rtk-policy.$(/usr/bin/basename -- "$destination").XXXXXXXX")"
    /usr/bin/install -m 600 -- "$source" "$staged_path"
    indexes+=("$index")
    destinations+=("$destination")
    staged+=("$staged_path")
    archive_destinations+=("")
    original_moved+=(0)
    installed+=(0)
  done

  [ "${#indexes[@]}" -gt 0 ] || return 0

  if [ "$need_archive" -eq 1 ]; then
    codex_global_prepare_archive_base "$uid"
    archive_root="$(/usr/bin/mktemp -d \
      "$CODEX_GLOBAL_META_ROOT/var/lib/envctl/legacy-archives/codex-rtk-policy.XXXXXXXX")"
    for position in "${!indexes[@]}"; do
      [ "${had_original[$position]}" -eq 1 ] || continue
      destination="${destinations[$position]}"
      relative="active-home/${destination#"$CODEX_GLOBAL_REAL_HOME"/}"
      archive_destination="$archive_root/$relative"
      /usr/bin/install -d -m 700 -- "$(/usr/bin/dirname -- "$archive_destination")"
      archive_destinations[$position]="$archive_destination"
    done
  fi

  rollback_policy_projection() {
    local rollback_position rollback_destination rollback_archive rollback_staged
    [ "$committed" -eq 0 ] || return 0
    for rollback_position in "${!indexes[@]}"; do
      rollback_destination="${destinations[$rollback_position]}"
      rollback_archive="${archive_destinations[$rollback_position]}"
      rollback_staged="${staged[$rollback_position]}"
      if [ "${installed[$rollback_position]}" -eq 1 ]; then
        /usr/bin/rm -rf -- "$rollback_destination"
      fi
      if [ "${original_moved[$rollback_position]}" -eq 1 ] \
          && { [ -e "$rollback_archive" ] || [ -L "$rollback_archive" ]; }; then
        /usr/bin/mv -T --no-copy -- "$rollback_archive" "$rollback_destination"
      fi
      if [ -e "$rollback_staged" ] || [ -L "$rollback_staged" ]; then
        /usr/bin/rm -f -- "$rollback_staged"
      fi
    done
    if [ -n "$archive_root" ] && [ -d "$archive_root" ] \
        && [ -z "$(/usr/bin/find "$archive_root" -mindepth 1 -print -quit)" ]; then
      /usr/bin/rmdir -- "$archive_root"
    fi
  }
  trap rollback_policy_projection EXIT

  for position in "${!indexes[@]}"; do
    destination="${destinations[$position]}"
    if [ "${had_original[$position]}" -eq 1 ]; then
      archive_destination="${archive_destinations[$position]}"
      /usr/bin/mv -T --no-copy -- "$destination" "$archive_destination"
      original_moved[$position]=1
    fi
    /usr/bin/mv -T -- "${staged[$position]}" "$destination"
    installed[$position]=1
  done

  codex_global_require_policy_projection
  committed=1
)

codex_global_config_policy() {
  local feature_mode="${1:-strict}" repair_features=0 uid mode

  case "$feature_mode" in
    strict) ;;
    repair) repair_features=1 ;;
    *) codex_global_die "invalid config-policy mode: $feature_mode" ;;
  esac

  [ -d "$CODEX_GLOBAL_CONFIG_ROOT" ] && [ ! -L "$CODEX_GLOBAL_CONFIG_ROOT" ] \
    || codex_global_die \
      "active-home Codex config root must be a real directory: $CODEX_GLOBAL_CONFIG_ROOT"
  [ -f "$CODEX_GLOBAL_CONFIG" ] && [ ! -L "$CODEX_GLOBAL_CONFIG" ] \
    || codex_global_die "missing active-home Codex config: $CODEX_GLOBAL_CONFIG"
  uid="$(/usr/bin/id -u)"
  [ "$(/usr/bin/stat -c '%u' -- "$CODEX_GLOBAL_CONFIG")" = "$uid" ] \
    || codex_global_die "refusing foreign-owned active-home Codex config"
  mode="$(/usr/bin/stat -c '%a' -- "$CODEX_GLOBAL_CONFIG")"
  [ "${mode: -2}" = "00" ] \
    || codex_global_die "active-home Codex config must not be group/world accessible"

  LC_ALL=C /usr/bin/awk -v repair_features="$repair_features" '
    function trim(value) {
      sub(/^[[:space:]]+/, "", value)
      sub(/[[:space:]]+$/, "", value)
      return value
    }
    function refuse(message) {
      print "codex-global: " message > "/dev/stderr"
      bad = 1
    }
    function expected_url(name) {
      if (name == "exa") return "\"https://mcp.exa.ai/mcp\""
      if (name == "openaiDeveloperDocs") return "\"https://developers.openai.com/mcp\""
      return ""
    }
    {
      line = $0
      sub(/\r$/, "", line)
      line = trim(line)
      if (line == "" || substr(line, 1, 1) == "#") next

      if (substr(line, 1, 2) == "[[") {
        if (line ~ /^\[\[(features|mcp_servers|marketplaces|plugins)(\.|\]\])/) {
          refuse("array tables cannot own feature, MCP, plugin, or marketplace runtime state")
        }
        current_mcp = ""
        next
      }

      if (substr(line, 1, 1) == "[") {
        close_pos = index(line, "]")
        if (close_pos == 0) {
          refuse("malformed active-home table header")
          current_mcp = ""
          next
        }
        header = trim(substr(line, 2, close_pos - 2))
        tail = trim(substr(line, close_pos + 1))
        if (tail != "" && substr(tail, 1, 1) != "#") {
          refuse("malformed active-home table header")
        }
        current_mcp = ""

        if (header ~ /^(marketplaces|plugins)(\.|$)/) {
          refuse("forbidden active-home plugin or marketplace table: " header)
          next
        }
        if (header == "features") {
          seen_features_table++
          if (seen_features_table > 1) {
            refuse("duplicate active-home features table")
          }
        }
        if (header == "mcp_servers") {
          refuse("MCP configuration must use one explicit allowed server table")
          next
        }
        if (header ~ /^mcp_servers\./) {
          name = substr(header, length("mcp_servers.") + 1)
          if (name != "exa" && name != "openaiDeveloperDocs") {
            refuse("forbidden active-home MCP server: " name)
            next
          }
          seen_table[name]++
          if (seen_table[name] > 1) {
            refuse("duplicate active-home MCP server table: " name)
          }
          current_mcp = name
        }
        next
      }

      equals = index(line, "=")
      if (equals == 0) next
      key = trim(substr(line, 1, equals - 1))
      value = trim(substr(line, equals + 1))
      sub(/[[:space:]]+#.*$/, "", value)
      value = trim(value)

      if (current_mcp != "") {
        if (key != "url" || value != expected_url(current_mcp)) {
          refuse("allowed MCP `" current_mcp "` must contain only its canonical remote URL")
        } else {
          seen_url[current_mcp]++
          if (seen_url[current_mcp] > 1) {
            refuse("duplicate URL for active-home MCP server: " current_mcp)
          }
        }
      } else if (header == "features" && \
          (key == "plugins" || key == "remote_plugin")) {
        seen_feature[key]++
        if (seen_feature[key] > 1) {
          refuse("duplicate features." key " assignment")
        }
        if (value == "false") {
          disabled_feature[key] = 1
        } else if (!repair_features) {
          refuse("features." key " must be explicitly false")
        }
      } else if (header == "" && \
          key ~ /^(features|mcp_servers|marketplaces|plugins)(\.|$)/) {
        refuse("inline feature, MCP, plugin, or marketplace runtime authority is forbidden")
      }
    }
    END {
      for (name in seen_table) {
        if (seen_url[name] != 1) {
          refuse("allowed MCP `" name "` must contain exactly one canonical remote URL")
        }
      }
      if (!repair_features) {
        if (disabled_feature["plugins"] != 1) {
          refuse("features.plugins must be explicitly false")
        }
        if (disabled_feature["remote_plugin"] != 1) {
          refuse("features.remote_plugin must be explicitly false")
        }
      }
      exit bad ? 1 : 0
    }
  ' "$CODEX_GLOBAL_CONFIG" \
    || codex_global_die "active-home Codex config violates the remote-only allowlist"
}

codex_global_feature_disabled() {
  local feature="$1"
  LC_ALL=C /usr/bin/awk -v feature="$feature" '
    function trim(value) {
      sub(/^[[:space:]]+/, "", value)
      sub(/[[:space:]]+$/, "", value)
      return value
    }
    {
      line = trim($0)
      if (line ~ /^\[features\][[:space:]]*(#.*)?$/) {
        in_features = 1
        next
      }
      if (line ~ /^\[/) in_features = 0
      if (!in_features) next
      equals = index(line, "=")
      if (equals == 0) next
      key = trim(substr(line, 1, equals - 1))
      value = trim(substr(line, equals + 1))
      sub(/[[:space:]]+#.*$/, "", value)
      value = trim(value)
      if (key == feature) {
        count++
        if (value == "false") disabled++
      }
    }
    END { exit count == 1 && disabled == 1 ? 0 : 1 }
  ' "$CODEX_GLOBAL_CONFIG"
}

codex_global_unrelated_config_fingerprint() {
  LC_ALL=C /usr/bin/awk '
    /^\[features\][[:space:]]*(#.*)?$/ { in_features = 1; next }
    /^\[/ { in_features = 0 }
    in_features && /^[[:space:]]*(plugins|remote_plugin)[[:space:]]*=/ { next }
    { print }
  ' "$CODEX_GLOBAL_CONFIG" \
    | /usr/bin/sed '/^[[:space:]]*$/d' \
    | /usr/bin/sha256sum \
    | /usr/bin/cut -d' ' -f1
}

codex_global_disable_feature() {
  local feature="$1" codex="$CODEX_GLOBAL_REAL_HOME/.nix-profile/bin/codex"
  [ -x "$codex" ] \
    || codex_global_die "profile-owned Codex frontdoor is unavailable for feature repair"
  /usr/bin/env -i \
    HOME="$CODEX_GLOBAL_REAL_HOME" \
    META_ROOT="$CODEX_GLOBAL_META_ROOT" \
    PATH="$CODEX_GLOBAL_REAL_HOME/.nix-profile/bin:$CODEX_GLOBAL_REAL_HOME/.nix-profile/toolbin:/usr/bin:/bin" \
    "$codex" features disable "$feature" >/dev/null \
    || codex_global_die "official profile Codex could not disable feature: $feature"
  codex_global_config_policy repair
  codex_global_feature_disabled "$feature" \
    || codex_global_die "official feature disable did not converge: $feature"
}

codex_global_repair_features() (
  local backup before after feature committed=0
  backup="$(/usr/bin/mktemp \
    "$CODEX_GLOBAL_CONFIG_ROOT/.config.toml.envctl-feature-backup.XXXXXXXX")"
  /usr/bin/cp --preserve=all --reflink=auto -- "$CODEX_GLOBAL_CONFIG" "$backup"
  # shellcheck disable=SC2329 # Invoked indirectly by the EXIT/signal trap below.
  rollback_feature_config() {
    if [ "$committed" -eq 1 ]; then
      /usr/bin/rm -f -- "$backup"
    elif [ -e "$backup" ] || [ -L "$backup" ]; then
      /usr/bin/mv -Tf -- "$backup" "$CODEX_GLOBAL_CONFIG"
    fi
  }
  trap rollback_feature_config EXIT HUP INT TERM

  before="$(codex_global_unrelated_config_fingerprint)"
  for feature in plugins remote_plugin; do
    codex_global_feature_disabled "$feature" \
      || codex_global_disable_feature "$feature"
  done
  codex_global_config_policy
  after="$(codex_global_unrelated_config_fingerprint)"
  if [ "$after" != "$before" ]; then
    printf '%s\n' \
      'codex-global: official feature repair changed unrelated editable config' >&2
    exit 1
  fi
  committed=1
)

codex_global_shadow_paths() {
  local path root

  for path in \
    "$CODEX_GLOBAL_CONFIG_ROOT/plugins" \
    "$CODEX_GLOBAL_REAL_HOME/.local/state/oh-my-codex" \
    "$CODEX_GLOBAL_REAL_HOME/.local/share/codex-binary-backups"; do
    if [ -e "$path" ] || [ -L "$path" ]; then
      printf '%s\0' "$path"
    fi
  done

  root="$CODEX_GLOBAL_CONFIG_ROOT/.tmp"
  if [ -L "$root" ] || { [ -e "$root" ] && [ ! -d "$root" ]; }; then
    printf '%s\0' "$root"
  elif [ -d "$root" ]; then
    for path in "$root/plugins" "$root/plugins.sha" "$root/plugins.sync.lock"; do
      if [ -e "$path" ] || [ -L "$path" ]; then
        printf '%s\0' "$path"
      fi
    done
  fi

  for root in \
    "$CODEX_GLOBAL_CONFIG_ROOT/cache" \
    "$CODEX_GLOBAL_CONFIG_ROOT/tmp"; do
    if [ -L "$root" ] || { [ -e "$root" ] && [ ! -d "$root" ]; }; then
      printf '%s\0' "$root"
    elif [ -d "$root" ]; then
      /usr/bin/find "$root" -mindepth 1 -maxdepth 1 \
        \( -iname '*plugin*' -o -iname '*marketplace*' \) -print0
    fi
  done
}

codex_global_no_shadows() {
  local path found=0
  while IFS= read -r -d '' path; do
    printf 'codex-global: forbidden Codex runtime shadow: %s\n' "$path" >&2
    found=1
  done < <(codex_global_shadow_paths)
  [ "$found" -eq 0 ]
}

codex_global_archive_shadows() {
  local uid path archive_root relative destination
  local -a shadows=()
  uid="$(/usr/bin/id -u)"
  mapfile -d '' -t shadows < <(codex_global_shadow_paths)
  [ "${#shadows[@]}" -gt 0 ] || return 0

  for path in "${shadows[@]}"; do
    codex_global_require_safe_existing_chain "$CODEX_GLOBAL_REAL_HOME" \
      "$(/usr/bin/dirname -- "$path")" "$uid"
    [ "$(/usr/bin/stat -c '%u' -- "$path")" = "$uid" ] \
      || codex_global_die "refusing foreign Codex runtime shadow: $path"
    case "$path" in
      "$CODEX_GLOBAL_REAL_HOME"/*) ;;
      *) codex_global_die "runtime shadow escaped active home: $path" ;;
    esac
  done

  codex_global_prepare_archive_base "$uid"
  archive_root="$(/usr/bin/mktemp -d \
    "$CODEX_GLOBAL_META_ROOT/var/lib/envctl/legacy-archives/codex-global-shadows.XXXXXXXX")"
  for path in "${shadows[@]}"; do
    relative="active-home/${path#"$CODEX_GLOBAL_REAL_HOME"/}"
    destination="$archive_root/$relative"
    /usr/bin/install -d -m 700 -- "$(/usr/bin/dirname -- "$destination")"
    [ ! -e "$destination" ] && [ ! -L "$destination" ] \
      || codex_global_die "runtime-shadow archive collision: $destination"
    /usr/bin/mv -T --no-copy -- "$path" "$destination"
    printf 'codex-global: archived forbidden runtime shadow %s -> %s\n' \
      "$path" "$destination"
  done
}

codex_global_repair() {
  local profile_action="$1"

  "$CODEX_GLOBAL_PROFILE_LIFECYCLE" "$profile_action" >/dev/null
  codex_global_config_policy repair
  codex_global_repair_features \
    || codex_global_die "official feature repair transaction rolled back"
  codex_global_sync_policy_projection
  codex_global_sync_hook_dispatcher
  codex_global_require_rtk_policy_acceptance
  codex_global_archive_shadows
  codex_global_no_shadows \
    || codex_global_die "forbidden Codex runtime shadows remain after repair"
  "$CODEX_GLOBAL_PROFILE_LIFECYCLE" verify >/dev/null
  codex_global_config_policy
}

codex_global_main() {
  local action="${1:-}"
  [ "$#" -eq 1 ] \
    || codex_global_die \
      "usage: envctl-codex-global-baseline-lifecycle.sh detect|verify|install|fix|remove"
  codex_global_setup

  case "$action" in
    detect)
      "$CODEX_GLOBAL_PROFILE_LIFECYCLE" detect >/dev/null
      codex_global_config_policy
      codex_global_require_policy_projection
      codex_global_require_hook_dispatcher
      codex_global_require_rtk_policy_acceptance
      codex_global_no_shadows
      ;;
    verify)
      "$CODEX_GLOBAL_PROFILE_LIFECYCLE" verify >/dev/null
      codex_global_config_policy
      codex_global_require_policy_projection
      codex_global_require_hook_dispatcher
      codex_global_require_rtk_policy_acceptance
      codex_global_no_shadows \
        || codex_global_die "forbidden Codex runtime shadows are present"
      printf 'codex-global: verified editable active-home config and shadow-free runtime\n'
      ;;
    install|fix)
      codex_global_repair "$action"
      printf 'codex-global: %s preserved config and retired runtime shadows\n' "$action"
      ;;
    remove)
      "$CODEX_GLOBAL_PROFILE_LIFECYCLE" detect >/dev/null
      codex_global_config_policy
      codex_global_require_policy_projection
      codex_global_require_hook_dispatcher
      codex_global_require_rtk_policy_acceptance
      codex_global_archive_shadows
      codex_global_no_shadows \
        || codex_global_die "forbidden Codex runtime shadows remain after removal"
      "$CODEX_GLOBAL_PROFILE_LIFECYCLE" verify >/dev/null
      codex_global_config_policy
      codex_global_require_policy_projection
      codex_global_require_hook_dispatcher
      codex_global_require_rtk_policy_acceptance
      printf 'codex-global: removed only forbidden runtime shadows; profile and config remain\n'
      ;;
    *)
      codex_global_die \
        "usage: envctl-codex-global-baseline-lifecycle.sh detect|verify|install|fix|remove"
      ;;
  esac
}

if [[ "${BASH_SOURCE[0]}" == "$0" ]]; then
  codex_global_main "$@"
fi
