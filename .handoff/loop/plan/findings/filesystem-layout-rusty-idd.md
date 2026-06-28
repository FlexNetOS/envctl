# filesystem-layout — rusty-idd

axis: filesystem-layout
target: rusty-idd
mode: READ-ONLY audit (no production files mutated, no files moved)
repo root: /home/drdave/Desktop/meta/rusty-idd
git toplevel: /home/drdave/Desktop/meta/rusty-idd (own git repo; registered in /home/drdave/Desktop/meta/.meta.yaml:278)
standards applied: FHS 3.0, XDG Base Directory Spec, Cargo/Rust workspace conventions, envctl/meta placement invariants (meta/CLAUDE.md "everything meta uses lives in meta" + ADR-install-locations), repo-local ADR-0018 (faithful-adopt)
date: 2026-06-26

---

## Summary verdict

rusty-idd is a **repo-local product repo** (a genuine FlexNetOS org repo, its own git history,
registered as a meta peer). FHS/XDG apply weakly to a source tree; the load-bearing standards here are
**Cargo workspace layout** and **repo hygiene**. The repo is structurally sound at the `crates/` layer
but is dominated by **vendored upstream trees**: of 4,716 tracked files, **3,744 (79%) live under
`imports/` (1,055) and `third_party/` (2,689)**. Three high-severity hygiene/placement defects:
(1) `crates/config/` masquerades as a crate but is not one; (2) tracked `*.idd-bak-*` snapshot litter
(19 MANIFEST backups on disk, 3 tracked) plus untracked root backup files; (3) triple-vendored
`handoff` / `intent-driven-template` / `codegraph` trees inflating the repo and creating drift surfaces.
No unmanaged global/system/user writes were found from the repo tree itself (all state is repo-local);
build output is correctly externalized to `/target` (ignored, 30 GB).

---

## path inventory

