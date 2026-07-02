# filesystem-layout — handoff (continuity kernel, cycle 2; union with rusty-idd)

**Axis:** `filesystem-layout` · **Target:** `handoff` (the `hf` continuity kernel)
**Read-only worktree:** `/home/drdave/Desktop/meta/.worktrees/plan-handoff-cycle2/handoff` @ `f6abf96`
**North-star RESIDENCY (owner):** north-star lives @ `$META_ROOT + handoff` → handoff must be a clean, portable kernel ROOT.
**Date:** 2026-06-26 · **Baselines:** FHS 3.0, XDG, Rust/Cargo, envctl/meta invariant, repo-local conventions.
**Laws applied:** read-only; fail-closed; every row cites a path + a standard/convention.

---

## 0. Verification of inbound KEY FACTS (cite-or-correct)

| Inbound claim | Verdict | Evidence |
|---|---|---|
| 21 Cargo members (16 kernel + 5 rusty-idd-lineage under `crates/`) | CONFIRMED | `Cargo.toml` `members = [...]` lists 16 kernel + `crates/{cli,core,runner,spec,tui}`. |
| `hf` + `ledger` depend on RuVector via `../../RuVector/*` PATH deps | CONFIRMED | `ledger/Cargo.toml:16-20` (`rvf-runtime/index/types/crypto = { path = "../../RuVector/crates/rvf/..." }`); `hf/Cargo.toml:48,52,57` (`ruvector-verified`, `ruvector-domain-expansion`, `cognitum-gate-tilezero` all `../../RuVector/...`). |
| `_workspace/` + `_workspace_prev/` are **committed** working dirs | **REFUTED** | `git ls-files _workspace _workspace_prev` → 0 tracked; dirs absent on disk; `.gitignore` lines `/_workspace/`, `/_workspace_prev/` ignore them. They are **ignored ephemeral** working dirs, not committed. Re-scoped below as a gitignore-hygiene row, not root clutter. |
| Vendored `crates/tui/vendor/syntect` | **CORRECTED** | The vendored syntect fork is at **repo root** `vendor/syntect/` (`Cargo.toml` `[patch.crates-io] syntect = { path = "vendor/syntect" }`; `.gitignore` `vendor/syntect/target|testdata|Cargo.lock`). `crates/tui/Cargo.toml` has **no** `syntect` ref (0 matches) and **no** `crates/tui/vendor/` dir exists. `tui` pulls syntect transitively via `tui-markdown`; the patch is workspace-global. |
| `intent-driven-template/` + `spike/` non-member dirs | CONFIRMED | Neither appears in `Cargo.toml` members (`grep -c "intent-driven\|spike" Cargo.toml` → 0). `intent-driven-template/openspec/schemas/intent-driven/schema.yaml` (1 tracked file); `spike/ruvocal-mcp-bridge/` (6 tracked files, Node.js). |

---

## 1. Path inventory

`kind` ∈ source|test|doc|script|config|manifest|generated|cache|state|toolchain|artifact|spike.
Tracked = git-tracked; Ignored = matched by `.gitignore`. Owner = which plane/role.

### 1a. Root manifests / metadata (tracked)

| path | kind | owner | mutability | tracked | evidence |
|---|---|---|---|---|---|
| `Cargo.toml` | manifest | workspace | low | yes | `[workspace] resolver="3"`, 21 members, `[patch.crates-io] syntect`. |
| `Cargo.lock` | manifest | cargo | churn | yes | 97.2K; correct to commit (workspace ships binaries `hf`/`hf-mcp`/`rusty-idd`). |
| `VERSION` | manifest | release | low | yes | `0.1.0`; mirrors `release-please`. |
| `release-please-config.json`, `.release-please-manifest.json` | config | release | low | yes | release-please plane. |
| `renovate.json`, `commitlint.config.cjs`, `qodana.yaml` | config | CI/lint | low | yes | tooling configs at root (tool-conventional). |
| `.gitattributes`, `.gitignore` | config | git | low | yes | `.gitattributes` forces `eol=lf` (oracle-fixture byte-stability). |
| `Makefile` | script | build | low | yes | dev entrypoints. |

