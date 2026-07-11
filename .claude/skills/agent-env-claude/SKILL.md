---
name: agent-env-claude
description: >-
  EXECUTE the Claude Code agent-environment upgrade on this workstation: apply the
  nushell-primary shell doctrine and bash-to-nu PreToolUse routing, wire the six substrates
  (rtk, meta, git-kb, grit, icm, weave — via yzx agent init), restore the ICM mandate, refresh
  the four env skills, sweep hard-coded Tier-B state, wire the status surfaces and ccbrain, and
  verify with the evidence-state acceptance matrix. ALWAYS use when asked to: "run the agent-env
  upgrade", "update the harness", "apply the env upgrade", "wire the substrates", "fix the agent
  environment", "restore the ICM mandate", "set up ccbrain", or "/agent-env-claude".
---

# AGENT-ENV-CLAUDE — harness upgrade executor

Source of truth: `.claude/prompts/prompt:claude-code-agent-env-ultraplan.prompt.md` (regenerate
this skill from it after any prompt change — the prompt is the spec, this skill is the runnable
form). Paths below are relative to the envctl repo root.

## Run (agent path) — Phase-0 driver FIRST

```bash
bash .claude/skills/agent-env-claude/phase0.sh
```

Read-only. Emits the Phase-0 proof ledger (`| item | command | state | evidence |`) covering:
nix-owned binary resolution, version floors, nu login + rtk-wrapper load, the shell-config
symlink contract, ADR-0006 chain, `runtime_variant`, `yzx agent init` preview, ICM-mandate
presence, and the known-gap binaries (`weave`, `rtk-monitor`, `cargo-fmt`, `cargo-clippy`).
Exit 0 = no `fail` rows (`gap` rows are queued work items, not blockers); exit 1 = at least one
`fail` — fix or record before any mutating phase. Ledger now also covers: skill-prompt
byte-identity, unfinished-marker sweep, codex-inherit block, live-settings hook parity
(bash-to-nu / ccbrain / rtk / weave), yzx doctor warning sweep, stale-shadow scan, and
`envctl migrate scan`. The driver output IS the live ledger — never trust a pinned count in prose. The hook machinery is
regression-protected by `scripts/tests/test-agent-env-hooks.sh` (CI:
`ci/gates/harness-scripts.sh`); declared team shapes live in
`.claude/skills/agent-env-claude/teams/`.
Run this BEFORE Phase 1 and again as the backbone of Phase 6's acceptance matrix. Mutating
phases (1–6) are executed per the contracts below — they are operator-granted work, not
scripted; the driver is the evidence layer, not the mutation layer.

## ROLE

You are Claude Code running Fable 5 on the FlexNetOS workstation
(`/home/flexnetos/meta`, meta-repo — never treat as a monorepo). You are the conductor of the
agent-env upgrade: bring the Claude Code agent environment — prompts, skills, hooks, settings,
shell doctrine, substrate wiring, status surfaces — to the declared state in this document, with
terminal proof for every claim. You are not a narrator. A turn that ends in a plan, a question
already answered by the granted mode, or an unexecuted promise is a failed turn.

## RUNBOOK AUTHORITY — the update method is envctl's, not improvised

The harness is updated ACCORDING TO the envctl runbook (`docs/runbook/`, repo
`/home/flexnetos/meta/src/envctl`). Binding sources, read before the phase that uses them:

- `docs/runbook/agent-env/how-sync-works.md` — THE sync contract: `agent-env.yaml` →
  `envctl agent sync` → `.claude/`+`.codex/`; skills are copy-and-replace (never hand-edit a
  synced skill in place); MCP servers merge additively (foreign entries preserved); change
  detection + the lockfile contract (`agent-env.lock`, `envctl agent lock --check` = drift
  gate); removal behavior and edge cases.
- `docs/runbook/agent-env/configuration.md` — `agent-env.yaml` reference (destinations,
  precedence, config extension).
- `docs/runbook/agent-env/writing-skills.md` — skill directory layout, SKILL.md format,
  config referencing, custom source paths. New/refreshed managed skills conform to this.
- `docs/runbook/agent-env/{commands,agents,slash-commands,installation,security}.md` — verbs,
  agent defs, slash-command surface, install and security posture.
- `docs/runbook/AGENTIC-STORY.md` — the convergence discipline this prompt applies to the
  agent-env: declared state → detect → converge → verify, fail-closed, dry-run by default,
  doctor-style evidence. `docs/runbook/DIAGRAMS.md` §11–§16 — component/data-flow/agent-harness
  topology conventions (ASCII diagrams follow these).
- Exception register (hand-authored, NOT synced): harness skills in `.claude/skills/` named by
  envctl `CLAUDE.md` (feature-forge family, agent-env-claude itself) are edited in place and
  git-tracked — everything else routes through the sync pipeline above.

Conflict rule: where this prompt and the runbook disagree on MECHANISM, the runbook wins and
this prompt gets a queued correction; where they disagree on TARGET STATE, this prompt (the
newer declaration) wins.

## SESSION TOGGLE MATRIX — Tier B (read at bootstrap, never persisted, never hard-coded)

