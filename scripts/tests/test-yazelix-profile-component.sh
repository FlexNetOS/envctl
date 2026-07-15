#!/usr/bin/env bash
set -euo pipefail
umask 077

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
lifecycle="$root/assets/scripts/envctl-yazelix-profile-lifecycle.sh"
manifest="$root/manifest/nix-yazelix.toml"
tmp="$(mktemp -d)"
trap '/usr/bin/rm -rf --one-file-system -- "$tmp"' EXIT

# shellcheck source=../../assets/scripts/envctl-yazelix-profile-lifecycle.sh
# shellcheck disable=SC1091
source "$lifecycle"

uid="$(id -u)"
meta="$tmp/meta"
real="$tmp/home"
store_root="$tmp/nix/store"
fake_candidate="$store_root/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa-lifeos-foundation-yzx"
foreign_store="$store_root/bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb-foreign"
state="$tmp/profile.json"
generation_counter="$tmp/generation-counter"
build_counter="$tmp/build-counter"
add_counter="$tmp/add-counter"
remove_counter="$tmp/remove-counter"

mkdir -p "$meta/src/yazelix" "$real/.local/state/nix/profiles" \
  "$store_root" "$fake_candidate/bin" "$fake_candidate/toolbin" \
  "$fake_candidate/nushell/config" "$foreign_store"
printf '{ outputs = _: {}; }\n' >"$meta/src/yazelix/flake.nix"
printf '{"nodes":{},"root":"root","version":7}\n' >"$meta/src/yazelix/flake.lock"
printf 0 >"$generation_counter"
printf 0 >"$build_counter"
printf 0 >"$add_counter"
printf 0 >"$remove_counter"

make_executable() {
  local path="$1" label="$2"
  printf '%s\n' '#!/bin/sh' "printf '%s\\n' '$label'" >"$path"
  chmod 755 "$path"
}

make_executable "$fake_candidate/bin/yzx" 'yzx fixture'
make_executable "$fake_candidate/toolbin/codex" 'codex fixture'
make_executable "$fake_candidate/toolbin/rtk" 'rtk fixture'
make_executable "$fake_candidate/toolbin/nu" 'nu fixture'
make_executable "$fake_candidate/bin/yzx-desktop-launch" 'desktop fixture'
make_executable "$fake_candidate/bin/yzx-agent-workspace-launch" 'agent workspace fixture'
ln -s ../toolbin/codex "$fake_candidate/bin/codex"
ln -s ../toolbin/rtk "$fake_candidate/bin/rtk"
printf '%s\n' 'use rtk_wrappers.nu *' >"$fake_candidate/nushell/config/config.nu"
printf '%s\n' \
  'export def --wrapped codex [...rest] { ^rtk codex ...$rest }' \
  'export def --wrapped cargo [...rest] { ^rtk cargo ...$rest }' \
  >"$fake_candidate/nushell/config/rtk_wrappers.nu"
mkdir -p "$fake_candidate/share/applications" "$fake_candidate/share/yazelix"
write_desktop_fixture() {
  printf '%s\n' \
    '[Desktop Entry]' \
    'Version=1.4' \
    'Type=Application' \
    'Name=New Yazelix - Kitty' \
    'Comment=Yazi + Zellij + Helix integrated terminal environment' \
    'Icon=yazelix' \
    'StartupWMClass=com.yazelix.Yazelix' \
    'Terminal=false' \
    'X-Yazelix-Managed=true' \
    'Exec=/usr/bin/env sh -lc "exec ~/.nix-profile/bin/yzx-desktop-launch"' \
    'Categories=Development;' \
    >"$fake_candidate/share/applications/com.yazelix.Yazelix.Kitty.desktop"
  printf '%s\n' \
    '[Desktop Entry]' \
    'Version=1.4' \
    'Type=Application' \
    'Name=FlexNetOS Yazelix Agent' \
    'Comment=Yazelix Kitty with FlexNetOS agent workspace layout' \
    'Icon=yazelix' \
    'StartupWMClass=com.yazelix.Yazelix' \
    'Terminal=false' \
    'X-FlexNetOS-Managed=true' \
    'Exec=/usr/bin/env sh -lc "exec ~/.nix-profile/bin/yzx-agent-workspace-launch"' \
    'Categories=Development;' \
    >"$fake_candidate/share/applications/com.flexnetos.Yazelix.Agent.desktop"
}
write_desktop_fixture
for size in 48x48 64x64 128x128 256x256; do
  mkdir -p "$fake_candidate/share/icons/hicolor/$size/apps"
  printf 'fixture icon %s\n' "$size" \
    >"$fake_candidate/share/icons/hicolor/$size/apps/yazelix.png"
