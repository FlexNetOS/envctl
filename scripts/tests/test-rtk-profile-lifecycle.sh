#!/usr/bin/env bash
set -euo pipefail

ROOT="$(git -C "$(dirname "${BASH_SOURCE[0]}")" rev-parse --show-toplevel)"
LIFECYCLE="$ROOT/assets/scripts/envctl-rtk-profile-lifecycle.sh"

fail() {
  printf 'FAIL: %s\n' "$*" >&2
  exit 1
}

expect_failure() {
  local label="$1"
  shift
  if "$@" >"$tmp/$label.out" 2>"$tmp/$label.err"; then
    fail "expected failure: $label"
  fi
}

tmp="$(mktemp -d -t envctl-rtk-profile-contract.XXXXXXXX)"
trap 'rm -rf "$tmp"' EXIT
meta="$tmp/meta"
home="$tmp/home"
source_root="$tmp/source"
store="$tmp/nix/store"
package="$store/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa-rtk-0.43.0"
profile="$store/bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb-profile"
fixture_lifecycle="$source_root/assets/scripts/envctl-rtk-profile-lifecycle.sh"
yazelix_lifecycle="$source_root/assets/scripts/envctl-yazelix-profile-lifecycle.sh"

install -d -m 755 \
  "$meta" \
  "$home" \
  "$source_root/assets/scripts" \
  "$package/bin" \
  "$profile/bin" \
  "$profile/toolbin"
[ -x "$LIFECYCLE" ] || fail "missing executable lifecycle: $LIFECYCLE"
install -m 755 "$LIFECYCLE" "$fixture_lifecycle"

cat >"$yazelix_lifecycle" <<'YAZELIX'
#!/usr/bin/env bash
set -euo pipefail
printf '%s\n' "${1:?action required}" >>"$META_ROOT/yazelix-actions.log"
[ -f "$META_ROOT/profile-ready" ]
case "$1" in detect|verify|install|fix) ;; remove) exit 86 ;; *) exit 2 ;; esac
YAZELIX
chmod 755 "$yazelix_lifecycle"
: >"$meta/profile-ready"

cat >"$tmp/rtk_fixture.rs" <<'RUST'
fn main() {
    if std::env::args().skip(1).eq(["--version".to_string()]) {
        println!("rtk 0.43.0");
    } else {
        std::process::exit(2);
    }
}
RUST
# The active developer toolchain is profile-owned.  Do not depend on whatever
# inherited shell PATH a test runner happened to start with.
rustc_bin="${RUSTC:-$HOME/.nix-profile/bin/rustc}"
[ -x "$rustc_bin" ] || rustc_bin="$(command -v rustc 2>/dev/null || true)"
[ -n "$rustc_bin" ] && [ -x "$rustc_bin" ] || fail "no profile-owned rustc available for RTK fixture"
"$rustc_bin" "$tmp/rtk_fixture.rs" -o "$package/bin/rtk"
ln -s "$package/bin/rtk" "$profile/bin/rtk"
ln -s "$package/bin/rtk" "$profile/toolbin/rtk"
ln -s "$profile" "$home/.nix-profile"

run_lifecycle() {
  env -i \
    HOME="$home" \
    META_ROOT="$meta" \
    ENVCTL_REAL_HOME="$home" \
    ENVCTL_SOURCE_ROOT="$source_root" \
    ENVCTL_RTK_STORE_ROOT="$store" \
    PATH=/usr/bin:/bin \
    "$fixture_lifecycle" "$@"
}

[ "$(run_lifecycle detect)" = "" ] || fail "healthy detect emitted output"
[ "$(run_lifecycle verify)" = \
    "rtk-profile: verified one profile-owned RTK payload and no parallel shadows" ] \
  || fail "verify did not report the profile ownership contract"

# The RTK lifecycle must not accept a native binary when the Yazelix profile
# owner itself rejects the active profile. `rtk_profile_binary` is used as a
# conditional, so this specifically guards against Bash's conditional-errexit
# suppression.
rm "$meta/profile-ready"
expect_failure rejected-yazelix-owner run_lifecycle detect
: >"$meta/profile-ready"

mv "$package/bin/rtk" "$package/bin/rtk.elf"
printf '#!/bin/sh\nprintf "%s\\n" "rtk 0.43.0"\n' >"$package/bin/rtk"
chmod 755 "$package/bin/rtk"
expect_failure shebang-wrapper run_lifecycle detect
rm "$package/bin/rtk"
mv "$package/bin/rtk.elf" "$package/bin/rtk"

