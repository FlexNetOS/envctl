# prompt-architecture — weave (cycle 4)

Target: **weave** (the fleet TRANSPORT plane — A2A mailbox + injector + job/lease/approval
substrate). North-star split: `harness_hub` = Front-Door interpreter (intent → model language);
**weave = transport plane** (moves messages/jobs/leases/approvals between agent sessions; it does
not interpret intent). Code read-only at `/home/drdave/Desktop/meta/.worktrees/plan-weave-red/weave`.

Note: `reports/codemap-weave.md` is empty/absent in this worktree — surfaces enumerated directly
from source (cited below). N/A — no codemap to lift from.

---

## 1. Instruction surfaces

- CLAIM: `CLAUDE.md` is the operating contract — mandatory new-worktree-off-`origin/develop` ritual + `develop`→`master` branch model | evidence: `CLAUDE.md:28-46` | confidence: high
- CLAIM: Rust-native invariant is an explicit instruction surface with a 4-step drift-remediation procedure ("transform to Rust-native") run every session start | evidence: `CLAUDE.md:47-59` | confidence: high
- CLAIM: token-light MCP rule (WL-051/ADR-0003) is encoded as a first-class CLAUDE.md invariant — "adding a capability must not add standing tokens" | evidence: `CLAUDE.md:119` | confidence: high
- CLAIM: `.codex/AGENTS.md` is a parallel Codex-CLI instruction surface declaring the ECC baseline + the `.agents/skills/weave/SKILL.md` repo skill as separate from the Claude harness | evidence: `.codex/AGENTS.md:6-14` | confidence: high
- CLAIM: weave-orchestrator skill is the single entry point — a 4-agent Producer–Reviewer team (planner→implementer→verifier→guardian) self-coordinating via `SendMessage` + `.handoff/loop/` files | evidence: `.claude/skills/weave-orchestrator/SKILL.md:8-21,76-88` | confidence: high
- CLAIM: supporting skills `weave-invariants`, `weave-test-discipline`, `weave-drift-guard`, `weave-loop`, `session-relay` form the shared instruction library the agents draw on | evidence: `.claude/skills/weave-orchestrator/SKILL.md:21` ; `.claude/skills/` listing | confidence: high
- CLAIM: **ungoverned auto-generated instruction surfaces exist and contradict the Rust-native invariant.** `weave-instincts.yaml` (ecc-tools, 2026-06-02) instructs "Use camelCase naming convention" and "Use relative imports" — both FALSE for a snake_case, crate-pathed Rust repo. This is the *same misinformation drift* CLAUDE.md says it removed from the auto-generated `weave` skill on 2026-06-04, but the instinct/identity sidecars were never purged | evidence: `.claude/homunculus/instincts/inherited/weave-instincts.yaml:36-60` ; `CLAUDE.md:15` | confidence: high
- CLAIM: `identity.json` / `ecc-tools.json` are ecc-generated metadata sidecars (`suggestedBy: ecc-tools-repo-analysis`) — instruction-adjacent, generated, not regenerated against current code | evidence: `.claude/identity.json:11-13` ; `.claude/ecc-tools.json:1-6` | confidence: high
- CLAIM: there is **no committed `.claude/settings.json`** — the host hook/permission config is generated at runtime by `weave setup`, not version-controlled in the repo | evidence: `find .claude -name 'settings*.json'` → none ; `weave/src/setup.rs:87` | confidence: high

## 2. Tools granted (weave-mcp = largest tool-grant surface in the fleet)