| path | kind | owner | mutability | tracked/ignored | evidence |
|------|------|-------|-----------|-----------------|----------|
| `/home/drdave/Desktop/meta/rusty-idd/Cargo.toml` | workspace root manifest (virtual) | repo | source | tracked | `Cargo.toml:24-38` lists 11 members, `resolver="3"` |
| `crates/cli` | crate `rusty-idd-cli` (bin) | repo | source | tracked | `crates/cli/Cargo.toml:2` |
| `crates/core` | crate `rusty-idd-core` (std-only) | repo | source | tracked | `crates/core/Cargo.toml:2` |
| `crates/runner` | crate `rusty-idd-runner` | repo | source | tracked | `crates/runner/Cargo.toml:2`; god file `runner/src/runner.rs` 2,146 LOC / 74 KB |
| `crates/tui` | crate `rusty-idd-tui` | repo | source | tracked | `crates/tui/Cargo.toml:2`; god file `tui/src/app.rs` 5,708 LOC / 195 KB |
| `crates/spec` | crate `rusty-idd-spec` | repo | source | tracked | `crates/spec/Cargo.toml:2` |
| `crates/merge-tools` | crate `rusty-idd-merge-tools` | repo | source | tracked | `crates/merge-tools/Cargo.toml:2` |
| `crates/knowledge` | crate `rusty-idd-knowledge` | repo | source | tracked | `crates/knowledge/Cargo.toml:2`; god file `knowledge/src/lib.rs` 7,058 LOC / 247 KB — sole file in `src/` |
| `crates/work-order` | crate `work-order` (UNPREFIXED) | repo | source | tracked | `crates/work-order/Cargo.toml:2` — name `work-order`, not `rusty-idd-work-order` |
| `crates/external/codegraph-core` | crate `codegraph-core` (adopted) | upstream/forked | source | tracked | `crates/external/codegraph-core/Cargo.toml:2` |
| `crates/external/codegraph-parser` | crate `codegraph-parser` (adopted) | upstream/forked | source | tracked | `crates/external/codegraph-parser/Cargo.toml:2` |
| `crates/external/repomix-shared` | crate `repomix-shared` (adopted) | upstream/forked | source | tracked | `crates/external/repomix-shared/Cargo.toml:2`; patched via `Cargo.toml:40-41 [patch.crates-io]` |
| `crates/config` | **NOT a crate** — holds only `example.toml` | repo | source/sample | `example.toml` tracked | `crates/config/` has NO `Cargo.toml`; absent from `Cargo.toml` members list |
| `imports/` | vendored "faithfully-adopted" repos (handoff, prompt_hub) | upstream | vendored | **tracked (1,055 files, 17 MB)** | `git ls-files imports/` = 1055; ADR-0018 cited in `.gitignore:1-5,28-30` |
| `third_party/upstream/` | vendored upstream sources (ai-prompt, handoff, grit, prompts.chat, repomix-rs, codegraph-rust) | upstream | vendored | **tracked (2,689 files, 160 MB)** | `git ls-files third_party/` = 2689 |
| `intent-driven-template/` | scaffolding template tree (root copy) | repo | source/template | tracked | duplicated at `imports/handoff/intent-driven-template`, `third_party/upstream/handoff/intent-driven-template` |
| `.idd/` | IDD control-plane state (MANIFEST, LOCK, evidence, runs) | repo | mixed (state + litter) | mixed; `.idd/runs/` ignored (`.gitignore:8`) | `.idd/MANIFEST.tsv` 405 KB + 19 `MANIFEST.tsv.idd-bak-1..19` snapshots (3 tracked) |
| `.handoff/` | continuity-kernel state | repo/harness | runtime state | partially tracked | present at root |
| `adr/` | architecture decision records | repo | docs | tracked | referenced (ADR-0018) |
| `docs/`, `openspec/`, `AI_MERGE/` | docs / spec / merge evidence | repo | docs | tracked | root listing |
| `scripts/`, `Makefile`, `Justfile` | build/dev automation | repo | source | tracked | root listing |
| `.cargo/` | Cargo config (repo-local) | repo | config | tracked | root listing |
| `.githooks/`, `.github/` | VCS hooks + CI | repo | config | tracked | root listing |
| `.agents/`, `.claude/`, `.codex/`, `.devin/`, `.vscode/` | agent/IDE control surfaces | repo | config | mixed (`.vscode` ignored root-only `.gitignore:12`) | root listing |
| `target/` | Cargo build output | tool (cargo) | generated | **ignored** (`.gitignore:6`) | 30 GB; `git ls-files target/` = 0 — CORRECT externalization |
| `.worktrees/` | git worktrees | tool (git) | generated | ignored (`.gitignore:7`) | empty |
| `.env.contract.yaml.idd-bak-1`, `AGENTS.md.idd-bak-1`, `.env.schema.example.json.idd-bak-1` | root backup litter | tool (idd validate) | regenerable litter | ignored (`.gitignore:19`), untracked | `git ls-files --error-unmatch` → untracked |
| `Cargo.lock`, `VERSION`, `LICENSE`, `README.md`, `CLAUDE.md`, `GEMINI.md`, `AGENTS.md` | root manifests/docs | repo | source | tracked | root listing (42 root entries) |

---

## placement verdict (per anomaly)

### V1 — `crates/config/` is not a crate — FINDING (root-of-`crates/` clutter)
Verdict: **MISPLACED**. `crates/` is the Cargo workspace member namespace; every sibling has a
`Cargo.toml` and appears in `Cargo.toml:26-38`. `crates/config/` has only `example.toml` and **no
`Cargo.toml`**, and is **not** a workspace member. A sample/example config under the crate namespace
violates Cargo layout (members are crates) and the "no orphan dirs in the crate namespace" repo-local
convention. Per the standing rule "Missing ownership or root clutter is a finding, not a pass," this is
a finding, not benign. Expected location: a config sample belongs under the owning crate
(`crates/<owner>/examples/config.toml` or `crates/<owner>/config.example.toml`) or under a top-level
`examples/` / `docs/` tree — never as a non-crate dir inside `crates/`.

### V2 — `crates/work-order` package name unprefixed — FINDING (naming-convention drift)
Verdict: **INCONSISTENT**. 7 of 8 first-party crates use the `rusty-idd-*` package-name prefix
(`crates/cli/Cargo.toml:2` `rusty-idd-cli` … `crates/knowledge/Cargo.toml:2` `rusty-idd-knowledge`),
but `crates/work-order/Cargo.toml:2` declares bare `work-order`. This is a publish-namespace/collision
risk (a bare `work-order` could collide on crates.io and is indistinguishable from the vendored
`imports/handoff/work-order` and `third_party/upstream/handoff/work-order` trees). Repo-local
convention = `rusty-idd-*`.

