# Autoresearch audit — handoff (cycle 2)

Target: **handoff** — the continuity / witnessed-ledger kernel (`hf` + the `.handoff` ledger),
planned as the **UNION with rusty-idd**.
Auditor: plan-autoresearch-loop-auditor. Date: **2026-06-26**. Cycle: 2.
Scope per `…/envctl/.claude/skills/plan-autoresearch-loop/SKILL.md`: assess constant
**code auto-research** (git-kb indexing — handoff HAS `.kb/`; `handoff-index`; `handoff-drift` as the
drift/invalidation engine) and **web auto-research** (advisory/currency refresh; CI `cargo audit`),
plus how **stale** ledger/index state is **invalidated**. Read-only on target code; cites only.

Code (read-only): `/home/drdave/Desktop/meta/.worktrees/plan-handoff-cycle2/handoff` @
`f6abf962413bafe164d56fa26b70b0a5fdacb8a2`.
Context read: `reports/codemap-handoff.md`, `research/handoff.trends.md`,
`research/sources-handoff.jsonl`, graph snapshot `graph/handoff.{symbols,callgraph,metrics,graph,diff}.*`.

## Verdict (honest answer)

handoff is a **pure-Rust kernel + CLI** (`hf`), so **constant *runtime* auto-research as a resident
daemon is N/A — there is no long-lived process inside handoff that continuously re-indexes code or
polls the web.** But the capability is **NOT absent**; it exists in three real, distinct, **pull /
event-driven** layers, and (unlike rusty-idd, cycle-1) handoff additionally ships a *first-class
drift-detection engine* that is the explicit invalidation mechanism:

1. **handoff researches its own code via git-kb** — it now has a full local `.kb/` (HFTASK-0072), and
   the `hf kb` seam (`hf/src/kb.rs`, ADR-0003) drives the **git-kb** code/planning plane as a
   subprocess. `handoff-index` (`hf index`/`hf plan`) re-derives repo/test/owner/dependency maps + the
   task DAG from REAL kernel data each run (never fabricated). This is **code auto-research**, on demand.
2. **handoff does web auto-research in CI** — `rustsec/audit-check@v2` (the **cargo audit** action) runs
   on the develop→main promotion gate, fetching the RustSec advisory DB (web) each run;
   `.cargo/audit.toml` is a strict **`ignore = []`** (upgrade-only, never suppress) register;
   `renovate.json` opens dependency-currency PRs. The loop's own **90-day recency** trend note
   (`research/handoff.trends.md`) is the human web-research layer on top.