### 1b. Root docs / control-plane (tracked)

| path | kind | owner | mutability | tracked | evidence |
|---|---|---|---|---|---|
| `AGENTS.md`, `CLAUDE.md` (33K), `CONTRIBUTING.md` | doc | governance | med | yes | control-plane docs (governance axis owns content). |
| `NORTH-STAR.md`, `FLEET_GUIDE.md`, `LESSONS.md`, `NEEDS-HUMAN.md` | doc | governance | med | yes | direction/runbook layer. |
| `docs/` (ADR-0001..0019, `Continuity_Ledger_Kernel_PRD.md`, `ARCHITECTURE.md`, `TEST_MATRIX.md`, `backlog.yaml`, …) | doc | docs | med | yes | canonical docs surface (repo-local). |
| `docs/rusty-idd/{security-advisories.md,spec-engine-design.md,oracle-fixtures/}` | doc/fixture | docs | med | yes | rusty-idd-lineage docs + oracle fixtures (consumed by `include_str!`). |

### 1c. Cargo members — kernel group (16, tracked source)

| path | pkg | kind | owner | evidence |
|---|---|---|---|---|
| `hf/` (`src/`, `tests/`, `Cargo.toml`, `build.rs`) | `hf` | source | kernel | hub CLI; 2 bins `hf`,`hf-mcp` (`hf/Cargo.toml:9-14`). |
| `ledger/`, `work-order/`, `handoff-core/`, `handoff-schema/`, `handoff-policy/`, `handoff-fleet/`, `handoff-drift/`, `handoff-lease/`, `handoff-gatekeeper/`, `handoff-hooks/`, `handoff-index/`, `handoff-intake/`, `handoff-route/`, `handoff-secrets/`, `handoff-test-support/` | (per-dir) | source | kernel | each `<crate>/src` + `Cargo.toml`; standard Cargo layout. |

### 1d. Cargo members — rusty-idd lineage (5, tracked source)

| path | pkg | kind | owner | evidence |
|---|---|---|---|---|
| `crates/cli/` | `rusty-idd-cli` | source | ridd | 3rd binary `rusty-idd`. |
| `crates/core/`, `crates/runner/`, `crates/spec/` | `rusty-idd-{core,runner,spec}` | source | ridd | shared-lineage forks (codemap §4). |
| `crates/tui/` (+ `crates/tui/.claude/`, `crates/tui/openspec/`) | `rusty-idd-tui` | source | ridd | nested `.claude/skills` (OpenSpec) + `openspec/` doc dir inside the crate. |

### 1e. Vendored / toolchain (tracked source, ignored build output)

| path | kind | owner | mutability | tracked | evidence |
|---|---|---|---|---|---|
| `vendor/syntect/` | toolchain (vendored fork) | workspace | low | yes (src); `target/`,`testdata`,`Cargo.lock` ignored | `[patch.crates-io] syntect = { path = "vendor/syntect" }`; postcard fork (RUSTSEC-2025-0141 remediation), `docs/rusty-idd/security-advisories.md`. |
| `.cargo/audit.toml` | config | cargo-audit | low | yes | cargo-audit advisory config (repo-local Cargo convention). |

### 1f. Continuity state — `.handoff/` (mixed: tracked text + ignored binary)

| path | kind | owner | mutability | tracked | evidence |
|---|---|---|---|---|---|
| `.handoff/{tasks,decisions,context,deliveries,packets,policies,hooks,fleet,skills}/`, `active.md`, `policy.toml` | state (durable text) | kernel | high | yes (158 files) | ADR-0016/0018 D1 durability policy. |
| `.handoff/ledger.events.jsonl` (164K) | state (durable export) | kernel | high | yes | committed continuity truth; re-derives binary via `hf import`. |
| `.handoff/**/ledger.db`, `*.db-wal/-shm`, `*.rvf`, `*.rvf.lock`, `maps/`, `locks/`, `workspaces/` | cache/runtime | kernel | ephemeral | ignored | `.gitignore` `.handoff/**/ledger.db` etc. (correct text-vs-binary split). |
| `.handoff/loop/` | state | harness | high | yes | planning-loop durable dir. |

