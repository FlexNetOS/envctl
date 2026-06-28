# prompt-architecture — rusty-idd

Target: `/home/drdave/Desktop/meta/rusty-idd` (READ-ONLY review). Method:
`.claude/skills/plan-prompt-architecture` (skill dir absent in target; applied from harness intent).
Scope: agent-facing prompt/tool/model/instruction surfaces reviewed AS ARCHITECTURE.
Laws: read-only, fail-closed, every CLAIM cites `file:line`.

## Verdict (headline)

rusty-idd's prompt architecture is a deliberate **engine-owned single-source-of-truth control
plane**: the *workflow* lives in the Rust engine (`rusty-idd next` / spec artifact-DAG), and the
per-vendor instruction surfaces are reduced to ~10-line **thin adapters that are GENERATED and
drift-gated** (ADR-0010, ADR-0015). This is architecturally strong. The residual risk is at the
**edges the render gate does not cover**: root `CLAUDE.md`/`GEMINI.md` are hand-maintained
duplicate prose (outside `render --check`); the **model lanes** (Codex gpt-5.5 vs Claude opus)
are declared in scattered repo-local TOML/test fixtures with no governing decision record; and
every lifecycle hook is a compile-gated `cargo run` of the engine.

## Instruction surfaces (the prompt corpus)

| surface | file | role | always-loaded? |
|---|---|---|---|
| durable repo rules | `AGENTS.md:1-61` | North Star + Operating/Workflow/Codex rules + PR evidence | YES (large prose) |
| Claude bridge | `CLAUDE.md:1-17` | compatibility note → points at Rusty IDD workflow | YES (hand-authored) |
| Gemini bridge | `GEMINI.md:1-17` | near-identical to CLAUDE.md (vendor word swap only) | YES (hand-authored) |
| vendor adapters | `.codex/rusty-idd-adapter.md`, `.claude/…`, `.agents/…`, `.devin/…` | engine-GENERATED thin pointer to `rusty-idd next` | YES (but ~10 lines) |
| Codex subagents | `.codex/agents/rusty-idd-{explorer,gap-hunter,verifier,implementer}.toml` | role + sandbox + reasoning + `developer_instructions` | on-demand |
| Codex model loop | `.codex/loops/rusty-idd-model-loop.toml:11-51` | 3-pass read-only design loop with per-pass model/prompt | on-demand |
| Codex exec policy | `.codex/rules/default.rules:1-68` | prefix allow/deny/prompt rules | per-tool-call |
| Codex lifecycle hooks | `.codex/hooks.json:1-76` | SessionStart/Pre/PostToolUse/Stop/SubagentStop → engine | per-event |
| Claude hooks | `.claude/settings.json:3-14` | SessionStart only (`rusty-idd next`) | per-event |
| Claude command guard | `.claude/agent-guard.toml:1-23` | destructive-command deny + evidence list | per-tool-call |
| reusable skills | `.agents/skills/{rusty-idd-adopt-first,…-knowledge,…-verify,…-codex-rust-env}/SKILL.md` | workflow recipes | on-demand |
| generated context | `.idd/knowledge/*.md` (e.g. `operating-model.md`, `plan-context.md`, `report.md`) | graph/context artifacts agents read first | referenced |
| spec base | `openspec/specs/base.md` | OpenSpec lifecycle anchor | referenced |

## Tool grants (tools granted per agent surface)

| CLAIM | surface | grant | citation |
|---|---|---|---|
| C1 | Codex explorer | `sandbox_mode = "read-only"`, reasoning medium | `.codex/agents/rusty-idd-explorer.toml:3-4` |
| C2 | Codex gap-hunter | `sandbox_mode = "read-only"`, reasoning high | `.codex/agents/rusty-idd-gap-hunter.toml:3-4` |
| C3 | Codex verifier | `sandbox_mode = "read-only"`, reasoning high | `.codex/agents/rusty-idd-verifier.toml:3-4` |
| C4 | Codex implementer | `sandbox_mode = "workspace-write"`, reasoning high — the ONLY write-capable agent | `.codex/agents/rusty-idd-implementer.toml:3-4` |
| C5 | model-loop excludes the writer | loop runs only explore+gap-hunt+verify; implementer is NOT a pass → write is out-of-band, explicit-auth only | `.codex/loops/rusty-idd-model-loop.toml:11-51`; `AGENTS.md:39` |
| C6 | implementer is sole writer + gated on spec readiness | refuses if proposal/specs/design/ADR/tasks not ready (`rusty-idd spec status`) | `.codex/agents/rusty-idd-implementer.toml:6-9` |
| C7 | Codex exec policy: host control forbidden | `systemctl`/`kill`/`pkill`/`killall` → `forbidden` | `.codex/rules/default.rules:6-41` |
| C8 | Codex exec policy: installs prompt | `cargo install`/`npm -g`/`pip install` → `prompt` (route via meta/envctl) | `.codex/rules/default.rules:43-68` |
| C9 | Claude guard: destructive ops denied | `git reset --hard`, `git clean -fd[/ffdx]`, `git branch -D`, `rm -rf` | `.claude/agent-guard.toml:7-13` |
| C10 | Claude guard posture is advisory | `mode = "warn"` (NOT block) — softer than Codex `forbidden` | `.claude/agent-guard.toml:2` |
| C11 | Claude repo settings grant NO tool allowlist | only a SessionStart hook; tool permissions inherit user-global | `.claude/settings.json:3-14` |
| C12 | Codex concurrency budget | `max_threads=4`, `max_depth=1`, `job_max_runtime_seconds=1800`, `project_doc_max_bytes=65536` | `.codex/config.toml:4,11-13` |
| C13 | every hook = compile+run of the engine | all hooks `cargo run … --bin rusty-idd -- …` (next / codex workflow-check / env-check) | `.codex/hooks.json:8,21,34,46,56,68`; `.claude/settings.json:9` |

