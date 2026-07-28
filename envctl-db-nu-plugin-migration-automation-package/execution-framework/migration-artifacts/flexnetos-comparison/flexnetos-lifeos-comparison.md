# FlexNetOS vs lifeos comparison evidence

- Task: `REQ-201_FLEXNETOS_LIFEOS_COMPARISON`
- Observed: `2026-07-28`
- Method: read-only filesystem and Git metadata inspection; no network, processes, or secret-path contents were read.
- Scope: `/home/flexnetos/FlexNetOS` and `/home/flexnetos/lifeos`.

## Conclusion

FlexNetOS was the active **automation, execution, release, and provenance plane**. It hosted two self-hosted GitHub Actions runner work slots and the independently versioned `envctl`, `nu_plugin`, and `flexnetos_runner` checkouts used by that automation.

The present `/home/flexnetos/lifeos` directory is not a LifeOS application checkout or service. It is a small (292 KB) residual namespace containing only five `envctl` Codex-harness state/ledger files. It does not contain a Git repository, executable application source, package manifest, or runner configuration. Therefore, the evidence supports treating `lifeos` here as retained agent-runtime state, not as the operational replacement for FlexNetOS.

## Current-state comparison

| Dimension | FlexNetOS | lifeos |
|---|---|---|
| Root | `/home/flexnetos/FlexNetOS` | `/home/flexnetos/lifeos` |
| Observed size | 992 MB | 292 KB |
| Top-level role | Runner/release workspace | Residual harness state namespace |
| Git status | Root deliberately is not a Git repo; peer checkouts are versioned | No Git repository |
| Durable contents | Two Actions work slots, runner configuration, peer checkouts | Four JSONL ledgers and one model-router state JSON |
| Application/package surface | `envctl`, `nu_plugin`, `flexnetos_runner` source checkouts | None observed |

## Evidence

### FlexNetOS execution role

- The runner README calls `flexnetos_runner` the “execution plane” for GitHub-to-local automation. It describes a self-hosted GitHub Actions runner and a dispatcher that routes build/test, agent-task/review, loop-cycle, lease, and worktree jobs to existing kernels.
- Its documented operation is an organization-scoped FlexNetOS runner with labels `self-hosted,linux,x64,local,flexnetos`; the checked workspace contains both `actions-runner-01-work` and `actions-runner-02-work`.
- Runner marker files exist for both `actions-runner-01` and `actions-runner-02`, including `.runner` and `.service` markers in the runner repository’s operations tree.
- The two work slots each contain checkouts of `envctl`, `nu_plugin`, and `flexnetos_runner`. The slot-01 remotes identify `https://github.com/FlexNetOS/envctl` and `https://github.com/FlexNetOS/nu_plugin`.
- `envctl`’s FlexNetOS boundary rule says the workspace root is a deliberately hollow orchestration area and identifies `envctl` as environment authority, `meta` as workspace/fleet authority, and Yazelix/Nix as toolchain owner.
- The release script resolves its default workspace root to `/home/flexnetos/FlexNetOS`, writes release output under that root, and uses runner-local writable state. This is direct evidence of a local release lane, not merely an archive.

### lifeos residual-state role

The only regular files currently found below `/home/flexnetos/lifeos` are:

- `src/envctl/home/agent-env/codex-harness/ledger/counters.jsonl`
- `src/envctl/home/agent-env/codex-harness/ledger/decisions.jsonl`
- `src/envctl/home/agent-env/codex-harness/ledger/harness.jsonl`
- `src/envctl/home/agent-env/codex-harness/ledger/model_router.jsonl`
- `src/envctl/home/agent-env/codex-harness/state/model-router/last-route.json`

These names and locations indicate persisted Codex-harness telemetry/selection state owned by the `envctl` home projection. They are not evidence of a LifeOS executable, UI, database, deployed service, or release pipeline.

## Relationship to supplied dependency artifacts

- The repository map and service-dependency graph describe the FlexNetOS filesystem as a control/runner environment, with `envctl`, control-plane, and runner relationships.
- The debug code map has primary hotspots in the slot-02 `envctl` checkout, including CI gates and CLI/engine code. That aligns with the active runner workspace conclusion.
- The prior comparison artifact (generated July 4) described a then-populated `FlexNetOS/src/lifeos` peer application. That peer is absent from the current target filesystem, so those historical claims are not used as evidence of present state.

## Limits and handling recommendation

This is a filesystem-state conclusion, not a claim that the historical LifeOS product never existed. If a prior `src/lifeos` checkout must be recovered, use Git/removable-storage history separately. For migration planning, preserve `/home/flexnetos/lifeos` as agent-runtime state until its consuming `envctl` projection has been identified and migrated; do not classify it as the LifeOS product workload.

## Evidence paths

- `src/flexnetos_runner/_work/actions-runner-01-work/flexnetos_runner/flexnetos_runner/README.md`
- `src/flexnetos_runner/_work/actions-runner-01-work/flexnetos_runner/flexnetos_runner/docs/portable-runner-bridge.md`
- `src/flexnetos_runner/_work/actions-runner-01-work/flexnetos_runner/flexnetos_runner/scripts/build-local-ubuntu-release.sh`
- `src/flexnetos_runner/_work/actions-runner-01-work/envctl/envctl/home/.claude/rules/flexnetos-boundaries.md`
- `migration-artifacts/art-102_repository_map/repository-map.md`
- `migration-artifacts/art-103_service_dep_graph/service-dependency-graph.md`
- `migration-artifacts/art-113_debug_code_map/debug-code-map.md`
