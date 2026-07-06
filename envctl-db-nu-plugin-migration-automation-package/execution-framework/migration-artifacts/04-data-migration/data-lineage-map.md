# Data Lineage Map

Task: `ART-109_DATA_LINEAGE`
Generated at: `2026-07-04T23:29:00+00:00`
Target root: `/home/flexnetos/FlexNetOS`

## Scope

This map traces critical migration fields from their schema or database origin, through static transformation evidence, to consuming protocol records, registry views, plugin-facing records, and proof artifacts. Blocked secret and private-key paths are excluded from the scan.

## Summary

| measure | value |
|---|---:|
| field count | 183 |
| critical field count | 27 |
| sql table count | 16 |
| schema source count | 37 |
| target files scanned | 2500 |
| fields with target references | 14 |
| protocol record count | 14 |

## Critical Field Lineage

| field | origin | transformation | consumption | target evidence |
|---|---|---|---|---:|
| `artifact_contract_id` | envctl_migration_recipes, envctl_migration_runs, MigrationRun | stable identifier propagation through run, operation, artifact, evidence, and graph-edge records; database persistence requires the field before record insertion | registry/proof consumers only | 0 |
| `artifact_id` | envctl_migration_artifacts, envctl_migration_validations, envctl Migration Artifact Record, envctl Migration Validation Result, ArtifactRecord | stable identifier propagation through run, operation, artifact, evidence, and graph-edge records; database persistence requires the field before record insertion | registry/proof consumers only | 0 |
| `command_hash` | envctl_migration_operations | hash derivation and comparison through sha256 file/content digests | registry/proof consumers only | 0 |
| `compare_root` | envctl_migration_targets, envctl Migration Target Descriptor, TargetDescriptor | static lineage only; no transformation-specific reference found | registry/proof consumers only | 0 |
| `content_hash` | envctl_migration_artifacts, envctl Migration Artifact Record, ArtifactRecord | hash derivation and comparison through sha256 file/content digests; static target scan found producer/update/write references | registry/proof consumers only | 6 |
| `contract_hash` | envctl_migration_artifact_contracts | hash derivation and comparison through sha256 file/content digests; database persistence requires the field before record insertion | registry/proof consumers only | 0 |
| `contract_id` | static target evidence | stable identifier propagation through run, operation, artifact, evidence, and graph-edge records | registry/proof consumers only | 0 |
| `descriptor_hash` | envctl_migration_targets | hash derivation and comparison through sha256 file/content digests; database persistence requires the field before record insertion | registry/proof consumers only | 0 |
| `event_hash` | envctl_migration_run_events, envctl Migration Run Event, RunEvent | hash derivation and comparison through sha256 file/content digests | registry/proof consumers only | 0 |
| `evidence_refs` | envctl Migration Artifact Record, envctl Migration Run Event, envctl Migration Validation Result | artifact evidence and graph-link materialization in proof and registry payloads | registry/proof consumers only | 2 |
| `id` | envctl_migration_targets, envctl_migration_packages, envctl_migration_artifact_contracts, pathRule, workspace | stable identifier propagation through run, operation, artifact, evidence, and graph-edge records; database persistence requires the field before record insertion | EvidenceRecord->nu_plugin, ValidationResult->nu_plugin | 18 |
| `links` | envctl Migration Artifact Record, ArtifactRecord | artifact evidence and graph-link materialization in proof and registry payloads; static target scan found producer/update/write references | registry/proof consumers only | 18 |
| `operation_id` | envctl_migration_run_events, envctl_migration_evidence, envctl_migration_approvals, envctl Migration Approval Request, operation, envctl Migration Operation | stable identifier propagation through run, operation, artifact, evidence, and graph-edge records; database persistence requires the field before record insertion | registry/proof consumers only | 0 |
| `package_hash` | envctl_migration_packages | hash derivation and comparison through sha256 file/content digests; database persistence requires the field before record insertion | registry/proof consumers only | 0 |
| `path` | envctl_migration_artifacts, envctl Migration Artifact Record, workspace, ArtifactRecord | static lineage only; no transformation-specific reference found | registry/proof consumers only | 18 |
| `previous_event_hash` | envctl_migration_run_events, envctl Migration Run Event, RunEvent | hash derivation and comparison through sha256 file/content digests | registry/proof consumers only | 0 |
| `primary_root` | envctl_migration_targets, envctl Migration Target Descriptor, TargetDescriptor | database persistence requires the field before record insertion | registry/proof consumers only | 0 |
| `proof_uri` | AgentLane, ExecutionPacket, TaskGraphRow | artifact evidence and graph-link materialization in proof and registry payloads | registry/proof consumers only | 0 |
| `recipe_hash` | envctl_migration_recipes | hash derivation and comparison through sha256 file/content digests; database persistence requires the field before record insertion | registry/proof consumers only | 0 |
| `recipe_id` | envctl_migration_runs, envctl Migration Recipe, MigrationRecipe, MigrationRun | stable identifier propagation through run, operation, artifact, evidence, and graph-edge records; database persistence requires the field before record insertion | registry/proof consumers only | 0 |
| `reproducibility_hash` | envctl_migration_runs, MigrationRun | hash derivation and comparison through sha256 file/content digests | registry/proof consumers only | 0 |
| `risk` | envctl_migration_operations, envctl_migration_approvals, envctl Migration Approval Request, operation, envctl Migration Operation | database persistence requires the field before record insertion | registry/proof consumers only | 18 |
| `run_id` | envctl_migration_operations, envctl_migration_run_events, envctl_migration_evidence, envctl Migration Approval Request, envctl Migration Artifact Record, envctl Migration Operation | stable identifier propagation through run, operation, artifact, evidence, and graph-edge records; database persistence requires the field before record insertion | registry/proof consumers only | 15 |
| `sha256` | envctl_migration_evidence, EvidenceRecord | hash derivation and comparison through sha256 file/content digests | registry/proof consumers only | 18 |
| `status` | envctl_migration_runs, envctl_migration_operations, envctl_migration_artifacts, envctl Migration Approval Request, envctl Migration Artifact Record, envctl Migration Operation | state normalization through enum/check constraints and validation status records; database persistence requires the field before record insertion | MigrationRun->nu_plugin | 18 |
| `target_id` | envctl_migration_targets, envctl_migration_runs, envctl Migration Target Descriptor, TargetDescriptor, MigrationRun | stable identifier propagation through run, operation, artifact, evidence, and graph-edge records; static target scan found producer/update/write references | registry/proof consumers only | 18 |
| `validator` | envctl_migration_validations, envctl Migration Validation Result, ValidationResult | database persistence requires the field before record insertion | registry/proof consumers only | 18 |