done
printf '%s\n' '{"name":"Yazelix Nova","version":"1.0.0-test"}' \
  >"$fake_candidate/share/yazelix/runtime_identity.json"

increment() {
  local file="$1" value
  value="$(<"$file")"
  printf '%s\n' "$((value + 1))" >"$file"
}

fake_activate_profile() {
  local number profile_store selector owned_store entry relative
  number="$(( $(<"$generation_counter") + 1 ))"
  printf '%s\n' "$number" >"$generation_counter"
  profile_store="$store_root/cccccccccccccccccccccccccccccc${number}-profile"
  mkdir -p "$profile_store/bin" "$profile_store/toolbin" "$profile_store/share"
  printf 'foreign fixture\n' >"$profile_store/share/foreign"
  owned_store="$(jq -r '.elements.lifeos_foundation_yzx.storePaths[0] // empty' "$state")"
  if [ -n "$owned_store" ]; then
    while IFS= read -r -d '' entry; do
      relative="${entry#"$owned_store/"}"
      mkdir -p "$profile_store/$(dirname "$relative")"
      ln -s "$entry" "$profile_store/$relative"
    done < <(find "$owned_store/bin" "$owned_store/toolbin" -mindepth 1 -maxdepth 1 -print0)
    while IFS= read -r -d '' entry; do
      relative="${entry#"$owned_store/"}"
      mkdir -p "$profile_store/$(dirname "$relative")"
      ln -s "$entry" "$profile_store/$relative"
    done < <(find "$owned_store/share" -type f -print0)
    for relative in nushell/config/config.nu nushell/config/rtk_wrappers.nu; do
      mkdir -p "$profile_store/$(dirname "$relative")"
      ln -s "$owned_store/$relative" "$profile_store/$relative"
    done
  fi
  selector="profile-${number}-link"
  ln -s "$profile_store" "$real/.local/state/nix/profiles/$selector"
  ln -sfn "$selector" "$real/.local/state/nix/profiles/profile"
}

write_foreign_only_state() {
  jq -n --arg store "$foreign_store" '{
    elements: {
      "fira-code": {
        active: true,
        priority: 5,
        storePaths: [$store]
      }
    },
    version: 3
  }' >"$state"
}

write_empty_state() {
  printf '%s\n' '{"elements":{},"version":3}' >"$state"
}

write_exact_owned_state() {
  local source="$1"
  jq --arg source "path:$source" \
    --arg attr "packages.x86_64-linux.lifeos_foundation_yzx" \
    --arg store "$fake_candidate" '.elements.lifeos_foundation_yzx = {
      active: true,
      attrPath: $attr,
      originalUrl: $source,
      outputs: null,
      priority: 4,
      storePaths: [$store],
      url: $source
    }' "$state" >"$state.next"
  mv "$state.next" "$state"
}

# Hermetic Nix seam. The production lifecycle always invokes the absolute
# Determinate-Nix binary; sourcing the lifecycle lets this test drive the same
# transaction logic over fake profile/store generations without host mutation.
yazelix_nix() {
  local first="${1:-}" second="${2:-}"
  if [ "$first" = build ]; then
    increment "$build_counter"
    printf '%s\n' "$fake_candidate"
    return 0
  fi
  if [ "$first" = profile ] && [ "$second" = list ]; then
    cat "$state"
    return 0
  fi
  if [ "$first" = profile ] && [ "$second" = add ]; then
    increment "$add_counter"
    write_exact_owned_state "$YAZELIX_SOURCE"
    fake_activate_profile
    return 0
  fi
  if [ "$first" = profile ] && [ "$second" = remove ]; then
    increment "$remove_counter"
    jq 'del(.elements.lifeos_foundation_yzx)' "$state" >"$state.next"
    mv "$state.next" "$state"
    fake_activate_profile
    return 0
  fi
  if [ "$first" = profile ] && [ "$second" = rollback ]; then
    return 99
  fi
  printf 'unexpected fake nix argv: %s\n' "$*" >&2
  return 98
}