### 1g. Knowledge / tooling state

| path | kind | owner | mutability | tracked | evidence |
|---|---|---|---|---|---|
| `.kb/store/**` | state (durable text) | gitkb | med | yes | committed; `.kb/.cache/`, `.kb/workspaces/`, `.kb/config.toml` ignored (ADR-0003/0018 D7). |
| `.githooks/{commit-msg,pre-commit,pre-push}` | script | git | low | yes | repo-local hook scripts. |
| `.github/` | config | CI | low | yes | workflows. |
| `.agent/skills-catalog.md` (313K) | artifact (generated catalog) | agent tooling | churn | yes | single 313K generated skills catalog committed at `.agent/`. |
| `.idea/` (13 files: `*.iml`, `runConfigurations/`, `deployment.xml`, `material_theme_*`, `git_toolbox_*`) | config (IDE) | user/IDE | low | yes | JetBrains IDE project files committed. |
| `.release-please-manifest.json` | (see 1a) | | | | |

### 1h. Non-member subtrees (tracked, NOT Cargo members)

| path | kind | owner | mutability | tracked | evidence |
|---|---|---|---|---|---|
| `intent-driven-template/openspec/schemas/intent-driven/schema.yaml` | doc/template | ridd | low | yes (1 file) | not in `members`; orphan template tree at root. |
| `spike/ruvocal-mcp-bridge/` (`package.json`, `package-lock.json`, `index.js`, `servers.json.example`, `test/smoke.js`, `README.md`) | spike (Node.js) | spike | low | yes (6 files) | not in `members`; Node spike at root; `node_modules/` ignored. |

### 1i. Ephemeral (ignored, absent on disk)

| path | kind | tracked | evidence |
|---|---|---|---|
| `/_workspace/`, `/_workspace_prev/` | state (working dir) | ignored | `.gitignore`; 0 tracked; absent in this worktree. |
| `/target`, `node_modules/`, `.grit` | cache | ignored | `.gitignore`. |

---

## 2. Placement verdicts

Status ∈ OK | DRIFT | LEGACY-COMPAT | OWNER-WALL | UNKNOWN. Standard ∈ FHS | XDG | Rust-Cargo | envctl-meta | repo-local.