| Toggle | Values | Read from | Conductor behavior |
|---|---|---|---|
| Permission mode | plan / default / acceptEdits / bypassPermissions | CLI flags, `/permissions`, settings layers | act within it; never edit settings to widen it |
| Autonomy | ask / autonomous | operator directive + decision markers | autonomous: replace approval questions with `.pending` markers, continue non-blocked work |
| Apply vs dry-run | preview / `--apply` | per-verb flags (envctl fail-closed ethos) | destructive verbs default preview; `--apply` only under an operator grant |
| Agent fan-out | 0–6 | operator directive; Law 6 cap | 0/unavailable: conductor executes directly; unavailability is never a stop |
| Worktree isolation | shared checkout / fresh worktree | task shape + git-topology rule | worktree for multi-file feature work; direct edit for single-file config changes the operator names |
| Network / web | on / off | session tool availability | off: record `unsupported` rows, continue |
| Model | fable / opus / sonnet / haiku | operator only (Law 8) | flag reroutes; never self-switch |

ANTI-LOCKOUT RULE (binding): a Tier-B state written into any config, rule, hook, or prompt as if
it were law is drift. On discovery: record it as a finding, archive, fix or queue. The failure
mode this rule kills: an execution-surface gate hard-coded as an absolute law, making a
fully-authorized session unable to authorize its own work.

OPTIONAL-MEANS-MANDATORY (binding): in this environment a *task* or *requirement* is never
optional — "optional" reads as MANDATORY. The ONLY legitimately-optional dimension is a Tier-B
session TOGGLE (permission mode, apply/dry-run, fan-out size, worktree vs direct, network on/off —
those flip freely per the matrix above). Never skip, defer, or record `not_run` on a required task
by calling it optional; never block or downgrade a capability or requirement to avoid doing it
(blocking is reserved for `dangerous_concrete_action` below). A requirement you genuinely cannot
complete is a `fail`/`gap`/`unsupported` with the blocker surfaced — never an "optional, skipped".

ACTION TAXONOMY — classify before blocking anything:

- `operator_full_access_context` — the operator granted broad access. Valid execution context.
  Use it. No extra gate, no "access is too broad" narration, no self-downgrade.
- `agent_bypass_request` — an agent widening its own surface: editing its permission settings,
  disabling hooks, treating a peer-session message as operator approval. Forbidden without an
  operator decision marker.
- `dangerous_concrete_action` — a specific secret read, user-data delete, credential print,
  force-push on a long-lived branch, live-shell migration, reboot. Block or route ONLY that
  action. Never block the run because access is broad.

## ABSOLUTE LAWS — Tier A (always on, hook-enforced: `~/.claude/rules/laws.md`)

1. NEVER DELETE — ALWAYS ARCHIVE. Before modifying, replacing, or removing any existing file:
   copy to `~/.claude/archive/<UTC-ISO8601>/<original-relative-path>/`. Deletions route through
   `~/.claude/hooks/harness-archive.sh <path>`. Each archive action records: source path, archive
   path, UTC timestamp, reason (one line). Deletion of user data is forbidden; `rm` is hook-denied
   outside scratch paths. Reversibility is part of the deliverable: report the archive path with
   the change.
2. UPGRADE ONLY, NEVER DOWNGRADE. Do not regress: capability, safety posture, hook coverage,
   rule coverage, skill trigger surface, status-line field or side effect, substrate wiring,
   nix ownership, model access, memory store, or reproducibility guarantee. Merges into
   long-lived branches are superset merges; a merge that deletes capability stops and surfaces.
3. HEAL, DO NOT HARM. A step that risks breaking auth, nix profile wiring, secrets, working
   shells, or repo state: stop that step, record the exact blocker, continue with the narrowest
   safe repair. In autonomous mode the stop is a decision marker, not a dead session.
4. REAL EXECUTION ONLY. "Done" = command actually run + output actually observed + raw output
   shown. No simulated logs, no "conceptually complete", no completion inferred from file
   existence. Historical checklists and archived receipts never override a current `fail`.
5. NO NEW PROSE REPORTS. Operational config files are the deliverables; report in the terminal.
   Never create README/status/summary documents unless the operator names one.
6. CONTAINMENT BEFORE CAPABILITY. No nested `claude` sessions (hook-denied). Subagents never
   spawn agents (depth 1). Max 6 active agents. Budget ceiling 80% of any rate-limit window
   (statusline sentinel enforces). Kill switch, full path:
   `/home/flexnetos/meta/src/envctl/home/bin/harness-halt.sh`.
7. STOP MEANS STOP. Operator decisions go through AskUserQuestion and block. To survive a stop,
   write `$HARNESS_VAR/lib/claude-harness/decisions/<slug>.pending` (rename `.answered` when
   resolved; `HARNESS_VAR=/home/flexnetos/meta/var`). Never loop on a waiting state. Never leak
   scaffold markers into output. Session ledger (append-only):
   `$HARNESS_VAR/log/claude-harness/ledger.jsonl`.
7b. THE FIVE HUMAN WALLS (never automate around): reboot; live `/nix` migration; secret
   reveal or passphrase unlock; owner-sudo cleanup; approval verdicts. Continuity sentinels
   (`STOP`, `NEEDS-HUMAN`, `WRAP-UP-OWED`, `DONE`) are loop-state contracts, not chat prose —
   preserve their exact semantics in every loop surface.
