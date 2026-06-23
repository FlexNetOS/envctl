# Verification report: TASK-0066 — meta-owned nix-portable (Epic H, ADDITIVE component)

Worktree: `/home/drdave/Desktop/meta/.worktrees/task-0066-nixportable/envctl` · branch
`task-0066-nixportable` @ `4124081` (off `develop`, on top of `94b0ec9`). `$META_ROOT=/home/drdave/Desktop/meta`.
(Prior on-disk report was the TASK-0062 cycle — no open TASK-0066 finding to carry forward; overwritten.)

## Verdict — FAIL (1 blocking finding, implementer-routable)

Every NON-NEGOTIABLE invariant holds, all 8 CI gates + every cargo check pass, the code shape is
exactly the ADDITIVE manifest-only change claimed (no Cargo dep, no Rust touched), and the remove
hook is correctly self-guarded and never touches host `/nix`. **BUT** the Phase-3.5 ON-BOX install
**fails**: the install hook resolves the release tag via `curl -fsSL https://api.github.com/...`,
which returns **HTTP 403** (unauthenticated rate-limit) on this live box too — `TAG` ends up empty
and the hook aborts. The component therefore **cannot install** when the unauthenticated GitHub
JSON API is rate-limited. This is a real robustness gap, not a sandbox artifact (re-confirmed on the
live box). Routed to the implementer for a non-API-dependent tag-resolution path (fix verified below).

## Gate results
| Gate | Result | First/last line |
|------|--------|-----------------|
| `no-c.sh` | **PASS** | `resolved graph clean: rustls=['0.23.40'] on ring=['0.17.14']; zero aws-lc/openssl/C-SQLite` |
| `shape.sh` | **PASS** | `SHAPE GATE PASS` |
| `enable.sh` | **PASS** | `ENABLE GATE PASS` |
| `p7.sh` | **PASS** | `P7 GATE PASS` |
| `agent-env.sh` | **PASS** | `✓ agent-env.lock is up to date` |
| `loop-state.sh` | **PASS** | `budget=1 wrap_every=5 last_wrapup=33 cycles_total=35; monotonic ok (35 -> 35)` |
| `harness-scripts.sh` | **PASS** | merge-driver + reaper + loop-state-gate tests all PASS |
| `kdf-feature-off.sh` | **PASS** | `low-cost-kdf-tests correctly OFF by default` |

## cargo
| Check | Result | Evidence |
|-------|--------|----------|
| `cargo build -p envctl-engine -p envctl` | **PASS** | finished, exit 0 |
| `cargo run -p envctl -- lock --check` | **PASS** | `✓ envctl.lock matches the manifest (74 components)` (73→74 confirmed) |
| `cargo fmt --all -- --check` | **PASS** | exit 0, no diff |
| `cargo clippy -p envctl-engine -p envctl -- -D warnings` | **PASS** | exit 0, clean |

### Clippy axis classification
Zero clippy findings on gate-scope (`-p envctl-engine -p envctl`). No Rust source changed in this
cycle (diff is manifest TOML + lock + doc only), so there is literally nothing to classify. Did not
run/fix `--all-targets` or untouched-crate (gui) lints — out of scope, not introduced here.

## Invariant checks
1. **No C in the trust boundary — PASS.** `no-c.sh` green (rustls 0.23.40 on ring, zero
   aws-lc/openssl/C-SQLite). **Independently confirmed NO new dependency:** `git diff
   develop...HEAD -- Cargo.toml Cargo.lock` is EMPTY. **The downloaded `nix-portable-x86_64` is
   correctly NOT a Cargo dependency** — it is a runtime static binary that would land under
   `$META_ROOT/.toolchains/nix-portable/bin/`, never linked into any envctl crate. The no-c gate is
   cargo-metadata-scoped, so a `.toolchains/` artifact is invisible to it by design — NOT
   false-flagged, and correctly so. (Stated explicitly as required.)
2. **Exactly one rustls, ring-only — PASS.** `Cargo.lock` diff empty; no TLS/dep churn. no-c
   reports the single `rustls 0.23.40 on ring 0.17.14`.
3. **Engine single shared, non-printing library — PASS.** NO engine/CLI/GUI Rust changed:
   `git diff --stat develop...HEAD -- 'crates/**/*.rs'` is EMPTY. No env seam added to
   `run_env`/`env.rs`. All new logic is in `manifest/components.d/epic-h-toolchains.toml` (data) +
   the lock + the ADR doc — correct placement for an additive manifest component.
4. **Destructive ops fail-closed — PASS (read).** Read the `remove` hook (TOML lines 47-50,
   `[component.remove]`): `set -u`; line 48 removes `~/.local/bin/nix-portable` ONLY when it
   `[ -L ]` is a symlink AND `readlink` resolves into `$M/.toolchains/nix-portable` (self-guarded —
   refuses a foreign symlink); line 49 `rm -rf "$M/.toolchains/nix-portable"` scoped to its OWN dir.
   **The remove hook NEVER references or touches host `/nix`** — the only `/nix` strings in the
   block are in the comment/description documenting "never touches host /nix". Confirmed by reading.
