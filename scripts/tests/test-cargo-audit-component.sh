#!/usr/bin/env bash
set -euo pipefail

fail() { echo "FAIL: $*" >&2; exit 1; }

root="$(git -C "$(dirname "${BASH_SOURCE[0]}")" rev-parse --show-toplevel)"
manifest="$root/manifest/components.d/cargo-audit.toml"
[ -f "$manifest" ] || fail "missing cargo-audit component"

tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT
meta="$tmp/meta"
fake_bin="$tmp/fake-bin"
mkdir -p "$meta/usr/bin" "$fake_bin"
touch "$meta/.meta.yaml"

cat >"$fake_bin/cargo" <<'SH'
#!/usr/bin/env bash
set -euo pipefail
case "${1:-}" in
  install)
    shift
    root=""
    version=""
    while [ "$#" -gt 0 ]; do
      case "$1" in
        --root) root="$2"; shift 2 ;;
        --version) version="$2"; shift 2 ;;
        *) shift ;;
      esac
    done
    [ "$version" = 0.22.2 ] || { echo "wrong version: $version" >&2; exit 1; }
    mkdir -p "$root/bin"
    printf '#!/bin/sh\necho cargo-audit 0.22.2\n' >"$root/bin/cargo-audit"
    chmod 755 "$root/bin/cargo-audit"
    ;;
  uninstall)
    shift
    root=""
    while [ "$#" -gt 0 ]; do
      case "$1" in --root) root="$2"; shift 2 ;; *) shift ;; esac
    done
    rm -f "$root/bin/cargo-audit"
    ;;
  *) echo "unexpected cargo command: $*" >&2; exit 1 ;;
esac
SH
chmod 755 "$fake_bin/cargo"

python3 - "$manifest" "$tmp" <<'PY'
import pathlib, sys, tomllib
data = tomllib.loads(pathlib.Path(sys.argv[1]).read_text())
component = next(c for c in data["component"] if c["id"] == "cargo-audit")
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
export PATH="$fake_bin:/usr/bin:/bin"

if bash "$tmp/detect.sh"; then fail "detect passed before install"; fi
bash "$tmp/install.sh"
bash "$tmp/detect.sh"
bash "$tmp/verify.sh"

private="$meta/usr/libexec/envctl/cargo-audit/bin/cargo-audit"
front="$meta/usr/bin/cargo-audit"
[ -x "$private" ] && [ ! -L "$private" ] || fail "private cargo-audit missing"
[ -x "$front" ] && [ ! -L "$front" ] || fail "cargo-audit frontdoor is not regular"
grep -Fqx '# managed-by: envctl component cargo-audit' "$front" || fail "marker missing"
grep -Fqx "export CARGO_HOME=\"\${CARGO_HOME:-$meta/.toolchains/cargo}\"" "$front" \
  || fail "frontdoor does not force the meta-owned Cargo home"
grep -Fqx "exec \"$private\" \"\$@\"" "$front" || fail "frontdoor target mismatch"
[ "$($front --version)" = 'cargo-audit 0.22.2' ] || fail "wrong installed version"

# The documented bare CI gate must discover the envctl-owned frontdoor by META_ROOT even when the
# caller's ambient PATH contains neither META_ROOT/usr/bin nor a Cargo-home cargo-audit.
gate_log="$tmp/cargo-audit-gate.log"
if ! env -u CARGO_HOME META_ROOT="$meta" PATH=/usr/bin:/bin \
  bash "$root/ci/gates/cargo-audit.sh" >"$gate_log" 2>&1; then
  cat "$gate_log" >&2
  fail "cargo-audit gate did not discover META_ROOT/usr/bin/cargo-audit"
fi
grep -Fq 'CARGO-AUDIT GATE PASS' "$gate_log" || fail "cargo-audit gate did not complete"
grep -Fq -- '--git-common-dir' "$root/ci/gates/cargo-audit.sh" \
  || fail "cargo-audit gate cannot recover META_ROOT from an external worktree"
grep -Fq '.meta.yaml' "$root/ci/gates/cargo-audit.sh" \
  || fail "cargo-audit gate does not use the meta workspace marker"

# A real meta workspace must not fall through to an ambient/user cargo-audit shadow when its
# declared frontdoor is absent.
mv "$front" "$front.saved"
cat >"$fake_bin/cargo-audit" <<'SH'
#!/bin/sh
printf '%s\n' 'cargo-audit 0.22.2'
exit 0
SH
chmod 755 "$fake_bin/cargo-audit"
if env -u CARGO_HOME META_ROOT="$meta" PATH="$fake_bin:/usr/bin:/bin" \
  bash "$root/ci/gates/cargo-audit.sh" >"$gate_log" 2>&1; then
  fail "cargo-audit gate accepted an ambient shadow without its managed frontdoor"
fi
grep -Fq 'managed cargo-audit frontdoor is missing' "$gate_log" \
  || fail "cargo-audit gate did not fail for the ownership reason"
mv "$front.saved" "$front"
rm -f "$fake_bin/cargo-audit"

first_hash="$(sha256sum "$front" "$private")"
bash "$tmp/fix.sh"
[ "$(sha256sum "$front" "$private")" = "$first_hash" ] || fail "fix is not idempotent"

bash "$tmp/remove.sh"
[ ! -e "$front" ] && [ ! -L "$front" ] || fail "managed frontdoor survived remove"
[ ! -e "$private" ] || fail "private payload survived remove"

printf '#!/bin/sh\necho foreign\n' >"$front"
chmod 755 "$front"
bash "$tmp/remove.sh"
grep -q foreign "$front" || fail "remove deleted a foreign frontdoor"

grep -Fq 'cargo install cargo-audit --locked --version 0.22.2' "$tmp/install.sh" \
  || fail "install is not locked to cargo-audit 0.22.2"

echo "PASS: cargo-audit 0.22.2 component is pinned, idempotent, and META_ROOT-owned"
