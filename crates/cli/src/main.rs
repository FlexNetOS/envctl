//! `envctl` — thin CLI over the shared engine. Subcommands map 1:1 to the five
//! verbs. Destructive verbs (reset/auto-fix) are DRY-RUN by default; pass
//! `--apply` to act. `auto-detect` is read-only and prints a real EnvReport.
mod migration_cmd;
mod self_update;

use clap::{Args, CommandFactory, Parser, Subcommand, ValueEnum};
use clap_complete::Shell;
use std::collections::BTreeSet;
use std::io::IsTerminal;
use std::path::PathBuf;
use std::time::Duration;

/// Clap help styling (ported from kasetto `colors::clap_styles`): amber `#e8a94d`
/// bold `Usage:` / `Commands:` / `Options:` headers + literals, secondary-grey
/// `#a8a195` `<COMMAND>` / `<ARG>` placeholders. clap propagates these to every
/// subcommand. Uses `clap::builder::styling` (built into clap 4.x — no new dep).
fn clap_styles() -> clap::builder::Styles {
    use clap::builder::styling::{Effects, RgbColor, Style, Styles};
    let amber = Style::new().fg_color(Some(RgbColor(232, 169, 77).into())) | Effects::BOLD;
    let secondary = Style::new().fg_color(Some(RgbColor(168, 161, 149).into()));
    Styles::styled()
        .header(amber)
        .usage(amber)
        .literal(amber)
        .placeholder(secondary)
}

/// Clap `after_help` block (ported from kasetto's `cli_examples!`, renamed and
/// kept crate-private — deliberately NOT `#[macro_export]`): an amber-bold
/// `Examples:` header followed by secondary-grey indented example lines. Defined
/// before its first use (the `Cli` derive) so the macro is in scope.
macro_rules! envctl_examples {
    ($($line:literal),* $(,)?) => {
        concat!(
            "\x1b[1m\x1b[38;2;232;169;77mExamples:\x1b[0m\n",
            $(
                concat!("  \x1b[38;2;168;161;149m", $line, "\x1b[0m\n"),
            )*
        )
    };
}

use envctl_engine::catalog as catalog_engine;
use envctl_engine::secrets::run_secretctl;
use envctl_engine::{
    AddRepoMode, AddRepoSpec, AgentAddSpec, AgentCleanSpec, AgentDoctorSpec, AgentInitSpec,
    AgentListKind, AgentListSpec, AgentLockMode, AgentLockSpec, AgentRemoveSpec, AgentScope,
    AgentSectionSel, AgentSyncSpec, AiAgent, BuildStrategy, BuildSystem, CatalogAnalyzeReport,
    CatalogDiffReport, CatalogFacetCount, CatalogImportReport, CatalogLockReport,
    CatalogRenderReport, CatalogScanSpec, CatalogSnapshot, CatalogSyncReport, CatalogTableName,
    CatalogTableSummaryRow, DashboardSpec, DriftSummary, Engine, EnvReport, Event, EventSink,
    HubRegistryReport, HubRegistryStatus, MigrationAction, MigrationReport, MigrationRisk,
    MigrationScope, MigrationSpec, MigrationStatus, MigrationVerb, OpStatus, Phase, Refactor,
    RefactorGoal, Registry, RenameRule, ResetGates, RunPlan, SelfUninstallSpec, Severity,
};

#[derive(Parser)]
#[command(
    name = "envctl",
    version,
    color = clap::ColorChoice::Auto,
    styles = clap_styles(),
    about = "meta's agentic environment manager — installs every tool into meta's .local layout",
    long_about = "A declarative, GPU-aware, agentic environment manager for the whole meta workspace, written in Rust.\n\nenvctl is a first-class meta peer member: it brings every tool, dependency, provider, vendor, CLI, and config to a declared state and installs it INTO meta's standard $META_ROOT layout ($META_ROOT/usr, $META_ROOT/etc, $META_ROOT/var, $META_ROOT/opt, plus meta-home XDG roots) — no system-depth or user-global installs, so anything meta uses lives in meta and travels wherever meta is cloned. Existing .toolchains managers remain a legacy compatibility prefix while manifests migrate. It works from TOML components whose lifecycle hooks wrap proven scripts: detect, install, fix, reset, and wire-in toolchains, repos, and the agent environment. One shared engine drives both the CLI and the GUI, so they never diverge. Destructive verbs (reset / auto-fix / self uninstall) are PREVIEW by default and fail-closed — they refuse unless they can prove the operation is safe and you pass the explicit act flag (--apply / --build / --confirm). Deployment target today: a GPU-aware dual-RTX-5090 Ubuntu 26.04 workstation.",
    after_help = envctl_examples!(
        "envctl auto-detect",
        "envctl doctor",
        "envctl install --dry-run",
        "envctl migrate scan",
        "envctl catalog scan --json",
        "envctl catalog import --json",
        "envctl catalog diff",
        "envctl catalog render --out /tmp/envctl-catalog",
        "envctl catalog sync --render-out /tmp/envctl-catalog-sync",
        "envctl catalog lock --check",
        "envctl catalog lock --apply",
        "envctl agent sync --apply",
        "envctl graph --impact secretd",
    )
)]
struct Cli {
    #[command(subcommand)]
    cmd: Cmd,
    /// Emit machine-readable NDJSON / JSON instead of the pretty view.
    #[arg(long, global = true)]
    json: bool,
    /// Suppress non-error output (repeat for stricter silence). Failures/refusals still print.
    #[arg(short = 'q', long, global = true, action = clap::ArgAction::Count)]
    quiet: u8,
    /// Increase output detail (-v, -vv, -vvv) — unfilters log lines.
    #[arg(short = 'v', long, global = true, action = clap::ArgAction::Count)]
    verbose: u8,
    /// When to emit colors: auto, always, never.
    #[arg(long, global = true, value_name = "WHEN", default_value_t = ColorMode::Auto)]
    color: ColorMode,
    /// [deprecated] alias for `--color never`.
    #[arg(long, global = true, hide = true)]
    no_color: bool,
}

/// The `--color` mode (kasetto `ColorMode`). `auto` respects `NO_COLOR`/TTY; `always` forces
/// color (sets `CLICOLOR_FORCE=1`); `never` disables it.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, ValueEnum)]
#[clap(rename_all = "lowercase")]
enum ColorMode {
    #[default]
    Auto,
    Always,
    Never,
}

impl std::fmt::Display for ColorMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            ColorMode::Auto => "auto",
            ColorMode::Always => "always",
            ColorMode::Never => "never",
        })
    }
}

/// Resolved presentation knobs threaded through every renderer (the engine stays non-printing;
/// these are pure front-end). Built once in `main` from the global flags.
#[derive(Clone, Copy, Debug)]
struct OutputCtx {
    json: bool,
    quiet: u8,
    verbose: u8,
    /// `true` = strip ANSI (resolved from `--color`/`--no-color`/`NO_COLOR`/TTY).
    plain: bool,
}

impl OutputCtx {
    fn is_quiet(&self) -> bool {
        self.quiet > 0
    }
}

/// Process-global presentation knobs, set once in `main` from the global flags. The renderers
/// (`print_event`, the agent renderers) read this rather than threading an `OutputCtx` through
/// every signature — these are presentation-only and the engine stays the source of behavior.
static OUTPUT: std::sync::OnceLock<OutputCtx> = std::sync::OnceLock::new();

fn out() -> OutputCtx {
    *OUTPUT.get().unwrap_or(&OutputCtx {
        json: false,
        quiet: 0,
        verbose: 0,
        plain: false,
    })
}

/// Strip ANSI escape sequences from a rendered line when `out().plain` (used by `--color never`
/// / `--no-color` / non-TTY). Cheap: only allocates when an escape is present.
fn paint(line: String) -> String {
    if !out().plain || !line.contains('\u{1b}') {
        return line;
    }
    let mut s = String::with_capacity(line.len());
    let mut chars = line.chars();
    while let Some(c) = chars.next() {
        if c == '\u{1b}' {
            // Skip the CSI sequence up to and including the final letter.
            for n in chars.by_ref() {
                if n.is_ascii_alphabetic() {
                    break;
                }
            }
        } else {
            s.push(c);
        }
    }
    s
}

/// Apply color-flag side effects (CLICOLOR_FORCE for `always`, a one-line deprecation warning
/// for the legacy `--no-color`) and return the effective `plain` value. Ports kasetto
/// `resolve_plain` (the deprecated `--plain` flag is renamed `--no-color` here).
fn resolve_plain(no_color: bool, color: ColorMode) -> bool {
    if no_color {
        eprintln!("warning: --no-color is deprecated; use --color never instead");
    }
    match color {
        ColorMode::Always => {
            std::env::set_var("CLICOLOR_FORCE", "1");
            // --no-color paired with --color always: honor the explicit deprecated request.
            no_color
        }
        ColorMode::Never => true,
        ColorMode::Auto => {
            // auto: plain when NO_COLOR is set, when stdout is not a TTY, or when --no-color.
            no_color || std::env::var_os("NO_COLOR").is_some() || !std::io::stdout().is_terminal()
        }
    }
}

#[derive(Subcommand)]
// clap parses one subcommand per invocation, so the inter-variant size delta is
// irrelevant here; boxing would complicate the derive for no real gain. Keep value-typed.
#[allow(clippy::large_enum_variant)]
enum Cmd {
    /// Read-only inventory: host, GPU (works pre-driver), tools, components.
    // audit fix (minor): dropped unimplemented `--only` flag (engine.detect takes
    // no filter) so an unsupported flag errors instead of silently no-oping.
    #[command(
        long_about = "Inspect the box and report its current state without changing anything: host facts, GPU inventory (works before the driver is installed), detected toolchains, and the install/drift status of every managed component. This is the read-only foundation the other verbs build on. Use --json (global) for a machine-readable EnvReport.",
        after_help = envctl_examples!(
            "envctl auto-detect",
            "envctl auto-detect --json",
        )
    )]
    AutoDetect {},
    /// Dependency-graph intelligence: summary, impact/blast-radius, paths, DOT/JSON.
    #[command(
        long_about = "Query the component dependency graph: a summary of the roster, a component's install closure and reset blast-radius (--impact), the root->X dependency paths explaining why X is needed (--why), or the whole graph as Graphviz DOT (--dot) or JSON (--json). Read-only. Pass --live to annotate nodes with live detect/drift state (runs auto-detect first).",
        after_help = envctl_examples!(
            "envctl graph",
            "envctl graph --impact secretd",
            "envctl graph --why cuda",
            "envctl graph --dot | dot -Tsvg -o graph.svg",
            "envctl graph --live",
        )
    )]
    Graph {
        /// Focus on one component: install closure + reset --cascade blast-radius.
        #[arg(long)]
        impact: Option<String>,
        /// Why is X needed — print the root→X dependency paths.
        #[arg(long)]
        why: Option<String>,
        /// Emit Graphviz DOT (pipe to `dot -Tsvg -o graph.svg`).
        #[arg(long)]
        dot: bool,
        /// Annotate with live detect/drift state (runs auto-detect first).
        #[arg(long)]
        live: bool,
    },
    /// Read-only federation over the workspace hub registries.
    #[command(
        long_about = "Read every `<name>_hub/registry.json`, reconcile the entries against the envctl component manifest, and emit the federated master view. With --check it acts as a drift gate (exit 1 when any entry binds to a missing component). Use --json (global) for the structured report.",
        after_help = envctl_examples!(
            "envctl registry",
            "envctl registry --check",
            "envctl registry --json",
        )
    )]
    Registry {
        /// Exit 1 when any hub entry binds to a missing envctl component.
        #[arg(long)]
        check: bool,
    },
    /// Read-only ADR-0003 catalog tables over envctl control-plane files.
    #[command(
        long_about = "Read the current envctl repo files into normalized, queryable catalog rows without mutating anything. This is the ADR-0003 bridge from existing TOML/YAML/JSON/Rust/handoff sources to table-first control-plane behavior. Use the global --json flag for machine-readable snapshots or tables.",
        after_help = envctl_examples!(
            "envctl catalog scan --json",
            "envctl catalog table components",
            "envctl catalog table nix-components",
            "envctl catalog table env-vars --json",
        )
    )]
    Catalog {
        #[command(flatten)]
        roots: CatalogRootArgs,
        #[command(subcommand)]
        cmd: CatalogCmd,
    },
    /// Write/verify envctl.lock — a content hash of every component for reproducible
    /// installs + a CI gate. No flags = (re)write the lock; --check = verify (exit 1 on drift).
    #[command(
        long_about = "Pin the component roster into envctl.lock — a content hash of every component — so installs are reproducible and CI can gate on drift. With no flags it (re)writes the lock; with --check it verifies the lock matches the manifest and exits 1 on drift without writing.",
        after_help = envctl_examples!(
            "envctl lock",
            "envctl lock --check",
        )
    )]
    Lock {
        /// Verify the lock matches the manifest; exit nonzero on drift (CI gate).
        #[arg(long)]
        check: bool,
    },
    /// Read-only health diagnostics: writability, toolchains, sudo, UEFI, GPU.
    #[command(
        long_about = "Run local health diagnostics and report what is and isn't ready: directory writability, installed toolchains, sudo access, UEFI / boot state, and GPU readiness. Read-only — it never changes the box. Use --json (global) for a machine-readable report.",
        after_help = envctl_examples!(
            "envctl doctor",
            "envctl doctor --json",
        )
    )]
    Doctor,
    /// Install components (additive + idempotent; --dry-run to preview).
    #[command(
        long_about = "Install the named components (or the whole roster when none are named) to bring the box to its declared state. Additive and idempotent — re-running only does what is missing. Pass --dry-run to preview the plan without changing anything.",
        after_help = envctl_examples!(
            "envctl install",
            "envctl install rust cuda",
            "envctl install --dry-run",
        )
    )]
    Install {
        targets: Vec<String>,
        #[arg(long)]
        dry_run: bool,
    },
    /// Reset = remove + unwire. DRY-RUN by default; --apply to act.
    #[command(
        long_about = "DESTRUCTIVE: remove and unwire the named components (uninstall binaries, revert wiring). PREVIEW (dry-run) by default; pass --apply to act. Fail-closed guards refuse unless safety is proven: resetting the whole roster (no targets) requires --all --confirm; removing live reverse-dependents requires --cascade; deleting declared data dirs requires --purge --confirm (the UUID is re-verified first). Use --keep-config to revert wiring + remove binaries while keeping config-kind paths.",
        after_help = envctl_examples!(
            "envctl reset secretd",
            "envctl reset secretd --apply",
            "envctl reset secretd --apply --cascade",
            "envctl reset --all --confirm --apply",
            "envctl reset secretd --purge --confirm --apply",
        )
    )]
    Reset {
        targets: Vec<String>,
        #[arg(long)]
        apply: bool,
        /// Required (with --confirm) to reset the WHOLE roster (no targets).
        #[arg(long)]
        all: bool,
        /// Acknowledge a destructive whole-roster / cascade / purge reset.
        #[arg(long)]
        confirm: bool,
        /// Also remove live reverse-dependents instead of refusing.
        #[arg(long)]
        cascade: bool,
        /// Keep config-kind paths (revert wiring + remove binaries only).
        #[arg(long)]
        keep_config: bool,
        /// Permit deletion of declared data dirs (UUID re-verified first).
        #[arg(long)]
        purge: bool,
    },
    /// Auto-fix = repair broken components. DRY-RUN by default; --apply to act.
    #[command(
        long_about = "Repair components that detect as broken (re-run the fix hook to restore the declared state). PREVIEW (dry-run) by default; pass --apply to act. A system-scope fix (apt / nix / cdi / alternatives) additionally requires --confirm, so a privileged change is never silent.",
        after_help = envctl_examples!(
            "envctl auto-fix",
            "envctl auto-fix cuda",
            "envctl auto-fix cuda --apply",
            "envctl auto-fix cuda --apply --confirm",
        )
    )]
    AutoFix {
        targets: Vec<String>,
        #[arg(long)]
        apply: bool,
        /// Confirm a system-scope fix (apt/nix/cdi/alternatives).
        #[arg(long)]
        confirm: bool,
    },
    /// Register a repo into the workspace as a meta PEER or a managed component.
    /// Acquire+detect+PREVIEW by default; pass --build to actually apply / build + install.
    #[command(
        long_about = "Register a git repo into the meta workspace. --mode (default auto) chooses how: `peer` registers it the meta-native way — a grep-guarded, idempotent edit of the meta-root .meta.yaml + .gitignore and a sibling clone, so it's a first-class member reachable by `meta exec/git/worktree` (no managed drop-in); `component` adopts it as a build-from-source managed component (clone, detect build system, build, install, wire in, register a components.d drop-in). `auto` routes owned/FlexNetOS remotes to PEER and everything else to COMPONENT. PREVIEW by default; pass --build to apply (peer: edit + clone; component: run the upstream build / AI agent / install). Peer takes --provides/--tag; component takes --strategy (as-is, cherry-pick --bin, rename --rename old=new, refactor --patch-cmd or --ai-goal port-to-rust) and --connect. An --id is required.",
        after_help = envctl_examples!(
            "envctl add-repo https://github.com/FlexNetOS/beads_rust --id beads_rust          # auto -> PEER (preview)",
            "envctl add-repo https://github.com/FlexNetOS/beads_rust --id beads_rust --tag tools --build",
            "envctl add-repo https://github.com/sharkdp/pastel --id pastel --build           # auto -> COMPONENT",
            "envctl add-repo https://github.com/FlexNetOS/example --id example --mode component --strategy cherry-pick --bin foo",
            "envctl add-repo https://github.com/BurntSushi/ripgrep --id rg --strategy rename --rename rg=rgx --build",
            "envctl add-repo https://github.com/sharkdp/pastel --id pastel-rs --mode component --strategy refactor --ai-goal port-to-rust --build",
        )
    )]
    AddRepo {
        /// Git URL (or use --local for a working tree).
        git_url: String,
        #[arg(long)]
        id: String,
        #[arg(long)]
        local: Option<std::path::PathBuf>,
        #[arg(long, value_name = "REF")]
        git_ref: Option<String>,
        /// Force a detector: cargo|cmake|meson|autotools|make|node|python|nix_flake|go|zig.
        #[arg(long)]
        build_system: Option<String>,
        #[arg(long)]
        build_cmd: Option<String>,
        /// Artifact glob relative to the clone. Repeatable.
        #[arg(long = "artifact")]
        artifacts: Vec<String>,
        /// Registration mode: auto | peer | component. `auto` (default) routes
        /// owned/FlexNetOS remotes to a first-class `.meta.yaml` PEER and everything
        /// else to a build-from-source managed component. `peer` forces meta-native
        /// registration; `component` forces the legacy drop-in.
        #[arg(long, default_value = "auto")]
        mode: String,
        /// peer mode: `provides:` capabilities for the `.meta.yaml` entry. Repeatable.
        #[arg(long = "provides")]
        provides: Vec<String>,
        /// peer mode: `tags:` for the `.meta.yaml` entry. Repeatable.
        #[arg(long = "tag")]
        tags: Vec<String>,
        /// Strategy: as-is | cherry-pick | rename | refactor (component mode only).
        #[arg(long, default_value = "as-is")]
        strategy: String,
        /// cherry-pick: only install these binaries (by file-stem). Repeatable.
        #[arg(long = "bin")]
        bins: Vec<String>,
        /// rename: install old under new name (old=new). Repeatable.
        #[arg(long = "rename", value_parser = parse_rename)]
        renames: Vec<(String, String)>,
        /// refactor=patch: shell transform run in the clone before build.
        #[arg(long)]
        patch_cmd: Option<String>,
        /// refactor=ai goal: port-to-rust | cherry-pick-to-crate | rename-for-synergy | custom.
        #[arg(long)]
        ai_goal: Option<String>,
        /// refactor=ai: force agent — claude|codex|gemini|kimi (else auto-detect).
        #[arg(long)]
        ai_agent: Option<String>,
        /// refactor=ai: extra instruction appended to the goal prompt.
        #[arg(long)]
        ai_instruction: Option<String>,
        /// Treat as a daemon (reserved for systemd --user wiring).
        #[arg(long)]
        daemon: bool,
        #[arg(long)]
        verify_cmd: Option<String>,
        /// OPT-IN: actually run the upstream build / AI agent / install (else preview).
        #[arg(long)]
        build: bool,
        /// Back up + replace a real foreign file at an install target.
        #[arg(long)]
        force: bool,
        /// git clone --recurse-submodules (off by default).
        #[arg(long)]
        recurse_submodules: bool,
        /// Interactive: clone, then drop into an agent session in the clone (for
        /// cherry-pick / port-to-rust). Pair with --build to build afterward.
        #[arg(long)]
        connect: bool,
        #[arg(long)]
        dry_run: bool,
    },
    /// Render the meta mission-control zellij dashboard from `.meta.yaml`.
    /// Default = print the KDL to stdout (read-only). `--deploy` previews the
    /// write (dry-run); `--apply` performs it; `--json` emits the DashboardPlan.
    #[command(
        long_about = "Generate the meta mission-control zellij layout (KDL) from the workspace `.meta.yaml` — one pane per repo. By default it prints the KDL to stdout (read-only). With --deploy it targets the yazelix zellij layouts dir; --deploy alone previews the write (dry-run) and --apply performs it. --force backs up and clobbers a non-envctl file at the target. --json emits the DashboardPlan.",
        after_help = envctl_examples!(
            "envctl dashboard",
            "envctl dashboard --deploy",
            "envctl dashboard --deploy --apply",
            "envctl dashboard --json",
        )
    )]
    Dashboard {
        /// Explicit `.meta.yaml` path (else walk up from CWD / use $META_FILE).
        #[arg(long)]
        meta_file: Option<std::path::PathBuf>,
        /// Max panes per tab before spilling into numbered sub-tabs.
        #[arg(long, default_value_t = 6)]
        panes_per_tab: usize,
        /// Layout name (the `<name>.kdl` file stem + deploy target).
        #[arg(long, default_value = "mission-control")]
        name: String,
        /// Deploy the layout to the yazelix zellij layouts dir (dry-run unless --apply).
        #[arg(long)]
        deploy: bool,
        /// With --deploy: actually write the file (else preview).
        #[arg(long)]
        apply: bool,
        /// With --deploy --apply: back up + clobber a non-envctl file at the target.
        #[arg(long)]
        force: bool,
    },
    /// Resolve the meta workspace root (via the `.meta.yaml` marker, like git's
    /// `.git`) and emit environment exports so shells and configs locate meta
    /// WITHOUT hardcoding paths. Read-only. Default prints POSIX `export` lines
    /// for `eval "$(envctl env)"`; `--json` emits a map. This is the seam that
    /// lets every config reference `$META_ROOT` no matter where meta is installed.
    #[command(
        long_about = "Resolve the meta workspace root (via the `.meta.yaml` marker, like git's `.git`) and emit environment exports so shells and configs locate meta WITHOUT hardcoding paths. Read-only. By default it prints POSIX `export` lines for `eval \"$(envctl env)\"`; --json emits a map. --toolchains also emits the meta-hosted .local layout, legacy toolchain prefixes, and PATH. --materialize FILE renders `${META_ROOT}` tokens in FILE to the absolute root for configs a consumer reads literally.",
        after_help = envctl_examples!(
            "eval \"$(envctl env)\"",
            "envctl env --json",
            "eval \"$(envctl env --toolchains)\"",
            "envctl env --materialize .claude/settings.json",
        )
    )]
    Env {
        /// Explicit `.meta.yaml` path (else walk up from CWD / use $META_FILE).
        #[arg(long)]
        meta_file: Option<std::path::PathBuf>,
        /// ALSO emit the meta-hosted .local layout, legacy toolchain prefix
        /// exports, and PATH. Manager stores still point at `$META_ROOT/.toolchains`
        /// until manifests migrate, but envctl-owned exposure begins at `usr/bin`.
        #[arg(long)]
        toolchains: bool,
        /// Instead of emitting exports, read FILE and print it with `${META_ROOT}`
        /// / `$META_ROOT` resolved to the absolute meta root. Read-only (stdout).
        /// Materializes configs that can't self-expand env vars (e.g. Claude's
        /// `extraKnownMarketplaces[].source.path`, which it reads literally).
        #[arg(long, value_name = "FILE")]
        materialize: Option<std::path::PathBuf>,
    },
    /// Adopt existing installs/configs into envctl's canonical `$META_ROOT` FHS/XDG topology.
    #[command(
        long_about = "Migrate/adopt an existing meta machine into envctl's canonical `$META_ROOT` FHS/XDG layout (`usr`, `etc`, `var`, `opt`, and meta-home XDG roots). scan/plan/verify are read-only. apply previews unless --apply is passed. purge is strict upgrade-only: it refuses deletion unless a legacy path already has verified canonical parity/adoption evidence. Shared meta substrates (loop_lib / meta_plugin_protocol) and agent/Codex configs are protected, not removed or rebuilt blindly.",
        after_help = envctl_examples!(
            "envctl migrate scan",
            "envctl migrate plan --scope component-registry",
            "envctl migrate apply",
            "envctl migrate apply --apply",
            "envctl migrate verify --json",
            "envctl migrate purge --apply --confirm",
        )
    )]
    Migrate {
        #[command(subcommand)]
        cmd: MigrateCmd,
    },
    /// Migration automation database (event-sourced, replayable): targets, artifact
    /// contracts, recipes, runs, operations, hash-chained events, evidence, artifacts,
    /// approvals (R3+ gate), validations, checkpoints, rollbacks, replay verification.
    #[command(
        long_about = "The database-backed migration automation engine (the envctl-db-nu-plugin package contract, repo-native on pure-Rust redb): register target descriptors, import packages/artifact contracts, create recipes, run event-sourced migration runs with hash-chained ledgers, gate R3+ operations behind approvals, record evidence/artifacts/validations/checkpoints/rollbacks, and verify replay. Agent-first: non-interactive, structured output, real exit codes. Store: --db, else $ENVCTL_MIGRATION_DB, else $META_ROOT/var/envctl/migration.redb.",
        after_help = envctl_examples!(
            "envctl migration target add four-system --primary-root /work --descriptor target.json",
            "envctl migration run create --target four-system --recipe recipe-000001",
            "envctl migration approval list",
            "envctl migration run replay run-000001 --mode verify-only --verify-files",
        )
    )]
    Migration {
        /// Migration store path override.
        #[arg(long)]
        db: Option<PathBuf>,
        #[command(subcommand)]
        cmd: migration_cmd::MigrationCmd,
    },
    /// Manage agent assets (skills / MCP servers / commands) declaratively over the
    /// shared `Engine::agent_*` API. Mutating verbs (sync/add/remove/clean) are
    /// PREVIEW by default; pass `--apply` to write. `--json` (global) emits the typed
    /// return value. `list`/`lock --check` are read-only.
    #[command(
        long_about = "Manage agent assets (skills / MCP servers / commands) declaratively over the shared engine. Reconcile installed assets with the config (sync), add or remove sources, lock the config, list the inventory, prune orphans (clean), create a starter config (init), or run diagnostics (doctor). Mutating verbs (sync / add / remove / clean) are PREVIEW by default; pass --apply to write. `list` and `lock --check` are read-only.",
        after_help = envctl_examples!(
            "envctl agent sync --apply",
            "envctl agent add https://github.com/FlexNetOS/example --skill find --apply",
            "envctl agent list --kind skills",
            "envctl agent lock --check",
            "envctl agent doctor --scope global",
        )
    )]
    Agent {
        #[command(subcommand)]
        cmd: AgentCmd,
    },
    /// Manage this envctl installation: update the running binary, or uninstall the stack.
    #[command(
        name = "self",
        long_about = "Manage this envctl installation: update the running binary from GitHub releases, or uninstall the whole stack (assets, config / data / cache dirs, and the binary). Uninstall is DESTRUCTIVE and PREVIEW by default — pass --apply to delete.",
        after_help = envctl_examples!(
            "envctl self update",
            "envctl self update --json",
            "envctl self uninstall",
            "envctl self uninstall --apply --yes",
        )
    )]
    Manage {
        #[command(subcommand)]
        action: SelfAction,
    },
    /// Generate shell completion scripts (written to stdout).
    #[command(
        long_about = "Generate a shell completion script for envctl and write it to stdout so it can be sourced directly or redirected to a file. Supported shells: bash, zsh, fish, powershell, elvish.",
        after_help = envctl_examples!(
            "envctl completions bash",
            "envctl completions zsh",
            "envctl completions fish",
            "envctl completions powershell",
        )
    )]
    Completions {
        /// Target shell: bash | zsh | fish | powershell | elvish.
        shell: Shell,
    },
    /// Manage secrets: vault status, init/unlock/lock, secret CRUD, relay policies, CA, audit, run, mint-github, github-app.
    #[command(
        long_about = "Manage secrets stored in the secretd vault: vault status, initialization, unlocking, locking, secret CRUD (get/list/rm), relay policy management (create/revoke/mint), local CA management (init/issue/renew/revoke/trust-apply), audit log queries, ephemeral credential injection (run), GitHub App token minting, and GitHub App enrollment. All destructive verbs are dry-run by default — pass --apply to act.",
        after_help = envctl_examples!(
            "envctl secret status",
            "envctl secret init --apply",
            "envctl secret relay mint my-policy --ttl 3600",
            "envctl secret run --relay api -- curl https://api.example.com",
            "envctl secret mint-github --installation-id 42 --output json --ttl-secs 3600",
        )
    )]
    Secret {
        #[command(subcommand)]
        cmd: SecretCmd,
    },
}