| # | path | verdict | standard | expected location | citation / rationale |
|---|---|---|---|---|---|
| V1 | `ledger/Cargo.toml:16-20` RuVector `../../RuVector/*` path deps | **DRIFT (residency-breaking)** | Rust-Cargo + envctl-meta | a residency-stable source (vendored sibling, `[patch]`-pinned, or published crate) | path escapes the kernel root to `$META_ROOT/RuVector`; **the north-star "$META_ROOT + handoff = portable kernel root" is false** — moving/cloning `handoff` alone leaves `hf`/`ledger` unbuildable. In THIS worktree `../../RuVector` does not exist (CI clones `meta-ruvector` as sibling, `hf/Cargo.toml:46`). See V2. |
| V2 | `hf/Cargo.toml:48,52,57` `ruvector-verified`/`-domain-expansion`/`cognitum-gate-tilezero` `../../RuVector/*` | **DRIFT (residency-breaking)** | Rust-Cargo | same as V1 | same `../../` escape; widens the standalone blocker beyond `ledger`. |
| V3 | `crates/{cli,core,runner,spec,tui}` (rusty-idd-lineage forks) | **DRIFT (duplicate-lineage)** | Rust-Cargo + repo-local | a single canonical crate set (dedup ~95% shared with `rusty-idd`) | codemap §4 + union: `tui`~0.1%/`spec`~1.2% near-identical, `runner`~11%, `core`~33%, `cli`~40% diverged; handoff's copies are the **stale partial fork**. Two repos carry `rusty-idd-{cli,core,runner,spec,tui}` under the SAME pkg names → workspace-collision risk on union. See §4. |
| V4 | `vendor/syntect/` (root) consumed via workspace `[patch.crates-io]` | **OK** | Rust-Cargo | repo-root `vendor/` | standard cargo vendoring; build artifacts ignored; security rationale documented (`docs/rusty-idd/security-advisories.md`). NOTE: patch is workspace-global, so it travels with the union (good for a single kernel root). |
| V5 | `.handoff/` durable text tracked, binary ledger ignored | **OK** | repo-local | `.handoff/` | ADR-0016/0018 D1 text-vs-binary split is explicit and correct (`.gitignore`). |
| V6 | `.kb/store/**` tracked, `.kb/.cache/`+`workspaces/` ignored | **OK** | repo-local | `.kb/` | ADR-0003/0018 D7; mirrors `.handoff` precedent. |
| V7 | `.idea/` (13 IDE files committed) | **DRIFT** | XDG + repo-local | un-track; user/IDE config belongs in the dev's `$XDG_CONFIG_HOME`/global gitignore, not the kernel root | JetBrains files are **user-level** state crossing into a **repo-local** portable kernel; clutters the north-star root and leaks one dev's machine config (`deployment.xml`, `material_theme_project_new.xml`). `runConfigurations/` is convenience, not kernel truth. |
| V8 | `.agent/skills-catalog.md` (313K, generated) | **DRIFT** | repo-local | a `generated`/ignored surface, or regenerate-on-demand | a 313K generated catalog committed as a single blob churns the kernel root with derived data (same class as `.handoff/maps/`, which is correctly ignored). No regen/owner doc found. |
| V9 | `intent-driven-template/` (1-file orphan tree) | **DRIFT** | repo-local | `docs/` template surface or a registered crate/asset dir | a 3-level-deep single-file template at the kernel root with no member/owner; root clutter under the portability mandate. |
| V10 | `spike/ruvocal-mcp-bridge/` (Node.js spike) | **DRIFT (mixed-ecosystem root clutter)** | repo-local + Rust-Cargo | a `spikes/` doc-surface, a separate repo, or removal | a JS/Node experiment (`package.json`) living at a **pure-Rust** (`unsafe_code=deny`, no-C) kernel root; non-member, untyped, no purpose doc beyond README. Violates "clean portable kernel root". |
| V11 | `/_workspace/`, `/_workspace_prev/` | **OK (ignored)** | repo-local | ignored ephemeral | `.gitignore` `/_workspace/` `/_workspace_prev/`; not committed (inbound "committed" claim refuted). No action beyond keeping the ignore. |
| V12 | `schemas/{task,packet,session}.schema.json` (root) | **LEGACY-COMPAT / DRIFT (generated-at-root)** | repo-local | mark `task.schema.json` as **generated** (schemars source-of-truth is `work-order::WorkOrder`) and gate regen | codemap §5.1: `title:"WorkOrder"` is generated from `work-order`; a committed generated file at root with no `generated/` marker invites drift between the Rust type and the JSON. `packet`/`session` schemas need their source-of-truth named or marked hand-authored. |
| V13 | `crates/tui/.claude/` + `crates/tui/openspec/` nested inside a crate | **UNKNOWN→DRIFT-risk** | repo-local | crate dirs should hold Rust/`src`/`tests`/assets; tool config (`.claude/skills`) at crate depth fragments the control plane | governance axis owns the content; filesystem-wise, nesting an agent-tooling dir inside a library crate mixes semantics (tool-config in a source crate). Flag for governance cross-check, not an FHS violation. |
| V14 | root tool configs (`qodana.yaml`,`renovate.json`,`commitlint.config.cjs`) | **OK** | repo-local | root | tool-conventional root placement; each has an owning tool. |
| V15 | `.cargo/audit.toml`, `.githooks/`, `.github/`, `Makefile` | **OK** | Rust-Cargo + repo-local | root | standard repo-local tooling surfaces. |

