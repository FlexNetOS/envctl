//! `envctl migration` — the CLI face of `envctl_engine::migration_db`, verbs per
//! the package's specs/command-surface.md: package/contract/recipe/target/plan/
//! run/approval/control/rollback plus the record verbs (op/evidence/artifact/
//! validation/edge/checkpoint/session/event) the pipeline drives.
//!
//! Agent-first: non-interactive, structured output only (pretty JSON for humans,
//! compact JSON with --json), real exit codes (any engine error exits non-zero).

use anyhow::{anyhow, Context};
use clap::Subcommand;
use envctl_engine::migration_db::{
    self, ActorType, ApprovalDecision, ArtifactStatus, HumanMode, MigrationDb, OpStatus,
    OperationSpec, ReplayMode, Risk, RunSpec, RunStatus, TargetSpec, TargetType, ValidationSpec,
    ValidationStatus,
};
use std::path::PathBuf;

#[derive(Subcommand)]
pub enum MigrationCmd {
    /// Inspect or import external prompt packages (versioned, content-hashed).
    #[command(subcommand)]
    Package(PackageCmd),
    /// Versioned artifact contracts (changes create new versions, never edits).
    #[command(subcommand)]
    Contract(ContractCmd),
    /// Migration recipes bound to a contract version.
    #[command(subcommand)]
    Recipe(RecipeCmd),
    /// Target descriptors (the systems a run imports/exports).
    #[command(subcommand)]
    Target(TargetCmd),
    /// Runs: create/start/status/events/ops/artifacts/validations/replay/export.
    #[command(subcommand)]
    Run(RunCmd),
    /// Operations: queue, request-start (approval-gated at R3+), complete.
    #[command(subcommand)]
    Op(OpCmd),
    /// Approval queue: list open, approve, deny — decisions append events.
    #[command(subcommand)]
    Approval(ApprovalCmd),
    /// Run control: pause / resume / cancel.
    #[command(subcommand)]
    Control(ControlCmd),
    /// Rollback metadata: plan and record execution.
    #[command(subcommand)]
    Rollback(RollbackCmd),
    /// Record evidence for a run (uri + kind + sha256).
    Evidence {
        run: String,
        #[arg(long)]
        uri: String,
        #[arg(long, default_value = "raw_log")]
        kind: String,
        /// Explicit sha256; or use --hash-file to hash the uri path now.
        #[arg(long)]
        sha256: Option<String>,
        #[arg(long, default_value_t = false)]
        hash_file: bool,
        #[arg(long)]
        op: Option<String>,
        #[arg(long)]
        metadata: Option<String>,
        #[command(flatten)]
        actor: ActorArgs,
    },
    /// Record/refresh an artifact row (upsert by artifact id).
    Artifact {
        run: String,
        #[arg(long)]
        artifact_id: String,
        #[arg(long)]
        title: String,
        #[arg(long)]
        artifact_type: Option<String>,
        #[arg(long, default_value = "partial")]
        status: String,
        #[arg(long)]
        path: Option<String>,
        #[arg(long)]
        content_hash: Option<String>,
        #[arg(long, default_value_t = false)]
        hash_file: bool,
        #[arg(long)]
        op: Option<String>,
        #[command(flatten)]
        actor: ActorArgs,
    },
    /// Record a validation result.
    Validation {
        run: String,
        #[arg(long)]
        validator: String,
        #[arg(long)]
        status: String,
        #[arg(long)]
        artifact: Option<String>,
        #[arg(long)]
        op: Option<String>,
        #[arg(long)]
        details: Option<String>,
        #[command(flatten)]
        actor: ActorArgs,
    },
    /// Record a dependency/data-flow graph edge.
    Edge {
        run: String,
        #[arg(long)]
        from: String,
        #[arg(long)]
        to: String,
        #[arg(long, default_value = "depends_on")]
        edge_type: String,
        #[arg(long)]
        artifact: Option<String>,
    },
    /// Record a checkpoint (snapshot/rollback anchor).
    Checkpoint {
        run: String,
        #[arg(long)]
        kind: String,
        #[arg(long, name = "ref")]
        reference: String,
        #[arg(long)]
        hash: Option<String>,
        #[command(flatten)]
        actor: ActorArgs,
    },
    /// Record an agent or plugin session.
    #[command(subcommand)]
    Session(SessionCmd),
    /// Append a custom event to a run's hash-chained ledger.
    Event {
        run: String,
        #[arg(long)]
        event_type: String,
        #[arg(long)]
        phase: Option<String>,
        #[arg(long)]
        op: Option<String>,
        #[arg(long, default_value = "{}")]
        payload: String,
        #[command(flatten)]
        actor: ActorArgs,
    },
}