- CLAIM: weave-mcp dispatches **70 `weave_*` ops** (match arms) with **74 catalog entries**, behind ONE standing meta-tool — the fleet's largest tool grant by op-count | evidence: `weave-mcp/src/mcp.rs:457-540` (70 arms) ; `tool_catalog` `weave-mcp/src/mcp.rs:4108` (74 `"name"` entries) | confidence: high
- CLAIM: progressive disclosure — `tools/list` returns a SINGLE `weave` meta-tool by default; the 70 ops are reached via meta-tool `describe`/`search`/`list`/`call`, not as standing tools | evidence: `weave-mcp/src/mcp.rs:4807-4819` (`tools()` returns `[meta_tool_def()]`) ; `:389,446-447` | confidence: high
- CLAIM: the standing surface is byte-budget-gated at `MAX_STANDING_TOOLS_BYTES = 8192` (≈2k tokens) and test-enforced (`standing_mcp_surface_is_within_token_budget`, `progressive_default_surface_is_just_the_meta_tool`) | evidence: `weave-mcp/src/mcp.rs:224-233,6503-6512` | confidence: high
- CLAIM: eager-flat mode (all ops as standing tools) is opt-in via `WEAVE_MCP_EAGER=1`, explicitly exempt from the budget | evidence: `weave-mcp/src/mcp.rs:4779-4789` ; `CLAUDE.md:119` | confidence: high
- CLAIM: the grant is heavily **mutating** — destructive/state-changing ops include `weave_clear`(scope=all, confirm-gated), `weave_spawn_peer`, `weave_kill_peer`, `weave_daemon_start`/`_stop`, `weave_memory_write`/`_delete`, `weave_set_peer_policy`, `weave_set_message_priority`, plus job/lease/schedule mutation | evidence: `weave-mcp/src/mcp.rs:295-334` (standing-aware list) ; `:483-537` | confidence: high
- CLAIM: **`weave_setup` / `weave_uninstall` are exposed as MCP tools** — i.e. an agent can, over MCP, mutate the host's `~/.claude/settings.json`, register/deregister MCP servers, and merge lifecycle hooks. This is a privileged self-configuration grant inside the tool surface | evidence: `weave-mcp/src/mcp.rs:323-324` ; `weave/src/setup.rs:39-59,193-217` | confidence: high
- CLAIM: `weave_web` grants governed **stealth browser** access (navigate/snapshot/click/extract) via obscura, deny-by-default, SSRF-guarded, optional lease rate-limit + durable audit job (ADR-0002) | evidence: `weave-mcp/src/mcp.rs:4765,5665-5726` | confidence: high
- CLAIM: `weave_ask_permission` grants an **agent-to-agent authorization** primitive — request approval for a peer's mutating tool (Bash/Edit/Write); unanswered asks time out at 300s = denied | evidence: `weave-mcp/src/mcp.rs:4688` | confidence: high
- CLAIM: lease/job grants (`weave_lease_reserve/_release/_list/_sweep`, `weave_job_create/_delegate/_claim/_update/_cancel`, `weave_claim_orchestrator`) are the A2A coordination plane the parallel plan-loop reuses as its lease | evidence: `weave-mcp/src/mcp.rs:500-537` | confidence: high
- CLAIM: Codex lane grants a DIFFERENT, Node.js-backed tool set — 6 npx MCP servers (github, context7, exa-remote, memory, playwright, sequential-thinking) + live web_search, under `sandbox_mode="workspace-write"` | evidence: `.codex/config.toml:4-29` | confidence: high
- CLAIM: meta-tool `call` mode re-applies the safe-HTTP destructive-op gate to the INNER op and refuses to target `weave` itself — the meta-tool is not a gate bypass (test-locked) | evidence: `weave-mcp/src/mcp.rs:4851-4852,4926-4929,6790-6802` | confidence: high

## 3. Model lanes

