# rusty-idd — governance/settings/config findings

Axis: **governance+settings+config**. Target root: `/home/drdave/Desktop/meta/rusty-idd`
(read-only). Fail-closed: a missing/empty expected surface is recorded as a finding, not a
silent pass. Every CLAIM cites `file:line`.

## Surfaces scanned
| surface | path(s) | present | evidence |
|---|---|---|---|
| rules (.claude) | `.claude/rules/meta-destructive-commands.md` | yes | 20-line destructive-guard rule |
| rules (.codex) | `.codex/rules/default.rules` | yes | prefix_rules: systemctl/kill/pkill/killall/cargo-install/npm-g/pip |
| instructions | `AGENTS.md` (60 lines), `CLAUDE.md` (16), `GEMINI.md` (16), `.github/copilot-instructions.md` | yes | AGENTS.md is the canon; CLAUDE/GEMINI are byte-identical 609B bridge stubs |
| `.kb/AGENTS.md` | — | **no** | rusty-idd uses its OWN `.idd/knowledge` engine, not FlexNetOS `.kb`. N/A — by design |
| hooks (claude) | `.claude/settings.json` | yes | **SessionStart only** (1 of N lifecycle events) |
| hooks (codex) | `.codex/hooks.json` | yes | SessionStart + PreToolUse + PostToolUse + Stop(×2) + SubagentStop |
| hooks (git) | `.githooks/{pre-commit,commit-msg,pre-push}` | yes | pre-commit=fmt; commit-msg=commitlint; pre-push=`make ci` |
| policy (agent-guard) | `.claude/agent-guard.toml` | yes | mode="warn", deny[], evidence.required[] |
| policy (handoff) | `.handoff/policy.toml` | **no** | continuity is via `.handoff/tasks/*.task.json` (handoff.task.v1) + `.handoff/context/capsule.json`. N/A — schema-contract model, no policy.toml |
| ADR | `adr/` | yes (dir present) | AGENTS.md§5 / settings.json cite ADR-0010/0015/0018 |
| runtime settings (claude) | `.claude/settings.json` | yes | hooks only; **no `permissions`, `env`, `model`, `mcpServers`** block |
| runtime settings (codex) | `.codex/config.toml` | yes | `[features].hooks=true`, `[agents] max_threads=4 max_depth=1 job_max_runtime=1800` |
| codex agents | `.codex/agents/*.toml` (4) | yes | explorer/gap-hunter (read-only) + implementer (workspace-write) + verifier |
| codex loop | `.codex/loops/rusty-idd-model-loop.toml` | yes | 3-pass read-only model loop (gpt-5.5-mini/gpt-5.5) |
| skills | `.agents/skills/*` (4) | yes | adopt-first, codex-rust-env, knowledge, verify. **No `.claude/skills/`, no `.claude/agents/`** |
| Cargo / workspace | `Cargo.toml` | yes | 11 members, `resolver="3"`, `rust-version="1.88"`, `[patch.crates-io] repomix-shared` |
| toolchain pin | `rust-toolchain.toml` | **no** | absent; CI pins toolchain via script default (see CLAIM gov-003) |
| cargo config | `.cargo/audit.toml` | yes | advisory baseline (2 accepted-risk RUSTSEC warnings) |
| `.kb/config.toml` | — | **no** | N/A — no `.kb` (see above) |
| CI workflows | `.github/workflows/{ci,promote-verify,on-push-main,release,codeql,semantic-pr-title}.yml` | yes | dev gate (ci) + promote gate + release-please |
| dep automation | `renovate.json` + `.github/dependabot.yml` | yes (**both**) | overlapping ecosystems (see CLAIM gov-006) |
| meta registration | `meta/.meta.yaml:278` | yes | `rusty-idd: {repo, tags:[tools,idd]}` (default develop) |
| validation gate | `crates/core/src/validation.rs` (26 `require_file`) | yes | existence-only gate over governance files |

## Hygiene detectors
| detector | result | evidence | risk |
|---|---|---|---|
| MCP rot | **none found** | no `[mcp_servers.*]` in `.codex/config.toml`; no `mcpServers` in `.claude/settings.json` | none — clean |
| Skill overload | **none found** | 4 skills in `.agents/skills/`; no listing-budget key set; no `.claude/skills/` | low |
| Token burn | **present** | every hook is `cargo run --quiet … rusty-idd --` (compiles+runs binary); `.codex/hooks.json:15-39` fires it on BOTH PreToolUse+PostToolUse (matcher `Bash\|apply_patch`) → 2 cargo invocations per Bash call, plus SessionStart/Stop×2/SubagentStop; `timeout 180` each | medium |
| Permission drift | **partial** | `.claude/settings.json` has no `permissions.allow/deny` (inherits user-global); destructive guard declared but unenforced (CLAIM gov-002); `agent-guard.toml:2` mode="warn" | high |
| Config drift | **present** | toolchain nightly-vs-stable (gov-003), broken commitlint hook (gov-004), dual dep-bots (gov-006), 35 `*.idd-bak-*` litter (gov-007) | medium |
| Cross-surface drift | **present** | claude(1 event) vs codex(6 hooks) fail-OPEN (gov-001); rule/policy with no teeth (gov-002); ungated codex control plane (gov-008) | high |