## Model lanes

| CLAIM | lane | model(s) | citation |
|---|---|---|---|
| M1 | Codex design lane (headless) | `gpt-5.5-mini` (explore, gap-hunt) + `gpt-5.5` (verify) | `.codex/loops/rusty-idd-model-loop.toml:19,33,42` |
| M2 | Codex invocation | `codex exec --json --sandbox <mode> --model <m> -c model_reasoning_effort="<r>"` | `crates/cli/src/commands/codex.rs:1859-1873` |
| M3 | Claude/Gemini are "bridge" lanes, not headless model lanes | both delegate to the engine; no model pinned in product flow | `CLAUDE.md:1-16`, `GEMINI.md:1-16` |
| M4 | Claude interactive lane appears only as runner config examples | `interactive_command: "claude-i"` / `"claude --model opus"` are TEST/doc fixtures, not a hardwired product lane | `crates/runner/src/config.rs:308,315,340,542` |
| M5 | Codex reads Claude/Gemini docs as fallback project docs | `project_doc_fallback_filenames = ["CLAUDE.md","GEMINI.md"]` → cross-vendor doc coupling | `.codex/config.toml:5` |
| M6 | model identifiers are repo-tracked but ungoverned | `gpt-5.5*` lives in a committed TOML; no ADR pins/owns the model-lane choice (only the generic "Upgrade only" rule) | `.codex/loops/rusty-idd-model-loop.toml:19,33,42`; `AGENTS.md:42` |

Lane summary: effectively **dual-lane by convention** — Codex (gpt-5.5 family) drives the
read-only 3-pass design loop; Claude (opus, interactive) is the human-facing executor; Gemini is
a bridge stub. There is no single decision record that *names* the lanes or their version policy.

## Hidden architectural couplings

| CLAIM | coupling | citation |
|---|---|---|
| H1 | All 4 vendor adapters are ENGINE-GENERATED + drift-checked → instruction surface is single-source-of-truth | `crates/cli/src/commands/render.rs:18-22,47-67`; `Justfile:38-39,119` |
| H2 | Adapters are byte-identical except the vendor name (verified by diff: only lines 1/3/5 differ) | `.codex/rusty-idd-adapter.md` vs `.claude/…` vs `.agents/…` (diff) |
| H3 | `.devin/` is in the VENDOR set + has a rendered adapter, but `.gemini`/`.kimi` are ABSENT despite being named across ADR-0010/0015 prose → adapter set ≠ doc-named surface set | `crates/cli/src/commands/render.rs:18-23`; `adr/0010…:25`; dir listing |
| H4 | Lifecycle hooks depend on the workspace COMPILING (cold `cargo run`, 180s timeout) → a non-building tree breaks SessionStart/PreToolUse gating | `.codex/hooks.json:8-9,20-21`; `.claude/settings.json:9` |
| H5 | Codex fallback project doc = `CLAUDE.md`/`GEMINI.md` → the Codex lane inherits the Claude/Gemini bridge prose | `.codex/config.toml:5` |
| H6 | Generated context artifact bakes a machine-absolute path into a git-tracked instruction surface | `.idd/knowledge/operating-model.md:3-4` (`/home/drdave/Desktop/meta`) |
| H7 | `next` front door + both `next` and `spec next` are the SAME oracle (cannot disagree) — adapters obtain direction by calling it, not by carrying prose | `adr/0015-harness-control-plane.md:39-46` |

## Governance controls

