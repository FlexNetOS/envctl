//! Acceptance-criteria coverage (package prompts/ACCEPTANCE_CRITERIA.md, envctl half):
//! target parse/validate, package import, recipe from contract, run creation,
//! operations append events, evidence/artifact records, the R3 approval gate
//! (block + decide-as-events), validation queryability, replay verify, rollback
//! metadata. Each test runs on its own throwaway store.

use super::api::*;
use super::model::*;
use super::replay::ReplayMode;
use super::{canonical_json, sha256_hex, MigrationDb};
use serde_json::json;
use std::collections::BTreeMap;

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

fn target_descriptor(target_id: impl Into<String>, target_type: TargetType) -> TargetDescriptor {
    TargetDescriptor {
        schema_version: 1,
        target_id: target_id.into(),
        target_type,
        primary_root: "/tmp".into(),
        compare_root: None,
        output_root: "migration-artifacts".into(),
        include: vec!["**/*".into()],
        exclude: vec![],
        collectors: BTreeMap::from([("filesystem".into(), true)]),
        safety: TargetSafety {
            default_mode: HumanMode::ApprovalGated,
            max_auto_risk: Risk::R2,
            allow_network: false,
            allow_destructive: false,
        },
        artifact_contract: NamedVersion {
            name: "full-migration-artifact-contract".into(),
            version: NamedVersionValue::String("1.0.0".into()),
        },
        recipe: NamedVersion {
            name: "four-system-unify".into(),
            version: NamedVersionValue::String("1.0.0".into()),
        },
        metadata: serde_json::Map::new(),
    }
}

fn seed_run(db: &MigrationDb, tag: &str) -> Run {
    let target = db
        .register_target(TargetSpec {
            descriptor: target_descriptor(format!("target-{tag}"), TargetType::Mixed),
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
    let mut empty_id = target_descriptor("", TargetType::Codebase);
    let err = db.register_target(TargetSpec {
        descriptor: empty_id.clone(),
    });
    assert!(err.is_err(), "empty target_id must refuse");
    empty_id.target_id = "x".into();
    empty_id.schema_version = 0;
    let err = db.register_target(TargetSpec {
        descriptor: empty_id,
    });
    assert!(err.is_err(), "schema_version zero must refuse");
}

#[test]
fn duplicate_target_id_conflicts() {
    let db = temp_db("dup");
    let spec = TargetSpec {
        descriptor: target_descriptor("same", TargetType::Data),
    };
    db.register_target(spec.clone()).expect("first insert");
    assert!(db.register_target(spec).is_err(), "UNIQUE(target_id)");
}

#[test]
fn target_descriptor_json_and_yaml_normalize_to_same_hash() {
    let json = br#"{
        "schema_version": 1,
        "target_id": "same",
        "target_type": "codebase",
        "primary_root": "/tmp/repo",
        "safety": {
            "default_mode": "approval-gated",
            "max_auto_risk": "R2",
            "allow_network": false,
            "allow_destructive": false
        },
        "artifact_contract": {"name": "contract", "version": 1},
        "recipe": {"name": "recipe", "version": "1"}
    }"#;
    let yaml = br#"
schema_version: 1
target_id: same
target_type: codebase
primary_root: /tmp/repo
safety:
  default_mode: approval-gated
  max_auto_risk: R2
  allow_network: false
  allow_destructive: false
artifact_contract:
  name: contract
  version: 1
recipe:
  name: recipe
  version: "1"
"#;
    let json_descriptor = parse_target_descriptor(json).expect("valid JSON descriptor");
    let yaml_descriptor = parse_target_descriptor(yaml).expect("valid YAML descriptor");
    let json_value = serde_json::to_value(json_descriptor).unwrap();
    let yaml_value = serde_json::to_value(yaml_descriptor).unwrap();
    assert_eq!(canonical_json(&json_value), canonical_json(&yaml_value));
    assert_eq!(
        sha256_hex(canonical_json(&json_value).as_bytes()),
        sha256_hex(canonical_json(&yaml_value).as_bytes())
    );
}

#[test]
fn target_descriptor_parser_refuses_missing_and_invalid_nested_fields() {
    let missing_recipe = br#"
schema_version: 1
target_id: target
target_type: codebase
primary_root: /tmp
safety:
  default_mode: approval-gated
  max_auto_risk: R2
  allow_network: false
  allow_destructive: false
artifact_contract: {name: contract, version: 1}
"#;
    assert!(parse_target_descriptor(missing_recipe).is_err());

    let invalid_safety = br#"
schema_version: 1
target_id: target
target_type: codebase
primary_root: /tmp
safety:
  default_mode: fail-closed
  max_auto_risk: R2
  allow_network: false
  allow_destructive: false
artifact_contract: {name: contract, version: 1}
recipe: {name: recipe, version: 1}
"#;
    assert!(parse_target_descriptor(invalid_safety).is_err());
}

#[test]
fn target_descriptor_metadata_defaults_to_object_and_rejects_scalar() {
    let omitted = br#"
schema_version: 1
target_id: target
target_type: codebase
primary_root: /tmp
safety:
  default_mode: approval-gated
  max_auto_risk: R2
  allow_network: false
  allow_destructive: false
artifact_contract: {name: contract, version: 1}
recipe: {name: recipe, version: 1}
"#;
    let descriptor = parse_target_descriptor(omitted).expect("metadata is optional");
    let normalized = serde_json::to_value(descriptor).unwrap();
    assert_eq!(normalized["metadata"], json!({}));

    let scalar = br#"
schema_version: 1
target_id: target
target_type: codebase
primary_root: /tmp
safety:
  default_mode: approval-gated
  max_auto_risk: R2
  allow_network: false
  allow_destructive: false
artifact_contract: {name: contract, version: 1}
recipe: {name: recipe, version: 1}
metadata: invalid
"#;
    assert!(parse_target_descriptor(scalar).is_err());
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
fn illegal_transitions_refuse() {
    let db = temp_db("illegal");
    let run = seed_run(&db, "illegal");
    assert!(
        db.run_set_status(&run.id, RunStatus::Completed, ActorType::Agent, "t", None)
            .is_err(),
        "created -> completed is not an edge"
    );
}
