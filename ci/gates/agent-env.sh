#!/usr/bin/env bash
# agent-env.sh - agent-env (absorbed kasetto v3.2.0) drift gate. (TASK-0040)
#
# Closes a claimed-but-unwired enforcement: CLAUDE.md long stated "CI enforces with
# `envctl agent ... --locked`", but no gate existed because the config files were never
# migrated off the retired kasetto binary's names (kasetto.yaml/.lock), so the absorbed CLI
# could not find `agent-env.yaml` and any gate would have failed "config not found". With the
# config migrated (TASK-0040), this gate makes the enforcement real.
#
# Fail-closed: the committed agent-env.yaml must match agent-env.lock. The gate runs
# `envctl agent lock --config agent-env.yaml --check --locked`: read-only, zero-network
# (no fetch), and exits 1 on config<->lock drift. Sibling to ci/gates/{no-c,shape,enable,p7}.sh.
set -euo pipefail
ROOT="${ENVCTL_GATE_ROOT:-$(/usr/bin/git rev-parse --show-toplevel)}"
cd "$ROOT"

fail() {
  echo "AGENT-ENV GATE FAIL - $*" >&2
  exit 1
}

meta_candidate="${META_ROOT:-}"
if [ -z "$meta_candidate" ]; then
  common_dir="$(/usr/bin/git -C "$ROOT" rev-parse --path-format=absolute --git-common-dir)" \
    || fail "cannot resolve the checkout's common git directory"
  cursor="$(dirname "$common_dir")"
  while [ "$cursor" != "/" ]; do
    if [ -f "$cursor/.meta.yaml" ]; then
      meta_candidate="$cursor"
      break
    fi
    parent="$(dirname "$cursor")"
    [ "$parent" != "$cursor" ] || break
    cursor="$parent"
  done
fi

# A self-hosted Actions checkout may physically live below the Meta workspace while still being a
# standalone CI clone. Its explicit profile toolbin owner takes precedence over ancestor discovery.
if [ "${GITHUB_ACTIONS:-}" = "true" ] && [ -n "${ENVCTL_GITHUB_PROFILE_TOOLBIN:-}" ]; then
  meta_candidate=""
fi