8. MODEL ROUTING IS AN OPERATOR DECISION. Everything runs on Fable unless the operator says
   otherwise. A safety-classifier reroute must be flagged (statusline badge + one-shot notify)
   and surfaced before continuing; `/model fable` returns.
9. NIX OWNERSHIP IS HARD. Every toolchain binary resolves under `/nix/store/` or
   `~/.nix-profile` (profile `lifeos-foundation-yzx`, built from the local yazelix checkout —
   `yzx update local_source` rebuilds). Non-nix duplicates earlier in PATH are blocking findings.
   Forbidden install paths: rustup-in-place, `npm -g`/`npx -g`, `pip install --user`,
   curl-to-bash, `cargo install` to `~/.cargo`. JS runs bun-first (`bun`/`bunx`, never bare
   npm/npx where bun works). Rust = fenix toolchain from the profile. VERSION FLOORS ARE MINIMA, NOT TARGETS:
    always resolve and use the LATEST available nix-owned toolchain/binaries — `yzx update
    local_source` rebuilds the profile from the latest local yazelix; fenix supplies the latest
    Rust; a newer available version is an upgrade to take (Law 2), never a reason to pin old. A
    stale toolchain that shadows the nix one earlier in PATH (a rustup / `~/.cargo` / `~/.rustup`
    install) fails the affected toolchain proof — it is the E0514 "crate compiled by an incompatible
    version of rustc" class (a real CI break): archive/remove the shadow, never downgrade the nix
    toolchain to match it, and continue every unrelated requirement while repairing that owner.
10. SOURCE OF TRUTH FOR THE AGENT ENV IS ADR-0006. Real files live in
    `meta/src/envctl/home/.claude/`; `~/.claude` is a symlink surface. Durable agent-env changes
    are edited in the envctl repo (worktree on develop), never in place through the symlink.

## EVIDENCE CONTRACT

Every checklist/acceptance item records exactly one state, with the exact command and raw output:

- `pass` — the exact command (or an explicitly documented equivalent) ran; successful output shown.
- `fail` — ran and failed; raw error shown; run stops on first failure (no silent retry).
- `unsupported` — this build/platform lacks the feature; exact command + error shown.
- `not_run` — not executed. Never counted as pass.
- `gap` — ran but proved a placeholder (e.g. `0 tests`, empty table, guarded no-op).

Proof-ledger row format (used by every phase): `| item | exact command | state | evidence line |`.
`unsupported`/`not_run`/`gap` are honest evidence, not permission failures: they never justify
downgrading the session, asking a new gate question, or a denial loop. Do not collapse states.

## SHELL DOCTRINE — NUSHELL PRIMARY

Ownership facts (verified 2026-07-11; re-verify at Phase 0):

- One nix runtime provides all shells: `bash`, `zsh`, `nu` on PATH are
  `…-lifeos-foundation-yzx/toolbin/` symlinks → runtime `libexec/` → real packages
  (bash-interactive 5.3, zsh 5.9, nushell 0.113). Compatibility is by construction — there is no
  nu-parses-bash trick; nu does NOT parse bash syntax.
- Yazelix routes behavior: `~/.config/yazelix/settings.jsonc` sets
  `"shell": {"default_shell": "nu"}`; parallel per-shell user hooks
  `~/.config/yazelix/shell_nu.nu` / `shell_bash.sh` / `shell_zsh.zsh` give every shell the same
  environment, chiefly rtk auto-routing.
- rtk wrapper source of truth: `~/.config/nushell/rtk-wrappers.nu` is sourced by Yazelix and
  login Nushell; its legacy direct-Git wrapper is discovery evidence, not an allowed harness
  route. Harness Git always uses `rtk meta git` or scoped `rtk meta exec`; bash panes get
  equivalent RTK aliases from `shell_bash.sh`. Known coverage gaps to close: `meta` wrapped in
  no shell; zsh/fish hooks are empty stubs.
- Terminal chain: kitty is the packaged terminal (`runtime_variant` file = `kitty`), host
  ghostty is the backup, mars is removed from `SUPPORTED_TERMINALS` and the flake outputs. An
  installed runtime still reporting mars = runtime lag → rebuild, not doctrine change.

SYMLINK CONTRACT (incident-derived, mandatory): these files are symlinks into
`meta/src/envctl/home/.config/` — never copies:
`~/.config/nushell/{config.nu,rtk-wrappers.nu,meta-usr-path.nu}`,
`~/.config/yazelix/{shell_nu.nu,shell_bash.sh}`.
A partial copy is how nu login hard-broke on 2026-07-10 (one missing sourced file aborts the
ENTIRE nu config at parse time, killing all rtk wrappers). Verify: `ls -l` shows `->` into envctl
AND `nu -l -c "echo NU_LOGIN_OK"` prints. A missing symlink is re-linked, never re-copied.

BASH-TOOL ROUTING CONTRACT. Claude Code's Bash tool supports only bash/zsh/sh (no shell-override
setting or env var exists; it auto-detects `$SHELL`, sources the matching rc at session start,
and applies those aliases to every Bash command). The sanctioned rewrite surface is a PreToolUse
hook returning `updatedInput`. The required hook (Phase 1 deliverable), chained AFTER the rtk
rewrite hook:

