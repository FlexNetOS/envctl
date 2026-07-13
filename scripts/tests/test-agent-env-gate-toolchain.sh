#!/usr/bin/env bash
set -euo pipefail

fail() { echo "FAIL: $*" >&2; exit 1; }

root="$(git -C "$(dirname "${BASH_SOURCE[0]}")" rev-parse --show-toplevel)"
gate="$root/ci/gates/agent-env.sh"
tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT

meta="$tmp/meta"
canonical_repo="$meta/src/envctl"
managed_worktree="$tmp/managed/envctl"
ambient="$tmp/ambient"
toolchain_log="$tmp/toolchain.log"
ambient_called="$tmp/ambient-called"
mkdir -p "$canonical_repo" "$ambient" "$meta/.toolchains/cargo/bin" \
  "$meta/.toolchains/rustup"
printf 'projects: {}\n' >"$meta/.meta.yaml"

git -C "$canonical_repo" init -q
git -C "$canonical_repo" config user.name fixture
git -C "$canonical_repo" config user.email fixture@example.invalid
git -C "$canonical_repo" commit -q --allow-empty -m initial
mkdir -p "$(dirname "$managed_worktree")"
git -C "$canonical_repo" worktree add -q -b fixture-managed "$managed_worktree"

for tool in cargo rustc rustup; do
  cat >"$meta/.toolchains/cargo/bin/$tool" <<'SH'
#!/usr/bin/env bash
set -euo pipefail
printf '%s\n' "$(basename "$0")" >>"${TOOLCHAIN_LOG:?}"
printf '%s fixture\n' "$(basename "$0")"
SH
  chmod 755 "$meta/.toolchains/cargo/bin/$tool"

  cat >"$ambient/$tool" <<'SH'
#!/usr/bin/env bash
set -euo pipefail
: >"${AMBIENT_CALLED:?}"
echo "ambient toolchain must never be invoked" >&2
exit 99
SH
  chmod 755 "$ambient/$tool"
done

env -u META_ROOT \
  ENVCTL_GATE_ROOT="$managed_worktree" \
  ENVCTL_AGENT_ENV_GATE_TOOLCHAIN_PROBE_ONLY=1 \
  TOOLCHAIN_LOG="$toolchain_log" \
  AMBIENT_CALLED="$ambient_called" \
  PATH="$ambient:/usr/bin:/bin" \
  bash "$gate" >"$tmp/out" 2>"$tmp/err"

[ ! -e "$ambient_called" ] || fail "managed-worktree gate invoked ambient Cargo/Rust"
grep -Fqx 'cargo' "$toolchain_log" || fail "canonical cargo was not probed"
grep -Fqx 'rustc' "$toolchain_log" || fail "canonical rustc was not probed"
grep -Fq "META_ROOT=$meta" "$tmp/out" || fail "gate did not derive canonical meta root"
grep -Fq "CARGO_HOME=$meta/.toolchains/cargo" "$tmp/out" \
  || fail "gate did not select canonical meta Cargo home"
grep -Fq "RUSTUP_HOME=$meta/.toolchains/rustup" "$tmp/out" \
  || fail "gate did not select canonical meta Rustup home"

# Hosted fork/CI_FORCE_HOSTED jobs are standalone clones without a meta owner. Their fallback is
# explicit to GitHub Actions and must invoke rustup-selected payloads, never an earlier PATH cargo.
hosted_repo="$tmp/hosted/envctl"
hosted_home="$tmp/hosted-home"
hosted_cargo_home="$hosted_home/.cargo"
hosted_rustup_home="$hosted_home/.rustup"
hosted_toolchain="$hosted_rustup_home/toolchains/stable-fixture"
hosted_ambient="$tmp/hosted-ambient"
hosted_log="$tmp/hosted-toolchain.log"
hosted_ambient_called="$tmp/hosted-ambient-called"
mkdir -p "$hosted_repo" "$hosted_cargo_home/bin" "$hosted_toolchain/bin" "$hosted_ambient"
git -C "$hosted_repo" init -q

