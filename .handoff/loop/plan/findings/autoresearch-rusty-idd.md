# Autoresearch audit — rusty-idd

Target: **rusty-idd** (intent-driven control-plane organ / fabric AXIS of the `meta` fleet).
Auditor: plan-autoresearch-loop-auditor. Date: **2026-06-26**.
Scope per `.agents/skills/plan-autoresearch-loop/SKILL.md`: assess constant **code auto-research**
(git-kb / continuous symbol refresh) and **web auto-research** (90-day recency / advisory refresh),
and how findings are **invalidated when stale**. Read-only on target code; cites only.

Context read: `reports/codemap-rusty-idd.md`, `research/rusty-idd.trends.md`,
`research/sources-rusty-idd.jsonl`, graph snapshot `graph/rusty-idd.{md,json}`.

## Verdict (honest answer)

rusty-idd is a **CLI control plane**, so "constant auto-research" as a *live runtime daemon feature*
is **N/A** — there is no resident process that continuously re-indexes code or polls the web. But the
capability is **NOT absent**: it exists in two real, distinct layers, both **command/CI-driven (pull)
rather than continuous (push)**:

1. **The loop researches rusty-idd** (the convergence-target framing). The plan loop's **code
   auto-research** runs `git-kb code` against rusty-idd and snapshots it to
   `graph/rusty-idd.{json,md}`; its **web auto-research** is the 90-day trend/advisory note
   (`research/rusty-idd.trends.md`). This is external to the product and is where the **serde_yaml
   deprecation** the prompt names was actually caught.
2. **rusty-idd does its own code auto-research** — the **`knowledge` crate** is a product-level
   code indexer (`index_workspace`/`refresh_workspace`/`load_index`, exposed as `rusty-idd knowledge`),
   and the repo carries real **web auto-research automation in CI** (`cargo audit --deny warnings`
   pulls the RustSec DB every run; `dependabot.yml` opens weekly currency PRs). Stale-evidence
   invalidation is enforced by two **fail-closed** CI gates (IDD manifest-refresh diff; advisory
   baseline with re-evaluation triggers).

So: continuous/daemon auto-research = **N/A — CLI, no resident process**; pull-mode code + web
auto-research with fail-closed staleness gates = **PRESENT and verifiable**. The one genuine gap:
the residual deprecated `serde_yaml` is invisible to the repo's own web gate (`cargo audit`) and is
caught only by the loop's human trend research — see CLAIM-7 / UPGRADE rows.

---

## 1. Code auto-research

| # | CLAIM | Evidence |
|---|-------|----------|
| C1 | **No continuous symbol refresh / no `git-kb` in product code.** rusty-idd has zero `git kb`/`git-kb`/`gitkb` references in `crates/`. `git-kb` is the meta-level FlexNetOS daemon (`meta/.claude/rules/code-intelligence.md`), used *by the loop on* rusty-idd, not *by* rusty-idd. | `grep -rln 'git kb\|git-kb\|gitkb' crates/` → empty |
| C2 | **The loop's code graph IS snapshotted + diffable** (the "graph updates" cadence). The git-kb-derived graph lives at the plan dir, not in the repo. | `.handoff/loop/plan/graph/rusty-idd.json`, `…/graph/rusty-idd.md` exist; codemap §header cites `graph/rusty-idd.{md,json}`. No `graph/rusty-idd.*` inside the repo (`ls graph/` → absent). |
| C3 | **rusty-idd ships its OWN product code-research engine** — the `knowledge` crate: `index_workspace` (`crates/knowledge/src/lib.rs:851`), `refresh_workspace` (`:1221`), `load_index` (`:1256`), `query_knowledge_index`. Surfaced via `rusty-idd knowledge` (`crates/cli/src/commands/knowledge.rs:9-10,245-260`); writes `.idd/knowledge/index.json` + report.md. | cited lines |
| C4 | **That engine is ONE-SHOT (pull), not continuous.** It parses via vendored codegraph (`parse_source_file_with_codegraph`, `lib.rs:1280`; uses `codegraph_parser::language::LanguageRegistry` + `languages::extract_for_language`, `lib.rs:11-12`). It uses **no** `Watcher`/`notify`/`spawn`/`IncrementalUpdater` — runs only when the `index`/`refresh` command is invoked. | `grep 'Watcher\|notify::\|IncrementalUpdater\|spawn' crates/knowledge/src/lib.rs` → empty |
| C5 | **Continuous-watch infra exists but is vendored-dead for rusty-idd.** codegraph-core ships `src/watch/`, `src/incremental/{mod,updater}.rs`, `notify = "8"` (`external/codegraph-core/Cargo.toml:29`) and codegraph-parser ships `src/watcher.rs` — but `knowledge` imports only the static parse/extract path (C4). The push-mode capability is present in-tree, unwired. | dir listings; `Cargo.toml:29`; C4 |