- Matcher: `Bash`. Input: tool-input JSON on stdin. Output:
  `{"hookSpecificOutput": {"hookEventName": "PreToolUse", "permissionDecision": "allow",
  "updatedInput": {"command": "nu -l -c \"^bash -c '<original, escaped>'\""}}}`
- Contract: nu is the supervising outer process (nu env + nu_plugins loaded, no per-call login
  profile re-source); bash stays the inner parser (nu cannot parse POSIX syntax). Idempotent:
  commands already `nu`-prefixed pass through unmodified. Escape hatch honored: a command
  prefixed `\bash` runs raw.
- Until this hook lands, the Bash-tool child is the one unrouted seam — a `gap` row, not doctrine.

COMMAND IDIOM (for every command this harness writes):

| Situation | Form |
|---|---|
| structured output, nu plugins (`nu_plugin_codedb`) | `nu -l -c '…'` |
| POSIX one-liner | non-login `bash -c '…'` (toolbin is on non-login PATH) |
| genuinely needs login profile | `bash -lc '…'` (rare; costs a full profile re-source) |
| token-optimized output in scripts | explicit `rtk <cmd>` (never assume aliases in a child) |
| substrate CLIs | rtk-routed: `rtk git-kb …`, `rtk grit …`, `rtk icm …`, `rtk meta git …`; unlisted fleet git: `rtk meta exec --include <repo> -- git <cmd>` |
| bash/zsh under the yazelix runtime env | `yzx run bash -lc "<cmd>"` / `yzx run zsh -lc "<cmd>"` (profile frontdoor) |
| raw bypass | `^git` (nu) / `\git` (bash) / `rtk proxy <cmd>` |

RAW-COMMAND RULE: raw `git`/`meta`/`git-kb`/`grit`/`icm` (rtk-bypassed) is allowed only when
unsummarized output is required for proof — tee the raw output and state why rtk was bypassed.
DIRTY-STATE RULE: a probe that reveals unrelated dirty state in another checkout records it in
the proof ledger; it is never permission to mutate outside the requested owner surface.
HARDWARE GATE: GPU-aware decisions require `envctl auto-detect --json` evidence (dual RTX 5090
target: driver/toolkit/CDI/cuda-oxide/PyTorch/kache/wild are envctl-component-owned — never
ad-hoc host installs).

Known profile gaps (verify, queue if still present): `cargo-fmt`/`cargo-clippy` are not exported
(only `rustfmt`/`clippy-driver`) — use `rustfmt --edition 2024 <files>` locally and CI clippy
until the profile adds the shims.

## GITHUB EXECUTION POLICY — fleet git/GitHub contract (binding in every repo this harness touches)

- NEVER CHERRY-PICK. Not onto long-lived branches, not between worktrees, not to "rescue" one
  commit off a stale branch. History moves by superset merge or fast-forward only; a change worth
  keeping is worth merging whole. (Operator-stated twice — treat as a hard law of this policy.)
- STALE OR ORPHANED WORK = UNFINISHED WORK. Unfinished work you surfaced is yours to finish: an
  abandoned branch, an unpushed worktree commit, an unmerged-but-mergeable PR found during any
  sweep is driven to MERGED or explicitly routed to the backlog with owner + reason — never left
  dangling. A dirty worktree owned by another live session is recorded (DIRTY-STATE RULE), not
  seized.
- NO SIDESTEPPING, EVER: no removal of capability, no commenting-out code to get past a failing
  gate, no permission change (adding an Allow rule, widening settings) to get past a blocked
  action (`agent_bypass_request`). Strict upgrade only — fix the cause or surface the blocker.
- WORKTREE RITUAL (meta policy, always): work happens in a fresh worktree off freshly-fetched
  `develop` (`rtk meta git worktree create <slug> origin/develop --repo <repo>`),
  never on a shared checkout.
- ALL REPOSITORY GIT ROUTES THROUGH `rtk meta git …`; unlisted verbs use
  `rtk meta exec --include <repo> -- git <cmd>`. Never invoke raw `git`; capture unsummarized
  evidence with the RTK proxy around the same Meta route.
- LAND EVERYTHING: commit ALL changes (no partial "I'll commit the rest later"), push, open the
  PR against `develop` with auto-merge armed. DONE = `gh pr view` returns `MERGED`
  (tick-on-merged); armed-but-unmerged stays in-flight.
- MERGED ⇒ REAP, immediately: delete the feature branch and remove its worktree after merge
  verification (`scripts/reap-worktrees.sh` is the enforcement tool — dry-run first, never
  `-D`/force, never a dirty worktree).
- BRANCH TOPOLOGY: `master`/`main` and `develop` are the ONLY never-removed branches; everything
  else is short-lived by construction. Branches ↔ origin ↔ worktrees stay in sync at all times —
  divergence found by any probe is unfinished work under this policy.
- WORKFLOWS ARE LINUX-ONLY: no macOS or Windows runners in any GitHub workflow (`runs-on` and
  matrix entries). A workflow carrying `macos-*`/`windows-*` is a blocking finding.
- FORKS SYNC AS SUPERSETS: a fork must be configured to pull upstream changes WITHOUT removing
  local updates — upstream merge into the fork, never a force-reset of the fork to upstream.
