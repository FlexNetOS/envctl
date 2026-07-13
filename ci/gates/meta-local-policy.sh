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
FRONTDOOR_TMP="$(mktemp)"
trap 'rm -f "$TMP" "$SOURCE_LIST" "$FRONTDOOR_TMP"' EXIT

git ls-files -z --cached --others --exclude-standard -- "${ACTIVE_PATHS[@]}" >"$SOURCE_LIST"


if grep -RIn -- '--archive-backup-dotfiles\|ARCHIVE_BACKUP_DOTFILES\|apply_backup_dotfile_archive\|is_backup_dotfile\|archive-backup' \
  scripts/audit-meta-local-paths.sh scripts/tests/test-meta-local-path-audit.sh >/dev/null; then
  echo "meta-local-policy: stale backup-only archive mode must not return; use --apply-history-archives" >&2
  exit 1
fi

# Reject real-home .local install directives and the retired symlink-farm design across active
# sources. Reference-only comments and the one negative-test diagnostic are filtered by line shape;
# the audit and its tests are no longer blanket path exemptions.
if [ -s "$SOURCE_LIST" ] && xargs -0 grep -HEnI "$PATTERN" <"$SOURCE_LIST" |
  grep -v '^[^:]*:[0-9]*:[[:space:]]*#' |
  grep -v '^ci/gates/meta-local-policy.sh:' |
  grep -v '^crates/engine/src/migration.rs:' |
  grep -v '^scripts/tests/test-meta-local-path-audit.sh:[0-9]*:[[:space:]]*echo "expected .*~/.local' |
  grep -v '^home/.codex/AGENTS.md:' |
  grep -v '^home/.codex/AGENTS.rtk.md:' |
  grep -v '^home/.codex/mined-live/rules/default.rules:' |
  grep -v '^home/agent-env/PORTABLE_CODEX_LOGS.md:' >"$TMP"; then
  echo "meta-local-policy: real-home .local install or retired symlink-farm references remain in active sources:" >&2
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
  if ! grep -Fq -- "$needle" "$path"; then
    echo "meta-local-policy: $message" >&2
    exit 1
  fi
}

check_present crates/engine/src/install.rs 'fn install_regular_frontdoor' \
  'installer must write regular executable frontdoors, not symlink/copy canonical usr/bin'
check_absent crates/engine/src/install.rs 'std::os::unix::fs::symlink' \
  'installer must not create canonical usr/bin symlink frontdoors'
check_present crates/engine/src/register.rs 'envctl_frontdoor()' \
  'add-repo drop-ins must expose regular frontdoor wrappers'
check_present crates/engine/src/model.rs 'MetaFrontdoorSymlink' \
  'meta boundary model must distinguish canonical usr/bin symlink regressions'
check_present crates/engine/src/detect.rs 'MetaFrontdoorSymlink' \
  'doctor must flag canonical usr/bin symlink regressions'

python3 - "$SOURCE_LIST" >"$FRONTDOOR_TMP" <<'PY'
from pathlib import Path
import re
import sys

ln_re = re.compile(r'\bln\s+-sfn?\s+(?P<src>"[^"]+"|[^ \t\n;]+)\s+(?P<dst>"[^"]+"|[^ \t\n;]+)')
install_res = [
    re.compile(r'\binstall\s+-Dm?755\s+(?P<src>"[^"]+"|[^ \t\n;]+)\s+(?P<dst>"[^"]+"|[^ \t\n;]+)'),
    re.compile(r'\binstall\s+(?:-[A-Za-z0-9]+\s+)*-m\s*755\s+(?P<src>"[^"]+"|[^ \t\n;]+)\s+(?P<dst>"[^"]+"|[^ \t\n;]+)'),
]

allowed_path_fragments = (
    '/.local/bin/',
    '/.config/',
    '/.toolchains/cargo/bin/',
    'portability-links.toml',
)

def q(value: str) -> str:
    return value.strip('"').strip("'")

def canonical_usr_bin(dst: str) -> bool:
    d = q(dst)
    return any(token in d for token in (
        '$META_ROOT/usr/bin/',
        '${META_ROOT}/usr/bin/',
        '$M/usr/bin/',
        '${M}/usr/bin/',
        '$BIN/',
    )) or d.endswith('/usr/bin')