#[derive(clap::Args)]
pub struct ActorArgs {
    /// Actor type appended to the ledger: system|agent|human|plugin|external.
    #[arg(long, default_value = "agent")]
    actor: String,
    /// Actor identity (defaults to $ENVCTL_ACTOR_ID, then "envctl-cli").
    #[arg(long)]
    actor_id: Option<String>,
}

impl ActorArgs {
    fn resolve(&self) -> anyhow::Result<(ActorType, String)> {
        let actor = ActorType::parse(&self.actor)?;
        let id = self
            .actor_id
            .clone()
            .or_else(|| std::env::var("ENVCTL_ACTOR_ID").ok())
            .unwrap_or_else(|| "envctl-cli".to_string());
        Ok((actor, id))
    }
}

#[derive(Subcommand)]
pub enum PackageCmd {
    /// Content-hash a package directory without recording it.
    Inspect {
        path: PathBuf,
    },
    /// Import (record) a package: name + path + content hash + manifest.
    Import {
        name: String,
        path: PathBuf,
    },
    List,
}

#[derive(Subcommand)]
pub enum ContractCmd {
    /// Import a contract version from a JSON file.
    Import {
        name: String,
        version: String,
        #[arg(long)]
        file: PathBuf,
        #[arg(long)]
        package: Option<String>,
    },
    List,
    Show {
        id: String,
    },
}

#[derive(Subcommand)]
pub enum RecipeCmd {
    /// Create a recipe version from a JSON file (must contain a steps array).
    Create {
        name: String,
        version: String,
        #[arg(long)]
        contract: String,
        #[arg(long)]
        file: PathBuf,
    },
    List,
    Show {
        id: String,
    },
}

#[derive(Subcommand)]
pub enum TargetCmd {
    /// Register a target descriptor (JSON file), validated + content-hashed.
    Add {
        target_id: String,
        #[arg(long, default_value = "mixed")]
        target_type: String,
        #[arg(long)]
        primary_root: String,
        #[arg(long)]
        compare_root: Option<String>,
        #[arg(long)]
        descriptor: PathBuf,
        #[arg(long, default_value = "fail-closed")]
        safety_mode: String,
        #[arg(long, default_value = "R2")]
        max_auto_risk: String,
    },
    List,
    Show {
        target_id: String,
    },
    /// Parse + validate a descriptor file without registering it.
    Validate {
        descriptor: PathBuf,
    },
}

