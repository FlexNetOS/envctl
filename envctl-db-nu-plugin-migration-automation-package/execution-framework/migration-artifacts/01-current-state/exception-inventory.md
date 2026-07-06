# Exception Inventory

- Task: `ART-134_EXCEPTION_INVENTORY`
- Contract artifact: `artifact:01-current-state-exception-inventory-md`
- Canonical path: `migration-artifacts/01-current-state/exception-inventory.md`
- Generated at: `2026-07-04T23:28:58+00:00`
- Scope: special cases, one-off scripts, manual processes, and explicit approval/rollback boundaries in the migration package.

## Summary

- Exceptions inventoried: `10`
- Inputs: target descriptor registry, package scan, envctl database model, shared protocol proof, and artifact registry proof.
- Downstream gate: `VER-300_UNIT_VALIDATION` must see this artifact, registry hash, and validation links.

## Exceptions

| exception id | category | surface | path | risk | status | handling |
|---|---|---|---|---|---|---|
| EXC-ART134-001 | manual_process | Package installation into local envctl and nu_plugin checkouts | `../INSTALL_IN_REPOS.sh` | medium | documented | Run only from the package root with explicit --envctl-repo and --nu-plugin-repo arguments; preserve additive copy semantics. |
| EXC-ART134-002 | one_off_script | Codex master prompt bootstrap | `../RUN_WITH_CODEX_ENVCTL.sh` | medium | requires_runtime_check | Treat the generated run context and Codex CLI flags as environment-specific; validate the command surface before operator handoff. |
| EXC-ART134-003 | manual_process | Background helper launch | `docs/INSTALL_BOOTSTRAP.md` | medium | documented | Require the task proof named by the packet before advancing dependencies, and keep the codex-exec stdout log as execution evidence. |
| EXC-ART134-004 | human_approval | Approval-gated operator session | `examples/nu/operator-session-template.nu` | high | approval_required_for_risky_modes | Keep approval rows and reason strings in envctl evidence; stop goal-loop advancement while approvals are pending. |
| EXC-ART134-005 | security_exception | Blocked secret-bearing paths | `scripts/artifact_registry.py` | high | enforced | Use only policy-safe artifact paths and rely on redaction controls for scan/log/proof persistence. |
| EXC-ART134-006 | completion_gate | Proof ledger as the task completion source | `proof_records/proof_ledger.jsonl` | medium | required | Append a task proof after writing artifacts, then refresh generated/status_from_proofs.json from the ledger. |
| EXC-ART134-007 | rollback_boundary | Task-specific rollback by generated-file removal | `generated/execution_packets/ART-134_EXCEPTION_INVENTORY.json` | medium | documented | Remove the task-local artifacts, report, heartbeat, log, proof, and ledger entry together if ART-134 is rolled back. |
| EXC-ART134-008 | legacy_package_carryover | Prior FlexNetOS migration package retained under source/ | `../source/codex-flexnetos-migration-prompt-package` | medium | carried_as_input | Treat source/codex-flexnetos-migration-prompt-package as evidence input, not as the active envctl-nu_plugin execution surface. |
| EXC-ART134-009 | external_path | Migration target and repo roots supplied outside the execution framework | `generated/envctl_target_registry.json` | medium | parameterized | Resolve roots at execution time and keep artifacts under migration-artifacts/ or execution-framework/ unless a packet explicitly allows external repo writes. |
| EXC-ART134-010 | manual_validation | Replay and execute-full validation modes | `docs/SECURITY_REDACTION.md` | high | approval_gated | Run verify-only or dry-run replay for automated evidence; require approval records before execute-full. |

## Evidence References

- `EXC-ART134-001`: `../INSTALL_IN_REPOS.sh`, `docs/INSTALL_BOOTSTRAP.md`, `generated/install_bootstrap_manifest.json`
- `EXC-ART134-002`: `../RUN_WITH_CODEX_ENVCTL.sh`, `docs/INSTALL_BOOTSTRAP.md`
- `EXC-ART134-003`: `docs/INSTALL_BOOTSTRAP.md`, `state/active_packet_pids.json`
- `EXC-ART134-004`: `examples/nu/operator-session-template.nu`, `docs/GOAL_LOOP_PROTOCOL.md`, `docs/SECURITY_REDACTION.md`
- `EXC-ART134-005`: `scripts/artifact_registry.py`, `docs/SECURITY_REDACTION.md`, `generated/security_redaction_validation_report.json`
- `EXC-ART134-006`: `docs/GOAL_LOOP_PROTOCOL.md`, `scripts/status_from_proofs.py`, `proof_records/proof_ledger.jsonl`
- `EXC-ART134-007`: `generated/execution_packets/ART-134_EXCEPTION_INVENTORY.json`, `history/pre_execution_framework_manifest.json`
- `EXC-ART134-008`: `../source/codex-flexnetos-migration-prompt-package/RUN_WITH_CODEX.sh`, `../source/codex-flexnetos-migration-prompt-package/expected-output/migration-artifacts-tree.md`
- `EXC-ART134-009`: `generated/envctl_target_registry.json`, `docs/FILESYSTEM_BOUNDARIES.md`, `generated/execution_packets/ART-134_EXCEPTION_INVENTORY.json`
- `EXC-ART134-010`: `docs/SECURITY_REDACTION.md`, `generated/envctl_run_ledger_report.json`, `generated/operation_state_machine.json`