## Origin And Consumption Details

### `artifact_contract_id`

- Criticality: `critical`
- SQL origin: `envctl_migration_recipes.artifact_contract_id`, `envctl_migration_runs.artifact_contract_id`
- Schema origin: `MigrationRun`
- Transformation: stable identifier propagation through run, operation, artifact, evidence, and graph-edge records; database persistence requires the field before record insertion

### `artifact_id`

- Criticality: `critical`
- SQL origin: `envctl_migration_artifacts.artifact_id`, `envctl_migration_validations.artifact_id`
- Schema origin: `envctl Migration Artifact Record`, `envctl Migration Validation Result`, `ArtifactRecord`, `ValidationResult`
- Transformation: stable identifier propagation through run, operation, artifact, evidence, and graph-edge records; database persistence requires the field before record insertion

### `command_hash`

- Criticality: `critical`
- SQL origin: `envctl_migration_operations.command_hash`
- Transformation: hash derivation and comparison through sha256 file/content digests

### `compare_root`

- Criticality: `critical`
- SQL origin: `envctl_migration_targets.compare_root`
- Schema origin: `envctl Migration Target Descriptor`, `TargetDescriptor`
- Transformation: static lineage only; no transformation-specific reference found

### `content_hash`

- Criticality: `critical`
- SQL origin: `envctl_migration_artifacts.content_hash`
- Schema origin: `envctl Migration Artifact Record`, `ArtifactRecord`
- Transformation: hash derivation and comparison through sha256 file/content digests; static target scan found producer/update/write references

