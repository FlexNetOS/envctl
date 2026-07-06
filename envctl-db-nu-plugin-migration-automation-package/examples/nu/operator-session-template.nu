# Operator session template for envctl migration automation.
# Adapt command names to the actual nu_plugin command signatures after implementation.

plugin use envctl

let target = (envctl migration target add --descriptor examples/target-descriptors/flexnetos-vs-lifeos.yaml)
let plan = (envctl migration plan --target $target.target_id --recipe codex-flexnetos-full-artifact-run)
let run = (envctl migration run --plan $plan.plan_id --mode approval-gated)

envctl migration status $run.run_id
envctl migration timeline $run.run_id
envctl migration approvals $run.run_id
envctl migration artifacts $run.run_id
envctl migration validations $run.run_id
envctl migration replay $run.run_id --verify-hashes