expect_failure() {
  local label="$1"
  shift
  if ( "$@" ) >"$tmp/$label.out" 2>"$tmp/$label.err"; then
    printf 'expected failure: %s\n' "$label" >&2
    exit 1
  fi
}

yazelix_setup "$meta" "$real" "$store_root"
write_empty_state
write_exact_owned_state "$meta/src/yazelix"
fake_activate_profile
mv "$real/.local/state/nix/profiles/profile" "$real/.local/state/nix/profile"
mv "$real/.local/state/nix/profiles/profile-1-link" \
  "$real/.local/state/nix/profile-1-link"
ln -s "$real/.local/state/nix/profile" "$real/.nix-profile"

# A sole-element legacy XDG selector is adopted transactionally: the canonical
# candidate is built before mutation, a new profiles/profile generation is
# created and verified, the frontdoor switches atomically, and legacy selector
# links are archived instead of remaining a parallel runtime owner.
yazelix_install_core "$uid" >"$tmp/install.out"
[ "$(<"$build_counter")" -eq 1 ]
[ "$(<"$add_counter")" -eq 1 ]
[ "$(<"$remove_counter")" -eq 0 ]
[ "$(readlink "$real/.nix-profile")" = "$real/.local/state/nix/profiles/profile" ]
[ ! -e "$real/.local/state/nix/profile" ] \
  && [ ! -L "$real/.local/state/nix/profile" ]
[ ! -e "$real/.local/state/nix/profile-1-link" ] \
  && [ ! -L "$real/.local/state/nix/profile-1-link" ]
find "$meta/var/lib/envctl/legacy-archives" -type l -name profile -print -quit | grep -q .
yazelix_only_foundation_element "$(<"$state")"
yazelix_validate_installed "$uid"
[ ! -e "$meta/.nix-profile" ] && [ ! -L "$meta/.nix-profile" ]

# Exact incumbent keeps its generation and mutation state; it still builds the source candidate
# so an edited path flake cannot leave stale bytes behind under the same originalUrl.
generation_before="$(readlink "$real/.local/state/nix/profiles/profile")"
yazelix_install_core "$uid" >"$tmp/idempotent.out"
[ "$(<"$build_counter")" -eq 2 ]
[ "$(<"$add_counter")" -eq 1 ]
[ "$(<"$remove_counter")" -eq 0 ]
[ "$(readlink "$real/.local/state/nix/profiles/profile")" = "$generation_before" ]

# A source-drifted element is upgraded through remove+add only after candidate proof.
jq '.elements.lifeos_foundation_yzx.originalUrl = "github:old/yazelix"
    | .elements.lifeos_foundation_yzx.url = "github:old/yazelix"' \
  "$state" >"$state.next"
mv "$state.next" "$state"
fake_activate_profile
yazelix_install_core "$uid" >"$tmp/upgrade.out"
[ "$(<"$build_counter")" -eq 3 ]
[ "$(<"$add_counter")" -eq 2 ]
[ "$(<"$remove_counter")" -eq 1 ]
yazelix_only_foundation_element "$(<"$state")"
yazelix_element_exact "$(<"$state")" "$fake_candidate"

# A second profile element is never a neutral state: the lifecycle rejects it
# before building or mutating the canonical foundation element.
jq '.elements["fira-code"] = {active: true, priority: 5, storePaths: ["/tmp/foreign"]}' \
  "$state" >"$state.next"
