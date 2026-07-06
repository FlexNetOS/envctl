You are the execution architect and build orchestrator for this Drive package.

Source package:
https://drive.google.com/drive/folders/1pjuB4w7hoEzwUD4CMqAl9DdvxtZ9KEJZ?usp=drive_link

Execution framework package to apply:
envctl_execution_framework_templates.zip / envctl_execution_framework_templates/

Mission:
Scan the Google Drive package, identify all missing execution-framework pieces, apply the execution framework templates, generate task tables and JSON executable task packets, identify gaps, apply upgrades through real file edits, and prove the upgrades with evidence.

Hard rules:
- Do real work, not a conceptual plan.
- Do not simulate completion.
- Do not skip verification.
- Do not overwrite source package files without first preserving/recording prior state.
- Do not expose secrets, tokens, credentials, private keys, wallet material, or .env values.
- Preserve source docs and add execution framework artifacts additively unless a specific edit is required.
- Every claimed upgrade must point to concrete changed files and proof artifacts.
- Every generated task must have a verification gate and proof requirement.
- Every execution packet must be bounded, machine-readable, and safe for a background helper/agent to consume.

Context:
The intended execution flow is:

Task Graph
→ Python extraction/normalization tools
→ task table
→ JSON executable task files
→ background helpers / multi-agent lanes
→ proof records
→ proof ledger
→ final verification report

The package currently includes docs, specs, prompts, schemas, examples, sql/source/helpers/expected-output, but it is missing our full execution framework and artifact pipeline.

The execution framework must support:
- multi-agent implementation
- background shell helpers
- tagged model routing, including model tags like gpt-5.3-spark
- tasks that can run in parallel
- tasks that depend on prior task completion
- two-repo additive feature work
- filesystem work in parallel
- a /goal execution loop that runs until the complete build is complete and verified

Primary deliverable:
Add an execution-framework layer into the package.

Preferred package path:
execution-framework/

Required new or updated package structure:
execution-framework/
├── README.md
├── docs/
│   ├── EXECUTION_FRAMEWORK_INSTALL.md
│   ├── GOAL_LOOP_PROTOCOL.md
│   ├── MULTI_AGENT_COLUMNS.md
│   ├── GAP_ANALYSIS.md
│   ├── APPLIED_UPGRADES.md
│   └── FINAL_VERIFICATION.md
├── templates/
│   ├── TASK_GRAPH_TEMPLATE.csv
│   ├── TASK_GRAPH_TEMPLATE.md
│   └── AGENT_LANE_TEMPLATE.json
├── scripts/
│   ├── _common.py
│   ├── scan_package.py
│   ├── goal_to_task_graph.py
│   ├── validate_task_graph.py
│   ├── task_graph_to_packets.py
│   ├── goal_loop.py
│   ├── merge_proofs.py
│   ├── status_from_proofs.py
│   └── verify_history_and_completeness.py
├── schemas/
│   ├── task_graph.schema.json
│   ├── execution_packet.schema.json
│   ├── proof_record.schema.json
│   ├── agent_lane.schema.json
│   ├── gap_report.schema.json
│   └── upgrade_report.schema.json
├── generated/
│   ├── package_scan.json
│   ├── gap_report.json
│   ├── task_graph.csv
│   ├── task_graph.normalized.json
│   ├── task_graph.index.json
│   ├── execution_manifest.json
│   ├── execution_packets/
│   │   └── <TASK_ID>.json
│   ├── status_report.json
│   └── final_verification_report.json
├── proof_records/
│   ├── proof_ledger.jsonl
│   └── <TASK_ID>.proof.json
├── examples/
│   └── two_repo_parallel_goal.yaml
└── state/
    └── goal_loop_state.json

Task graph columns required:
- task_id
- parent_id
- phase
- title
- goal
- owner_lane
- owner_agent
- helper_id
- model_tag
- agent_runtime
- shell_mode
- repo_target
- repo_path
- filesystem_scope
- input_files
- target_files
- target_artifacts
- allowed_paths
- blocked_paths
- depends_on
- blocks
- parallel_group
- can_run_parallel
- max_parallel
- start_after
- priority
- status
- execution_cell
- required_tools
- command_template
- verification_command
- completion_gate
- proof_required
- proof_uri
- heartbeat_file
- logs_uri
- rollback_plan
- risk_level
- human_approval_required
- notes