for raw in Path(sys.argv[1]).read_bytes().split(b'\0'):
    if not raw:
        continue
    path = raw.decode()
    if path == 'ci/gates/meta-local-policy.sh':
        continue
    if not (path.startswith('manifest/') or path.startswith('assets/scripts/') or path.startswith('scripts/') or path.startswith('crates/engine/src/register.rs')):
        continue
    if any(fragment in path for fragment in allowed_path_fragments):
        continue
    try:
        text = Path(path).read_text(errors='ignore')
    except OSError:
        continue
    for idx, line in enumerate(text.splitlines(), 1):
        stripped = line.lstrip()
        if stripped.startswith('#'):
            continue
        m = ln_re.search(line)
        if m and canonical_usr_bin(m.group('dst')):
            print(f"{path}:{idx}: canonical usr/bin frontdoor symlink: {line}")
        if 'install -d' in line or 'install -dm' in line:
            continue
        for rx in install_res:
            m = rx.search(line)
            if m and canonical_usr_bin(m.group('dst')):
                print(f"{path}:{idx}: canonical usr/bin frontdoor direct copy: {line}")
PY

if [ -s "$FRONTDOOR_TMP" ]; then
  echo "meta-local-policy: canonical usr/bin frontdoors must be regular executable wrappers, not symlinks/direct copies:" >&2
  cat "$FRONTDOOR_TMP" >&2
  exit 1
fi

check_present manifest/grit.toml 'export CARGO_HOME="$META_ROOT/.toolchains/cargo"' \
  'grit must force cargo installs into the meta toolchains cargo home'
check_present manifest/prompt_hub.toml 'export CARGO_HOME="$META_ROOT/.toolchains/cargo"' \
  'prompt_hub must force cargo installs into the meta toolchains cargo home'
check_absent home/.gitconfig '\.local/bin/gh|/home/drdave/Desktop/meta/\.local/bin/gh' \
  "managed git credential helper must use the canonical META_ROOT usr/bin gh front door"
check_absent scripts/audit-meta-local-paths.sh \
  'only intentional real-home bridge|ln -sfn "\$META_ROOT/\.local" "\$local_link"|created \$local_link -> \$META_ROOT/\.local|relinked \$local_link -> \$META_ROOT/\.local|expected symlink to \$META_ROOT/\.local' \
  'real-home .local must never be created, replaced, or relinked to META_ROOT'

if ! grep -q 'Yazelix real-home Nix profile guard' manifest/components.d/portability-links.toml || \
   ! grep -q 'Never replace the whole real-home .local tree' manifest/components.d/portability-links.toml || \
   ! grep -q -- '--profile-shadow-guard-only --require-yazelix-profile' manifest/components.d/portability-links.toml || \
   ! grep -q 'validate_yazelix_profile_chain' scripts/audit-meta-local-paths.sh || \
   ! grep -q 'real-home .local must remain a real directory' scripts/audit-meta-local-paths.sh || \
   ! grep -q 'unknown real-home user-bin entry' scripts/audit-meta-local-paths.sh || \
   ! grep -q 'unknown META_ROOT usr/bin symlink' scripts/audit-meta-local-paths.sh || \
   ! grep -q 'cmp -s "\$path" "\$replacement"' scripts/audit-meta-local-paths.sh || \
   ! grep -q 'gitnexus)' scripts/audit-meta-local-paths.sh; then
  echo "meta-local-policy: missing exact Yazelix profile, real-directory, or fail-closed shadow contract" >&2
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

if ! grep -Fq '$ENVCTL_REAL_HOME/.nix-profile -> $ENVCTL_REAL_HOME/.local/state/nix/profiles/profile' docs/adr-install-locations-and-local-state.md || \
   ! grep -Fq '$ENVCTL_REAL_HOME/.nix-profile      -> real-home Nix profile state' home/README.md; then
  echo "meta-local-policy: Yazelix Nix profile preservation policy is not documented in the canonical ADR/home README" >&2
  exit 1
fi

for agent_doc in AGENTS.md CLAUDE.md; do
  if grep -Fq 'active install sources target $META_ROOT/.local only' "$agent_doc" || \
     grep -Fq 'installs two launchers on `$META_ROOT/.local/bin`' "$agent_doc"; then
    echo "meta-local-policy: $agent_doc has stale meta-local install-location documentation" >&2
    exit 1
  fi

  check_present "$agent_doc" 'active install sources target $META_ROOT FHS/XDG only; Yazelix real-home Nix profile preserved' \
    "$agent_doc must document the FHS/XDG meta-local policy in the gate list"
  check_present "$agent_doc" 'installs two launchers on `$META_ROOT/usr/bin`' \
    "$agent_doc dashboard docs must name the canonical usr/bin launcher location"
done
check_present docs/adr-install-locations-and-local-state.md '## Real-home dot-entry relocation map' \
  'install-location ADR must document the real-home dot-entry relocation map'
check_present docs/adr-install-locations-and-local-state.md '`$META_ROOT/.ideavimrc`' \
  'install-location ADR must document the .ideavimrc canonical target'
check_present docs/adr-install-locations-and-local-state.md '`$META_ROOT/.config/gphoto`' \
  'install-location ADR must document the .gphoto canonical target'