- CLAIM: **standalone lane = single-model (all Opus).** Orchestrator hard-codes `model: "opus"` for every agent; all four agent defs carry `model: opus` | evidence: `.claude/skills/weave-orchestrator/SKILL.md:10,42,46,50,54` ; `.claude/agents/weave-{planner,implementer,verifier,guardian}.md:5` | confidence: high
- CLAIM: **autonomous loop lane = dual-model / cross-vendor.** Local Opus runs plan→implement→verify→deliver; Phase-4 guardian is delegated to **MiniMax `minimax-m3:cloud`** (non-Anthropic) as the external invariant/drift/docs gate | evidence: `.claude/agents/weave-guardian.md:16` ; `.claude/skills/weave-orchestrator/SKILL.md:56` ; `.claude/skills/weave-loop/SKILL.md:78` | confidence: high
- CLAIM: the loop runner defaults the guardian model to MiniMax (`MODEL="${WEAVE_MODEL:-minimax-m3:cloud}"`) and the local agent to `claude` (`AGENT_CMD`), overridable by env | evidence: `.claude/skills/weave-loop/scripts/ralph-weave.sh:18-21` | confidence: high
- CLAIM: resume hand-off explicitly pins the resumed lane to Opus (`model=opus`) | evidence: `.claude/skills/session-relay/SKILL.md:33` | confidence: high
- CLAIM: no-downgrade / fail-closed routing — guardian BLOCK wins over verifier GREEN; "never ship RED or BLOCK"; agent retry-once-then-proceed-with-note | evidence: `.claude/skills/weave-orchestrator/SKILL.md:92-95` | confidence: high
- CLAIM: weave is the TRANSPORT the dual-model strategy uses — MiniMax is spawned via the configured guardian command and writes its verdict into the shared `.handoff/loop/` ledger that weave-transported sessions read; weave itself carries no model-routing logic (consistent with transport-not-interpreter) | evidence: `.claude/skills/weave-loop/scripts/ralph-weave.sh:1-11` ; `.claude/skills/weave-orchestrator/SKILL.md:56-62` | confidence: medium

## 4. Hidden architectural couplings

- CLAIM: `weave setup` couples weave to the **host filesystem + multi-provider config plane** — writes `~/.claude/settings.json` (Claude, confirmed), `~/.codex/config.toml` `notify` key (Codex, partial), Gemini/Aider scaffolds (unconfirmed) — implying cross-provider install infrastructure, not just a binary | evidence: `weave/src/setup.rs:39-59,87` | confidence: high
- CLAIM: `resolve_setup_exe` pins a STABLE binary path (never an ephemeral `target/.worktrees` build) — a hidden infra requirement: the installed hook must point to a durable on-disk binary or the global MCP+hooks dangle | evidence: `weave/src/setup.rs:104,168` ; `CLAUDE.md:92` | confidence: high
- CLAIM: the SQLite DB file *is* the broker — "no-daemon push"; peers/leases/jobs/asks/reads are all DB tables; the tool grant implies durable local state + per-reader read tracking, not transient memory | evidence: `CLAUDE.md:22,106-110` | confidence: high
- CLAIM: dual mutually-exclusive backends (`sqlite` default / `libsql` Turso) — a `compile_error!` guards enabling both; any `Store` change implies dual-backend infra + a remote (Turso) networking surface behind `libsql` | evidence: `CLAUDE.md:74-82,135` | confidence: high
- CLAIM: native injector implies process-spawn infra into other terminal muxes (tmux/zellij/kitty/wezterm/screen) — "no shell ever", explicit argv only; the prompt-level no-shell invariant is load-bearing security architecture | evidence: `CLAUDE.md:114` ; `:97,110` | confidence: high
- CLAIM: `.codex/config.toml` couples the Codex lane to **Node.js/npx runtime** (6 `npx -y …` servers) + a remote Exa URL — a non-Rust tooling dependency at the agent layer, in tension with the Rust-native invariant (sidecar metadata, not part of the shippable binary build, so CLAUDE.md tolerates it — but it is an ungoverned external-runtime coupling) | evidence: `.codex/config.toml:8-29` ; `CLAUDE.md:53-57` | confidence: medium
- CLAIM: PreToolUse gate couples authorization to weave's own message bus — a "dangerous tool" decision is resolved by routing a ToolPermission *ask* to a peer over the same mailbox weave provides (self-referential governance substrate) | evidence: `weave/src/main.rs:8857-9008` | confidence: high

## 5. Governance controls

