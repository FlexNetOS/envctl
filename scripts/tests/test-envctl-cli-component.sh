#!/usr/bin/env bash
set -euo pipefail

fail() { echo "FAIL: $*" >&2; exit 1; }

root="$(git -C "$(dirname "${BASH_SOURCE[0]}")" rev-parse --show-toplevel)"
manifest="$root/manifest/components.d/envctl-cli.toml"
[ -f "$manifest" ] || fail "missing envctl CLI owner component"

tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT
meta="$tmp/meta"
repo="$tmp/repo"
loop_lib="$tmp/loop_lib"
protocol="$tmp/meta_plugin_protocol"
fake_bin="$tmp/fake-bin"
shadow_cargo_called="$tmp/shadow-cargo-called"
mkdir -p "$repo/crates/cli/src" "$loop_lib/src" "$protocol/src" "$fake_bin" "$meta/usr/bin" \
  "$meta/src/envctl/target/release" "$meta/.toolchains/cargo/bin" "$meta/.toolchains/rustup"
printf '[workspace]\nmembers = ["crates/cli"]\n' >"$repo/Cargo.toml"
printf '# fixture lock\n' >"$repo/Cargo.lock"
printf '[toolchain]\nchannel = "stable"\n' >"$repo/rust-toolchain.toml"
printf '[package]\nname = "envctl-fixture"\nversion = "0.1.0"\nedition = "2024"\n' \
  >"$repo/crates/cli/Cargo.toml"
printf 'fn main() {}\n' >"$repo/crates/cli/src/main.rs"
printf '[package]\nname = "loop_lib"\nversion = "0.1.0"\nedition = "2021"\n' \
  >"$loop_lib/Cargo.toml"
printf 'pub fn build_command() {}\n' >"$loop_lib/src/lib.rs"
printf '[package]\nname = "meta_plugin_protocol"\nversion = "0.1.0"\nedition = "2021"\n' \
  >"$protocol/Cargo.toml"
printf 'pub struct Protocol;\n' >"$protocol/src/lib.rs"
printf '#!/bin/sh\necho legacy\n' >"$meta/src/envctl/target/release/envctl"
chmod 755 "$meta/src/envctl/target/release/envctl"
ln -s "$meta/src/envctl/target/release/envctl" "$meta/usr/bin/envctl"

cat >"$meta/.toolchains/cargo/bin/cargo" <<'SH'
#!/usr/bin/env bash
set -euo pipefail
[ "${1:-}" = build ] || { echo "unexpected cargo command: $*" >&2; exit 1; }
mkdir -p "${CARGO_TARGET_DIR:?}/release"
printf '#!/bin/sh\necho envctl 0.1.0\n' >"$CARGO_TARGET_DIR/release/envctl"
chmod 755 "$CARGO_TARGET_DIR/release/envctl"
SH
chmod 755 "$meta/.toolchains/cargo/bin/cargo"

cat >"$meta/usr/bin/cargo" <<'SH'
#!/usr/bin/env bash
set -euo pipefail
: >"${SHADOW_CARGO_CALLED:?}"
echo "META_ROOT usr/bin cargo shadow must never be invoked" >&2
exit 99
SH
chmod 755 "$meta/usr/bin/cargo"

python3 - "$manifest" "$tmp" <<'PY'
import pathlib, sys, tomllib
data = tomllib.loads(pathlib.Path(sys.argv[1]).read_text())
component = next(c for c in data["component"] if c["id"] == "envctl-cli")
out = pathlib.Path(sys.argv[2])
for phase in ("detect", "install", "verify", "fix", "remove"):
    hook = component[phase]
    script = hook.get("script")
    if script is None:
        assert hook["command"] == "bash" and hook["args"][:1] == ["-lc"]
        script = hook["args"][1]
    path = out / f"{phase}.sh"
    path.write_text(script)
    path.chmod(0o755)
PY

export META_ROOT="$meta"
export ENVCTL_SOURCE_ROOT="$repo"
export SHADOW_CARGO_CALLED="$shadow_cargo_called"
export PATH="$fake_bin:/usr/bin:/bin"
unset CARGO_HOME RUSTUP_HOME

if bash "$tmp/detect.sh"; then fail "detect passed before install"; fi
bash "$tmp/install.sh"
bash "$tmp/detect.sh"
bash "$tmp/verify.sh"

private="$meta/usr/libexec/envctl/cli/bin/envctl"
receipt="$meta/usr/libexec/envctl/cli/install-receipt"
front="$meta/usr/bin/envctl"
[ -x "$private" ] && [ ! -L "$private" ] || fail "private payload is not a regular executable"
[ -f "$receipt" ] && [ ! -L "$receipt" ] || fail "install receipt is not a regular file"
[ -x "$front" ] && [ ! -L "$front" ] || fail "frontdoor is not a regular executable"
grep -Fqx '# managed-by: envctl component envctl-cli' "$receipt" \
  || fail "install receipt marker missing"
grep -Eq '^source_sha256=[0-9a-f]{64}$' "$receipt" \
  || fail "install receipt source identity missing"
grep -Eq '^payload_sha256=[0-9a-f]{64}$' "$receipt" \
  || fail "install receipt payload identity missing"
grep -Fqx '# managed-by: envctl component envctl-cli' "$front" || fail "frontdoor marker missing"
grep -Fqx "export META_ROOT=\"\${META_ROOT:-$meta}\"" "$front" || fail "META_ROOT default missing"
grep -Fqx "export ENVCTL_MANIFEST_DIR=\"\${ENVCTL_MANIFEST_DIR:-$meta/src/envctl/manifest}\"" "$front" \
  || fail "manifest default missing"