---

## 3. Boundary map (system / user / meta / repo-local)

```
SYSTEM-level (/usr,/etc,/var, systemd) ............ NONE. No installs/writes. OK — kernel is local-first.

USER-level ($HOME, $XDG_*) ........................ LEAK: .idea/ (13 JetBrains files) = one dev's
                                                    user/IDE state committed INTO the repo-local kernel
                                                    root [V7]. Wrong direction (user→repo).

META-level ($META_ROOT) ........................... CROSS-BOUNDARY DEP: hf+ledger reach UP to
                                                    $META_ROOT/RuVector via ../../RuVector/* [V1,V2].
                                                    The north-star says handoff is the portable ROOT,
                                                    but two kernel crates depend on a META-level sibling
                                                    => handoff is NOT standalone-portable. This is THE
                                                    residency contradiction.
                                                    (RuVector itself: $META_ROOT/RuVector -> meta-ruvector
                                                    symlink; absent in the worktree sibling.)

REPO-LOCAL (handoff/) ............................. The intended home for everything the kernel needs.
   OK: crates/* + 16 handoff-* + hf + ledger + work-order (source),
       vendor/syntect (vendored toolchain, travels with repo),
       .handoff/.kb (durable text committed, binary ignored),
       docs/ scripts/ .githooks/ .github/ .cargo/.
   DRIFT (clutter, no owner/route): .agent/skills-catalog.md [V8],
       intent-driven-template/ [V9], spike/ruvocal-mcp-bridge/ (Node) [V10],
       schemas/*.json generated-at-root unmarked [V12].
   DUPLICATE-LINEAGE: crates/{cli,core,runner,spec,tui} forked vs rusty-idd [V3/§4].
```

**Boundary verdict:** the kernel is clean at the SYSTEM boundary (local-first, no global installs — passes the envctl-meta invariant for "no unmanaged system/user installs"). It FAILS the **portable-root** mandate on two counts: (1) a META→sibling upward path-dep [V1/V2], and (2) USER→repo IDE leakage [V7]. Root clutter [V8/V9/V10/V12] further dilutes "clean portable kernel root".

---

## 4. Union residency — target crate layout (if rusty-idd MERGES into handoff)

**Recommendation (consistent with union-handoff-rusty-idd.md Option A): MERGE, handoff is north-star; DEDUP the 5 shared crates into ONE canonical set.** Both repos declare identical pkg names `rusty-idd-{cli,core,runner,spec,tui}`; a naive co-location would be a **Cargo workspace name collision**, so dedup is mandatory, not optional.

Target layout @ `$META_ROOT/handoff` (single workspace, single canonical crate set):

```
handoff/                              # the portable kernel ROOT (north-star)
├── Cargo.toml                        # one [workspace]; members = kernel(16) + shared(5) + restored ridd cmds
├── vendor/syntect/                   # vendored toolchain (workspace [patch], travels with repo) [V4 OK]
├── hf/  ledger/  work-order/  handoff-*/   # KERNEL (16) — unchanged
├── crates/                           # CANONICAL shared set (dedup winner = rusty-idd side, the superset)
│   ├── cli/    (rusty-idd-cli)       # + RESTORE stripped cmds: codex,deploy,harness,knowledge,
│   │                                 #   merge-tools,next,render,spec-plan-integration (union §2)
│   ├── core/   runner/  spec/  tui/  # reconcile to rusty-idd superset; KEEP handoff's HFTASK-0082
│   │                                 #   error-handling hardening (unwrap/expect/panic=deny)
│   └── (rusty-idd extras folded in: config, knowledge, merge-tools, external/{codegraph-*,repomix-shared})
├── docs/  schemas/  scripts/  .handoff/  .kb/   # repo-local surfaces
└── RuVector strategy applied here (see UPGRADE U1) so the root is truly standalone
```