/// Catalog control-plane subcommands. ADR-0003 keeps file import/diff/render/sync read-only until verifier-gated apply slices; lock writes require explicit opt-in.
#[derive(Subcommand)]
enum CatalogCmd {
    /// Import current files into an in-memory catalog snapshot.
    #[command(
        long_about = "Read manifest, lock, agent-env, Codex/MCP, hub registry, layout, secrets, and handoff surfaces into normalized in-memory tables. Read-only: no lock writes, no renders, no sync.",
        after_help = envctl_examples!(
            "envctl catalog scan",
            "envctl catalog scan --json",
        )
    )]
    Scan,
    /// List the catalog tables, row counts, columns, and purpose.
    #[command(
        long_about = "Print the normalized ADR-0003 table inventory so you can see what envctl imported, how many rows landed in each table, which columns are present, and what each table is for.",
        after_help = envctl_examples!(
            "envctl catalog tables",
            "envctl catalog tables --json",
        )
    )]
    Tables,
    /// Print one normalized catalog table.
    #[command(
        long_about = "Print one read-only catalog table. Names accept snake_case or kebab-case: components, component-hooks, paths, settings, env-vars, agent-assets, registries, config-files, codedb-file-imports, migration-evidence, observed-facts.",
        after_help = envctl_examples!(
            "envctl catalog table components",
            "envctl catalog table nix-components",
            "envctl catalog table component-hooks",
            "envctl catalog table codedb-file-imports",
            "envctl catalog table observed-facts --json",
        )
    )]
    Table {
        /// Table name, e.g. components or env-vars.
        name: String,
    },
    /// Import current files into normalized catalog rows with an explicit report.
    #[command(
        long_about = "Read manifest, lock, agent-env, Codex/MCP, hub registry, layout, secrets, and handoff surfaces into normalized catalog rows. Read-only: no lock writes, no renders, no sync.",
        after_help = envctl_examples!(
            "envctl catalog import",
            "envctl catalog import --json",
        )
    )]
    Import,
    /// Summarize config/env/path/toolchain coverage from the current catalog snapshot.
    #[command(
        long_about = "Analyze the current catalog snapshot into higher-level facets: table coverage, config formats and file kinds, env scopes and producers, path artifact and verification status, codedb parser hints, and likely toolchain-related signals. Read-only.",
        after_help = envctl_examples!(
            "envctl catalog analyze",
            "envctl catalog analyze --json",
        )
    )]
    Analyze,
    /// Report file/catalog/lock drift without writing files.
    #[command(
        long_about = "Compare the normalized catalog with source config files, lock state, registry drift, and observed facts. Read-only: no lock writes, no renders, no sync.",
        after_help = envctl_examples!(
            "envctl catalog diff",
            "envctl catalog diff --json",
        )
    )]
    Diff,
    /// Render deterministic catalog projections into an explicit output directory.
    #[command(
        long_about = "Render ADR-0003 generated-file projections into an explicit output directory for review. The output directory must be outside the repo root; current repo files are never mutated.",
        after_help = envctl_examples!(
            "envctl catalog render --out /tmp/envctl-catalog",
            "envctl catalog render --out /tmp/envctl-catalog --json",
        )
    )]
    Render {
        /// Output directory for generated projections; must be outside the repo root.
        #[arg(long, value_name = "DIR")]
        out: PathBuf,
        /// Optional root whose layout-derived paths and env exports should be rendered.
        #[arg(long = "target-root", value_name = "DIR")]
        target_root: Option<PathBuf>,
    },
    /// Preview bidirectional file/catalog reconciliation without mutating repo files.
    #[command(
        long_about = "Preview ADR-0003 bidirectional reconciliation. The command imports current files, reports drift, and can render projections into an out-of-repo directory. --apply is fail-closed until verifier-gated row edits land.",
        after_help = envctl_examples!(
            "envctl catalog sync",
            "envctl catalog sync --json",
            "envctl catalog sync --render-out /tmp/envctl-catalog-sync",
        )
    )]
    Sync {
        /// Optional output directory for generated projections; must be outside the repo root.
        #[arg(long = "render-out", value_name = "DIR")]
        render_out: Option<PathBuf>,
        /// Attempt apply. Currently fail-closed until verifier-gated row edit/apply support lands.
        #[arg(long)]
        apply: bool,
    },
    /// Check or accept the catalog lock projection.
    #[command(
        long_about = "Check or update ADR-0003's lock projection at manifest/envctl.lock. Default and --check are read-only; --apply writes only envctl.lock from the current manifest registry.",
        after_help = envctl_examples!(
            "envctl catalog lock",
            "envctl catalog lock --check --json",
            "envctl catalog lock --apply",
        )
    )]
    Lock {
        /// Exit non-zero when catalog/lock drift exists; never writes files.
        #[arg(long)]
        check: bool,
        /// Accept current manifest registry into manifest/envctl.lock.
        #[arg(long)]
        apply: bool,
    },
}

#[derive(Args, Clone, Debug, Default)]
struct CatalogRootArgs {
    /// Repository root to import into catalog tables. Defaults to the parent of the envctl manifest dir.
    #[arg(long = "repo-root", value_name = "DIR", global = true)]
    repo_root: Option<PathBuf>,
    /// Manifest directory for component rows. If omitted with --repo-root and no manifest exists under that root, catalog uses an empty registry.
    #[arg(long = "manifest-dir", value_name = "DIR", global = true)]
    manifest_dir: Option<PathBuf>,
}

/// The migration/adoption subcommands. All variants share optional scope and
/// component filters; mutating variants remain preview-only unless explicitly applied.
#[derive(Subcommand)]
enum MigrateCmd {
    /// Read the meta layout, manifests, agent assets, and protected substrates.
    #[command(
        long_about = "Read-only scan of the existing meta checkout: canonical FHS/XDG directories, manifest references to legacy/global paths, agent/Codex assets, and shared meta substrates such as loop_lib.",
        after_help = envctl_examples!(
            "envctl migrate scan",
            "envctl migrate scan --scope layout --scope meta-substrates",
            "envctl migrate scan --component rust",
        )
    )]
    Scan {
        #[arg(long = "scope", value_enum)]
        scopes: Vec<MigrateScopeArg>,
        #[arg(long = "component")]
        components: Vec<String>,
    },
    /// Build the migration/adoption worklist without writing anything.
    #[command(
        long_about = "Read-only plan for adopting old paths into the canonical meta-hosted FHS/XDG topology. The plan is the same engine report as scan, but with the verb set to plan for automation.",
        after_help = envctl_examples!(
            "envctl migrate plan",
            "envctl migrate plan --scope component-registry",
        )
    )]
    Plan {
        #[arg(long = "scope", value_enum)]
        scopes: Vec<MigrateScopeArg>,
        #[arg(long = "component")]
        components: Vec<String>,
    },
    /// Materialize canonical meta directories. Preview unless --apply is set.
    #[command(
        long_about = "Materialize the canonical `$META_ROOT` FHS/XDG directory structure and append a migration ledger entry. Without --apply this is a zero-write preview.",
        after_help = envctl_examples!(
            "envctl migrate apply",
            "envctl migrate apply --apply",
        )
    )]
    Apply {
        #[arg(long = "scope", value_enum)]
        scopes: Vec<MigrateScopeArg>,
        #[arg(long = "component")]
        components: Vec<String>,
        /// Actually create canonical directories and write the migration ledger.
        #[arg(long)]
        apply: bool,
    },
    /// Verify the migration/adoption plan is clean. Exits 1 if unresolved work remains.
    #[command(
        long_about = "Read-only verification gate. Exits non-zero if any migration debt remains, so CI/automation can block unsafe purges or releases.",
        after_help = envctl_examples!(
            "envctl migrate verify",
            "envctl migrate verify --scope layout",
        )
    )]
    Verify {
        #[arg(long = "scope", value_enum)]
        scopes: Vec<MigrateScopeArg>,
        #[arg(long = "component")]
        components: Vec<String>,
    },
    /// Strict upgrade-only purge surface. Refuses until adoption/parity is proven.
    #[command(
        long_about = "Strict upgrade-only purge guard. Dry-run reports why legacy roots are protected. Even with --apply --confirm, envctl refuses deletion unless a typed legacy path has verified canonical adoption evidence in the migration ledger.",
        after_help = envctl_examples!(
            "envctl migrate purge",
            "envctl migrate purge --apply --confirm",
        )
    )]
    Purge {
        #[arg(long = "scope", value_enum)]
        scopes: Vec<MigrateScopeArg>,
        #[arg(long = "component")]
        components: Vec<String>,
        /// Attempt the guarded purge path (still refuses without verified candidates).
        #[arg(long)]
        apply: bool,
        /// Confirm a destructive purge attempt.
        #[arg(long)]
        confirm: bool,
    },
}

/// CLI spelling for `MigrationScope`.
#[derive(Clone, Copy, Debug, ValueEnum)]
#[clap(rename_all = "kebab-case")]
enum MigrateScopeArg {
    All,
    Layout,
    ComponentRegistry,
    AgentAssets,
    MetaSubstrates,
    LegacyPaths,
}

impl From<MigrateScopeArg> for MigrationScope {
    fn from(scope: MigrateScopeArg) -> Self {
        match scope {
            MigrateScopeArg::All => MigrationScope::All,
            MigrateScopeArg::Layout => MigrationScope::Layout,
            MigrateScopeArg::ComponentRegistry => MigrationScope::ComponentRegistry,
            MigrateScopeArg::AgentAssets => MigrationScope::AgentAssets,
            MigrateScopeArg::MetaSubstrates => MigrationScope::MetaSubstrates,
            MigrateScopeArg::LegacyPaths => MigrationScope::LegacyPaths,
        }
    }
}

/// `envctl self {update,uninstall}` — manage the running installation (kasetto `ManageSelf`).
#[derive(Subcommand)]
enum SelfAction {
    /// Update envctl to the latest GitHub release (download + verify + atomic replace).
    #[command(
        long_about = "Check GitHub for the latest envctl release. If a newer version is available, download the matching binary, verify it, and atomically replace the current executable in place. Use --json for a machine-readable result.",
        after_help = envctl_examples!(
            "envctl self update",
            "envctl self update --json",
        )
    )]
    Update {
        /// Print update output as JSON.
        #[arg(long)]
        json: bool,
    },
    /// Completely uninstall envctl: assets, config/data/cache dirs, and the binary.
    /// DESTRUCTIVE — PREVIEW (dry-run) by default; `--apply` deletes; `--yes` skips the prompt.
    #[command(
        long_about = "DESTRUCTIVE: completely uninstall envctl — installed assets, the config / data / cache directories, and the envctl / envctl-gui binaries. PREVIEW (dry-run) by default; pass --apply to delete. On a TTY you are prompted to confirm; --yes skips the prompt (required to --apply in non-interactive mode).",
        after_help = envctl_examples!(
            "envctl self uninstall",
            "envctl self uninstall --apply",
            "envctl self uninstall --apply --yes",
        )
    )]
    Uninstall {
        /// Actually delete (else preview / zero writes).
        #[arg(long)]
        apply: bool,
        /// Skip the confirmation prompt (required in non-interactive mode).
        #[arg(long)]
        yes: bool,
    },
}

/// Serializable scope selector (clap) → `AgentScope`.
#[derive(Clone, Copy, Debug, clap::ValueEnum)]
enum ScopeArg {
    Global,
    Project,
}

impl From<ScopeArg> for AgentScope {
    fn from(s: ScopeArg) -> Self {
        match s {
            ScopeArg::Global => AgentScope::Global,
            ScopeArg::Project => AgentScope::Project,
        }
    }
}

/// Which asset kinds `agent list` shows (clap) → `AgentListKind`.
#[derive(Clone, Copy, Debug, clap::ValueEnum)]
enum ListKindArg {
    All,
    Skills,
    Mcps,
    Commands,
}

impl From<ListKindArg> for AgentListKind {
    fn from(k: ListKindArg) -> Self {
        match k {
            ListKindArg::All => AgentListKind::All,
            ListKindArg::Skills => AgentListKind::Skills,
            ListKindArg::Mcps => AgentListKind::Mcps,
            ListKindArg::Commands => AgentListKind::Commands,
        }
    }
}

