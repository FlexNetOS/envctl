//! CLI entry point for `envctl-commit-worker` (ARCHBP-042 / blueprint
//! invariant 7): envctl is the exclusive production PostgreSQL/RuVector
//! committer. This binary is a thin wrapper over
//! `envctl_commit_worker::drain_and_commit` — all durability, ordering, and
//! witness-chain logic lives in the library; this file only parses
//! arguments and prints the result as JSON.

use clap::{Parser, Subcommand};
use envctl_commit_worker::{activation, drain_and_commit};
use std::path::PathBuf;

#[derive(Parser)]
#[command(
    name = "envctl-commit-worker",
    version,
    about = "envctl's exclusive PostgreSQL/RuVector ingress committer (ARCHBP-042)"
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Drain `codedb_outbox_export` beyond the reconciliation cursor and
    /// commit the batch durably, one transaction at a time.
    Drain {
        /// PostgreSQL connection string (libpq keyword/value format), e.g.
        /// "host=/path/to/socket port=5432 dbname=envctl_commit_test".
        /// Must name an explicit Unix-socket host — TCP is refused
        /// fail-closed (the commit worker requires the envctl TLS policy
        /// for any TCP path, which this CLI does not implement).
        #[arg(long)]
        conn: String,

        /// Maximum staging rows committed per transaction/batch.
        #[arg(long, default_value_t = 500)]
        max_batch: usize,

        /// Exercise the full read/witness path but abort every batch
        /// before COMMIT: nothing durable, nothing acknowledged. Maps
        /// directly onto the library's `fail_before_commit` failpoint, so
        /// this is a real dry run of the transaction body, not a simulated
        /// one. Because the transaction always aborts, a batch that finds
        /// staging rows to drain always surfaces as an error here — that
        /// is the documented, correct behavior of the failpoint, not a
        /// bug in this wrapper.
        #[arg(long)]
        dry_run: bool,
    },

    /// Materialize the release activations `lifeos_release.promote` approved
    /// (blueprint §17 step 15): swap the activation symlink atomically and
    /// acknowledge only after the swap succeeds. Previews by default.
    Activate {
        /// PostgreSQL connection string; Unix-socket host required, as above.
        #[arg(long)]
        conn: String,

        /// The activation symlink this release materializes onto.
        #[arg(long)]
        link: PathBuf,

        /// The approved generation the link should point at.
        #[arg(long)]
        target: PathBuf,

        /// Perform the swap and acknowledge. Without this the command reports
        /// what it would do and touches neither the filesystem nor the outbox.
        #[arg(long)]
        apply: bool,
    },
}

fn main() {
    let cli = Cli::parse();
    let rendered = match cli.command {
        Command::Drain {
            conn,
            max_batch,
            dry_run,
        } => drain_and_commit(&conn, max_batch, dry_run)
            .map(|receipt| serde_json::to_string_pretty(&receipt).expect("receipt serializes")),
        Command::Activate {
            conn,
            link,
            target,
            apply,
        } => activation::materialize(&conn, &link, &target, apply)
            .map(|outcomes| serde_json::to_string_pretty(&outcomes).expect("outcomes serialize")),
    };

    match rendered {
        Ok(json) => println!("{json}"),
        Err(err) => {
            let payload = serde_json::json!({ "error": err.to_string() });
            eprintln!(
                "{}",
                serde_json::to_string_pretty(&payload).expect("error payload always serializes")
            );
            std::process::exit(1);
        }
    }
}