**Dedup decision per crate (which side wins; effort from codemap §4 divergence):**

| crate | canonical winner | effort | rationale |
|---|---|---|---|
| `tui` | rusty-idd (≡ handoff, ~0.1% diff) | trivial reconcile | near-identical; keep handoff lint attrs. |
| `spec` | rusty-idd superset | cheap (~1.2%) | identical modulo HFTASK-0082 lint hardening → port the lint onto the winner. |
| `runner` | rusty-idd superset | moderate (~11%) | real per-file diff. |
| `core` | rusty-idd superset | substantial (~33%) | handoff is the stale partial fork; converge UP, re-apply hardening. |
| `cli` | rusty-idd superset | substantial (~40%) | rusty-idd has 8 extra command modules handoff stripped; restore on top of kernel. |

**Residency rule for the union:** because handoff is the north-star ROOT, the canonical crate set lives **in handoff** and rusty-idd's repo is retired or becomes a thin consumer; do NOT keep two `rusty-idd-*` workspaces (collision). The work-order seam becomes a **library dependency** (rusty-idd code links handoff's `work-order` + `handoff-schema::validate_card`) instead of the current **mirrored file copy** (`rusty-idd/crates/work-order/src/lib.rs:35` mirrors `schemas/task.schema.json`) — see UPGRADE U2.

---

## 5. UPGRADE rows (axis: filesystem-layout)

> Each carries target_surface, evidence, expected_location, migration_plan, acceptance, risk_tier (APPLY|PROPOSE|REGENERATE), reversibility. Planning is read-only; Feature Forge implements moves. No code mutated here.

