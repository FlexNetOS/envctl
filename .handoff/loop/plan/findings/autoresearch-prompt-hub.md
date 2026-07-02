# autoresearch — prompt-hub

Target `<T>` = prompt-hub (repo `/home/drdave/Desktop/meta/prompt_hub`).
Axis: `autoresearch` (constant code + web auto-research cadence).
Auditor: plan-autoresearch-loop-auditor. Read-only on target; this artifact is the only write.
Evidence date: 2026-06-27. All paths absolute.

Verdict up front: prompt-hub has a **real, running code auto-research loop** (`.handoff/loop/`
research-ledger + verified findings) plus a **real web/advisory cadence** (Dependabot daily +
`security.yml` daily `cargo audit`/`cargo deny`). The headline "rich" surface —
`multi-model-evaluation`, `external-ai-apis`, `ai-test-doc-generation` — is **opt-in scaffolding**,
not a closed loop: all 7 AI workflows gate on `vars.ENABLE_AI_WORKFLOWS == 'true'` (default unset →
skip green) and their output is uploaded as throwaway artifacts for manual review, never committed or
fed back. The genuine gaps are (a) the `git-kb` code graph has **0 call edges** (symbol-only, so no
real hotspot/blast-radius/dead-code intelligence) and is polluted by `vendor/`, and (b) there is **no
90-day recency-gated web research and no source ledger for web sources** — recency lives only in the
meta planning harness prompt, not in any prompt-hub artifact or CI.

---

## 1. Code auto-research

How the target's code facts are (re)gathered each cycle, via `git-kb` and the `.handoff/loop`.

| ID | CLAIM | Evidence | Verdict |
|----|-------|----------|---------|
| C-CODE-1 | The repo IS code-indexed by `git-kb` (`code auto-research` substrate present). | `git kb code stats` in repo: Symbols 1,429,131; Files 22,199; Last indexed `2026-06-27T05:50:51Z`; binary `/home/drdave/.local/bin/git-kb` v0.2.10. | CONFIRMED |
| C-CODE-2 | The `git-kb` call graph is NOT built — symbol-only index. | `git kb code stats` → `Call edges: 0`, `Unresolved calls: 0`. `git kb code callers search` returns matches all annotated `(no callers)`. `git kb code entrypoints` → "No entrypoints found. Run `git-kb code entrypoints --refresh`." | CONFIRMED |
| C-CODE-3 | Graph intelligence (hotspots / blast-radius / dead-code) is therefore unreliable. | With 0 call edges, `git kb code dead` flags trait-impl methods (`plugins/example_search_backend/src/lib.rs` `name`/`version`/`initialize`) as "no callers found" — false positives from missing edges, not true dead code. | CONFIRMED |
| C-CODE-4 | The index is polluted by vendored crates → prompt-hub's own signal is diluted. | `git kb code symbols OsRng` resolves to `vendor/rand_core-0.6.4/src/os.rs:48`; `git kb code callers search` top hits are `vendor/aho-corasick/...`, `vendor/deltae/docs/doc/search.js`. `stats` reports 16,804 rust files indexed while the repo's own `prompt-hub/src` is a few hundred. | CONFIRMED |
| C-CODE-5 | A real, committed code auto-research loop exists for this target (not just the index). | `/home/drdave/Desktop/meta/prompt_hub/.handoff/loop/research-ledger.md` enumerates dimensions D1–D6 (read-path/RBAC/storage/caching) with status legend `[ ]/[~]/[x]/[!]`; `/home/drdave/Desktop/meta/prompt_hub/.handoff/loop/findings/{D1_D5,D2,D3,D4_D6,verdicts}.md` carry the claims + verdicts. | CONFIRMED |
| C-CODE-6 | The AI doc/test-generation workflow is PR-time scaffolding, NOT a code auto-research loop. | `/home/drdave/Desktop/meta/prompt_hub/.github/workflows/ai-test-doc-generation.yml`: every job `if: ${{ vars.ENABLE_AI_WORKFLOWS == 'true' }}`; output saved to `/tmp/generated/*` and `upload-artifact` `retention-days: 7`; PR comment says "Download… Incorporate into your PR if acceptable". No commit-back, no ledger, no verification gate. | CONFIRMED |
| C-CODE-7 | The index claims freshness but has no scheduled re-index for this repo's CI. | `git kb code stats` → `Stale files: 0`, index built today; `git-kb` exposes `prune` ("Remove stale branch symbols") and config `[hooks] context_injection=true` in `/home/drdave/Desktop/meta/prompt_hub/.kb/config.toml`. No `.github/workflows/*` re-indexes the graph; refresh is local/daemon-driven only. | CONFIRMED |

`git-kb` commands run (grounding, read-only): `git kb code stats`, `git kb code doctor`,
`git kb code entrypoints`, `git kb code callers search`, `git kb code dead`,
`git kb code symbols row_to_prompt` (→ `prompt-hub/src/storage.rs:1550`), `git kb code symbols OsRng`.

