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

/// after_help Examples block for the migration verbs (local sibling of main.rs's
/// crate-root `envctl_examples!`, which is textually out of scope for this module;
/// emits the same `Examples:` contract the help-gap test enforces).
macro_rules! mig_examples {
    ($($line:literal),+ $(,)?) => {
        concat!("Examples:\n", $("  ", $line, "\n",)+)
    };
}

#[derive(Subcommand)]
pub enum MigrationCmd {
    /// Inspect or import external prompt packages (versioned, content-hashed).
    #[command(
        subcommand,
        long_about = "External prompt packages as versioned, content-hashed imports: every file is sha256'd in a sorted walk, so a package's identity is stable and replay can verify it. Read-only on the package directory.",
        after_help = mig_examples!(
            "envctl migration package inspect ./my-package",
            "envctl migration package import my-package ./my-package",
        )
    )]
    Package(PackageCmd),
    /// Versioned artifact contracts (changes create new versions, never edits).
    #[command(
        subcommand,
        long_about = "The artifact contract registry. Contracts are versioned and content-hashed; a run references exactly one contract version, and contract changes create NEW versions — silent edits are refused by the UNIQUE(name, version) constraint.",
        after_help = mig_examples!(
            "envctl migration contract import full-migration-artifact-contract 1.0.0 --file contract.json",
            "envctl migration contract show full-migration-artifact-contract@1.0.0",
        )
    )]
    Contract(ContractCmd),
    /// Migration recipes bound to a contract version.
    #[command(
        subcommand,
        long_about = "Migration recipes: versioned, content-hashed step lists (a steps array with step_id/operation_type/risk) bound to an artifact contract version. The recipe hash is part of every run's replay identity.",
        after_help = mig_examples!(
            "envctl migration recipe create four-system-unify 1.0.0 --contract contract-000001 --file recipe.json",
            "envctl migration recipe list",
        )
    )]
    Recipe(RecipeCmd),
    /// Target descriptors (the systems a run imports/exports).
    #[command(
        subcommand,
        long_about = "Target descriptors: the systems a migration run reads from and writes to. Descriptors are JSON objects, validated and content-hashed at registration; target_id is unique; max_auto_risk caps what may start without an approval.",
        after_help = mig_examples!(
            "envctl migration target add four-system --primary-root /work --descriptor target.json",
            "envctl migration target validate target.json",
        )
    )]
    Target(TargetCmd),
    /// Runs: create/start/status/events/ops/artifacts/validations/replay/export.
    #[command(
        subcommand,
        long_about = "Migration runs: the event-sourced execution unit. Every status transition walks the fail-closed state machine (created -> planning -> running -> validating -> completed) and appends to the run's hash-chained ledger; completion stamps a reproducibility hash; replay verifies every recorded hash.",
        after_help = mig_examples!(
            "envctl migration run create --target four-system --recipe recipe-000001",
            "envctl migration run replay run-000001 --mode verify-only --verify-files",
        )
    )]
    Run(RunCmd),
    /// Operations: queue, request-start (approval-gated at R3+), complete.
    #[command(
        subcommand,
        long_about = "Operations inside a run: queued with an idempotency key (duplicates return the recorded row), started through the approval gate (R3+ or anything above the target's max_auto_risk parks in awaiting_approval), completed with terminal status — every transition an event.",
        after_help = mig_examples!(
            "envctl migration op add run-000001 --operation-type capture --risk R1",
            "envctl migration op start op-000001",
        )
    )]
    Op(OpCmd),
    /// Approval queue: list open, approve, deny — decisions append events.
    #[command(
        subcommand,
        long_about = "The approval gate's queue. Agents and humans use the SAME surface (authority, not state, is the difference): decisions record decider, rationale, and evidence refs as ledger events, then move the gated operation to ready or denied.",
        after_help = mig_examples!(
            "envctl migration approval list",
            "envctl migration approval approve approval-000001 --by agent-reviewer --reason \"evidence checked\"",
        )
    )]
    Approval(ApprovalCmd),
    /// Run control: pause / resume / cancel.
    #[command(
        subcommand,
        long_about = "Run control verbs: pause a running run, resume a paused one, cancel a run that must stop. Each is a state-machine transition appended to the ledger — no hidden side effects.",
        after_help = mig_examples!(
            "envctl migration control pause run-000001",
            "envctl migration control resume run-000001",
        )
    )]
    Control(ControlCmd),
    /// Rollback metadata: plan and record execution.
    #[command(
        subcommand,
        long_about = "Rollback metadata for operations that need it. Plans are recorded as JSON (the never-delete doctrine: exports write NEW trees, so rollback is point-back-at-originals); execution is the pipeline's job and lands here as status + events.",
        after_help = mig_examples!(
            "envctl migration rollback plan run-000001 --plan '{\"originals\":\"untouched\"}'",
            "envctl migration rollback list run-000001",
        )
    )]
    Rollback(RollbackCmd),
    /// Record evidence for a run (uri + kind + sha256).
    #[command(
        long_about = "Record an evidence row for a run: uri + kind + sha256 (pass --hash-file to hash the uri path now). Recording appends an evidence.recorded event; replay --verify-files re-hashes on-disk evidence.",
        after_help = mig_examples!(
            "envctl migration evidence run-000001 --uri ./baseline.sha256 --kind parity_baseline --hash-file",
        )
    )]
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
    #[command(
        long_about = "Record or refresh an artifact row (upsert on artifact id within the run): title, type, status, path, content hash (--hash-file hashes the path now), evidence and link JSON. Updates refresh — history lives in the event ledger.",
        after_help = mig_examples!(
            "envctl migration artifact run-000001 --artifact-id unified-idd-tree --title \"Unified .idd export\" --status complete --path ./out --hash-file",
        )
    )]
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
    #[command(
        long_about = "Record a validation result (pass/fail/warn/blocked/unknown) from a named validator, optionally linked to an artifact and operation. Rows feed the scorecard view; recording appends a validation.recorded event.",
        after_help = mig_examples!(
            "envctl migration validation run-000001 --validator byte-parity --status pass --details '{\"mismatched\":0}'",
        )
    )]
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
    #[command(
        long_about = "Record a dependency or data-flow graph edge (from_node -> to_node, typed) for a run — the substrate for wikilink graphs, cross-system task twins, and blast-radius queries.",
        after_help = mig_examples!(
            "envctl migration edge run-000001 --from kb:tasks/alpha --to handoff:TASK-0001 --edge-type same_logical_task",
        )
    )]
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
    #[command(
        long_about = "Record a checkpoint: a named snapshot/rollback anchor (kind + ref + optional hash) for a run or operation. Recording appends a checkpoint.recorded event.",
        after_help = mig_examples!(
            "envctl migration checkpoint run-000001 --kind baseline --ref baselines/kb.sha256",
        )
    )]
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
    #[command(
        subcommand,
        long_about = "Record agent and plugin sessions: who (or what) drove a run, with model label, authority level, and session JSON — the _agent_sessions/_plugin_sessions provenance the agent-review protocol requires.",
        after_help = mig_examples!(
            "envctl migration session agent --run run-000001 --name agent-reviewer --authority operator",
        )
    )]
    Session(SessionCmd),
    /// Append a custom event to a run's hash-chained ledger.
    #[command(
        long_about = "Append a custom event to a run's hash-chained, append-only ledger (event_hash covers the previous hash + the whole envelope, so history cannot be silently rewritten). All actions append events; no hidden side effects.",
        after_help = mig_examples!(
            "envctl migration event run-000001 --event-type phase.completed --payload '{\"phase\":\"import\"}'",
        )
    )]
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
    #[command(
        long_about = "Content-hash a package directory (sorted sha256 walk) and print its identity WITHOUT recording anything — the read-only preview of `package import`.",
        after_help = mig_examples!("envctl migration package inspect ./my-package")
    )]
    Inspect { path: PathBuf },
    /// Import (record) a package: name + path + content hash + manifest.
    #[command(
        long_about = "Record a package: name, path, stable content hash over every file, and a small manifest (file count + bytes). UNIQUE(name, hash) — reimporting identical content conflicts instead of duplicating.",
        after_help = mig_examples!("envctl migration package import my-package ./my-package")
    )]
    Import { name: String, path: PathBuf },
    /// All recorded packages.
    #[command(
        long_about = "List every recorded package with its content hash and import time.",
        after_help = mig_examples!("envctl --json migration package list")
    )]
    List,
}