#[derive(Subcommand)]
pub enum RunCmd {
    /// Create a run (target row-id or natural id + recipe id).
    Create {
        #[arg(long)]
        target: String,
        #[arg(long)]
        recipe: String,
        #[arg(long, default_value = "agent-only")]
        human_mode: String,
        #[arg(long)]
        initiated_by: Option<String>,
        #[arg(long)]
        tool_versions: Option<String>,
        #[command(flatten)]
        actor: ActorArgs,
    },
    /// created -> planning -> running (the happy-path start).
    Start {
        run: String,
        #[command(flatten)]
        actor: ActorArgs,
    },
    /// running -> validating.
    Validate {
        run: String,
        #[command(flatten)]
        actor: ActorArgs,
    },
    /// validating -> completed; stamps the reproducibility hash.
    Complete {
        run: String,
        #[command(flatten)]
        actor: ActorArgs,
    },
    /// Latest-status view (one run, or --all).
    Status {
        run: Option<String>,
        #[arg(long, default_value_t = false)]
        all: bool,
    },
    /// The live timeline (hash-chained events joined to operations).
    Events {
        run: String,
    },
    /// Operations for a run (--queue for the live queue only).
    Ops {
        run: String,
        #[arg(long, default_value_t = false)]
        queue: bool,
    },
    Artifacts {
        run: String,
    },
    /// Validation rows + scorecard.
    Validations {
        run: String,
    },
    /// Replay: verify-only | dry-run-plan (execute-again refuses at the engine).
    Replay {
        run: String,
        #[arg(long, default_value = "verify-only")]
        mode: String,
        /// Re-hash evidence/artifact files on disk, not just recorded hashes.
        #[arg(long, default_value_t = false)]
        verify_files: bool,
    },
    /// Replay readiness view (hash coverage + open approvals).
    Readiness {
        run: String,
    },
    /// Export the whole run as one bundle (the plugin read surface).
    Export {
        run: String,
    },
    List,
}

#[derive(Subcommand)]
pub enum OpCmd {
    /// Queue an operation (idempotent on the derived idempotency key).
    Add {
        run: String,
        #[arg(long)]
        operation_type: String,
        #[arg(long)]
        phase: Option<String>,
        #[arg(long, default_value = "R1")]
        risk: String,
        #[arg(long)]
        step: Option<String>,
        #[arg(long)]
        command: Option<String>,
        #[arg(long)]
        input: Option<String>,
        #[command(flatten)]
        actor: ActorArgs,
    },
    /// Request start: R3+ (or above the target's max auto risk) parks in
    /// awaiting_approval with an open approval; safe risks start running.
    Start {
        op: String,
        #[command(flatten)]
        actor: ActorArgs,
    },
    /// Terminal transition: succeeded (default) or --failed with detail.
    Complete {
        op: String,
        #[arg(long, default_value_t = false)]
        failed: bool,
        #[arg(long)]
        detail: Option<String>,
        #[command(flatten)]
        actor: ActorArgs,
    },
    Show {
        op: String,
    },
}

#[derive(Subcommand)]
pub enum ApprovalCmd {
    /// Open approvals (all runs, or --run).
    List {
        #[arg(long)]
        run: Option<String>,
    },
    Approve {
        approval: String,
        #[arg(long)]
        by: String,
        #[arg(long)]
        reason: String,
        #[arg(long)]
        evidence: Option<String>,
        #[command(flatten)]
        actor: ActorArgs,
    },
    Deny {
        approval: String,
        #[arg(long)]
        by: String,
        #[arg(long)]
        reason: String,
        #[arg(long)]
        evidence: Option<String>,
        #[command(flatten)]
        actor: ActorArgs,
    },
}

#[derive(Subcommand)]
pub enum ControlCmd {
    Pause {
        run: String,
        #[command(flatten)]
        actor: ActorArgs,
    },
    Resume {
        run: String,
        #[command(flatten)]
        actor: ActorArgs,
    },
    Cancel {
        run: String,
        #[command(flatten)]
        actor: ActorArgs,
    },
}

#[derive(Subcommand)]
pub enum RollbackCmd {
    /// Record a rollback plan (JSON) for a run/operation.
    Plan {
        run: String,
        #[arg(long, default_value = "point-back-at-originals")]
        rollback_type: String,
        #[arg(long)]
        plan: String,
        #[arg(long)]
        op: Option<String>,
        #[command(flatten)]
        actor: ActorArgs,
    },
    List {
        run: String,
    },
}