```
UPGRADE[U1] axis: filesystem-layout  title: Make handoff standalone-portable — resolve RuVector ../../ path deps
  target_surface: ledger/Cargo.toml:16-20 ; hf/Cargo.toml:48,52,57
  evidence: rvf-{runtime,index,types,crypto} + ruvector-{verified,domain-expansion} + cognitum-gate-tilezero
            all `path = "../../RuVector/crates/..."`; sibling RuVector absent in worktree; north-star demands
            $META_ROOT+handoff be a portable kernel root [V1,V2 / boundary META-level].
  expected_location: a residency-stable RuVector source INSIDE or PINNED-TO the kernel root — one of:
            (a) vendor RuVector crates under handoff/vendor/ruvector/ + workspace [patch] (mirrors syntect V4);
            (b) publish the rvf-*/ruvector-* crates and depend by version; or
            (c) git dependency on FlexNetOS/meta-ruvector pinned by rev (no filesystem `../../`).
  migration_plan: choose strategy via ADR; for (a) `cargo vendor`-style copy of the 6 crates + add
            [patch.crates-io]/[patch] entries; flip path deps to vendored/published/git; keep CI's
            sibling-clone as a documented fallback only.
  acceptance: `cd handoff && cargo build --workspace` succeeds with NO sibling `../../RuVector` present
            (clone handoff alone into a scratch dir, no RuVector beside it, build green).
  risk_tier: PROPOSE  (architecture decision; affects witness-chain crypto + formal-verification deps)
  reversibility: HIGH — Cargo dep-source swap; revert restores path deps. Vendored copy is additive.
```
```
UPGRADE[U2] axis: filesystem-layout  title: Dedup rusty-idd-* crates into ONE canonical set (union residency)
  target_surface: crates/{cli,core,runner,spec,tui}/Cargo.toml (pkg rusty-idd-*) vs rusty-idd repo
  evidence: identical pkg names in two workspaces (collision on co-location); handoff side is the stale
            partial fork (codemap §4: cli~40%/core~33%/runner~11% diverged; tui/spec near-identical).
  expected_location: single canonical set under handoff/crates/ (rusty-idd superset as winner), with
            handoff's HFTASK-0082 lint hardening re-applied; rusty-idd's 8 stripped CLI cmds restored.
  migration_plan: per-crate diff/merge per the §4 table (trivial→tui/spec, substantial→core/cli);
            converge to rusty-idd superset, port lint attrs, restore commands, fold rusty-idd extras
            (config/knowledge/merge-tools/external/*); retire the duplicate rusty-idd workspace.
  acceptance: one workspace builds with no duplicate `rusty-idd-*` package names; `cargo metadata`
            shows each pkg once; differential golden tests (spec/tui parity) pass.
  risk_tier: PROPOSE  (cross-repo structural merge; per-crate reconciliation)
  reversibility: MEDIUM — git history preserves both forks; revert re-splits. core/cli merges are the risk.
```
```
UPGRADE[U3] axis: filesystem-layout  title: Un-track .idea/ user-IDE state from the kernel root
  target_surface: .idea/ (13 files: *.iml, runConfigurations/, deployment.xml, material_theme_*, git_toolbox_*)
  evidence: JetBrains user/IDE config committed into a repo-local portable kernel [V7]; user→repo leak.
  expected_location: developer's $XDG_CONFIG_HOME / global gitignore; NOT committed.
  migration_plan: `git rm -r --cached .idea` + add `/.idea/` to .gitignore (optionally keep shared
            runConfigurations under a documented `.idea/runConfigurations/` allow if the team wants them).
  acceptance: `git ls-files .idea` → empty (or only the explicitly-allowed shared run configs); root tree
            no longer carries one machine's deployment/theme config.
  risk_tier: APPLY  (additive ignore + cache-removal; no source change)
  reversibility: HIGH — files remain in history; un-ignore to restore.
```
```
UPGRADE[U4] axis: filesystem-layout  title: Route the generated .agent/skills-catalog.md off the committed root
  target_surface: .agent/skills-catalog.md (313K generated blob)
  evidence: large derived catalog committed at root with no regen/owner doc [V8]; same derived class as
            .handoff/maps (correctly ignored).
  expected_location: a generated/ or ignored surface, regenerated on demand by its producing tool.
  migration_plan: identify the generator; either (a) ignore it + document `make skills-catalog`, or
            (b) move under a `generated/` dir with a regen gate; record the producer in docs.
  acceptance: root no longer churns a 313K derived blob; a doctor check regenerates it deterministically.
  risk_tier: REGENERATE  (derived artifact; rebuild from source of truth)
  reversibility: HIGH — regenerate at any time.
```
```
UPGRADE[U5] axis: filesystem-layout  title: Home or remove the root orphans (intent-driven-template/, spike/)
  target_surface: intent-driven-template/openspec/schemas/intent-driven/schema.yaml ; spike/ruvocal-mcp-bridge/
  evidence: non-member subtrees at a pure-Rust kernel root; intent-driven-template = 1-file orphan [V9];
            spike = a Node.js experiment at a no-C kernel root [V10].
  expected_location: intent-driven-template → docs/templates/ (or a registered asset dir);
            spike → a `spikes/` doc-surface, a separate repo, or removal once captured in docs.
  migration_plan: move the template under docs/templates/ with an owner note; for the spike, either
            extract to its own repo/registry entry or delete after recording its finding in docs/.
  acceptance: repo root contains only owned, routed surfaces; no non-member ecosystem-foreign tree at root.
  risk_tier: PROPOSE  (placement/ownership decision)
  reversibility: HIGH — git mv; history retained.
```
```
UPGRADE[U6] axis: filesystem-layout  title: Mark schemas/*.schema.json provenance (generated vs authored)
  target_surface: schemas/{task,packet,session}.schema.json
  evidence: task.schema.json title:"WorkOrder" is generated from work-order::WorkOrder schemars
            (codemap §5.1); committed at root with no generated-marker/regen gate [V12].
  expected_location: schemas/ is acceptable, but task.schema.json must be marked GENERATED with a regen
            command; packet/session must name their source-of-truth (or be marked hand-authored).
  migration_plan: add a header/sidecar declaring source + `make schemas` (regen task.schema.json from
            work-order); a doctor/golden check fails if the committed JSON drifts from the Rust type.
  acceptance: regenerating task.schema.json yields byte-identical output (golden); packet/session
            provenance documented.
  risk_tier: REGENERATE
  reversibility: HIGH.
```

