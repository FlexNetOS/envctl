# Architecture Plan — Meta Fleet Agent-Environment Convergence

## Verdict

GO with a mandatory narrow Meta/GitKB baseline. The scope is parent Meta plus
the 28 `.meta.yaml` projects whose declared path begins with `src/`; it is not
filesystem discovery of every directory below `src/`.

## Ownership model

- Meta owns fleet selection, policy, review evidence, and execution through
  `meta project list --json` and `meta exec --include … -- <command>`.
- Each peer remains an independent Git repository and owns its committed
  `agent-env.yaml` and `agent-env.lock`.
- Envctl remains the per-project agent-environment engine.
- Yazelix remains the only owner of the real-home Nix profile and Nu/RTK
  runtime. Agent-env may attest that contract, never mutate it.

## Verified constraints

- `meta git review` currently passes through to `git review`, which is absent;
  it is not a fleet review executor.
- Project-scope agent sync is rooted at the selected config's parent. Using
  envctl's config while standing in another repo writes envctl outputs, not the
  target peer.
- Envctl's existing `doctor` is CWD/lock based and cannot accept a config or
  prove destination content hashes; it is insufficient as a fleet convergence
  gate.
- Config inheritance through `../envctl` is non-portable. Remote inheritance
  is unsuitable for a strict zero-network audit.

## Policy payload and skill adoption

- Adopt a new narrow Meta/GitKB review-policy skill for every selected
  participant.
- Do not copy envctl-only `agent-env-codex`, `agent-env-config`,
  `env-stabilize`, or `env-toolchain-install` into peers.
- Keep `codedb-config-tables` conditional on a verified profile-owned CodeDB
  frontdoor and repo need.
- Keep Exa conditional on a per-assistant MCP-owner inventory.
- Preserve each repo's native Claude/Codex/GitKB adapters; do not replace them
  with a Claude-shaped mirror.

## Delivery sequence

1. Add Meta fleet policy/config templates and an inventory/plan command that
   selects root + declared `src/*` peers from `.meta.yaml`.
2. Add config-aware, locked envctl audit support that validates
   config → lock → installed skill hashes and reports MCP ownership conflicts.
3. Add the Meta CI/runbook gate and repair stale agent-env runbook command and
   shell guidance.
4. Generate a reviewed config+lock in a worktree for each participant, then
   execute preview before `--apply` through the Meta frontdoor.
5. Require all selected participants to pass locked audit, lock check, native
   adapter conflict check, and RTK/Yazelix preflight before merge.

## Required verification

- Envctl unit/integration coverage for config-aware audit and unsupported
  configs.
- `ci/gates/agent-env.sh`, `ci/gates/meta-local-policy.sh`, and
  `ci/gates/yazelix-codex-runtime.sh`.
- Meta fleet inventory and dry-run tests; all 29 participant results recorded.
- RTK preflight uses `/home/flexnetos/.nix-profile/toolbin/nu` and
  `/home/flexnetos/.nix-profile/bin/rtk` 0.43.0.
