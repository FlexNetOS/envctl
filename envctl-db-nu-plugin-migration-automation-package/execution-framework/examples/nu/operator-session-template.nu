# Operator session template for envctl migration automation.

plugin use envctl

let targets = (envctl migration target list)
let target = ($targets | where name == "flexnetos-vs-lifeos" | first)
let plan = (envctl migration run plan --target $target.target_id --recipe codex-flexnetos-full-artifact-run --mode approval-gated)
let run = (envctl migration run start --plan $plan.plan_id --mode approval-gated)

envctl migration status $run.run_id
envctl migration proof $run.run_id
envctl migration replay $run.run_id --verify-hashes

let pending = (envctl migration approvals $run.run_id | where status == "pending" | first)
envctl migration approve --approval $pending.approval_id --run $run.run_id --reason "operator approved from structured Nu session"

envctl migration pause --run $run.run_id --reason "operator checkpoint"
envctl migration resume --run $run.run_id