### V3 — God files: `knowledge/src/lib.rs`, `tui/src/app.rs`, `runner/src/runner.rs` — FINDING
Verdict: **STRUCTURE DEFECT (in-file layout)**. `crates/knowledge/src/lib.rs` is **7,058 LOC / 247 KB
and the ONLY file in `crates/knowledge/src/`** — zero module decomposition. `crates/tui/src/app.rs` =
5,708 LOC / 195 KB; `crates/runner/src/runner.rs` = 2,146 LOC / 74 KB. This is a filesystem-layout
concern: a crate's `src/` should reflect its module tree on disk (one concern → one file/dir), not a
single mega-file. Not an OWNER-WALL item (no external write); routed to refactor backlog.

### V4 — Vendored `imports/` (1,055 tracked files, 17 MB) — QUALIFIED / DOCUMENTED
Verdict: **DOCUMENTED DECISION, but a placement liability**. `imports/` holds full "faithfully-adopted"
repos (`imports/handoff/`, `imports/prompt_hub/`) carrying their own `.git`-less dotfolders
(`.claude/`, `.kb/`, `.handoff/`, `.idea/`, `.github/`, `_workspace/`). This is intentional per
**ADR-0018** (cited verbatim in `.gitignore:1-5,28-30`, with `!imports/**` re-include net). Because it
is documented and owned by the repo, it is QUALIFIED — but tracking 1,055 vendored files inside a
product repo is a vendoring-policy liability (review noise, license/security blast radius, drift). The
correct mechanism for upstream code is a git submodule / vendoring lockfile, not a committed copy of
another repo's working tree (including its `_workspace/`, `_workspace_prev/`, `.idea/`).

### V5 — Vendored `third_party/upstream/` (2,689 tracked files, 160 MB) — FINDING
Verdict: **MISPLACED VOLUME**. `third_party/upstream/` vendors entire upstream projects
(`ai-prompt`, `handoff`, `grit`, `prompts.chat`, `repomix-rs`, `codegraph-rust`). `codegraph-rust`
alone vendors 15+ crates (`third_party/upstream/codegraph-rust/crates/codegraph-{core,parser,graph,
vector,mcp,...}`) **plus a duplicate `docs/crates/` mirror of all of them**. Meanwhile the repo also
keeps an **adopted fork subset** at `crates/external/codegraph-core` and `crates/external/codegraph-parser`.
So codegraph exists in **two owners** (forked-in-workspace vs full-upstream-vendored) — duplication and
guaranteed drift. 160 MB of committed third-party source is a vendoring-policy violation regardless of
intent; upstreams belong behind submodules/pinned-vendor manifests, not committed wholesale.

### V6 — Triple-vendored `handoff` / `intent-driven-template` — FINDING (duplication)
Verdict: **DUPLICATION DEFECT**. `handoff` tree appears 3×: `imports/handoff/`,
`third_party/upstream/handoff/`, `imports/prompt_hub/.claude/skills/prompt-loop/handoff`.
`intent-driven-template` appears 3×: root `intent-driven-template/`, `imports/handoff/intent-driven-template/`,
`third_party/upstream/handoff/intent-driven-template/`. Multiple authoritative-looking copies of the
same tree make "which is source of truth" undecidable and silently drift.

### V7 — `.idd-bak-*` snapshot litter — FINDING (tracked + untracked litter)
Verdict: **LITTER, partially TRACKED**. `.idd/` holds 19 `MANIFEST.tsv.idd-bak-1..19` snapshots
(MANIFEST.tsv itself is 405 KB) of which **3 are tracked** (`.idd/MANIFEST.tsv.idd-bak-{1,2,3}` appear
in `git ls-files`) while bak-4..19 are ignored-but-present on disk — an inconsistent state: the
`*.idd-bak-*` ignore rule (`.gitignore:19`) was added after bak-1..3 were already committed, so they
escape the gate. Root-level `.env.contract.yaml.idd-bak-1`, `AGENTS.md.idd-bak-1`,
`.env.schema.example.json.idd-bak-1` are untracked litter sitting beside their sources. Regenerable
snapshots should never be tracked, and a `.gitignore` rule that the actual tracked files violate is a
gate that does not hold.