#[derive(Subcommand)]
pub enum ContractCmd {
    /// Import a contract version from a JSON file.
    #[command(
        long_about = "Import an artifact contract version from a JSON file (optionally linked to a source package). Content-hashed; UNIQUE(name, version) — changes create new versions, never edits.",
        after_help = mig_examples!(
            "envctl migration contract import full-migration-artifact-contract 1.0.0 --file contract.json"
        )
    )]
    Import {
        name: String,
        version: String,
        #[arg(long)]
        file: PathBuf,
        #[arg(long)]
        package: Option<String>,
    },
    /// All contract versions.
    #[command(
        long_about = "List every contract version with its content hash.",
        after_help = mig_examples!("envctl --json migration contract list")
    )]
    List,
    /// One contract by row id or name@version.
    #[command(
        long_about = "Show one contract by row id or name@version, including its full contract JSON and hash.",
        after_help = mig_examples!("envctl migration contract show full-migration-artifact-contract@1.0.0")
    )]
    Show { id: String },
}

#[derive(Subcommand)]
pub enum RecipeCmd {
    /// Create a recipe version from a JSON file (must contain a steps array).
    #[command(
        long_about = "Create a recipe version from a JSON file bound to an artifact contract id. The JSON must contain a steps array (step_id/operation_type/risk per step); the recipe hash joins the run's replay identity.",
        after_help = mig_examples!(
            "envctl migration recipe create four-system-unify 1.0.0 --contract contract-000001 --file recipe.json"
        )
    )]
    Create {
        name: String,
        version: String,
        #[arg(long)]
        contract: String,
        #[arg(long)]
        file: PathBuf,
    },
    /// All recipe versions.
    #[command(
        long_about = "List every recipe version with its contract binding and hash.",
        after_help = mig_examples!("envctl --json migration recipe list")
    )]
    List,
    /// One recipe by row id or name@version.
    #[command(
        long_about = "Show one recipe by row id or name@version, including its steps JSON and hash.",
        after_help = mig_examples!("envctl migration recipe show four-system-unify@1.0.0")
    )]
    Show { id: String },
}