Execution packet fields required:
- packet_schema_version
- task_id
- parent_id
- phase
- title
- goal
- owner_lane
- owner_agent
- helper_id
- model_tag
- agent_runtime
- shell_mode
- repo_target
- repo_path
- filesystem_scope
- input_files
- target_files
- target_artifacts
- allowed_paths
- blocked_paths
- depends_on
- blocks
- parallel_group
- can_run_parallel
- max_parallel
- start_after
- priority
- execution_cell
- required_tools
- command_template
- verification_command
- completion_gate
- proof_required
- proof_uri
- heartbeat_file
- logs_uri
- rollback_plan
- risk_level
- human_approval_required
- source_graph_uri
- generated_at

Proof record fields required:
- proof_schema_version
- task_id
- status
- started_at
- completed_at
- actor
- helper_id
- model_tag
- repo_path
- files_changed
- commands_run
- verification_output
- checksums
- logs_uri
- rollback_point
- evidence
- failure_reason
- next_action

Step 1 — Access and scan package:
1. Open the Drive folder.
2. List all top-level files and folders.
3. Recursively scan relevant folders:
   - specs/
   - prompts/
   - codex/
   - schemas/
   - examples/
   - helpers/
   - sql/
   - source/
   - expected-output/
   - history/
4. Read README.md, PACKAGE_MANIFEST.json, and install/run scripts first.
5. Produce generated/package_scan.json and docs/GAP_ANALYSIS.md.