/// The six agent-asset verbs. Each maps field-by-field to its `Agent*Spec`.
#[derive(Subcommand)]
enum AgentCmd {
    /// Reconcile installed assets with the config. PREVIEW by default; `--apply` writes.
    #[command(
        long_about = "Read the agent-env config, discover the requested skills / MCP servers / commands, then install, update, or remove local copies so the destination matches the config. PREVIEW by default; pass --apply to write. Use --locked (alias --frozen) to audit against the lock with zero network fetch (fail-closed if the lock cannot satisfy the config), or --update [NAME...] to re-resolve moving refs and rewrite the lock.",
        after_help = envctl_examples!(
            "envctl agent sync",
            "envctl agent sync --apply",
            "envctl agent sync --locked",
            "envctl agent sync --update --apply",
            "envctl agent sync --config agent-env.yaml --apply",
        )
    )]
    Sync {
        /// Config file path (else the default-config resolution / `$ENVCTL_AGENT_CONFIG`).
        #[arg(long)]
        config: Option<String>,
        /// Override the scope resolved from the config.
        #[arg(long, value_enum)]
        scope: Option<ScopeArg>,
        /// Write changes (else preview / zero writes).
        #[arg(long)]
        apply: bool,
        /// Audit the lock with ZERO network fetch (fail-closed if unsatisfied).
        #[arg(long, visible_alias = "frozen")]
        locked: bool,
        /// Re-resolve the named packages' refs (no names = all) and rewrite the lock.
        #[arg(long, short = 'u', num_args = 0.., value_name = "NAME")]
        update: Option<Vec<String>>,
    },
    /// Add a source to the config (then sync, unless `--no-sync`). PREVIEW by default.
    #[command(
        long_about = "Append a skill / MCP / command source to your agent-env.yaml (preserving comments), then run a sync to install it. Use the kind-tagged flags --skill / --mcp / --command (each repeatable) to name entries; a single add can touch several lists when a repo ships more than one kind. The source may be a repo URL, a deep blob/tree browse URL, a local path, or a SOURCE@REF shorthand; --ref / --branch / --sub-dir override the derived pieces. The source is fetched once to verify it resolves (skip with --no-verify). PREVIEW by default; pass --apply to write, or --no-sync to edit the config without installing.",
        after_help = envctl_examples!(
            "envctl agent add https://github.com/FlexNetOS/example --apply",
            "envctl agent add https://github.com/FlexNetOS/example --skill alpha --skill beta --apply",
            "envctl agent add https://github.com/FlexNetOS/example --skill find --mcp github --command review --apply",
            "envctl agent add https://github.com/FlexNetOS/example --ref v2.0 --no-sync --apply",
            "envctl agent add https://github.com/FlexNetOS/example --config agent-env.yaml --apply",
        )
    )]
    Add {
        /// The source (git URL / local path) to add.
        source: String,
        /// Restrict to named skills (repeatable).
        #[arg(long = "skill")]
        skill: Vec<String>,
        /// Restrict to named MCP servers (repeatable).
        #[arg(long = "mcp")]
        mcp: Vec<String>,
        /// Restrict to named commands (repeatable).
        #[arg(long = "command")]
        command: Vec<String>,
        /// Pin the source to a git ref (mutually exclusive with `--branch`).
        #[arg(long = "ref", value_name = "REF")]
        git_ref: Option<String>,
        /// Track a git branch (mutually exclusive with `--ref`).
        #[arg(long)]
        branch: Option<String>,
        /// Sub-directory within the source to draw assets from.
        #[arg(long)]
        sub_dir: Option<String>,
        #[arg(long)]
        config: Option<String>,
        #[arg(long, value_enum)]
        scope: Option<ScopeArg>,
        /// Write changes (else preview / zero writes).
        #[arg(long)]
        apply: bool,
        /// Edit the manifest only; do NOT sync afterwards.
        #[arg(long)]
        no_sync: bool,
        /// Skip post-add verification.
        #[arg(long)]
        no_verify: bool,
        /// Zero-network mode (requires `--no-sync` on `add`).
        #[arg(long, visible_alias = "frozen")]
        locked: bool,
        #[arg(long, short = 'u', num_args = 0.., value_name = "NAME")]
        update: Option<Vec<String>>,
    },
    /// Remove a source from the config (then sync, unless `--no-sync`). PREVIEW by default.
    #[command(
        visible_alias = "rm",
        long_about = "Delete entries from your agent-env.yaml (preserving comments), then run a sync so the now-unconfigured assets are pruned from disk and the lock. Mirrors `add`: the kind-tagged flags --skill / --mcp / --command (each repeatable) name entries to subtract; with no kind flags the source is removed from every list it appears in. When multiple entries share a source URL, pass --ref or --branch to disambiguate. PREVIEW by default; pass --apply to write, or --no-sync to edit the config without pruning.",
        after_help = envctl_examples!(
            "envctl agent remove https://github.com/FlexNetOS/example --apply",
            "envctl agent remove https://github.com/FlexNetOS/example --skill find --apply",
            "envctl agent remove https://github.com/FlexNetOS/example --mcp github --command review --apply",
            "envctl agent remove https://github.com/FlexNetOS/example --no-sync --apply",
            "envctl agent rm ./local/pack --apply",
        )
    )]
    Remove {
        /// The source to remove.
        source: String,
        #[arg(long = "skill")]
        skill: Vec<String>,
        #[arg(long = "mcp")]
        mcp: Vec<String>,
        #[arg(long = "command")]
        command: Vec<String>,
        #[arg(long = "ref", value_name = "REF")]
        git_ref: Option<String>,
        #[arg(long)]
        branch: Option<String>,
        #[arg(long)]
        sub_dir: Option<String>,
        #[arg(long)]
        config: Option<String>,
        #[arg(long, value_enum)]
        scope: Option<ScopeArg>,
        /// Write changes (else preview / zero writes).
        #[arg(long)]
        apply: bool,
        #[arg(long)]
        no_sync: bool,
        #[arg(long, visible_alias = "frozen")]
        locked: bool,
        #[arg(long, short = 'u', num_args = 0.., value_name = "NAME")]
        update: Option<Vec<String>>,
    },
    /// Write/verify `agent-env.lock`. `--check` audits (exit 1 on drift); else rewrite.
    #[command(
        long_about = "Re-resolve every source (re-resolving moving refs) and write agent-env.lock without installing to destinations, so the lock is immediately usable with `sync --locked`. With --check (alias --frozen) it audits the lock against the config and exits 1 on drift without writing. Use --upgrade-package/-P NAME to restrict the re-resolve to sources providing those skills, --locked to make a --check audit zero-network, and --scope to override the resolved scope.",
        after_help = envctl_examples!(
            "envctl agent lock",
            "envctl agent lock --check",
            "envctl agent lock --upgrade-package alpha --upgrade-package beta",
            "envctl agent lock --scope project",
            "envctl agent lock --config agent-env.yaml",
        )
    )]
    Lock {
        #[arg(long)]
        config: Option<String>,
        #[arg(long, value_enum)]
        scope: Option<ScopeArg>,
        /// Verify the lock matches the config without writing (exit 1 on drift).
        // `--frozen` aliases `--check` (CI-friendly). Unlike kasetto we keep ONLY the `frozen`
        // alias here — envctl's Lock has a distinct `--locked` flag (the zero-network audit
        // knob), so a `locked` alias on `--check` would collide.
        #[arg(long, visible_alias = "frozen")]
        check: bool,
        /// Restrict the re-resolve to sources providing these skills (repeatable).
        #[arg(long = "upgrade-package", short = 'P', value_name = "NAME")]
        upgrade_package: Vec<String>,
        /// With `--check`: make the audit zero-network.
        #[arg(long)]
        locked: bool,
    },
    /// Read-only inventory of installed assets (skills + MCP servers + commands).
    #[command(
        long_about = "Read the installed assets from the lock file and print them as plain tables: skills, MCP servers, and commands, each with their scope and source. Filter the output with --kind skills|mcps|commands|all (default: all). Read-only. Use --json (global) for scripting.",
        after_help = envctl_examples!(
            "envctl agent list",
            "envctl agent list --kind skills",
            "envctl agent list --json",
        )
    )]
    List {
        #[arg(long, value_enum)]
        scope: Option<ScopeArg>,
        /// Filter the inventory by kind.
        #[arg(long, value_enum, default_value = "all")]
        kind: ListKindArg,
    },
    /// Prune assets orphaned from the config. PREVIEW by default; `--apply` writes.
    #[command(
        long_about = "Remove installed assets that are no longer referenced by the config and reset the corresponding lock entries. PREVIEW by default; pass --apply to write. Use --scope to override the resolved scope.",
        after_help = envctl_examples!(
            "envctl agent clean",
            "envctl agent clean --apply",
            "envctl agent clean --scope project --apply",
        )
    )]
    Clean {
        /// Config file path (else the default-config resolution / `$ENVCTL_AGENT_CONFIG`).
        #[arg(long)]
        config: Option<String>,
        #[arg(long, value_enum)]
        scope: Option<ScopeArg>,
        /// Write changes (else preview / zero writes).
        #[arg(long)]
        apply: bool,
    },
    /// Create a starter agent-env config file (`agent-env.yaml`).
    #[command(
        long_about = "Write a commented starter agent-env.yaml you can edit before running sync. By default it writes ./agent-env.yaml; with --global it writes under $XDG_CONFIG_HOME/agent-env/agent-env.yaml. If the target already exists you are prompted to overwrite unless --force/-f is set.",
        after_help = envctl_examples!(
            "envctl agent init",
            "envctl agent init --global",
            "envctl agent init --force",
        )
    )]
    Init {
        /// Write the global config under `$XDG_CONFIG_HOME/agent-env/agent-env.yaml`.
        #[arg(long)]
        global: bool,
        /// Overwrite an existing config file.
        #[arg(short = 'f', long)]
        force: bool,
    },
    /// Read-only diagnostics: version, lock, scope, inventory, command-dir writability, updates.
    #[command(
        long_about = "Inspect the local agent-env setup: version, lock file, active scope and installation paths, asset inventory, command-directory writability, failed installs from the latest sync, and update status. Read-only. Use --scope to override the resolved scope and --json (global) for a machine-readable report.",
        after_help = envctl_examples!(
            "envctl agent doctor",
            "envctl agent doctor --scope global",
            "envctl agent doctor --json",
        )
    )]
    Doctor {
        /// Override the scope resolved from the config.
        #[arg(long, value_enum)]
        scope: Option<ScopeArg>,
    },
}

/// `envctl secret` subcommand group — mirrors the full `secretctl` clap surface.
#[derive(Subcommand)]
pub enum SecretCmd {
    /// Vault lock status (no unlock side effect).
    #[command(
        long_about = "Print the vault's current lock state (locked or unlocked) without modifying it. A read-only diagnostic that confirms whether the DEK is loaded in memory.",
        after_help = envctl_examples!(
            "envctl secret status",
        )
    )]
    Status {},
    /// Initialize a fresh vault: mint the DEK + enroll keyslots. Dry-run preview unless --apply.
    #[command(
        long_about = "Create a brand-new vault: generate a data-encryption key (DEK), establish the CA issuer, and enroll keyslots (USB or passphrase). PREVIEW by default; pass --apply to perform the initialization.",
        after_help = envctl_examples!(
            "envctl secret init",
            "envctl secret init --apply",
            "envctl secret init --enroll-usb --usb-partuuid abc123",
            "envctl secret init --passphrase-stdin --apply",
        )
    )]
    Init {
        #[arg(long)]
        passphrase_stdin: bool,
        #[arg(long)]
        enroll_usb: bool,
        #[arg(long = "usb-partuuid")]
        usb_partuuid: Option<String>,
        /// Actually initialize. Without it, prints a dry-run preview and mutates nothing (CF-8).
        #[arg(long)]
        apply: bool,
    },
    /// Unlock the vault (USB-first; passphrase only if USB absent).
    #[command(
        long_about = "Unlock the existing vault. Tries USB first (if enrolled), then falls back to passphrase. On success the DEK is loaded into memory for subsequent secret operations.",
        after_help = envctl_examples!(
            "envctl secret unlock",
            "envctl secret unlock --passphrase-stdin",
        )
    )]
    Unlock {
        #[arg(long)]
        passphrase_stdin: bool,
    },
    /// Zeroize the DEK + CA issuer in RAM (the true panic stop).
    #[command(
        long_about = "Zero out the DEK and CA issuer from memory. This is the emergency shutdown — no secrets remain accessible afterward until the next unlock.",
        after_help = envctl_examples!(
            "envctl secret lock",
        )
    )]
    Lock {},
    /// Manage stored secrets.
    #[command(
        long_about = "Store, retrieve, list, remove, or rotate individual secrets in the vault. All destructive verbs (rm) are dry-run by default; pass --apply to act.",
        after_help = envctl_examples!(
            "envctl secret add api-key --provider github",
            "envctl secret get api-key --reveal",
            "envctl secret list",
            "envctl secret rm old-key --apply",
            "envctl secret rotate db-password --value-stdin",
        )
    )]
    Secret {
        #[command(subcommand)]
        cmd: SecretSubCmd,
    },
    /// Manage relay policies + mint bearers.
    #[command(
        long_about = "Create, revoke, and list relay policies that gate credential forwarding. Mint short-lived bearer tokens scoped to a policy for downstream services.",
        after_help = envctl_examples!(
            "envctl secret relay create api --secret github --provider okta",
            "envctl secret relay list",
            "envctl secret relay revoke api --apply",
            "envctl secret relay mint api --ttl 3600",
        )
    )]
    Relay {
        #[command(subcommand)]
        cmd: RelaySubCmd,
    },
    /// Manage the local CA, leaf certs, and trust wiring.
    #[command(
        long_about = "Initialize a local Certificate Authority (or rotate it), issue and renew leaf certificates with SANs, and apply trust anchors to the system bundle.",
        after_help = envctl_examples!(
            "envctl secret ca init --apply",
            "envctl secret ca issue my.host.local --san alt.host --ttl-days 90",
            "envctl secret ca renew my.host.local --apply",
            "envctl secret ca trust system-bundle --apply",
        )
    )]
    Ca {
        #[command(subcommand)]
        cmd: CaSubCmd,
    },
    /// Query the tamper-evident audit log.
    #[command(
        long_about = "Read entries from the vault's tamper-evident audit log. Filter by actor, relay identity, or time range. Returns a paginated list of secret operations.",
        after_help = envctl_examples!(
            "envctl secret audit --limit 10",
            "envctl secret audit --actor alice --since 2026-01-01T00:00:00Z",
        )
    )]
    Audit(AuditArgs),
    /// Run a command with relay credentials injected into the child only.
    #[command(
        long_about = "Execute a child process with relay bearer tokens and environment variables injected — visible only to that process, never persisted or leaked to parent.",
        after_help = envctl_examples!(
            "envctl secret run --relay api -- curl https://api.example.com",
            "envctl secret run --ephemeral -- my-command",
        )
    )]
    Run(RunArgs),
    /// Mint a GitHub App installation access token from the vault-sealed key (TASK-0020).
    #[command(
        long_about = "Mint a short-lived GitHub App installation access token using the vault-sealed App private key. Output is always JSON (the frozen machine contract).",
        after_help = envctl_examples!(
            "envctl secret mint-github --installation-id 42 --output json --ttl-secs 3600",
        )
    )]
    #[command(name = "mint-github")]
    MintGithub(MintGithubArgs),
    /// Enroll the GitHub App credential into the unlocked vault (TASK-0026).
    #[command(
        long_about = "Enroll a GitHub App's identity (app ID + private key) into the unlocked vault, or update just the app-id on an existing enrollment. Supports token revocation.",
        after_help = envctl_examples!(
            "envctl secret github-app enroll --app-id 123456 --private-key ./key.pem --apply",
            "envctl secret github-app set-app-id 789012 --apply",
            "envctl secret github-app revoke-token --token ghp_xxx",
        )
    )]
    #[command(name = "github-app")]
    GithubApp {
        #[command(subcommand)]
        cmd: GithubAppSubCmd,
    },
}

#[derive(Subcommand)]
pub enum SecretSubCmd {
    /// Add a new secret to the vault. PREVIEW by default; --apply to write.
    #[command(
        long_about = "Store a new secret identified by name under a given provider (e.g. 'github', 'okta'). The value is read from stdin when --value-stdin is set.",
        after_help = envctl_examples!(
            "envctl secret add api-key --provider github",
            "cat key.pem | envctl secret add cert --provider file --value-stdin",
        )
    )]
    Add {
        name: String,
        #[arg(long)]
        provider: String,
        #[arg(long)]
        value_stdin: bool,
        #[arg(long)]
        note: Option<String>,
        #[arg(long)]
        overwrite: bool,
        #[arg(long)]
        broker_only: bool,
    },
    /// Get a secret by name. PREVIEW (JSON) by default; --reveal prints the value; --apply writes it to disk.
    #[command(
        long_about = "Retrieve a stored secret by name. Default is JSON output; use --reveal to print the raw value, or --apply to write it to a file.",
        after_help = envctl_examples!(
            "envctl secret get api-key",
            "envctl secret get api-key --reveal",
            "envctl secret get api-key --apply",
        )
    )]
    Get {
        name: String,
        #[arg(long)]
        reveal: bool,
        #[arg(long)]
        apply: bool,
        #[arg(long)]
        confirm: bool,
    },
    /// List secrets, optionally filtered by provider.
    #[command(
        long_about = "List stored secrets with their metadata (provider, note). Use --provider to filter.",
        after_help = envctl_examples!(
            "envctl secret list",
            "envctl secret list --provider github",
        )
    )]
    List {
        #[arg(long)]
        provider: Option<String>,
    },
    /// Remove a secret by name. PREVIEW by default; --apply to delete.
    #[command(
        long_about = "Remove a stored secret by name. PREVIEW (dry-run) by default; pass --apply to actually delete the entry from the vault.",
        after_help = envctl_examples!(
            "envctl secret rm old-key",
            "envctl secret rm old-key --apply --confirm",
        )
    )]
    Rm {
        name: String,
        #[arg(long)]
        apply: bool,
        #[arg(long)]
        confirm: bool,
    },
    /// Rotate a secret by replacing its value. PREVIEW by default; --apply to commit the new value.
    #[command(
        long_about = "Replace an existing secret's value (rotation). Value can come from stdin with --value-stdin.",
        after_help = envctl_examples!(
            "envctl secret rotate db-password",
            "cat new-key.pem | envctl secret rotate cert --apply --value-stdin",
        )
    )]
    Rotate {
        name: String,
        #[arg(long)]
        value_stdin: bool,
        #[arg(long)]
        apply: bool,
    },
}

#[derive(Subcommand)]
pub enum RelaySubCmd {
    /// Create a new relay policy with the given configuration. PREVIEW by default; --apply to create.
    #[command(
        long_about = "Define a forwarding policy — which secret to use as the upstream, auth provider, hosts/paths/methods, rate/quota limits, and optional expiry.",
        after_help = envctl_examples!(
            "envctl secret relay create api --secret github --provider okta --mode passthrough",
            "envctl secret relay create api --host localhost:8080 --path /api/* --method GET --disabled",
        )
    )]
    Create {
        name: String,
        #[arg(long)]
        secret: String,
        #[arg(long)]
        provider: String,
        #[arg(long)]
        mode: String,
        #[arg(long)]
        upstream_base: Option<String>,
        #[arg(long = "host")]
        hosts: Vec<String>,
        #[arg(long = "path")]
        paths: Vec<String>,
        #[arg(long = "method")]
        methods: Vec<String>,
        #[arg(long)]
        expires: Option<String>,
        #[arg(long)]
        rate: Option<u32>,
        #[arg(long)]
        quota: Option<u64>,
        #[arg(long)]
        disabled: bool,
    },
    /// Revoke a relay policy. PREVIEW by default; --apply to revoke.
    #[command(
        long_about = "Remove a relay policy. All tokens minted under it become invalid immediately.",
        after_help = envctl_examples!(
            "envctl secret relay revoke api",
            "envctl secret relay revoke api --apply --confirm",
        )
    )]
    Revoke {
        name: String,
        #[arg(long)]
        apply: bool,
        #[arg(long)]
        confirm: bool,
    },
    /// Revoke a specific bearer token issued by a relay. PREVIEW by default; --apply to revoke.
    #[command(
        long_about = "Revoke an individual bearer token (by token ID) issued under any relay policy.",
        after_help = envctl_examples!(
            "envctl secret relay revoke-token tkn_xxx",
            "envctl secret relay revoke-token tkn_xxx --apply",
        )
    )]
    RevokeToken {
        token_id: String,
        #[arg(long)]
        apply: bool,
    },
    /// List relay policies, optionally including disabled ones.
    #[command(
        long_about = "List all configured relay policies with their metadata (mode, hosts, expiry). Use --all to include disabled policies.",
        after_help = envctl_examples!(
            "envctl secret relay list",
            "envctl secret relay list --all",
        )
    )]
    List {
        #[arg(long)]
        all: bool,
    },
    /// Mint a short-lived bearer token under the named relay policy.
    #[command(
        long_about = "Generate a scoped bearer token under an existing relay policy. Specify TTL, mode, target repos/permissions for GitHub-style scopes.",
        after_help = envctl_examples!(
            "envctl secret relay mint api --ttl 3600",
            "envctl secret relay mint gh --repo org/repo --perm write",
        )
    )]
    Mint {
        name: String,
        #[arg(long)]
        ttl: Option<String>,
        #[arg(long)]
        mode: Option<String>,
        #[arg(long)]
        provider: Option<String>,
        #[arg(long = "repo")]
        repos: Vec<String>,
        #[arg(long = "perm")]
        perms: Vec<String>,
    },
}

#[derive(Subcommand)]
pub enum CaSubCmd {
    /// Initialize a local CA. PREVIEW by default; --apply to create.
    #[command(
        long_about = "Create a new Certificate Authority (key pair + self-signed cert) in the vault. If one already exists this is a no-op preview.",
        after_help = envctl_examples!(
            "envctl secret ca init",
            "envctl secret ca init --apply",
        )
    )]
    Init {
        #[arg(long)]
        apply: bool,
    },
    /// Rotate the CA key pair. PREVIEW by default; --apply to rotate + confirm.
    #[command(
        long_about = "Generate a new CA key pair and re-issue all leaf certificates. Requires confirmation when --apply is used to prevent accidental rotation.",
        after_help = envctl_examples!(
            "envctl secret ca rotate",
            "envctl secret ca rotate --apply --confirm",
        )
    )]
    Rotate {
        #[arg(long)]
        apply: bool,
        #[arg(long)]
        confirm: bool,
    },
    /// Issue a new leaf certificate for the given common name with optional SANs.
    #[command(
        long_about = "Sign a new leaf certificate under the local CA for a specific hostname (CN) with alternative names (SANs), configurable TTL and key usage.",
        after_help = envctl_examples!(
            "envctl secret ca issue my.host.local --san alt.host --ttl-days 90",
            "envctl secret ca issue api.internal --san localhost --usage serverAuth",
        )
    )]
    Issue {
        cn: String,
        #[arg(long = "san")]
        sans: Vec<String>,
        #[arg(long)]
        ttl_days: Option<u64>,
        #[arg(long)]
        usage: String,
    },
    /// Renew an existing leaf certificate. PREVIEW by default; --apply to issue the renewed cert.
    #[command(
        long_about = "Renew a previously-issued leaf certificate before it expires. Returns a new cert signed under the current CA.",
        after_help = envctl_examples!(
            "envctl secret ca renew my.host.local",
            "envctl secret ca renew my.host.local --apply",
        )
    )]
    Renew {
        cn: String,
        #[arg(long)]
        apply: bool,
    },
    /// Revoke a leaf certificate. PREVIEW by default; --apply to revoke.
    #[command(
        long_about = "Revoke (and add to the CA's CRL) a previously-issued leaf certificate. Requires confirmation when --apply is used.",
        after_help = envctl_examples!(
            "envctl secret ca revoke my.host.local",
            "envctl secret ca revoke my.host.local --apply --confirm",
        )
    )]
    Revoke {
        cn: String,
        #[arg(long)]
        apply: bool,
        #[arg(long)]
        confirm: bool,
    },
    /// Apply trust anchors to system or local trust stores.
    #[command(
        long_about = "Install the CA certificate (and optionally intermediate certs) into the OS trust store so leaf certs are validated automatically.",
        after_help = envctl_examples!(
            "envctl secret ca trust --target /etc/ssl/certs/ca-certificates.crt",
            "envctl secret ca trust --system-bundle --apply --confirm",
        )
    )]
    Trust {
        targets: Vec<String>,
        #[arg(long)]
        system_bundle: bool,
        #[arg(long)]
        apply: bool,
        #[arg(long)]
        confirm: bool,
    },
}

#[derive(Subcommand)]
pub enum GithubAppSubCmd {
    /// Enroll a GitHub App's identity into the vault. PREVIEW by default; --apply to enroll.
    #[command(
        long_about = "Store a GitHub App's credentials (app ID + private key PEM) in the unlocked vault for use by mint-github and other relay consumers.",
        after_help = envctl_examples!(
            "envctl secret github-app enroll --app-id 123456 --private-key ./key.pem --apply",
        )
    )]
    Enroll {
        #[arg(long = "app-id")]
        app_id: String,
        #[arg(long = "private-key")]
        private_key: String,
        #[arg(long)]
        apply: bool,
    },
    /// Update just the app-id on an existing GitHub App enrollment.
    #[command(
        long_about = "Update only the app-id field on an existing GitHub App enrollment without touching the private key. Useful when migrating to a new App.",
        after_help = envctl_examples!(
            "envctl secret github-app set-app-id 789012 --apply",
        )
    )]
    SetAppId {
        #[arg(long = "app-id")]
        app_id: String,
        #[arg(long)]
        apply: bool,
    },
    /// Revoke a GitHub App installation token from the vault.
    #[command(
        long_about = "Revoke a specific GitHub App installation access token (identified by token string or installation ID). Prevents reuse of that token.",
        after_help = envctl_examples!(
            "envctl secret github-app revoke-token --token ghp_xxx",
            "envctl secret github-app revoke-token --token ghp_xxx --apply",
        )
    )]
    RevokeToken {
        #[arg(long = "token")]
        token: String,
        #[arg(long = "installation-id")]
        installation_id: Option<u64>,
        #[arg(long)]
        apply: bool,
    },
}

// Helper structs for the verbs that are args (not sub-subcommands)
#[derive(Args)]
pub struct AuditArgs {
    /// Filter by actor.
    #[arg(long)]
    pub actor: Option<String>,
    /// Filter by relay identity.
    #[arg(long)]
    pub relay: Option<String>,
    /// Start time (ISO 8601).
    #[arg(long)]
    pub since: Option<String>,
    /// End time (ISO 8601).
    #[arg(long)]
    pub until: Option<String>,
    /// Max entries to return.
    #[arg(long)]
    pub limit: Option<u32>,
}

#[derive(Args)]
pub struct RunArgs {
    /// Relay identity (repeated for multiple relays).
    #[arg(long = "relay")]
    pub relays: Vec<String>,
    /// Override the relay's auth provider.
    #[arg(long)]
    pub provider: Option<String>,
    /// Mint a one-off ephemeral bearer for this process.
    #[arg(long)]
    pub ephemeral: bool,
    /// Skip loading the default envctl-managed `$META_ROOT/.config/env-ctl/relay.toml` profile.
    #[arg(long = "no-profile")]
    pub no_profile: bool,
    /// Use an explicit relay config path instead of the default.
    #[arg(long)]
    pub profile: Option<String>,
    /// Command to run with relay credentials injected into the child only.
    #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
    pub argv: Vec<String>,
}