## CLAIM rows

- CLAIM[gov-001] axis: governance+settings+config | surface: `.claude/settings.json` vs `.codex/hooks.json` | evidence: `.claude/settings.json:3-14` (only `SessionStart`) vs `.codex/hooks.json:3-74` (SessionStart, PreToolUse, PostToolUse, Stop×2, SubagentStop running `rusty-idd codex workflow-check`/`env-check`) | confidence: High | **Fail-OPEN drift:** the Codex harness enforces the workflow gate, the destructive/scope checks, and the handoff/subagent-stop checks at 6 lifecycle points; the Claude harness enforces NONE of them — it only computes the front-door `next` at session start. A Claude agent runs ungated. `render.rs:25` (`ADAPTER_FILE = "rusty-idd-adapter.md"`) shows `rusty-idd render` emits ONLY the adapter prose, NOT the hooks — so `settings.json`/`hooks.json` are hand-maintained and drift unbounded, contradicting the adapter.md claim (`.claude/rusty-idd-adapter.md:3`) that the dir is engine-generated and `render --check` fails on drift.

- CLAIM[gov-002] axis: governance+settings+config | surface: `.claude/agent-guard.toml` + `.claude/rules/meta-destructive-commands.md` | evidence: `.claude/agent-guard.toml:7-13` deny[`git reset --hard`,`git clean -fd`,`git branch -D`,`rm -rf`] with `mode = "warn"` (`:2`); the ONLY consumer is `crates/core/src/validation.rs:48` `require_file(root, ".claude/agent-guard.toml", …)` which checks **existence only** — the `deny`/`mode`/`evidence.required` keys are never parsed or enforced | confidence: High | **Policy with no teeth:** no PreToolUse hook in `.claude/settings.json` invokes the guard, and `.codex/rules/default.rules` forbids systemctl/kill/cargo-install but NOT the git/rm destructive commands. The destructive-command guard is decorative in both harnesses; the parent meta rule (`meta/.claude/rules/meta-destructive-commands.md`) claims these "trigger PreToolUse denial" — false for rusty-idd.

- CLAIM[gov-003] axis: governance+settings+config | surface: `Cargo.toml` vs `scripts/ci/envctl-rust-env.sh` + `.github/workflows/ci.yml` | evidence: `Cargo.toml:22` `rust-version = "1.88"` (stable) vs `scripts/ci/envctl-rust-env.sh:121` `toolchain="${RUSTY_IDD_RUST_TOOLCHAIN:-nightly}"` and `ci.yml:29,37` cache keys `…-nightly-…`; `ci.yml:51` runs `cargo clippy --all-targets --all-features -- -D warnings` | confidence: High | **Toolchain drift:** CI builds/lints on nightly while the manifest advertises stable 1.88, and no `rust-toolchain.toml` reconciles them. A contributor on stable 1.88 (and the meta-root preflight clippy mirror) sees a different lint set than the nightly `-D warnings` CI gate → false-green locally, red in CI.

- CLAIM[gov-004] axis: governance+settings+config | surface: `.githooks/commit-msg` + `commitlint.config.cjs` | evidence: `.githooks/commit-msg:4-5` runs `npx --no -- commitlint --edit "$1"`; no `package.json`/`bun.lock`/`pnpm-lock.yaml` exists at repo root (verified absent); `commitlint.config.cjs:2` extends `@commitlint/config-conventional` (undeclared dependency) | confidence: High | **Broken hook:** `--no` forbids on-the-fly install, so `commitlint` resolves to nothing → the hook errors when `npx` is present (or silently skips per `:6-7` when absent). Either way commit-message conventions are unenforced locally, and installing the dep would violate AGENTS.md§Codex-rule-8 (no user-global installs). `semantic-pr-title.yml` enforces titles in CI but commit bodies go unchecked.