**Code-research summary:** rusty-idd is researched by the loop (git-kb → snapshot/diff) and self-indexes
on demand (`knowledge`); neither is a continuous daemon. The unwired codegraph watch/incremental
modules are a latent upgrade path (turn one-shot into watch) — but adding a resident watcher to a CLI
is a deliberate architecture decision, not a free win.

## 2. Web auto-research

| # | CLAIM | Evidence |
|---|-------|----------|
| C6 | **Advisory web auto-research IS automated in CI** — `cargo audit --deny warnings` runs in both the develop gate (`.github/workflows/ci.yml:60-61`) and the develop→main promotion gate (`.github/workflows/promote-verify.yml:86-87`). Each run fetches the RustSec advisory DB (web), so new advisories surface automatically and fail the build. | cited lines |
| C7 | **Dependency-currency web auto-research is automated** — `.github/dependabot.yml` runs weekly `cargo` + `github-actions` update PRs (limit 5 each). CodeQL scanning via `.github/workflows/codeql.yml`. | `dependabot.yml` (verbatim); workflow listing |
| C8 | **The loop's web auto-research is the 90-day recency trend note**, every load-bearing claim URL-and-date stamped, recency window 2026-03-28→2026-06-26, in-window/older flagged; backed by a 27-row machine ledger. | `research/rusty-idd.trends.md` §A,§E,§G; `research/sources-rusty-idd.jsonl` (27 rows) |
| C9 | **The serde_yaml deprecation the prompt names is REAL + residual, and was caught by the loop, NOT by the CI gate.** Confirmed residual: `external/codegraph-core/Cargo.toml:40` (`serde_yaml = "0.9"`) and `imports/prompt_hub/Cargo.toml:27` + `prompthub/Cargo.toml:64`; keeps `serde_yaml 0.9.34+deprecated` in `Cargo.lock:3400`. First-party crates already migrated to **serde_norway** (`crates/spec/Cargo.toml:23-25` "NOT serde_yaml/serde_yml", `crates/runner/Cargo.toml:21-23`). | cited manifest lines; `Cargo.lock:3378,3400` |
| C10 | **serde_yaml is INVISIBLE to the repo's own web gate.** It is absent from `.cargo/audit.toml` ignore-list and from `docs/rusty-idd/security-advisories.md`, yet CI is green — because RustSec tracks it as **issue #2132 (not a published advisory)**, so `cargo audit` does not flag it. The repo's web auto-research therefore has a blind spot the loop's trend research covers. | `grep 'serde_yaml\|2132' .cargo/audit.toml docs/rusty-idd/security-advisories.md` → empty; trends §A4 cites advisory-db **issue** #2132 (ledger row `in_recency_window:false`) |
| C11 | **Advisory recency is corroborated current.** tokio 1.52.3 unaffected by 2026 advisories (RUSTSEC-2026-0057/0060 = legacy tokio 0.1); clap 4.6.1 / ratatui 0.30.2 / serde 1.0.228 current, no advisory. | `research/rusty-idd.trends.md` §A1-A3,A6; ledger rows (RustSec, accessed 2026-06-26) |

## 3. Cadence & stale-evidence invalidation

| # | CLAIM | Evidence |
|---|-------|----------|
| C12 | **Fail-closed staleness gate #1 (code index):** CI regenerates the IDD manifest and diffs it — `rusty-idd -- manifest --out .idd/MANIFEST.tsv` then `git diff --exit-code -- .idd/MANIFEST.tsv` (`ci.yml:56-59`). If the committed index is **stale** vs source, the gate **fails** → stale code-research is invalidated automatically. Same pattern available via `knowledge refresh_workspace` (C3). | `ci.yml:56-59` |
| C13 | **Fail-closed staleness gate #2 (advisories):** `cargo audit --deny warnings` fails on **any new** advisory; the **`.cargo/audit.toml`** baseline is a managed accepted-risk register with **explicit re-evaluation triggers**, and `docs/rusty-idd/security-advisories.md` records disposition per advisory. Stale exceptions are **removed when a forward fix lands** — e.g. RUSTSEC-2026-0009 (`time`→0.3.47) and RUSTSEC-2026-0186 (`memmap2`→0.9.11) were *remediated and deleted* from the baseline; only 2 unmaintained-only warnings (bincode/yaml-rust via syntect, no upgrade path) remain accepted. | `.cargo/audit.toml` (ignore-list + rationale); `docs/rusty-idd/security-advisories.md` Remediated/Accepted-risk tables |
| C14 | **The loop's web findings carry their own invalidation rule:** 90-day recency window + in-window/older-flagged tags + per-row `published_at`/`in_recency_window` in the ledger → out-of-window evidence is explicitly marked stale (e.g. serde_yaml #2132 row `in_recency_window:false`; OpenSpec 1.0 baseline flagged-older). | `research/rusty-idd.trends.md` §E; `sources-rusty-idd.jsonl` |
| C15 | **No per-cycle automatic refresh binding the two layers.** The loop's code graph + trend note are refreshed only when the planning loop runs; the repo's CI gates fire only on push/PR. There is no scheduled job that re-runs the trend research or re-snapshots the graph on a timer → cadence is **event-driven, not constant**. (Consistent with N/A-as-runtime-feature.) | absence: no cron/schedule in `.github/workflows/` for research; trend note is a per-cycle artifact |