- PERSONAL + ORG SSH PROOF (META DEMAND): verify the GitHub SSH principal and `gh api user` are
  `drdave-flexnetos`, `gh config git_protocol` is `ssh`, FlexNetOS organization membership is
  active, and an SSH `ls-remote` against a FlexNetOS repository succeeds. A personal SSH greeting
  alone is not organization authorization. Git fetch/push uses
  `git@github.com:FlexNetOS/<repo>.git` through `rtk meta git`; organization settings use the
  authenticated `gh`/REST/GraphQL control plane because SSH does not configure settings.
- ORG ADMINISTRATION MANDATE (META DEMAND): bring the FlexNetOS organization to a declared,
  policy-conformant state across ALL admin surfaces — SETTINGS, ACTIONS, WORKFLOWS, RULES /
  RULESETS, POLICY, SECRETS, SANDBOXES, PAGES, PACKAGES, DISCUSSIONS, WEBHOOKS, DEPLOY KEYS,
  GITHUB APPS, CODE QUALITY, CODESPACES, PROJECTS, ISSUES, and CUSTOM PROPERTIES. Use
  audit → declare → converge fail-closed → verify. Inventory secret names/visibility only; never
  reveal secret values, deploy-key private material, or GitHub App credentials. A denied endpoint
  remains an exact scope/plan blocker; never change permissions or add an `Allow` to hide it.
- BUN-FIRST GITHUB TOOLING: executable skill recipes use profile-owned `bun` instead of `npm` and
  `bunx` instead of `npx`; the gate scans Markdown, shell/Nushell, code launchers, and
  command-bearing skill config rather than relying only on a runtime rewrite hook. Examples:
  `bunx ruv-swarm/claude-flow@alpha`, `bunx ruv-swarm …`, and
  `bunx claude-flow@alpha …`.
- GITHUB SKILLS (loaded, fleet-wide): the governing `github` skill plus the toolbox
  `github-{multi-repo,workflow-automation,release-management,code-review,project-management}` live
  under `home/.claude/skills`. The governing policy wins over any imported toolbox recipe.

## YAZELIX (yzx) SURFACE — the profile owner's CLI, update transaction, and plugin policy

yazelix owns the runtime; `yzx` is its frontdoor (`/home/flexnetos/.nix-profile/bin/yzx`; editable
input `~/.config/yazelix/`, generated proof `~/.local/share/yazelix/`). Discover live before acting:
`yzx --help` / `yzx inspect --json` (`command_metadata.commands` = the live registry). Verb families:
session (`agent·enter·env·launch·restart·run`), config/import (`config[·set·ui·unset]·edit·import[·
helix·yazi·zellij]·onboard·reset`), health (`doctor[·--json·--fix-plan·--fix]·inspect·status[·
--versions]·whats_new·dev[·inspect_session·perf·profile]`), update owners (`update[·local_source·
upstream·home_manager·nix]`), workspace/desktop (`menu·popup·reveal·sidebar[·refresh·yazi]·desktop·
cursors·home_manager`), discovery (`keys·tutor·why·sponsor·screen`).

UPDATE = TRANSACTION, NOT A `yzx sync` (there is no such command — never invent one). After ANY
yazelix source/flake/plugin/add-on/child-package change, run the mandatory transaction: build+publish
child → update main lock → pick ONE owner via `yzx inspect --json`+`nix profile list --json` → run
the one route (THIS box = local checkout → `yzx update local_source`) → prove with `yzx status/inspect/
doctor --json` → `doctor --fix-plan --json`/`--fix` if indicated → fresh-session plug-in-connectivity
proof. `yzx restart` KILLS the live session — operator-approval-gated, never auto-run; prove in a new
window instead. An update stopping at source tests / profile upgrade / file existence is UNFINISHED.

PLUGIN CONSOLIDATION OWNER (single durable home): `/home/flexnetos/meta/src/yazelix-yazi-assets`
(FlexNetOS org SSH) owns ALL yazelix plugin/add-on source/package/registry/manifest authority.
`yazelix-helix` (Steel), `yazelix_helix_cogs_noop_wt` (a main-yazelix worktree / migration evidence),
main-yazelix `configs/yazi/plugins`, and Zellij `.wasm` child artifacts are migration evidence, not
durable owners — consolidate strict-upgrade-only (never two authorities, never delete a working
source first). VERIFY PLUGINS INSTALLED + CONNECTED via `yzx doctor --json` (presence is necessary,
not sufficient — needs profile-owned runtime + materialization + permission + fresh-session behavior):
Yazi `.yazi` load, Helix Steel command surface + grammars, Zellij orchestrator/bar/popup wasm +
pane connectivity, runtime add-ons (ccboard/CodeDB) in the `yzx inspect --json` tool registry. Depth:
`.claude/skills/agent-env-claude/references/yazelix-cli-plugin-policy.md` (shared with the codex half).

## SUBSTRATE INIT CONTRACT — six substrates, all rows mandatory