- CLAIM[gov-005] axis: governance+settings+config | surface: `.codex/hooks.json` + `.claude/settings.json` | evidence: `.codex/hooks.json:15-39` PreToolUse+PostToolUse both `matcher: "Bash|apply_patch"` each exec `cargo run --quiet … workflow-check`; `.claude/settings.json:9` and `.codex/hooks.json:8` SessionStart exec `cargo run --quiet … next` | confidence: High | **Token/latency burn:** every Bash/apply_patch tool call triggers two `cargo run` workflow-check passes (pre+post); session start and every Stop/SubagentStop add more. Even incremental, this is repeated process-spawn + link overhead on the hot path of agent action, `timeout 180` per call. No prebuilt-binary path (`cargo build` once → call the artifact).

- CLAIM[gov-006] axis: governance+settings+config | surface: `renovate.json` + `.github/dependabot.yml` | evidence: `renovate.json:12-14` `matchManagers:[cargo, github-actions]`; `.github/dependabot.yml:3,8` `package-ecosystem: cargo` + `github-actions` weekly | confidence: High | **Dual dependency bots:** Renovate and Dependabot both manage the identical cargo + github-actions ecosystems → duplicate/competing update PRs and config split-brain. Both are existence-gated by `validation.rs` so the redundancy is institutionalized.

- CLAIM[gov-007] axis: governance+settings+config | surface: `.idd/` + repo root backup litter | evidence: 35 `*.idd-bak-*` files (e.g. `.idd/MANIFEST.tsv.idd-bak-1..19`, `AGENTS.md.idd-bak-1`, `.env.contract.yaml.idd-bak-1`, `.github/workflows/ci.yml.idd-bak-1` 6.3K stale beside live 2.2K `ci.yml`); gitignored via `.gitignore:19` `*.idd-bak-*` | confidence: High | **Hygiene/litter:** the validate/manifest tooling rotates `*.idd-bak-N` with no cap (19 MANIFEST snapshots). Untracked, so not a correctness risk, but unbounded disk growth and a stale CI snapshot (`ci.yml.idd-bak-1`) that no longer matches the live workflow can mislead readers.

- CLAIM[gov-008] axis: governance+settings+config | surface: `crates/core/src/validation.rs` | evidence: `validation.rs:21-48` `require_file` covers AGENTS.md, `.claude/agent-guard.toml`, the 4 CI workflows, etc. — but does NOT require `.codex/hooks.json`, `.codex/config.toml`, `.codex/rules/default.rules`, `.codex/agents/*`, `.codex/loops/*`, or `.agents/skills/*` | confidence: High | **Ungated control plane:** the harness that is ACTUALLY enforced (Codex, 6 hooks) is invisible to `rusty-idd validate`; deletion or drift of `.codex/hooks.json` (the only real workflow gate) would pass validation green. The gate also checks existence only, so an emptied/relaxed governance file passes.

## UPGRADE rows

- UPGRADE[gov-001] target-surface: `.claude/settings.json` + render pipeline · evidence: CLAIM gov-001 (`.claude/settings.json:3-14` vs `.codex/hooks.json:3-74`) · impact: closes the fail-OPEN gap so Claude agents are gated identically to Codex (workflow-check + handoff/subagent checks) · effort: M · risk-tier: **PROPOSE** (wires new enforcement hooks + permission surface; owner-walled) · acceptance-criterion: `.claude/settings.json` declares PreToolUse/PostToolUse/Stop/SubagentStop hooks invoking `rusty-idd codex workflow-check`, AND `rusty-idd render claude --check` exits non-zero when the hooks drift (i.e. render now owns settings.json hooks, not just the adapter.md) · reversibility: high — revert the settings.json block + render template commit; no code/data migration.

- UPGRADE[gov-002] target-surface: `.claude/agent-guard.toml` enforcement path · evidence: CLAIM gov-002 (`agent-guard.toml:2,7-13`; `validation.rs:48` existence-only) · impact: gives the destructive-command deny[] real teeth (PreToolUse denial), matching the stated rule and the parent-meta contract · effort: M · risk-tier: **PROPOSE** (adds a hard denial path + flips `mode` warn→block; never weaken — this strengthens) · acceptance-criterion: a PreToolUse hook (claude) + a `prefix_rule` set (`.codex/rules/default.rules`) deny `git reset --hard`/`git clean -fd`/`git branch -D`/`rm -rf`, AND a test asserts each is blocked; `mode="block"` · reversibility: high — revert hook + rule rows; guard returns to warn-only.

