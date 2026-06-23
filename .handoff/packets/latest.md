# Handoff Packet (latest) — handoff.packet.v2

## 1. North Star
envctl owns and contains the meta environment: every FlexNetOS tool/dotfile/.local/bin resolves inside meta; user-global ($HOME/.local, ~/.claude) holds ONLY symlinks into meta; envctl exports META_ROOT (resolved from the .meta.yaml marker, like meta_core's META_DATA_DIR) so no config hardcodes paths; secrets are held and auto-injected. Heal not harm; never downgrade; never delete (archive).

## 2. State Precedence
Git > .handoff/ledger.db > tasks/*.task.json > active.md > this packet.

## 3. Progress
Done: 51/54.  Tamper-evident events verified: 73.

## 0. Next Action / Direction
- **Next safe task:** TASK-0053 — Route verified GitHub transport doctrine into envctl
- **Next command:** `hf checkpoint TASK-0053`
- **Why it is next:** resume the in-progress task (status Claimed) before starting any new work.
- **Cycle / context budget:** context — wrap at ~50% of the context window (cycle_flush=4 caps a runaway cycle); this session is at cycle 0/4.
- **Ready to ship:** no (`hf ship` once the cycle is full / context budget hit).
- **Blocking walls:** TASK-0002 (blocked_by TASK-0001) · TASK-0003 (blocked_by TASK-0002) · TASK-0009 (status Blocked; blocked_by TASK-0018) · TASK-0013 (blocked_by TASK-0012) · TASK-0014 (blocked_by TASK-0012, TASK-0013) · TASK-0015 (blocked_by TASK-0012) · TASK-0016 (blocked_by TASK-0012) · TASK-0017 (blocked_by TASK-0012) · TASK-0018 (blocked_by TASK-0012, TASK-0013, TASK-0014) · TASK-0024 (blocked_by TASK-0002) · TASK-0026 (blocked_by TASK-0020) · TASK-0027 (blocked_by TASK-0020) · TASK-0028 (blocked_by TASK-0020) · TASK-0031-PR2 (blocked_by TASK-0031) · TASK-0031-PR2C (blocked_by TASK-0031-PR2) · TASK-0031 (blocked_by TASK-0030) · TASK-0032 (blocked_by TASK-0031) · TASK-0033 (status Blocked) · TASK-0038 (blocked_by TASK-0035) · TASK-0039 (blocked_by TASK-0031-PR2) · TASK-0044 (blocked_by TASK-0001, TASK-0002, TASK-0003) · TASK-0047 (blocked_by TASK-0046; NEEDS-HUMAN) · TASK-0050 (blocked_by TASK-0049) · TASK-0051 (blocked_by TASK-0043) · TASK-0052 (blocked_by TASK-0044)

## 4. Remaining (next safe first)
- [P2] **TASK-0009** — Relocate kasetto + kst (BLOCKED: superseded by Epic C built-in absorption)
- [P1] **TASK-0033** — VPS Profile B (BLOCKED owner-gated): F7 install gate + F8/OI-SM-2 authorizer + OI-SM-3 trusted-time
- [P0] **TASK-0053** — Route verified GitHub transport doctrine into envctl

## 5. Next Best Task
**TASK-0053** — Route verified GitHub transport doctrine into envctl
  objective: # Overview

Capture and route the verified meta GitHub transport and automation doctrine into envctl so envctl can implement the missing credential/merge-gate pieces without relying on stale assumptions or raw GitHub API output.

Deep research started from `meta/.kb/AGENTS.md` and loaded the meta KB/context. The proof is source-grounded: `.meta.yaml`, live git remotes, `.github_org` architecture docs/TODO, `handoff` ADR/source, `flexnetos_github_app` source, and live `gh`/SSH checks.

## Goals

- Make envctl aware that local `git` over SSH is the repository source of truth for FlexNetOS repos.
- Keep `gh` CLI/API as GitHub workflow orchestration, but require re-query/cross-checks against git refs, PR state, and required checks before trusting mutations.
- Route the missing envctl-owned GitHub credential work into the existing envctl handoff loop, especially scoped GitHub App token mint/injection and policy-drift token provisioning.
- Preserve the fail-closed model: agents do not hold broad merge tokens, do not native-APPROVE their own PRs, and do not force-merge red checks.

## Acceptance Criteria

- [ ] envctl backlog/task docs include the GitHub transport doctrine: SSH git is repo truth; `gh` is orchestration; raw API/connector output is advisory until re-queried.
- [ ] envctl exposes/validates the scoped GitHub App token path needed by downstream consumers: `secretctl mint-github --installation-id 140063898 --output json` and related enroll/revoke flows remain byte-stable.
- [ ] envctl has a concrete owner path for `POLICY_DRIFT_TOKEN` / app-minted equivalent so `.github` policy drift can read branch protection, rulesets, environments, and repo settings in strict mode.
- [ ] Any envctl implementation keeps tokens broker-only/scoped/short-lived and never logs secrets.
- [ ] Integration proof cross-checks `flexnetos_github_app` consumer expectations, especially merge-gate check-run writer expectations.
- [ ] Verification uses SSH-backed git refs plus `gh` re-query; no raw API mutation is treated as success without read-back.
- [ ] Handoff continuity is exported/committed using the current redb-backed ledger plus deterministic JSONL export; do not describe this as SQLite.

## Context / Proof

- `meta/.kb/AGENTS.md` requires KB/context-first operation and says the document is the plan.
- `.meta.yaml` currently configures 66/66 project repos as `git@github.com:FlexNetOS/...` SSH URLs.
- Live sample origins for `meta`, `.github_org`, `envctl`, `meta-ruvector`, `rusty-idd`, `weave`, `handoff`, and `flexnetos_github_app` are SSH.
- `git ls-remote --symref origin HEAD` from meta succeeds over SSH.
- `gh auth status` is logged in, but `gh config get git_protocol` reports `https`, so `gh` must not be treated as the git transport source of truth.
- `.github_org/TODO.md` records that default `GITHUB_TOKEN` cannot read branch protection, rulesets, or repo settings; strict policy drift needs a provisioned token from envctl.
- `.kb/store/documents/tasks/github-local-model-pivot.md` records cloud-token burn from automatic Claude review flows and the requirement to move GitHub automation to local model / opt-in review.
- `.kb/store/documents/incidents/release-please-token-unavailable.md` records that `GITHUB_TOKEN`-created PRs do not trigger CI, so release PRs cannot pass required checks/auto-merge until the proper org secret/token path is granted.
- `.github_org/architecture/map/01-meta-control-plane.md` records that `gh` mutations can silently succeed and must be re-queried; it also records GitHub auto-merge/API edge cases.
- `.github_org/architecture/plan/2026-06-17-deep-review-upgrade-plan.md` records a concrete policy-applier hazard: `gh repo view` resolving from the wrong CWD can mutate the wrong repo unless owner/repo is asserted.
- `flexnetos_github_app/crates/app-core/src/merge_gate.rs` says the App should post a verdict as a required GitHub check-run and arm native auto-merge only after green; it must never be a native bot APPROVE, and the current `UnwiredMergeGate` fails closed.
- `handoff` source/ADRs record the out-of-band review verdict model: judgment is recorded in handoff/weave state and enforced via required check/merge gate, not by bot approving the PR.

## Envctl Scope

Primary envctl areas:

- `.handoff/loop/backlog.md` and relevant task cards for GitHub App mint/enroll/revoke/token provisioning.
- `crates/secretd`, `crates/secretctl`, `crates/secrets-engine` GitHub App provider mint path.
- Any envctl agent/environment injection surfaces that provide short-lived GitHub tokens to `gh`/workflow automation.

Consumer cross-checks:

- `../flexnetos_github_app/crates/app-core/src/mint.rs`
- `../flexnetos_github_app/crates/app-core/src/merge_gate.rs`
- `.github_org` policy drift scripts and workflows.

## Notes

This is not a request to avoid the GitHub API entirely. It is a requirement to use it through controlled `gh`/App paths with explicit owner/repo selection, least privilege, read-back verification, and SSH git as the repository truth.


## 6. Resume Commands
```bash
hf resume
hf claim TASK-0053
```

## 7. Machine Summary
```json
{
  "done": [
    "TASK-0001",
    "TASK-0002",
    "TASK-0003",
    "TASK-0004",
    "TASK-0005",
    "TASK-0006",
    "TASK-0007",
    "TASK-0008",
    "TASK-0010",
    "TASK-0011",
    "TASK-0012",
    "TASK-0013",
    "TASK-0014",
    "TASK-0015",
    "TASK-0016",
    "TASK-0017",
    "TASK-0018",
    "TASK-0019",
    "TASK-0020",
    "TASK-0021",
    "TASK-0022",
    "TASK-0023",
    "TASK-0024",
    "TASK-0025",
    "TASK-0026",
    "TASK-0027",
    "TASK-0028",
    "TASK-0029",
    "TASK-0030",
    "TASK-0031-PR2",
    "TASK-0031-PR2C",
    "TASK-0031",
    "TASK-0032",
    "TASK-0034",
    "TASK-0035",
    "TASK-0036",
    "TASK-0037",
    "TASK-0038",
    "TASK-0039",
    "TASK-0041",
    "TASK-0042",
    "TASK-0043",
    "TASK-0044",
    "TASK-0045",
    "TASK-0046",
    "TASK-0047",
    "TASK-0048",
    "TASK-0049",
    "TASK-0050",
    "TASK-0051",
    "TASK-0052"
  ],
  "next_command": "hf claim TASK-0053",
  "next_task_id": "TASK-0053",
  "project": "handoff (Continuity Ledger Kernel)",
  "remaining": [
    "TASK-0009",
    "TASK-0033",
    "TASK-0053"
  ],
  "schema": "handoff.packet.v2",
  "tasks_total": 54,
  "witnessed_events_verified": 73
}
```

## Contract Proof (ADR-0011 — ruvector-verified/Lean)
Active task **TASK-0053** — AgentContract PROVEN via ruvector-verified (3 obligation(s)).
- ✓ `intent:objective` (Eq.refl proof-term #0)
- ✓ `intent:path_scope` (Eq.refl proof-term #1)
- ✓ `intent:acceptance` (Eq.refl proof-term #2)
3 proof-term(s) · proof-hash `4fae6edd4fe50dc5` · binding `0xae6da17cbec6ac55` · verifier `0x00010000` (lean-agentic 0.1.0).