check_present docs/adr-install-locations-and-local-state.md '`$META_ROOT/.local/share/vscode-shared`' \
  'install-location ADR must document the .vscode-shared canonical target'
check_present docs/adr-install-locations-and-local-state.md '`$META_ROOT/.local/share/claude/claude.json`' \
  'install-location ADR must document the .claude.json canonical target'
check_present docs/adr-install-locations-and-local-state.md '`$META_ROOT/var/lib/ollama`' \
  'install-location ADR must document the .ollama canonical target'
check_present docs/adr-install-locations-and-local-state.md 'owner-supervised-vault-or-bridge' \
  'install-location ADR must document sensitive/broad config residual handling'
check_present docs/adr-meta-tool-location-and-portability.md 'Real-home dot-entry review loop' \
  'portability ADR must document the audit review loop'
check_present docs/adr-meta-tool-location-and-portability.md '--inventory-summary' \
  'portability ADR must document inventory summary output'
check_present docs/adr-meta-tool-location-and-portability.md '--deep-link-summary' \
  'portability ADR must document deep-link summary output'
check_present docs/adr-meta-tool-location-and-portability.md '--fail-real-home-deep-links' \
  'portability ADR must document fail-closed deep-link audits'
check_present docs/adr-meta-tool-location-and-portability.md '--apply-history-archives' \
  'portability ADR must document history/archive opt-in mutation'
check_present docs/adr-meta-tool-location-and-portability.md '--migrate-dot <entry>' \
  'portability ADR must document named dot-entry migrations'
check_present home/README.md 'agent-env.yaml` + `agent-env.lock`' \
  'home README must document agent-env as the current agent layer authority'
check_present home/README.md 'Review loop and known materialized host-local paths' \
  'home README must document the audit review loop and reviewed residuals'
check_present home/README.md '--deep-link-summary' \
  'home README must show deep-link audit output flags'