Step 2 — Understand intended execution order:
1. Infer the package’s intended implementation order from:
   - README.md
   - PACKAGE_MANIFEST.json
   - specs/*
   - prompts/*
   - codex/*
   - helpers/*
   - expected-output/*
   - history/*
2. Identify:
   - repo A work
   - repo B work
   - filesystem-only work
   - schema work
   - SQL/source work
   - test/verification work
   - install/bootstrap work
   - run/replay work
3. Identify which tasks can run in parallel.
4. Identify which tasks depend on prior completion.
5. Identify tasks requiring human approval.
6. Identify missing artifacts and missing verification gates.

Step 3 — Gap identification:
Create generated/gap_report.json and docs/GAP_ANALYSIS.md with:
- missing execution framework files
- missing schemas
- missing scripts
- missing task graph
- missing executable JSON packets
- missing proof templates
- missing verification steps
- missing multi-agent/lane columns
- missing parallelism metadata
- missing dependency metadata
- missing two-repo coordination details
- missing filesystem work boundaries
- missing rollback plans
- missing history/revision verification
- missing final completion criteria

Each gap must include:
- gap_id
- source_file_or_folder
- evidence
- severity
- affected phase
- recommended fix
- files to add/edit
- verification method

Step 4 — Apply execution-framework upgrades:
Apply the missing framework into the package by adding/updating the required files.

For every file edit or new file:
- record it in docs/APPLIED_UPGRADES.md
- include before/after intent
- include why the file was needed
- include the verification method
- include expected downstream consumer

Do not claim an upgrade unless a file actually changed or was created.

Step 5 — Generate task table:
Use the execution framework to generate:

generated/task_graph.csv

The task graph must include:
- all package implementation tasks
- all parallelizable tasks
- all dependency chains
- two-repo additive feature work
- filesystem work
- verification tasks
- final build completion tasks
- history verification tasks
- proof ledger tasks

Also generate:
- generated/task_graph.normalized.json
- generated/task_graph.index.json

Step 6 — Validate task table:
Run or create:

scripts/validate_task_graph.py

Validation must check:
- required columns exist
- task IDs are unique
- dependencies reference existing tasks
- no circular dependency exists
- parallel groups are valid
- required proof_uri exists
- allowed_paths and blocked_paths exist
- each executable task has verification_command or completion_gate
- each task has owner_lane, helper_id or owner_agent, and model_tag
- every two-repo task has repo_target and repo_path
- filesystem tasks have filesystem_scope
- no task can be Complete without proof_uri

Write:
generated/task_graph.validation_report.json

Step 7 — Generate JSON executable files:
Run or create:

scripts/task_graph_to_packets.py

Generate:
generated/execution_manifest.json
generated/execution_packets/<TASK_ID>.json

Execution packet rules:
- one task per packet
- no giant prose dumps
- no unrelated context
- only bounded task scope
- include dependency metadata
- include parallel group metadata
- include allowed/blocked paths
- include verification and proof requirements
- include agent lane/model tags
- include rollback plan

Step 8 — Create /goal execution loop:
Run or create:

scripts/goal_loop.py

The /goal loop must:
1. Load generated/task_graph.normalized.json.
2. Load proof_records/proof_ledger.jsonl if it exists.
3. Compute runnable tasks.
4. Dispatch tasks whose dependencies are satisfied.
5. Respect can_run_parallel, parallel_group, max_parallel, and depends_on.
6. Produce execution packets for runnable tasks.
7. Wait for proof records.
8. Merge proofs.
9. Update status.
10. Repeat until:
   - all tasks complete, or
   - a blocker/human approval task is reached, or
   - verification fails.

The loop must write:
state/goal_loop_state.json
generated/status_report.json

Step 9 — Multi-agent lane design:
Create or update:
docs/MULTI_AGENT_COLUMNS.md
templates/AGENT_LANE_TEMPLATE.json
schemas/agent_lane.schema.json

Include lane examples:
- lane_a_planning
- lane_b_repo_a
- lane_c_repo_b
- lane_d_filesystem
- lane_e_verification
- lane_f_release
- lane_g_history_scan

Each lane must support:
- helper_id
- model_tag
- agent_runtime
- shell_mode
- allowed_paths
- blocked_paths
- heartbeat_file
- logs_uri
- proof_uri
- current_task
- status

Step 10 — Two-repo parallel implementation support:
Add task graph examples and execution packet examples showing:
- repo A additive feature task
- repo B additive feature task
- shared filesystem task
- integration task depending on repo A and repo B
- verification task depending on both repos and filesystem artifacts
- release task depending on all verification

Step 11 — Proof and upgrade evidence:
Create proof templates and require proof for every upgrade.

For applied upgrades, generate:
generated/upgrade_report.json
docs/APPLIED_UPGRADES.md

Each upgrade record must include:
- upgrade_id
- gap_id
- files_added
- files_modified
- commands_run
- validation_result
- evidence_uri
- checksum
- rollback_plan

Step 12 — Final verification:
Run a final verification step using:

scripts/verify_history_and_completeness.py

This script must scan:
- current package files
- PACKAGE_MANIFEST.json
- README.md
- specs/
- prompts/
- codex/
- schemas/
- examples/
- helpers/
- sql/
- source/
- expected-output/
- history/
- generated task graph
- execution packets
- proof records
- applied upgrade report
- Drive file revision/history metadata where accessible

Final verification must prove:
1. Every source requirement appears in the task graph or is explicitly marked out of scope.
2. Every gap was either fixed or documented with blocker evidence.
3. Every added/modified file is listed in applied upgrades.
4. Every executable task has a JSON packet.
5. Every packet has dependencies, parallel metadata, verification gate, and proof URI.
6. Every completed task has a proof record.
7. Every proof record is included in proof_ledger.jsonl.
8. No package folder section was ignored.
9. No old history/revision requirement was dropped.
10. The /goal loop can compute a complete run path or stop at an explicit blocker.
11. The package can be handed to Codex/Claude/background helpers without rereading the full docs.

Write:
generated/final_verification_report.json
docs/FINAL_VERIFICATION.md

Step 13 — Completion criteria:
The task is only complete when all of the following exist and validate:
- docs/GAP_ANALYSIS.md
- docs/APPLIED_UPGRADES.md
- docs/FINAL_VERIFICATION.md
- generated/package_scan.json
- generated/gap_report.json
- generated/task_graph.csv
- generated/task_graph.normalized.json
- generated/task_graph.index.json
- generated/execution_manifest.json
- generated/execution_packets/*.json
- generated/status_report.json
- generated/final_verification_report.json
- schemas/*.schema.json
- scripts/*.py
- templates/TASK_GRAPH_TEMPLATE.csv
- proof_records/proof_ledger.jsonl or explicit explanation if no execution proofs exist yet

Final response format:
Return a concise completion report with:

1. Access status
2. Files scanned
3. Gaps found
4. Upgrades applied
5. Files created
6. Files modified
7. Task graph summary
8. Execution packet count
9. Parallel groups created
10. Dependency chains created
11. Two-repo work coverage
12. Filesystem work coverage
13. Proof records created
14. Verification commands run
15. Final verification result
16. Remaining blockers, if any
17. Links/paths to generated artifacts

Do not stop at recommendations. Apply the upgrades, generate the tables, generate the JSON executable files, and verify completeness.