cat >"$hosted_cargo_home/bin/rustup" <<'SH'
#!/usr/bin/env bash
set -euo pipefail
case "${1:-} ${2:-}" in
  'which cargo') printf '%s/toolchains/stable-fixture/bin/cargo\n' "${RUSTUP_HOME:?}" ;;
  'which rustc') printf '%s/toolchains/stable-fixture/bin/rustc\n' "${RUSTUP_HOME:?}" ;;
  *) echo "unexpected rustup invocation: $*" >&2; exit 64 ;;
esac
SH
chmod 755 "$hosted_cargo_home/bin/rustup"

for tool in cargo rustc; do
  cat >"$hosted_toolchain/bin/$tool" <<'SH'
#!/usr/bin/env bash
set -euo pipefail
printf '%s\n' "$(basename "$0")" >>"${HOSTED_TOOLCHAIN_LOG:?}"
printf '%s hosted-fixture\n' "$(basename "$0")"
SH
  chmod 755 "$hosted_toolchain/bin/$tool"

  cat >"$hosted_ambient/$tool" <<'SH'
#!/usr/bin/env bash
set -euo pipefail
: >"${HOSTED_AMBIENT_CALLED:?}"
echo "hosted ambient toolchain must never be invoked" >&2
exit 99
SH
  chmod 755 "$hosted_ambient/$tool"
done

env -u META_ROOT \
  ENVCTL_GATE_ROOT="$hosted_repo" \
  ENVCTL_AGENT_ENV_GATE_TOOLCHAIN_PROBE_ONLY=1 \
  GITHUB_ACTIONS=true \
  HOME="$hosted_home" \
  CARGO_HOME="$hosted_cargo_home" \
  RUSTUP_HOME="$hosted_rustup_home" \
  HOSTED_TOOLCHAIN_LOG="$hosted_log" \
  HOSTED_AMBIENT_CALLED="$hosted_ambient_called" \
  PATH="$hosted_ambient:$hosted_cargo_home/bin:/usr/bin:/bin" \
  bash "$gate" >"$tmp/hosted-out" 2>"$tmp/hosted-err"

[ ! -e "$hosted_ambient_called" ] || fail "hosted fallback invoked ambient Cargo/Rust"
grep -Fqx 'cargo' "$hosted_log" || fail "hosted rustup-selected cargo was not probed"
grep -Fqx 'rustc' "$hosted_log" || fail "hosted rustup-selected rustc was not probed"
grep -Fq 'TOOLCHAIN_MODE=github-rustup' "$tmp/hosted-out" \
  || fail "gate did not report the explicit GitHub rustup fallback"
grep -Fq "CARGO_BIN=$hosted_toolchain/bin/cargo" "$tmp/hosted-out" \
  || fail "gate did not select rustup's hosted cargo payload"

if env -u META_ROOT -u GITHUB_ACTIONS \
  ENVCTL_GATE_ROOT="$hosted_repo" \
  ENVCTL_AGENT_ENV_GATE_TOOLCHAIN_PROBE_ONLY=1 \
  HOME="$hosted_home" \
  CARGO_HOME="$hosted_cargo_home" \
  RUSTUP_HOME="$hosted_rustup_home" \
  HOSTED_AMBIENT_CALLED="$hosted_ambient_called" \
  PATH="$hosted_ambient:$hosted_cargo_home/bin:/usr/bin:/bin" \
  bash "$gate" >"$tmp/standalone-out" 2>"$tmp/standalone-err"; then
  fail "standalone no-meta checkout accepted ambient Rust outside GitHub Actions"
fi
grep -Fq 'ambient Rust is forbidden outside GitHub Actions' "$tmp/standalone-err" \
  || fail "standalone no-meta refusal was unclear"
[ ! -e "$hosted_ambient_called" ] || fail "standalone refusal invoked ambient Cargo/Rust"

echo "PASS: agent-env gate owns meta worktree Rust and validates GitHub rustup fallback"