Note: `/home/drdave/Desktop/meta/prompt_hub/.gitignore` has no `.kb` entry and `.kb/.gitignore` is
empty — the `.kb/store` documents are tracked, but the multi-gigabyte code-index cache is not gated,
a residency risk for a vendored repo.

## 2. Web auto-research

How external/world facts (dependency currency, advisories, vendor docs) are refreshed.

| ID | CLAIM | Evidence | Verdict |
|----|-------|----------|---------|
| C-WEB-1 | Dependency-currency `web auto-research` runs on a fixed cadence (Dependabot). | `/home/drdave/Desktop/meta/prompt_hub/.github/dependabot.yml`: `cargo` `interval: daily` (limit 10), `github-actions` `interval: weekly`. | CONFIRMED |
| C-WEB-2 | Dependabot decisions are reasoning-anchored, not blind bumps. | Same file pins `password-hash` and ignores standalone semver-major/minor bumps with a rationale tied to `argon2 0.5.x` re-export and closed PR #109 — evidence the currency feed is curated. | CONFIRMED |
| C-WEB-3 | Advisory `web auto-research` runs daily against RustSec (real, ungated). | `/home/drdave/Desktop/meta/prompt_hub/.github/workflows/security.yml`: `schedule: cron '0 0 * * *'`; jobs `cargo install cargo-audit && cargo audit` and `cargo install cargo-deny --locked && cargo deny check`. Comment notes installing current `cargo-deny` to parse newer RustSec advisories (CVSS 4.0). | CONFIRMED |
| C-WEB-4 | A reasoning-anchored advisory remediation loop is scaffolded but NOT wired. | `/home/drdave/Desktop/meta/prompt_hub/.github/workflows/security_remediation.yml`: `schedule cron '0 6 * * 1'`, gated on `ENABLE_AI_WORKFLOWS`, consolidates `cargo audit`/`cargo deny`/`gh api .../dependabot/alerts`, but the final step `Remediation agent (placeholder)` only `echo`s — the agent runner is not connected. | CONFIRMED |
| C-WEB-5 | `audit_sync.yml` is internal doc-sync automation, NOT web research. | `/home/drdave/Desktop/meta/prompt_hub/.github/workflows/audit_sync.yml`: triggers on `push: paths docs/audits/**`; diffs changed audit JSON and runs a Python updater (`scripts/update_<task-tracker>_from_audit.py`) against the root task tracker; commits `[skip ci]`. It propagates already-produced audit findings; it fetches nothing external. (Also a Rust-native exception: Python in a Rust-native repo.) | CONFIRMED |
| C-WEB-6 | There is NO 90-day recency window and NO source ledger for web sources in prompt-hub. | `grep -rinE '90[ -]day\|recency\|stale\|invalidat\|autoresearch'` over `*.md`: the only `90-day` hit is `/home/drdave/Desktop/meta/prompt_hub/prompts/plan-loop-parallel-run.md:62` (the meta planning-harness prompt, not a prompt-hub artifact). `.handoff/loop/research-ledger.md` has zero `http/advisory/cve/web/date` rows — it is code-only (D1–D6). No web-source ledger file exists. | CONFIRMED |
| C-WEB-7 | The multi-model / external-AI surface is opt-in scaffolding, not autoresearch. | 7 workflows gate on `ENABLE_AI_WORKFLOWS` (`grep -rl` → `multi-model-evaluation.yml`, `ai-code-review.yml`, `ai-test-doc-generation.yml`, `ai-safety-deployment.yml`, `security_remediation.yml`, `external-ai-apis.yml`, workflows/README.md). `external-ai-apis.yml` calls `api.anthropic.com` (`claude-opus-4-1`) and `api.devin.ai` only on label `ai-review` + the var; `multi-model-evaluation.yml` pipes model output to `jq '.'` in logs with no persistence. Output is ephemeral; nothing is committed or invalidated. | CONFIRMED |

## 3. Cadence

| Phase | Required refresh (what fires) | Source of truth |
|-------|------------------------------|-----------------|
| Per-cycle (code) | `git kb code` re-read of the target dimension; append to `.handoff/loop/findings/Dn.md` + `verdicts.md`. Index auto-refreshes via the local daemon/`[hooks]` in `.kb/config.toml`. | `research-ledger.md`; `git kb code stats` `Last indexed` |
| Per-PR (advisory, ungated) | none AI; `cargo audit` + `cargo deny` run on push/PR. | `security.yml` |
| Daily (web/advisory) | Dependabot opens cargo PRs; `security.yml` cron `0 0 * * *` re-queries RustSec. | `dependabot.yml`, `security.yml` |
| Weekly (web/actions) | Dependabot bumps GitHub Actions; `security_remediation.yml` cron `0 6 * * 1` (inert until `ENABLE_AI_WORKFLOWS`). | `dependabot.yml`, `security_remediation.yml` |
| Batch-boundary (deep) | re-run `git kb code entrypoints --refresh` + `git kb code prune` to rebuild edges/drop `stale` branch symbols (currently never done → edges=0). | `git-kb code` help |
| Resume | re-read `research-ledger.md` + `verdicts.md`; re-`stats` to confirm index freshness. | `.handoff/loop/` |
| Stale invalidation | manual only: the root task tracker flags the committed SARIF (`2026-06-04`) as `stale`; `.handoff/history/LESSONS.md` L1 records "Stale-backlog drift". No automated rule re-opens `[x]` rows on source change. | task tracker; `LESSONS.md` |