## 4. Upgrade rows (`axis: autoresearch`)

| id | upgrade | evidence | acceptance | risk | reversibility |
|----|---------|----------|------------|------|---------------|
| U1 | **Close the serde_yaml blind spot in the repo's own web gate.** Either migrate `external/codegraph-core` + `imports/prompt_hub` off `serde_yaml`→`serde_norway`/`serde-yaml-ng`, OR add an explicit tracked entry (with re-eval trigger) to `docs/rusty-idd/security-advisories.md` so the residual is governed, not silent. | C9, C10; trends §A4-A5 | `serde_yaml` gone from `Cargo.lock`, OR a registered accepted-risk row exists | low (vendored/import crates) | revert manifest edit |
| U2 | **Promote serde_yaml#2132 to a gate-enforced check.** Add a `cargo audit` deny for the deprecated `serde_yaml` (or a `cargo deny` bans rule) so the repo's CI — not just the loop — invalidates it. Today only human trend research catches it. | C10, C13 | CI fails while `serde_yaml 0.9.34+deprecated` is in the lock | low | drop the rule |
| U3 | **Make code auto-research push-mode (optional).** Wire codegraph-core `watch/`+`incremental/` (already vendored, `notify=8`) into `knowledge refresh_workspace` for a `--watch` mode, keeping the one-shot default. | C4, C5 | `rusty-idd knowledge index --watch` re-indexes on file change | med (resident process in a CLI — needs ADR) | feature-gated; default stays one-shot |
| U4 | **Add a scheduled cadence for the loop's web research.** A timer/`schedule` re-run of the trend note + graph snapshot so recency invalidation is constant, not only when the loop happens to run. | C15 | trend note `Date:` advances on schedule; out-of-window rows auto-flagged | low | remove the schedule |

## 5. Gate handoff — tests/gates that prove missing stale-evidence checks fail closed

- **Code-index staleness — ALREADY fail-closed:** `ci.yml:56-59` (`git diff --exit-code` on regenerated
  `.idd/MANIFEST.tsv`). RED test for a *new* gate (U3): touch a source symbol, do **not** refresh the
  knowledge index, assert CI/`knowledge` reports drift → must fail. Currently no such check binds
  `.idd/knowledge/index.json` freshness (only MANIFEST.tsv is diffed) — that is the gap a RED test pins.
- **Advisory staleness — ALREADY fail-closed:** `cargo audit --deny warnings` (`ci.yml:60-61`,
  `promote-verify.yml:86-87`). RED test for U2: add `serde_yaml` deny → assert the build fails while the
  deprecated crate is in `Cargo.lock` (proves the current green build is a false-negative for #2132).
- **Recency invalidation — loop-enforced, not CI-enforced:** the 90-day window lives in the trend note
  (C14), not a gate. RED test for U4: assert a trend row whose `published_at` is >90 days before the
  note `Date:` is flagged `in_recency_window:false` (the ledger already does this — codify as a check so
  a *stale, unflagged* row fails closed).

---

### Markers (gate)
- code auto-research / **git-kb**: §1 (C1-C5) — git-kb is the loop's tool; `knowledge` is the product's.
- web auto-research / **90-day** / recency: §2 (C6-C11), §3 C14.
- **stale** / **invalidate**: §3 (C12-C15), §5.

### Sources / paths (all absolute)
- Findings artifact: `/home/drdave/Desktop/meta/.worktrees/plan-fleet-convergence/envctl/.handoff/loop/plan/findings/autoresearch-rusty-idd.md`
- Code (read-only): `/home/drdave/Desktop/meta/rusty-idd/` — `.github/workflows/ci.yml`, `.github/workflows/promote-verify.yml`, `.github/dependabot.yml`, `.cargo/audit.toml`, `docs/rusty-idd/security-advisories.md`, `crates/knowledge/src/lib.rs`, `crates/cli/src/commands/knowledge.rs`, `crates/external/codegraph-core/{Cargo.toml,src/watch,src/incremental}`, `crates/external/codegraph-parser/src/watcher.rs`, `crates/spec/Cargo.toml`, `crates/runner/Cargo.toml`, `imports/prompt_hub/Cargo.toml`, `Cargo.lock`
- Plan context: `/home/drdave/Desktop/meta/.worktrees/plan-fleet-convergence/envctl/.handoff/loop/plan/reports/codemap-rusty-idd.md`, `…/research/rusty-idd.trends.md`, `…/research/sources-rusty-idd.jsonl`, `…/graph/rusty-idd.{json,md}`