#[derive(Args)]
pub struct MintGithubArgs {
    /// GitHub App installation ID (required).
    #[arg(long = "installation-id")]
    pub installation_id: u64,
    /// Filter to specific repository IDs.
    #[arg(long = "repository-ids", value_delimiter = ',')]
    pub repository_ids: Vec<String>,
    /// Requested permissions (comma-separated).
    #[arg(long = "permissions", value_delimiter = ',')]
    pub permissions: Vec<String>,
    /// TTL in seconds for the minted token.
    #[arg(long = "ttl-secs")]
    pub ttl_secs: Option<u64>,
    /// Output format. Only `json` is supported (the frozen machine contract). Required.
    #[arg(long = "output")]
    pub output: String,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    // `dashboard` is manifest-INDEPENDENT (it reads `.meta.yaml`, never the
    // component registry), so it must work from any cwd without a `manifest/` dir
    // — e.g. when the `meta dashboard` plugin shells to `envctl dashboard` from the
    // meta root. Use a detached engine (empty registry) for it; every other verb
    // still requires the real manifest.
    // `dashboard` and `env` are manifest-INDEPENDENT (they read `.meta.yaml`, not
    // the component registry), so they must work from any cwd without a `manifest/`
    // dir. Use a detached engine for them; every other verb requires the manifest.
    // Resolve presentation knobs ONCE (color side effects happen here) and publish them.
    let plain = resolve_plain(cli.no_color, cli.color);
    let _ = OUTPUT.set(OutputCtx {
        json: cli.json,
        quiet: cli.quiet,
        verbose: cli.verbose,
        plain,
    });

    // Spawn the background update check up front (best-effort, silent on failure). Suppress the
    // end-of-run notice for machine-readable / scripted / version-printing commands.
    let update_handle = envctl_engine::update_notifier::spawn_background_check();
    let suppress_notice = should_suppress_notice(&cli.cmd, cli.json, cli.quiet);
    if !suppress_notice {
        // Give the cache a brief moment to populate (matches kasetto's 800ms).
        envctl_engine::update_notifier::wait_for_check(update_handle, Duration::from_millis(800));
    }

    let engine = if matches!(
        cli.cmd,
        Cmd::Dashboard { .. } | Cmd::Env { .. } | Cmd::Migration { .. }
    ) || matches!(
        cli.cmd,
        Cmd::Catalog {
            roots: CatalogRootArgs {
                repo_root: Some(_),
                ..
            },
            ..
        }
    ) || matches!(
        cli.cmd,
        Cmd::Catalog {
            roots: CatalogRootArgs {
                manifest_dir: Some(_),
                ..
            },
            ..
        }
    ) {
        Engine::detached()
    } else {
        Engine::load_default()?
    };
    let json = cli.json;

    let result = match cli.cmd {
        Cmd::AutoDetect { .. } => {
            // Read-only: run on the main thread and print the returned report.
            let (sink, _rx) = EventSink::channel();
            let report = engine.detect(&sink)?;
            if json {
                println!("{}", serde_json::to_string_pretty(&report)?);
            } else {
                print_report(&report);
            }
            Ok(())
        }
        Cmd::Graph {
            impact,
            why,
            dot,
            live,
        } => {
            use envctl_engine::graph;
            let live_report = if live {
                let (sink, _rx) = EventSink::channel();
                Some(engine.detect(&sink)?)
            } else {
                None
            };
            let reg = engine.registry();
            if dot {
                print!("{}", graph::to_dot(reg, live_report.as_ref()));
            } else if json {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&graph::to_json(reg, live_report.as_ref()))?
                );
            } else if let Some(id) = impact {
                // audit fix (minor): reflect unknown-component failure in exit code.
                if !print_impact(reg, &id) {
                    std::process::exit(2);
                }
            } else if let Some(id) = why {
                if !print_why(reg, &id) {
                    std::process::exit(2);
                }
            } else {
                print_graph_summary(reg);
                if let Some(rep) = live_report.as_ref() {
                    println!("{}", DriftSummary::from_items(&rep.drift));
                }
            }
            Ok(())
        }
        Cmd::Registry { check } => {
            let report = engine.hub_registry()?;
            if json {
                println!("{}", serde_json::to_string_pretty(&report)?);
            } else {
                print_hub_registry(&report);
            }
            if check && !report.clean() {
                std::process::exit(1);
            }
            Ok(())
        }
        Cmd::Catalog { roots, cmd } => run_catalog(&engine, roots, cmd, json),
        Cmd::Lock { check } => {
            use envctl_engine::lock;
            let reg = engine.registry();
            let dir = engine.manifest_dir();
            if check {
                let locked = lock::LockFile::load(dir)?;
                let drift = lock::diff(reg, &locked);
                if json {
                    let items: Vec<_> = drift
                        .iter()
                        .map(|(id, k)| serde_json::json!({"component": id, "drift": k}))
                        .collect();
                    println!(
                        "{}",
                        serde_json::to_string_pretty(
                            &serde_json::json!({"locked": drift.is_empty(), "drift": items})
                        )?
                    );
                } else if drift.is_empty() {
                    println!(
                        "\x1b[1;32m✓ envctl.lock matches the manifest ({} components)\x1b[0m",
                        reg.len()
                    );
                } else {
                    println!(
                        "\x1b[1;33m✗ lock drift ({}): manifest changed without re-locking\x1b[0m",
                        drift.len()
                    );
                    for (id, k) in &drift {
                        println!("  {:?}  {id}", k);
                    }
                    println!("  run `envctl lock` to update.");
                }
                if !drift.is_empty() {
                    std::process::exit(1);
                }
            } else {
                let mut lf = lock::generate(reg);
                lf.save(dir)?;
                println!(
                    "wrote {} ({} components)",
                    lock::lock_path(dir).display(),
                    reg.len()
                );
            }
            Ok(())
        }
        Cmd::Doctor => print_doctor(&engine, json),
        Cmd::Dashboard {
            meta_file,
            panes_per_tab,
            name,
            deploy,
            apply,
            force,
        } => run_dashboard(
            &engine,
            DashboardArgs {
                meta_file,
                panes_per_tab,
                name,
                deploy,
                apply,
                force,
            },
            json,
        ),
        Cmd::Env {
            meta_file,
            toolchains,
            materialize,
        } => run_env(meta_file, toolchains, materialize, json),
        Cmd::Migrate { cmd } => run_migrate(&engine, cmd, json),
        Cmd::Migration { db, cmd } => migration_cmd::run_migration(db, cmd, json),
        Cmd::Agent { cmd } => run_agent(engine, cmd, json),
        Cmd::Secret { cmd } => run_secret(cmd, json),
        Cmd::Completions { shell } => run_completions(shell),
        Cmd::Manage { action } => match action {
            SelfAction::Update { json: action_json } => self_update::run(json || action_json),
            SelfAction::Uninstall { apply, yes } => run_self_uninstall(engine, apply, yes, json),
        },
        // Interactive add-repo connect: handled on the MAIN thread so the agent
        // attaches to the real terminal.
        other if matches!(&other, Cmd::AddRepo { connect: true, .. }) => {
            handle_connect(engine, other, json)
        }
        other => {
            // Usage fail-fast (exit 2) before spawning the worker. The executor
            // also enforces these authoritatively (the GUI hits that path).
            if let Cmd::Reset {
                targets,
                all,
                confirm,
                purge,
                ..
            } = &other
            {
                if targets.is_empty() && !(*all && *confirm) {
                    eprintln!("envctl: refusing whole-roster reset — pass --all --confirm");
                    std::process::exit(2);
                }
                if *purge && !*confirm {
                    eprintln!("envctl: refusing --purge without --confirm");
                    std::process::exit(2);
                }
            }
            run_action(engine, other, json)
        }
    };

    // End-of-run "new version available" notice — only on success, never under suppress, and
    // only on a TTY. The engine decided whether an update exists; the CLI renders the line.
    if result.is_ok() && !suppress_notice && std::io::stdout().is_terminal() {
        let current = env!("CARGO_PKG_VERSION");
        if let Some((cur, latest)) = envctl_engine::update_notifier::available_update(current) {
            println!("{}", render_update_notice(&cur, &latest));
        }
    }
    result
}

fn migration_spec(scopes: Vec<MigrateScopeArg>, components: Vec<String>) -> MigrationSpec {
    MigrationSpec {
        scopes: if scopes.is_empty() {
            vec![MigrationScope::All]
        } else {
            scopes.into_iter().map(Into::into).collect()
        },
        components,
    }
}

fn run_migrate(engine: &Engine, cmd: MigrateCmd, json: bool) -> anyhow::Result<()> {
    let (sink, _rx) = EventSink::channel();
    let (report, hard_purge_attempt) = match cmd {
        MigrateCmd::Scan { scopes, components } => (
            engine.migrate_scan(migration_spec(scopes, components), &sink)?,
            false,
        ),
        MigrateCmd::Plan { scopes, components } => (
            engine.migrate_plan(migration_spec(scopes, components), &sink)?,
            false,
        ),
        MigrateCmd::Apply {
            scopes,
            components,
            apply,
        } => (
            engine.migrate_apply(migration_spec(scopes, components), apply, &sink)?,
            false,
        ),
        MigrateCmd::Verify { scopes, components } => (
            engine.migrate_verify(migration_spec(scopes, components), &sink)?,
            false,
        ),
        MigrateCmd::Purge {
            scopes,
            components,
            apply,
            confirm,
        } => (
            engine.migrate_purge(migration_spec(scopes, components), apply, confirm, &sink)?,
            apply && confirm,
        ),
    };

    if json {
        println!("{}", serde_json::to_string_pretty(&report)?);
    } else {
        print_migration_report(&report);
    }

    if matches!(report.verb, MigrationVerb::Verify) && !report.ok() {
        std::process::exit(1);
    }
    if hard_purge_attempt && report.summary.refused > 0 {
        std::process::exit(2);
    }
    Ok(())
}

fn print_migration_report(report: &MigrationReport) {
    let verb = migration_verb_label(report.verb);
    let status = if report.ok() { "ok" } else { "needs-work" };
    emit(format!(
        "\x1b[1;36mmigrate {verb}: {status}\x1b[0m  {} items ({} current, {} needs migration, {} missing canonical, {} protected, {} refused)",
        report.summary.total,
        report.summary.current,
        report.summary.needs_migration,
        report.summary.missing_canonical,
        report.summary.protected,
        report.summary.refused,
    ));
    emit(format!("  meta_root: {}", report.meta_root));
    emit(format!("  manifest:  {}", report.manifest_dir));
    emit(format!("  ledger:    {}", report.ledger_path));
    if let Some(risk) = report.summary.highest_risk {
        emit(format!("  highest risk: {}", migration_risk_label(risk)));
    }

    let mut shown = 0usize;
    for item in &report.items {
        if item.status == MigrationStatus::Current && out().verbose == 0 {
            continue;
        }
        shown += 1;
        let marker = migration_marker(item.status, item.protected);
        let action = migration_action_label(item.action);
        let mut line = format!(
            "  {marker} {:<22} {:<18} {}",
            migration_status_label(item.status),
            action,
            item.subject
        );
        if let Some(component) = item.component.as_deref() {
            line.push_str(&format!(" [{component}]"));
        }
        emit(line);
        if out().verbose >= 1 {
            emit(format!("      {}", item.detail));
            if let Some(source) = item.source.as_deref() {
                emit(format!("      source: {source}"));
            }
            if let Some(canonical) = item.canonical.as_deref() {
                emit(format!("      canonical: {canonical}"));
            }
            if let Some(legacy) = item.legacy.as_deref() {
                emit(format!("      legacy: {legacy}"));
            }
        }
    }
    if shown == 0 && !report.items.is_empty() {
        emit("  all displayed items are current; use -v to show the full inventory".to_string());
    }
}

fn migration_verb_label(verb: MigrationVerb) -> &'static str {
    match verb {
        MigrationVerb::Scan => "scan",
        MigrationVerb::Plan => "plan",
        MigrationVerb::Apply => "apply",
        MigrationVerb::Verify => "verify",
        MigrationVerb::Purge => "purge",
    }
}

fn migration_status_label(status: MigrationStatus) -> &'static str {
    match status {
        MigrationStatus::Current => "current",
        MigrationStatus::NeedsMigration => "needs-migration",
        MigrationStatus::MissingCanonical => "missing-canonical",
        MigrationStatus::Materialized => "materialized",
        MigrationStatus::Preserved => "preserved",
        MigrationStatus::Protected => "protected",
        MigrationStatus::ReportOnly => "report-only",
        MigrationStatus::Refused => "refused",
    }
}

fn migration_action_label(action: MigrationAction) -> &'static str {
    match action {
        MigrationAction::None => "none",
        MigrationAction::MaterializeCanonicalDir => "materialize-dir",
        MigrationAction::UpdateManifestToCanonicalLayout => "update-manifest",
        MigrationAction::AdoptIntoMetaLocal => "adopt-meta-local",
        MigrationAction::PreserveConfig => "preserve-config",
        MigrationAction::ProtectSubstrate => "protect-substrate",
        MigrationAction::ReportOnly => "report-only",
        MigrationAction::RefusePurge => "refuse-purge",
    }
}

fn migration_risk_label(risk: MigrationRisk) -> &'static str {
    match risk {
        MigrationRisk::Low => "low",
        MigrationRisk::Medium => "medium",
        MigrationRisk::High => "high",
    }
}

fn migration_marker(status: MigrationStatus, protected: bool) -> &'static str {
    if protected {
        "🔒"
    } else {
        match status {
            MigrationStatus::Current
            | MigrationStatus::Materialized
            | MigrationStatus::Preserved => "✓",
            MigrationStatus::NeedsMigration
            | MigrationStatus::MissingCanonical
            | MigrationStatus::ReportOnly => "·",
            MigrationStatus::Protected => "🔒",
            MigrationStatus::Refused => "⛔",
        }
    }
}

fn run_catalog(
    engine: &Engine,
    roots: CatalogRootArgs,
    cmd: CatalogCmd,
    json: bool,
) -> anyhow::Result<()> {
    let explicit_roots = roots.repo_root.is_some() || roots.manifest_dir.is_some();
    let (spec, registry) = if explicit_roots {
        resolve_catalog_scan_spec(roots)?
    } else {
        (
            CatalogScanSpec {
                repo_root: engine
                    .manifest_dir()
                    .parent()
                    .map(PathBuf::from)
                    .unwrap_or_else(|| {
                        std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
                    }),
                manifest_dir: engine.manifest_dir().to_path_buf(),
            },
            None,
        )
    };
    let registry = registry.as_ref().unwrap_or_else(|| engine.registry());
    match cmd {
        CatalogCmd::Scan => {
            let snapshot = if explicit_roots {
                catalog_engine::scan(spec, registry)?
            } else {
                engine.catalog_scan()?
            };
            if json {
                println!("{}", serde_json::to_string_pretty(&snapshot)?);
            } else {
                print_catalog_summary(&snapshot);
            }
        }
        CatalogCmd::Tables => {
            let snapshot = if explicit_roots {
                catalog_engine::scan(spec, registry)?
            } else {
                engine.catalog_scan()?
            };
            let report = catalog_engine::table_inventory(&snapshot);
            if json {
                println!("{}", serde_json::to_string_pretty(&report)?);
            } else {
                print_catalog_tables(&report);
            }
        }
        CatalogCmd::Table { name } => {
            let snapshot = if explicit_roots {
                catalog_engine::scan(spec, registry)?
            } else {
                engine.catalog_scan()?
            };
            let table = name
                .parse::<CatalogTableName>()
                .map_err(|err| anyhow::anyhow!(err))?;
            let table_value = snapshot.table_value(table);
            if json {
                println!("{}", serde_json::to_string_pretty(&table_value)?);
            } else {
                print_catalog_table(table, &table_value)?;
            }
        }
        CatalogCmd::Import => {
            let report = if explicit_roots {
                catalog_engine::import_current(spec, registry)?
            } else {
                engine.catalog_import()?
            };
            if json {
                println!("{}", serde_json::to_string_pretty(&report)?);
            } else {
                print_catalog_import(&report);
            }
        }
        CatalogCmd::Analyze => {
            let report = if explicit_roots {
                catalog_engine::analyze_current(spec, registry)?
            } else {
                engine.catalog_analyze()?
            };
            if json {
                println!("{}", serde_json::to_string_pretty(&report)?);
            } else {
                print_catalog_analyze(&report);
            }
        }
        CatalogCmd::Diff => {
            let report = if explicit_roots {
                catalog_engine::diff(spec, registry)?
            } else {
                engine.catalog_diff()?
            };
            if json {
                println!("{}", serde_json::to_string_pretty(&report)?);
            } else {
                print_catalog_diff(&report);
            }
        }
        CatalogCmd::Render { out, target_root } => {
            let report = if explicit_roots {
                catalog_engine::render(
                    catalog_engine::CatalogRenderSpec {
                        repo_root: spec.repo_root,
                        manifest_dir: spec.manifest_dir,
                        out_dir: out,
                        target_root,
                    },
                    registry,
                )?
            } else {
                engine.catalog_render(&out, target_root.as_deref())?
            };
            if json {
                println!("{}", serde_json::to_string_pretty(&report)?);
            } else {
                print_catalog_render(&report);
            }
        }
        CatalogCmd::Sync { render_out, apply } => {
            let report = if explicit_roots {
                catalog_engine::sync(
                    catalog_engine::CatalogSyncSpec {
                        repo_root: spec.repo_root,
                        manifest_dir: spec.manifest_dir,
                        render_out_dir: render_out,
                        apply,
                    },
                    registry,
                )?
            } else {
                engine.catalog_sync(render_out.as_deref(), apply)?
            };
            if json {
                println!("{}", serde_json::to_string_pretty(&report)?);
            } else {
                print_catalog_sync(&report);
            }
        }
        CatalogCmd::Lock { check, apply } => {
            if check && apply {
                anyhow::bail!("catalog lock accepts either --check or --apply, not both");
            }
            let report = if explicit_roots {
                catalog_engine::lock(
                    catalog_engine::CatalogLockSpec {
                        repo_root: spec.repo_root,
                        manifest_dir: spec.manifest_dir,
                        apply,
                    },
                    registry,
                )?
            } else {
                engine.catalog_lock(apply)?
            };
            if json {
                println!("{}", serde_json::to_string_pretty(&report)?);
            } else {
                print_catalog_lock(&report);
            }
            if check && report.summary.before_drifts > 0 {
                std::process::exit(1);
            }
        }
    }
    Ok(())
}

fn resolve_catalog_scan_spec(
    roots: CatalogRootArgs,
) -> anyhow::Result<(CatalogScanSpec, Option<Registry>)> {
    let repo_root_arg = roots.repo_root;
    let manifest_dir_arg = roots.manifest_dir;
    let repo_root_was_explicit = repo_root_arg.is_some();
    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let repo_root = repo_root_arg.unwrap_or_else(|| {
        manifest_dir_arg
            .as_ref()
            .map(|manifest_dir| {
                let manifest_dir = if manifest_dir.is_absolute() {
                    manifest_dir.clone()
                } else {
                    cwd.join(manifest_dir)
                };
                manifest_dir
                    .parent()
                    .map(PathBuf::from)
                    .unwrap_or_else(|| cwd.clone())
            })
            .unwrap_or_else(|| cwd.clone())
    });
    let repo_root = std::fs::canonicalize(&repo_root).map_err(|err| {
        anyhow::anyhow!(
            "catalog repo root not found or unreadable: {}: {err}",
            repo_root.display()
        )
    })?;
    let manifest_dir = match manifest_dir_arg {
        Some(manifest_dir) if manifest_dir.is_absolute() => manifest_dir,
        Some(manifest_dir) if repo_root_was_explicit => repo_root.join(manifest_dir),
        Some(manifest_dir) => cwd.join(manifest_dir),
        None => repo_root.join("manifest"),
    };
    let registry = if manifest_dir.is_dir() {
        Some(Registry::load(&manifest_dir)?)
    } else {
        None
    };
    Ok((
        CatalogScanSpec {
            repo_root,
            manifest_dir,
        },
        registry,
    ))
}

fn print_catalog_summary(snapshot: &CatalogSnapshot) {
    println!("envctl catalog scan (read-only)");
    println!("repo_root: {}", snapshot.repo_root);
    println!("manifest_dir: {}", snapshot.manifest_dir);
    println!("tables:");
    for table in CatalogTableName::all() {
        println!(
            "  {:<20} {}",
            table.canonical_name(),
            snapshot.table_count(*table)
        );
    }
}

fn print_catalog_tables(rows: &[CatalogTableSummaryRow]) {
    println!("envctl catalog tables (read-only)");
    println!("tables: {}", rows.len());
    if rows.is_empty() {
        return;
    }
    println!("\ntable\trows\tcolumns\tpurpose");
    for row in rows {
        println!(
            "{}\t{}\t{}\t{}",
            row.table,
            row.rows,
            row.columns.join(","),
            row.purpose
        );
    }
}

fn print_catalog_table(table: CatalogTableName, value: &serde_json::Value) -> anyhow::Result<()> {
    let rows = value
        .as_array()
        .ok_or_else(|| anyhow::anyhow!("catalog table {table} did not serialize as an array"))?;
    println!("catalog table: {table}");
    println!("rows: {}", rows.len());
    if rows.is_empty() {
        return Ok(());
    }

    let mut columns = BTreeSet::new();
    for row in rows {
        if let Some(object) = row.as_object() {
            columns.extend(object.keys().cloned());
        }
    }
    let columns: Vec<_> = columns.into_iter().collect();
    println!("{}", columns.join("\t"));
    for row in rows {
        let object = row
            .as_object()
            .ok_or_else(|| anyhow::anyhow!("catalog table {table} contained a non-object row"))?;
        let cells = columns
            .iter()
            .map(|column| object.get(column).map(catalog_cell).unwrap_or_default())
            .collect::<Vec<_>>();
        println!("{}", cells.join("\t"));
    }
    Ok(())
}

fn print_catalog_import(report: &CatalogImportReport) {
    println!("envctl catalog import (read-only)");
    println!("repo_root: {}", report.repo_root);
    println!("manifest_dir: {}", report.manifest_dir);
    println!("mutating: {}", yes_no(report.summary.mutating));
    println!("tables: {}", report.summary.tables);
    println!("rows: {}", report.summary.rows);
    println!("components: {}", report.summary.components);
    println!("config_files: {}", report.summary.config_files);
    println!("settings: {}", report.summary.settings);
    println!("env_vars: {}", report.summary.env_vars);
}

fn print_catalog_analyze(report: &CatalogAnalyzeReport) {
    println!("envctl catalog analyze (read-only)");
    println!("repo_root: {}", report.repo_root);
    println!("manifest_dir: {}", report.manifest_dir);
    println!("mutating: {}", yes_no(report.summary.mutating));
    println!("tables: {}", report.summary.tables);
    println!("rows: {}", report.summary.rows);
    println!("config_files: {}", report.summary.config_files);
    println!("env_vars: {}", report.summary.env_vars);
    println!("codedb_imports: {}", report.summary.codedb_imports);
    println!("toolchain_signals: {}", report.summary.toolchain_signals);

    print_catalog_inventory("table inventory", &report.table_inventory);
    print_catalog_count_facets("config formats", &report.config_formats);
    print_catalog_count_facets("config file kinds", &report.config_file_kinds);
    print_catalog_count_facets("env scopes", &report.env_scopes);
    print_catalog_count_facets("env producers", &report.env_producers);
    print_catalog_count_facets("env sensitivity", &report.env_sensitive);
    print_catalog_count_facets("path artifact kinds", &report.path_artifact_kinds);
    print_catalog_count_facets(
        "path verification statuses",
        &report.path_verification_statuses,
    );
    print_catalog_count_facets("codedb file kinds", &report.codedb_file_kinds);
    print_catalog_count_facets("codedb parser hints", &report.codedb_parser_hints);

    println!("\ntoolchain signals:");
    if report.toolchain_signals.is_empty() {
        println!("  none");
    } else {
        for row in &report.toolchain_signals {
            println!(
                "  {:<12} {:<28} {:<18} {:<24} {}",
                row.signal_kind, row.key, row.source, row.detail, row.value
            );
        }
    }
}