- UPGRADE[gov-003] target-surface: `rust-toolchain.toml` (new) + `ci.yml`/`envctl-rust-env.sh` · evidence: CLAIM gov-003 (`Cargo.toml:22` 1.88 vs `envctl-rust-env.sh:121` nightly) · impact: one authoritative pinned toolchain shared by local + CI; ends nightly-vs-stable lint divergence · effort: M · risk-tier: **PROPOSE** (toolchain change can surface new lints) · acceptance-criterion: `rust-toolchain.toml` channel == the value `envctl-rust-env.sh` resolves, AND `cargo clippy --all-targets --all-features -D warnings` passes on that pinned channel == the channel in CI cache keys · reversibility: high — delete `rust-toolchain.toml`, restore script default.

- UPGRADE[gov-004] target-surface: `.githooks/commit-msg` · evidence: CLAIM gov-004 (no node manifest; `npx --no` cannot resolve commitlint) · impact: commit-message conventions actually enforced without a forbidden user-global install · effort: S · risk-tier: **PROPOSE** (governance hook behavior change) · acceptance-criterion: the hook either (a) invokes an envctl/meta-tracked commitlint binary, or (b) is replaced by a Rust-native conventional-commit check in `rusty-idd`, AND a malformed message is rejected in a test — no `npx --no` dead path remains · reversibility: high — revert the hook file.

- UPGRADE[gov-005] target-surface: `.codex/hooks.json` + `.claude/settings.json` hook commands · evidence: CLAIM gov-005 (2× `cargo run` per Bash call) · impact: removes per-tool-call compile/link overhead on the agent hot path · effort: S · risk-tier: **PROPOSE** (changes how the gate is invoked) · acceptance-criterion: hooks call a prebuilt artifact (e.g. `$root/target/release/rusty-idd` built once at SessionStart) instead of `cargo run` per call, AND PreToolUse no longer rebuilds; gate semantics unchanged (workflow-check still runs) · reversibility: high — restore the `cargo run` command strings.

- UPGRADE[gov-006] target-surface: `renovate.json` xor `.github/dependabot.yml` · evidence: CLAIM gov-006 (both manage cargo+github-actions) · impact: single dependency-update authority, no duplicate PRs · effort: S · risk-tier: **PROPOSE** (removing a governance file `validation.rs` currently requires) · acceptance-criterion: exactly one bot manages cargo+github-actions; the retired file is removed AND its `require_file` row dropped from `validation.rs` so the gate stays green · reversibility: high — re-add the file + require_file row.

- UPGRADE[gov-007] target-surface: `*.idd-bak-*` rotation in the validate/manifest tooling · evidence: CLAIM gov-007 (35 backups; 19 MANIFEST snapshots; stale `ci.yml.idd-bak-1`) · impact: bounded backup litter; no stale-CI-snapshot confusion · effort: S · risk-tier: **APPLY** (prune of gitignored, regenerable, untracked litter only — no tracked/source change) · acceptance-criterion: the rotation caps retained `*.idd-bak-*` per source to a small N (e.g. 3) and prunes older; repo holds ≤3 backups per source after a validate run · reversibility: trivial — backups are regenerable; nothing tracked is touched.

- UPGRADE[gov-008] target-surface: `crates/core/src/validation.rs` · evidence: CLAIM gov-008 (codex control plane + `.agents/skills` ungated) · impact: the actually-enforced harness becomes gate-protected against silent deletion/drift · effort: S · risk-tier: **PROPOSE** (extends a CI gate — stricter) · acceptance-criterion: `validation.rs` `require_file`s `.codex/hooks.json`, `.codex/config.toml`, `.codex/rules/default.rules`, and at least the 4 `.codex/agents/*.toml`, AND `rusty-idd validate` fails when any is removed · reversibility: high — revert the added require_file rows.

## Gaps / owner walls
- **No `rust-toolchain.toml`** — toolchain truth lives only in a shell-script default (`envctl-rust-env.sh:121`); confirmed by file absence. Blocks reproducible local lint parity (gov-003).
- **`render` scope** — `render.rs` owns only `rusty-idd-adapter.md`; whether it SHOULD own `settings.json`/`hooks.json` is an owner/ADR decision (touches ADR-0010 "thin adapter" boundary). gov-001/gov-002 are PROPOSE-walled on this.
- **Relaxation law** — gov-002 (warn→block) and gov-008 (stricter gate) STRENGTHEN guards; none of these upgrades weakens a rule, policy, hook, gate, or permission. No owner relaxation is proposed.
- **`.kb`/`.handoff/policy.toml` absent by design** — N/A (rusty-idd uses `.idd/knowledge` + `handoff.task.v1` file/schema contract, per codemap); not a defect, recorded for completeness.
- **Lockfile/generated artifacts** — `Cargo.lock` present; any dep change is REGENERATE (run cargo, never hand-edit) — not in scope of these rows.