#[derive(Subcommand)]
pub enum TargetCmd {
    /// Register a target descriptor (JSON file), validated + content-hashed.
    #[command(
        long_about = "Register a target: unique target_id, type (codebase|data|infrastructure|integration|mixed), primary root, optional compare root, a JSON descriptor (validated + content-hashed), safety mode, and max_auto_risk — the cap above which operations require approval.",
        after_help = mig_examples!(
            "envctl migration target add four-system --primary-root /work --descriptor target.json --max-auto-risk R2"
        )
    )]
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
    /// All registered targets.
    #[command(
        long_about = "List every registered target with its descriptor hash and risk cap.",
        after_help = mig_examples!("envctl --json migration target list")
    )]
    List,
    /// One target by natural target_id or row id.
    #[command(
        long_about = "Show one target by natural target_id or row id, including its full descriptor JSON.",
        after_help = mig_examples!("envctl migration target show four-system")
    )]
    Show { target_id: String },
    /// Parse + validate a descriptor file without registering it.
    #[command(
        long_about = "Parse and validate a descriptor JSON file and print the descriptor hash it WOULD get — nothing is recorded. The read-only preview of `target add`.",
        after_help = mig_examples!("envctl migration target validate target.json")
    )]
    Validate { descriptor: PathBuf },
}