### V8 — `target/` externalization — PASS
Verdict: **CORRECT**. 30 GB build output under `/target` is ignored (`.gitignore:6`,
`git ls-files target/` = 0). Build artifacts correctly externalized from source per Cargo convention.
Size is a disk-hygiene note, not a placement defect.

### V9 — Unmanaged global/system/user writes — N/A (none found from repo tree)
Verdict: **N/A — no evidence the repo writes outside its own root**. All state (`.idd/`, `.handoff/`,
`target/`, `.worktrees/`) is repo-local. No OWNER-WALL trigger from the static tree. (Runtime write
behavior of the `rusty-idd` binary to `$HOME`/XDG/system paths is OUT OF SCOPE for a static layout
audit and is flagged below as an enforcement gap to verify, not asserted.)

---

## boundary map (what belongs where)

```
SYSTEM-LEVEL (FHS: /usr, /etc, /var)        -> none. rusty-idd installs nothing system-wide. OWNER-WALL if ever proposed.
USER-LEVEL (XDG: ~/.config ~/.local ~/.cache) -> none from repo tree. Any binary write here must be envctl-owned (preview/apply/lock/rollback/parity) -> PROPOSE/OWNER-WALL.
META-LEVEL ($META_ROOT/.toolchains, .meta.yaml) -> registration only: meta/.meta.yaml:278 lists rusty-idd as a peer. Tools rusty-idd USES (cargo, etc.) are envctl-managed in meta, NOT in this repo.
REPO-LOCAL (this audit's domain)
  source        -> crates/{cli,core,runner,tui,spec,merge-tools,knowledge,work-order}  [first-party]
  adopted forks -> crates/external/{codegraph-core,codegraph-parser,repomix-shared}    [owned forks, in workspace]
  config sample -> SHOULD be crates/<owner>/examples/ or /examples  (NOT crates/config/  <- V1 defect)
  docs/spec     -> docs/, adr/, openspec/, AI_MERGE/ (evidence/history per CLAUDE.md)
  control-plane -> .idd/ (state, NOT snapshot backups), .handoff/ (continuity kernel)
  agent/IDE cfg -> .claude/ .codex/ .agents/ .devin/ .github/ .githooks/ .cargo/
  generated     -> /target (ignored), /.worktrees (ignored), .idd/runs (ignored)   [PASS]
  vendored      -> imports/ (ADR-0018 faithful-adopt) + third_party/upstream/  <- belong behind
                   submodule/pinned-vendor manifest, NOT 3,744 committed files (V4/V5/V6)
  litter        -> *.idd-bak-* (regenerable; must be fully untracked + ignored)  <- V7 defect
```

Boundary integrity: the only boundary rusty-idd touches beyond REPO-LOCAL is META-LEVEL **registration**
(`.meta.yaml`). It does not (statically) write to USER/SYSTEM levels. The internal repo-local boundary
between `crates/` (workspace members), `crates/external/` (owned forks), `imports/`+`third_party/`
(vendored upstream), and `.idd/`/`.handoff/` (control-plane) is **blurred** by: V1 (non-crate in crate
namespace), V5/V6 (codegraph forked AND vendored = two owners), and V7 (litter tracked alongside state).

---

## UPGRADE rows (axis: filesystem-layout)