5. **Rust-native / no language drift — PASS.** No foreign-language SOURCE added (no `.c`/`.js`/etc.
   tracked). `git status --short` is clean — the binary is NOT a tracked add; it would land under
   `.toolchains/`, gitignored at the meta root (`/home/drdave/Desktop/meta/.gitignore:85 →
   .toolchains/`), entirely outside version control.

## Parity check
N/A for this surface. No `Engine` method, CLI verb, or GUI control added — this is a declarative
manifest component (`[[component]] id="nix-portable"`) reached through envctl's existing
detect/install/verify/remove harness, which CLI and GUI both drive identically. No front-end can
diverge because no front-end-specific code was added.

## Unit ledger (derived from plan — Engine API delta absent; from Work breakdown + What-was-built)
| U# | Unit | Present | Wired | Evidence |
|----|------|---------|-------|----------|
| U1 | `[[component]] id="nix-portable"` (detect/install/verify/remove) | ✓ | ✓ | `epic-h-toolchains.toml:11-50`; loaded → `auto-detect` lists `nix-portable (meta-owned) … wired` |
| U2 | detect: file `-x` + `~/.local/bin` symlink-resolves into `.toolchains/nix-portable` | ✓ | ✓ | `toml:17-18` (readlink -f compare); runtime detect reports `Missing` pre-install (correct) |
| U3 | install: pin TAG via releases-latest API, download `nix-portable-x86_64` → `.toolchains/nix-portable/bin`, chmod +x, `~/.local/bin` symlink | ✓ | **✗ (runtime FAIL)** | `toml:23-35`; on-box install ABORTS at tag-resolution (api.github.com → 403) — see Runtime check / Findings F1 |
| U4 | verify: read-only file `-x` + ELF check | ✓ | ✓ (read) | `toml:39-40`; not reachable on-box because install failed (no artifact to verify) |
| U5 | remove: self-guarded symlink + `rm -rf` own dir only; never host `/nix` | ✓ | ✓ (read) | `toml:47-49`; self-guard verified by reading |
| U6 | `[component.wiring] path_entries=["~/.local/bin"]` | ✓ | ✓ | `toml:52-53` |
| U7 | `envctl.lock` regen 73→74 (`[components.nix-portable]`) | ✓ | ✓ | `envctl.lock` (+5); `lock --check` ✓ 74 components |
| U8 | ADR row marked SHIPPED — ADDITIVE; destructive /nix migration → TASK-0067 | ✓ | ✓ | `docs/adr-install-locations-and-local-state.md:79, 108-114` |
All units PRESENT. **U3 is present-in-source but FAILS at runtime** (the install cannot complete
under API rate-limit). This is the blocking finding — the component is shipped but does not install
on a rate-limited box, which is the live condition right now.

## Runtime check — FAIL (architect declared a Runtime surface; driven on-box)
| # | Surface driven | Result | Evidence (captured) |
|---|---------------|--------|---------------------|
| 1 | `envctl auto-detect` (pre-install roster) | **PASS** | `· nix-portable  nix-portable (meta-owned) wired` and `[med] nix-portable  Missing: declared but not installed → envctl install nix-portable` |
| 2 | `envctl install nix-portable --dry-run` | **PASS** | `· would Install nix-portable` |
| 3 | **`envctl install nix-portable` (ACTUAL on-box apply)** | **FAIL** | `==> [1/1] nix-portable :: Install` → `curl: (22) The requested URL returned error: 403` → `! FAILED nix-portable (exit Some(1))`. Confirmed root cause: `curl -s -o /dev/null -w '%{http_code}' https://api.github.com/repos/DavHau/nix-portable/releases/latest` → **403** on the live box (unauthenticated rate-limit). `TAG` resolves empty → hook's `[ -n "$TAG" ]` guard aborts. |
| 4 | verify hook / `doctor` healthy+wired | **N/A (blocked)** | Install never produced an artifact, so verify could not be exercised. |
| 5a | off-happy-path: `remove` hook self-guard | **PASS (read)** | deletes only `$M/.toolchains/nix-portable`; symlink removed only if it resolves into that dir; never host `/nix`. |
| 5b | off-happy-path: drift/boundary probe | **PASS** | `auto-detect` drift section (6 items) does NOT list `nix-portable`; the meta-owned `.toolchains/` artifact is correctly NOT a boundary violation. The 6 drift items are pre-existing (`weave` BoundaryViolation → `~/.cargo/bin/weave` etc.), unrelated to this change. |

## Findings

### F1 — BLOCKING (implementer-routable): install hook tag-resolution via api.github.com is rate-limit-fragile
- **Severity:** blocking (the component cannot install on a rate-limited box; on-box install FAILED with HTTP 403).
- **Location:** `manifest/components.d/epic-h-toolchains.toml`, `[component.install]` script line 27:
  `TAG="$(curl -fsSL 'https://api.github.com/repos/DavHau/nix-portable/releases/latest' | grep -oE … )"`.