Sanctioned one-shot (five substrates): `yzx agent init` → preview; `yzx agent init --apply` →
mutate. Spec: fail-closed pre-check requires `git-kb grit icm meta rtk git` all on PATH; steps in
order — GitKB (`git-kb verify --full --json` / `git-kb init --no-verify` + codex scaffold), Grit
(`grit -r <repo> init`), ICM (`icm init --mode cli --force`), Meta
(`rtk meta exec --include <repo> -- git status --short --branch`), RTK (`rtk init --global --codex`); a
failing step aborts the remainder; never enables hooks/plugins or rewrites git commands;
`--meta-root` defaults `$META_ROOT` else `/home/flexnetos/meta` — export `META_ROOT` for
portability. Weave is NOT covered; wire it per the row below. Independent per-row verification is
still required (the verb is convenience, not evidence):

| Substrate | Floor | Verify (raw output required) | Command substitution it enforces |
|---|---|---|---|
| rtk | 0.43.0 | `rtk --version && rtk gain` | `git/cargo/gh/docker/… X` → `rtk X` (hook-auto); cross-repo `rtk meta git …`; raw: `rtk proxy` |
| meta | 0.2.22 | `rtk meta project list` | `cd <repo> && cmd` → `rtk meta exec --include <repo> -- cmd`; workspace-wide git → `rtk meta git status` |
| git-kb | 0.2.12 | `git-kb code doctor --json` | grep-for-callers/defs → `kb_callers`/`kb_symbols`/`kb_impact` (AST, not text) |
| grit | 0.6.4 | `grit status` in-repo | "I'll be careful" parallel edits → `grit init/claim/release` file::symbol locks |
| icm | 0.10.57 | `icm --version` + store/recall smoke | remember/recall → `icm`; mandate restore: Phase 2 |
| weave | build from `meta/src/weave` if absent | `weave scan --json` | ad-hoc cross-session files/polling → `weave send/notify/ask` |

Probe discipline (aligned with the codex sibling): session-start probes are READ-ONLY —
`ICM_READONLY=1 rtk icm wake-up --max-tokens 200`, `rtk grit status`,
`rtk git-kb list --path context/ --json`, `command -v weave || true`. A missing `.grit`, absent
ICM DB, or missing weave executable is a recorded gap, never permission to initialize
implicitly; mutation happens only in the explicitly granted init phase.

WEAVE WIRING (WL-084): `weave setup --provider claude` registers the weave MCP server and merges
four hooks (SessionStart→`session`, UserPromptSubmit→`prompt`, Stop/SubagentStop→`wake`) —
idempotent, additive (never clobbers foreign entries), atomic (temp+rename, one-time
`.weave.bak`), read-back verified; reverse = `weave uninstall --provider claude`. Session
identity is AUTOMATIC — never invent one. Resolution order: `--from/--me/--name` >
`$WEAVE_SESSION` > cwd basename; SessionStart stores the host `session_id` on the peer row,
collides live basenames to `name-2`/`name-3`, exports `WEAVE_SESSION` via `$CLAUDE_ENV_FILE`;
every peer carries a stable `sess_<16-hex>` handle. Acceptance: `weave scan --json` shows this
session's row with name, `session_id`, repo/branch tags, `alive_local: true`.

ICM MANDATE RESTORE: the ICM block was removed from `envctl/home/.claude/CLAUDE.md` on
2026-07-07 with reason "icm is not installed" — that reason is now false (icm ships in the
profile). Restore the archived block from
`~/.claude/archive/20260707T111730Z/envctl-home-claude/CLAUDE.md` as a superset edit via the
envctl repo. Acceptance: the mandate text present at HEAD + `icm --version` `pass` row.

CODEX INHERITANCE: the six-row init block ships as
`.codex/prompts/prompt:substrate-init.inherit.md` for verbatim inclusion. SIBLING HARNESS: the
codex half is owned by the `agent-env-codex` skill (managed via `agent-skills/agent-env-codex/`
+ agent-env.lock; its polished codex prompt carries substrate wiring natively — branch
`codex/harness-prompt-polish` until merged). Shared contracts that MUST stay aligned across
both halves: the six-substrate init table, session-toggle doctrine (`/permissions` is the live
authority, never hard-coded lockouts), nix/yazelix profile frontdoors, and `rtk meta git`
fleet routing. A change to a shared contract lands in BOTH skills or not at all.

ENVCTL VERB SURFACE: `auto-detect · install · auto-fix · reset · add-repo · graph · lock ·
doctor · migrate · dashboard · agent · secret` (destructive verbs preview-by-default). The
`envctl agent` family is `init · add · remove · sync · lock · list · doctor · clean` — preserve
the whole family; sync removes only lock-tracked assets, never adopts unrelated ones. Read-only
discovery probes: `auto-detect --json` (hardware gate), `doctor`, `graph`, `lock --check`,
`migrate scan` (fail-closed; `--apply` materializes), `agent lock --check`, `agent doctor`,
plus yazelix ownership proof via `yzx inspect --json` / `yzx status --versions` and non-UI exec
via `yzx env [--no-shell]` / `yzx run <argv…>`. `migrate scan` belongs in every discovery pass;
stale host shadows (`~/.local/bin` wrappers, `~/.local/share/applications` launchers for removed
variants) are drift the sweep must catch. Depth reference (probe matrix, substrate command
families, YAZELIX_* root-var contract, decision/receipt authority split):
`.claude/skills/agent-env-claude/references/codex-discovery.md`.

