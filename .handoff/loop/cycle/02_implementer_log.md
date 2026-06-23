# Implementation log: TASK-0066 — meta-owned nix-portable (Epic H, ADDITIVE component only)

## Changes
- manifest/components.d/epic-h-toolchains.toml: appended a new `[[component]] id="nix-portable"`
  block AFTER the `libgccjit` block. Mirrors the **mise** single-static-binary idiom (curl direct to
  `$DEST/bin/nix-portable` → `~/.local/bin/nix-portable` symlink) with an `rm -rf "$DEST"` idempotence
  guard (ollama/llvm idiom). Tag pinned via the DavHau/nix-portable releases API
  (`grep -oE 'v[0-9]+'`), asset `nix-portable-x86_64`. detect = file-exists + executable + symlink
  resolves to DEST (mise readlink idiom). verify = NO mutation/network — file + executable +
  `file ... | grep -q 'ELF'` (libgccjit verify idiom). remove = symlink-ownership-guarded
  (`[ -L t ] && readlink t | grep -q "$M/.toolchains/nix-portable"`) + `rm -rf` DEST — NEVER touches
  host `/nix`. `[component.wiring] path_entries = ["~/.local/bin"]`. Comment documents bwrap runtime
  / `NP_RUNTIME=bwrap` / default store `~/.nix-portable` / additive-only + TASK-0067 deferral.
- manifest/envctl.lock: regenerated via `cargo run -p envctl -- lock` (NOT hand-edited). Net-new
  `[components.nix-portable]` (content_hash=`88e98a62e8fc4be2`, requires=[], resolved=""). +5 lines,
  one entry. Component count **73 → 74**.
- docs/adr-install-locations-and-local-state.md: (1) added a `nix` row to the §System-depth
  convergence-plan table marked **COMPONENT SHIPPED — ADDITIVE** (TASK-0066) with the destructive
  migration **DEFERRED to supervised TASK-0067**; (2) added a "Component SHIPPED (TASK-0066) —
  ADDITIVE only" bullet under the existing `nix-portable` prose, restating that it never touches host
  `/nix` and the destructive de-nix is supervised TASK-0067.

## Engine API
No Engine API change. No engine/CLI/GUI Rust touched, no `run_env`/`env.rs` change, no Cargo dep.
Purely a declarative manifest component the engine already knows how to run + the regenerated lock +
a doc. No front-end parity surface (nix-portable self-manages `~/.nix-portable`; no meta-owned env
seam like OLLAMA_LIBRARY_PATH/GCC_PATH was added, per the architect plan).

## Tests added
None — no Rust changed, so there is no new code path to unit-test. The component's behavior is
exercised by the existing manifest-driven detect/verify lifecycle (proven via `auto-detect`, below)
and the lock round-trip gate (`lock --check`).

## Build/test status (real output)
- `cargo build -p envctl-engine -p envctl` — PASS (Finished dev profile, 10.95s; no Rust changed).
- `cargo run -p envctl -- auto-detect | grep nix-portable` — PARSES + rostered:
  `[med] nix-portable  Missing: declared but not installed → envctl install nix-portable`
  (and `· nix-portable  nix-portable (meta-owned) wired` in the components roster). No parse error.
- `cargo run -p envctl -- lock` — `wrote manifest/envctl.lock (74 components)`.
- `cargo run -p envctl -- lock --check` — `✓ envctl.lock matches the manifest (74 components)` exit 0.
- `cargo fmt --all -- --check` — PASS (exit 0).
- `cargo clippy -p envctl-engine -p envctl -- -D warnings` — PASS (Finished, no issues).
- `bash ci/gates/no-c.sh` — PASS (`NO-C GATE PASS`; rustls 0.23.40 on ring 0.17.14, zero C-SQLite).
- `bash ci/gates/shape.sh` — PASS (`SHAPE GATE PASS`).
- `bash ci/gates/loop-state.sh` — PASS (`LOOP-STATE GATE PASS`).

## Network install — NOT exercised end-to-end (sandbox limitation; documented)
- The GitHub **releases API** (`api.github.com/repos/DavHau/nix-portable/releases/latest`, used by
  the install hook to PIN the tag) returns **HTTP 403** from this sandbox (unauthenticated
  rate-limit), so the full install hook (tag resolve → curl → chmod → symlink → verify) could NOT be
  run here.
