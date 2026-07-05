
# envctl Execution Framework Layer

This layer turns the envctl database + nu_plugin migration automation package into an executable, verifiable, multi-agent work system.

It is additive by design: it does not downgrade or replace the source package. It adds a task graph, JSON execution packets, proof records, state reports, validation scripts, and operator templates.

## Required local execution

Run from this directory:

```bash
python3 scripts/validate_task_graph.py generated/task_graph.csv
python3 scripts/task_graph_to_packets.py generated/task_graph.csv
python3 scripts/goal_loop.py generated/task_graph.csv
python3 scripts/verify_install_bootstrap.py
python3 scripts/verify_run_replay.py
python3 scripts/verify_history_and_completeness.py
```

These commands validate the task graph, generate bounded task packets, compute runnable work, merge proof state, and prove package completeness.

## Install And Bootstrap

The repo install and Codex/background helper command templates are generated and verified by:

```bash
python3 scripts/verify_install_bootstrap.py
```

The generated operator entrypoint is `docs/INSTALL_BOOTSTRAP.md`; the machine-readable command template manifest is `generated/install_bootstrap_manifest.json`. The verifier checks the package-level `INSTALL_IN_REPOS.sh` and `RUN_WITH_CODEX_ENVCTL.sh` inputs, writes `state/REQ-044_INSTALL_BOOTSTRAP.heartbeat.json`, and records proof at `proof_records/REQ-044_INSTALL_BOOTSTRAP.proof.json`.

## Run And Replay

The run/replay operator command templates are generated and verified by:

```bash
python3 scripts/verify_run_replay.py
```

The generated operator entrypoint is `docs/RUN_REPLAY.md`; the machine-readable command template manifest is `generated/run_replay_manifest.json`. The verifier checks replay, rollback, approval, and convenience-template prerequisites, writes `state/REQ-045_RUN_REPLAY.heartbeat.json`, and records proof at `proof_records/REQ-045_RUN_REPLAY.proof.json`.

## Source intent preserved

The framework implements the attached execution prompt requirements and the migration artifact contract: inventory, dependency maps, data flow graphs, database/schema lineage, API/event contracts, debugging maps, validation/reconciliation evidence, cutover, rollback, governance, and decommission controls.
