#!/usr/bin/env bash
set -euo pipefail

ROOT="$(git rev-parse --show-toplevel)"
GATE="$ROOT/ci/gates/systemd-user-ownership.sh"
TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT

mkdir -p \
  "$TMP/manifest/components.d" \
  "$TMP/home/.config/systemd/user" \
  "$TMP/crates/engine/src"

write_typed_owner() {
  local path="$1" unit="$2"
  cat >"$path" <<EOF
[[component]]
id = "$unit"
[[component.wiring.systemd_user]]
name = "$unit"
enable = true
content = "unit"
EOF
}

write_typed_owner "$TMP/manifest/env-ctl.toml" env-ctl.service
write_typed_owner "$TMP/manifest/sqld.toml" sqld.service
cat >"$TMP/manifest/components.d/epic-h-toolchains.toml" <<'EOF'
[[component]]
id = "kache"
description = "Profile-owned validation only; no envctl systemd unit projection."
EOF
cat >"$TMP/manifest/components.d/portability-links.toml" <<'EOF'
[[component]]
id = "home-config-links"
description = "Systemd units are excluded; the wiring engine owns discovery."
EOF
cat >"$TMP/crates/engine/src/layout.rs" <<'EOF'
impl MetaLayout {
    pub fn systemd_user_dir(&self) {}
}
EOF
cat >"$TMP/crates/engine/src/wiring.rs" <<'EOF'
fn contract(u: &Unit, canonical: &Path, bridge: &Path) -> Result<()> {
    let layout = MetaLayout::from_env_required()?;
    real_user_xdg_config_home(&layout)?;
    ensure_owned_systemd_bridge_or_absent(bridge, canonical)?;
    std::os::unix::fs::symlink(canonical, bridge)?;
    let property = "--property=FragmentPath";
    run_systemctl(&["--user", "daemon-reload"])?;
    run_systemctl(&["--user", "enable", "--now", &u.name])?;
    Ok(())
}
// ============================== systemd --user ==============================
// ================================ apt repos =================================
EOF

baseline="$($GATE "$TMP")"
grep -Fq 'SYSTEMD USER OWNERSHIP GATE PASS' <<<"$baseline"

expect_failure() {
  local label="$1" needle="$2"
  shift 2
  local output
  if output="$($GATE "$TMP" 2>&1)"; then
    echo "expected gate failure for $label" >&2
    exit 1
  fi
  grep -Fq "$needle" <<<"$output" || {
    echo "wrong diagnostic for $label: $output" >&2
    exit 1
  }
  "$@"
}

printf '[Service]\n' >"$TMP/home/.config/systemd/user/env-ctl.service"
expect_failure home-projection 'tracked home-tree unit projection' \
  rm "$TMP/home/.config/systemd/user/env-ctl.service"

write_typed_owner "$TMP/manifest/components.d/duplicate.toml" env-ctl.service
expect_failure duplicate-owner 'env-ctl.service has duplicate or wrong owners' \
  rm "$TMP/manifest/components.d/duplicate.toml"

write_typed_owner "$TMP/manifest/components.d/forbidden-profile-unit.toml" kache.service
expect_failure profile-owned-unit 'unregistered active unit projection' \
  rm "$TMP/manifest/components.d/forbidden-profile-unit.toml"

cat >"$TMP/manifest/components.d/imperative.toml" <<'EOF'
[[component]]
id = "imperative"
[component.install]
kind = "script"
script = '''cat >"$M/.config/systemd/user/foreign.service" <<UNIT'''
EOF
expect_failure imperative-owner 'imperative systemd unit materializer' \
  rm "$TMP/manifest/components.d/imperative.toml"

cp "$TMP/manifest/env-ctl.toml" "$TMP/manifest/env-ctl.toml.clean"
printf '\n# ExecStart=%%h/Desktop/meta/usr/bin/secretd\n' >>"$TMP/manifest/env-ctl.toml"
expect_failure retired-path 'contains retired workstation path' \
  mv "$TMP/manifest/env-ctl.toml.clean" "$TMP/manifest/env-ctl.toml"

mv "$TMP/manifest/sqld.toml" "$TMP/sqld.toml.saved"
expect_failure missing-owner 'missing expected owner manifest/sqld.toml for sqld.service' \
  mv "$TMP/sqld.toml.saved" "$TMP/manifest/sqld.toml"

cp "$TMP/manifest/components.d/portability-links.toml" "$TMP/portability-links.toml.saved"
cat >"$TMP/manifest/components.d/portability-links.toml" <<'EOF'
[[component]]
id = "home-config-links"
[component.install]
kind = "script"
script = '''
link .config/systemd/user/env-ctl.service
'''
EOF
expect_failure portability-owner 'portability-links must not materialize systemd user units' \
  mv "$TMP/portability-links.toml.saved" "$TMP/manifest/components.d/portability-links.toml"

final="$($GATE "$TMP")"
grep -Fq 'SYSTEMD USER OWNERSHIP GATE PASS' <<<"$final"
echo "SYSTEMD USER OWNERSHIP GATE TEST PASS"
