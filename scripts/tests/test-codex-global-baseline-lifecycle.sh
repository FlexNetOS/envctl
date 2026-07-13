#!/usr/bin/env bash
set -euo pipefail

ROOT="$(git -C "$(dirname "${BASH_SOURCE[0]}")" rev-parse --show-toplevel)"
LIFECYCLE="$ROOT/assets/scripts/envctl-codex-global-baseline-lifecycle.sh"

fail() {
  printf 'FAIL: %s\n' "$*" >&2
  exit 1
}

assert_absent() {
  [ ! -e "$1" ] && [ ! -L "$1" ] || fail "expected absent path: $1"
}

tmp="$(mktemp -d -t envctl-codex-global-contract.XXXXXXXX)"
trap 'rm -rf "$tmp"' EXIT
meta="$tmp/meta"
real_home="$tmp/home"
source_root="$tmp/source"
fixture_lifecycle="$source_root/assets/scripts/envctl-codex-global-baseline-lifecycle.sh"
profile_lifecycle="$source_root/assets/scripts/envctl-codex-profile-lifecycle.sh"
config="$real_home/.codex/config.toml"

install -d -m 755 \
  "$meta" \
  "$real_home/.codex" \
  "$real_home/.nix-profile/bin" \
  "$real_home/.nix-profile/toolbin" \
  "$source_root/assets/scripts" \
  "$source_root/home/.codex"
[ -x "$LIFECYCLE" ] || fail "missing executable lifecycle: $LIFECYCLE"
install -m 755 "$LIFECYCLE" "$fixture_lifecycle"

printf '@%s/.codex/RTK.md\n@%s/.codex/AGENTS.rtk.md\n' \
  "$real_home" "$real_home" >"$source_root/home/.codex/AGENTS.md"
for policy in \
  home/.codex/RTK.md \
  home/.codex/AGENTS.rtk.md \
  home/AGENTS.md \
  home/AGENTS.rtk.md; do
  printf 'tracked policy: %s\n' "$policy" >"$source_root/$policy"
done

cat >"$profile_lifecycle" <<'PROFILE'
#!/usr/bin/env bash
set -euo pipefail
action="${1:?action required}"
printf '%s\n' "$action" >>"$META_ROOT/profile-actions.log"
printf 'profile:%s\n' "$action" >>"$META_ROOT/global-actions.log"
[ -f "$META_ROOT/profile-ready" ] || exit 1
case "$action" in
  detect|verify|install|fix) ;;
  remove) exit 86 ;;
  *) exit 2 ;;
esac
PROFILE
chmod 755 "$profile_lifecycle"
: >"$meta/profile-ready"

cat >"$real_home/.nix-profile/bin/codex" <<'CODEX'
#!/usr/bin/env bash
set -euo pipefail
[ "${1:-}" = features ] && [ "${2:-}" = disable ]
feature="${3:?feature required}"
case "$feature" in plugins|remote_plugin) ;; *) exit 2 ;; esac
shadow_state=absent
[ -e "$HOME/.codex/plugins" ] && shadow_state=present
printf 'feature:%s:shadow-%s\n' "$feature" "$shadow_state" \
  >>"$META_ROOT/global-actions.log"
[ ! -f "$META_ROOT/feature-no-converge" ] || exit 0
if [ -f "$META_ROOT/feature-corrupt-unrelated" ]; then
  sed -i 's/model = "gpt-5.6-sol"/model = "corrupted-by-feature-command"/' \
    "$HOME/.codex/config.toml"
fi