grep -Fqx "exec \"$private\" \"\$@\"" "$front" || fail "frontdoor target mismatch"
find "$meta/var/lib/envctl/legacy-archives" -type l -o -type f | grep -q '/usr/bin/envctl$' \
  || fail "legacy envctl symlink was not archived"

first_hash="$(sha256sum "$front" "$private" "$receipt")"
bash "$tmp/fix.sh"
[ "$(sha256sum "$front" "$private" "$receipt")" = "$first_hash" ] \
  || fail "fix is not idempotent"

# Required-line greps are insufficient: an injected command can coexist with every expected line.
# Detect/verify must require the byte-exact managed wrapper and must not execute the injection.
expected_front="$tmp/expected-front"
cp "$front" "$expected_front"
injection_ran="$tmp/frontdoor-injection-ran"
sed -i "/^exec /i : > \"$injection_ran\"" "$front"
if bash "$tmp/detect.sh"; then fail "detect accepted an injected managed frontdoor"; fi
if bash "$tmp/verify.sh"; then fail "verify accepted an injected managed frontdoor"; fi
[ ! -e "$injection_ran" ] || fail "verify executed an injected frontdoor command"
cp "$expected_front" "$front"
bash "$tmp/verify.sh"

# Refusal on a foreign frontdoor is all-or-nothing: preflight ownership before rebuilding or
# committing the private payload/receipt, even when canonical source has changed.
owned_before="$(sha256sum "$private" "$receipt")"
printf '#!/bin/sh\necho foreign-frontdoor\n' >"$front"
chmod 755 "$front"
foreign_before="$(sha256sum "$front")"
printf 'fn pending_generation() {}\n' >>"$repo/crates/cli/src/main.rs"
if bash "$tmp/fix.sh"; then fail "fix replaced a foreign regular frontdoor"; fi
[ "$(sha256sum "$private" "$receipt")" = "$owned_before" ] \
  || fail "failed fix changed the private payload or receipt"
[ "$(sha256sum "$front")" = "$foreign_before" ] \
  || fail "failed fix changed the foreign frontdoor"
cp "$expected_front" "$front"
bash "$tmp/fix.sh"
bash "$tmp/verify.sh"

# A source change that preserves the public version must invalidate the installed generation.
# This is the exact hostile state that made the absorbed agent-env verifier report false drift.
printf 'fn generation_two() {}\n' >>"$repo/crates/cli/src/main.rs"
[ "$($private --version)" = 'envctl 0.1.0' ] || fail "fixture version unexpectedly changed"
if bash "$tmp/detect.sh"; then fail "detect accepted a same-version stale payload"; fi
if bash "$tmp/verify.sh"; then fail "verify accepted a same-version stale payload"; fi
bash "$tmp/fix.sh"
bash "$tmp/detect.sh"
bash "$tmp/verify.sh"

# The envctl binary consumes required meta sibling path dependencies. Their source generation is
# part of the payload identity too, even though envctl's own package version remains unchanged.
printf 'pub fn generation_two() {}\n' >>"$loop_lib/src/lib.rs"
if bash "$tmp/detect.sh"; then fail "detect ignored loop_lib source generation drift"; fi
if bash "$tmp/verify.sh"; then fail "verify ignored loop_lib source generation drift"; fi
bash "$tmp/fix.sh"
bash "$tmp/verify.sh"

printf 'pub struct ProtocolV2;\n' >>"$protocol/src/lib.rs"
if bash "$tmp/detect.sh"; then fail "detect ignored meta_plugin_protocol source generation drift"; fi
if bash "$tmp/verify.sh"; then fail "verify ignored meta_plugin_protocol source generation drift"; fi
bash "$tmp/fix.sh"
bash "$tmp/verify.sh"

[ ! -e "$shadow_cargo_called" ] || fail "install/fix invoked the META_ROOT usr/bin cargo shadow"

# The receipt binds the payload bytes too: a same-version replacement is not trusted.
printf '#!/bin/sh\necho envctl 0.1.0\n# same-version tamper\n' >"$private"
chmod 755 "$private"
if bash "$tmp/detect.sh"; then fail "detect accepted a tampered same-version payload"; fi
if bash "$tmp/verify.sh"; then fail "verify accepted a tampered same-version payload"; fi
bash "$tmp/fix.sh"
bash "$tmp/verify.sh"

# Receipt corruption is fail-closed even when source and payload are otherwise intact.
sed -i 's/^source_sha256=./source_sha256=0/' "$receipt"
if bash "$tmp/detect.sh"; then fail "detect accepted a forged install receipt"; fi
if bash "$tmp/verify.sh"; then fail "verify accepted a forged install receipt"; fi
bash "$tmp/fix.sh"
bash "$tmp/verify.sh"

bash "$tmp/remove.sh"
[ ! -e "$front" ] && [ ! -L "$front" ] || fail "managed frontdoor survived remove"
[ ! -e "$private" ] || fail "managed private payload survived remove"
[ ! -e "$receipt" ] || fail "managed install receipt survived remove"

printf '#!/bin/sh\necho foreign\n' >"$front"
chmod 755 "$front"
if bash "$tmp/remove.sh"; then fail "remove accepted a foreign frontdoor"; fi
grep -q foreign "$front" || fail "remove deleted a foreign frontdoor"
[ ! -e "$shadow_cargo_called" ] || fail "a lifecycle hook invoked the META_ROOT usr/bin cargo shadow"

echo "PASS: envctl CLI component rejects stale/tampered generations and preserves foreign files"