#[derive(Subcommand)]
pub enum RunCmd {
    /// Create a run (target row-id or natural id + recipe id).
    #[command(
        long_about = "Create a run in status `created` against a target (row id or natural target_id) and a recipe id; the run binds the recipe's contract version and appends run.created with the target/recipe hashes.",
        after_help = mig_examples!("envctl migration run create --target four-system --recipe recipe-000001")
    )]
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
    #[command(
        long_about = "Walk the happy-path start: created -> planning -> running, each transition an event; started_at is stamped on entering running.",
        after_help = mig_examples!("envctl migration run start run-000001")
    )]
    Start {
        run: String,
        #[command(flatten)]
        actor: ActorArgs,
    },
    /// running -> validating.
    #[command(
        long_about = "Transition a running run into validating — the phase where parity/behavior validators record their results before completion.",
        after_help = mig_examples!("envctl migration run validate run-000001")
    )]
    Validate {
        run: String,
        #[command(flatten)]
        actor: ActorArgs,
    },
    /// validating -> completed; stamps the reproducibility hash.
    #[command(
        long_about = "Complete a validating run. The completion event lands first, then the reproducibility hash is stamped over target + recipe + contract + tool versions + the final event hash — the value replay must land on.",
        after_help = mig_examples!("envctl migration run complete run-000001")
    )]
    Complete {
        run: String,
        #[command(flatten)]
        actor: ActorArgs,
    },
    /// Latest-status view (one run, or --all).
    #[command(
        long_about = "The run_latest_status view: status, counts (operations, failures, open approvals, artifacts), and last event time — one run or --all.",
        after_help = mig_examples!("envctl --json migration run status run-000001")
    )]
    Status {
        run: Option<String>,
        #[arg(long, default_value_t = false)]
        all: bool,
    },
    /// The live timeline (hash-chained events joined to operations).
    #[command(
        long_about = "The live_timeline view: every ledger event in sequence, joined to its operation's type and status.",
        after_help = mig_examples!("envctl --json migration run events run-000001")
    )]
    Events { run: String },
    /// Operations for a run (--queue for the live queue only).
    #[command(
        long_about = "Operations for a run — all of them, or with --queue only the live ones (queued/ready/awaiting_approval/running/blocked).",
        after_help = mig_examples!("envctl migration run ops run-000001 --queue")
    )]
    Ops {
        run: String,
        #[arg(long, default_value_t = false)]
        queue: bool,
    },
    /// The artifact index for a run.
    #[command(
        long_about = "The artifact_index view: every artifact row for the run with status, path, and content hash.",
        after_help = mig_examples!("envctl --json migration run artifacts run-000001")
    )]
    Artifacts { run: String },
    /// Validation rows + scorecard.
    #[command(
        long_about = "Validation rows plus the validation_scorecard rollup (pass/fail/warn/blocked/unknown counts) for a run.",
        after_help = mig_examples!("envctl --json migration run validations run-000001")
    )]
    Validations { run: String },
    /// Replay: verify-only | dry-run-plan (execute-again refuses at the engine).
    #[command(
        long_about = "Replay a run: verify-only recomputes every recorded hash (target/recipe/contract, full event chain, command hashes, evidence/artifacts — with --verify-files re-hashing files on disk, approvals, reproducibility hash) and exits non-zero on any mismatch; dry-run-plan additionally prints the recipe steps; execute-again refuses at the engine (destructive replay needs an approved R3+ operation).",
        after_help = mig_examples!("envctl migration run replay run-000001 --mode verify-only --verify-files")
    )]
    Replay {
        run: String,
        #[arg(long, default_value = "verify-only")]
        mode: String,
        /// Re-hash evidence/artifact files on disk, not just recorded hashes.
        #[arg(long, default_value_t = false)]
        verify_files: bool,
    },
    /// Replay readiness view (hash coverage + open approvals).
    #[command(
        long_about = "The replay_readiness view: reproducibility hash presence, evidence/artifact rows missing hashes, and open approvals — what stands between this run and a verifiable replay.",
        after_help = mig_examples!("envctl --json migration run readiness run-000001")
    )]
    Readiness { run: String },
    /// Export the whole run as one bundle (the plugin read surface).
    #[command(
        long_about = "Export the whole run as one JSON bundle — run, target, recipe, contract, operations, events, evidence, artifacts, approvals, validations, graph edges, checkpoints, rollbacks. The nu_plugin/agent read surface.",
        after_help = mig_examples!("envctl --json migration run export run-000001 > run-bundle.json")
    )]
    Export { run: String },
    /// All runs.
    #[command(
        long_about = "List every run row (status, bindings, timestamps).",
        after_help = mig_examples!("envctl --json migration run list")
    )]
    List,
}

#[derive(Subcommand)]
pub enum OpCmd {
    /// Queue an operation (idempotent on the derived idempotency key).
    #[command(
        long_about = "Queue an operation on a run: type, phase, risk (R0-R5), optional recipe step, redacted command (hashed), and input JSON. The idempotency key derives from run + type + target hash + step + input hash — re-adding returns the recorded operation instead of duplicating.",
        after_help = mig_examples!(
            "envctl migration op add run-000001 --operation-type capture --risk R1 --step import-kb --input '{\"system\":\"kb\"}'"
        )
    )]
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
    #[command(
        long_about = "Request that an operation start. The approval gate applies here: R3+ risk (or anything above the target's max_auto_risk) parks the operation in awaiting_approval with an OPEN approval row; already-approved or safe operations transition to running. Every path appends events.",
        after_help = mig_examples!("envctl migration op start op-000001")
    )]
    Start {
        op: String,
        #[command(flatten)]
        actor: ActorArgs,
    },
    /// Terminal transition: succeeded (default) or --failed with detail.
    #[command(
        long_about = "Complete a running operation: succeeded by default, or --failed with a JSON detail recorded as the operation's error and in the transition event.",
        after_help = mig_examples!("envctl migration op complete op-000001")
    )]
    Complete {
        op: String,
        #[arg(long, default_value_t = false)]
        failed: bool,
        #[arg(long)]
        detail: Option<String>,
        #[command(flatten)]
        actor: ActorArgs,
    },
    /// One operation row.
    #[command(
        long_about = "Show one operation row: status, risk, idempotency key, command hash, timestamps.",
        after_help = mig_examples!("envctl --json migration op show op-000001")
    )]
    Show { op: String },
}

