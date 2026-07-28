//! Acceptance-criteria coverage (package prompts/ACCEPTANCE_CRITERIA.md, envctl half):
//! target parse/validate, package import, recipe from contract, run creation,
//! operations append events, evidence/artifact records, the R3 approval gate
//! (block + decide-as-events), validation queryability, replay verify, rollback
//! metadata. Each test runs on its own throwaway store.

use super::api::*;
use super::model::*;
use super::replay::{ReplayMode, ReplayRequest, ReplayRequestMode, ReplayResultStatus};
use super::MigrationDb;
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
fn duplicate_target_id_upserts_descriptor() {
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
    let first = db.register_target(spec.clone()).expect("first insert");
    let second = db.register_target(spec).expect("idempotent upsert");
    assert_eq!(first.id, second.id, "natural target id remains stable");
    assert_eq!(
        db.targets().expect("targets").len(),
        1,
        "upsert does not duplicate"
    );
}

#[test]
fn descriptor_is_schema_validated_and_authoritative() {
    let raw = json!({
        "schema_version": 1,
        "target_id": "from-descriptor",
        "target_type": "infrastructure",
        "primary_root": "/infra",
        "safety": {"default_mode": "approval-gated", "max_auto_risk": "R3", "allow_network": false, "allow_destructive": false},
        "artifact_contract": {"name": "contract", "version": 1},
        "recipe": {"name": "recipe", "version": "1.0.0"}
    });
    let (descriptor, parsed_raw) = super::descriptor::parse_target_descriptor(
        &serde_json::to_string(&raw).expect("json"),
        Some("json"),
    )
    .expect("valid descriptor");
    let target = temp_db("descriptor")
        .register_target(descriptor.into_spec(parsed_raw).expect("spec"))
        .expect("register");
    assert_eq!(target.target_id, "from-descriptor");
    assert_eq!(target.target_type, TargetType::Infrastructure);
    assert_eq!(target.safety_mode, "approval-gated");
    assert_eq!(target.max_auto_risk, Risk::R3);
}

#[test]
fn yaml_descriptor_parses_and_missing_safety_is_rejected() {
    let yaml = r#"
schema_version: 1
target_id: yaml-target
target_type: codebase
primary_root: /repo
safety:
  default_mode: agent-only
  max_auto_risk: R2
  allow_network: false
  allow_destructive: false
artifact_contract:
  name: contract
  version: 1
recipe:
  name: recipe
  version: 1
"#;
    let (descriptor, raw) =
        super::descriptor::parse_target_descriptor(yaml, Some("yaml")).expect("yaml descriptor");
    assert_eq!(
        descriptor.into_spec(raw).expect("spec").target_id,
        "yaml-target"
    );

    let invalid =
        r#"{"schema_version":1,"target_id":"x","target_type":"codebase","primary_root":"/x"}"#;
    assert!(super::descriptor::parse_target_descriptor(invalid, Some("json")).is_err());
}