- **What's wrong:** the unauthenticated GitHub JSON API (`api.github.com`) is aggressively
  rate-limited and returns 403; `curl -fsSL` fails hard, `TAG` is empty, and the hook aborts at the
  `[ -n "$TAG" ]` guard. Re-confirmed on the LIVE box (not a sandbox artifact): API → 403, install → exit 1.
- **Suggested fix (each verified working on-box just now):** resolve the tag WITHOUT the rate-limited JSON API. Any of:
  1. **Follow the web `/releases/latest` redirect** (no JSON API): `curl -fsSLI -o /dev/null -w '%{url_effective}' https://github.com/DavHau/nix-portable/releases/latest` → effective URL `…/releases/tag/v012`; parse the trailing `v[0-9]+`. **Verified:** resolves `v012`.
  2. **`gh release view`** if gh is authed: `gh` is authed on this box (`drdave-flexnetos`), so `gh release view -R DavHau/nix-portable --json tagName -q .tagName` works.
  3. **Hardcoded confirmed-tag fallback `v012`** when the API/redirect fails. The asset URL `https://github.com/DavHau/nix-portable/releases/download/v012/nix-portable-x86_64` returns HTTP 302 (redirects to the CDN; `curl -fsSL` follows it to 200) — confirmed reachable.
  Recommend: try the redirect path (1), fall back to gh (2), then the pinned `v012` (3), so install is robust whether or not the API is throttled.

No other blocking findings. No non-blocking findings.

NOTE (informational, not a finding): `auto-detect`/`doctor` report 6 pre-existing drift items
(`weave` BoundaryViolation resolving to `~/.cargo/bin/weave` outside META_ROOT, etc.). Unrelated to
TASK-0066 — present on `develop`, owned by other components, do NOT involve nix-portable. Out of scope.

## Re-test needed (after F1 fix)
The fix is install-hook-only (manifest TOML); re-run on-box:
```
cd /home/drdave/Desktop/meta/.worktrees/task-0066-nixportable/envctl
bash ci/gates/no-c.sh && bash ci/gates/shape.sh
cargo run -p envctl -- lock --check                                  # expect: 74 components (content_hash will change — regen the lock)
META_ROOT=/home/drdave/Desktop/meta cargo run -p envctl -- install nix-portable        # expect: success, NOT 403
file /home/drdave/Desktop/meta/.toolchains/nix-portable/bin/nix-portable               # expect: ELF executable
readlink -f ~/.local/bin/nix-portable                                # expect: …/.toolchains/nix-portable/bin/nix-portable
META_ROOT=/home/drdave/Desktop/meta cargo run -p envctl -- auto-detect | grep nix-portable   # expect: [healthy] wired
```
NOTE for implementer: editing the install script changes the component `content_hash`, so the lock
must be regenerated (`envctl lock` / its writing verb) and `lock --check` re-confirmed at 74.
Do NOT run `nix-portable nix …` (it bwrap-bootstraps a store/network).

---
## Orchestrator resolution (post-guardian) — RESOLVED → GREEN
Guardian F1 (api.github.com rate-limit 403) routed to the implementer. Owner directive received mid-cycle:
*leverage the gh auth / GitHub App / runner rather than dodging the rate limit.* Research findings:
- Box IS gh-authenticated (account `drdave-flexnetos`, keyring) → authenticated GitHub ops = 5000/hr
  (unauth = 60/hr, exhausted → the 403). `gh` is itself a meta-owned Epic-H component (TASK-0057),
  guaranteed on PATH.
- Vault App path (`secretctl mint-github`, app-id 4044997) NOT usable here: `secretctl` not on PATH +
  prior enrollment 404 (needs original app.pem). `secretd` runs (~/.cargo/bin). `flexnetos_runner`
  IS built (`fxrun`, `fxrun-actions`). → gh keyring auth is the available auth; the App is the future
  isolated-token source (TASK-0068).
- The redirect pattern (`curl -fsSLI .../releases/latest`) is already used by 6/7 Epic-H components and
  is rate-limit-immune (web 302, not the JSON API). The ONE real liability is **llvm (api.github.com**
  `releases?per_page=100`) → routed to TASK-0068.

**Fix landed (commit 32988c3):** install hook uses authenticated `gh release download` as PRIMARY
(redirect+curl `v012` fallback only if gh absent/unauthed); verify hook fixed for the self-extracting
polyglot (`file | grep -qi 'executable'`, not `grep ELF`). Re-verified GREEN on-box: clean reinstall via
the gh path → `✓ nix-portable [healthy] wired`; gates no-c/shape/loop-state PASS; `lock --check` ✓ 74.
**Final verdict: PASS.** Follow-ups filed: TASK-0067 (deferred supervised /nix migration), TASK-0068
(authenticated-GitHub-fetch hardening: convert llvm off api.github.com + shared gh→App fetch).