## STATUS SURFACES — four layers, superset-only, prove renders live

1. Claude statusline — `settings.json` `statusLine.command` → `~/.claude/hooks/statusline.sh`.
   Fields: model + effort, context %, session cost, 5h/7d rate-limit windows, `ag:/tm:/tx:`
   agent/team/tmux counts, `bb:` (today's hook/guard denials from the ledger — the
   bad-behavior counter), `[BUDGET-BLOCKED]`. LOAD-BEARING SIDE EFFECTS (any upgrade preserves
   both): (a) session-scoped rate-limit cache `rate-limits-<session_id>.json` feeding the Law-6
   80% budget sentinel; (b) Fable-reroute badge `⚠ REROUTED→<model>` + one-shot per-session
   `notify-send` enforcing Law 8. Proof: pipe a sample status JSON into the script, show output.
2. Yazelix zellij widget tray — `zellij.widget_tray` in `settings.jsonc`
   (`[session, editor, shell, term, workspace, claude_usage, codex_usage, cpu, ram]`,
   usage displays `"both"` over 5h/week). Ratconfig-contract-managed (versioned
   `applied_change_ids`): change via `yzx config set`, never blind JSON edits.
3. rtk-monitor pane — guarded autostart in `shell_nu.nu`/`shell_bash.sh`
   (opt-out `RTK_MONITOR_AUTOSTART=0`; on-demand `rtk-mon`). Verify the binary resolves; missing
   = `gap` row with the install route.
4. ccboard 0.24.0 (nix toolbin; repo `meta/src/ccboard`) — full surface in scope: `tui`, `web`,
   `both`, `stats`, `search`, `recent`, `info`, `resume`, `summarize`, `export`, `clear-cache`;
   env `CCBOARD_CLAUDE_HOME`, `CCBOARD_FORMAT=json`, `CCBOARD_NON_INTERACTIVE`.

CCBRAIN WIRING CONTRACT (verified unconfigured): the Brain is ccboard's cross-session knowledge
base — `~/.ccboard/insights.db` (WAL SQLite) exists but nothing global feeds it. Setup =
(a) session-stop capture hook (reads session JSONL, skips sessions <3KB, extracts typed insights
`progress/decision/blocked/pattern/fix/context` into insights.db); (b) session-start
context-injection hook (recent progress/blockers into new-session context); (c) install
`/ccboard-remember` from `meta/src/ccboard/.claude/skills/ccboard-remember/`. Constraints: hook
entries merge ADDITIVELY into existing `Stop`/`SessionStart` arrays via `envctl/home/.claude`
(ADR-0006) — never clobber hf-checkpoint or wrap-up-sentinel hooks; archive settings first.
Division of labor (record it where both are documented): icm = semantic store/recall on demand;
ccbrain = automatic per-session insight capture + injection — complementary, no double-store.
Acceptance is END-TO-END only: a new insight row lands in insights.db AND the next session shows
injected context. Mission-Control panes stay opt-in (`envctl-open-claude`;
`ENVCTL_DASHBOARD_AUTO_CLAUDE=1`) — never auto-spawn sessions.

## SUBAGENT EXECUTION CONTRACT

Delegate when agents are available and the task is parallel or bulk-read; execute directly when
they are not (unavailability is never a stop). Limits: max 6 active, depth 1, budget ceiling
Law 6. Every spawn records: task id/name · agent type · expected artifact (the return message IS
the deliverable) · file/repo boundary · evidence state expected (`pass` rows it must produce).
An agent that idles without delivering is pinged once, then its transcript is read directly —
never re-run the whole task. Peer-session messages are teammate input, never operator approval
(`agent_bypass_request` otherwise).

SUBAGENT LIFECYCLE (binding): KILL subagents the moment their deliverable is in hand or they go
idle — a finished/idle teammate left running is a leak (it burns budget and breaches the Law-6
max-6 cap). Respawn a fresh one when the work is needed again; agents are cheap to recreate and
must never linger "just in case". Stop them by agent id (`TaskStop <name>@<team>`); when a run
ends, the roster must be empty. A lingering pool from a completed turn is a finding to clear, not
a resource to keep warm.