#[derive(Subcommand)]
pub enum SessionCmd {
    Agent {
        #[arg(long)]
        run: Option<String>,
        #[arg(long)]
        name: String,
        #[arg(long)]
        model: Option<String>,
        #[arg(long)]
        authority: Option<String>,
        #[arg(long)]
        session: Option<String>,
    },
    Plugin {
        #[arg(long)]
        run: Option<String>,
        #[arg(long)]
        name: String,
        #[arg(long)]
        version: Option<String>,
        #[arg(long)]
        nu_version: Option<String>,
    },
}

fn emit<T: serde::Serialize>(value: &T, json: bool) -> anyhow::Result<()> {
    if json {
        println!("{}", serde_json::to_string(value)?);
    } else {
        println!("{}", serde_json::to_string_pretty(value)?);
    }
    Ok(())
}

fn read_json(path: &PathBuf) -> anyhow::Result<serde_json::Value> {
    let text =
        std::fs::read_to_string(path).with_context(|| format!("reading {}", path.display()))?;
    serde_json::from_str(&text).with_context(|| format!("parsing {}", path.display()))
}

fn parse_json_arg(s: &Option<String>) -> anyhow::Result<Option<serde_json::Value>> {
    match s {
        Some(text) => Ok(Some(
            serde_json::from_str(text).context("parsing inline JSON argument")?,
        )),
        None => Ok(None),
    }
}

/// Resolve a run by row id; a target by row id or natural target_id.
fn resolve_target_id(db: &MigrationDb, key: &str) -> anyhow::Result<String> {
    if let Ok(t) = db.target_by_natural_id(key) {
        return Ok(t.id);
    }
    db.targets()?
        .into_iter()
        .find(|t| t.id == key)
        .map(|t| t.id)
        .ok_or_else(|| anyhow!("target not found: {key}"))
}

