# Cycle artifact — Architect plan: TASK-0066 (nix-portable, ADDITIVE component only)

VERDICT: GO

## Scope (orchestrator-bounded)
ADDITIVE half only: meta-owned `nix-portable` Epic-H component (download DavHau/nix-portable static
binary → `.toolchains/nix-portable/bin/nix-portable` + `~/.local/bin/nix-portable` symlink). The
DESTRUCTIVE migration (remove host `/nix`, re-provision yazelix off Determinate nix) is DEFERRED to a
SUPERVISED follow-up card (TASK-0067) — it touches the owner's LIVE interactive shell (yazelix runs
from `/nix`). Mirrors the TASK-0054/0055 install-vs-risky-part split.

## Authoritative download
- Repo: `DavHau/nix-portable`; latest stable tag **`v012`**; asset **`nix-portable-x86_64`** (~68 MB).
- Pin via releases API (mirror gh/llvm discipline): `TAG=$(curl -fsSL api.github.com/repos/DavHau/nix-portable/releases/latest | grep -oE '"tag_name": *"[^"]+"' | head -1 | grep -oE 'v[0-9]+')`.
- URL: `https://github.com/DavHau/nix-portable/releases/download/${TAG}/nix-portable-x86_64`.
- Runtime needs `bwrap` (0.11.1 present, AppArmor-sanctioned); `NP_RUNTIME=bwrap` selector (raw userns
  blocked by `apparmor_restrict_unprivileged_userns=1`). Default store `~/.nix-portable`. NO env seam.

## Unit ledger (completeness contract)
- **U1** `manifest/components.d/epic-h-toolchains.toml` :: new `[[component]] id="nix-portable"` —
  mirror the **mise** block (single static binary). detect: `[ -x DEST/bin/nix-portable ]` + symlink
  resolves to DEST. install (login_shell): `M="${META_ROOT:-$HOME/Desktop/meta}"`; DEST=`$M/.toolchains/nix-portable`;
  pin TAG via API; `mktemp -d`+`trap`; `rm -rf "$DEST"`; `install -d -m 755 "$DEST/bin"`; curl asset →
  `$DEST/bin/nix-portable`; `chmod +x`; `ln -sfn ... ~/.local/bin/nix-portable`. verify (NO mutation/network —
  binary passes all args to nix, no native --help/--version; first real run bwrap-bootstraps a store):
  file-exists + executable + `file "$f" | grep -q 'ELF'` (libgccjit idiom). remove: symlink-ownership-guarded
  (`[ -L t ] && readlink t | grep -q "$M/.toolchains/nix-portable" && rm -f t`) + `rm -rf "$M/.toolchains/nix-portable"`
  (NEVER touches host /nix). wiring: `path_entries = ["~/.local/bin"]`. Comment: bwrap runtime + NP_RUNTIME=bwrap.
- **U2** `manifest/envctl.lock` :: new `[components.nix-portable]` (content_hash, requires=[], resolved="");
  count **73 → 74** (additive). Regen via `cargo run -p envctl -- lock`; `lock --check` exits 0.
- **U3** `docs/adr-install-locations-and-local-state.md` :: §System-depth convergence — mark nix-portable
  component SHIPPED (additive) + destructive migration DEFERRED to SUPERVISED TASK-0067.
- **U4** (orchestrator, NOT implementer) `.handoff/loop/backlog.md` — file `- [!!]` SUPERVISED TASK-0067
  for the deferred destructive migration (re-provision yazelix via nix-portable; migrate ~/.bashrc
  auto-enter + nix/home-manager/yazelix components off Determinate; retire `manifest/nix-yazelix.toml`
  id="nix" + `/nix/nix-installer uninstall` — ONLY in a human-supervised window).

## run_env / env.rs: NOT touched (justified)
nix-portable self-manages `~/.nix-portable` (default `$HOME`); no meta-owned lib-path redirect like
OLLAMA_LIBRARY_PATH/LIBCLANG_PATH/GCC_PATH. Implementer must NOT add a gratuitous NP_LOCATION/NP_RUNTIME export.

## Invariants (all PASS)
- no-C trust boundary: artifact is a `.toolchains/` runtime binary, NOT a Cargo dep → no-c.sh unaffected.
- one rustls/ring-only: no dep change.
- engine single non-printing lib: no engine code; declarative component only.
- destructive fail-closed/dry-run: component is purely additive; remove is self-guarded; NEVER touches /nix.
- rust-native/no drift: sanctioned meta-owned-binary pattern (gh/mise/ollama/llvm precedent).
- id collision: `manifest/nix-yazelix.toml` has id="nix" (Determinate installer, out of scope); new id
  `nix-portable` is distinct/additive (grep: no pre-existing nix-portable id).

## Runtime surface
- `envctl auto-detect` lists `nix-portable` (parses + rostered) — read-only, safe.
- `envctl lock --check` exits 0, "74 components".
- On-box after `install --apply`: `doctor` healthy+wired; `~/.local/bin/nix-portable` → DEST; ELF executable.
- Excluded from verify/guardian-required surface: functional `nix-portable nix --version` (bwrap-bootstraps
  a store/network on first run).