- CLAIM: **PreToolUse enforcing approval gate (WL-055) is deny-by-default and OPT-IN.** Installed only with `weave setup --pretooluse` (Claude-only); matcher `Bash|Edit|Write`. Without an approver → DENY; broadcast approver → DENY; no approval within timeout → DENY | evidence: `weave/src/main.rs:201-206,8826-8919,9008` ; `weave/src/setup.rs:181-217` | confidence: high
- CLAIM: default leaves PreToolUse uninstalled "so the gate never surprise-blocks a session" — a deliberate human-ergonomics-vs-safety tradeoff (governance is off unless explicitly armed) | evidence: `weave/src/setup.rs:196` | confidence: high
- CLAIM: destructive-op gates — `weave_clear{scope:all}` requires `confirm:true`; `weave_web` deny-by-default + SSRF + lease/audit; input caps (`MAX_BODY` 65536, `MAX_INJECT_CHARS` 240, `MAX_IDENT_LEN`, `id_valid`) | evidence: `CLAUDE.md:117-118` ; `weave-mcp/src/mcp.rs:4229,4765` | confidence: high
- CLAIM: standing-token budget is a governance control (drift guard test) treating tool-table bloat as the same species of risk as a heavyweight dependency | evidence: `CLAUDE.md:119` ; `weave-mcp/src/mcp.rs:224-233` | confidence: high
- CLAIM: artifact-gate governance — every phase writes a numbered `.handoff/loop/0N_*.md`; guardian APPROVE/BLOCK is the merge gate; loop delivery writes `.handoff/loop/NEEDS-HUMAN` + halts on failure | evidence: `.claude/skills/weave-orchestrator/SKILL.md:42-74` | confidence: high
- CLAIM: six required CI checks (`rustfmt`, `clippy`, `test`, `build (libsql)`, `sign`, `libsql+sign`) gate auto-merge; `sync-master` workflow enforces no-downgrade ancestor guard `develop`→`master` | evidence: `CLAUDE.md:43-44` | confidence: high
- CLAIM: Codex lane governance = `approval_policy="on-request"` + `sandbox_mode="workspace-write"` + `max_threads=6, max_depth=1` — a separate, weaker-by-default control plane than Claude's gated path | evidence: `.codex/config.toml:4-5,34-36` | confidence: high
- CLAIM: governance GAP — the ecc-generated instinct/identity sidecars are not on any drift-guard gate; `weave-drift-guard` targets build/runtime Rust intrusions, not stale instruction misinformation, so the camelCase/relative-import falsehoods persist ungoverned | evidence: `CLAUDE.md:51-57` (drift scan scopes build/runtime) ; `.claude/homunculus/instincts/inherited/weave-instincts.yaml:36-60` | confidence: medium

## 6. ADR candidates / no-ADR rationale

Existing ADRs (do not duplicate): ADR-0001 handoff kernel, ADR-0002 obscura web, ADR-0003 token-light
multi-surface, ADR-0004 Rust-native human surfaces, ADR-0005 cross-machine push
(`.handoff/decisions/`).

- ADR-CANDIDATE: Cross-vendor model lane — delegating the Phase-4 invariant/drift/docs guardian to a non-Anthropic model (MiniMax `minimax-m3:cloud`) in autonomous loop mode | reason: a different vendor's model is the final correctness/security gate before auto-merge; trust boundary, fallback, and availability of that lane are architectural and currently ADR-uncovered (the existing ADRs cover surfaces/transport, not model routing).
- ADR-CANDIDATE: PreToolUse peer-as-approver authorization plane (WL-055) — deny-by-default, opt-in, routing dangerous-tool approval through weave's own mailbox | reason: it makes weave a security control plane (A2A authorization), not just transport; the opt-in/default-off posture, 300s-timeout-as-deny, and "no committed settings.json" combine into a governance decision worth recording.
- ADR-CANDIDATE: `weave_setup`/`weave_uninstall` exposed over MCP as a privileged host-config mutation grant | reason: an agent can rewrite host `~/.claude/settings.json` and (de)register MCP servers via a tool call — a self-modifying-infrastructure capability whose blast radius (and why it is in the tool table vs. CLI-only) should be an explicit decision.
- ADR-CANDIDATE: Governance of ecc-tools-generated instruction sidecars (instincts/identity/ecc-tools.json) — bring them under the drift guard or freeze/remove them | reason: an ungoverned generated instruction surface actively contradicts the repo's headline Rust-native invariant (camelCase/relative-imports for a snake_case Rust repo); the same drift class was already remediated once for the `weave` skill (CLAUDE.md:15) but not for these — a standing decision on their lifecycle is architectural.
- NO-ADR: progressive-disclosure meta-tool + standing-token budget | reason: already governed by ADR-0003 (token-light multi-surface).
- NO-ADR: obscura `weave_web` stealth-browser grant | reason: already governed by ADR-0002.
- NO-ADR: Rust-native invariant / human surfaces | reason: already governed by ADR-0004; drift procedure lives in CLAUDE.md.
- NO-ADR: `model: "opus"` standalone single-model lane | reason: routine default, no backend/vendor coupling beyond the host; only the dual-model loop lane (above) is architectural.
- NO-ADR: `.handoff/loop/` numbered artifact protocol | reason: routine harness state convention under the already-adopted handoff kernel (ADR-0001).