- The **asset URL itself** IS reachable: `curl -fsSLI` on
  `https://github.com/DavHau/nix-portable/releases/download/v012/nix-portable-x86_64` → **HTTP 200**.
  This confirms the architect's pinned tag (`v012`) and asset name (`nix-portable-x86_64`) are valid.
- ACTION FOR GUARDIAN/ORCHESTRATOR: run the on-box `envctl install --apply` scoped to `nix-portable`
  (or the install hook body) where the GitHub API is authenticated/reachable. Expect: tag resolves to
  `v012`, ~68 MB download to `.toolchains/nix-portable/bin/nix-portable`,
  `~/.local/bin/nix-portable` → DEST symlink, `file` reports ELF, verify hook PASS, auto-detect
  `[healthy] wired`. Do NOT run `nix-portable nix ...` — first real run bwrap-bootstraps a store.

## Deviations
- **Install hook does NOT use `mktemp -d`+`trap`.** The plan/U1 said "match whatever the mise block
  does — if mise curls direct to DEST, do the same." The mise block curls straight to
  `$DEST/bin/mise` with no tmp dir; since nix-portable is the same single-binary shape and we
  `rm -rf "$DEST"` first, I curled direct to `$DEST/bin/nix-portable` (no tmp). This matches the
  mise idiom exactly and is simpler/atomic-enough for a single file. (The architect's U1 listed
  `mktemp -d`+`trap` as one option but explicitly made it optional and deferred to the mise idiom.)
- Otherwise implemented exactly per the U1/U2/U3 ledger. U4 (the TASK-0067 backlog card) was NOT
  filed — that is the orchestrator's job per the brief.

## Lock count delta
73 → 74 (additive, net-new id `nix-portable`).

## Pre-existing drift seen
None. `cargo fmt --check` and `cargo clippy -p envctl-engine -p envctl -- -D warnings` were both
clean. I touched only TOML + lock + a doc, so no Rust-line drift was possible; no `--workspace`
clippy run (GUI crate may carry inherited lints per prior cycles, unrelated to this change).

## Handoff notes (guardian)
- **no-C invariant is safe — do NOT false-flag.** The nix-portable binary is a runtime artifact
  downloaded into `.toolchains/` at install time; it is NEVER a Cargo dependency and never linked
  into a workspace crate. `ci/gates/no-c.sh` (cargo-metadata-scoped) ran PASS and is provably
  unaffected.
- **Additive-only / NEVER touches host `/nix`.** detect/install/verify/remove all operate solely on
  `$M/.toolchains/nix-portable` + the `~/.local/bin/nix-portable` symlink. The remove hook is
  symlink-ownership-guarded (only removes `~/.local/bin/nix-portable` if it resolves into our DEST).
  The destructive `/nix` migration is explicitly DEFERRED to supervised TASK-0067 (not in this PR).
- **verify hook is read-only by design** (file + executable + ELF check) — the binary forwards all
  args to nix and has no native `--version`/`--help`; first real run bwrap-bootstraps a store, so
  there is intentionally NO functional `nix-portable nix --version` in verify. This is the correct
  shape, mirroring libgccjit's read-only `file`-based verify.
- **Network install was NOT exercised on-box** (GitHub API 403 in sandbox); the asset URL was
  confirmed HTTP 200 and the tag (`v012`) validated. The guardian should run the install where the
  API is reachable to confirm tag-pin + verify-pass before merge.
- **Lock is machine-generated** (`cargo run -p envctl -- lock`), not hand-edited; `lock --check`
  exits 0 at 74 components.
- Sequential single-implementer; no grit/parallel mode, no symbols claimed.

## Re-run note (guardian FAIL → fixed) — 2 fixes

### F1 (guardian-routed): tag resolution via api.github.com is rate-limit-fragile
- **Fix (manifest-TOML only):** in the nix-portable install hook, replaced the
  `TAG=$(curl ...api.github.com/.../releases/latest | grep tag_name ...)` JSON-API line + its
  hard-abort guard with the web `/releases/latest` redirect (no API, no unauth rate-limit) and a
  last-known-good fallback:
  ```
  TAG="$(curl -fsSLI -o /dev/null -w '%{url_effective}' 'https://github.com/DavHau/nix-portable/releases/latest' 2>/dev/null | grep -oE 'v[0-9]+$')"
  [ -n "$TAG" ] || TAG="v012"
  ```