---

## 6. Feature-Forge enforcement handoff (make drift FAIL in CI)

Exact checks to add so each verdict cannot recur. (Planning specifies; Feature Forge implements + the only permitted planning mutation is additive RED tests.)

| ID | type | check | guards verdict |
|---|---|---|---|
| E1 | gate (CI job) | **Standalone-build gate:** in a clean container, `git clone handoff` into a dir with NO sibling `RuVector/`, then `cargo build --workspace`. MUST pass (after U1). Fails today → proves the residency bug. | V1,V2,U1 |
| E2 | doctor / unit | **No `../../` path-dep doctor:** parse every member `Cargo.toml`; assert no `path = "../../..."` escaping the repo root (`git rev-parse --show-toplevel`). Fail-closed. | V1,V2 |
| E3 | gate | **No duplicate pkg-name gate:** `cargo metadata` → assert each package name unique across the workspace; assert no two crates both named `rusty-idd-*` after union. | V3,U2 |
| E4 | golden | **Schema-from-type golden:** regenerate `schemas/task.schema.json` from `work-order::WorkOrder` and diff against the committed file; mismatch fails. | V12,U6 |
| E5 | doctor | **Root-clutter gate:** allowlist of permitted top-level entries; any new untracked-into-an-owned-surface root dir/file (e.g. a new `intent-driven-template`-style orphan, an `.idea/`, a foreign-ecosystem `spike/`) fails until routed + owner-documented. | V7,V8,V9,V10 |
| E6 | doctor | **Generated-artifact-not-committed gate:** assert `.agent/skills-catalog.md` (and `.handoff/maps/`, `*/ledger.db`) are either ignored or carry a regen-marker; a committed unmarked generated blob fails. | V8,U4 |
| E7 | unit | **.gitignore residency invariant test:** assert `/_workspace/`, `/_workspace_prev/`, `.handoff/**/ledger.db`, `.kb/.cache/` remain ignored AND `.handoff/ledger.events.jsonl` + `.kb/store/**` remain tracked (the text-vs-binary contract). | V5,V6,V11 |
| E8 | gate | **No-C / pure-Rust root gate:** fail CI if a non-Rust package manifest (`package.json`, etc.) appears outside an explicitly-allowed spike surface — keeps the kernel root pure-Rust. | V10 |

---

## 7. Confidence + honesty notes

- **Confidence: HIGH** on V1/V2 (RuVector residency break — direct Cargo evidence, sibling absent in worktree), V3/§4 (dedup — corroborated by codemap + union), V7 (`.idea/` tracked — `git ls-files`), the `_workspace` correction (0 tracked), and the `vendor/syntect`-at-root correction (root `[patch]`, no `crates/tui/vendor`).
- **CORRECTED two inbound facts** (not silently accepted): `_workspace*` are ignored, not committed [V11]; syntect is at root `vendor/`, not `crates/tui/vendor/` [§0].
- **N/A — system-level installs:** none exist; the kernel is local-first with no `/usr`,`/etc`,`/var`,systemd writes, so the envctl "no unmanaged system install" invariant passes on the system boundary (the failures are at the META-portability and USER-leak boundaries, not system).
- **V13 (`crates/tui/.claude`+`openspec` nesting)** is flagged DRIFT-risk and routed to the governance axis for content ownership; filesystem verdict is "mixed semantics inside a source crate", not an FHS breach — kept QUALIFIED, not asserted.
- Read-only honored: no files moved, no production code touched. Only this findings doc written.