mv "$state.next" "$state"
fake_activate_profile
expect_failure parallel-profile-element yazelix_install_core "$uid"
[ "$(<"$build_counter")" -eq 3 ] && [ "$(<"$add_counter")" -eq 2 ]
jq 'del(.elements["fira-code"])' "$state" >"$state.next"
mv "$state.next" "$state"
fake_activate_profile
yazelix_validate_installed "$uid"

# Home Manager is part of the one foundation element; a separately installed
# Home Manager element is a second profile owner and must fail closed.
jq '.elements["home-manager"] = {
  "active": true,
  "priority": 5,
  "attrPath": "packages.x86_64-linux.home-manager",
  "originalUrl": "path:/foreign/home-manager",
  "storePaths": ["/nix/store/ffffffffffffffffffffffffffffffff-home-manager"]
}' "$state" >"$state.next"
mv "$state.next" "$state"
fake_activate_profile
expect_failure home-manager-parallel-profile-element yazelix_install_core "$uid"
jq 'del(.elements["home-manager"])' "$state" >"$state.next"
mv "$state.next" "$state"
fake_activate_profile
yazelix_validate_installed "$uid"

# Missing canonical source fails before Nix or profile mutation.
rm "$meta/src/yazelix/flake.lock"
expect_failure missing-source yazelix_install_core "$uid"
[ "$(<"$build_counter")" -eq 3 ] && [ "$(<"$add_counter")" -eq 2 ]
printf '{"nodes":{},"root":"root","version":7}\n' >"$meta/src/yazelix/flake.lock"

# Hostile profile and META_ROOT ownership paths are refused, not repaired in place.
rm "$real/.nix-profile"
ln -s "$tmp/foreign-profile" "$real/.nix-profile"
expect_failure hostile-frontdoor yazelix_install_core "$uid"
rm "$real/.nix-profile"
ln -s "$real/.local/state/nix/profiles/profile" "$real/.nix-profile"
ln -s "$tmp/foreign-profile" "$meta/.nix-profile"
expect_failure meta-profile-shadow yazelix_install_core "$uid"
rm "$meta/.nix-profile"

# A profile toolbin that no longer resolves to the owned element fails Detect.
active_generation="$(readlink -f "$real/.nix-profile")"
rm "$active_generation/toolbin/rtk"
ln -s "$foreign_store/rtk" "$active_generation/toolbin/rtk"
expect_failure hostile-toolbin yazelix_validate_installed "$uid"
fake_activate_profile

# The user-local entries are stale only when the active profile has a canonical,
# content-valid launcher. Missing or store-owner-bypassing profile desktop entries
# therefore fail before any shadow archival can occur.
active_generation="$(readlink -f "$real/.nix-profile")"
rm "$active_generation/share/applications/com.yazelix.Yazelix.Kitty.desktop"
expect_failure missing-profile-desktop yazelix_validate_installed "$uid"
fake_activate_profile
sed -i 's#^Exec=.*#Exec="/tmp/foreign/yzx" desktop launch#' \
  "$fake_candidate/share/applications/com.yazelix.Yazelix.Kitty.desktop"
expect_failure hostile-profile-desktop-exec yazelix_validate_installed "$uid"
write_desktop_fixture
yazelix_validate_installed "$uid"
active_generation="$(readlink -f "$real/.nix-profile")"
rm "$active_generation/share/applications/com.flexnetos.Yazelix.Agent.desktop"
expect_failure missing-agent-profile-desktop yazelix_validate_installed "$uid"
fake_activate_profile
rm "$fake_candidate/share/icons/hicolor/128x128/apps/yazelix.png"
expect_failure missing-profile-icon yazelix_validate_installed "$uid"
printf 'fixture icon 128x128\n' \
  >"$fake_candidate/share/icons/hicolor/128x128/apps/yazelix.png"
yazelix_validate_installed "$uid"

# Explicit repair archives stale user-bin and desktop shadows only after the
# profile-owned replacement has already passed. It does not rebuild a current element.
mkdir -p "$real/.local/bin" "$real/.local/share/applications"
make_executable "$real/.local/bin/yzx" 'legacy yzx shadow'
printf '[Desktop Entry]\nName=legacy Yazelix\n' \
  >"$real/.local/share/applications/com.yazelix.Yazelix.Kitty.desktop"