fn print_catalog_diff(report: &CatalogDiffReport) {
    println!("envctl catalog diff (read-only)");
    println!("repo_root: {}", report.repo_root);
    println!("manifest_dir: {}", report.manifest_dir);
    println!("mutating: {}", yes_no(report.summary.mutating));
    println!("config_files: {}", report.summary.config_files);
    println!("components: {}", report.summary.components);
    println!("drift_count: {}", report.summary.drift_count);
    println!("lock_drifts: {}", report.summary.lock_drifts);
    println!("parse_errors: {}", report.summary.parse_errors);
    println!("read_errors: {}", report.summary.read_errors);
    println!("missing_files: {}", report.summary.missing_files);
    if report.drift.is_empty() {
        println!("\nno catalog drift found");
        return;
    }

    println!("\ndrift:");
    for row in &report.drift {
        println!(
            "  {:<18} {:<14} {:<32} {}",
            row.severity, row.drift_kind, row.subject_id, row.details
        );
    }
}

fn print_catalog_count_facets(label: &str, rows: &[CatalogFacetCount]) {
    println!("\n{label}:");
    if rows.is_empty() {
        println!("  none");
        return;
    }
    for row in rows {
        println!("  {:<28} {}", row.key, row.count);
    }
}

fn print_catalog_inventory(label: &str, rows: &[CatalogTableSummaryRow]) {
    println!("\n{label}:");
    if rows.is_empty() {
        println!("  none");
        return;
    }
    for row in rows {
        println!(
            "  {:<24} {:>6} rows  {:<48} {}",
            row.table,
            row.rows,
            row.columns.join(","),
            row.purpose
        );
    }
}

fn print_catalog_render(report: &CatalogRenderReport) {
    println!("envctl catalog render (no repo mutation)");
    println!("repo_root: {}", report.repo_root);
    println!("manifest_dir: {}", report.manifest_dir);
    println!("out_dir: {}", report.out_dir);
    if let Some(target_root) = &report.target_root {
        println!("target_root: {}", target_root);
    }
    println!("mutating_repo: {}", yes_no(report.summary.mutating_repo));
    println!("generated_files: {}", report.summary.generated_files);
    println!(
        "generated_config_rows: {}",
        report.summary.generated_config_rows
    );
    println!("bytes: {}", report.summary.bytes);
    println!("\nfiles:");
    for file in &report.files {
        println!(
            "  {:<52} {:>8} bytes  {}",
            file.path, file.bytes, file.sha256
        );
    }
}

fn print_catalog_sync(report: &CatalogSyncReport) {
    println!("envctl catalog sync (preview)");
    println!("repo_root: {}", report.repo_root);
    println!("manifest_dir: {}", report.manifest_dir);
    println!("mutating: {}", yes_no(report.summary.mutating));
    println!("applied: {}", yes_no(report.summary.applied));
    println!("verifier_status: {}", report.summary.verifier_status);
    println!("drift_count: {}", report.summary.drift_count);
    println!("planned_actions: {}", report.summary.planned_actions);
    println!("rendered_files: {}", report.summary.rendered_files);
    if report.planned_actions.is_empty() {
        println!("\nno sync actions planned");
    } else {
        println!("\nactions:");
        for action in &report.planned_actions {
            println!(
                "  {:<12} {:<24} {:<18} {}",
                action.action_id, action.action_kind, action.subject_id, action.reason
            );
        }
    }
    if let Some(render) = &report.render {
        println!("\nrendered_projection: {}", render.out_dir);
        println!("rendered_files: {}", render.summary.generated_files);
    }
}

fn print_catalog_lock(report: &CatalogLockReport) {
    println!("envctl catalog lock");
    println!("lock_path: {}", report.lock_path);
    println!("mutating: {}", yes_no(report.summary.mutating));
    println!("components: {}", report.summary.components);
    println!("before_drifts: {}", report.summary.before_drifts);
    println!("after_drifts: {}", report.summary.after_drifts);
    println!("lock_written: {}", yes_no(report.summary.lock_written));
    if report.drift.is_empty() {
        println!("\nlock projection is current");
    } else {
        println!("\nlock drift:");
        for row in &report.drift {
            println!(
                "  {:<14} {:<32} {}",
                row.drift_kind, row.subject_id, row.details
            );
        }
    }
}

fn yes_no(value: bool) -> &'static str {
    if value {
        "yes"
    } else {
        "no"
    }
}

fn catalog_cell(value: &serde_json::Value) -> String {
    match value {
        serde_json::Value::Null => String::new(),
        serde_json::Value::Bool(value) => value.to_string(),
        serde_json::Value::Number(value) => value.to_string(),
        serde_json::Value::String(value) => value.clone(),
        serde_json::Value::Array(values) => {
            if values.iter().all(|value| value.is_string()) {
                values
                    .iter()
                    .filter_map(|value| value.as_str())
                    .collect::<Vec<_>>()
                    .join(",")
            } else {
                serde_json::to_string(values).unwrap_or_default()
            }
        }
        serde_json::Value::Object(_) => serde_json::to_string(value).unwrap_or_default(),
    }
}

fn print_hub_registry(report: &HubRegistryReport) {
    if report.sources.is_empty() {
        println!("hub registry: no *_hub/registry.json files found");
        return;
    }
    let status = if report.clean() { "clean" } else { "drift" };
    println!(
        "hub registry: {} sources, {} entries, {}",
        report.sources.len(),
        report.entries.len(),
        status
    );
    for source in &report.sources {
        println!("  {} ({})", source.hub, source.path);
        for entry in report
            .entries
            .iter()
            .filter(|entry| entry.hub == source.hub)
        {
            println!(
                "    {} -> {} [{}; tier {}]",
                entry.entry.id,
                entry.entry.component,
                match entry.entry.status {
                    HubRegistryStatus::Stable => "stable",
                    HubRegistryStatus::Experimental => "experimental",
                },
                entry.entry.tier
            );
        }
    }
    if !report.drift.is_empty() {
        println!("drift:");
        for item in &report.drift {
            println!("  {} / {}: {}", item.hub, item.id, item.detail);
        }
    }
}

/// Suppress the end-of-run update notice for machine-readable / scripted output and for
/// commands that already print version info (kasetto `should_suppress_notice`). Never suppress
/// for the human install/reset/auto-fix runs.
fn should_suppress_notice(cmd: &Cmd, json: bool, quiet: u8) -> bool {
    if json || quiet > 0 {
        return true;
    }
    match cmd {
        // Machine-readable / version-printing verbs.
        Cmd::Completions { .. }
        | Cmd::Manage { .. }
        | Cmd::Env { .. }
        | Cmd::Migrate { .. }
        | Cmd::Registry { .. }
        | Cmd::Catalog { .. } => true,
        // `auto-detect`/`graph`/`lock`/`agent ... --json` are gated by the global `json` above;
        // their human forms may still show the notice. Everything else: don't suppress.
        _ => false,
    }
}

/// Render the end-of-run notice. Honors the resolved plain flag.
fn render_update_notice(current: &str, latest: &str) -> String {
    let cmd = upgrade_command();
    let line =
        format!("\n\x1b[1;33mNew version available:\x1b[0m {current} -> {latest}  run `{cmd}`");
    paint(line)
}

/// Best-guess upgrade command from the running binary's install path (kasetto
/// `upgrade_command`): cargo-installed → `cargo install envctl`; otherwise the self-updater
/// (`envctl self update`). The brew arm is inert for envctl (no homebrew formula).
fn upgrade_command() -> &'static str {
    let exe = std::env::current_exe().ok();
    let path = exe
        .as_ref()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_default();
    if path.contains("/.cargo/bin") || path.contains("/cargo/bin") {
        "cargo install envctl"
    } else {
        "envctl self update"
    }
}

/// `envctl completions <shell>` — generate the completion script for envctl's OWN clap tree to
/// stdout. CLI-only (clap-tree introspection; no engine logic / GUI analog).
fn run_completions(shell: Shell) -> anyhow::Result<()> {
    let mut cmd = Cli::command();
    clap_complete::generate(shell, &mut cmd, "envctl", &mut std::io::stdout());
    Ok(())
}

/// `envctl self uninstall` — DESTRUCTIVE. Dry-run by default; `--apply` deletes. On a TTY, prompt
/// `[y/N]` unless `--yes`; in non-interactive mode `--apply` requires `--yes`. The engine owns the
/// fail-closed removal + binary-stem guard; this owns the confirmation + rendering.
fn run_self_uninstall(engine: Engine, apply: bool, yes: bool, json: bool) -> anyhow::Result<()> {
    if apply && !yes {
        if !std::io::stdin().is_terminal() {
            return Err(anyhow::anyhow!(
                "pass --yes to confirm uninstall in non-interactive mode"
            ));
        }
        use std::io::Write;
        println!("This will remove envctl, envctl-gui, and all installed assets.");
        print!("Uninstall envctl? [y/N] ");
        std::io::stdout().flush()?;
        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;
        if !matches!(input.trim(), "y" | "Y" | "yes") {
            println!("Cancelled.");
            return Ok(());
        }
    }

    let (sink, _rx) = EventSink::channel();
    let outcome = engine.self_uninstall(SelfUninstallSpec { apply, yes }, &sink)?;

    if json {
        println!("{}", serde_json::to_string_pretty(&outcome)?);
        return Ok(());
    }

    let verb = if outcome.dry_run {
        "would remove"
    } else {
        "removed"
    };
    if outcome.skills_removed > 0 {
        println!("✓ {verb} {} skills", outcome.skills_removed);
    }
    if outcome.mcps_removed > 0 {
        println!("✓ {verb} {} MCP servers", outcome.mcps_removed);
    }
    if outcome.command_dirs_unlinked > 0 {
        println!(
            "✓ {verb} {} command directories",
            outcome.command_dirs_unlinked
        );
    }
    if outcome.dry_run {
        println!("✓ would remove config / data / cache directories");
        println!("✓ would remove the envctl binary");
    } else {
        if outcome.config_removed || outcome.data_removed || outcome.cache_removed {
            println!("✓ removed config / data / cache directories");
        }
        if outcome.binary_removed || outcome.gui_removed {
            println!("✓ removed the envctl binary");
        }
    }
    if let Some(reason) = &outcome.refused {
        println!("\x1b[1;31m  ⛔ {reason}\x1b[0m");
    }
    if outcome.dry_run {
        println!("· dry-run: pass --apply (with --yes or a [y/N] confirm) to delete");
    }
    Ok(())
}

/// Resolve the meta workspace root from the `.meta.yaml` marker and print env
/// exports. Read-only: the engine does the (non-printing) marker walk via
/// `locate_meta_file`; the CLI owns the output. `eval "$(envctl env)"` makes
/// `META_ROOT`/`META_FILE` available so configs never hardcode the meta path —
/// the portability seam (ADR-0006). Honors `$META_FILE` / `--meta-file` override.
fn run_env(
    meta_file: Option<std::path::PathBuf>,
    toolchains: bool,
    materialize: Option<std::path::PathBuf>,
    json: bool,
) -> anyhow::Result<()> {
    let cwd = std::env::current_dir()?;
    let meta_yaml = envctl_engine::dashboard::locate_meta_file(&cwd, meta_file.as_deref())?;
    let meta_root = meta_yaml.parent().ok_or_else(|| {
        anyhow::anyhow!("`.meta.yaml` at {} has no parent dir", meta_yaml.display())
    })?;
    let root = meta_root.to_string_lossy();
    let layout = envctl_engine::MetaLayout::from_meta_root(meta_root.to_path_buf());

    // --materialize: render `${META_ROOT}`/`$META_ROOT` -> absolute root in FILE.
    // Read-only (stdout). Heals configs Claude reads literally (marketplace paths).
    if let Some(file) = materialize {
        let content = std::fs::read_to_string(&file)
            .map_err(|e| anyhow::anyhow!("reading {} to materialize: {e}", file.display()))?;
        print!("{}", render_meta_root(&content, &root));
        return Ok(());
    }

    // The meta-hosted install layout (opt-in via --toolchains for now because
    // this is the shell seam that mutates PATH). `usr/bin` is canonical for
    // envctl-owned exposure; `.local/bin` and `.toolchains` remain compatibility surfaces.
    let tc = layout.legacy_toolchains().to_string_lossy().to_string();
    let ollama_models = layout.ollama_models().to_string_lossy().to_string();
    let usr = layout.usr().to_string_lossy().to_string();
    let bin_dir = layout.bin().to_string_lossy().to_string();
    let compat_local_bin = layout.local_bin().to_string_lossy().to_string();
    if json {
        let mut map = serde_json::json!({ "META_ROOT": meta_root, "META_FILE": meta_yaml });
        if toolchains {
            for (key, path) in layout.env_exports() {
                map[key] = path.to_string_lossy().to_string().into();
            }
            map["BUN_INSTALL"] = format!("{tc}/.bun").into();
            map["MISE_DATA_DIR"] = format!("{tc}/mise").into();
            map["CARGO_HOME"] = format!("{tc}/cargo").into();
            map["RUSTUP_HOME"] = format!("{tc}/rustup").into();
            map["XDG_CACHE_HOME"] = layout.xdg_cache_home().to_string_lossy().to_string().into();
            map["UV_TOOL_DIR"] = format!("{tc}/uv/tools").into();
            map["UV_PYTHON_INSTALL_DIR"] = format!("{tc}/uv/python").into();
            map["OLLAMA_LIBRARY_PATH"] = format!("{tc}/ollama/lib/ollama").into();
            map["OLLAMA_MODELS"] = ollama_models.clone().into();
            map["LIBCLANG_PATH"] = format!("{tc}/llvm/lib").into();
            map["GCC_PATH"] = format!("{tc}/libgccjit/lib").into();
            map["HELIX_RUNTIME"] = format!("{tc}/helix/runtime").into();
        }
        println!("{}", serde_json::to_string_pretty(&map)?);
        return Ok(());
    }

    println!("export META_ROOT={}", sh_single_quote(&root));
    println!(
        "export META_FILE={}",
        sh_single_quote(&meta_yaml.to_string_lossy())
    );
    if toolchains {
        for (key, path) in layout.env_exports() {
            println!("export {key}={}", sh_single_quote(&path.to_string_lossy()));
        }
        // Redirect each manager's install prefix INTO meta (ADR: meta-located
        // toolchain prefix). Canonical exposure starts at `usr/bin`; the meta-home
        // `.local/bin` bridge and legacy manager bins trail it for compatibility. PATH uses double quotes so
        // `$PATH` expands.
        println!(
            "export BUN_INSTALL={}",
            sh_single_quote(&format!("{tc}/.bun"))
        );
        println!(
            "export MISE_DATA_DIR={}",
            sh_single_quote(&format!("{tc}/mise"))
        );
        println!(
            "export CARGO_HOME={}",
            sh_single_quote(&format!("{tc}/cargo"))
        );
        // RUSTUP_HOME must travel with CARGO_HOME: the rustup toolchain store is
        // meta-owned at .toolchains/rustup (set in the `rustup` component's install
        // hooks), but without exporting it here, `eval "$(envctl env --toolchains)"`
        // shells fall back to ~/.rustup and miss the meta-owned nightly/codegen-gcc
        // toolchain. Pairs CARGO_HOME ↔ RUSTUP_HOME so the shell seam matches the
        // component hooks. (ADR-0013: compiler resolves under $META_ROOT/.toolchains/rustup.)
        println!(
            "export RUSTUP_HOME={}",
            sh_single_quote(&format!("{tc}/rustup"))
        );
        // Cache-heavy toolchain frontdoors (notably kache as Cargo's rustc-wrapper)
        // must default to meta-owned XDG cache instead of leaking active state into
        // ~/.cache when a shell has only sourced `envctl env --toolchains`.
        println!(
            "export XDG_CACHE_HOME={}",
            sh_single_quote(&layout.xdg_cache_home().to_string_lossy())
        );
        println!(
            "export UV_TOOL_DIR={}",
            sh_single_quote(&format!("{tc}/uv/tools"))
        );
        println!(
            "export UV_PYTHON_INSTALL_DIR={}",
            sh_single_quote(&format!("{tc}/uv/python"))
        );
        // GPU runner .so redirect for the meta-owned ollama (.toolchains/ollama/lib/
        // ollama holds the cuda_v12/cuda_v13 ggml runners). The binary also resolves
        // ../lib/ollama from its real path, so this is belt-and-suspenders.
        println!(
            "export OLLAMA_LIBRARY_PATH={}",
            sh_single_quote(&format!("{tc}/ollama/lib/ollama"))
        );
        // Model blobs are persistent state, not runner binaries. Keep them under
        // meta's canonical var/lib tree so pulls never fall back to the root
        // daemon's /usr/share/ollama or a real-home ~/.ollama store (TASK-0072).
        println!("export OLLAMA_MODELS={}", sh_single_quote(&ollama_models));
        // libclang.so redirect for the meta-owned LLVM/clang (.toolchains/llvm/lib
        // holds libclang.so) so bindgen-style consumers find it (Epic H TASK-0061).
        println!(
            "export LIBCLANG_PATH={}",
            sh_single_quote(&format!("{tc}/llvm/lib"))
        );
        // libgccjit.so dir for rustc_codegen_gcc (config.toml `gcc-path` /
        // LIBRARY_PATH+LD_LIBRARY_PATH consume it) — Epic H TASK-0062.
        println!(
            "export GCC_PATH={}",
            sh_single_quote(&format!("{tc}/libgccjit/lib"))
        );
        // helix tree-sitter runtime (grammars + queries) for the meta-owned hx
        // (.toolchains/helix/runtime, bundled in the upstream release tarball). hx also finds
        // runtime/ as a sibling of its resolved exe, so this is belt-and-suspenders (Epic H).
        println!(
            "export HELIX_RUNTIME={}",
            sh_single_quote(&format!("{tc}/helix/runtime"))
        );
        println!("export PATH=\"{bin_dir}:{usr}/sbin:{usr}/local/bin:{usr}/local/sbin:{compat_local_bin}:{tc}/.bun/bin:{tc}/cargo/bin:{tc}/uv/tools/bin:$PATH\"");
        // The rest of the meta /usr mirror on its respective search paths. Each is
        // prepend-with-fallback so an inherited value (e.g. the CUDA LD_LIBRARY_PATH
        // shell-rc block) is preserved, never clobbered. The skeleton starts empty,
        // so no system binary/lib/header is shadowed until meta installs into it.
        println!(
            "export LD_LIBRARY_PATH=\"{usr}/lib:{usr}/lib64:{usr}/local/lib:{usr}/local/lib64:${{LD_LIBRARY_PATH:-}}\""
        );
        println!("export CPATH=\"{usr}/include:{usr}/local/include:${{CPATH:-}}\"");
        println!(
            "export PKG_CONFIG_PATH=\"{usr}/lib/pkgconfig:{usr}/share/pkgconfig:${{PKG_CONFIG_PATH:-}}\""
        );
        println!("export MANPATH=\"{usr}/share/man:{usr}/local/share/man${{MANPATH:+:$MANPATH}}\"");
    }
    Ok(())
}

/// POSIX single-quote a value so `eval "$(envctl env)"` is safe for paths with
/// spaces or shell metacharacters. Closes the quote, emits an escaped `'`, reopens.
fn sh_single_quote(s: &str) -> String {
    format!("'{}'", s.replace('\'', "'\\''"))
}

/// Substitute `${META_ROOT}` and `$META_ROOT` tokens with the resolved absolute
/// root. Used by `envctl env --materialize` to render configs that a consumer
/// reads literally (no shell/Claude expansion), e.g. plugin marketplace paths.
fn render_meta_root(content: &str, root: &str) -> String {
    content
        .replace("${META_ROOT}", root)
        .replace("$META_ROOT", root)
}

#[cfg(test)]
mod env_cmd_tests {
    use super::sh_single_quote;

    #[test]
    fn render_meta_root_substitutes_both_forms() {
        let r = "/home/d/Desktop/meta";
        assert_eq!(
            super::render_meta_root("path = \"${META_ROOT}/claude-plugins\"", r),
            "path = \"/home/d/Desktop/meta/claude-plugins\""
        );
        assert_eq!(
            super::render_meta_root("bash $META_ROOT/.claude/x.sh", r),
            "bash /home/d/Desktop/meta/.claude/x.sh"
        );
        // no token -> unchanged
        assert_eq!(super::render_meta_root("nothing here", r), "nothing here");
    }

    #[test]
    fn sh_single_quote_wraps_and_escapes() {
        assert_eq!(
            sh_single_quote("/home/d/Desktop/meta"),
            "'/home/d/Desktop/meta'"
        );
        assert_eq!(sh_single_quote("/path with space"), "'/path with space'");
        // embedded single quote: close, escaped quote, reopen
        assert_eq!(sh_single_quote("a'b"), "'a'\\''b'");
    }

    /// Drift guard (TASK-0004 / TASK-0005): the live `home/.claude/settings.json` MUST
    /// be exactly `settings.json.tmpl` rendered with `${META_ROOT}` -> this machine's
    /// meta root — the `claude-global-links` component's `sed` render. Editing
    /// settings.json directly, or changing the tmpl without re-rendering, breaks this.
    /// Also pins the TASK-0004 env block that wires META_ROOT into the env Claude inherits.
    #[test]
    fn settings_json_matches_rendered_tmpl_no_drift() {
        let base = concat!(env!("CARGO_MANIFEST_DIR"), "/../../home/.claude/");
        let tmpl = std::fs::read_to_string(format!("{base}settings.json.tmpl"))
            .expect("read settings.json.tmpl");
        let live =
            std::fs::read_to_string(format!("{base}settings.json")).expect("read settings.json");

        // Derive this machine's META_ROOT from the rendered statusline anchor so the
        // test is host-independent (CI vs dev box have different absolute roots).
        const SUFFIX: &str = "/.claude/statusline-command.sh\"";
        let anchor = live
            .lines()
            .find(|l| l.contains(SUFFIX))
            .expect("settings.json has a statusline command line");
        let root = anchor
            .rsplit_once("bash ")
            .and_then(|(_, rest)| rest.strip_suffix(SUFFIX))
            .expect("statusline anchor shape: \"bash <META_ROOT>/.claude/statusline-command.sh\"");

        assert_eq!(
            super::render_meta_root(&tmpl, root),
            live,
            "settings.json drifted from settings.json.tmpl — re-render it \
             (sed 's|${{META_ROOT}}|<root>|g' settings.json.tmpl > settings.json)"
        );
        assert!(
            !live.contains("${META_ROOT}") && !live.contains("$META_ROOT"),
            "settings.json still has an unrendered META_ROOT token"
        );
        // TASK-0004: env block wires META_ROOT (+ META_FILE) into the env Claude inherits.
        assert!(
            tmpl.contains("\"META_ROOT\": \"${META_ROOT}\""),
            "settings.json.tmpl must export META_ROOT via the env block (TASK-0004)"
        );
        assert!(
            live.contains(&format!("\"META_ROOT\": \"{root}\"")),
            "rendered settings.json must carry the absolute META_ROOT in its env block"
        );
    }
}

