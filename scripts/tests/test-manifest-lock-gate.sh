#!/usr/bin/env bash
# Hermetic regression test: the gate accepts a clean read-only check and rejects
# a command that mutates tracked manifest input even when that command exits 0.
set -euo pipefail

repo_root="$(git -C "$(dirname "${BASH_SOURCE[0]}")" rev-parse --show-toplevel)"
tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT

fixture="$tmp/repo"
mkdir -p "$fixture/ci/gates" "$fixture/manifest" "$fixture/bin"
cp "$repo_root/ci/gates/manifest-lock.sh" "$fixture/ci/gates/manifest-lock.sh"
printf '[[component]]\nid = "stub"\nname = "Stub"\n' >"$fixture/manifest/base.toml"
printf 'version = 1\n' >"$fixture/manifest/envctl.lock"

git -C "$fixture" init -q
git -C "$fixture" config user.email test@example.invalid
git -C "$fixture" config user.name test
git -C "$fixture" add manifest ci/gates/manifest-lock.sh
git -C "$fixture" commit -qm fixture

cat >"$fixture/bin/cargo" <<'EOF'
#!/usr/bin/env bash
set -euo pipefail
printf '%s\n' "$*" >"$FAKE_ARGS_FILE"
if [[ "${FAKE_MUTATE:-0}" == 1 ]]; then
  printf '# mutation\n' >>manifest/base.toml
fi
EOF
chmod +x "$fixture/bin/cargo"

FAKE_ARGS_FILE="$tmp/args" PATH="$fixture/bin:$PATH" \
  bash "$fixture/ci/gates/manifest-lock.sh" >"$tmp/clean.out" 2>"$tmp/clean.err"
grep -Fxq 'run --locked -p envctl -- --color never lock --check' "$tmp/args"
grep -Fq 'MANIFEST-LOCK GATE PASS' "$tmp/clean.out"

git -C "$fixture" checkout -q -- manifest/base.toml
set +e
FAKE_ARGS_FILE="$tmp/args-mutating" FAKE_MUTATE=1 PATH="$fixture/bin:$PATH" \
  bash "$fixture/ci/gates/manifest-lock.sh" >"$tmp/mutating.out" 2>"$tmp/mutating.err"
rc=$?
set -e
[[ "$rc" -ne 0 ]] || { echo "FAIL: mutating fake cargo passed the gate" >&2; exit 1; }
grep -Fq 'mutated tracked manifest inputs' "$tmp/mutating.err"

echo "manifest-lock gate tests: PASS"