#[test]
fn descriptor_extension_fields_are_accepted() {
    let raw = json!({
        "schema_version": 1,
        "target_id": "extensible-target",
        "target_type": "mixed",
        "primary_root": "/repo",
        "compare_root": null,
        "output_root": "migration-artifacts",
        "include": ["src/**", "tools/**"],
        "exclude": [".git/**", "node_modules/**"],
        "safety": {
            "default_mode": "agent-only",
            "max_auto_risk": "R2",
            "allow_network": false,
            "allow_destructive": false,
        },
        "collectors": {
            "filesystem": true,
            "git": true,
            "apis": false,
        },
        "artifact_contract": {"name": "contract", "version": 1},
        "recipe": {"name": "recipe", "version": "1.0.0"},
        "metadata": {"purpose": "integration"},
        "plugin_hints": {"scan": "deep"},
    });

    let (descriptor, parsed_raw) = super::descriptor::parse_target_descriptor(
        &serde_json::to_string(&raw).unwrap(),
        Some("json"),
    )
    .expect("extended descriptor");
    assert_eq!(descriptor.output_root, "migration-artifacts");
    assert_eq!(descriptor.include.len(), 2);
    assert_eq!(descriptor.exclude.len(), 2);
    assert_eq!(descriptor.collectors.get("filesystem"), Some(&true));
    assert_eq!(descriptor.metadata, json!({"purpose": "integration"}));
    assert_eq!(
        descriptor.extensions.get("plugin_hints"),
        Some(&json!({"scan":"deep"}))
    );

    let target = descriptor
        .into_spec(parsed_raw)
        .expect("target spec from extended descriptor");
    assert_eq!(target.target_id, "extensible-target");
    assert_eq!(target.target_type, TargetType::Mixed);
    assert_eq!(target.safety_mode, "agent-only");
    assert_eq!(target.max_auto_risk, Risk::R2);
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
fn checkpoints_are_idempotent_and_rollback_handles_fail_closed() {
    let db = temp_db("rollback-boundary");
    let run = seed_run(&db, "rollback-boundary");
    let safe = db
        .add_operation(
            OperationSpec {
                run_id: run.id.clone(),
                operation_type: "safe".into(),
                phase: None,
                risk: Risk::R2,
                idempotency_key: None,
                recipe_step_id: None,
                command_redacted: None,
                input: None,
                parent_operation_id: None,
            },
            ActorType::Agent,
            "t",
        )
        .expect("safe operation");
    let first = db
        .add_checkpoint(
            &run.id,
            Some(&safe.id),
            "artifact",
            "generated/boundary.json",
            None,
            Some(json!({"repeat_safe": true})),
            ActorType::Agent,
            "t",
        )
        .expect("checkpoint");
    let duplicate = db
        .add_checkpoint(
            &run.id,
            Some(&safe.id),
            "artifact",
            "generated/boundary.json",
            None,
            Some(json!({"repeat_safe": true})),
            ActorType::Agent,
            "t",
        )
        .expect("idempotent checkpoint");
    assert_eq!(first.id, duplicate.id);
    assert_eq!(db.checkpoints(&run.id).expect("checkpoints").len(), 1);
    assert!(db
        .add_checkpoint(
            &run.id,
            Some(&safe.id),
            "artifact",
            "secrets/token.txt",
            None,
            None,
            ActorType::Agent,
            "t"
        )
        .is_err());

    let handle = db
        .plan_rollback(
            &run.id,
            Some(&safe.id),
            "verify-boundary",
            json!({"checkpoint_id": first.id}),
            ActorType::Agent,
            "t",
        )
        .expect("safe rollback");
    let running = db
        .rollback_set_status(
            &handle.id,
            RollbackStatus::Running,
            ActorType::Agent,
            "t",
            None,
        )
        .expect("running");
    let completed = db
        .rollback_set_status(
            &running.id,
            RollbackStatus::Succeeded,
            ActorType::Agent,
            "t",
            Some(json!({"verified": true})),
        )
        .expect("succeeded");
    assert_eq!(completed.status, RollbackStatus::Succeeded);
    assert!(db
        .rollback_set_status(
            &completed.id,
            RollbackStatus::Running,
            ActorType::Agent,
            "t",
            None
        )
        .is_err());
}

#[test]
fn risky_rollback_requires_approval_before_planning() {
    let db = temp_db("rollback-approval");
    let run = seed_run(&db, "rollback-approval");
    let risky = db
        .add_operation(
            OperationSpec {
                run_id: run.id.clone(),
                operation_type: "risky".into(),
                phase: None,
                risk: Risk::R4,
                idempotency_key: None,
                recipe_step_id: None,
                command_redacted: None,
                input: None,
                parent_operation_id: None,
            },
            ActorType::Agent,
            "t",
        )
        .expect("risky operation");
    let handle = db
        .plan_rollback(
            &run.id,
            Some(&risky.id),
            "restore-boundary",
            json!({"checkpoint_ref": "history/manifest.json"}),
            ActorType::Agent,
            "t",
        )
        .expect("risky rollback");
    assert_eq!(handle.status, RollbackStatus::AwaitingApproval);
    assert!(db
        .rollback_set_status(
            &handle.id,
            RollbackStatus::Planned,
            ActorType::Agent,
            "t",
            None
        )
        .is_err());
    let approval = db
        .approvals(&run.id)
        .expect("approval")
        .pop()
        .expect("approval row");
    db.approval_decide(
        &approval.id,
        ApprovalDecision::Approve,
        ActorType::Human,
        "reviewer",
        "approved rollback",
        None,
    )
    .expect("approve");
    assert_eq!(
        db.rollback_set_status(
            &handle.id,
            RollbackStatus::Planned,
            ActorType::Agent,
            "t",
            None
        )
        .expect("approved plan")
        .status,
        RollbackStatus::Planned
    );
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
fn replay_request_rehashes_recorded_evidence() {
    let db = temp_db("replay-request");
    let run = seed_run(&db, "replay-request");
    let evidence_path = "Cargo.toml";
    let evidence_hash = super::sha256_hex(
        &std::fs::read(evidence_path).expect("workspace Cargo.toml must be available to tests"),
    );
    db.add_evidence(
        &run.id,
        None,
        evidence_path,
        "source",
        Some(&evidence_hash),
        false,
        None,
        ActorType::Agent,
        "t",
    )
    .expect("record evidence");
    db.run_set_status(&run.id, RunStatus::Planning, ActorType::Agent, "t", None)
        .expect("planning");
    db.run_set_status(&run.id, RunStatus::Running, ActorType::Agent, "t", None)
        .expect("running");
    db.run_set_status(&run.id, RunStatus::Validating, ActorType::Agent, "t", None)
        .expect("validating");
    db.complete_run(&run.id, ActorType::Agent, "t")
        .expect("complete");

    let result = db
        .replay_request(
            ReplayRequest {
                replay_id: "replay-request-1".into(),
                run_id: run.id.clone(),
                mode: ReplayRequestMode::DryRun,
                requested_by: "test-agent".into(),
                operation_ids: Vec::new(),
                target_descriptor_id: None,
                reason: None,
            },
            true,
        )
        .expect("replay request");

    assert_eq!(result.status, ReplayResultStatus::Pass);
    assert_eq!(result.hash_status.evidence_matches, 1);
    assert!(result.hash_status.evidence_mismatches.is_empty());
    assert!(result.missing_evidence.is_empty());
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