| file | line | role | evidence |
|---|---:|---|---|
| `LOCAL_WORKAROUNDS.md` | 397 | `reference` | Only its stored commit `content_hash` was changed, to match the full hash |
| `WORKLOG.md` | 701 | `reference` | Root cause was a corrupt stored `content_hash` in the latest Meta KB commit: |
| `src/teri/.kb/store/commits/019f2513-a1ca-78d1-be48-e73c238fec21.json` | 2 | `transformation` | "commit": {"author":"drdave <flexnetos@de-flex.net>","changes":[{"change_type":"modified","content_hash":"bde72726786c614ddfcf9e13417b3f...","doc_type":"task","document_id":"019f24 |
| `src/teri/.kb/store/commits/019f2513-a1ca-78d1-be48-e73c238fec21.json` | 3 | `reference` | "content_hash": "e97f68fb43cc546bbcfe44f13d1324...", |
| `src/teri/.kb/store/commits/019f24f9-5569-7c11-ac97-45e05ef21c9b.json` | 2 | `reference` | "commit": {"author":"drdave <flexnetos@de-flex.net>","changes":[{"change_type":"created","content_hash":"9582f232695d22d3d7664e9f381d36...","doc_type":"task","document_id":"019f24f |
| `src/teri/.kb/store/commits/019f24f9-5569-7c11-ac97-45e05ef21c9b.json` | 3 | `reference` | "content_hash": "492784e92564e1853eaba5d3a61b4b...", |

### `contract_hash`

- Criticality: `critical`
- SQL origin: `envctl_migration_artifact_contracts.contract_hash`
- Transformation: hash derivation and comparison through sha256 file/content digests; database persistence requires the field before record insertion

### `contract_id`

- Criticality: `critical`
- Transformation: stable identifier propagation through run, operation, artifact, evidence, and graph-edge records

### `descriptor_hash`

- Criticality: `critical`
- SQL origin: `envctl_migration_targets.descriptor_hash`
- Transformation: hash derivation and comparison through sha256 file/content digests; database persistence requires the field before record insertion

### `event_hash`

- Criticality: `critical`
- SQL origin: `envctl_migration_run_events.event_hash`
- Schema origin: `envctl Migration Run Event`, `RunEvent`
- Transformation: hash derivation and comparison through sha256 file/content digests

### `evidence_refs`

- Criticality: `critical`
- Schema origin: `envctl Migration Artifact Record`, `envctl Migration Run Event`, `envctl Migration Validation Result`, `RunEvent`, `ArtifactRecord`, `ValidationResult`
- Transformation: artifact evidence and graph-link materialization in proof and registry payloads

| file | line | role | evidence |
|---|---:|---|---|
| `src/teri/.worktrees/issue-86-source-wires/.worktrees/external-sources/brain-in-the-fish/crates/core/src/server.rs` | 821 | `reference` | let evidence_refs = input.evidence_used.clone(); |
| `src/teri/.worktrees/issue-86-source-wires/.worktrees/external-sources/brain-in-the-fish/crates/core/src/server.rs` | 847 | `reference` | for ev_ref in &evidence_refs { |

### `id`

- Criticality: `critical`
- SQL origin: `envctl_migration_targets.id`, `envctl_migration_packages.id`, `envctl_migration_artifact_contracts.id`, `envctl_migration_recipes.id`, `envctl_migration_runs.id`, `envctl_migration_operations.id`
- Schema origin: `pathRule`, `workspace`
- Transformation: stable identifier propagation through run, operation, artifact, evidence, and graph-edge records; database persistence requires the field before record insertion
- Consumption: `EvidenceRecord` via `envctl_migration_evidence` to `nu_plugin`, `ValidationResult` via `envctl_migration_validations` to `nu_plugin`

| file | line | role | evidence |
|---|---:|---|---|
| `LOCAL_WORKAROUNDS.md` | 969 | `reference` | xdg-terminal-exec --print-id /home/flexnetos/.nix-profile/bin/yzx desktop launch |
| `WORKLOG.md` | 1742 | `reference` | whoami: Cloud domain gitkb.com, user id:379904488992935178, no organization memberships. |
| `WORKLOG.md` | 1910 | `reference` | Session id: 019f1f6d-3c28-7b01-9cb8-af33ab... |
| `.kb/skills/code-intelligence/SKILL.md` | 39 | `reference` | git-kb code flow <flow-id> --json # Inspect one flow |
| `.kb/skills/refactor-safety/SKILL.md` | 28 | `reference` | git-kb code callers "<full-symbol-id>" --json |
| `.kb/skills/refactor-safety/SKILL.md` | 34 | `reference` | git-kb code callees "<full-symbol-id>" --json |
| `.kb/skills/refactor-safety/SKILL.md` | 46 | `reference` | git-kb code refs "<full-symbol-id>" --json |
| `.kb/skills/gitkb/SKILL.md` | 83 | `reference` | git-kb code flow <flow-id> --json # Inspect one flow |

### `links`

- Criticality: `critical`
- Schema origin: `envctl Migration Artifact Record`, `ArtifactRecord`
- Transformation: artifact evidence and graph-link materialization in proof and registry payloads; static target scan found producer/update/write references

| file | line | role | evidence |
|---|---:|---|---|
| `LOCAL_WORKAROUNDS.md` | 1111 | `reference` | That directory mirrors the managed standalone layout and links `codex`, |
| `WORKLOG.md` | 1657 | `reference` | created 14 Claude skill links, skipped 41 existing files, merged .claude/settings.json. |
| `WORKLOG.md` | 1661 | `consumption` | skipped 53 existing files/links; Codex scaffold was already present. |
| `.kb/AGENTS.md` | 690 | `reference` | Go back and add links: |
| `.kb/skills/kb-close/SKILL.md` | 38 | `reference` | Check if the document has a "Completion Evidence" section (commit hashes, PR links, test results, verification steps). This is distinct from the "Progress Log" (chronological work  |
| `.kb/skills/kb-close/SKILL.md` | 42 | `reference` | > - Commit hashes or PR links |
| `.kb/skills/kb-handoff/SKILL.md` | 49 | `transformation` | 2. Update the relevant section (mark phases complete, update metrics, add PR links) |
| `.kb/skills/kb-create/SKILL.md` | 197 | `reference` | - Any related documents found during discovery (as suggested links) |

### `operation_id`

- Criticality: `critical`
- SQL origin: `envctl_migration_run_events.operation_id`, `envctl_migration_evidence.operation_id`, `envctl_migration_approvals.operation_id`, `envctl_migration_validations.operation_id`, `envctl_migration_checkpoints.operation_id`, `envctl_migration_rollbacks.operation_id`
- Schema origin: `envctl Migration Approval Request`, `operation`, `envctl Migration Operation`, `envctl Migration Run Event`, `Operation`, `RunEvent`
- Transformation: stable identifier propagation through run, operation, artifact, evidence, and graph-edge records; database persistence requires the field before record insertion

### `package_hash`

- Criticality: `critical`
- SQL origin: `envctl_migration_packages.package_hash`
- Transformation: hash derivation and comparison through sha256 file/content digests; database persistence requires the field before record insertion

### `path`

- Criticality: `critical`
- SQL origin: `envctl_migration_artifacts.path`
- Schema origin: `envctl Migration Artifact Record`, `workspace`, `ArtifactRecord`
- Transformation: static lineage only; no transformation-specific reference found

| file | line | role | evidence |
|---|---:|---|---|
| `LOCAL_WORKAROUNDS.md` | 80 | `reference` | was launched from an older runtime path, while regenerated host state now points |
| `LOCAL_WORKAROUNDS.md` | 100 | `reference` | The Bubblewrap crash path: |
| `LOCAL_WORKAROUNDS.md` | 155 | `reference` | frontdoor path. Built the envctl release binary from source, then exposed it |
| `LOCAL_WORKAROUNDS.md` | 211 | `reference` | yazelix_flexnetos_foundation -> path:/home/flexnetos/FlexNetOS/src/... |
| `LOCAL_WORKAROUNDS.md` | 234 | `reference` | codex doctor --all -> 17 ok, 2 notes, 1 warn, 0 fail; bundled /nix/store/gh9cc.../codex-path/rg detected |
| `LOCAL_WORKAROUNDS.md` | 295 | `reference` | Duplicate curated cache was preserved, then moved out of the active cache path: |
| `LOCAL_WORKAROUNDS.md` | 524 | `reference` | path ahead of the profile in `PATH`; clean-environment `yzx doctor` removed the |
| `LOCAL_WORKAROUNDS.md` | 533 | `reference` | /home/flexnetos/.nix-profile/bin/yzx desktop install --print-path |

### `previous_event_hash`

- Criticality: `critical`
- SQL origin: `envctl_migration_run_events.previous_event_hash`
- Schema origin: `envctl Migration Run Event`, `RunEvent`
- Transformation: hash derivation and comparison through sha256 file/content digests

### `primary_root`

- Criticality: `critical`
- SQL origin: `envctl_migration_targets.primary_root`
- Schema origin: `envctl Migration Target Descriptor`, `TargetDescriptor`
- Transformation: database persistence requires the field before record insertion

### `proof_uri`

- Criticality: `critical`
- Schema origin: `AgentLane`, `ExecutionPacket`, `TaskGraphRow`
- Transformation: artifact evidence and graph-link materialization in proof and registry payloads

### `recipe_hash`

- Criticality: `critical`
- SQL origin: `envctl_migration_recipes.recipe_hash`
- Transformation: hash derivation and comparison through sha256 file/content digests; database persistence requires the field before record insertion

### `recipe_id`

- Criticality: `critical`
- SQL origin: `envctl_migration_runs.recipe_id`
- Schema origin: `envctl Migration Recipe`, `MigrationRecipe`, `MigrationRun`
- Transformation: stable identifier propagation through run, operation, artifact, evidence, and graph-edge records; database persistence requires the field before record insertion

### `reproducibility_hash`

- Criticality: `critical`
- SQL origin: `envctl_migration_runs.reproducibility_hash`
- Schema origin: `MigrationRun`
- Transformation: hash derivation and comparison through sha256 file/content digests

### `risk`

- Criticality: `critical`
- SQL origin: `envctl_migration_operations.risk`, `envctl_migration_approvals.risk`
- Schema origin: `envctl Migration Approval Request`, `operation`, `envctl Migration Operation`, `Operation`, `ApprovalRequest`
- Transformation: database persistence requires the field before record insertion

| file | line | role | evidence |
|---|---:|---|---|
| `src/teri/README.md` | 43 | `consumption` | relations to be tested at zero risk. |
| `src/teri/tests/community_loop_e2e.rs` | 83 | `reference` | risk: "activity_decline".to_string(), |
| `src/teri/src/seed/community/mod.rs` | 155 | `reference` | /// A predicted health risk for a whole domain/space (e.g. activity collapse, |
| `src/teri/src/seed/community/mod.rs` | 159 | `reference` | /// Domain (space) the risk is scoped to. |
| `src/teri/src/seed/community/mod.rs` | 162 | `reference` | pub risk: String, |
| `src/teri/src/seed/community/mod.rs` | 223 | `reference` | /// Push per-space health-risk predictions. |
| `src/teri/src/seed/community/mod.rs` | 360 | `reference` | let risk = SpaceHealthRisk { |
| `src/teri/src/seed/community/mod.rs` | 362 | `reference` | risk: "activity_decline".to_string(), |

### `run_id`

- Criticality: `critical`
- SQL origin: `envctl_migration_operations.run_id`, `envctl_migration_run_events.run_id`, `envctl_migration_evidence.run_id`, `envctl_migration_artifacts.run_id`, `envctl_migration_graph_edges.run_id`, `envctl_migration_approvals.run_id`
- Schema origin: `envctl Migration Approval Request`, `envctl Migration Artifact Record`, `envctl Migration Operation`, `envctl Migration Run Event`, `envctl Migration Validation Result`, `Operation`
- Transformation: stable identifier propagation through run, operation, artifact, evidence, and graph-edge records; database persistence requires the field before record insertion

| file | line | role | evidence |
|---|---:|---|---|
| `src/teri/.worktrees/issue-86-source-wires/.worktrees/external-sources/openevolve/examples/sldbench/run.sh` | 69 | `reference` | local run_id="run_${run}" |
| `src/teri/.worktrees/issue-86-source-wires/.worktrees/external-sources/openevolve/examples/sldbench/run.sh` | 70 | `reference` | local output_dir="${RESULTS_BASE_DIR}/${task_name}/${model}/${run_id}" |
| `src/teri/.worktrees/issue-86-source-wires/.worktrees/external-sources/openevolve/examples/sldbench/run.sh` | 90 | `reference` | echo " -> No best program found. Evolving for ${task_name}/${model}/${run_id}..." |
| `src/teri/.worktrees/issue-86-source-wires/.worktrees/external-sources/openevolve/examples/sldbench/run.sh` | 105 | `reference` | echo " -> Evaluating best program for ${task_name}/${model}/${run_id}..." |
| `src/teri/.worktrees/issue-86-source-wires/.worktrees/external-sources/openevolve/examples/sldbench/run.sh` | 111 | `reference` | echo " -> WARNING: Evolution failed. No best program found for ${task_name}/${model}/${run_id}. Evaluation skipped." |
| `src/teri/.worktrees/issue-86-source-wires/.worktrees/external-sources/BettaFish/ReportEngine/nodes/chapter_generation_node.py` | 203 | `reference` | run_id = run_dir.name |
| `src/teri/.worktrees/issue-86-source-wires/.worktrees/external-sources/BettaFish/ReportEngine/nodes/chapter_generation_node.py` | 204 | `reference` | self._ensure_run_state(run_id) |
| `src/teri/.worktrees/issue-86-source-wires/.worktrees/external-sources/BettaFish/ReportEngine/nodes/chapter_generation_node.py` | 227 | `reference` | run_id, |

### `sha256`

- Criticality: `critical`
- SQL origin: `envctl_migration_evidence.sha256`
- Schema origin: `EvidenceRecord`
- Transformation: hash derivation and comparison through sha256 file/content digests

| file | line | role | evidence |
|---|---:|---|---|
| `LOCAL_WORKAROUNDS.md` | 26 | `reference` | sha256: 8eac967312222bad4805b929db293f... |
| `LOCAL_WORKAROUNDS.md` | 115 | `reference` | sha256: 8884e5a881cfb5a4e028a37cec1387... |
| `LOCAL_WORKAROUNDS.md` | 137 | `reference` | sha256: b7b5d0a6b38f6e53713df85dc18146... |
| `LOCAL_WORKAROUNDS.md` | 183 | `reference` | sha256: 1f39f9fdf1eec4b830eca1ea86e5ee... |
| `LOCAL_WORKAROUNDS.md` | 185 | `reference` | sha256: d1e088fca402c482e6fe0c3f27b02e... |
| `LOCAL_WORKAROUNDS.md` | 187 | `reference` | sha256: fcfd97c24468a5d6bd17e8d6793efc... |
| `LOCAL_WORKAROUNDS.md` | 299 | `reference` | sha256: 4648832c3718caf39aba99a8d4fe33... |
| `LOCAL_WORKAROUNDS.md` | 721 | `reference` | sha256: f649d213c082f8eb86413b01495baa... |

### `status`

- Criticality: `critical`
- SQL origin: `envctl_migration_runs.status`, `envctl_migration_operations.status`, `envctl_migration_artifacts.status`, `envctl_migration_approvals.status`, `envctl_migration_validations.status`, `envctl_migration_rollbacks.status`
- Schema origin: `envctl Migration Approval Request`, `envctl Migration Artifact Record`, `envctl Migration Operation`, `envctl Migration Validation Result`, `AgentLane`, `ProofRecord`
- Transformation: state normalization through enum/check constraints and validation status records; database persistence requires the field before record insertion
- Consumption: `MigrationRun` via `envctl_migration_runs` to `nu_plugin`

| file | line | role | evidence |
|---|---:|---|---|
| `LOCAL_WORKAROUNDS.md` | 74 | `reference` | /home/flexnetos/.nix-profile/bin/yzx status -> Generated runtime state up to date, repair needed no |
| `LOCAL_WORKAROUNDS.md` | 79 | `reference` | window restart to consume the regenerated status-bar command. Its active session |
| `LOCAL_WORKAROUNDS.md` | 324 | `reference` | clean-env yzx status --json runtime_dir -> /nix/store/p45lnz6nsvjzvhjlbai... |
| `LOCAL_WORKAROUNDS.md` | 325 | `reference` | clean-env yzx status --json generated_state_materializatio... -> noop |
| `LOCAL_WORKAROUNDS.md` | 379 | `reference` | PATH=/home/flexnetos/FlexNetOS/usr/bin:$PATH meta exec -- /home/flexnetos/FlexNetOS/usr/... status --json -> 17 commands complete |
| `LOCAL_WORKAROUNDS.md` | 504 | `reference` | clean-env yzx status --json runtime_dir -> /nix/store/hk4m00bfnqddhlymfy8... |
| `LOCAL_WORKAROUNDS.md` | 505 | `reference` | clean-env yzx status --json generated_state_materializatio... -> noop |
| `LOCAL_WORKAROUNDS.md` | 668 | `reference` | src/meta git-kb status --json -> clean at 019f1f89-ddf0-72a2-89c1-5147f1... |

### `target_id`

- Criticality: `critical`
- SQL origin: `envctl_migration_targets.target_id`, `envctl_migration_runs.target_id`
- Schema origin: `envctl Migration Target Descriptor`, `TargetDescriptor`, `MigrationRun`
- Transformation: stable identifier propagation through run, operation, artifact, evidence, and graph-edge records; static target scan found producer/update/write references; database persistence requires the field before record insertion

| file | line | role | evidence |
|---|---:|---|---|
| `src/teri/tests/social_world_e2e_test.rs` | 51 | `reference` | Some(id) if n.is_multiple_of(2) => Ok(format!("LIKE_POST(target_id=post-{id})")), |
| `src/teri/src/sim/mod.rs` | 48 | `reference` | Like { target_kind: TargetKind, target_id: String }, |
| `src/teri/src/sim/mod.rs` | 50 | `reference` | Dislike { target_kind: TargetKind, target_id: String }, |
| `src/teri/src/sim/mod.rs` | 75 | `reference` | SocialAction::Like { target_kind: TargetKind::Post, target_id } => { |
| `src/teri/src/sim/mod.rs` | 76 | `transformation` | write!(f, "Liked post: {}", target_id) |
| `src/teri/src/sim/mod.rs` | 78 | `reference` | SocialAction::Like { target_kind: TargetKind::Comment, target_id } => { |
| `src/teri/src/sim/mod.rs` | 79 | `transformation` | write!(f, "Liked comment: {}", target_id) |
| `src/teri/src/sim/mod.rs` | 81 | `reference` | SocialAction::Dislike { target_kind: TargetKind::Post, target_id } => { |

### `validator`

- Criticality: `critical`
- SQL origin: `envctl_migration_validations.validator`
- Schema origin: `envctl Migration Validation Result`, `ValidationResult`
- Transformation: database persistence requires the field before record insertion

| file | line | role | evidence |
|---|---:|---|---|
| `WORKLOG.md` | 494 | `reference` | The scaffold validator passed 13 of 14 copied plugin payloads. The |
| `WORKLOG.md` | 495 | `reference` | `codex-security` payload was left intact after the scaffold validator rejected a |
| `src/teri/.worktrees/issue-86-source-wires/.worktrees/external-sources/BettaFish/regenerate_latest_md.py` | 112 | `reference` | validator = IRValidator() |
| `src/teri/.worktrees/issue-86-source-wires/.worktrees/external-sources/BettaFish/regenerate_latest_md.py` | 115 | `reference` | ok, errors = validator.validate_chapter(chapter) |
| `src/teri/.worktrees/issue-86-source-wires/.worktrees/external-sources/BettaFish/regenerate_latest_html.py` | 112 | `reference` | validator = IRValidator() |
| `src/teri/.worktrees/issue-86-source-wires/.worktrees/external-sources/BettaFish/regenerate_latest_html.py` | 115 | `reference` | ok, errors = validator.validate_chapter(chapter) |
| `src/teri/.worktrees/issue-86-source-wires/.worktrees/external-sources/BettaFish/README.md` | 168 | `consumption` | │ │ └── validator.py # 章节JSON结构校验器 |
| `src/teri/.worktrees/issue-86-source-wires/.worktrees/external-sources/BettaFish/README-EN.md` | 167 | `consumption` | │ │ └── validator.py # Chapter JSON structure validator |

### `actor`

- Criticality: `supporting`
- Schema origin: `ProofRecord`, `ProofRecord`
- Transformation: static lineage only; no transformation-specific reference found

### `actor_id`

- Criticality: `supporting`
- SQL origin: `envctl_migration_run_events.actor_id`
- Schema origin: `envctl Migration Run Event`, `RunEvent`
- Transformation: stable identifier propagation through run, operation, artifact, evidence, and graph-edge records

### `actor_type`

- Criticality: `supporting`
- SQL origin: `envctl_migration_run_events.actor_type`
- Schema origin: `envctl Migration Run Event`, `RunEvent`
- Transformation: database persistence requires the field before record insertion

### `agent_name`

- Criticality: `supporting`
- SQL origin: `envctl_migration_agent_sessions.agent_name`
- Transformation: database persistence requires the field before record insertion

### `agent_runtime`

- Criticality: `supporting`
- Schema origin: `AgentLane`, `ExecutionPacket`, `TaskGraphRow`
- Transformation: static lineage only; no transformation-specific reference found

### `allow_destructive`

- Criticality: `supporting`
- Schema origin: `safety`
- Transformation: static lineage only; no transformation-specific reference found

### `allow_network`

- Criticality: `supporting`
- Schema origin: `safety`
- Transformation: static lineage only; no transformation-specific reference found

### `allowed_paths`

- Criticality: `supporting`
- Schema origin: `AgentLane`, `ExecutionPacket`, `envctl Background Helper Filesystem Boundaries`, `TaskGraphRow`
- Transformation: static lineage only; no transformation-specific reference found

### `approval_id`

- Criticality: `supporting`
- Schema origin: `envctl Migration Approval Request`, `ApprovalRequest`, `ApprovalDecision`
- Transformation: stable identifier propagation through run, operation, artifact, evidence, and graph-edge records

### `approval_policy`

- Criticality: `supporting`
- SQL origin: `envctl_migration_runs.approval_policy`
- Transformation: static lineage only; no transformation-specific reference found

### `artifact_contract`

- Criticality: `supporting`
- Schema origin: `envctl Migration Target Descriptor`, `TargetDescriptor`
- Transformation: static lineage only; no transformation-specific reference found

### `artifact_type`

- Criticality: `supporting`
- SQL origin: `envctl_migration_artifacts.artifact_type`
- Schema origin: `envctl Migration Artifact Record`, `ArtifactRecord`
- Transformation: static lineage only; no transformation-specific reference found

### `authority_level`

- Criticality: `supporting`
- SQL origin: `envctl_migration_agent_sessions.authority_level`
- Transformation: static lineage only; no transformation-specific reference found

### `blocked_paths`

- Criticality: `supporting`
- Schema origin: `AgentLane`, `ExecutionPacket`, `envctl Background Helper Filesystem Boundaries`, `TaskGraphRow`
- Transformation: static lineage only; no transformation-specific reference found

### `blocks`

- Criticality: `supporting`
- Schema origin: `ExecutionPacket`, `TaskGraphRow`
- Transformation: static lineage only; no transformation-specific reference found

### `can_run_parallel`

- Criticality: `supporting`
- Schema origin: `ExecutionPacket`, `TaskGraphRow`
- Transformation: static lineage only; no transformation-specific reference found

### `checksums`

- Criticality: `supporting`
- Schema origin: `ProofRecord`, `ProofRecord`
- Transformation: static lineage only; no transformation-specific reference found

### `code`

- Criticality: `supporting`
- Schema origin: `StructuredError`
- Transformation: static lineage only; no transformation-specific reference found

### `command_redacted`

- Criticality: `supporting`
- SQL origin: `envctl_migration_operations.command_redacted`
- Transformation: static lineage only; no transformation-specific reference found

### `command_template`

- Criticality: `supporting`
- Schema origin: `ExecutionPacket`, `TaskGraphRow`
- Transformation: static lineage only; no transformation-specific reference found

### `commands`

- Criticality: `supporting`
- Schema origin: `nu_plugin envctl Migration Command Manifest`
- Transformation: static lineage only; no transformation-specific reference found

### `commands_run`

- Criticality: `supporting`
- Schema origin: `ProofRecord`, `ProofRecord`
- Transformation: static lineage only; no transformation-specific reference found

### `completed_at`

- Criticality: `supporting`
- Schema origin: `ProofRecord`, `ProofRecord`
- Transformation: static lineage only; no transformation-specific reference found

### `completed_at_utc`

- Criticality: `supporting`
- SQL origin: `envctl_migration_runs.completed_at_utc`, `envctl_migration_operations.completed_at_utc`
- Schema origin: `ReplayResult`
- Transformation: static lineage only; no transformation-specific reference found

### `completion_gate`

- Criticality: `supporting`
- Schema origin: `ExecutionPacket`, `TaskGraphRow`
- Transformation: static lineage only; no transformation-specific reference found

### `confidence`

- Criticality: `supporting`
- SQL origin: `envctl_migration_graph_edges.confidence`
- Transformation: static lineage only; no transformation-specific reference found

### `contract_json`

- Criticality: `supporting`
- SQL origin: `envctl_migration_artifact_contracts.contract_json`
- Transformation: database persistence requires the field before record insertion

### `contract_name`

- Criticality: `supporting`
- SQL origin: `envctl_migration_artifact_contracts.contract_name`
- Transformation: database persistence requires the field before record insertion

### `contract_version`

- Criticality: `supporting`
- SQL origin: `envctl_migration_artifact_contracts.contract_version`
- Transformation: database persistence requires the field before record insertion

### `created_at_utc`

- Criticality: `supporting`
- SQL origin: `envctl_migration_targets.created_at_utc`, `envctl_migration_artifact_contracts.created_at_utc`, `envctl_migration_recipes.created_at_utc`, `envctl_migration_runs.created_at_utc`, `envctl_migration_operations.created_at_utc`, `envctl_migration_run_events.created_at_utc`
- Schema origin: `MigrationRun`
- Transformation: database persistence requires the field before record insertion

### `current_task`

- Criticality: `supporting`
- Schema origin: `AgentLane`
- Transformation: static lineage only; no transformation-specific reference found

### `decided_at_utc`

- Criticality: `supporting`
- SQL origin: `envctl_migration_approvals.decided_at_utc`
- Schema origin: `ApprovalDecision`
- Transformation: static lineage only; no transformation-specific reference found

### `decided_by`

- Criticality: `supporting`
- SQL origin: `envctl_migration_approvals.decided_by`
- Schema origin: `envctl Migration Approval Request`, `ApprovalRequest`, `ApprovalDecision`
- Transformation: static lineage only; no transformation-specific reference found

### `decision`

- Criticality: `supporting`
- Schema origin: `ApprovalDecision`
- Transformation: static lineage only; no transformation-specific reference found
- Consumption: `ApprovalDecision` via `envctl_migration_approvals` to `nu_plugin`

### `default_mode`

- Criticality: `supporting`
- Schema origin: `safety`
- Transformation: static lineage only; no transformation-specific reference found

### `depends_on`

- Criticality: `supporting`
- Schema origin: `phase`, `ExecutionPacket`, `TaskGraphRow`
- Transformation: static lineage only; no transformation-specific reference found

### `description`

- Criticality: `supporting`
- Schema origin: `pathRule`
- Transformation: static lineage only; no transformation-specific reference found

### `descriptor_json`

- Criticality: `supporting`
- SQL origin: `envctl_migration_targets.descriptor_json`
- Transformation: database persistence requires the field before record insertion

### `details_json`

- Criticality: `supporting`
- SQL origin: `envctl_migration_validations.details_json`
- Transformation: static lineage only; no transformation-specific reference found

### `edge_id`

- Criticality: `supporting`
- Schema origin: `GraphEdge`
- Transformation: stable identifier propagation through run, operation, artifact, evidence, and graph-edge records

### `edge_type`

- Criticality: `supporting`
- SQL origin: `envctl_migration_graph_edges.edge_type`
- Schema origin: `GraphEdge`
- Transformation: database persistence requires the field before record insertion

### `error_json`

- Criticality: `supporting`
- SQL origin: `envctl_migration_operations.error_json`
- Transformation: static lineage only; no transformation-specific reference found

### `event_id`

- Criticality: `supporting`
- Schema origin: `ApprovalDecision`
- Transformation: stable identifier propagation through run, operation, artifact, evidence, and graph-edge records

| file | line | role | evidence |
|---|---:|---|---|
| `src/teri/src/api/mod.rs` | 72 | `reference` | pub event_id: String, |
| `src/teri/src/api/mod.rs` | 84 | `reference` | event_id: format!("tick-{}", snapshot.tick), |
| `src/teri/src/api/mod.rs` | 96 | `reference` | event_id: format!("gap-{}", missed_ticks), |
| `src/teri/src/api/mod.rs` | 108 | `reference` | /// field, so the SSE wire format stays uniform (`{ tick, data, event_id }` always). |
| `src/teri/src/api/mod.rs` | 109 | `reference` | /// The `event_id` is the fixed string `"sim-end"` (no count suffix: there is exactly one |
| `src/teri/src/api/mod.rs` | 119 | `reference` | event_id: "sim-end".to_string(), |
| `src/teri/src/api/mod.rs` | 502 | `reference` | assert_eq!(event.event_id, "tick-42"); |
| `src/teri/src/api/mod.rs` | 526 | `reference` | assert_eq!(event.event_id, "gap-42"); |

### `event_seq`

- Criticality: `supporting`
- SQL origin: `envctl_migration_run_events.event_seq`
- Schema origin: `envctl Migration Run Event`, `RunEvent`
- Transformation: database persistence requires the field before record insertion

### `event_type`

- Criticality: `supporting`
- SQL origin: `envctl_migration_run_events.event_type`
- Schema origin: `envctl Migration Run Event`, `RunEvent`
- Transformation: database persistence requires the field before record insertion

### `evidence`

- Criticality: `supporting`
- Schema origin: `ProofRecord`, `ProofRecord`
- Transformation: static lineage only; no transformation-specific reference found
- Consumption: `EvidenceRecord` via `envctl_migration_evidence` to `nu_plugin`

### `evidence_id`

- Criticality: `supporting`
- Schema origin: `EvidenceRecord`
- Transformation: stable identifier propagation through run, operation, artifact, evidence, and graph-edge records

### `evidence_json`

- Criticality: `supporting`
- SQL origin: `envctl_migration_artifacts.evidence_json`, `envctl_migration_graph_edges.evidence_json`, `envctl_migration_validations.evidence_json`
- Transformation: static lineage only; no transformation-specific reference found

### `evidence_kind`

- Criticality: `supporting`
- SQL origin: `envctl_migration_evidence.evidence_kind`
- Schema origin: `EvidenceRecord`
- Transformation: database persistence requires the field before record insertion

### `evidence_refs_json`

- Criticality: `supporting`
- SQL origin: `envctl_migration_run_events.evidence_refs_json`
- Transformation: static lineage only; no transformation-specific reference found

### `execution_cell`

- Criticality: `supporting`
- Schema origin: `ExecutionPacket`, `TaskGraphRow`
- Transformation: static lineage only; no transformation-specific reference found

### `failure_reason`

- Criticality: `supporting`
- Schema origin: `ProofRecord`, `ProofRecord`
- Transformation: static lineage only; no transformation-specific reference found

### `files_changed`

- Criticality: `supporting`
- Schema origin: `ProofRecord`, `ProofRecord`
- Transformation: static lineage only; no transformation-specific reference found

## Validation

- Files scanned: `2500`
- Truncated: `True`
- Skipped: `{"max_files_reached": 1, "too_large": 11, "unsupported_suffix": 733}`
- Artifact registry persisted path and content hash for the canonical markdown, task markdown, and JSON artifact.
- Registry links include REQ-024 artifact registry, REQ-040 shared protocol schemas, and VER-300 unit validation.
