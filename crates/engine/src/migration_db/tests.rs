//! Acceptance-criteria coverage (package prompts/ACCEPTANCE_CRITERIA.md, envctl half):
//! target parse/validate, package import, recipe from contract, run creation,
//! operations append events, evidence/artifact records, the R3 approval gate
//! (block + decide-as-events), validation queryability, replay verify, rollback
//! metadata. Each test runs on its own throwaway store.

use super::api::*;
use super::model::*;
use super::replay::{ReplayMode, ReplayRequest};
use super::{sha256_hex, MigrationDb};
use serde_json::json;

fn temp_db(tag: &str) -> MigrationDb {
    let path = std::env::temp_dir().join(format!(
        "envctl-migration-db-test-{tag}-{}-{}.redb",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    MigrationDb::open(&path).expect("open temp store")
}

fn seed_run(db: &MigrationDb, tag: &str) -> Run {
    let target = db
        .register_target(TargetSpec {
            target_id: format!("target-{tag}"),
            target_type: TargetType::Mixed,
            primary_root: "/tmp".into(),
            compare_root: None,
            descriptor: json!({"systems": ["kb", "meta", "handoff", "idd"], "tag": tag}),
            safety_mode: "fail-closed".into(),
            max_auto_risk: Risk::R2,
        })
        .expect("register target");
    let contract = db
        .import_contract(
            "full-migration-artifact-contract",
            "1.0.0",
            json!({"artifacts": [{"id": "unified-idd-tree"}]}),
            None,
        )
        .expect("import contract");
    let recipe = db
        .create_recipe(
            "four-system-unify",
            "1.0.0",
            &contract.id,
            json!({"steps": [
                {"step_id": "import", "operation_type": "capture", "risk": "R1"},
                {"step_id": "export", "operation_type": "materialize", "risk": "R2"},
            ]}),
        )
        .expect("create recipe");
    db.create_run(
        RunSpec {
            target_id: target.id,
            recipe_id: recipe.id,
            human_mode: HumanMode::AgentOnly,
            initiated_by: Some("test".into()),
            sandbox_policy: None,
            approval_policy: None,
            tool_versions: Some(json!({"envctl": "test"})),
        },
        ActorType::Agent,
        "test-agent",
    )
    .expect("create run")
}

#[test]
fn target_validation_refuses_bad_specs() {
    let db = temp_db("badspec");
    let err = db.register_target(TargetSpec {
        target_id: "".into(),
        target_type: TargetType::Codebase,
        primary_root: "/tmp".into(),
        compare_root: None,
        descriptor: json!({}),
        safety_mode: "fail-closed".into(),
        max_auto_risk: Risk::R1,
    });
    assert!(err.is_err(), "empty target_id must refuse");
    let err = db.register_target(TargetSpec {
        target_id: "x".into(),
        target_type: TargetType::Codebase,
        primary_root: "/tmp".into(),
        compare_root: None,
        descriptor: json!("not an object"),
        safety_mode: "fail-closed".into(),
        max_auto_risk: Risk::R1,
    });
    assert!(err.is_err(), "non-object descriptor must refuse");
}

#[test]
fn duplicate_target_id_conflicts() {
    let db = temp_db("dup");
    let spec = TargetSpec {
        target_id: "same".into(),
        target_type: TargetType::Data,
        primary_root: "/tmp".into(),
        compare_root: None,
        descriptor: json!({"a": 1}),
        safety_mode: "fail-closed".into(),
        max_auto_risk: Risk::R1,
    };
    db.register_target(spec.clone()).expect("first insert");
    assert!(db.register_target(spec).is_err(), "UNIQUE(target_id)");
}

#[test]
fn operations_append_hash_chained_events() {
    let db = temp_db("events");
    let run = seed_run(&db, "events");
    let op = db
        .add_operation(
            OperationSpec {
                run_id: run.id.clone(),
                operation_type: "capture".into(),
                phase: Some("import".into()),
                risk: Risk::R1,
                idempotency_key: None,
                recipe_step_id: Some("import".into()),
                command_redacted: Some("codedb scan <root>".into()),
                input: Some(json!({"system": "kb"})),
                parent_operation_id: None,
            },
            ActorType::Agent,
            "test-agent",
        )
        .expect("add op");
    let (op, approval) = db
        .op_request_start(&op.id, ActorType::Agent, "test-agent")
        .expect("start");
    assert_eq!(op.status, OpStatus::Running, "R1 starts without approval");
    assert!(approval.is_none());
    db.op_set_status(
        &op.id,
        OpStatus::Succeeded,
        ActorType::Agent,
        "test-agent",
        None,
    )
    .expect("succeed");
    let events = db.events(&run.id).expect("events");
    assert!(events.len() >= 4, "created + queued + transitions");
    for (i, ev) in events.iter().enumerate() {
        assert_eq!(ev.event_seq as usize, i + 1, "dense seq");
        assert!(ev.event_hash.is_some(), "every event hashed");
        if i > 0 {
            assert_eq!(
                ev.previous_event_hash,
                events[i - 1].event_hash,
                "chain links"
            );
        }
    }
}

#[test]
fn idempotent_operation_returns_existing() {
    let db = temp_db("idem");
    let run = seed_run(&db, "idem");
    let spec = OperationSpec {
        run_id: run.id.clone(),
        operation_type: "capture".into(),
        phase: None,
        risk: Risk::R1,
        idempotency_key: None,
        recipe_step_id: Some("import".into()),
        command_redacted: None,
        input: Some(json!({"system": "kb"})),
        parent_operation_id: None,
    };
    let a = db
        .add_operation(spec.clone(), ActorType::Agent, "t")
        .expect("first");
    let b = db
        .add_operation(spec, ActorType::Agent, "t")
        .expect("second");
    assert_eq!(a.id, b.id, "same idempotency key returns the recorded op");
}

#[test]
fn r3_gate_blocks_until_agent_approval_decides_as_events() {
    let db = temp_db("gate");
    let run = seed_run(&db, "gate");
    let op = db
        .add_operation(
            OperationSpec {
                run_id: run.id.clone(),
                operation_type: "materialize".into(),
                phase: Some("export".into()),
                risk: Risk::R3,
                idempotency_key: None,
                recipe_step_id: Some("export".into()),
                command_redacted: Some("write unified tree".into()),
                input: None,
                parent_operation_id: None,
            },
            ActorType::Agent,
            "test-agent",
        )
        .expect("add op");
    let (op, approval) = db
        .op_request_start(&op.id, ActorType::Agent, "test-agent")
        .expect("request start");
    assert_eq!(op.status, OpStatus::AwaitingApproval, "R3 blocks");
    let approval = approval.expect("approval row created");
    assert_eq!(approval.status, ApprovalStatus::Open);

    // The op cannot run while the approval is open.
    assert!(db
        .op_set_status(&op.id, OpStatus::Succeeded, ActorType::Agent, "t", None)
        .is_err());

    // Agent reviewer decides through the same surface a human would.
    db.approval_decide(
        &approval.id,
        ApprovalDecision::Approve,
        ActorType::Agent,
        "agent-reviewer",
        "evidence checked against contract; parity baselines recorded",
        Some(json!(["evidence-000001"])),
    )
    .expect("decide");
    let (op, none) = db
        .op_request_start(&op.id, ActorType::Agent, "test-agent")
        .expect("restart after approval");
    assert!(none.is_none(), "approved: no second approval row");
    assert_eq!(op.status, OpStatus::Running);

    let events = db.events(&run.id).expect("events");
    let types: Vec<&str> = events.iter().map(|e| e.event_type.as_str()).collect();
    assert!(types.contains(&"approval.requested"));
    assert!(types.contains(&"approval.decided"));
}

#[test]
fn denial_is_terminal_for_the_operation() {
    let db = temp_db("deny");
    let run = seed_run(&db, "deny");
    let op = db
        .add_operation(
            OperationSpec {
                run_id: run.id.clone(),
                operation_type: "destructive".into(),
                phase: None,
                risk: Risk::R5,
                idempotency_key: None,
                recipe_step_id: None,
                command_redacted: None,
                input: None,
                parent_operation_id: None,
            },
            ActorType::Agent,
            "t",
        )
        .expect("add");
    let (_, approval) = db
        .op_request_start(&op.id, ActorType::Agent, "t")
        .expect("start");
    let approval = approval.expect("R5 must gate");
    db.approval_decide(
        &approval.id,
        ApprovalDecision::Deny,
        ActorType::Agent,
        "agent-reviewer",
        "deny-by-default: no evidence attached",
        None,
    )
    .expect("deny");
    let op = db.operation(&op.id).expect("op");
    assert_eq!(op.status, OpStatus::Denied);
}

#[test]
fn evidence_artifacts_validations_rollbacks_recorded_and_queryable() {
    let db = temp_db("records");
    let run = seed_run(&db, "records");
    let ev = db
        .add_evidence(
            &run.id,
            None,
            "/tmp/evidence.log",
            "raw_log",
            Some("ab".repeat(32).as_str()),
            false,
            Some(json!({"lines": 10})),
            ActorType::Agent,
            "t",
        )
        .expect("evidence");
    assert!(ev.sha256.is_some());
    db.upsert_artifact(
        &run.id,
        "unified-idd-tree",
        "Unified .idd export",
        Some("tree"),
        ArtifactStatus::Partial,
        Some("/tmp/out"),
        None,
        None,
        Some(json!([ev.id])),
        None,
        ActorType::Agent,
        "t",
    )
    .expect("artifact");
    // Upsert refreshes, does not duplicate.
    db.upsert_artifact(
        &run.id,
        "unified-idd-tree",
        "Unified .idd export",
        Some("tree"),
        ArtifactStatus::Complete,
        Some("/tmp/out"),
        Some("cd".repeat(32).as_str()),
        None,
        None,
        None,
        ActorType::Agent,
        "t",
    )
    .expect("artifact update");
    let arts = db.artifacts(&run.id).expect("artifacts");
    assert_eq!(arts.len(), 1);
    assert_eq!(arts[0].status, ArtifactStatus::Complete);

    db.add_validation(
        ValidationSpec {
            run_id: run.id.clone(),
            validator: "byte-parity".into(),
            status: ValidationStatus::Pass,
            artifact_id: Some("unified-idd-tree".into()),
            operation_id: None,
            details: Some(json!({"files": 419, "mismatched": 0})),
            evidence: None,
        },
        ActorType::Agent,
        "t",
    )
    .expect("validation");
    let score = db.view_scorecard(&run.id).expect("scorecard");
    assert_eq!(score.pass_count, 1);

    let rb = db
        .plan_rollback(
            &run.id,
            None,
            "point-back-at-originals",
            json!({"originals": "untouched; export wrote a new tree"}),
            ActorType::Agent,
            "t",
        )
        .expect("rollback");
    assert_eq!(rb.status, RollbackStatus::Planned);
    assert_eq!(db.rollbacks(&run.id).expect("rollbacks").len(), 1);
}

#[test]
fn run_lifecycle_and_replay_verify() {
    let db = temp_db("replay");
    let run = seed_run(&db, "replay");
    db.run_set_status(&run.id, RunStatus::Planning, ActorType::Agent, "t", None)
        .expect("planning");
    db.run_set_status(&run.id, RunStatus::Running, ActorType::Agent, "t", None)
        .expect("running");
    db.run_set_status(&run.id, RunStatus::Validating, ActorType::Agent, "t", None)
        .expect("validating");
    let run = db
        .complete_run(&run.id, ActorType::Agent, "t")
        .expect("complete");
    assert_eq!(run.status, RunStatus::Completed);
    assert!(run.reproducibility_hash.is_some());

    let report = db
        .replay(&run.id, ReplayMode::VerifyOnly, false)
        .expect("replay");
    assert!(
        report.ok,
        "verify must pass on an untampered run: {:?}",
        report.checks
    );
    let status = db.view_run_status(&run.id).expect("status view");
    assert_eq!(status.status, RunStatus::Completed);
    let readiness = db.view_replay_readiness(&run.id).expect("readiness");
    assert!(readiness.has_reproducibility_hash);

    let bundle = db.export_run(&run.id).expect("bundle");
    assert_eq!(bundle.run.id, run.id);
    assert!(!bundle.events.is_empty());
}

#[test]
fn replay_reconstructs_scoped_plan_and_rehashes_evidence() {
    let db = temp_db("reproduce");
    let run = seed_run(&db, "reproduce");
    let root = std::env::temp_dir().join(format!(
        "envctl-replay-root-{}-{}",
        std::process::id(),
        run.id
    ));
    std::fs::create_dir_all(&root).expect("create replay root");
    let evidence_path = root.join("proof.json");
    std::fs::write(&evidence_path, b"{\"verified\":true}\n").expect("write proof");
    let evidence_hash = sha256_hex(&std::fs::read(&evidence_path).expect("read proof"));

    let op = db
        .add_operation(
            OperationSpec {
                run_id: run.id.clone(),
                operation_type: "capture".into(),
                phase: Some("import".into()),
                risk: Risk::R1,
                idempotency_key: None,
                recipe_step_id: Some("import".into()),
                command_redacted: Some("codedb capture <target>".into()),
                input: Some(json!({"packet": "REQ-027"})),
                parent_operation_id: None,
            },
            ActorType::Agent,
            "test-agent",
        )
        .expect("add deterministic operation");
    db.add_evidence(
        &run.id,
        Some(&op.id),
        "proof.json",
        "proof_record",
        Some(&evidence_hash),
        false,
        Some(json!({"required_replay_input": true})),
        ActorType::Agent,
        "test-agent",
    )
    .expect("record evidence");
    db.add_checkpoint(
        &run.id,
        Some(&op.id),
        "artifact-boundary",
        "proof.json",
        Some(&evidence_hash),
        None,
        ActorType::Agent,
        "test-agent",
    )
    .expect("record checkpoint");
    db.run_set_status(
        &run.id,
        RunStatus::Planning,
        ActorType::Agent,
        "test-agent",
        None,
    )
    .expect("planning");
    db.run_set_status(
        &run.id,
        RunStatus::Running,
        ActorType::Agent,
        "test-agent",
        None,
    )
    .expect("running");
    db.run_set_status(
        &run.id,
        RunStatus::Validating,
        ActorType::Agent,
        "test-agent",
        None,
    )
    .expect("validating");
    db.complete_run(&run.id, ActorType::Agent, "test-agent")
        .expect("complete");

    let report = db
        .reproduce(ReplayRequest {
            replay_id: "replay-scoped".into(),
            run_id: run.id.clone(),
            mode: ReplayMode::DryRunPlan,
            requested_by: "test-agent".into(),
            operation_ids: vec![op.id.clone()],
            target_descriptor_id: None,
            reason: Some("acceptance test".into()),
            replay_root: Some(root),
        })
        .expect("reproduce");
    assert_eq!(report.status, "pass");
    assert!(report.ok);
    assert_eq!(report.operation_replay_plan.len(), 1);
    assert_eq!(report.operation_replay_plan[0].checkpoint_refs.len(), 1);
    assert_eq!(report.hash_checks[0].status, "match");
    assert!(!report.replay_input_hash.is_empty());
    assert!(report.errors.is_empty());

    let missing = db.reproduce(ReplayRequest {
        replay_id: "replay-unknown-op".into(),
        run_id: run.id,
        mode: ReplayMode::DryRunPlan,
        requested_by: "test-agent".into(),
        operation_ids: vec!["op-does-not-exist".into()],
        target_descriptor_id: None,
        reason: None,
        replay_root: None,
    });
    assert!(missing.is_err(), "unknown operation ids fail closed");
}

#[test]
fn apply_replay_blocks_open_approval_and_non_determinism() {
    let db = temp_db("replay-apply-blocked");
    let run = seed_run(&db, "replay-apply-blocked");
    let op = db
        .add_operation(
            OperationSpec {
                run_id: run.id.clone(),
                operation_type: "manual_operator".into(),
                phase: Some("cutover".into()),
                risk: Risk::R4,
                idempotency_key: None,
                recipe_step_id: None,
                command_redacted: Some("operator performs cutover".into()),
                input: Some(json!({
                    "non_deterministic": true,
                    "replay_note": "requires an approved operator window"
                })),
                parent_operation_id: None,
            },
            ActorType::Agent,
            "test-agent",
        )
        .expect("add manual operation");
    db.op_request_start(&op.id, ActorType::Agent, "test-agent")
        .expect("request approval");
    db.run_set_status(
        &run.id,
        RunStatus::Planning,
        ActorType::Agent,
        "test-agent",
        None,
    )
    .expect("planning");
    db.run_set_status(
        &run.id,
        RunStatus::Running,
        ActorType::Agent,
        "test-agent",
        None,
    )
    .expect("running");
    db.run_set_status(
        &run.id,
        RunStatus::Validating,
        ActorType::Agent,
        "test-agent",
        None,
    )
    .expect("validating");
    db.complete_run(&run.id, ActorType::Agent, "test-agent")
        .expect("complete fixture");

    let report = db
        .reproduce(ReplayRequest {
            replay_id: "replay-apply".into(),
            run_id: run.id,
            mode: ReplayMode::Apply,
            requested_by: "test-agent".into(),
            operation_ids: vec![op.id],
            target_descriptor_id: None,
            reason: Some("prove fail-closed apply".into()),
            replay_root: None,
        })
        .expect("apply report");
    assert_eq!(report.status, "blocked");
    assert!(!report.ok);
    assert_eq!(report.required_approvals.len(), 1);
    assert_eq!(report.non_deterministic_operations.len(), 1);
    assert!(report.safe_next_action.contains("request human approval"));
}

#[test]
fn illegal_transitions_refuse() {
    let db = temp_db("illegal");
    let run = seed_run(&db, "illegal");
    assert!(
        db.run_set_status(&run.id, RunStatus::Completed, ActorType::Agent, "t", None)
            .is_err(),
        "created -> completed is not an edge"
    );
}