3. **Stale-state invalidation is a kernel feature, not just a CI diff** — `handoff-drift` / `hf drift`
   (HFTASK-0046/0005, schema `handoff.drift_report.v1`) computes a 5-surface intent_lock and
   **invalidates** cards whose objective/path_scope/acceptance/constraint/**northstar** drifted, plus a
   `handoff_state_stale` advisory; it is **fail-closed** (exit 1 → `PreHandoff` hook `fail_mode=block`)
   and runs in CI. Separately, a concrete **stale-index incident** was caught this cycle: the legacy
   `.git/gitkb/code.db` (pre-peel `develop` snapshot, missing every `handoff-*` crate) was detected and
   **discarded**, the live `.kb/.cache/gitkb.db` re-indexed `--force`.

So: continuous/daemon auto-research = **N/A — kernel/CLI, no resident process**; pull/event-mode code +
web auto-research with **fail-closed** drift invalidation = **PRESENT and verifiable**. Genuine gaps:
the advisory web gate fires only on **promotion**, not every PR (asymmetric cadence), and git-kb index
freshness has **no automated staleness gate** — the stale `code.db` was caught by human cartography, not
by a check (see UPGRADE rows U1/U3).

---

## 1. Code auto-research (git-kb + index/maps)

| # | CLAIM | Evidence |
|---|-------|----------|
| C1 | **handoff drives `git-kb` directly — the `hf kb` seam (ADR-0003).** `hf/src/kb.rs` runs `git-kb` as a subprocess (`Command::new("git-kb")…`, `run_kb_in`) to mint handoff cards FROM git-kb tasks — *"the planning plane (git-kb) feeds the execution plane (.handoff). Read-only on the kb."* One-way by construction. | `hf/src/kb.rs:1-4` (doc), `:33` (`git-kb` subprocess), `:166` `cmd_mint_from_kb`, `:176`/`:271`/`:281` (`git-kb show/commit`) |
| C2 | **handoff has its OWN full local `.kb/` (git-kb code intelligence), HFTASK-0072.** `kb_root` resolves the repo's own `.kb/` FIRST, falling back to the meta-root `.kb/`. The `.kb/AGENTS.md` declares `context_source: gitkb`, MCP-primary / git-kb-CLI-fallback. Live store: `.kb/.cache/gitkb.db`. | `hf/src/kb.rs:11-29` (`kb_root`, "handoff now has a full local `.kb`"); `.kb/AGENTS.md` (`context_source: gitkb`, `git-kb list/checkout/board`); `.kb/.cache/gitkb.db{,-wal,-shm}` present |
| C3 | **`handoff-index` is on-demand code auto-research: `hf index`/`hf plan` re-derive repo nav maps + the task DAG.** `cmd_index` writes `.handoff/maps/{repo,test,owner,dependency}-map.json` + a nav README; `cmd_plan` builds the topological task DAG. *"Every map is derived from REAL data the kernel already holds … never fabricated."* | `handoff-index/src/lib.rs:11-17` (doc), `:76`/`:93`/`:104`/`:139`/`:225` `build_*`, `:256` `fs::write`, `:261` `cmd_index` |
| C4 | **It is ONE-SHOT (pull), not continuous.** `hf kb`, `hf index`, `hf plan` run only when the verb is invoked or a hook fires them; there is no `notify`/watcher/resident loop inside handoff that re-indexes on file save. The meta-level git-kb daemon (`meta/.claude/rules/code-intelligence.md`) is what watches; handoff itself calls git-kb pull-mode. | absence of `notify::`/`Watcher`/`spawn`-watch in `hf/src/kb.rs` + `handoff-index/src/lib.rs`; the only `git-kb` calls are blocking subprocess captures (`hf/src/kb.rs:33`) |
| C5 | **The git-kb code graph IS snapshotted + diffable (the "graph updates" cadence) — external to the repo.** This cycle's graph lives under the plan dir (not in the target repo): symbols/callgraph/metrics + a diff note. Cycle-2 is the **baseline** handoff snapshot (no prior `handoff.*` to diff). | `graph/handoff.{symbols,callgraph,metrics}.json`, `graph/handoff.graph.md:4-5`, `graph/handoff.diff.md:3-7` ("No prior snapshot — baseline established this run") |

**Code-research summary:** handoff *consumes* git-kb (the `hf kb` seam + its own `.kb/`) and *re-derives*
repo intelligence on demand (`handoff-index`); the loop *snapshots* the git-kb graph each cycle. None of
these is a resident daemon inside handoff — code auto-research is pull/hook/CI-driven.

## 2. Web auto-research (advisory + currency)

| # | CLAIM | Evidence |
|---|-------|----------|
| C6 | **Advisory web auto-research IS automated in CI** — the `audit` job runs `rustsec/audit-check@v2` (the `cargo audit` action), which fetches the RustSec advisory DB (web) each run and fails on a new advisory. | `.github/workflows/promote-verify.yml:203-211` (`audit:` → `uses: rustsec/audit-check@v2`); header `:4` ("…and cargo audit --deny warnings") |
| C7 | **The audit register is strict, upgrade-only, fail-closed: `ignore = []`.** `.cargo/audit.toml` ignores NOTHING; the two unmaintained advisories it hit (RUSTSEC-2024-0320 yaml-rust, RUSTSEC-2025-0141 bincode) were **FIXED** upgrade-only via the vendored syntect fork (→ `yaml-rust2`/`postcard`), not suppressed. "If either reappears … `cargo audit` SHOULD fail — do not add an `ignore`." | `.cargo/audit.toml` (verbatim: `[advisories] ignore = []` + rationale) |
| C8 | **Dependency-currency web auto-research is automated** via **Renovate** (`config:recommended`) — opens currency PRs. (handoff uses Renovate; rusty-idd cycle-1 used Dependabot — fleet inconsistency noted.) | `renovate.json` (verbatim) |
| C9 | **The advisory gate is ASYMMETRIC: it runs on the promotion gate, NOT on every PR/push.** `cargo audit`/RustSec appears only in `promote-verify.yml` (develop→main); `ci.yml` (the per-PR gate) has **no** `audit`/`deny`/`rustsec` step. A new advisory therefore surfaces only at promotion, not at PR time. | `grep 'audit\|deny\|advisor\|rustsec' .github/workflows/ci.yml` → 0 matches; only `promote-verify.yml:203-211` |
| C10 | **The loop's web auto-research is the 90-day recency trend note**, every load-bearing claim URL-and-date stamped, window **2026-03-28 → 2026-06-26**, in-window/older flagged, backed by a machine ledger (`sources-handoff.jsonl`). Tool pins verified current: redb 4.1.0, blake3 1.8.5, ed25519-dalek 2.2.0 / curve25519-dalek 4.1.3 (past all signing advisories). | `research/handoff.trends.md` §A,§E; `research/sources-handoff.jsonl` |
| C11 | **The one C-dependency residual is governed, advisory-clear, and feature-gated** — rusqlite 0.31.0 / libsqlite3-sys 0.28.0 are `legacy-sqlite`-only (the C-SQLite→redb `hf migrate` importer), never in the default no-C build, all pins past their advisories. Currency lag is low-priority because gated behind the trust boundary. | `research/handoff.trends.md` §A4; target `ledger/Cargo.toml:23,37` |

## 3. Cadence & stale-evidence invalidation

| # | CLAIM | Evidence |
|---|-------|----------|
| C12 | **`handoff-drift` IS the kernel's invalidation engine — it detects and BLOCKS on stale intent.** `cmd_drift` computes a 5-surface intent_lock (objective/path_scope/acceptance/constraint/**northstar**) and emits `handoff.drift_report.v1`; on any drift it **exits 1** so the `PreHandoff` (`fail_mode=block`) hook stops the loop. "Fail-closed." | `handoff-drift/src/lib.rs:602` `cmd_drift`, `:675-677` `process::exit(1)` ("hard-fail so PreHandoff … stops"), `:10-11` doc ("Fail-closed") |
| C13 | **The `northstar` surface is doctrine-revision drift = explicit staleness invalidation.** A card minted against a **superseded** North-Star revision is flagged: *"northstar drift: {} — minted against a superseded doctrine revision (re-mint)"*; checked against `handoff_core::current_northstar_revision()`. This invalidates intent that has gone stale vs the current doctrine. | `handoff-drift/src/lib.rs:370` (`current_northstar_revision`), `:408-417` (northstar drift branch + `required_actions` "re-mint against the current North Star") |
| C14 | **`handoff_state_stale` is a dedicated stale-state advisory (PRD §12.3 #10):** an active task with material (changed) files but **no witnessed checkpoint** — "handoff state not refreshed after material changes." Advisory (not blocking), surfaced as a reminder. | `handoff-drift/src/lib.rs:314-317` (field doc), `:623` (JSON `handoff_state_stale`), `:645-650` (human advisory) |
| C15 | **The drift gate is wired fail-closed into the lifecycle hooks AND CI.** `.handoff/hooks/hooks.toml` fires `hf drift --json && hf policy check-handoff --json` on `PreHandoff` with `fail_mode = "block"`; `promote-verify.yml` runs `./target/debug/hf drift` as a CI step. | `.handoff/hooks/hooks.toml` (PreHandoff block); `promote-verify.yml:199-201` ("Run hf drift") |
| C16 | **A REAL stale-index incident was caught + invalidated this cycle.** The legacy `.git/gitkb/code.db` held a **stale pre-peel `develop` snapshot (3412 symbols, missing ALL `handoff-*` crates)** and was **detected and NOT used**; the live `.kb/.cache/gitkb.db` (branch `feat/hftask-0072-full-kb-adoption`) was re-indexed `--force`, confirmed via `git-kb code doctor --json` showing `handoff-*` in `file_breakdown` before any metric was derived. | `graph/handoff.diff.md:35-37` (integrity note); `graph/handoff.graph.md:5-6` ("stale `.git/gitkb/code.db` … was detected and NOT used"); the stale `code.db` is physically present at `/home/drdave/Desktop/meta/handoff/.git/gitkb/code.db` (28 MB), the live store at `…/handoff/.kb/.cache/gitkb.db` |
| C17 | **`hf doctor` is the broader kernel staleness/health sweep — and is being hardened toward fail-closed.** `cmd_doctor` (HFTASK-0049) verifies witness chain + ledger replay + residency; the roadmap explicitly extends it to a fail-closed invariant sweep + **stale-lock self-heal** (detect a provably-dead `*.rvf.lock` and reclaim it, refusing live/unverifiable holders). NOTE: this `hf doctor` is distinct from `git-kb code doctor` (C16). | `hf/src/main.rs:439-443`/`:519` `cmd_doctor`; `hf/src/main.rs:2929-2931` (roadmap: "hf doctor fail-closed invariant sweep + stale-lock self-heal") |
| C18 | **No per-cycle automatic refresh binds the layers; cadence is event-driven, not constant.** The git-kb graph + trend note refresh only when the planning loop runs; `hf drift`/`hf index` fire on hooks/CLI; the advisory gate fires on promotion. No scheduled/cron job re-snapshots the graph or re-runs trend research on a timer. (Consistent with N/A-as-runtime-feature.) | absence of cron/`schedule:` for research in `.github/workflows/`; C9 (audit on promote only); trend note is a per-cycle artifact |

## 4. Upgrade rows (`axis: autoresearch`)

| id | upgrade | evidence | acceptance | risk | reversibility |
|----|---------|----------|------------|------|---------------|
| U1 | **Add an automated git-kb index-staleness gate.** The stale `.git/gitkb/code.db` (C16) was caught by human cartography, not a check. Add a `hf doctor` (or CI) assertion that the active git-kb store's `code doctor --json` `file_breakdown` includes every current workspace crate (e.g. all `handoff-*`), failing closed on a pre-peel/partial index. | C16, C17 | `hf doctor` DEGRADED + exit 1 when the index is missing a workspace crate | low (additive check) | drop the assertion |
| U2 | **Make the advisory web gate symmetric — run `cargo audit` on every PR, not only promotion.** Today a new RustSec advisory surfaces only at develop→main (C9); add the `rustsec/audit-check` job to `ci.yml` so per-PR builds also fetch + enforce the advisory DB. | C6, C9 | a new advisory fails the per-PR `ci.yml`, not just `promote-verify.yml` | low | remove the CI job |
| U3 | **Add a scheduled cadence for the loop's web/code research.** A timer/`schedule` re-run of the 90-day trend note + git-kb graph snapshot so recency invalidation is constant, not only when the loop happens to run (C18). | C18 | trend note `Date:` advances on schedule; out-of-window rows auto-flagged; graph diff produced each tick | low | remove the schedule |
| U4 | **Align fleet dependency-currency tooling.** handoff uses **Renovate** (C8) while rusty-idd cycle-1 uses **Dependabot** — pick one for the union to avoid duplicate/conflicting currency PRs across the converging control plane. | C8; cycle-1 `findings/autoresearch-rusty-idd.md` C7 | one currency bot governs both converged repos | low | revert config |
| U5 | **Delete the last C dependency once fleet legacy ledgers are migrated.** Remove the `legacy-sqlite` feature + rusqlite/libsqlite3-sys (C11) to eliminate the only C dep and the only feature-gated advisory residual — a strict no-C upgrade. | C11; trends §A4 | `cargo tree -i rusqlite` empty in all feature sets | med (needs all fleet ledgers migrated first) | re-add feature |

## 5. Gate handoff — tests/gates that prove missing stale-evidence checks fail closed

- **Intent/doctrine staleness — ALREADY fail-closed:** `hf drift` exits 1 → `PreHandoff` `fail_mode=block`
  (C12/C13/C15). RED test for the northstar surface: mint a card, bump
  `current_northstar_revision()`, assert `hf drift --json` reports `northstar_revision_match:false` and
  exits non-zero (proves a superseded-doctrine card is **invalidated**, not silently accepted).
- **git-kb index staleness — NOT YET gated (the U1 gap):** the stale `.git/gitkb/code.db` (C16) had no
  automated guard. RED test: stand up a partial/pre-peel index missing a workspace crate, run the
  proposed `hf doctor` index check, assert DEGRADED + exit 1 (currently no check binds index freshness —
  that is the gap the RED test pins).
- **Advisory staleness — fail-closed only at promotion (the U2 gap):** `rustsec/audit-check@v2`
  (`ignore = []`, C6/C7) blocks promotion. RED test for U2: re-introduce a known unmaintained crate
  (e.g. `bincode`/`yaml-rust`) into the tree and assert the **per-PR** `ci.yml` fails (today it passes —
  a per-PR false-negative until the audit job is added there).
- **Recency invalidation — loop-enforced, not CI-enforced:** the 90-day window lives in the trend note
  (C10), not a gate. RED test for U3: assert a trend row whose `published_at` is >90 days before the note
  `Date:` is flagged `in_recency_window:false` (the ledger already does this — codify so a *stale,
  unflagged* row fails closed).

---

### Markers (gate)
- **code auto-research** / **git-kb**: §1 (C1-C5) — handoff drives git-kb (`hf kb` seam, its own `.kb/`,
  `handoff-index` maps).
- **web auto-research** / **90-day** / **recency**: §2 (C6-C11), §3 C10/C18.
- **stale** / **invalidate**: §3 (C12-C18) — `handoff-drift` is the invalidation engine; C16 = the real
  stale `code.db` incident, discarded; §5 RED tests.

### Sources / paths (all absolute)
- Findings artifact: `/home/drdave/Desktop/meta/.worktrees/plan-fleet-convergence/envctl/.handoff/loop/plan/findings/autoresearch-handoff.md`
- Code (read-only) `/home/drdave/Desktop/meta/.worktrees/plan-handoff-cycle2/handoff/` @ `f6abf962413bafe164d56fa26b70b0a5fdacb8a2`:
  `hf/src/kb.rs`, `hf/src/main.rs` (`cmd_doctor`, roadmap), `handoff-index/src/lib.rs`,
  `handoff-drift/src/lib.rs`, `.kb/AGENTS.md`, `.kb/.cache/gitkb.db`, `.handoff/hooks/hooks.toml`,
  `.cargo/audit.toml`, `renovate.json`, `.github/workflows/{ci.yml,promote-verify.yml}`, `ledger/Cargo.toml`
- Stale index (real incident, physical): `/home/drdave/Desktop/meta/handoff/.git/gitkb/code.db` (stale, discarded)
- Plan context: `/home/drdave/Desktop/meta/.worktrees/plan-fleet-convergence/envctl/.handoff/loop/plan/` —
  `reports/codemap-handoff.md`, `research/handoff.trends.md`, `research/sources-handoff.jsonl`,
  `graph/handoff.{graph,diff,symbols,callgraph,metrics}.{md,json}`