if [ -n "$meta_candidate" ]; then
  TOOLCHAIN_MODE="meta"
  [ -d "$meta_candidate" ] || fail "resolved META_ROOT is not a directory: $meta_candidate"
  [ -f "$meta_candidate/.meta.yaml" ] \
    || fail "resolved META_ROOT lacks its .meta.yaml ownership marker: $meta_candidate"
  META_ROOT="$(readlink -f -- "$meta_candidate")"
  [ -n "$META_ROOT" ] || fail "resolved META_ROOT cannot be canonicalized"
  export META_ROOT
  export CARGO_HOME="$META_ROOT/.toolchains/cargo"
  export RUSTUP_HOME="$META_ROOT/.toolchains/rustup"
  export PATH="$CARGO_HOME/bin:$META_ROOT/usr/bin:/usr/bin:/bin"

  for tool in cargo rustc rustup; do
    path="$CARGO_HOME/bin/$tool"
    [ -x "$path" ] || fail "canonical meta toolchain is missing executable $path"
    resolved="$(readlink -f -- "$path")"
    case "$resolved" in
      "$META_ROOT"/*) ;;
      *) fail "canonical meta tool $path escapes META_ROOT (resolved $resolved)" ;;
    esac
  done
  CARGO_BIN="$CARGO_HOME/bin/cargo"
  RUSTC_BIN="$CARGO_HOME/bin/rustc"
  [ "$(command -v cargo)" = "$CARGO_BIN" ] \
    || fail "Cargo did not resolve from canonical CARGO_HOME"
elif [ "${GITHUB_ACTIONS:-}" = "true" ] && [ -n "${ENVCTL_GITHUB_PROFILE_TOOLBIN:-}" ]; then
  # FlexNetOS self-hosted runners consume the immutable, profile-owned Rust payload directly.
  # They intentionally do not carry a second rustup home under the isolated runner HOME.
  TOOLCHAIN_MODE="github-profile"
  : "${ENVCTL_GITHUB_PROFILE_HOME:?self-hosted profile home is required}"
  profile_home="$(readlink -f -- "$ENVCTL_GITHUB_PROFILE_HOME")"
  profile_toolbin="$profile_home/.nix-profile/toolbin"
  [ "$ENVCTL_GITHUB_PROFILE_TOOLBIN" = "$profile_toolbin" ] \
    || fail "self-hosted profile toolbin must be exactly $profile_toolbin"
  store_root="${ENVCTL_NIX_STORE_ROOT:-/nix/store}"
  toolbin_root="$(readlink -f -- "$profile_toolbin")"
  case "$toolbin_root" in
    "$store_root"/*/toolbin) ;;
    *) fail "self-hosted profile toolbin is not Nix-store-owned: $toolbin_root" ;;
  esac
  CARGO_BIN="$profile_toolbin/cargo"
  RUSTC_BIN="$profile_toolbin/rustc"
  for tool in "$CARGO_BIN" "$RUSTC_BIN"; do
    [ -x "$tool" ] || fail "self-hosted profile tool is missing or non-executable: $tool"
    resolved="$(readlink -f -- "$tool")"
    case "$resolved" in
      "$store_root"/*) ;;
      *) fail "self-hosted profile tool escapes the Nix store: $tool -> $resolved" ;;
    esac
  done
  export HOME="$profile_home"
  export CARGO_HOME="$profile_home/.cargo"
  export RUSTUP_HOME="$profile_home/.rustup"
  export PATH="$profile_toolbin:/usr/bin:/bin"
  unset META_ROOT
elif [ "${GITHUB_ACTIONS:-}" = "true" ]; then
  # Hosted fork / CI_FORCE_HOSTED jobs have no meta owner. Their runner is ephemeral, but PATH may
  # still contain unrelated package-manager cargo shims. Accept only the payloads selected by the
  # runner's explicit rustup home; never invoke `cargo` or `rustc` by ambient name.
  TOOLCHAIN_MODE="github-rustup"
  : "${HOME:?HOME is required for the GitHub rustup fallback}"
  export CARGO_HOME="${CARGO_HOME:-$HOME/.cargo}"
  export RUSTUP_HOME="${RUSTUP_HOME:-$HOME/.rustup}"
  rustup_bin="$(PATH="$CARGO_HOME/bin:/usr/bin:/bin" command -v rustup || true)"
  [ "$rustup_bin" = "$CARGO_HOME/bin/rustup" ] \
    || fail "GitHub fallback requires the runner rustup at $CARGO_HOME/bin/rustup"
  [ -x "$rustup_bin" ] || fail "GitHub runner rustup is not executable"
  CARGO_BIN="$(readlink -f -- "$("$rustup_bin" which cargo)")"
  RUSTC_BIN="$(readlink -f -- "$("$rustup_bin" which rustc)")"
  [ -x "$CARGO_BIN" ] || fail "rustup-selected GitHub cargo is not executable: $CARGO_BIN"
  [ -x "$RUSTC_BIN" ] || fail "rustup-selected GitHub rustc is not executable: $RUSTC_BIN"
  rustup_root="$(readlink -f -- "$RUSTUP_HOME")"
  case "$CARGO_BIN" in
    "$rustup_root"/toolchains/*/bin/cargo) ;;
    *) fail "rustup-selected GitHub cargo escapes RUSTUP_HOME: $CARGO_BIN" ;;
  esac
  case "$RUSTC_BIN" in
    "$rustup_root"/toolchains/*/bin/rustc) ;;
    *) fail "rustup-selected GitHub rustc escapes RUSTUP_HOME: $RUSTC_BIN" ;;
  esac
  [ "$(dirname "$(dirname "$CARGO_BIN")")" = "$(dirname "$(dirname "$RUSTC_BIN")")" ] \
    || fail "GitHub cargo and rustc resolve from different rustup toolchains"
  github_toolchain_bin="$(dirname "$CARGO_BIN")"
  export PATH="$github_toolchain_bin:$CARGO_HOME/bin:/usr/bin:/bin"
  unset META_ROOT
else
  fail "META_ROOT is unset and no .meta.yaml owns the checkout; ambient Rust is forbidden outside GitHub Actions"
fi

if [ "${ENVCTL_AGENT_ENV_GATE_TOOLCHAIN_PROBE_ONLY:-0}" = 1 ]; then
  "$CARGO_BIN" --version >/dev/null
  "$RUSTC_BIN" --version >/dev/null
  printf 'TOOLCHAIN_MODE=%s\nMETA_ROOT=%s\nCARGO_HOME=%s\nRUSTUP_HOME=%s\nCARGO_BIN=%s\n' \
    "$TOOLCHAIN_MODE" "${META_ROOT:-}" "$CARGO_HOME" "$RUSTUP_HOME" "$CARGO_BIN"
  exit 0
fi

echo "AGENT-ENV GATE: verify component uses the owned envctl generation"
bash scripts/tests/test-agent-env-component.sh
echo "AGENT-ENV GATE: verify managed-worktree toolchain ownership"
bash scripts/tests/test-agent-env-gate-toolchain.sh

echo "AGENT-ENV GATE: build envctl"
"$CARGO_BIN" build -q -p envctl
BIN="${CARGO_TARGET_DIR:-target}/debug/envctl"

render_tmp="$(mktemp -d)"
trap 'rm -rf "$render_tmp"' EXIT
echo "AGENT-ENV GATE: render catalog"
"$BIN" catalog render --out "$render_tmp/catalog" --target-root "$(pwd)" >/dev/null
echo "AGENT-ENV GATE: verify Yazelix MCP mirror"
ENVCTL_RENDERED_CODEX_CONFIG="$render_tmp/catalog/.codex/config.toml" bash scripts/tests/test-agent-mcp-yazelix-mirror.sh

echo "AGENT-ENV GATE: verify agent-env lock"
if "$BIN" agent lock --config agent-env.yaml --check --locked; then
  :
else
  echo "AGENT-ENV GATE FAIL - agent-env.yaml drifted from agent-env.lock" >&2
  echo "  fix: 'envctl agent lock --config agent-env.yaml' to rewrite the lock, then commit agent-env.lock" >&2
  exit 1
fi

echo "AGENT-ENV GATE: verify active Codex/Claude skill mirrors"
bash scripts/tests/test-agent-env-active-mirrors.sh

echo "AGENT-ENV GATE: verify locked drift counterexample"
counterexample="$render_tmp/locked-drift-counterexample"
mkdir -p "$counterexample/source/alpha" "$counterexample/destination"
printf '%s\n' '---' 'name: alpha' '---' 'original' > "$counterexample/source/alpha/SKILL.md"
printf 'destination: %s\nscope: project\nskills:\n  - source: %s\n    skills:\n      - alpha\n' \
  "$counterexample/destination" "$counterexample/source" > "$counterexample/agent-env.yaml"
"$BIN" agent lock --config "$counterexample/agent-env.yaml" --color never >/dev/null
printf '%s\n' '---' 'name: alpha' '---' 'drifted' > "$counterexample/source/alpha/SKILL.md"
if "$BIN" agent lock --config "$counterexample/agent-env.yaml" --check --locked --color never >/dev/null 2>&1; then
  echo "AGENT-ENV GATE FAIL - locked drift counterexample false-passed" >&2
  exit 1
fi

echo "AGENT-ENV GATE PASS"