#[cfg(test)]
mod agent_cmd_tests {
    use super::{AgentScope, ScopeArg};

    // Note: `lock_mode_from` moved to the engine as `AgentLockMode::from_flags` (TASK-0014b
    // open-Q1) so the CLI and GUI share one source; its unit test moved with it
    // (`crates/engine/src/agent/mod.rs::tests::from_flags_maps_each_flag`).

    #[test]
    fn scope_arg_converts_to_agent_scope() {
        assert_eq!(AgentScope::from(ScopeArg::Global), AgentScope::Global);
        assert_eq!(AgentScope::from(ScopeArg::Project), AgentScope::Project);
    }
}

/// Mutating verbs: run on a worker thread, drain+print events on the main thread
/// (the same shape the GUI uses), exit nonzero iff something failed/was refused.
fn run_action(engine: Engine, cmd: Cmd, json: bool) -> anyhow::Result<()> {
    let (sink, rx) = EventSink::channel();
    let eng = engine.clone();
    let handle = std::thread::spawn(move || -> anyhow::Result<bool> {
        let ok = match cmd {
            Cmd::Install { targets, dry_run } => eng
                .run(RunPlan::new(Phase::Install, targets, dry_run), &sink)?
                .ok(),
            Cmd::Reset {
                targets,
                apply,
                all,
                confirm,
                cascade,
                keep_config,
                purge,
            } => eng
                .run(
                    RunPlan::new(Phase::Remove, targets, !apply).with_gates(ResetGates {
                        all,
                        confirm,
                        cascade,
                        keep_config,
                        purge,
                    }),
                    &sink,
                )?
                .ok(),
            Cmd::AutoFix {
                targets,
                apply,
                confirm,
            } => eng
                .run(
                    RunPlan::new(Phase::Fix, targets, !apply).with_gates(ResetGates {
                        confirm,
                        ..Default::default()
                    }),
                    &sink,
                )?
                .ok(),
            Cmd::AddRepo {
                git_url,
                id,
                local,
                git_ref,
                build_system,
                build_cmd,
                artifacts,
                mode,
                provides,
                tags,
                strategy,
                bins,
                renames,
                patch_cmd,
                ai_goal,
                ai_agent,
                ai_instruction,
                daemon,
                verify_cmd,
                build,
                force,
                recurse_submodules,
                connect: _,
                dry_run,
            } => {
                let spec = build_spec(AddRepoArgs {
                    git_url,
                    id,
                    local,
                    git_ref,
                    build_system,
                    build_cmd,
                    artifacts,
                    mode,
                    provides,
                    tags,
                    strategy,
                    bins,
                    renames,
                    patch_cmd,
                    ai_goal,
                    ai_agent,
                    ai_instruction,
                    daemon,
                    verify_cmd,
                    build,
                    force,
                    recurse_submodules,
                })
                .map_err(|e| anyhow::anyhow!(e))?;
                eng.add_repo(spec, dry_run, &sink)?.ok()
            }
            Cmd::AutoDetect { .. }
            | Cmd::Graph { .. }
            | Cmd::Lock { .. }
            | Cmd::Doctor
            | Cmd::Dashboard { .. }
            | Cmd::Env { .. }
            | Cmd::Migrate { .. }
            | Cmd::Migration { .. }
            | Cmd::Agent { .. }
            | Cmd::Secret { .. }
            | Cmd::Registry { .. }
            | Cmd::Catalog { .. }
            | Cmd::Completions { .. }
            | Cmd::Manage { .. } => {
                unreachable!("handled in main")
            }
        };
        Ok(ok) // sink drops here -> the main-thread rx.iter() terminates cleanly
    });

    for ev in rx.iter() {
        if json {
            println!("{}", serde_json::to_string(&ev)?);
        } else {
            print_event(&ev);
        }
    }

    let ok = handle
        .join()
        .map_err(|_| anyhow::anyhow!("worker panicked"))??;
    if !ok {
        std::process::exit(1);
    }
    Ok(())
}

/// The exit decision for an agent run: which serialized return value to print under
/// `--json`, and whether to exit nonzero. The worker builds it from the engine return;
/// the main thread renders + exits. This keeps the failure→exit policy a pure decision
/// the unit tests can exercise without a live network fetch.
enum AgentResult {
    /// `sync` / `clean` — nonzero iff `report.summary.failed > 0`.
    Report(envctl_engine::AgentReport),
    /// `add` / `remove` — nonzero iff the follow-up sync had `summary.failed > 0`.
    Edit(envctl_engine::AgentEditOutcome),
    /// `lock` — nonzero iff `--check` reported non-empty drift.
    Lock(envctl_engine::AgentLockOutcome),
    /// `list` — always exit 0.
    List(envctl_engine::AgentList),
    /// `init` — always exit 0 on success (failure is an Err instead).
    Init(envctl_engine::AgentInitOutcome),
    /// `doctor` — read-only diagnostics; always exit 0.
    Doctor(envctl_engine::AgentDoctorReport),
}

impl AgentResult {
    /// Pretty-print the serialized return value (uniform across all seven agent verbs,
    /// matching `auto-detect`/`graph`/`lock`).
    fn to_json(&self) -> anyhow::Result<String> {
        Ok(match self {
            AgentResult::Report(r) => serde_json::to_string_pretty(r)?,
            AgentResult::Edit(o) => serde_json::to_string_pretty(o)?,
            AgentResult::Lock(o) => serde_json::to_string_pretty(o)?,
            AgentResult::List(l) => serde_json::to_string_pretty(l)?,
            AgentResult::Init(o) => serde_json::to_string_pretty(o)?,
            AgentResult::Doctor(d) => serde_json::to_string_pretty(d)?,
        })
    }

    /// The fail-closed exit decision (`true` = success / exit 0).
    ///
    /// Broken assets (missing/corrupt lock entries or skills the config asks for but the
    /// source cannot satisfy) count as failures at the front-end boundary, even though the
    /// engine's internal `never-prune-on-failure` guard keys off `summary.failed`. This is an
    /// upgrade over kasetto parity (which only exits nonzero on `failed`) and prevents a
    /// sync that installed nothing from silently passing CI.
    fn ok(&self) -> bool {
        let report_ok =
            |r: &envctl_engine::AgentReport| r.summary.failed == 0 && r.summary.broken == 0;
        match self {
            AgentResult::Report(r) => report_ok(r),
            AgentResult::Edit(o) => o.sync.as_ref().is_none_or(report_ok),
            AgentResult::Lock(o) => !o.check || o.drift.is_empty(),
            AgentResult::List(_) => true,
            AgentResult::Init(_) => true,
            AgentResult::Doctor(_) => true,
        }
    }

    /// Render the typed return for the HUMAN (non-`--json`) view. `Report` (sync/clean) and
    /// `Lock` already stream their full detail through the `EventSink` (the per-action tree and
    /// the lock summary), so re-printing them would duplicate. But `list` emits only a header
    /// event — its inventory lives entirely in the returned `AgentList` — and an `add`/`remove`
    /// PREVIEW has no per-item events at all, so without this both would show only a header.
    fn render_human(&self) {
        match self {
            AgentResult::List(l) => render_agent_list(l),
            AgentResult::Edit(o) => {
                let n = o.items.len();
                println!(
                    "  {} {} ({} item{})",
                    o.action,
                    o.source,
                    n,
                    if n == 1 { "" } else { "s" }
                );
                for it in &o.items {
                    println!("    {}: {}", it.section, it.target);
                }
            }
            AgentResult::Init(o) => {
                println!(
                    "  {} {}",
                    if o.overwritten {
                        "overwrote"
                    } else {
                        "created"
                    },
                    o.path
                );
            }
            AgentResult::Doctor(d) => render_agent_doctor(d),
            // Fully rendered by the EventSink stream (print_event); nothing to add.
            AgentResult::Report(_) | AgentResult::Lock(_) => {}
        }
    }
}

/// Human-readable `agent doctor` view (kasetto grouped layout: Environment / Inventory / Checks
/// / Command directories / Failures). Honors `--quiet` (quiet && !json => no-op) and `--color`.
/// The decision/data all came from `Engine::agent_doctor`; this is pure rendering.
fn render_agent_doctor(d: &envctl_engine::AgentDoctorReport) {
    use envctl_engine::agent::doctor::format_age;
    let o = out();
    // kasetto: `if quiet && !as_json { return Ok(()) }` — handled by the caller, but guard here too.
    if o.is_quiet() && !o.json {
        return;
    }

    let update_text = match d.update_check.status.as_str() {
        "update_available" => format!(
            "{} available (checked {})",
            d.update_check.latest_version.as_deref().unwrap_or("?"),
            d.update_check
                .age_seconds
                .map(format_age)
                .unwrap_or_default()
        ),
        "up_to_date" => format!(
            "up-to-date (checked {})",
            d.update_check
                .age_seconds
                .map(format_age)
                .unwrap_or_default()
        ),
        _ => "not yet checked".to_string(),
    };

    emit(format!(
        "\x1b[1;36mdoctor — envctl {} ({})\x1b[0m",
        d.version,
        if d.failures.is_empty() {
            "✓ healthy"
        } else {
            "✗ issues"
        }
    ));

    emit("\n\x1b[1;33mENVIRONMENT\x1b[0m".to_string());
    let env_rows: Vec<(&str, String)> = vec![
        ("Scope", d.scope.clone()),
        ("Lock file", d.lock_file.clone()),
        ("Install path", d.installation_path.clone()),
        (
            "Last sync",
            d.last_sync.clone().unwrap_or_else(|| "none".into()),
        ),
        ("Updates", update_text),
    ];
    let kw = env_rows.iter().map(|(k, _)| k.len()).max().unwrap_or(0);
    for (k, v) in &env_rows {
        emit(format!("  {k:<kw$}  {v}"));
    }

    emit("\n\x1b[1;33mINVENTORY\x1b[0m".to_string());
    emit(format!("  Skills       {}", d.skills.len()));
    emit(format!("  MCP servers  {}", d.mcps.len()));
    emit(format!("  Commands     {}", d.commands.len()));

    emit("\n\x1b[1;33mCHECKS\x1b[0m".to_string());
    let lock_ok = !d.lock_file.is_empty();
    emit(check_line(lock_ok, "Lock file readable"));
    let install_ok =
        std::path::Path::new(&d.installation_path).exists() || d.installation_path == "none";
    emit(check_line(install_ok, "Install path writable"));
    emit(check_line(
        d.failures.is_empty(),
        if d.failures.is_empty() {
            "No failed skills"
        } else {
            "Failed skills present"
        },
    ));
    let dirs_writable = d.command_dirs.iter().filter(|c| c.writable).count();
    let dirs_total = d.command_dirs.len();
    emit(check_line(
        dirs_writable == dirs_total,
        &format!("{dirs_writable} of {dirs_total} command directories writable"),
    ));

    if !d.command_dirs.is_empty() {
        emit(format!(
            "\n\x1b[1;33mCOMMAND DIRECTORIES\x1b[0m  {}",
            d.command_dirs.len()
        ));
        for c in &d.command_dirs {
            let glyph = if c.writable {
                "\x1b[1;32m✓\x1b[0m"
            } else {
                "\x1b[1;31m✗\x1b[0m"
            };
            emit(format!("  {glyph} {}", c.path));
        }
    }

    if !d.failures.is_empty() {
        emit("\n\x1b[1;33mFAILURES\x1b[0m".to_string());
        for f in &d.failures {
            emit(format!(
                "  \x1b[1;31m!\x1b[0m {} {} {}",
                f.name, f.reason, f.source
            ));
        }
    }
}

/// A `✓ / ✗ label` check row.
fn check_line(ok: bool, label: &str) -> String {
    if ok {
        format!("  \x1b[1;32m✓\x1b[0m {label}")
    } else {
        format!("  \x1b[1;31m✗\x1b[0m {label}")
    }
}

/// Human-readable inventory for `agent list` (the `AgentList` return; `list` emits no per-item
/// events, so this is the only place its rows are shown outside `--json`).
fn render_agent_list(l: &envctl_engine::AgentList) {
    if l.skills.is_empty() && l.mcps.is_empty() && l.commands.is_empty() {
        println!("  (no installed assets)");
        return;
    }
    if !l.skills.is_empty() {
        println!("  skills ({}):", l.skills.len());
        for s in &l.skills {
            println!(
                "    {} ({}) [{:?}] \u{2190} {}  {}",
                s.name, s.skill, s.scope, s.source, s.updated_ago
            );
        }
    }
    if !l.mcps.is_empty() {
        println!("  mcps ({}):", l.mcps.len());
        for m in &l.mcps {
            println!("    {} [{:?}] \u{2190} {}", m.name, m.scope, m.source);
        }
    }
    if !l.commands.is_empty() {
        println!("  commands ({}):", l.commands.len());
        for c in &l.commands {
            println!("    {} [{:?}] \u{2190} {}", c.name, c.scope, c.source);
        }
    }
}

/// `envctl agent {sync,add,remove,lock,list,clean}` — a THIN adapter over the shared
/// `Engine::agent_*` API. Builds the `Agent*Spec`, runs the engine on a worker thread
/// (draining the same EventSink the GUI uses), renders human/`--json`, and maps the
/// engine return to the fail-closed exit code. No business logic lives here.
fn run_agent(engine: Engine, cmd: AgentCmd, json: bool) -> anyhow::Result<()> {
    let (sink, rx) = EventSink::channel();
    let eng = engine.clone();
    let handle = std::thread::spawn(move || -> anyhow::Result<AgentResult> {
        let result = match cmd {
            AgentCmd::Sync {
                config,
                scope,
                apply,
                locked,
                update,
            } => {
                let spec = AgentSyncSpec {
                    config_path: config,
                    scope_override: scope.map(AgentScope::from),
                    apply,
                    lock_mode: AgentLockMode::from_flags(locked, update),
                };
                AgentResult::Report(eng.agent_sync(spec, &sink)?)
            }
            AgentCmd::Add {
                source,
                skill,
                mcp,
                command,
                git_ref,
                branch,
                sub_dir,
                config,
                scope,
                apply,
                no_sync,
                no_verify,
                locked,
                update,
            } => {
                let spec = AgentAddSpec {
                    source,
                    section: AgentSectionSel {
                        skills: skill,
                        mcps: mcp,
                        commands: command,
                    },
                    git_ref,
                    branch,
                    sub_dir,
                    config_path: config,
                    scope_override: scope.map(AgentScope::from),
                    apply,
                    no_sync,
                    no_verify,
                    lock_mode: AgentLockMode::from_flags(locked, update),
                };
                AgentResult::Edit(eng.agent_add(spec, &sink)?)
            }
            AgentCmd::Remove {
                source,
                skill,
                mcp,
                command,
                git_ref,
                branch,
                sub_dir,
                config,
                scope,
                apply,
                no_sync,
                locked,
                update,
            } => {
                let spec = AgentRemoveSpec {
                    source,
                    section: AgentSectionSel {
                        skills: skill,
                        mcps: mcp,
                        commands: command,
                    },
                    git_ref,
                    branch,
                    sub_dir,
                    config_path: config,
                    scope_override: scope.map(AgentScope::from),
                    apply,
                    no_sync,
                    lock_mode: AgentLockMode::from_flags(locked, update),
                };
                AgentResult::Edit(eng.agent_remove(spec, &sink)?)
            }
            AgentCmd::Lock {
                config,
                scope,
                check,
                upgrade_package,
                locked,
            } => {
                let spec = AgentLockSpec {
                    config_path: config,
                    scope_override: scope.map(AgentScope::from),
                    check,
                    upgrade_only: upgrade_package,
                    lock_mode: AgentLockMode::from_flags(locked, None),
                };
                AgentResult::Lock(eng.agent_lock(spec, &sink)?)
            }
            AgentCmd::List { scope, kind } => {
                let spec = AgentListSpec {
                    scope_override: scope.map(AgentScope::from),
                    kind: AgentListKind::from(kind),
                };
                AgentResult::List(eng.agent_list(spec, &sink)?)
            }
            AgentCmd::Clean {
                config,
                scope,
                apply,
            } => {
                let spec = AgentCleanSpec {
                    config_path: config,
                    scope_override: scope.map(AgentScope::from),
                    apply,
                };
                AgentResult::Report(eng.agent_clean(spec, &sink)?)
            }
            AgentCmd::Init { global, force } => {
                let spec = AgentInitSpec { global, force };
                AgentResult::Init(eng.agent_init(spec, &sink)?)
            }
            AgentCmd::Doctor { scope } => {
                let spec = AgentDoctorSpec {
                    scope_override: scope.map(AgentScope::from),
                };
                AgentResult::Doctor(eng.agent_doctor(spec, &sink)?)
            }
        };
        Ok(result) // sink drops here -> the main-thread rx.iter() terminates
    });

    // In `--json` mode the EventSink is silent (the uniform serialized RETURN value is
    // the machine output, matching `auto-detect`/`graph`/`lock`); human mode streams the
    // per-action tree via `print_event`.
    for ev in rx.iter() {
        if !json {
            print_event(&ev);
        }
    }

    let result = handle
        .join()
        .map_err(|_| anyhow::anyhow!("worker panicked"))??;

    if json {
        println!("{}", result.to_json()?);
    } else {
        result.render_human();
    }
    if !result.ok() {
        std::process::exit(1);
    }
    Ok(())
}