## Upgrade rows

- UPGRADE: Bring ecc-generated sidecars (`weave-instincts.yaml`, `identity.json`, `ecc-tools.json`) under the session-start drift guard, or delete the stale misinformation | axis: prompt-architecture | rationale: an instruction surface telling agents to use camelCase + relative imports directly contradicts the Rust-native invariant and the prior skill-drift fix | evidence: `.claude/homunculus/instincts/inherited/weave-instincts.yaml:36-60` ; `CLAUDE.md:15,51-57` | blast: instruction-only (no code/build) | risk: low
- UPGRADE: Document the dual-model loop lane (MiniMax guardian) as an ADR + state fallback when the MiniMax lane is unavailable | axis: prompt-architecture | rationale: a non-Anthropic model is the pre-auto-merge gate; availability/fallback is currently implicit (`WEAVE_SKIP_GUARDIAN=1` disables the gate entirely) | evidence: `.claude/skills/weave-loop/scripts/ralph-weave.sh:18-21,38-40` ; `.claude/agents/weave-guardian.md:16` | blast: loop-delivery governance | risk: med
- UPGRADE: Record the `weave_setup`-over-MCP + PreToolUse-peer-approver governance posture (opt-in, deny-by-default, default-off) so the "off unless armed" tradeoff is an auditable decision | axis: prompt-architecture | rationale: host-config mutation + the only A2A authorization gate are currently default-disabled and uncommitted | evidence: `weave/src/setup.rs:193-217` ; `weave/src/main.rs:8857-9008` | blast: host config + A2A auth | risk: med

---

### Top findings (for parent)
1. **Largest tool grant in the fleet is token-safe by design:** 70 `weave_*` ops / 74 catalog entries collapse to ONE standing `weave` meta-tool (progressive disclosure), byte-budget-gated + test-locked; meta-tool `call` re-applies the destructive-op gate so it is not a bypass (`weave-mcp/src/mcp.rs:457-540,4807-4819,4926-4929`).
2. **Dual-model / cross-vendor lane is ADR-uncovered:** standalone is all-Opus, but the autonomous loop delegates the final invariant/drift guardian to **MiniMax `minimax-m3:cloud`** (`ralph-weave.sh:18-21`; `weave-guardian.md:16`). A non-Anthropic model gates auto-merge — top ADR candidate.
3. **weave is a security/authorization control plane, not just transport:** PreToolUse gate (deny-by-default, opt-in) + `weave_ask_permission` route dangerous-tool approval through weave's own mailbox; `weave_setup` over MCP can rewrite host `~/.claude/settings.json` (`main.rs:8857-9008`; `setup.rs:193-217`).
4. **Active ungoverned instruction drift:** ecc-generated `weave-instincts.yaml` tells agents to use camelCase + relative imports — false for this Rust repo and the exact drift class CLAUDE.md already fixed once for the auto-generated skill, but never for the instinct/identity sidecars (`weave-instincts.yaml:36-60`; `CLAUDE.md:15`).
5. **Transport-not-interpreter holds:** weave carries no intent-interpretation or model-routing logic; it moves messages/jobs/leases/approvals and lets MiniMax/Opus write verdicts into the shared `.handoff/loop/` ledger — consistent with the north-star split (harness_hub interprets, weave transports).

Findings file: `/home/drdave/Desktop/meta/.worktrees/plan-weave/envctl/.handoff/loop/plan/findings/prompt-architecture-weave.md`