config="$HOME/.codex/config.toml"
temporary="$(mktemp "$HOME/.codex/.config.toml.XXXXXXXX")"
awk -v feature="$feature" '
  BEGIN { in_features = 0; saw_features = 0; wrote = 0 }
  /^\[features\][[:space:]]*(#.*)?$/ {
    in_features = 1
    saw_features = 1
    print
    next
  }
  /^\[/ {
    if (in_features && !wrote) {
      print feature " = false"
      wrote = 1
    }
    in_features = 0
  }
  {
    if (in_features && $0 ~ "^[[:space:]]*" feature "[[:space:]]*=") {
      if (!wrote) print feature " = false"
      wrote = 1
      next
    }
    print
  }
  END {
    if (!saw_features) {
      print ""
      print "[features]"
    }
    if (!wrote) print feature " = false"
  }
' "$config" >"$temporary"
chmod --reference="$config" "$temporary"
mv -T "$temporary" "$config"
CODEX
chmod 755 "$real_home/.nix-profile/bin/codex"
ln -s ../bin/codex "$real_home/.nix-profile/toolbin/codex"

# Hermetic lifecycle seam for the real `rtk init --global --codex --show`
# acceptance call. This validates exact argv and global-line parsing; the live
# acceptance proof is run separately with the profile RTK binary.
cat >"$real_home/.nix-profile/bin/rtk" <<'RTK'
#!/usr/bin/env bash
set -euo pipefail
[ "$*" = 'init --global --codex --show' ]
if [ -f "$HOME/.codex/RTK.md" ] && [ -f "$HOME/.codex/AGENTS.md" ]; then
  printf '%s\n' \
    'rtk Configuration (Codex CLI):' \
    '[ok] Global RTK.md: configured' \
    '[ok] Global AGENTS.md: configured'
else
  printf '%s\n' \
    '[--] Global RTK.md: not found' \
    '[--] Global AGENTS.md: not found'
fi
RTK
chmod 755 "$real_home/.nix-profile/bin/rtk"
ln -s ../bin/rtk "$real_home/.nix-profile/toolbin/rtk"

write_unpinned_config() {
  cat >"$config" <<'TOML'
model = "gpt-5.6-sol"
service_tier = "fast"

[projects."/home/flexnetos/meta/src/envctl"]
trust_level = "trusted"

[apps.connector_fixture.tools."github.create_pull_request"]
approval_mode = "approve"

[mcp_servers.openaiDeveloperDocs]
url = "https://developers.openai.com/mcp"
TOML
  chmod 600 "$config"
}

write_valid_config() {
  write_unpinned_config
  cat >>"$config" <<'TOML'

[features]
plugins = false
remote_plugin = false
TOML
}

unrelated_config_fingerprint() {
  awk '
    /^\[features\][[:space:]]*(#.*)?$/ { in_features = 1; next }
    /^\[/ { in_features = 0 }
    in_features && /^[[:space:]]*(plugins|remote_plugin)[[:space:]]*=/ { next }
    { print }
  ' "$config" | sed '/^[[:space:]]*$/d' | sha256sum | cut -d' ' -f1
}

run_lifecycle() {
  env -i \
    HOME="$real_home" \
    META_ROOT="$meta" \
    ENVCTL_REAL_HOME="$real_home" \
    ENVCTL_SOURCE_ROOT="$source_root" \
    PATH=/usr/bin:/bin \
    "$fixture_lifecycle" "$@"
}

if run_lifecycle detect >"$tmp/missing.out" 2>"$tmp/missing.err"; then
  fail "detect accepted a missing active-home config"
fi
grep -Fq 'missing active-home Codex config' "$tmp/missing.err" \
  || fail "missing config refusal was not explicit"
assert_absent "$meta/var/lib/envctl/legacy-archives"

write_valid_config
if run_lifecycle detect >"$tmp/policy-missing.out" 2>"$tmp/policy-missing.err"; then
  fail "detect accepted missing active-home RTK policy projections"
fi
grep -Fq 'missing or drifted active-home RTK policy projection' \
  "$tmp/policy-missing.err" \
  || fail "missing RTK policy projection refusal was not explicit"

write_unpinned_config
unrelated_before="$(unrelated_config_fingerprint)"
if run_lifecycle detect >"$tmp/features.out" 2>"$tmp/features.err"; then
  fail "detect accepted implicit enabled plugin features"
fi
grep -Fq 'features.plugins must be explicitly false' "$tmp/features.err" \
  || fail "missing plugin-disable refusal was not explicit"
install -d -m 700 "$real_home/.codex/plugins"
: >"$meta/global-actions.log"
run_lifecycle fix >/dev/null
grep -Fqx 'plugins = false' "$config" \
  || fail "fix did not disable the stable plugins feature"
grep -Fqx 'remote_plugin = false' "$config" \
  || fail "fix did not disable remote plugin discovery"
grep -Fqx 'model = "gpt-5.6-sol"' "$config" \
  || fail "feature repair changed the unrelated model setting"
grep -Fqx '[apps.connector_fixture.tools."github.create_pull_request"]' "$config" \
  || fail "feature repair removed unrelated connected-app settings"
grep -Fqx 'url = "https://developers.openai.com/mcp"' "$config" \
  || fail "feature repair changed the allowed remote MCP"
[ "$(unrelated_config_fingerprint)" = "$unrelated_before" ] \
  || fail "feature repair changed an unrelated editable config line"
assert_absent "$real_home/.codex/plugins"
for policy in \
  .codex/AGENTS.md \
  .codex/RTK.md \
  .codex/AGENTS.rtk.md \
  AGENTS.md \
  AGENTS.rtk.md; do
  cmp -s "$source_root/home/$policy" "$real_home/$policy" \
    || fail "fix did not project tracked RTK policy: $policy"
  [ "$(stat -c '%a' "$real_home/$policy")" = 600 ] \
    || fail "projected RTK policy is not private: $policy"
done
expected_actions="$(cat <<'ACTIONS'
profile:fix
feature:plugins:shadow-present
feature:remote_plugin:shadow-present
profile:verify
ACTIONS
)"
[ "$(cat "$meta/global-actions.log")" = "$expected_actions" ] \
  || fail "feature repair/profile/shadow ordering drifted"
repaired_hash="$(sha256sum "$config" | cut -d' ' -f1)"
archives_before="$(find "$meta/var/lib/envctl/legacy-archives" -mindepth 1 -maxdepth 1 -type d | wc -l)"
run_lifecycle fix >/dev/null
[ "$(sha256sum "$config" | cut -d' ' -f1)" = "$repaired_hash" ] \
  || fail "idempotent feature repair rewrote active config"
archives_after="$(find "$meta/var/lib/envctl/legacy-archives" -mindepth 1 -maxdepth 1 -type d | wc -l)"
[ "$archives_before" -eq "$archives_after" ] \
  || fail "idempotent feature repair created an empty archive"
[ "$(grep -c '^feature:' "$meta/global-actions.log")" -eq 2 ] \
  || fail "idempotent repair re-ran already-converged official feature commands"

printf 'stale local policy\n' >"$real_home/.codex/AGENTS.rtk.md"
if run_lifecycle detect >"$tmp/policy-drift.out" 2>"$tmp/policy-drift.err"; then
  fail "detect accepted a drifted active-home RTK policy projection"
fi
grep -Fq 'missing or drifted active-home RTK policy projection' \
  "$tmp/policy-drift.err" \
  || fail "drifted RTK policy projection refusal was not explicit"
archives_before="$(find "$meta/var/lib/envctl/legacy-archives" -mindepth 1 -maxdepth 1 -type d | wc -l)"
run_lifecycle fix >/dev/null
cmp -s "$source_root/home/.codex/AGENTS.rtk.md" \
  "$real_home/.codex/AGENTS.rtk.md" \
  || fail "fix did not restore the drifted RTK policy projection"
archives_after="$(find "$meta/var/lib/envctl/legacy-archives" -mindepth 1 -maxdepth 1 -type d | wc -l)"
[ "$archives_after" -eq "$((archives_before + 1))" ] \
  || fail "drifted RTK policy was not archived exactly once"
find "$meta/var/lib/envctl/legacy-archives" \
  -path '*/active-home/.codex/AGENTS.rtk.md' -type f -print -quit \
  | grep -q . || fail "drifted RTK policy archive is missing"

# TOML permits a trailing comment on a table header. Policy validation already
# accepts this form; transactional repair must preserve it and must not mistake
# its own two feature edits for unrelated-config drift.
write_unpinned_config
cat >>"$config" <<'TOML'

[features] # operator note must survive official repair
TOML
run_lifecycle fix >/dev/null
grep -Fqx '[features] # operator note must survive official repair' "$config" \
  || fail "feature repair dropped a valid features-table comment"
grep -Fqx 'plugins = false' "$config" \
  || fail "commented features table did not converge plugins=false"
grep -Fqx 'remote_plugin = false' "$config" \
  || fail "commented features table did not converge remote_plugin=false"
write_valid_config

before_hash="$(sha256sum "$config" | cut -d' ' -f1)"
[ "$(run_lifecycle detect)" = "" ] || fail "healthy detect emitted output"
[ "$(run_lifecycle verify)" = "codex-global: verified editable active-home config and shadow-free runtime" ] \
  || fail "verify did not report the active-home ownership contract"
[ "$(sha256sum "$config" | cut -d' ' -f1)" = "$before_hash" ] \
  || fail "validation rewrote editable config"

cat >>"$config" <<'TOML'

[mcp_servers.exa]
url = "https://mcp.exa.ai/mcp"
TOML
run_lifecycle verify >/dev/null \
  || fail "global policy rejected the project-compatible remote exa entry"
write_valid_config

cat >>"$config" <<'TOML'

[mcp_servers.openaiDeveloperDocs]
url = "https://developers.openai.com/mcp"
TOML
if run_lifecycle detect >/dev/null 2>&1; then
  fail "detect accepted a duplicate allowed MCP table"
fi
write_valid_config

cat >>"$config" <<'TOML'

[mcp_servers.exa.headers]
Authorization = "forbidden"
TOML
if run_lifecycle detect >/dev/null 2>&1; then
  fail "detect accepted nested state under an allowed MCP name"
fi
write_valid_config

cat >"$config" <<'TOML'
model = "gpt-5.6-sol"
mcp_servers = { exa = { url = "https://mcp.exa.ai/mcp" } }
TOML
chmod 600 "$config"
if run_lifecycle detect >/dev/null 2>&1; then
  fail "detect accepted inline MCP runtime authority"
fi
write_valid_config

write_unpinned_config
install -d -m 700 "$real_home/.codex/plugins"
: >"$meta/feature-no-converge"
no_converge_hash="$(sha256sum "$config" | cut -d' ' -f1)"
if run_lifecycle fix >"$tmp/no-converge.out" 2>"$tmp/no-converge.err"; then
  fail "fix accepted a non-convergent official feature command"
fi
grep -Fq 'official feature disable did not converge: plugins' "$tmp/no-converge.err" \
  || fail "non-convergent official feature refusal was not explicit"
[ -d "$real_home/.codex/plugins" ] \
  || fail "shadow cleanup ran before official feature convergence"
[ "$(sha256sum "$config" | cut -d' ' -f1)" = "$no_converge_hash" ] \
  || fail "non-convergent feature transaction did not restore exact config bytes"
grep -Fqx 'model = "gpt-5.6-sol"' "$config" \
  || fail "non-convergent feature command changed unrelated config"
rm -f "$meta/feature-no-converge"
rm -rf "$real_home/.codex/plugins"
write_valid_config

write_unpinned_config
install -d -m 700 "$real_home/.codex/plugins"
: >"$meta/feature-corrupt-unrelated"
rollback_hash="$(sha256sum "$config" | cut -d' ' -f1)"
if run_lifecycle fix >"$tmp/corrupt.out" 2>"$tmp/corrupt.err"; then
  fail "fix accepted an official feature command that changed unrelated config"
fi
grep -Fq 'official feature repair changed unrelated editable config' "$tmp/corrupt.err" \
  || fail "unrelated-config mutation refusal was not explicit"
[ "$(sha256sum "$config" | cut -d' ' -f1)" = "$rollback_hash" ] \
  || fail "feature transaction did not roll back the exact editable config"
[ -d "$real_home/.codex/plugins" ] \
  || fail "shadow cleanup ran after a rolled-back feature transaction"
rm -f "$meta/feature-corrupt-unrelated"
rm -rf "$real_home/.codex/plugins"
write_valid_config

cat >>"$config" <<'TOML'

[mcp_servers.context7]
command = "bunx"
args = ["@upstash/context7-mcp"]
TOML
hostile_hash="$(sha256sum "$config" | cut -d' ' -f1)"
if run_lifecycle fix >"$tmp/local-launch.out" 2>"$tmp/local-launch.err"; then
  fail "fix accepted a forbidden local-launch MCP"
fi
grep -Fq 'forbidden active-home MCP server: context7' "$tmp/local-launch.err" \
  || fail "local-launch MCP refusal did not name the forbidden server"
[ "$(sha256sum "$config" | cut -d' ' -f1)" = "$hostile_hash" ] \
  || fail "fix rewrote hostile editable config instead of failing closed"
write_valid_config

sed -i 's#url = "https://developers.openai.com/mcp"#command = "openai-docs-mcp"#' "$config"
if run_lifecycle detect >"$tmp/allowed-local.out" 2>"$tmp/allowed-local.err"; then
  fail "detect accepted a local launcher under an allowed MCP name"
fi
grep -Fq 'must contain only its canonical remote URL' "$tmp/allowed-local.err" \
  || fail "allowed-name local-launch refusal was not explicit"
write_valid_config

cat >>"$config" <<'TOML'

[marketplaces.stale]
source = "/tmp/stale"

[plugins."stale@marketplace"]
enabled = true
TOML
if run_lifecycle detect >"$tmp/plugin-table.out" 2>"$tmp/plugin-table.err"; then
  fail "detect accepted marketplace/plugin runtime authority"
fi
grep -Fq 'forbidden active-home plugin or marketplace table' "$tmp/plugin-table.err" \
  || fail "plugin-table refusal was not explicit"
write_valid_config

mv "$config" "$tmp/config.real"
ln -s "$tmp/config.real" "$config"
if run_lifecycle detect >/dev/null 2>&1; then
  fail "detect accepted a symlinked active-home config"
fi
rm "$config"
mv "$tmp/config.real" "$config"
chmod 644 "$config"
if run_lifecycle detect >/dev/null 2>&1; then
  fail "detect accepted group/world-readable active-home config"
fi
chmod 600 "$config"

install -d -m 700 \
  "$real_home/.codex/plugins/cache" \
  "$real_home/.codex/.tmp/plugins" \
  "$real_home/.codex/cache/remote_plugin_catalog" \
  "$real_home/.codex/tmp/marketplace-bundle" \
  "$real_home/.codex/sessions/keep" \
  "$real_home/.local/state/oh-my-codex" \
  "$real_home/.local/share/codex-binary-backups"
: >"$real_home/.codex/plugins/cache/catalog"
: >"$real_home/.codex/.tmp/plugins.sha"
: >"$real_home/.codex/.tmp/plugins.sync.lock"
: >"$real_home/.codex/cache/remote_plugin_catalog/catalog.json"
: >"$real_home/.codex/tmp/marketplace-bundle/plugin.json"
: >"$real_home/.codex/sessions/keep/session.jsonl"
: >"$real_home/.codex/auth.json"
: >"$real_home/.local/state/oh-my-codex/state.json"
: >"$real_home/.local/share/codex-binary-backups/old-codex"
chmod 600 "$real_home/.codex/auth.json"

if run_lifecycle detect >"$tmp/shadows.out" 2>"$tmp/shadows.err"; then
  fail "detect accepted forbidden plugin/cache runtime shadows"
fi
grep -Fq 'forbidden Codex runtime shadow' "$tmp/shadows.err" \
  || fail "runtime-shadow refusal was not explicit"
before_hash="$(sha256sum "$config" | cut -d' ' -f1)"
run_lifecycle fix >/dev/null
grep -Fqx fix "$meta/profile-actions.log" \
  || fail "fix did not validate/repair through the profile owner first"
for path in \
  "$real_home/.codex/plugins" \
  "$real_home/.codex/.tmp/plugins" \
  "$real_home/.codex/.tmp/plugins.sha" \
  "$real_home/.codex/.tmp/plugins.sync.lock" \
  "$real_home/.codex/cache/remote_plugin_catalog" \
  "$real_home/.codex/tmp/marketplace-bundle" \
  "$real_home/.local/state/oh-my-codex" \
  "$real_home/.local/share/codex-binary-backups"; do
  assert_absent "$path"
done
[ -f "$real_home/.codex/sessions/keep/session.jsonl" ] \
  || fail "fix removed unrelated generated session state"
[ -f "$real_home/.codex/auth.json" ] \
  || fail "fix removed active authentication state"
[ "$(sha256sum "$config" | cut -d' ' -f1)" = "$before_hash" ] \
  || fail "shadow cleanup rewrote editable config"
find "$meta/var/lib/envctl/legacy-archives" -path '*/active-home/.codex/plugins' -type d -print -quit \
  | grep -q . || fail "fix did not archive the plugin runtime shadow"
find "$meta/var/lib/envctl/legacy-archives" -path '*/active-home/.local/state/oh-my-codex' -type d -print -quit \
  | grep -q . || fail "fix did not archive the legacy plugin state"
run_lifecycle verify >/dev/null

archives_before="$(find "$meta/var/lib/envctl/legacy-archives" -mindepth 1 -maxdepth 1 -type d | wc -l)"
run_lifecycle fix >/dev/null
archives_after="$(find "$meta/var/lib/envctl/legacy-archives" -mindepth 1 -maxdepth 1 -type d | wc -l)"
[ "$archives_before" -eq "$archives_after" ] \
  || fail "idempotent fix created an empty archive"

install -d -m 700 "$real_home/.codex/plugins"
cat >>"$config" <<'TOML'

[mcp_servers.github]
url = "https://example.invalid/mcp"
TOML
if run_lifecycle fix >/dev/null 2>&1; then
  fail "fix cleaned shadows despite invalid editable config"
fi
[ -d "$real_home/.codex/plugins" ] \
  || fail "fail-closed config validation mutated runtime shadows"
write_valid_config
run_lifecycle remove >/dev/null
assert_absent "$real_home/.codex/plugins"
[ -f "$config" ] || fail "remove deleted the editable active-home config"
if grep -Fqx remove "$meta/profile-actions.log"; then
  fail "global remove delegated destructive profile removal"
fi

rm -rf "${meta:?}/var"
archive_escape="$tmp/archive-escape"
install -d -m 755 "$archive_escape"
ln -s "$archive_escape" "$meta/var"
install -d -m 700 "$real_home/.codex/plugins"
if run_lifecycle fix >/dev/null 2>&1; then
  fail "fix accepted a symlinked archive parent"
fi
[ -d "$real_home/.codex/plugins" ] \
  || fail "unsafe archive refusal moved a runtime shadow"
[ -z "$(find "$archive_escape" -mindepth 1 -print -quit)" ] \
  || fail "unsafe archive refusal wrote through the symlinked parent"
rm "$meta/var"
install -d -m 755 "$meta/var"
run_lifecycle fix >/dev/null

external_cache="$tmp/external-cache"
install -d -m 700 "$external_cache/remote_plugin_catalog"
rm -rf "$real_home/.codex/cache"
ln -s "$external_cache" "$real_home/.codex/cache"
if run_lifecycle detect >/dev/null 2>&1; then
  fail "detect accepted a symlinked generated cache root"
fi
run_lifecycle fix >/dev/null
assert_absent "$real_home/.codex/cache"
[ -d "$external_cache/remote_plugin_catalog" ] \
  || fail "cache-shadow retirement followed and modified a symlink target"

install -d -m 700 "$real_home/.codex/plugins"
rm -f "$meta/profile-ready"
if run_lifecycle fix >/dev/null 2>&1; then
  fail "fix accepted a missing profile-owned Codex runtime"
fi
[ -d "$real_home/.codex/plugins" ] \
  || fail "profile failure did not stop shadow mutation"

echo "PASS: Codex global baseline preserves editable config and archives only forbidden runtime shadows"