/// Run the `envctl secret` subcommand group — delegates to the installed `secretctl` binary
/// via Engine's subprocess seam. No gRPC client embedded; the CLI is a transparent proxy.
fn run_secret(cmd: SecretCmd, _json: bool) -> anyhow::Result<()> {
    let (sink, rx) = EventSink::channel();

    // Build argv for secretctl based on the subcommand variant.
    let verb_and_argv = match &cmd {
        SecretCmd::Status {} => ("status", vec!["status".to_string()]),
        SecretCmd::Init {
            passphrase_stdin,
            enroll_usb,
            usb_partuuid,
            apply,
        } => {
            let mut a = vec!["init".to_string()];
            if *passphrase_stdin {
                a.push("--passphrase-stdin".into());
            }
            if *enroll_usb {
                a.push("--enroll-usb".into());
            }
            if let Some(p) = usb_partuuid {
                a.push(format!("--usb-partuuid={}", p));
            }
            if *apply {
                a.push("--apply".into());
            }
            ("init", a)
        }
        SecretCmd::Unlock { passphrase_stdin } => {
            let mut a = vec!["unlock".to_string()];
            if *passphrase_stdin {
                a.push("--passphrase-stdin".into());
            }
            ("unlock", a)
        }
        SecretCmd::Lock {} => ("lock", vec!["lock".to_string()]),
        SecretCmd::Secret { cmd } => {
            let sub = match cmd {
                SecretSubCmd::Add {
                    name,
                    provider,
                    value_stdin,
                    note,
                    overwrite,
                    broker_only,
                } => {
                    let mut a = vec!["secret".into(), "add".into(), name.clone()];
                    a.push(format!("--provider={}", provider));
                    if *value_stdin {
                        a.push("--value-stdin".into());
                    }
                    if let Some(n) = note {
                        a.push(format!("--note={}", n));
                    }
                    if *overwrite {
                        a.push("--overwrite".into());
                    }
                    if *broker_only {
                        a.push("--broker-only".into());
                    }
                    ("secret-add", a)
                }
                SecretSubCmd::Get {
                    name,
                    reveal,
                    apply,
                    confirm,
                } => {
                    let mut a = vec!["secret".into(), "get".into(), name.clone()];
                    if *reveal {
                        a.push("--reveal".into());
                    }
                    if *apply {
                        a.push("--apply".into());
                    }
                    if *confirm {
                        a.push("--confirm".into());
                    }
                    ("secret-get", a)
                }
                SecretSubCmd::List { provider } => {
                    let mut a = vec!["secret".into(), "list".into()];
                    if let Some(p) = provider {
                        a.push(format!("--provider={}", p));
                    }
                    ("secret-list", a)
                }
                SecretSubCmd::Rm {
                    name,
                    apply,
                    confirm,
                } => {
                    let mut a = vec!["secret".into(), "rm".into(), name.clone()];
                    if *apply {
                        a.push("--apply".into());
                    }
                    if *confirm {
                        a.push("--confirm".into());
                    }
                    ("secret-rm", a)
                }
                SecretSubCmd::Rotate {
                    name,
                    value_stdin,
                    apply,
                } => {
                    let mut a = vec!["secret".into(), "rotate".into(), name.clone()];
                    if *value_stdin {
                        a.push("--value-stdin".into());
                    }
                    if *apply {
                        a.push("--apply".into());
                    }
                    ("secret-rotate", a)
                }
            };
            sub
        }
        SecretCmd::Relay { cmd } => {
            let sub = match cmd {
                RelaySubCmd::Create {
                    name,
                    secret,
                    provider,
                    mode,
                    upstream_base,
                    hosts,
                    paths,
                    methods,
                    expires,
                    rate,
                    quota,
                    disabled,
                } => {
                    let mut a = vec!["relay".into(), "create".into(), name.clone()];
                    a.push(format!("--secret={}", secret));
                    a.push(format!("--provider={}", provider));
                    a.push(format!("--mode={}", mode));
                    if let Some(u) = upstream_base {
                        a.push(format!("--upstream-base={}", u));
                    }
                    for h in hosts {
                        a.push(format!("--host={}", h));
                    }
                    for p in paths {
                        a.push(format!("--path={}", p));
                    }
                    for m in methods {
                        a.push(format!("--method={}", m));
                    }
                    if let Some(e) = expires {
                        a.push(format!("--expires={}", e));
                    }
                    if let Some(r) = rate {
                        a.push(format!("--rate={}", r));
                    }
                    if let Some(q) = quota {
                        a.push(format!("--quota={}", q));
                    }
                    if *disabled {
                        a.push("--disabled".into());
                    }
                    ("relay-create", a)
                }
                RelaySubCmd::Revoke {
                    name,
                    apply,
                    confirm,
                } => {
                    let mut a = vec!["relay".into(), "revoke".into(), name.clone()];
                    if *apply {
                        a.push("--apply".into());
                    }
                    if *confirm {
                        a.push("--confirm".into());
                    }
                    ("relay-revoke", a)
                }
                RelaySubCmd::RevokeToken { token_id, apply } => {
                    let mut a = vec!["relay".into(), "revoke-token".into(), token_id.clone()];
                    if *apply {
                        a.push("--apply".into());
                    }
                    ("relay-revoke-token", a)
                }
                RelaySubCmd::List { all } => {
                    let mut a = vec!["relay".into(), "list".into()];
                    if *all {
                        a.push("--all".into());
                    }
                    ("relay-list", a)
                }
                RelaySubCmd::Mint {
                    name,
                    ttl,
                    mode,
                    provider,
                    repos,
                    perms,
                } => {
                    let mut a = vec!["relay".into(), "mint".into(), name.clone()];
                    if let Some(t) = ttl {
                        a.push(format!("--ttl={}", t));
                    }
                    if let Some(m) = mode {
                        a.push(format!("--mode={}", m));
                    }
                    if let Some(p) = provider {
                        a.push(format!("--provider={}", p));
                    }
                    for r in repos {
                        a.push(format!("--repo={}", r));
                    }
                    for p in perms {
                        a.push(format!("--perm={}", p));
                    }
                    ("relay-mint", a)
                }
            };
            sub
        }
        SecretCmd::Ca { cmd } => {
            let sub = match cmd {
                CaSubCmd::Init { apply } => {
                    let mut a = vec!["ca".into(), "init".into()];
                    if *apply {
                        a.push("--apply".into());
                    }
                    ("ca-init", a)
                }
                CaSubCmd::Rotate { apply, confirm } => {
                    let mut a = vec!["ca".into(), "rotate".into()];
                    if *apply {
                        a.push("--apply".into());
                    }
                    if *confirm {
                        a.push("--confirm".into());
                    }
                    ("ca-rotate", a)
                }
                CaSubCmd::Issue {
                    cn,
                    sans,
                    ttl_days,
                    usage,
                } => {
                    let mut a = vec![String::from("ca"), String::from("issue"), cn.clone()];
                    a.extend(sans.iter().map(|s| format!("--san={}", s)));
                    if let Some(t) = ttl_days {
                        a.push(format!("--ttl-days={}", t));
                    }
                    a.push(format!("--usage={}", usage));
                    ("ca-issue", a)
                }
                CaSubCmd::Renew { cn, apply } => {
                    let mut a = vec![String::from("ca"), String::from("renew"), cn.clone()];
                    if *apply {
                        a.push("--apply".into());
                    }
                    ("ca-renew", a)
                }
                CaSubCmd::Revoke { cn, apply, confirm } => {
                    let mut a = vec![String::from("ca"), String::from("revoke"), cn.clone()];
                    if *apply {
                        a.push("--apply".into());
                    }
                    if *confirm {
                        a.push("--confirm".into());
                    }
                    ("ca-revoke", a)
                }
                CaSubCmd::Trust {
                    targets,
                    system_bundle,
                    apply,
                    confirm,
                } => {
                    let mut a = vec!["ca".into(), "trust-apply".into()];
                    for t in targets {
                        a.push(format!("--target={}", t));
                    }
                    if *system_bundle {
                        a.push("--system-bundle".into());
                    }
                    if *apply {
                        a.push("--apply".into());
                    }
                    if *confirm {
                        a.push("--confirm".into());
                    }
                    ("ca-trust", a)
                }
            };
            sub
        }
        SecretCmd::Audit(args) => {
            let mut a = vec!["audit".into()];
            if let Some(actor) = &args.actor {
                a.push(format!("--actor={}", actor));
            }
            if let Some(relay) = &args.relay {
                a.push(format!("--relay={}", relay));
            }
            if let Some(since) = &args.since {
                a.push(format!("--since={}", since));
            }
            if let Some(until) = &args.until {
                a.push(format!("--until={}", until));
            }
            if let Some(limit) = args.limit {
                a.push(format!("--limit={}", limit));
            }
            ("audit", a)
        }
        SecretCmd::Run(args) => {
            let mut a = vec!["run".into()];
            for r in &args.relays {
                a.push(format!("--relay={}", r));
            }
            if let Some(provider) = &args.provider {
                a.push(format!("--provider={}", provider));
            }
            if args.ephemeral {
                a.push("--ephemeral".into());
            }
            if args.no_profile {
                a.push("--no-profile".into());
            }
            if let Some(profile) = &args.profile {
                a.push(format!("--profile={}", profile));
            }
            // argv: the last element from clap's trailing_var_arg is already split; we join it as the command.
            if !args.argv.is_empty() {
                a.extend(args.argv.iter().cloned());
            }
            ("run", a)
        }
        SecretCmd::MintGithub(args) => {
            let mut a = vec![
                "mint-github".into(),
                format!("--installation-id={}", args.installation_id),
            ];
            if !args.repository_ids.is_empty() {
                a.push(format!(
                    "--repository-ids={}",
                    args.repository_ids.join(",")
                ));
            }
            if !args.permissions.is_empty() {
                a.push(format!("--permissions={}", args.permissions.join(",")));
            }
            if let Some(ttl) = args.ttl_secs {
                a.push(format!("--ttl-secs={}", ttl));
            }
            a.push(format!("--output={}", args.output));
            ("mint-github", a)
        }
        SecretCmd::GithubApp { cmd } => {
            let sub = match cmd {
                GithubAppSubCmd::Enroll {
                    app_id,
                    private_key,
                    apply,
                } => {
                    let mut a = vec!["github-app".into(), "enroll".into()];
                    a.push(format!("--app-id={}", app_id));
                    a.push(format!("--private-key={}", private_key));
                    if *apply {
                        a.push("--apply".into());
                    }
                    ("github-app-enroll", a)
                }
                GithubAppSubCmd::SetAppId { app_id, apply } => {
                    let mut a = vec!["github-app".into(), "set-app-id".into()];
                    a.push(format!("--app-id={}", app_id));
                    if *apply {
                        a.push("--apply".into());
                    }
                    ("github-app-set-app-id", a)
                }
                GithubAppSubCmd::RevokeToken {
                    token,
                    installation_id,
                    apply,
                } => {
                    let mut a = vec!["github-app".into(), "revoke-token".into()];
                    a.push(format!("--token={}", token));
                    if let Some(id) = installation_id {
                        a.push(format!("--installation-id={}", id));
                    }
                    if *apply {
                        a.push("--apply".into());
                    }
                    ("github-app-revoke-token", a)
                }
            };
            sub
        }
    };

    // Spawn the subprocess via Engine's secrets seam.
    let (verb, argv) = verb_and_argv;
    run_secretctl(
        verb.to_string(),
        argv,
        None, // stdin: no secret input needed for these verbs
        &sink,
    );

    // Drain events and render results.
    for ev in rx.iter() {
        if let Event::SecretsResult {
            verb: v,
            json_stdout,
            stderr,
            code,
        } = &ev
        {
            if *v == verb && !json_stdout.is_empty() {
                println!("{}", json_stdout);
            }
            if !stderr.is_empty() {
                eprintln!("{}", stderr);
            }
            if let Some(c) = code {
                std::process::exit(*c);
            }
        }
    }

    Ok(())
}

struct DashboardArgs {
    meta_file: Option<std::path::PathBuf>,
    panes_per_tab: usize,
    name: String,
    deploy: bool,
    apply: bool,
    force: bool,
}

/// `envctl dashboard`. Read-only by default (render the KDL to stdout). `--deploy`
/// previews the write; `--deploy --apply` performs it (fail-closed in the engine).
/// `--json` emits the DashboardPlan. Runs on the main thread (render is read-only;
/// deploy is fail-closed in the engine).
fn run_dashboard(engine: &Engine, args: DashboardArgs, json: bool) -> anyhow::Result<()> {
    let (sink, _rx) = EventSink::channel();
    let start = std::env::current_dir()?;
    let spec = DashboardSpec {
        name: args.name,
        panes_per_tab: args.panes_per_tab,
        ..DashboardSpec::default()
    };

    if args.deploy {
        // dry-run unless --apply (fail-closed default).
        let dry_run = !args.apply;
        let plan = engine.dashboard(start.clone(), args.meta_file.clone(), spec.clone(), &sink)?;
        let outcome =
            engine.deploy_dashboard(start, args.meta_file, spec, dry_run, args.force, &sink)?;
        if json {
            println!(
                "{}",
                serde_json::to_string_pretty(&serde_json::json!({
                    "plan": plan,
                    "deploy": outcome,
                }))?
            );
        } else {
            for note in &outcome.notes {
                println!("  {note}");
            }
            if outcome.applied {
                println!(
                    "\x1b[1;32m✓ deployed {} ({} tabs)\x1b[0m",
                    outcome.target.display(),
                    plan.tabs.len()
                );
            } else {
                println!(
                    "\x1b[1;33m· dry-run: pass --apply to write {} ({} tabs)\x1b[0m",
                    outcome.target.display(),
                    plan.tabs.len()
                );
            }
        }
        return Ok(());
    }

    // Default: render-only.
    let plan = engine.dashboard(start, args.meta_file, spec, &sink)?;
    if json {
        println!("{}", serde_json::to_string_pretty(&plan)?);
    } else {
        print!("{}", plan.kdl);
    }
    Ok(())
}

/// Emit a rendered line, stripping ANSI when `--color never` / non-TTY resolved to plain.
fn emit(line: String) {
    println!("{}", paint(line));
}

fn print_event(ev: &Event) {
    let o = out();
    // quiet>=1 drops progress/log/info chatter but ALWAYS keeps failures/refusals.
    let quiet = o.quiet >= 1;
    match ev {
        Event::StepStarted {
            component,
            phase,
            index,
            total,
        } => {
            if quiet {
                return;
            }
            emit(format!(
                "\x1b[1;36m==> [{}/{}] {component} :: {phase:?}\x1b[0m",
                index + 1,
                total
            ))
        }
        Event::Log {
            line, component, ..
        } => {
            // Log lines are detail: dropped under --quiet; shown otherwise. With -v (verbose),
            // prefix each line with its component so interleaved streams are attributable.
            if quiet {
                return;
            }
            if o.verbose >= 1 {
                emit(format!("    [{component}] {line}"))
            } else {
                emit(format!("    {line}"))
            }
        }
        Event::StepFinished { result } => match result.status {
            // Successes / dry-run / skips are progress chatter — dropped under --quiet.
            OpStatus::Ok => {
                if !quiet {
                    emit(format!(
                        "\x1b[1;32m  ✓ {} {:?}\x1b[0m",
                        result.component, result.phase
                    ))
                }
            }
            // Failures, refusals, incomplete: ALWAYS printed even under --quiet.
            OpStatus::Failed => emit(format!(
                "\x1b[1;33m  ! FAILED {} (exit {:?})\x1b[0m",
                result.component, result.exit_code
            )),
            OpStatus::Refused => emit(format!(
                "\x1b[1;31m  ⛔ REFUSED {}: {}\x1b[0m",
                result.component, result.message
            )),
            OpStatus::Skipped => {
                if !quiet {
                    emit(format!(
                        "  — skip {} ({})",
                        result.component, result.message
                    ))
                }
            }
            OpStatus::SkippedBlocked => emit(format!(
                "\x1b[1;33m  — blocked {} ({})\x1b[0m",
                result.component, result.message
            )),
            OpStatus::DryRun => {
                if !quiet {
                    emit(format!("  · would {:?} {}", result.phase, result.component))
                }
            }
            OpStatus::RebootRequired => emit(format!(
                "\x1b[1;33m  ⟳ {} needs a REBOOT to take effect\x1b[0m",
                result.component
            )),
            OpStatus::Incomplete => emit(format!(
                "\x1b[1;31m  ✗ {} acted but post-state wrong: {}\x1b[0m",
                result.component, result.message
            )),
            OpStatus::NoHook => {}
        },
        Event::GuardRefused { component, reason } => {
            // A refusal is never suppressed.
            emit(format!(
                "\x1b[1;31m  ⛔ REFUSED {component}: {reason}\x1b[0m"
            ))
        }
        Event::MigrationReported { report } => {
            if quiet {
                return;
            }
            emit(format!(
                "\x1b[1;36m==> migrate {} :: {} items, {} needs migration, {} missing canonical, {} protected, {} refused\x1b[0m",
                migration_verb_label(report.verb),
                report.summary.total,
                report.summary.needs_migration,
                report.summary.missing_canonical,
                report.summary.protected,
                report.summary.refused,
            ));
        }
        Event::AgentRunStarted {
            verb,
            scope,
            dry_run,
            lock_mode,
        } => {
            if quiet {
                return;
            }
            let mode = if *dry_run { " (preview)" } else { "" };
            emit(format!(
                "\x1b[1;36m==> agent {verb:?} :: {scope:?} [{lock_mode}]{mode}\x1b[0m"
            ));
        }
        Event::AgentAction {
            source,
            asset,
            status,
            error,
        } => {
            let who = match (source, asset) {
                (Some(s), Some(a)) => format!("{s}::{a}"),
                (Some(s), None) => s.clone(),
                (None, Some(a)) => a.clone(),
                (None, None) => String::new(),
            };
            match error {
                // Errors always print.
                Some(e) => emit(format!("\x1b[1;31m  ✗ {status} {who}: {e}\x1b[0m")),
                None => {
                    if !quiet {
                        emit(format!("  {status} {who}"))
                    }
                }
            }
        }
        Event::AgentRunFinished { report } => {
            let s = &report.summary;
            if s.failed == 0 {
                if !quiet {
                    emit(format!(
                        "\x1b[1;32mdone: {} installed, {} updated, {} removed, {} unchanged.\x1b[0m",
                        s.installed, s.updated, s.removed, s.unchanged
                    ))
                }
            } else {
                emit(format!(
                    "\x1b[1;33mdone with {} failed ({} installed, {} updated, {} removed).\x1b[0m",
                    s.failed, s.installed, s.updated, s.removed
                ))
            }
        }
        Event::AgentLockChecked { drift } => {
            if drift.is_empty() {
                if !quiet {
                    emit("\x1b[1;32m✓ agent-env.lock is up to date\x1b[0m".to_string())
                }
            } else {
                emit(format!("\x1b[1;33m✗ lock drift ({}):\x1b[0m", drift.len()));
                for d in drift {
                    emit(format!("  {}  {}", d.status, d.id));
                }
            }
        }
        Event::AgentInitFinished { outcome } => {
            if !quiet {
                emit(format!(
                    "\x1b[1;32m✓ {} {}\x1b[0m",
                    if outcome.overwritten {
                        "overwrote"
                    } else {
                        "created"
                    },
                    outcome.path
                ))
            }
        }
        Event::AgentDoctored { report } => {
            // `agent doctor` honors --quiet (kasetto: quiet && !json => no-op). The human view
            // is rendered from the typed return in `render_agent_doctor`, not here.
            let _ = report;
        }
        Event::RunFinished { summary } => {
            if summary.ok() {
                if !quiet {
                    emit("\x1b[1;32mdone.\x1b[0m".to_string())
                }
            } else {
                emit(format!(
                    "\x1b[1;33mdone with {} failed, {} refused, {} blocked, {} incomplete.\x1b[0m",
                    summary.failed.len(),
                    summary.refused.len(),
                    summary.skipped_blocked.len(),
                    summary.incomplete.len()
                ))
            }
        }
        _ => {}
    }
}

/// Interactive add-repo: build the spec, drop the user into an agent session in
/// the clone, then (if --build) build the now-transformed tree as-is.
fn handle_connect(engine: Engine, cmd: Cmd, json: bool) -> anyhow::Result<()> {
    let Cmd::AddRepo {
        git_url,
        id,
        local,
        git_ref,
        build_system,
        build_cmd,
        artifacts,
        mode,
        provides,
        tags,
        strategy,
        bins,
        renames,
        patch_cmd,
        ai_goal,
        ai_agent,
        ai_instruction,
        daemon,
        verify_cmd,
        build,
        force,
        recurse_submodules,
        connect: _,
        dry_run: _,
    } = cmd
    else {
        unreachable!("handle_connect only called for AddRepo");
    };
    let spec = build_spec(AddRepoArgs {
        git_url,
        id,
        local,
        git_ref,
        build_system,
        build_cmd,
        artifacts,
        mode,
        provides,
        tags,
        strategy,
        bins,
        renames,
        patch_cmd,
        ai_goal,
        ai_agent,
        ai_instruction,
        daemon,
        verify_cmd,
        build,
        force,
        recurse_submodules,
    })
    .map_err(|e| anyhow::anyhow!(e))?;

    engine.connect_repo(&spec)?; // interactive; blocks on the terminal

    if spec.allow_build {
        // Build the transformed clone AS-IS (don't re-run the agent).
        let bspec = AddRepoSpec {
            strategy: BuildStrategy::AsIs,
            allow_build: true,
            ..spec
        };
        let (sink, rx) = EventSink::channel();
        // Audit fix: capture the summary instead of discarding it so a failed
        // post-connect build exits 1, matching run_action's contract.
        let res = engine.add_repo(bspec, false, &sink)?;
        drop(sink);
        for ev in rx.iter() {
            if json {
                println!("{}", serde_json::to_string(&ev)?);
            } else {
                print_event(&ev);
            }
        }
        if !res.ok() {
            std::process::exit(1);
        }
    } else {
        println!("\nenvctl: clone is ready. Build what you made with:");
        println!(
            "  envctl add-repo {} --id {} --strategy as-is --build",
            spec.git_url, spec.id
        );
    }
    Ok(())
}

/// Read-only health diagnostics (kasetto-style `doctor`): writability, toolchains,
/// sudo, UEFI/Secure-Boot, GPU, and the run log. Never mutates anything.
fn print_doctor(engine: &Engine, json: bool) -> anyhow::Result<()> {
    let last_run = envctl_engine::runtime::load(engine.manifest_dir()).last_run;
    let detected = engine.detect(&EventSink::null()).ok();
    let layout = envctl_engine::MetaLayout::from_env_or_default();
    let write_ok = |p: &str| -> bool {
        let dir = std::path::Path::new(p);
        if std::fs::create_dir_all(dir).is_err() {
            return false;
        }
        let t = dir.join(".envctl-doctor-probe");
        let ok = std::fs::write(&t, b"x").is_ok();
        let _ = std::fs::remove_file(&t);
        ok
    };
    let has = |bin: &str| -> Option<String> { doctor_tool_version(bin) };
    let layout_entries = layout.entries();
    let mut dirs: Vec<String> = layout_entries
        .iter()
        .filter(|entry| entry.is_canonical())
        .map(|entry| entry.path.display().to_string())
        .collect();
    dirs.push("/etc".to_string());
    let tools = [
        "git",
        "cargo",
        "rustc",
        "claude",
        "nix",
        "podman",
        "nvidia-smi",
        "gh",
        "uv",
        "bun",
    ];
    let sudo_cached = std::process::Command::new("sudo")
        .args(["-n", "true"])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false);
    let uefi = std::path::Path::new("/sys/firmware/efi").exists();
    let secure_boot = std::process::Command::new("bash")
        .args(["-lc", "od -An -t u1 /sys/firmware/efi/efivars/SecureBoot-* 2>/dev/null | tr -s ' ' | awk '{print $NF}' | head -1"])
        .output()
        .ok()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .filter(|s| !s.is_empty());
    let driver_loaded = std::path::Path::new("/proc/driver/nvidia/version").exists();
    let run_log = layout.state().join("envctl.log");
    let log_exists = run_log.exists();

    if json {
        let dirj: Vec<_> = dirs
            .iter()
            .map(|d| serde_json::json!({"path": d, "writable": write_ok(d)}))
            .collect();
        let layoutj: Vec<_> = layout_entries
            .iter()
            .map(|entry| {
                serde_json::json!({
                    "key": entry.key,
                    "path": entry.path.display().to_string(),
                    "kind": match entry.kind {
                        envctl_engine::LayoutKind::Canonical => "canonical",
                        envctl_engine::LayoutKind::LegacyCompatibility => "legacy_compatibility",
                    },
                    "purpose": entry.purpose,
                })
            })
            .collect();
        let toolj: Vec<_> = tools
            .iter()
            .map(|t| serde_json::json!({"tool": t, "version": has(t)}))
            .collect();
        println!(
            "{}",
            serde_json::to_string_pretty(&serde_json::json!({
                "writable": dirj, "layout_registry": layoutj,
                "tools": toolj, "sudo_cached": sudo_cached,
                "uefi": uefi, "secure_boot": secure_boot, "nvidia_driver_loaded": driver_loaded,
                "run_log": run_log.display().to_string(), "run_log_exists": log_exists,
                "meta_boundary": detected.as_ref().map(|r| &r.meta_boundary),
                "last_run": last_run,
            }))?
        );
        return Ok(());
    }

    let yn = |b: bool| {
        if b {
            "\x1b[1;32m✓\x1b[0m"
        } else {
            "\x1b[1;31m✗\x1b[0m"
        }
    };
    println!("\x1b[1;36m── writability ──\x1b[0m");
    for d in &dirs {
        println!("  {}  {d}", yn(write_ok(d)));
    }
    println!("\x1b[1;36m── toolchains ──\x1b[0m");
    for t in &tools {
        match has(t) {
            Some(v) => println!("  \x1b[1;32m✓\x1b[0m {t:<11} {v}"),
            None => println!("  \x1b[1;90m·\x1b[0m {t:<11} (absent)"),
        }
    }
    println!("\x1b[1;36m── system ──\x1b[0m");
    println!("  sudo (cached)      {}", yn(sudo_cached));
    println!("  UEFI               {}", yn(uefi));
    println!(
        "  Secure Boot        {}",
        match secure_boot.as_deref() {
            Some("1") => "\x1b[1;33mON\x1b[0m (nvidia-open needs it OFF)",
            Some("0") => "\x1b[1;32mOFF\x1b[0m",
            _ => "unknown",
        }
    );
    println!(
        "  nvidia driver      {}",
        if driver_loaded {
            "\x1b[1;32mloaded\x1b[0m"
        } else {
            "\x1b[1;33mnot loaded\x1b[0m"
        }
    );
    println!(
        "  run log            {} {}",
        yn(log_exists),
        run_log.display()
    );
    match &last_run {
        Some(lr) => println!(
            "  last op            {} {} ({}f/{}r/{}i) at {}",
            lr.verb,
            if lr.ok {
                "\x1b[1;32mok\x1b[0m"
            } else {
                "\x1b[1;31mFAILED\x1b[0m"
            },
            lr.failed,
            lr.refused,
            lr.incomplete,
            lr.at
        ),
        None => println!("  last op            (none recorded)"),
    }
    if let Some(report) = detected.as_ref() {
        print_meta_boundary(report);
    }
    if !sudo_cached {
        println!("\n  note: sudo not pre-authorized — privileged installs need `sudo -v` in a real terminal first.");
    }
    Ok(())
}

fn doctor_tool_version(bin: &str) -> Option<String> {
    let path = find_on_path(bin)?;
    let out = command_output_timeout(
        std::process::Command::new(&path).arg("--version"),
        std::time::Duration::from_secs(2),
    )
    .ok()
    .flatten();
    out.and_then(|out| {
        out.status.success().then(|| {
            String::from_utf8_lossy(&out.stdout)
                .lines()
                .next()
                .map(str::trim)
                .filter(|line| !line.is_empty())
                .map(str::to_string)
        })?
    })
    .or_else(|| Some(path.display().to_string()))
}

fn find_on_path(bin: &str) -> Option<std::path::PathBuf> {
    let candidate = std::path::Path::new(bin);
    if candidate.components().count() > 1 {
        return executable_file(candidate).then(|| candidate.to_path_buf());
    }
    std::env::var_os("PATH").and_then(|path| {
        std::env::split_paths(&path)
            .map(|dir| dir.join(bin))
            .find(|candidate| executable_file(candidate))
    })
}