if ! grep -Fq 'find "$REAL_HOME" -mindepth 1 -maxdepth 1 -name' scripts/audit-meta-local-paths.sh || \
   ! grep -Fq 'dot_entries_seen' scripts/audit-meta-local-paths.sh || \
   ! grep -Fq -- '--inventory) INVENTORY_PATH=' scripts/audit-meta-local-paths.sh || \
   ! grep -Fq -- '--inventory-summary) INVENTORY_SUMMARY_PATH=' scripts/audit-meta-local-paths.sh || \
   ! grep -Fq -- '--deep-link-inventory) DEEP_LINK_INVENTORY_PATH=' scripts/audit-meta-local-paths.sh || \
   ! grep -Fq -- '--deep-link-summary) DEEP_LINK_SUMMARY_PATH=' scripts/audit-meta-local-paths.sh || \
   ! grep -Fq -- '--fail-real-home-deep-links)' scripts/audit-meta-local-paths.sh || \
   ! grep -Fq -- '--migrate-dot) MIGRATE_DOTS+=' scripts/audit-meta-local-paths.sh || \
   ! grep -Fq -- '--apply-history-archives) APPLY_HISTORY_ARCHIVES=1' scripts/audit-meta-local-paths.sh || \
   ! grep -Fq 'migrate_real_home_dot' scripts/audit-meta-local-paths.sh || \
   ! grep -Fq 'is_migratable_dot' scripts/audit-meta-local-paths.sh || \
   ! grep -Fq 'app_config_target_for_dot' scripts/audit-meta-local-paths.sh || \
   ! grep -Fq 'is_portable_app_config_file_dot' scripts/audit-meta-local-paths.sh || \
   ! grep -Fq 'is_portable_app_config_dir_dot' scripts/audit-meta-local-paths.sh || \
   ! grep -Fq '.ideavimrc)' scripts/audit-meta-local-paths.sh || \
   ! grep -Fq '.gphoto)' scripts/audit-meta-local-paths.sh || \
   ! grep -Fq '.vscode-shared)' scripts/audit-meta-local-paths.sh || \
   ! grep -Fq '.forge)' scripts/audit-meta-local-paths.sh || \
   ! grep -Fq 'scan_deep_links' scripts/audit-meta-local-paths.sh || \
   ! grep -Fq 'classify_deep_link' scripts/audit-meta-local-paths.sh || \
   ! grep -Fq 'emit_deep_link_summary' scripts/audit-meta-local-paths.sh || \
   ! grep -Fq 'real-home-leak' scripts/audit-meta-local-paths.sh || \
   ! grep -Fq 'emit_inventory_summary' scripts/audit-meta-local-paths.sh || \
   ! grep -Fq 'target_class' scripts/audit-meta-local-paths.sh || \
   ! grep -Fq 'owner-supervised-vault-or-bridge' scripts/audit-meta-local-paths.sh || \
   ! grep -Fq 'canonical_target\tsensitive_hints\tblocker' scripts/audit-meta-local-paths.sh || \
   ! grep -Fq '$home/.zshrc' scripts/tests/test-meta-local-path-audit.sh || \
   ! grep -Fq '$home/.aws' scripts/tests/test-meta-local-path-audit.sh || \
   ! grep -Fq '$home/.cache' scripts/tests/test-meta-local-path-audit.sh || \
   ! grep -Fq '$home/.cargo' scripts/tests/test-meta-local-path-audit.sh || \
   ! grep -Fq -- '--migrate-dot .cargo' scripts/tests/test-meta-local-path-audit.sh || \
   ! grep -Fq -- '--migrate-dot .dotnet' scripts/tests/test-meta-local-path-audit.sh || \
   ! grep -Fq -- '--migrate-dot .gemini' scripts/tests/test-meta-local-path-audit.sh || \
   ! grep -Fq -- '--migrate-dot .ideavimrc' scripts/tests/test-meta-local-path-audit.sh || \
   ! grep -Fq -- '--migrate-dot .gphoto' scripts/tests/test-meta-local-path-audit.sh || \
   ! grep -Fq -- '--migrate-dot .vscode-shared' scripts/tests/test-meta-local-path-audit.sh || \
   ! grep -Fq -- '--migrate-dot .n8n-claude-bridge' scripts/tests/test-meta-local-path-audit.sh || \
   ! grep -Fq -- '--migrate-dot .pki' scripts/tests/test-meta-local-path-audit.sh || \
   ! grep -Fq -- '--migrate-dot .forge' scripts/tests/test-meta-local-path-audit.sh || \
   ! grep -Fq -- '--migrate-dot .ssh' scripts/tests/test-meta-local-path-audit.sh || \
   ! grep -Fq 'backup-pre-summary.tsv' scripts/tests/test-meta-local-path-audit.sh || \
   ! grep -Fq '.ollama' scripts/tests/test-meta-local-path-audit.sh || \
   ! grep -Fq '.kimi-code' scripts/tests/test-meta-local-path-audit.sh || \
   ! grep -Fq '.ideavimrc' scripts/tests/test-meta-local-path-audit.sh || \
   ! grep -Fq '.config/gphoto' scripts/tests/test-meta-local-path-audit.sh || \
   ! grep -Fq '.local/share/vscode-shared' scripts/tests/test-meta-local-path-audit.sh || \
   ! grep -Fq '.local/share/n8n-claude-bridge' scripts/tests/test-meta-local-path-audit.sh || \
   ! grep -Fq '.local/share/pki' scripts/tests/test-meta-local-path-audit.sh || \
   ! grep -Fq '.local/share/forge' scripts/tests/test-meta-local-path-audit.sh || \
   ! grep -Fq -- '--deep-link-inventory' scripts/tests/test-meta-local-path-audit.sh || \
   ! grep -Fq -- '--deep-link-summary' scripts/tests/test-meta-local-path-audit.sh || \
   ! grep -Fq -- '--fail-real-home-deep-links' scripts/tests/test-meta-local-path-audit.sh || \
   ! grep -Fq 'missing-target' scripts/tests/test-meta-local-path-audit.sh || \
   ! grep -Fq 'external-system' scripts/tests/test-meta-local-path-audit.sh || \
   ! grep -Fq 'real-home-leak' scripts/tests/test-meta-local-path-audit.sh || \
   ! grep -Fq 'real-home-dotfile-migration' scripts/tests/test-meta-local-path-audit.sh || \
   ! grep -Fq -- '--apply-history-archives' scripts/tests/test-meta-local-path-audit.sh || \
   ! grep -Fq 'inventory-summary.tsv' scripts/tests/test-meta-local-path-audit.sh || \
   ! grep -Fq 'apply_safe_yes' scripts/tests/test-meta-local-path-audit.sh || \
   ! grep -Fq 'sensitive_hints' scripts/tests/test-meta-local-path-audit.sh; then
  echo "meta-local-policy: meta-local path audit must walk, inventory, classify, and test every top-level real-home dot entry class" >&2
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

bash scripts/tests/test-meta-local-path-audit.sh
bash scripts/tests/test-envctl-cli-component.sh
bash scripts/tests/test-cargo-audit-component.sh
bash scripts/tests/test-toolchain-contract-gate.sh
bash scripts/tests/test-postgres-ruvector-component.sh
bash scripts/tests/test-sqld-component.sh
bash scripts/tests/test-manifest-lock-gate.sh
bash scripts/tests/test-source-selector-contract.sh
bash scripts/tests/test-codedb-upload-list-export.sh
bash scripts/tests/test-gh-fetch-contract.sh
bash scripts/tests/test-odysseus-install-idempotence.sh

echo "meta-local-policy: active install sources target META_ROOT FHS/XDG; Yazelix real-home Nix profile state is preserved"
