#!/usr/bin/env bash
set -euo pipefail

ROOT="$(git rev-parse --show-toplevel)"
cd "$ROOT"

PATTERN='(%h|~|\$HOME|\$\{HOME\})/\.local|symlink farm'

# Active install/provisioning sources must target META_ROOT.  The migration engine is excluded
# because it intentionally contains the forbidden spellings as detector inputs and tests.
ACTIVE_PATHS=(
  AGENTS.md
  CLAUDE.md
  manifest
  home
  agent-env.yaml
  agent-env.lock
  agent-skills/mcps
  .mcp.json
  .codex/config.toml
  crates/engine/src
  crates/secretctl/src
  crates/secrets-engine/src
  crates/agent-env/src
  crates/agent-env/tests/parity_vs_kasetto.rs
  ci
  scripts
  packaging
  assets/scripts
  .github
  docs/ADD-REPO.md
  docs/MIGRATION-ADOPTION.md
  docs/ops/02-envctl-component.md
  docs/adr-seed-usb-possession-factor.md
  docs/HANDOFF-kasetto-env-and-phase8.md
  docs/adr-install-locations-and-local-state.md
  docs/adr-meta-tool-location-and-portability.md
  docs/DESIGN-NOTES.md
  docs/ROADMAP.md
)

TMP="$(mktemp)"
SOURCE_LIST="$(mktemp)"
trap 'rm -f "$TMP" "$SOURCE_LIST"' EXIT

git ls-files -z --cached --others --exclude-standard -- "${ACTIVE_PATHS[@]}" >"$SOURCE_LIST"

if [ -s "$SOURCE_LIST" ] && xargs -0 grep -HEnI "$PATTERN" <"$SOURCE_LIST" |
  grep -v '^ci/gates/meta-local-policy.sh:' |
  grep -v '^crates/engine/src/migration.rs:' |
  grep -v '^scripts/audit-meta-local-paths.sh:' |
  grep -v '^scripts/tests/test-meta-local-path-audit.sh:' >"$TMP"; then
  echo "meta-local-policy: real-home .local/symlink-farm references remain in active install sources:" >&2
  cat "$TMP" >&2
  exit 1
fi


# High-confidence active-source regressions caught by the live audit work: front-door binaries must
# not be installed into $META_ROOT/.local/bin, cargo-installed tools must use the explicit meta
# .toolchains/cargo home, and managed git credential helpers must not route through legacy .local/bin.
check_absent() {
  local path="$1" pattern="$2" message="$3"
  if grep -HEnI "$pattern" "$path" >"$TMP" 2>/dev/null; then
    echo "meta-local-policy: $message" >&2
    cat "$TMP" >&2
    exit 1
  fi
}

check_absent manifest/components.d/meta-env-plugin.toml '\$META_ROOT/\.local/bin/meta-env|\.toolchains/meta-env' \
  "meta-env plugin must install private payloads under usr/libexec and expose only a usr/bin front door"
check_absent manifest/grit.toml '\$META_ROOT/\.cargo/bin' \
  'grit must not wire the legacy META_ROOT .cargo bin path'
check_absent manifest/prompt_hub.toml '\$META_ROOT/\.cargo/bin' \
  'prompt_hub must not wire the legacy META_ROOT .cargo bin path'

check_present() {
  local path="$1" needle="$2" message="$3"
  if ! grep -Fq "$needle" "$path"; then
    echo "meta-local-policy: $message" >&2
    exit 1
  fi
}

check_present manifest/grit.toml 'export CARGO_HOME="$META_ROOT/.toolchains/cargo"' \
  'grit must force cargo installs into the meta toolchains cargo home'
check_present manifest/prompt_hub.toml 'export CARGO_HOME="$META_ROOT/.toolchains/cargo"' \
  'prompt_hub must force cargo installs into the meta toolchains cargo home'
check_absent home/.gitconfig '\.local/bin/gh|/home/drdave/Desktop/meta/\.local/bin/gh' \
  "managed git credential helper must use the canonical META_ROOT usr/bin gh front door"

if ! grep -q 'home-local-single-link' manifest/components.d/portability-links.toml; then
  echo "meta-local-policy: missing single real-home .local bridge component" >&2
  exit 1
fi

if ! grep -Fq 'pub fn usr_bin(&self)' crates/engine/src/layout.rs || \
   ! grep -Fq 'self.usr_bin()' crates/engine/src/layout.rs || \
   ! grep -Fq 'pub fn var_lib_envctl(&self)' crates/engine/src/layout.rs || \
   ! grep -Fq 'pub fn xdg_config_home(&self)' crates/engine/src/layout.rs || \
   ! grep -Fq 'LegacyCompatibility' crates/engine/src/layout.rs; then
  echo "meta-local-policy: layout must expose canonical META_ROOT FHS/XDG paths and mark legacy compatibility prefixes" >&2
  exit 1
fi

if ! grep -Fq "ENVCTL_BIN_DIR" crates/cli/tests/env.rs || \
   ! grep -Fq "/usr/bin" crates/cli/tests/env.rs || \
   ! grep -Fq "ENVCTL_LOCAL_BIN" crates/cli/tests/env.rs; then
  echo "meta-local-policy: env output tests must prove usr/bin primary path plus .local/bin compatibility export" >&2
  exit 1
fi

if ! grep -Fq 'layout.local_bin()' crates/engine/src/runner.rs || \
   ! grep -Fq 'XDG_CONFIG_HOME' crates/engine/src/runner.rs || \
   ! grep -Fq 'layout.bin()' crates/engine/src/runner.rs; then
  echo "meta-local-policy: hook runner must force META_ROOT FHS/XDG env and PATH with usr/bin first" >&2
  exit 1
fi

if ! grep -Eq '\$ENVCTL_REAL_HOME/\.local -> \$META_ROOT/\.local' docs/adr-install-locations-and-local-state.md home/README.md; then
  echo "meta-local-policy: bridge policy is not documented in the canonical ADR/home README" >&2
  exit 1
fi

if ! grep -Fq 'trimmed.contains("~/.local")' crates/engine/src/migration.rs || \
   ! grep -Fq 'trimmed.contains("$HOME/.local")' crates/engine/src/migration.rs || \
   ! grep -Fq 'trimmed.contains("${HOME}/.local")' crates/engine/src/migration.rs || \
   ! grep -Fq 'trimmed.contains("%h/.local")' crates/engine/src/migration.rs; then
  echo "meta-local-policy: migration detector must flag legacy real-home .local spellings" >&2
  exit 1
fi

for path_file in crates/secrets-engine/src/paths.rs crates/secretctl/src/main.rs crates/secrets-engine/src/seam.rs; do
  if ! grep -q 'META_ROOT' "$path_file"; then
    echo "meta-local-policy: $path_file must prefer META_ROOT when explicit XDG roots are unset" >&2
    exit 1
  fi
done

echo "meta-local-policy: active install sources target META_ROOT FHS/XDG; only the single real-home .local bridge is allowed"