#[derive(Subcommand)]
pub enum ApprovalCmd {
    /// Open approvals (all runs, or --run).
    #[command(
        long_about = "The open_approvals view: every OPEN approval with its operation type, risk, requester, and request time — the queue an agent reviewer (or human) services.",
        after_help = mig_examples!("envctl --json migration approval list")
    )]
    List {
        #[arg(long)]
        run: Option<String>,
    },
    /// Approve: decision + rationale + evidence, appended as events.
    #[command(
        long_about = "Approve an open approval. The decider, rationale, and evidence refs are appended to the ledger as approval.decided, and the gated operation moves to ready. Agent reviewers run the SAME documented process a human would — deny by default when evidence is missing.",
        after_help = mig_examples!(
            "envctl migration approval approve approval-000001 --by agent-reviewer --reason \"parity baselines verified\" --evidence '[\"evidence-000001\"]'"
        )
    )]
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
    /// Deny: terminal for the gated operation, recorded as events.
    #[command(
        long_about = "Deny an open approval. The decision + rationale land in the ledger and the gated operation transitions to denied (terminal). Deny-by-default is the reviewer posture when evidence is missing.",
        after_help = mig_examples!(
            "envctl migration approval deny approval-000001 --by agent-reviewer --reason \"no evidence attached\""
        )
    )]
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
    /// running -> paused.
    #[command(
        long_about = "Pause a running run (running -> paused), appended as a status-change event.",
        after_help = mig_examples!("envctl migration control pause run-000001")
    )]
    Pause {
        run: String,
        #[command(flatten)]
        actor: ActorArgs,
    },
    /// paused -> running.
    #[command(
        long_about = "Resume a paused run (paused -> running), appended as a status-change event.",
        after_help = mig_examples!("envctl migration control resume run-000001")
    )]
    Resume {
        run: String,
        #[command(flatten)]
        actor: ActorArgs,
    },
    /// Terminal cancel from any live status.
    #[command(
        long_about = "Cancel a run (terminal; legal from created/planning/awaiting_approval/running/paused/blocked), appended as a status-change event with completed_at stamped.",
        after_help = mig_examples!("envctl migration control cancel run-000001")
    )]
    Cancel {
        run: String,
        #[command(flatten)]
        actor: ActorArgs,
    },
}

#[derive(Subcommand)]
pub enum RollbackCmd {
    /// Record a rollback plan (JSON) for a run/operation.
    #[command(
        long_about = "Record a rollback plan (JSON) for a run or operation, status `planned`, appended as rollback.planned. Under the never-delete doctrine the default type is point-back-at-originals: exports write NEW trees, so rollback never destroys anything.",
        after_help = mig_examples!(
            "envctl migration rollback plan run-000001 --plan '{\"originals\":\"untouched\"}'"
        )
    )]
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
    /// All rollback rows for a run.
    #[command(
        long_about = "List every rollback row for a run with status and plan JSON.",
        after_help = mig_examples!("envctl --json migration rollback list run-000001")
    )]
    List { run: String },
}

#[derive(Subcommand)]
pub enum SessionCmd {
    /// Record an agent session (name, model, authority level).
    #[command(
        long_about = "Record an agent session: agent name, model label, authority level (read_only|safe_execute|approval_request|operator|admin), and session JSON — the _agent_sessions provenance behind agent-serviced approvals.",
        after_help = mig_examples!(
            "envctl migration session agent --run run-000001 --name agent-reviewer --authority operator"
        )
    )]
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
    /// Record a nu_plugin session (name, version, nu version).
    #[command(
        long_about = "Record a nu_plugin session: plugin name, plugin version, and the Nushell version it registered against — the _plugin_sessions provenance for plugin-driven actions.",
        after_help = mig_examples!(
            "envctl migration session plugin --name nu_plugin_codedb --version 0.1.0 --nu-version 0.112.2"
        )
    )]
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