printf '[Desktop Entry]\nName=legacy FlexNetOS Yazelix Agent\n' \
  >"$real/.local/share/applications/com.flexnetos.Yazelix.Agent.desktop"
yazelix_install_core "$uid" >"$tmp/shadow-repair.out"
[ "$(<"$build_counter")" -eq 4 ] && [ "$(<"$add_counter")" -eq 2 ]
[ ! -e "$real/.local/bin/yzx" ] && [ ! -L "$real/.local/bin/yzx" ]
[ ! -e "$real/.local/share/applications/com.yazelix.Yazelix.Kitty.desktop" ]
[ ! -e "$real/.local/share/applications/com.flexnetos.Yazelix.Agent.desktop" ]
find "$meta/var/lib/envctl/legacy-archives" -type f -name yzx -print -quit | grep -q .
[ "$(find "$meta/var/lib/envctl/legacy-archives" -type f -name '*Yazelix*.desktop' | wc -l)" -eq 2 ]

# Remove drops the one exact owned element, and a second Remove is idempotent.
yazelix_remove_core "$uid" >"$tmp/remove.out"
[ "$(<"$remove_counter")" -eq 2 ]
jq -e '(.elements | length) == 0' "$state" >/dev/null
yazelix_require_profile_chain "$uid"
yazelix_remove_core "$uid" >"$tmp/remove-idempotent.out"
[ "$(<"$remove_counter")" -eq 2 ]

# The owned name is not sufficient authority to remove a foreign-source element.
write_exact_owned_state "$meta/src/yazelix"
jq '.elements.lifeos_foundation_yzx.originalUrl = "github:foreign/yazelix"
    | .elements.lifeos_foundation_yzx.url = "github:foreign/yazelix"' \
  "$state" >"$state.next"
mv "$state.next" "$state"
fake_activate_profile
expect_failure foreign-name-reuse yazelix_remove_core "$uid"
[ "$(<"$remove_counter")" -eq 2 ]

# Unsupported/malformed profile JSON fails closed before a candidate build.
printf '%s\n' '{"elements":[]}' >"$state"
expect_failure malformed-profile yazelix_install_core "$uid"
[ "$(<"$build_counter")" -eq 4 ]

# Manifest-level wiring must reach this lifecycle and must not restore the retired
# GitHub/Ghostty/user-desktop/META_ROOT-profile ownership model.
python3 - "$manifest" <<'PY'
import pathlib
import sys
import tomllib

path = pathlib.Path(sys.argv[1])
doc = tomllib.loads(path.read_text())
components = {item["id"]: item for item in doc["component"]}
yzx = components["yazelix"]
assert yzx["install"] == {
    "kind": "shipped_script",
    "path": "$ENVCTL_SOURCE_ROOT/assets/scripts/envctl-yazelix-profile-lifecycle.sh",
    "args": ["install"],
}
assert yzx["fix"]["args"] == ["fix"]
assert yzx["remove"]["args"] == ["remove"]
assert "lifeos_foundation_yzx" in yzx["description"]
assert "only element" in yzx["description"]
home_manager = components["home-manager"]
assert home_manager["requires"] == ["yazelix"]
assert home_manager["install"] == {
    "kind": "shipped_script",
    "path": "$ENVCTL_SOURCE_ROOT/assets/scripts/envctl-yazelix-profile-lifecycle.sh",
    "args": ["fix"],
}
assert home_manager["fix"] == home_manager["install"]
assert "remove" not in home_manager
assert "nix profile" not in repr(home_manager)
assert "yazelix-desktop" not in components
assert "yazelix-desktop" not in components["group-nix-yazelix"]["requires"]
text = path.read_text()
for retired in (
    "$META_ROOT/.nix-profile",
    "yzx desktop install",
    "github:FlexNetOS/yazelix#yazelix",
    "github:luccahuguet/yazelix#yazelix",
):
    assert retired not in text, retired
PY

bash -n "$lifecycle"
echo 'yazelix profile lifecycle fixture: PASS'