- **Verified on-box:** the redirect resolves to `TAG=v012` from this box (the JSON API still 403s),
  so the install hook no longer aborts. URL build / `rm -rf DEST` / curl→DEST / chmod / symlink
  unchanged. detect/remove/wiring unchanged.

### F2 (NEW bug surfaced by the now-working on-box install — implementer-fixed, same component scope)
- **Symptom:** install SUCCEEDED (real 68 MB / 68062412-byte artifact downloaded to
  `.toolchains/nix-portable/bin/nix-portable`, symlink + executable correct) but the component went
  `[unhealthy]` — `auto-detect`: "installed but verify failed".
- **Root cause:** the original verify hook (`file "$f" | grep -q 'ELF'`, per the U1 libgccjit idiom)
  is WRONG for this artifact. **nix-portable ships as a self-extracting `#!/usr/bin/env bash`
  wrapper around an embedded ELF payload** — so `file` classifies it as "Bourne-Again shell script,
  ASCII text executable", NOT "ELF". The download was correct; the verify assertion was wrong, so a
  correct install was permanently marked unhealthy.
- **Fix (manifest-TOML only, same component):** verify now checks file-exists + executable + the
  **embedded ELF magic** (binary-safe grep), which proves a real ~68 MB nix-portable binary landed
  (rules out an HTML error/redirect stub) without depending on the misleading `file` class:
  ```
  f="$M/.toolchains/nix-portable/bin/nix-portable"; [ -x "$f" ] && LC_ALL=C grep -qa $'\x7fELF' "$f"
  ```
  Confirmed `\x7fELF` magic IS present in the installed artifact; shebang is `#!/usr/bin/env bash`.
  This stays NO-mutation / NO-network (the architect's verify constraint).

### Re-verify (real output, post-both-fixes)
- On-box `cargo run -p envctl -- install nix-portable` — **SUCCEEDS** (`✓ nix-portable Install`,
  `wiring applied`, exit 0).
- `readlink -f ~/.local/bin/nix-portable` → `/home/drdave/Desktop/meta/.toolchains/nix-portable/bin/nix-portable`
  (resolves into `$META_ROOT/.toolchains`). Artifact `-x` YES, size 68062412 bytes (~68 MB),
  embedded `\x7fELF` magic present. Did NOT run `nix-portable nix ...` (would bwrap-bootstrap a store).
- `cargo run -p envctl -- auto-detect | grep nix-portable` — **`✓ nix-portable (meta-owned) [healthy] wired`**.
- `cargo run -p envctl -- doctor` — nix-portable NOT flagged (healthy); the only doctor flag is a
  pre-existing unrelated `weave` out-of-bound `.cargo/bin` install.
- `cargo run -p envctl -- lock` then `lock --check` — `✓ ... matches (74 components)` exit 0 (the
  install-script + verify bodies changed → content_hash regenerated; count unchanged at 74).
- `cargo fmt --all -- --check` — clean. `cargo clippy -p envctl-engine -p envctl -- -D warnings` —
  clean. `ci/gates/no-c.sh` PASS, `shape.sh` PASS, `loop-state.sh` PASS.
- **Component is left INSTALLED + healthy on the box** (the desired end state); the artifact is real.

### Note on F2 scope
F2 was a real correctness bug (a correct install reads unhealthy forever), surfaced only because the
F1 fix made the on-box install actually run. It's a localized fix to the SAME component's verify hook
(no Rust, no new dep, no design change), so I fixed it rather than handing back — it's exactly the
kind of "failing verify you can see is yours to fix before handoff" case. The architect's "ELF"
intent is preserved (the payload IS an ELF); only the detection method changed from the `file`
classifier (wrong for a polyglot script+ELF) to the embedded-magic grep.

## Re-run note 2 (SUPERSEDES re-run note 1) — owner directive: gh-authenticated fetch + polyglot verify

The coordinator superseded re-run note 1's F1 fix with an owner directive, and refined F2. Both are
manifest-TOML-only, same `nix-portable` block.

### FIX 1 (owner directive) — authenticate the fetch via the meta-owned gh, don't dodge the rate limit
- **Root cause (confirmed):** the box's UNAUTH GitHub API quota is 60/hr and exhausted → 403. But
  the box IS gh-authenticated (`gh auth status`: account `drdave-flexnetos`, keyring, scopes incl.
  `repo`) → authenticated ops get 5000/hr. And `gh` is itself a meta-owned Epic-H component
  (TASK-0057), guaranteed on PATH.
- **Fix:** the install hook now uses authenticated `gh release download --repo DavHau/nix-portable
  --pattern 'nix-portable-x86_64' --output "$DEST/bin/nix-portable" --clobber` as PRIMARY (no
  api.github.com JSON, no tag math, no rate-limit), with the web-redirect + curl + `v012` fallback
  ONLY if gh is absent/unauthed:
  ```
  if command -v gh >/dev/null 2>&1 && gh auth status >/dev/null 2>&1; then
    gh release download --repo DavHau/nix-portable --pattern 'nix-portable-x86_64' --output "$DEST/bin/nix-portable" --clobber
  else
    TAG="$(curl -fsSLI -o /dev/null -w '%{url_effective}' 'https://github.com/DavHau/nix-portable/releases/latest' 2>/dev/null | grep -oE 'v[0-9]+$')"; [ -n "$TAG" ] || TAG=v012
    curl -fsSL "https://github.com/DavHau/nix-portable/releases/download/${TAG}/nix-portable-x86_64" -o "$DEST/bin/nix-portable"
  fi
  ```
  `M=`/`DEST=`/`rm -rf "$DEST"; install -d -m 755 "$DEST/bin"` (moved BEFORE the fetch so both
  branches write into a clean dir) + `chmod +x` + `ln -sfn ... ~/.local/bin/nix-portable` unchanged.

### FIX 2 (refines re-run-note-1's F2) — verify via `grep -qi 'executable'`, not embedded-ELF-magic
- Same root cause: nix-portable is a self-extracting polyglot, `file` reports "Bourne-Again shell
  script, ASCII text executable", so `grep -q 'ELF'` fails on a good install.
- Per the owner/coordinator directive, the verify hook now uses the simpler, future-proof
  `file ... | grep -qi 'executable'` (matches "ASCII text executable" today AND "ELF ... executable"
  if upstream ever repackages) instead of re-run-note-1's `grep -qa $'\x7fELF'` embedded-magic
  check. Still file-exists + executable, still NO mutation/network.

### Re-verify (on-box, post-both-fixes — gh path EXERCISED)
- `gh auth status` → `✓ Logged in to github.com account drdave-flexnetos (keyring)`.
- Cleared the prior artifact (`rm -rf .toolchains/nix-portable` + the symlink) to force a FRESH
  gh-path install, then `cargo run -p envctl -- install nix-portable` → **`✓ nix-portable Install`,
  `wiring applied`, exit 0** (the `gh release download` branch ran, since gh is authed).
- Artifact: 64.9 MB / 68062412 bytes at `.toolchains/nix-portable/bin/nix-portable`, exec bit set;
  `file` → "Bourne-Again shell script, ASCII text executable"; verify-hook simulation
  `file | grep -qi 'executable'` → **PASS**.
- `readlink -f ~/.local/bin/nix-portable` → `/home/drdave/Desktop/meta/.toolchains/nix-portable/bin/nix-portable`
  (resolves into `$META_ROOT/.toolchains`). Did NOT run `nix-portable nix ...`.
- `auto-detect | grep nix-portable` → **`✓ nix-portable (meta-owned) [healthy] wired`**.
- `doctor` → nix-portable NOT flagged (healthy); only the pre-existing unrelated `weave` flag remains.
- `lock` → 74; `lock --check` → `✓ matches (74 components)` exit 0 (install+verify bodies changed →
  content_hash regenerated; count unchanged at 74).
- `cargo fmt --all -- --check` clean; `ci/gates/no-c.sh` / `shape.sh` / `loop-state.sh` all PASS.
- **Component left INSTALLED + healthy on the box.**

### Guardian note: NOTE on the install `--apply` flag
The envctl `install` verb mutates by DEFAULT (`--dry-run` is the opt-in); `install --apply` is
rejected (`unexpected argument '--apply'`). The correct mutating invocation is `install nix-portable`
(or bare `install`). This differs from `auto-fix`/`reset`, which DO take `--apply`. The on-box install
above used `install nix-portable` and succeeded.