| CLAIM | control | citation |
|---|---|---|
| G1 | render drift gate in CI | `render-check` (`render --all --check`) in `just ci` | `Justfile:38-39,119` |
| G2 | full CI gate chain includes codex/model surfaces | `…render-check adr-check … codex-env-check codex-runtime-audit codex-model-loop fmt-check lint audit` | `Justfile:119` |
| G3 | ADR immutability + supersede-don't-edit | accepted ADRs immutable; new decision = new ADR | `AGENTS.md:26`; `adr/0001…:42` |
| G4 | duplicate-ADR-number fail-closed gate | `spec adr list --check`; frozen `ACCEPTED_DUPLICATE_ADRS` baseline (0002/0004/0005/0006) | `adr/0016-adr-ledger-reconciliation.md:40-51` |
| G5 | required PR evidence + evidence list | build/test/lint/secret-scan/migration/rollback/manifest + guard `[evidence]` | `AGENTS.md:48-56`; `.claude/agent-guard.toml:15-23` |
| G6 | thin-adapter doctrine is engine-enforced, not honor-system | adapter body literally says "GENERATED … render --check (CI) fails on drift" | `.codex/rusty-idd-adapter.md:3,5` |
| G7 | design-first / write-requires-auth | default harness read-only; write pass needs explicit auth + ready OpenSpec | `AGENTS.md:39`; `adr/0001…:18-26` |

## Instruction coherence / drift findings

| CLAIM | drift | citation |
|---|---|---|
| D1 | `AGENTS.md` is still a LARGE always-loaded prose harness (3 rule sections) — the exact "token black hole" ADR-0015 indicts, yet it is not itself thinned/generated | `AGENTS.md:7-47`; `adr/0015…:9-15` |
| D2 | Root `CLAUDE.md`/`GEMINI.md` are ~99% duplicate hand-prose and are OUTSIDE the render set (render covers `.{vendor}/…-adapter.md` only) → silent drift path | `CLAUDE.md:1-16` vs `GEMINI.md:1-16`; `crates/cli/src/commands/render.rs:18-22,108-109` |
| D3 | Enforcement-posture mismatch: Claude guard `mode="warn"` vs Codex rules `decision="forbidden"` for comparable destructive intent | `.claude/agent-guard.toml:2`; `.codex/rules/default.rules:8` |
| D4 | Stray committed backups in the instruction tree (`AGENTS.md.idd-bak-1`, `.env.*.idd-bak-1`) | dir listing (`AGENTS.md.idd-bak-1`, etc.) |
| D5 | Machine-absolute path leaked into a generated, shared instruction artifact | `.idd/knowledge/operating-model.md:3` |
| D6 | Four frozen duplicate ADR numbers persist (documented + gated, not silent) | `adr/0016…:8-18` |

## ADR candidates

- **ADR-C1 — Bring root `CLAUDE.md`/`GEMINI.md` under the single-source-of-truth boundary.**
  render generates/drift-checks `.{vendor}/rusty-idd-adapter.md` (G1, H1) but the *root* bridge
  files are hand-maintained duplicates outside the gate (D2). Extending the render boundary (or
  generating the bridges) is a genuine architecture decision about where the SoT edge sits, and
  it composes ADR-0015's "vendor adapters stay minimal / drift-checked" intent (`adr/0015…:47-50`).
  Cited basis: `CLAUDE.md:1-16`, `GEMINI.md:1-16`, `render.rs:18-22`.

- **ADR-C2 — Model-lane policy of record.** No ADR names the lanes (Codex `gpt-5.5*` vs Claude
  `opus`) or their version/upgrade governance; the identifiers live in committed TOML + test
  fixtures only (M1, M4, M6). A decision record would pin the lane→model mapping, the reasoning
  tiers, and how "Upgrade only" (`AGENTS.md:42`) applies to model versions. Architectural because
  it governs which model executes which gate. Cited basis: `model-loop.toml:19,33,42`,
  `config.rs:308,542`, `codex.rs:1859-1873`.

- **ADR-C3 — Hook execution contract (compile-gated `cargo run` vs prebuilt binary).** Every
  lifecycle gate cold-runs the engine via cargo with a 180s timeout (H4), so agent gating fails
  whenever the workspace does not build. Choosing prebuilt-binary/`just`-shim hooks vs the current
  cargo-run is a durable cross-cutting decision. Cited basis: `.codex/hooks.json:8-9`,
  `.claude/settings.json:9`.

## No-ADR rationale (deliberately not ADR-worthy)

- **Guard-posture mismatch (D3)** — `warn` vs `forbidden` is a config-tuning reconciliation, not a
  new architectural commitment; fix in-place under existing governance, no ADR. (`agent-guard.toml:2`)
- **Absolute-path leak (D5)** — generator bug in the knowledge renderer; fix the generator, no
  decision record needed. (`operating-model.md:3`)
- **Stray `*.idd-bak-1` backups (D4)** — housekeeping/cleanup, not architecture.
- **`.gemini`/`.kimi` absence (H3)** — already covered by the existing "adoption of a new surface
  is a deliberate change" note in `render.rs:15-17`; no separate ADR until a surface is actually
  adopted.
- **Duplicate ADR numbers (D6)** — already resolved by ADR-0016; N/A — no new ADR.

## Confidence

High for surfaces, tool grants, model lanes, and governance controls (all read directly from
tracked files with line cites). Medium for the model-lane *intent* (M4/M6 inferred from the
absence of a decision record + fixture-only references, not a contradicting source).