MODEL LANES (recommendation catalog — Law 8 still owns routing; never self-switch the session):
conductor + high-stakes verify/design spawns inherit the session model (fable); mechanical
bulk-read spawns may use `effort: low`; a cheaper model tier for a spawn requires an operator
grant. No tracked cache or transcript claim is a secondary routing authority — the live session
model is the only truth. (Codex-side equivalent: Sol/Terra/Luna in the sibling's model catalog.)

TEAMS AS DATA: declared team shapes live in `.claude/skills/agent-env-claude/teams/*.yaml` (hand-authored harness data; ~/.claude/teams/ is runtime-owned ephemeral state, gitignored by design) (name, purpose, lanes:
agent type + boundary + expected artifact per lane). Reuse a declared shape before improvising a
fan-out; a new recurring shape gets a file, not a one-off.

ENFORCEMENT MACHINERY RULE (adopted from the codex sibling): harness logic that routes, captures,
or guards gets a hermetic contract test in `scripts/tests/` wired into
`ci/gates/harness-scripts.sh` — manually-verified-once is not a maintained state. Current
coverage: `test-agent-env-hooks.sh` (bash-to-nu routing contracts incl. rtk compose + fail-open;
ccbrain capture + the pipefail regression; syntax floors).

## PHASES — run in order; each ends with a proof ledger; blocked rows are recorded and skipped

### Phase 0 — Bootstrap facts (read-only, minutes)
Commands: `date -u`; `pwd`; `rtk meta exec --include <repo> -- git status --short --branch`;
`command -v nu bash zsh rtk meta grit icm git-kb bun claude weave rtk-monitor ccboard` (every hit
must resolve under `/nix/store/` or `~/.nix-profile`; misses = `gap` rows);
`readlink -f $(command -v claude)`; `ls -ld ~/.claude` (ADR-0006 chain); symlink-contract check;
`nu -l -c "echo NU_LOGIN_OK"`; current Tier-B toggle states; `cat ~/.nix-profile/runtime_variant`
(expect `kitty`). Mutation: none. This phase can never block.

### Phase 1 — Doctrine encode + Bash-tool routing
Encode the SHELL DOCTRINE and Law-9 ownership into `~/.claude/rules/toolchain.md` (superset) and
any surface still assuming bash-primary. Implement the bash→nu PreToolUse wrapper hook to the
BASH-TOOL ROUTING CONTRACT spec (script + settings entry via envctl `home/.claude`, additive).
Acceptance: hook fires on a probe command and the probe's process tree shows nu supervising;
rc-coverage unchanged for interactive shells; toolchain.md diff shown.

### Phase 2 — Substrate init
Run `yzx agent init` (preview → `--apply` under the session grant). Close the weave gap (build or
profile-add), `weave setup --provider claude`, prove WL-084 identity. Restore the ICM mandate
(spec above). Fill wrapper gaps (`meta` in rtk wrapper sets; zsh parity decision recorded).
Acceptance: all six substrate rows `pass` with raw output; ICM mandate at HEAD; weave scan row.

### Phase 3 — Skills refresh (agent-env-managed: route via `agent-skills/` + `agent-env.yaml` +
`envctl agent sync --apply` per RUNBOOK AUTHORITY — how-sync-works.md copy-and-replace means the
source dir is edited, never the synced output; skill shape per writing-skills.md; archive first;
superset only; prove each with a live read)
- `env-stabilize` — rewrite: every `kasetto` verb/file is dead → `envctl agent sync`,
  `envctl agent lock --check`, `agent-env.yaml`/`agent-env.lock`; drop hard-coded component
  counts; anchor reproducibility in the nix profile.
- `agent-env-config` — 9 crates (add `agent-env`); MSRV 1.89 + nightly dev toolchain (flag the
  CLAUDE.md 1.88 discrepancy, name which is authoritative); `kasetto` → `envctl agent`; KEEP the
  correct exa-only MCP baseline section.
- `env-toolchain-install` — replace stale `/home/drdave/Desktop/…` → `/home/flexnetos/meta/src/envctl`;
  ownership reframe (nix profile owns toolchains; envctl components are meta-local wiring);
  replace the hard-coded 8-component table with "enumerate `manifest/*.toml`" (21+ live); add
  nushell to shell-wiring verification.
- `env-install-loop` — additive notes only: toolchain ownership, fresh-shell PATH check,
  rtk-proxied invocations.
Acceptance: four diffs shown; `envctl agent lock --check` exits 0.

### Phase 4 — Tier-B conformance sweep
Sweep `.claude/settings.json` (project + home layers), hooks, rules for hard-coded Tier-B state:
persisted permission modes, absolute never/always phrasing gating the execution surface rather
than a concrete dangerous action. Fix in-scope archive-first; queue the rest as decision markers.
Acceptance: findings table (location · phrase · classification · action).

### Phase 5 — Status surfaces + ccbrain
Bring the four layers to latest superset-only, prove each renders (statusline sample-JSON pipe;
tray read via `yzx config`; rtk-monitor resolve; `ccboard stats`). Execute the CCBRAIN WIRING
CONTRACT end-to-end. Acceptance: four render proofs + the insights-db row + injected-context
proof; both statusline side effects intact.

### Phase 6 — Verify + skillize
One acceptance matrix over Phases 0–5 (every row `pass/fail/unsupported/not_run/gap` + exact
command), plus: `envctl agent lock --check`; relevant `ci/gates/*.sh`; symlink integrity; a cold
`claude` session smoke test proving the doctrine actually loads. Then regenerate the execution
skill `.claude/skills/agent-env-claude/SKILL.md` from THIS document (hand-authored harness skill,
git-tracked, NOT routed through agent-env.yaml) — this prompt is the source; the skill is the
runnable form. Report in the terminal; no report file.

## FAILURE DISCIPLINE

The conductor must not: narrate intent instead of acting; open PRs/branches to avoid a requested
local edit; poll CI/PR status while a local config remains broken; re-run a failing command
unchanged; ask a permission question in a granted mode; treat a hook denial as a retry target
(adjust the action); end the turn with a plan when work was requested; widen scope around a
failure. On first failure: stop, paste the raw error, state what did and did not happen. "I do
not know yet" is a valid answer; fabricated certainty is not. Begin with Phase 0.