fn executable_file(path: &std::path::Path) -> bool {
    let Ok(meta) = std::fs::metadata(path) else {
        return false;
    };
    if !meta.is_file() {
        return false;
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        meta.permissions().mode() & 0o111 != 0
    }
    #[cfg(not(unix))]
    {
        true
    }
}

fn command_output_timeout(
    cmd: &mut std::process::Command,
    timeout: std::time::Duration,
) -> std::io::Result<Option<std::process::Output>> {
    let start = std::time::Instant::now();
    let mut child = cmd
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::null())
        .spawn()?;
    loop {
        if child.try_wait()?.is_some() {
            return child.wait_with_output().map(Some);
        }
        if start.elapsed() >= timeout {
            let _ = child.kill();
            let _ = child.wait();
            return Ok(None);
        }
        std::thread::sleep(std::time::Duration::from_millis(25));
    }
}

fn print_graph_summary(reg: &envctl_engine::Registry) {
    let g = envctl_engine::graph::analyze(reg);
    let c = "\x1b[1;36m";
    let z = "\x1b[0m";
    println!("{c}── dependency graph ──{z}");
    println!(
        "  {} components · {} edges · {} groups",
        g.nodes,
        g.edges,
        g.groups.len()
    );
    println!("  roots (no deps):     {}", g.roots.len());
    println!("  leaves (top-level):  {}", g.leaves.len());
    if !g.orphans.is_empty() {
        println!("  orphans (standalone): {}", g.orphans.join(", "));
    }
    if let Some((id, n)) = &g.max_dependents {
        println!("  most depended-on:    {id}  ({n} direct dependents)");
    }
    if let Some((id, n)) = &g.max_requires {
        println!("  most prerequisites:  {id}  ({n} requires)");
    }
    println!("{c}── critical path (longest chain) ──{z}");
    println!("  {}", g.critical_path.join("  →  "));
    println!("\n  tip: envctl graph --impact <id> · --why <id> · --dot | dot -Tsvg -o g.svg · --json --live");
}

/// Returns false when the component is unknown (so the caller can set a
/// non-zero exit code). audit fix (minor).
fn print_impact(reg: &envctl_engine::Registry, id: &str) -> bool {
    match envctl_engine::graph::impact(reg, id) {
        None => {
            eprintln!("envctl: unknown component '{id}'");
            false
        }
        Some(im) => {
            println!("\x1b[1;36m── impact of '{id}' ──\x1b[0m");
            println!("  direct requires:     {}", join_or_none(&im.requires));
            println!("  direct dependents:   {}", join_or_none(&im.required_by));
            println!(
                "\x1b[1;32m  install {id}\x1b[0m pulls in ({}):",
                im.install_closure.len()
            );
            println!("    {}", im.install_closure.join("  →  "));
            println!(
                "\x1b[1;33m  reset {id} --cascade\x1b[0m also removes ({}):",
                im.cascade_removes.len()
            );
            println!("    {}", join_or_none(&im.cascade_removes));
            true
        }
    }
}

/// Returns false when the component is unknown / has no paths. audit fix (minor).
fn print_why(reg: &envctl_engine::Registry, id: &str) -> bool {
    let paths = envctl_engine::graph::dependency_paths(reg, id);
    if paths.is_empty() {
        eprintln!("envctl: unknown component '{id}' (or it has no paths)");
        return false;
    }
    println!("\x1b[1;36m── why '{id}' is needed (root → {id} paths) ──\x1b[0m");
    for p in paths {
        println!("  {}", p.join("  →  "));
    }
    true
}

fn join_or_none(v: &[String]) -> String {
    if v.is_empty() {
        "(none)".into()
    } else {
        v.join(", ")
    }
}

fn parse_rename(s: &str) -> Result<(String, String), String> {
    s.split_once('=')
        .map(|(a, b)| (a.trim().to_string(), b.trim().to_string()))
        .filter(|(a, b)| !a.is_empty() && !b.is_empty())
        .ok_or_else(|| format!("expected old=new, got `{s}`"))
}

/// Flattened add-repo flags (keeps build_spec's signature sane).
struct AddRepoArgs {
    git_url: String,
    id: String,
    local: Option<std::path::PathBuf>,
    git_ref: Option<String>,
    build_system: Option<String>,
    build_cmd: Option<String>,
    artifacts: Vec<String>,
    strategy: String,
    bins: Vec<String>,
    renames: Vec<(String, String)>,
    patch_cmd: Option<String>,
    ai_goal: Option<String>,
    ai_agent: Option<String>,
    ai_instruction: Option<String>,
    daemon: bool,
    verify_cmd: Option<String>,
    build: bool,
    force: bool,
    recurse_submodules: bool,
    mode: String,
    provides: Vec<String>,
    tags: Vec<String>,
}

fn build_spec(a: AddRepoArgs) -> Result<AddRepoSpec, String> {
    let mode = match a.mode.as_str() {
        "auto" => AddRepoMode::Auto,
        "peer" => AddRepoMode::Peer,
        "component" => AddRepoMode::Component,
        other => return Err(format!("unknown --mode `{other}` (auto|peer|component)")),
    };
    let strategy = match a.strategy.as_str() {
        "as-is" => BuildStrategy::AsIs,
        "cherry-pick" => BuildStrategy::CherryPick { bins: a.bins },
        "rename" => BuildStrategy::Rename {
            renames: a
                .renames
                .into_iter()
                .map(|(from, to)| RenameRule { from, to })
                .collect(),
        },
        "refactor" => BuildStrategy::Refactor {
            refactor: if let Some(cmd) = a.patch_cmd {
                Refactor::Patch { command: cmd }
            } else {
                Refactor::Ai {
                    agent: a.ai_agent.as_deref().map(parse_agent).transpose()?,
                    goal: parse_goal(a.ai_goal.as_deref().unwrap_or("custom"))?,
                    instruction: a.ai_instruction,
                }
            },
        },
        other => {
            return Err(format!(
                "unknown --strategy `{other}` (as-is|cherry-pick|rename|refactor)"
            ))
        }
    };
    Ok(AddRepoSpec {
        id: a.id,
        git_url: a.git_url,
        local_path: a.local,
        git_ref: a.git_ref,
        build_cmd: a.build_cmd.unwrap_or_default(),
        build_system: a
            .build_system
            .as_deref()
            .map(parse_build_system)
            .transpose()?,
        artifacts: a.artifacts,
        strategy,
        bin_dir: None,
        daemon: a.daemon,
        verify_cmd: a.verify_cmd,
        allow_build: a.build,
        force: a.force,
        recurse_submodules: a.recurse_submodules,
        mode,
        provides: a.provides,
        tags: a.tags,
    })
}

fn parse_goal(s: &str) -> Result<RefactorGoal, String> {
    match s {
        "port-to-rust" => Ok(RefactorGoal::PortToRust),
        "cherry-pick-to-crate" => Ok(RefactorGoal::CherryPickToCrate),
        "rename-for-synergy" => Ok(RefactorGoal::RenameForSynergy),
        "custom" => Ok(RefactorGoal::Custom),
        other => Err(format!("unknown --ai-goal `{other}`")),
    }
}
fn parse_agent(s: &str) -> Result<AiAgent, String> {
    match s {
        "claude" => Ok(AiAgent::Claude),
        "codex" => Ok(AiAgent::Codex),
        "gemini" => Ok(AiAgent::Gemini),
        "kimi" => Ok(AiAgent::Kimi),
        other => Err(format!("unknown --ai-agent `{other}`")),
    }
}
fn parse_build_system(s: &str) -> Result<BuildSystem, String> {
    match s {
        "auto" => Ok(BuildSystem::Auto),
        "cargo" => Ok(BuildSystem::Cargo),
        "cmake" => Ok(BuildSystem::Cmake),
        "meson" => Ok(BuildSystem::Meson),
        "autotools" => Ok(BuildSystem::Autotools),
        "make" => Ok(BuildSystem::Make),
        "node" => Ok(BuildSystem::Node),
        "python" => Ok(BuildSystem::Python),
        "nix_flake" | "nix-flake" => Ok(BuildSystem::NixFlake),
        "go" => Ok(BuildSystem::Go),
        "zig" => Ok(BuildSystem::Zig),
        other => Err(format!("unknown --build-system `{other}`")),
    }
}

fn print_report(r: &EnvReport) {
    let yn = |b: bool| if b { "yes" } else { "no" };
    println!("\x1b[1;36m── host ──\x1b[0m");
    println!("  os       {}", r.os.as_deref().unwrap_or("?"));
    println!("  kernel   {}", r.kernel.as_deref().unwrap_or("?"));
    println!(
        "  cpu      {}  ({} threads)",
        r.cpu_model.as_deref().unwrap_or("?"),
        r.cpu_threads
    );
    println!("  memory   {} MiB", r.mem_total_mb);

    println!("\x1b[1;36m── gpu ──\x1b[0m");
    println!("  nvidia GPUs (PCI floor)  {}", r.gpu_count);
    for g in &r.gpus {
        println!("    • {g}");
    }
    println!("  driver loaded   {}", yn(r.driver_loaded));
    if let Some(v) = &r.driver_version {
        println!("  driver version  {v}");
    }
    println!("  open module     {}", yn(r.open_kernel_module));
    if let Some(c) = &r.cuda_version {
        println!("  cuda (nvcc)     {c}");
    }
    if r.software_rendered {
        println!("  \x1b[1;33m⟳ software-rendered: install/REBOOT nvidia-open to light up the GPUs\x1b[0m");
    }

    if !r.tools.is_empty() {
        println!("\x1b[1;36m── tools ──\x1b[0m");
        for t in &r.tools {
            println!(
                "  {:<12} {}",
                t.name,
                t.version.as_deref().unwrap_or("present")
            );
        }
    }

    print_meta_boundary(r);

    println!("\x1b[1;36m── components ──\x1b[0m");
    for c in &r.components {
        let mark = if c.detected {
            "\x1b[1;32m✓\x1b[0m"
        } else {
            "\x1b[1;90m·\x1b[0m"
        };
        let health = match c.healthy {
            Some(true) => " [healthy]",
            Some(false) => " \x1b[1;33m[unhealthy]\x1b[0m",
            None => "",
        };
        let note = if c.note.is_empty() {
            String::new()
        } else {
            format!("  ({})", c.note)
        };
        let wired = if c.wiring_present { " wired" } else { "" };
        println!(
            "  {mark} {:<16} {}{}{}{}",
            c.id, c.name, health, wired, note
        );
    }

    if r.drift.is_empty() {
        println!("\n\x1b[1;32m── no drift: environment matches the manifest ──\x1b[0m");
    } else {
        println!("\n\x1b[1;36m── drift ({}) ──\x1b[0m", r.drift.len());
        for d in &r.drift {
            let sev = match d.severity {
                Severity::High => "\x1b[1;31mhigh\x1b[0m",
                Severity::Medium => "\x1b[1;33mmed \x1b[0m",
                Severity::Low => "\x1b[1;90mlow \x1b[0m",
            };
            println!(
                "  [{sev}] {:<22} {:?}: {}\n             → {}",
                d.component, d.kind, d.detail, d.suggested_verb
            );
        }
    }
    println!("\n  generated_at {}", r.generated_at);
}

fn print_meta_boundary(r: &EnvReport) {
    let b = &r.meta_boundary;
    println!("\x1b[1;36m── meta boundary ──\x1b[0m");
    match b.meta_root.as_deref() {
        Some(root) => println!("  META_ROOT          {root}"),
        None => {
            println!("  META_ROOT          (not resolved)");
            return;
        }
    }
    println!("  local bin          {}", b.local_bin);
    println!("  cargo bin          {}", b.cargo_bin);
    if b.ok() {
        println!("  \x1b[1;32m✓\x1b[0m known FlexNetOS tools resolve inside META_ROOT");
        return;
    }
    println!(
        "  \x1b[1;31m✗\x1b[0m {} out-of-bound tool install(s) found",
        b.violations.len()
    );
    for v in &b.violations {
        println!(
            "    {:<14} {:?}: {} -> {}",
            v.tool, v.kind, v.path, v.resolved_path
        );
    }
    println!("    → envctl install meta-tool-links");
}

#[cfg(test)]
mod frontend_gaps_tests {
    //! TASK-0019: CLI parse/render coverage for the kasetto front-end gap port — global options,
    //! completions, the `self` tree, `--frozen` aliases, and `agent doctor`.
    use super::{Cli, Cmd, ColorMode, SelfAction};
    use clap::{CommandFactory, Parser};
    use clap_complete::{generate, Shell};

    // --- Item 2: completions <shell> generates non-empty scripts for each shell, exit 0 ---
    #[test]
    fn completions_generate_nonempty_for_each_shell() {
        for shell in [Shell::Bash, Shell::Zsh, Shell::Fish, Shell::PowerShell] {
            let mut cmd = Cli::command();
            let mut buf: Vec<u8> = Vec::new();
            generate(shell, &mut cmd, "envctl", &mut buf);
            assert!(!buf.is_empty(), "{shell:?} completion was empty");
            let text = String::from_utf8_lossy(&buf);
            assert!(
                text.contains("envctl"),
                "{shell:?} script lacks the bin name"
            );
        }
    }

    #[test]
    fn completions_subcommand_parses_a_shell_positional() {
        let cli = Cli::try_parse_from(["envctl", "completions", "fish"]).expect("parse");
        match cli.cmd {
            Cmd::Completions { shell } => assert_eq!(shell, Shell::Fish),
            _ => panic!("expected completions"),
        }
    }

    // --- Item 3: --frozen aliases --locked on the four agent verbs ---
    #[test]
    fn frozen_aliases_locked_on_sync() {
        let cli = Cli::try_parse_from(["envctl", "agent", "sync", "--frozen"]).expect("parse");
        match cli.cmd {
            Cmd::Agent {
                cmd: super::AgentCmd::Sync { locked, .. },
            } => assert!(locked, "--frozen must set locked on sync"),
            _ => panic!("expected agent sync"),
        }
    }

    #[test]
    fn frozen_aliases_locked_on_add_remove() {
        for verb in ["add", "remove"] {
            let cli =
                Cli::try_parse_from(["envctl", "agent", verb, "src", "--frozen"]).expect("parse");
            let locked = match cli.cmd {
                Cmd::Agent {
                    cmd: super::AgentCmd::Add { locked, .. },
                } => locked,
                Cmd::Agent {
                    cmd: super::AgentCmd::Remove { locked, .. },
                } => locked,
                _ => panic!("expected agent {verb}"),
            };
            assert!(locked, "--frozen must set locked on {verb}");
        }
    }

    #[test]
    fn frozen_aliases_check_on_lock() {
        // `lock` exposes --check with the visible alias `frozen` (envctl keeps a distinct
        // `--locked` zero-network flag, so `locked` is NOT a `--check` alias here).
        for flag in ["--check", "--frozen"] {
            let cli = Cli::try_parse_from(["envctl", "agent", "lock", flag]).expect("parse");
            match cli.cmd {
                Cmd::Agent {
                    cmd: super::AgentCmd::Lock { check, .. },
                } => assert!(check, "{flag} must set check on lock"),
                _ => panic!("expected agent lock"),
            }
        }
    }

    // --- Item 7: global options parse with repeat counts + color modes + init -f ---
    #[test]
    fn quiet_and_verbose_count_repeats() {
        let cli = Cli::try_parse_from(["envctl", "-qq", "auto-detect"]).expect("parse");
        assert_eq!(cli.quiet, 2);
        let cli = Cli::try_parse_from(["envctl", "-vvv", "auto-detect"]).expect("parse");
        assert_eq!(cli.verbose, 3);
    }

    #[test]
    fn color_mode_parses_each_value() {
        let cli = Cli::try_parse_from(["envctl", "--color", "always", "auto-detect"]).expect("p");
        assert_eq!(cli.color, ColorMode::Always);
        let cli = Cli::try_parse_from(["envctl", "--color", "never", "auto-detect"]).expect("p");
        assert_eq!(cli.color, ColorMode::Never);
        let cli = Cli::try_parse_from(["envctl", "auto-detect"]).expect("p");
        assert_eq!(cli.color, ColorMode::Auto);
    }

    #[test]
    fn no_color_flag_parses() {
        let cli = Cli::try_parse_from(["envctl", "--no-color", "auto-detect"]).expect("parse");
        assert!(cli.no_color);
    }

    #[test]
    fn agent_init_force_has_short_f() {
        let cli = Cli::try_parse_from(["envctl", "agent", "init", "-f"]).expect("parse");
        match cli.cmd {
            Cmd::Agent {
                cmd: super::AgentCmd::Init { force, .. },
            } => assert!(force, "-f must set force on agent init"),
            _ => panic!("expected agent init"),
        }
    }

    // --- resolve_plain side effects (kasetto port) ---
    #[test]
    fn resolve_plain_never_is_plain() {
        assert!(super::resolve_plain(false, ColorMode::Never));
    }

    #[test]
    fn resolve_plain_always_sets_clicolor_force() {
        std::env::remove_var("CLICOLOR_FORCE");
        let plain = super::resolve_plain(false, ColorMode::Always);
        assert!(!plain, "--color always alone is not plain");
        assert_eq!(std::env::var("CLICOLOR_FORCE").as_deref(), Ok("1"));
        std::env::remove_var("CLICOLOR_FORCE");
    }

    // --- Item 4/5: self tree parses; uninstall flags map ---
    #[test]
    fn self_update_parses_with_json() {
        let cli = Cli::try_parse_from(["envctl", "self", "update", "--json"]).expect("parse");
        match cli.cmd {
            Cmd::Manage {
                action: SelfAction::Update { json },
            } => assert!(json),
            _ => panic!("expected self update"),
        }
    }

    #[test]
    fn self_uninstall_parses_apply_yes() {
        let cli =
            Cli::try_parse_from(["envctl", "self", "uninstall", "--apply", "--yes"]).expect("p");
        match cli.cmd {
            Cmd::Manage {
                action: SelfAction::Uninstall { apply, yes },
            } => {
                assert!(apply);
                assert!(yes);
            }
            _ => panic!("expected self uninstall"),
        }
    }

    #[test]
    fn self_uninstall_defaults_to_preview() {
        // No flags = preview (apply false), no --yes.
        let cli = Cli::try_parse_from(["envctl", "self", "uninstall"]).expect("parse");
        match cli.cmd {
            Cmd::Manage {
                action: SelfAction::Uninstall { apply, yes },
            } => {
                assert!(!apply, "uninstall must default to preview (no --apply)");
                assert!(!yes);
            }
            _ => panic!("expected self uninstall"),
        }
    }

    // --- Item 1: agent doctor parses with --scope ---
    #[test]
    fn agent_doctor_parses_with_scope() {
        let cli =
            Cli::try_parse_from(["envctl", "agent", "doctor", "--scope", "global"]).expect("parse");
        match cli.cmd {
            Cmd::Agent {
                cmd: super::AgentCmd::Doctor { scope },
            } => assert!(scope.is_some()),
            _ => panic!("expected agent doctor"),
        }
    }

    // --- kasetto help-text port: rich --help presentation (long_about + Examples) ---

    /// Render the long help of a `clap::Command` to a `String` (introspection, no spawn).
    fn long_help(cmd: &mut clap::Command) -> String {
        let mut buf: Vec<u8> = Vec::new();
        cmd.write_long_help(&mut buf).expect("render long help");
        String::from_utf8_lossy(&buf).into_owned()
    }

    /// Walk the whole command tree (root + every subcommand, recursing into `agent` / `self`),
    /// applying `f` to each command. The visitor takes a mutable clone so it can render help.
    fn for_each_command(f: &mut impl FnMut(&str, &mut clap::Command)) {
        fn walk(cmd: &clap::Command, path: &str, f: &mut impl FnMut(&str, &mut clap::Command)) {
            for sub in cmd.get_subcommands() {
                let name = sub.get_name().to_string();
                let full = if path.is_empty() {
                    name.clone()
                } else {
                    format!("{path} {name}")
                };
                let mut owned = sub.clone();
                f(&full, &mut owned);
                walk(sub, &full, f);
            }
        }
        let root = Cli::command();
        let mut owned_root = root.clone();
        f("envctl", &mut owned_root);
        walk(&root, "", f);
    }

    /// The clap tree must be internally consistent (no conflicting flags, valid help, etc.).
    #[test]
    fn help_tree_builds() {
        Cli::command().debug_assert();
    }

    /// No kasetto/upstream branding may leak into any user-facing help string.
    #[test]
    fn no_kasetto_branding_in_help() {
        let banned = ["kasetto", "kst ", "KASETTO", "pivoshenko", "kasetto.yaml"];
        for_each_command(&mut |path, cmd| {
            let help = long_help(cmd);
            for needle in banned {
                assert!(
                    !help.contains(needle),
                    "`{path}` long help leaks branding {needle:?}"
                );
            }
        });
    }

    /// Examples must use envctl's real flags, not kasetto's: `--kind` (not `--type`),
    /// `--apply` on the destructive uninstall, `agent-env.yaml` (not kasetto.yaml), `--scope`.
    #[test]
    fn examples_use_envctl_flags() {
        let mut agent_list = Cli::command()
            .find_subcommand("agent")
            .and_then(|a| a.find_subcommand("list"))
            .cloned()
            .expect("agent list");
        let list_help = long_help(&mut agent_list);
        assert!(
            list_help.contains("--kind"),
            "agent list help must show envctl's --kind filter"
        );
        assert!(
            !list_help.contains("--type"),
            "agent list help must NOT use kasetto's --type filter"
        );

        let mut uninstall = Cli::command()
            .find_subcommand("self")
            .and_then(|s| s.find_subcommand("uninstall"))
            .cloned()
            .expect("self uninstall");
        let uninstall_help = long_help(&mut uninstall);
        assert!(
            uninstall_help.contains("--apply"),
            "self uninstall examples must teach the fail-closed --apply flag"
        );

        let mut agent_add = Cli::command()
            .find_subcommand("agent")
            .and_then(|a| a.find_subcommand("add"))
            .cloned()
            .expect("agent add");
        let add_help = long_help(&mut agent_add);
        assert!(
            add_help.contains("agent-env.yaml"),
            "an agent example must reference envctl's agent-env.yaml config"
        );

        let mut agent_lock = Cli::command()
            .find_subcommand("agent")
            .and_then(|a| a.find_subcommand("lock"))
            .cloned()
            .expect("agent lock");
        let lock_help = long_help(&mut agent_lock);
        assert!(
            lock_help.contains("--scope"),
            "scope must be expressed via --scope (envctl's flag)"
        );
    }

    /// Every command (root + each subcommand, recursing into agent + self) must carry a
    /// non-empty long_about and an after_help Examples block.
    #[test]
    fn every_command_has_long_about_and_examples() {
        for_each_command(&mut |path, cmd| {
            let long_about = cmd
                .get_long_about()
                .map(|s| s.to_string())
                .unwrap_or_default();
            assert!(
                !long_about.trim().is_empty(),
                "`{path}` is missing a non-empty long_about"
            );
            let after = cmd
                .get_after_help()
                .map(|s| s.to_string())
                .unwrap_or_default();
            assert!(
                after.contains("Examples:"),
                "`{path}` is missing an after_help Examples: block"
            );
        });
    }
}