| id | axis | finding | expected location | migration plan | acceptance test | risk tier | reversibility |
|----|------|---------|-------------------|----------------|-----------------|-----------|---------------|
| FL-1 | filesystem-layout | V1 `crates/config/` non-crate clutter | `crates/<owner>/examples/config.toml` OR `/examples/config.toml` | `git mv crates/config/example.toml examples/rusty-idd-config.example.toml`; rmdir `crates/config`; update any path refs | gate: assert every dir in `crates/` (except `external/`) contains a `Cargo.toml`; `cargo metadata` member count unchanged (11) | low | trivial (`git mv` back); no code refs to break |
| FL-2 | filesystem-layout | V2 `work-order` unprefixed package name | package name `rusty-idd-work-order` | rename `[package].name` in `crates/work-order/Cargo.toml`; update dependents + `Cargo.lock` | gate: `cargo metadata` shows no first-party package without `rusty-idd-` prefix; `cargo build -p rusty-idd-work-order` green | medium | rename back; touches dependents |
| FL-3 | filesystem-layout | V3 god files (knowledge 7058 LOC single-file, tui app.rs 5708, runner 2146) | module-decomposed `src/` trees (`knowledge/src/{mod dirs}`) | split by concern into submodules behind unchanged public API; per FF refactor cycle (behavior-preserving) | golden: public API surface + tests unchanged pre/post; gate: no `src/*.rs` > 1500 LOC in first-party crates | medium | revert split commit; API unchanged so callers safe |
| FL-4 | filesystem-layout | V5/V6 `third_party/upstream/` 160 MB / 2689 files vendored wholesale + codegraph dual-owner | upstreams behind git submodules or a pinned vendor manifest; single codegraph owner (`crates/external/` only) | move `third_party/upstream/*` to submodules or `vendor/` with `cargo vendor` lock; delete in-workspace duplicate of whichever codegraph owner loses; document in ADR | gate: tracked-file count under `third_party/` drops to manifest+submodule pointers; `cargo build` green from pinned vendor; no two dirs named `codegraph-core` are both workspace-buildable | high | submodule add is reversible; deletion needs git history (recoverable) — snapshot first |
| FL-5 | filesystem-layout | V4 `imports/` 1055 vendored files (ADR-0018) | submodule/vendor-manifest if upstream; or keep but lockfiled | confirm ADR-0018 still intends committed working trees incl. `_workspace/`, `.idea/`; if not, convert to submodule; if yes, exclude derived subdirs (`_workspace_prev/`, `.idea/`) | gate: `imports/**/_workspace_prev` and `imports/**/.idea` not tracked; ADR-0018 link present in `.gitignore` | medium | ADR-governed; reversible per-subtree |
| FL-6 | filesystem-layout | V7 tracked `.idd/MANIFEST.tsv.idd-bak-{1,2,3}` + untracked root `*.idd-bak-1` litter | zero `*.idd-bak-*` tracked; all ignored | `git rm --cached .idd/MANIFEST.tsv.idd-bak-{1,2,3}` and any other tracked `*.idd-bak-*`; keep `.gitignore:19` rule | gate: `git ls-files | grep -c idd-bak` == 0; doctor: warn if `.idd/*.idd-bak-*` count > N on disk | low | `git rm --cached` is metadata-only; files stay on disk |

---

## Feature-Forge enforcement handoff (make drift fail in CI)

These are RED-test specs for Feature Forge to BUILD (this audit is read-only; it authors no production
code). Each makes a finding above fail-closed in CI.

1. **unit — crate-namespace purity (FL-1)**: test parses `cargo metadata`; asserts every child dir of
   `crates/` except `crates/external/` is a workspace member with a `Cargo.toml`. RED today (fails on
   `crates/config/`). File: new `tests/layout_crate_namespace.rs` in `crates/cli` or a workspace xtask.

2. **unit — package-name prefix (FL-2)**: test asserts every first-party package name starts with
   `rusty-idd-` (allowlist `crates/external/*`). RED today (fails on `work-order`).

3. **golden — vendor inventory drift (FL-4/FL-5/FL-6)**: golden file records the expected
   tracked-file count (or hash-set) under `imports/` and `third_party/`; CI diffs `git ls-files` against
   it so unreviewed vendored additions fail. Pair with: assert `git ls-files | grep idd-bak` == 0 (FL-6).

4. **doctor — file-size lint (FL-3)**: `rusty-idd doctor` subcommand (or xtask) warns/fails when any
   first-party `crates/*/src/**.rs` exceeds a LOC threshold (e.g. 1500); RED on knowledge/tui/runner.

5. **gate — single-owner check (FL-4)**: CI script fails if a crate name (e.g. `codegraph-core`) is
   buildable from two distinct workspace/vendor locations simultaneously.

6. **gate — gitignore-holds invariant (FL-6)**: CI asserts no tracked file matches a pattern in
   `.gitignore` (`git ls-files -ci --exclude-standard` returns empty) — catches the "rule added after
   commit" class that let `*.idd-bak-*` slip.

---

## Laws honored
- READ-ONLY: no production file mutated, no file moved (only this findings doc written).
- fail-closed: every UPGRADE carries an acceptance test that fails on drift.
- cited: every finding cites a real path / `file:line` / `git ls-files` count.
- no unmanaged global/system/user write found from the static tree (V9 = N/A with rationale; runtime
  binary write-paths flagged as an enforcement gap to verify, not asserted).