pub fn run_migration(
    db_path: Option<PathBuf>,
    cmd: MigrationCmd,
    json: bool,
) -> anyhow::Result<()> {
    let path = db_path.unwrap_or_else(MigrationDb::default_path);
    let db = MigrationDb::open(&path)?;
    match cmd {
        MigrationCmd::Package(c) => match c {
            PackageCmd::Inspect { path } => {
                // Inspect = import semantics without recording: hash into a throwaway store.
                let tmp = std::env::temp_dir().join(format!(
                    "envctl-migration-inspect-{}.redb",
                    std::process::id()
                ));
                let scratch = MigrationDb::open(&tmp)?;
                let pkg = scratch.import_package("inspect", &path)?;
                emit(
                    &serde_json::json!({
                        "package_path": pkg.package_path,
                        "package_hash": pkg.package_hash,
                        "manifest": pkg.manifest_json,
                    }),
                    json,
                )
            }
            PackageCmd::Import { name, path } => emit(&db.import_package(&name, &path)?, json),
            PackageCmd::List => emit(&db.packages()?, json),
        },
        MigrationCmd::Contract(c) => match c {
            ContractCmd::Import {
                name,
                version,
                file,
                package,
            } => {
                let contract = read_json(&file)?;
                emit(
                    &db.import_contract(&name, &version, contract, package.as_deref())?,
                    json,
                )
            }
            ContractCmd::List => emit(&db.contracts()?, json),
            ContractCmd::Show { id } => {
                let all = db.contracts()?;
                let found = all
                    .into_iter()
                    .find(|c| {
                        c.id == id || format!("{}@{}", c.contract_name, c.contract_version) == id
                    })
                    .ok_or_else(|| anyhow!("contract not found: {id}"))?;
                emit(&found, json)
            }
        },
        MigrationCmd::Recipe(c) => match c {
            RecipeCmd::Create {
                name,
                version,
                contract,
                file,
            } => {
                let recipe = read_json(&file)?;
                emit(&db.create_recipe(&name, &version, &contract, recipe)?, json)
            }
            RecipeCmd::List => emit(&db.recipes()?, json),
            RecipeCmd::Show { id } => {
                let found = db
                    .recipes()?
                    .into_iter()
                    .find(|r| r.id == id || format!("{}@{}", r.recipe_name, r.recipe_version) == id)
                    .ok_or_else(|| anyhow!("recipe not found: {id}"))?;
                emit(&found, json)
            }
        },
        MigrationCmd::Target(c) => match c {
            TargetCmd::Add {
                target_id,
                target_type,
                primary_root,
                compare_root,
                descriptor,
                safety_mode,
                max_auto_risk,
            } => {
                let spec = TargetSpec {
                    target_id,
                    target_type: TargetType::parse(&target_type)?,
                    primary_root,
                    compare_root,
                    descriptor: read_json(&descriptor)?,
                    safety_mode,
                    max_auto_risk: Risk::parse(&max_auto_risk)?,
                };
                emit(&db.register_target(spec)?, json)
            }
            TargetCmd::List => emit(&db.targets()?, json),
            TargetCmd::Show { target_id } => {
                let id = resolve_target_id(&db, &target_id)?;
                let found = db
                    .targets()?
                    .into_iter()
                    .find(|t| t.id == id)
                    .ok_or_else(|| anyhow!("target not found: {target_id}"))?;
                emit(&found, json)
            }
            TargetCmd::Validate { descriptor } => {
                let value = read_json(&descriptor)?;
                if !value.is_object() {
                    return Err(anyhow!("descriptor must be a JSON object"));
                }
                emit(
                    &serde_json::json!({
                        "valid": true,
                        "descriptor_hash": migration_db::sha256_hex(
                            migration_db::canonical_json(&value).as_bytes()
                        ),
                    }),
                    json,
                )
            }
        },
        MigrationCmd::Run(c) => match c {
            RunCmd::Create {
                target,
                recipe,
                human_mode,
                initiated_by,
                tool_versions,
                actor,
            } => {
                let (a, aid) = actor.resolve()?;
                let target_id = resolve_target_id(&db, &target)?;
                let spec = RunSpec {
                    target_id,
                    recipe_id: recipe,
                    human_mode: HumanMode::parse(&human_mode)?,
                    initiated_by,
                    sandbox_policy: None,
                    approval_policy: None,
                    tool_versions: parse_json_arg(&tool_versions)?,
                };
                emit(&db.create_run(spec, a, &aid)?, json)
            }
            RunCmd::Start { run, actor } => {
                let (a, aid) = actor.resolve()?;
                db.run_set_status(&run, RunStatus::Planning, a, &aid, None)?;
                emit(
                    &db.run_set_status(&run, RunStatus::Running, a, &aid, None)?,
                    json,
                )
            }
            RunCmd::Validate { run, actor } => {
                let (a, aid) = actor.resolve()?;
                emit(
                    &db.run_set_status(&run, RunStatus::Validating, a, &aid, None)?,
                    json,
                )
            }
            RunCmd::Complete { run, actor } => {
                let (a, aid) = actor.resolve()?;
                emit(&db.complete_run(&run, a, &aid)?, json)
            }
            RunCmd::Status { run, all } => match (all, run) {
                (false, Some(run)) => emit(&db.view_run_status(&run)?, json),
                _ => emit(&db.view_all_run_status()?, json),
            },
            RunCmd::Events { run } => emit(&db.view_timeline(&run)?, json),
            RunCmd::Ops { run, queue } => {
                if queue {
                    emit(&db.view_operation_queue(&run)?, json)
                } else {
                    emit(&db.operations(&run)?, json)
                }
            }
            RunCmd::Artifacts { run } => emit(&db.artifacts(&run)?, json),
            RunCmd::Validations { run } => emit(
                &serde_json::json!({
                    "scorecard": db.view_scorecard(&run)?,
                    "validations": db.validations(&run)?,
                }),
                json,
            ),
            RunCmd::Replay {
                run,
                mode,
                verify_files,
            } => {
                let report = db.replay(&run, ReplayMode::parse(&mode)?, verify_files)?;
                let ok = report.ok;
                emit(&report, json)?;
                if !ok {
                    return Err(anyhow!("replay verification failed"));
                }
                Ok(())
            }
            RunCmd::Readiness { run } => emit(&db.view_replay_readiness(&run)?, json),
            RunCmd::Export { run } => emit(&db.export_run(&run)?, json),
            RunCmd::List => emit(&db.runs()?, json),
        },
        MigrationCmd::Op(c) => match c {
            OpCmd::Add {
                run,
                operation_type,
                phase,
                risk,
                step,
                command,
                input,
                actor,
            } => {
                let (a, aid) = actor.resolve()?;
                let spec = OperationSpec {
                    run_id: run,
                    operation_type,
                    phase,
                    risk: Risk::parse(&risk)?,
                    idempotency_key: None,
                    recipe_step_id: step,
                    command_redacted: command,
                    input: parse_json_arg(&input)?,
                    parent_operation_id: None,
                };
                emit(&db.add_operation(spec, a, &aid)?, json)
            }
            OpCmd::Start { op, actor } => {
                let (a, aid) = actor.resolve()?;
                let (op, approval) = db.op_request_start(&op, a, &aid)?;
                emit(
                    &serde_json::json!({"operation": op, "approval": approval}),
                    json,
                )
            }
            OpCmd::Complete {
                op,
                failed,
                detail,
                actor,
            } => {
                let (a, aid) = actor.resolve()?;
                let to = if failed {
                    OpStatus::Failed
                } else {
                    OpStatus::Succeeded
                };
                emit(
                    &db.op_set_status(&op, to, a, &aid, parse_json_arg(&detail)?)?,
                    json,
                )
            }
            OpCmd::Show { op } => emit(&db.operation(&op)?, json),
        },
        MigrationCmd::Approval(c) => match c {
            ApprovalCmd::List { run } => emit(&db.view_open_approvals(run.as_deref())?, json),
            ApprovalCmd::Approve {
                approval,
                by,
                reason,
                evidence,
                actor,
            } => {
                let (a, _) = actor.resolve()?;
                emit(
                    &db.approval_decide(
                        &approval,
                        ApprovalDecision::Approve,
                        a,
                        &by,
                        &reason,
                        parse_json_arg(&evidence)?,
                    )?,
                    json,
                )
            }
            ApprovalCmd::Deny {
                approval,
                by,
                reason,
                evidence,
                actor,
            } => {
                let (a, _) = actor.resolve()?;
                emit(
                    &db.approval_decide(
                        &approval,
                        ApprovalDecision::Deny,
                        a,
                        &by,
                        &reason,
                        parse_json_arg(&evidence)?,
                    )?,
                    json,
                )
            }
        },
        MigrationCmd::Control(c) => {
            let (run, to, actor) = match c {
                ControlCmd::Pause { run, actor } => (run, RunStatus::Paused, actor),
                ControlCmd::Resume { run, actor } => (run, RunStatus::Running, actor),
                ControlCmd::Cancel { run, actor } => (run, RunStatus::Cancelled, actor),
            };
            let (a, aid) = actor.resolve()?;
            emit(&db.run_set_status(&run, to, a, &aid, None)?, json)
        }
        MigrationCmd::Rollback(c) => match c {
            RollbackCmd::Plan {
                run,
                rollback_type,
                plan,
                op,
                actor,
            } => {
                let (a, aid) = actor.resolve()?;
                let plan_json: serde_json::Value =
                    serde_json::from_str(&plan).context("parsing --plan JSON")?;
                emit(
                    &db.plan_rollback(&run, op.as_deref(), &rollback_type, plan_json, a, &aid)?,
                    json,
                )
            }
            RollbackCmd::List { run } => emit(&db.rollbacks(&run)?, json),
        },
        MigrationCmd::Evidence {
            run,
            uri,
            kind,
            sha256,
            hash_file,
            op,
            metadata,
            actor,
        } => {
            let (a, aid) = actor.resolve()?;
            let hash = match (sha256, hash_file) {
                (Some(h), _) => Some(h),
                (None, true) => {
                    let bytes = std::fs::read(&uri).with_context(|| format!("reading {uri}"))?;
                    Some(migration_db::sha256_hex(&bytes))
                }
                (None, false) => None,
            };
            emit(
                &db.add_evidence(
                    &run,
                    op.as_deref(),
                    &uri,
                    &kind,
                    hash.as_deref(),
                    false,
                    parse_json_arg(&metadata)?,
                    a,
                    &aid,
                )?,
                json,
            )
        }
        MigrationCmd::Artifact {
            run,
            artifact_id,
            title,
            artifact_type,
            status,
            path,
            content_hash,
            hash_file,
            op,
            actor,
        } => {
            let (a, aid) = actor.resolve()?;
            let hash = match (&content_hash, hash_file, &path) {
                (Some(h), _, _) => Some(h.clone()),
                (None, true, Some(p)) => {
                    let bytes = std::fs::read(p).with_context(|| format!("reading {p}"))?;
                    Some(migration_db::sha256_hex(&bytes))
                }
                _ => None,
            };
            emit(
                &db.upsert_artifact(
                    &run,
                    &artifact_id,
                    &title,
                    artifact_type.as_deref(),
                    ArtifactStatus::parse(&status)?,
                    path.as_deref(),
                    hash.as_deref(),
                    op.as_deref(),
                    None,
                    None,
                    a,
                    &aid,
                )?,
                json,
            )
        }
        MigrationCmd::Validation {
            run,
            validator,
            status,
            artifact,
            op,
            details,
            actor,
        } => {
            let (a, aid) = actor.resolve()?;
            let spec = ValidationSpec {
                run_id: run,
                validator,
                status: ValidationStatus::parse(&status)?,
                artifact_id: artifact,
                operation_id: op,
                details: parse_json_arg(&details)?,
                evidence: None,
            };
            emit(&db.add_validation(spec, a, &aid)?, json)
        }
        MigrationCmd::Edge {
            run,
            from,
            to,
            edge_type,
            artifact,
        } => emit(
            &db.add_graph_edge(
                &run,
                &from,
                &to,
                &edge_type,
                artifact.as_deref(),
                None,
                None,
            )?,
            json,
        ),
        MigrationCmd::Checkpoint {
            run,
            kind,
            reference,
            hash,
            actor,
        } => {
            let (a, aid) = actor.resolve()?;
            emit(
                &db.add_checkpoint(
                    &run,
                    None,
                    &kind,
                    &reference,
                    hash.as_deref(),
                    None,
                    a,
                    &aid,
                )?,
                json,
            )
        }
        MigrationCmd::Session(c) => match c {
            SessionCmd::Agent {
                run,
                name,
                model,
                authority,
                session,
            } => emit(
                &db.record_agent_session(
                    run.as_deref(),
                    &name,
                    model.as_deref(),
                    authority.as_deref(),
                    parse_json_arg(&session)?,
                )?,
                json,
            ),
            SessionCmd::Plugin {
                run,
                name,
                version,
                nu_version,
            } => emit(
                &db.record_plugin_session(
                    run.as_deref(),
                    &name,
                    version.as_deref(),
                    nu_version.as_deref(),
                    None,
                    None,
                )?,
                json,
            ),
        },
        MigrationCmd::Event {
            run,
            event_type,
            phase,
            op,
            payload,
            actor,
        } => {
            let (a, aid) = actor.resolve()?;
            let payload_json: serde_json::Value =
                serde_json::from_str(&payload).context("parsing --payload JSON")?;
            emit(
                &db.append_event(
                    &run,
                    &event_type,
                    phase.as_deref(),
                    a,
                    Some(&aid),
                    op.as_deref(),
                    payload_json,
                    None,
                )?,
                json,
            )
        }
    }
}