foreign="$store/cccccccccccccccccccccccccccccccc-foreign/bin"
install -d -m 755 "$foreign"
printf '#!/bin/sh\nexit 0\n' >"$foreign/rtk"
chmod 755 "$foreign/rtk"
rm "$profile/toolbin/rtk"
ln -s "$foreign/rtk" "$profile/toolbin/rtk"
expect_failure split-profile run_lifecycle detect
rm "$profile/toolbin/rtk"
ln -s "$package/bin/rtk" "$profile/toolbin/rtk"

install -d -m 755 \
  "$meta/.toolchains/cargo/bin" \
  "$meta/usr/bin" \
  "$home/.local/bin"
printf '#!/bin/sh\nexit 0\n' >"$meta/.toolchains/cargo/bin/rtk"
printf '#!/bin/sh\nexit 0\n' >"$meta/usr/bin/rtk"
printf '#!/bin/sh\nexit 0\n' >"$home/.local/bin/rtk"
chmod 755 \
  "$meta/.toolchains/cargo/bin/rtk" \
  "$meta/usr/bin/rtk" \
  "$home/.local/bin/rtk"
expect_failure parallel-shadows run_lifecycle detect
run_lifecycle fix >/dev/null
[ "$(tail -n 3 "$meta/yazelix-actions.log")" = $'fix\ndetect\ndetect' ] \
  || fail "fix did not repair through Yazelix before revalidating RTK"
for shadow in \
  "$meta/.toolchains/cargo/bin/rtk" \
  "$meta/usr/bin/rtk" \
  "$home/.local/bin/rtk"; do
  [ ! -e "$shadow" ] && [ ! -L "$shadow" ] \
    || fail "parallel RTK shadow survived repair: $shadow"
done
[ "$(find "$meta/var/lib/envctl/legacy-archives" -type f -name rtk | wc -l)" -eq 3 ] \
  || fail "repair did not archive every RTK shadow"
run_lifecycle verify >/dev/null

printf '#!/bin/sh\nexit 0\n' >"$home/.local/bin/rtk"
chmod 755 "$home/.local/bin/rtk"
run_lifecycle remove >/dev/null
[ -x "$home/.nix-profile/bin/rtk" ] || fail "remove deleted profile-owned RTK"
[ ! -e "$home/.local/bin/rtk" ] || fail "remove left a user-bin RTK shadow"
if grep -Fqx remove "$meta/yazelix-actions.log"; then
  fail "RTK remove delegated destructive ownership to Yazelix"
fi

python3 - "$ROOT/manifest/base.toml" <<'PY'
import pathlib
import sys
import tomllib

path = pathlib.Path(sys.argv[1])
components = {item["id"]: item for item in tomllib.loads(path.read_text())["component"]}
rtk = components["rtk"]
assert rtk["requires"] == ["yazelix"]
for phase in ("detect", "install", "verify", "fix", "remove"):
    assert rtk[phase] == {
        "kind": "shipped_script",
        "path": "$ENVCTL_SOURCE_ROOT/assets/scripts/envctl-rtk-profile-lifecycle.sh",
        "args": [phase],
    }
serialized = repr(rtk)
for forbidden in ("cargo install", "cargo uninstall", ".toolchains/cargo/bin:$PATH"):
    assert forbidden not in serialized, forbidden
PY

standalone_nu="$ROOT/home/.config/nushell/config.nu"
grep -Fqx 'use ~/.nix-profile/nushell/config/rtk_wrappers.nu *' "$standalone_nu" \
  || fail "standalone login Nu does not import the stable profile RTK module"
[ ! -e "$ROOT/home/.config/nushell/rtk-wrappers.nu" ] \
  || fail "envctl still carries a duplicate local Nu RTK module"
if grep -Rqs 'rtk-wrappers.nu' \
    "$ROOT/home/.config/yazelix/shell_nu.nu" \
    "$ROOT/manifest/components.d/portability-links.toml"; then
  fail "envctl still sources or projects the retired duplicate Nu RTK module"
fi
if grep -Eq 'cargo install .*rtk|rtk \(cargo install\)|rtk init -g' \
    "$ROOT/assets/scripts/yazelix-setup.sh"; then
  fail "first-login wizard still owns Cargo RTK installation or mutable init"
fi

bash -n "$LIFECYCLE"
printf '%s\n' 'PASS: RTK lifecycle keeps one Yazelix-profile payload and archives parallel shadows'