Stale/invalidate gaps to flag: (1) no rule that flips a `[x]` dimension back to `[~]` when its cited
`prompt-hub/src/*.rs` changes; (2) no max-age that marks a web/advisory finding `stale` and forces
re-fetch; (3) committed audit SARIF can rot silently (the `2026-06-04` instance already did).

## 4. Upgrade rows

| ID | UPGRADE (axis: autoresearch) | Evidence basis | Acceptance | Risk | Reversibility |
|----|------------------------------|----------------|------------|------|---------------|
| U-AR-1 | Build the `git-kb` call graph: scope the index to first-party paths (exclude `vendor/`, `worktrees/`, `_workspace/`) and run `git kb code entrypoints --refresh` so `Call edges > 0`. | C-CODE-2, C-CODE-4 | `git kb code stats` shows non-zero `Call edges` and `git kb code callers row_to_prompt` returns real call sites; `dead` no longer flags trait impls. | Low — read-only index config | High — re-index restores prior state |
| U-AR-2 | Add a scheduled re-index + `git kb code prune` step (CI or daemon hook) so the graph cannot silently go `stale`. | C-CODE-7 | A weekly job re-indexes first-party files and prunes stale branch symbols; `stats` `Last indexed` advances without manual run. | Low | High — remove the schedule |
| U-AR-3 | Add a web-source ledger (`.handoff/loop/web-source-ledger.md`) with `url`, `fetched-on`, `claim`, `expires-on`, gated to a rolling 90-day `recency` window. | C-WEB-6 | Every web/advisory-derived plan claim cites a dated ledger row; rows older than 90 days are marked `stale` and force re-fetch. | Low | High — delete the file |
| U-AR-4 | Wire the `security_remediation.yml` agent (replace the placeholder `echo` with a runner pointed at `skills/security-remediation/SKILL.md`) to close the advisory loop. | C-WEB-4 | A scheduled run opens one verified remediation PR (or escalates) instead of only emitting `alert-inventory.md`. | Med — opens PRs; keep `ENABLE_AI_WORKFLOWS` gate | High — unset the var |
| U-AR-5 | Add a code-evidence invalidation rule: when a file cited by a `[x]` dimension changes, re-open that row to `[~]` in `research-ledger.md`. | §3 stale gaps, `LESSONS.md` L1 | A `git diff` over cited paths flips affected ledger rows; a gate fails when a `[x]` row cites a since-modified file. | Low | High — drop the rule |
| U-AR-6 | Decide the AI-workflow surface: either promote one (e.g. `ai-code-review`) to a persisted, fed-back loop or document the rest as explicit scaffolding to cut maintenance/`MCP`-rot. | C-CODE-6, C-WEB-7 | Workflows README states, per workflow, "loop" vs "scaffold"; ephemeral-artifact jobs are consolidated. | Low | High — revert doc/config |
| U-AR-7 | Gate the `.kb` code-index cache in `.gitignore` (track `.kb/store`, ignore the cache) to prevent index residency bloat in a vendored repo. | C-CODE-1, `.gitignore`/`.kb/.gitignore` empty | `.kb/cache` ignored; `git status` clean after a re-index. | Low | High — remove the ignore line |

## 5. Gate handoff

Tests/gates that must FAIL CLOSED until the missing stale-evidence / autoresearch checks exist
(handed to plan-test-strategist / feature-forge as RED specs):

- **G-AR-1 (code graph non-empty):** assert `git kb code stats` reports `Call edges > 0` for the
  first-party scope. Currently RED — edges = 0 (C-CODE-2).
- **G-AR-2 (no stale code evidence):** for each `[x]` row in `research-ledger.md`, assert no cited
  `prompt-hub/src/*.rs` file has a mtime/commit newer than the verdict; else the row is `stale` →
  fail. Currently RED — no such check exists (U-AR-5).
- **G-AR-3 (web source recency):** assert every web/advisory-derived plan claim maps to a
  `web-source-ledger` row whose `fetched-on` is within the rolling 90-day `recency` window; expired
  rows `invalidate` the claim. Currently RED — no web ledger exists (C-WEB-6).
- **G-AR-4 (advisory loop closes):** assert `security_remediation.yml` produces a remediation PR or
  an explicit escalation, not just `alert-inventory.md`. Currently RED — agent step is a placeholder
  (C-WEB-4).
- **G-AR-5 (index cache residency):** assert `git status --porcelain` is clean after `git kb code`
  re-index (cache is ignored). Currently RED — `.kb` not in `.gitignore` (U-AR-7).

Note: G-AR-1, G-AR-3, G-AR-4 fail closed today; G-AR-2/G-AR-5 are not yet expressible because the
ledger-link and ignore mechanisms do not exist — building the gate is itself the first upgrade.
