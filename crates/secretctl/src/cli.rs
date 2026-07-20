//! The `env-ctl` command tree (clap derive). The surface mirrors `docs/SCAFFOLD-SPEC.md`.
//! Destructive verbs carry `--apply` (default dry-run, CF-8); root-of-trust verbs also `--confirm`.
use clap::{Args, Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(
    name = "env-ctl",
    about = "env-ctl — local secrets vault + credential broker"
)]
pub struct Cli {
    /// Emit machine-readable NDJSON instead of pretty output.
    #[arg(long, global = true)]
    pub json: bool,
    /// Override the daemon control socket path.
    #[arg(long, global = true)]
    pub socket: Option<String>,
    #[command(subcommand)]
    pub cmd: Cmd,
}

#[derive(Subcommand, Debug)]
pub enum Cmd {
    /// Internal, unattended bootstrap for sqld's Ed25519 verification key and matching client JWT.
    /// This surface is intentionally hidden: the envctl sqld component owns its invocation.
    #[command(name = "internal-sqld-auth-bootstrap", hide = true)]
    InternalSqldAuthBootstrap {
        /// Destination for sqld's SubjectPublicKeyInfo PUBLIC KEY PEM.
        #[arg(long = "public-key")]
        public_key: std::path::PathBuf,
        /// Destination for the matching EdDSA client JWT.
        #[arg(long = "client-token")]
        client_token: std::path::PathBuf,
    },
    /// Internal bounded readiness barrier for the envctl-owned sqld user service.
    /// It binds the listener to systemd's MainPID, proves auth is enforced, and then proves the
    /// managed bearer can execute SQL. The bearer is read only from a 0600 file.
    #[command(name = "internal-sqld-readiness-probe", hide = true)]
    InternalSqldReadinessProbe {
        /// systemd's MainPID for sqld.service.
        #[arg(long)]
        pid: u32,
        /// Canonical envctl-managed sqld payload expected at /proc/PID/exe.
        #[arg(long = "expected-executable")]
        expected_executable: std::path::PathBuf,
        /// Loopback TCP port owned by that PID.
        #[arg(long, default_value_t = 8080)]
        port: u16,
        /// Current-user-owned 0600 file containing the matching JWT.
        #[arg(long = "client-token")]
        client_token: std::path::PathBuf,
        /// Current-user-owned 0600 SHA-256 record for this exact helper executable.
        #[arg(long = "helper-digest")]
        helper_digest: std::path::PathBuf,
        /// Hard upper bound for listener/auth readiness.
        #[arg(long = "timeout-seconds", default_value_t = 20)]
        timeout_seconds: u64,
    },
    /// Machine-stable identity used by the sqld component after proving this helper's bytes.
    #[command(name = "internal-sqld-readiness-contract", hide = true)]
    InternalSqldReadinessContract {
        /// Current-user-owned 0600 SHA-256 record for this exact helper executable.
        #[arg(long = "helper-digest")]
        helper_digest: std::path::PathBuf,
    },
    /// Write a no-replace 0600 SHA-256 record for this helper. Used only while staging the
    /// component-owned helper and by the real pinned-sqld test runner.
    #[command(name = "internal-sqld-self-digest", hide = true)]
    InternalSqldSelfDigest {
        #[arg(long)]
        output: std::path::PathBuf,
        /// Final absolute executable path to record when the running helper is a staging file.
        #[arg(long = "installed-path")]
        installed_path: Option<std::path::PathBuf>,
    },
    /// Verify a regular current-user-owned file against an expected SHA-256 without external tools.
    #[command(name = "internal-sqld-verify-sha256", hide = true)]
    InternalSqldVerifySha256 {
        #[arg(long)]
        file: std::path::PathBuf,
        #[arg(long = "expected-sha256")]
        expected_sha256: String,
        /// Required file mode, expressed as four octal digits (for example 0755 or 0600).
        #[arg(long = "expected-mode")]
        expected_mode: String,
    },
    /// Verify an installed helper file against its current-user-owned 0600 path-bound digest.
    #[command(name = "internal-sqld-verify-owned-digest", hide = true)]
    InternalSqldVerifyOwnedDigest {
        #[arg(long)]
        file: std::path::PathBuf,
        #[arg(long)]
        digest: std::path::PathBuf,
    },
    /// Require an installed helper to match both its record and this freshly built verifier.
    #[command(name = "internal-sqld-verify-current-helper", hide = true)]
    InternalSqldVerifyCurrentHelper {
        #[arg(long)]
        file: std::path::PathBuf,
        #[arg(long)]
        digest: std::path::PathBuf,
    },
    /// Atomically exchange a complete staged helper generation with the active real directory.
    #[command(name = "internal-sqld-commit-helper-generation", hide = true)]
    InternalSqldCommitHelperGeneration {
        #[arg(long = "staged-dir")]
        staged_dir: std::path::PathBuf,
        #[arg(long = "current-dir")]
        current_dir: std::path::PathBuf,
    },
    /// Write the deterministic digest of checked-in inputs plus the pinned build toolchain bytes.
    #[command(name = "internal-sqld-source-digest", hide = true)]
    InternalSqldSourceDigest {
        #[arg(long = "source-root")]
        source_root: std::path::PathBuf,
        #[arg(long = "toolchain-file", required = true)]
        toolchain_files: Vec<std::path::PathBuf>,
        /// Compiler data roots (for example clang's built-in headers) consumed by the build.
        #[arg(long = "toolchain-root", required = true)]
        toolchain_roots: Vec<std::path::PathBuf>,
        /// Freshly staged, Cargo.lock-checksummed `.crate` archives used by this build.
        #[arg(long = "crate-archive", required = true)]
        crate_archives: Vec<std::path::PathBuf>,
        #[arg(long)]
        output: std::path::PathBuf,
    },
    /// Compare checked-in inputs and pinned toolchain bytes with a managed 0600 digest.
    #[command(name = "internal-sqld-verify-source-digest", hide = true)]
    InternalSqldVerifySourceDigest {
        #[arg(long = "source-root")]
        source_root: std::path::PathBuf,
        #[arg(long = "toolchain-file", required = true)]
        toolchain_files: Vec<std::path::PathBuf>,
        /// Compiler data roots (for example clang's built-in headers) consumed by the build.
        #[arg(long = "toolchain-root", required = true)]
        toolchain_roots: Vec<std::path::PathBuf>,
        /// Freshly staged, Cargo.lock-checksummed `.crate` archives used by this build.
        #[arg(long = "crate-archive", required = true)]
        crate_archives: Vec<std::path::PathBuf>,
        #[arg(long)]
        digest: std::path::PathBuf,
    },
    /// Vault lock status (no unlock side effect).
    Status,
    /// Initialize a fresh vault: mint the DEK + enroll keyslots. Dry-run preview unless `--apply`;
    /// REFUSES to overwrite an existing vault. The daemon forces the hardened Argon2 floor.
    Init {
        /// Read the passphrase from stdin (owner-only, over the peercred-gated socket).
        #[arg(long)]
        passphrase_stdin: bool,
        /// Also enroll a USB keyslot (requires `--usb-partuuid`; keyfile read on the daemon side).
        #[arg(long)]
        enroll_usb: bool,
        /// PARTUUID of the USB partition holding the keyfile (the slot selector; not the key).
        #[arg(long = "usb-partuuid")]
        usb_partuuid: Option<String>,
        /// Actually initialize. Without it, prints a dry-run preview and mutates nothing (CF-8).
        #[arg(long)]
        apply: bool,
    },
    /// Unlock the vault (USB-first; passphrase only if the USB is absent).
    Unlock {
        #[arg(long)]
        passphrase_stdin: bool,
    },
    /// Zeroize the DEK + CA issuer in RAM (the true panic stop).
    Lock,
    /// Manage stored secrets.
    Secret {
        #[command(subcommand)]
        cmd: SecretCmd,
    },
    /// Manage relay policies + mint bearers.
    Relay {
        #[command(subcommand)]
        cmd: RelayCmd,
    },
    /// Manage the local CA, leaf certs, and trust wiring.
    Ca {
        #[command(subcommand)]
        cmd: CaCmd,
    },
    /// Query the tamper-evident audit log.
    Audit(AuditArgs),
    /// Run a command with relay credentials injected into the child only.
    Run(RunArgs),
    /// Mint a GitHub App installation access token from the vault-sealed App key (TASK-0020). This is
    /// the FROZEN consumer-contract surface `flexnetos_github_app` shells: `--output json` prints
    /// EXACTLY `{"token":"...","expires_at_unix":<i64>}` to stdout (all logs to stderr).
    #[command(name = "mint-github")]
    MintGithub(MintGithubArgs),
    /// Enroll the GitHub App credential into the unlocked vault (TASK-0026): seal the App PEM as a
    /// broker-only secret + persist the non-secret App id, so `mint-github` can mint installation
    /// tokens. Dry-run preview unless `--apply`.
    #[command(name = "github-app")]
    GithubApp {
        #[command(subcommand)]
        cmd: GithubAppCmd,
    },
    /// Operator-box presence-token authorizer for SERVER-MODE Profile B (TASK-0033 / OI-SM-2). The
    /// OPERATOR-box role: `serve` signs presence tokens for a remote VPS over mTLS; `status` shows
    /// the local `[profile]` topology + whether the VPS authorizer link is configured.
    Authorizer {
        #[command(subcommand)]
        cmd: AuthorizerCmd,
    },
}

#[derive(Subcommand, Debug)]
pub enum AuthorizerCmd {
    /// Operator-box SIGNER: serve presence tokens to a remote VPS over mTLS. On each VPS challenge
    /// (instance id + server nonce + cert fp) this signs a short-lived [`PresenceToken`] with the
    /// operator Ed25519 seed and returns it + the operator's attested trusted time. The seed is read
    /// from `--seed-file` (32 raw bytes or 64-hex); it NEVER leaves this process and is zeroized.
    Serve {
        /// Bind address for the operator-box authorizer (e.g. `0.0.0.0:9443`).
        #[arg(long = "bind")]
        bind: String,
        /// Path to the operator Ed25519 signing seed (32 raw bytes or 64-hex). The public half is
        /// what the VPS pins as `operator_pubkey_hex`.
        #[arg(long = "seed-file")]
        seed_file: String,
        /// PEM path of this operator box's TLS server cert (presented to the VPS).
        #[arg(long = "cert")]
        cert: String,
        /// PEM path of this operator box's TLS server key.
        #[arg(long = "key")]
        key: String,
        /// PEM path of the CA the VPS client cert is verified against (mTLS; required — the signer
        /// fails closed without a client-CA, it never signs for an unauthenticated VPS).
        #[arg(long = "client-ca")]
        client_ca: String,
        /// Token TTL in seconds (clamped to the audited 5–15 min band; default 600 = 10 min).
        #[arg(long = "ttl-secs", default_value_t = 600)]
        ttl_secs: i64,
    },
    /// Show the local `[profile]` topology (on-box vs VPS) and, for a VPS, whether the operator
    /// authorizer link is configured. A read-only local config inspection (no daemon round-trip).
    Status,
}

#[derive(Subcommand, Debug)]
pub enum GithubAppCmd {
    /// Enroll the GitHub App private key (PEM) + App id. The PEM is sealed broker-only (un-revealable)
    /// under `github-app-private-key`; the App id is persisted as the non-secret `github-app-id` meta.
    /// installation-id is NOT enrolled — it is supplied per mint. Dry-run preview unless `--apply`.
    Enroll {
        /// The GitHub App id (non-secret, e.g. `4044997`). Required.
        #[arg(long = "app-id")]
        app_id: String,
        /// Path to the App private-key PEM file, or `-` to read it from stdin. The bytes are validated
        /// (must be a usable RSA App key) BEFORE any write and are NEVER printed.
        #[arg(long = "private-key")]
        private_key: String,
        /// Actually enroll. Without it, prints a dry-run preview (to stderr) and writes nothing (CF-8).
        #[arg(long)]
        apply: bool,
    },
    /// Set (or refresh) ONLY the non-secret `github-app-id` meta that `mint-github` reads, WITHOUT
    /// touching the sealed PEM. Heals an enrollment whose App key is already sealed under
    /// `github-app-private-key` but whose `github-app-id` meta is absent — the exact state that makes
    /// `mint-github` fail "GitHub App id not enrolled" even though the key is present (e.g. an App
    /// sealed by an older enroll path / a meta-schema drift). Requires the vault Unlocked. Dry-run
    /// preview unless `--apply`.
    SetAppId {
        /// The GitHub App id (non-secret, e.g. `4044997`). Required.
        #[arg(long = "app-id")]
        app_id: String,
        /// Actually persist the id meta. Without it, prints a dry-run preview (to stderr) and writes nothing.
        #[arg(long)]
        apply: bool,
    },
    /// Early-revoke a GitHub installation access token via `DELETE /installation/token` (TASK-0027).
    /// The token authenticates the revoke ITSELF (it is the kill-switch for an outstanding token —
    /// e.g. one already handed off to a child tool). Dry-run preview unless `--apply`. The token is
    /// NEVER printed in any mode.
    RevokeToken {
        /// The installation token to revoke, OR `-` to read it from stdin, OR an `@path` file. The
        /// bytes are NEVER printed. Recommended: `--token -` (avoids leaking it into argv/ps).
        #[arg(long = "token")]
        token: String,
        /// The installation id this token belongs to (optional, metadata-only; aids the audit trail).
        #[arg(long = "installation-id")]
        installation_id: Option<u64>,
        /// Actually revoke. Without it, prints a dry-run preview (to stderr) and contacts nothing.
        #[arg(long)]
        apply: bool,
    },
}

#[derive(Args, Debug)]
pub struct MintGithubArgs {
    /// The GitHub App installation id to mint a token for (required).
    #[arg(long = "installation-id")]
    pub installation_id: u64,
    /// Numeric repository IDs to scope the token to, comma-separated (e.g. `--repository-ids 10,20`).
    /// Omitted ⇒ the installation's full default repository scope.
    #[arg(long = "repository-ids", value_delimiter = ',')]
    pub repository_ids: Vec<String>,
    /// Least-privilege permissions, comma-separated `name:access` (e.g. `--permissions checks:write,contents:read`).
    /// Omitted ⇒ the installation's full default permission scope.
    #[arg(long = "permissions", value_delimiter = ',')]
    pub permissions: Vec<String>,
    /// Requested token lifetime in seconds (required; advisory — GitHub fixes the RAW installation
    /// token at ~1h, but the relay re-mints + re-injects it on a ≤24h rotation policy).
    #[arg(long = "ttl-secs")]
    pub ttl_secs: i64,
    /// Output format. Only `json` is supported (the frozen machine contract). Required.
    #[arg(long = "output")]
    pub output: String,
}

#[derive(Subcommand, Debug)]
pub enum SecretCmd {
    /// Add a secret (additive; backs up on overwrite).
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
        /// The real key is broker-only: `get --reveal` will refuse it.
        #[arg(long)]
        broker_only: bool,
    },
    /// Show metadata; the raw value only with `--reveal --apply` (audited; refused if broker-only).
    Get {
        name: String,
        #[arg(long)]
        reveal: bool,
        #[arg(long)]
        apply: bool,
        #[arg(long)]
        confirm: bool,
    },
    /// List secrets (metadata only).
    List {
        #[arg(long)]
        provider: Option<String>,
    },
    /// Remove a secret (destructive; dry-run unless `--apply`).
    Rm {
        name: String,
        #[arg(long)]
        apply: bool,
        #[arg(long)]
        confirm: bool,
    },
    /// Rotate a secret's value (destructive; dry-run unless `--apply`).
    Rotate {
        name: String,
        #[arg(long)]
        value_stdin: bool,
        #[arg(long)]
        apply: bool,
    },
}

#[derive(Subcommand, Debug)]
pub enum RelayCmd {
    /// Create a named relay policy (additive).
    Create {
        name: String,
        #[arg(long)]
        secret: String,
        #[arg(long)]
        provider: String,
        /// base-url | proxy | native
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
    /// Revoke a relay policy (destructive; dry-run unless `--apply`).
    Revoke {
        name: String,
        #[arg(long)]
        apply: bool,
        #[arg(long)]
        confirm: bool,
    },
    /// Revoke a single leaked bearer by its token id (OI-10).
    RevokeToken {
        token_id: String,
        #[arg(long)]
        apply: bool,
    },
    /// List relay policies.
    List {
        #[arg(long)]
        all: bool,
    },
    /// Mint a `<=24h` peer-bound bearer under a policy (USB-gated). With `--mode native --provider
    /// github` mints a native GitHub App installation token (TTL fixed ~1h by GitHub) instead.
    Mint {
        name: String,
        #[arg(long)]
        ttl: Option<String>,
        /// Data plane: base-url | proxy | native (default base-url).
        #[arg(long)]
        mode: Option<String>,
        /// Provider: anthropic | openai | github | generic (default generic; `github` for native mint).
        #[arg(long)]
        provider: Option<String>,
        /// Repository to scope a `--mode native` GitHub mint to (repeatable). Empty ⇒ all installed.
        #[arg(long = "repo")]
        repos: Vec<String>,
        /// Permission to scope a `--mode native` GitHub mint (`name:access`, repeatable). Defaults to
        /// `["checks:write"]` for `--mode native --provider github`. Empty ⇒ full installation scope.
        #[arg(long = "perm")]
        perms: Vec<String>,
    },
}

#[derive(Subcommand, Debug)]
pub enum CaCmd {
    /// Initialize the local CA.
    Init {
        #[arg(long)]
        apply: bool,
    },
    /// Rotate the CA (root-of-trust: `--apply --confirm`).
    Rotate {
        #[arg(long)]
        apply: bool,
        #[arg(long)]
        confirm: bool,
    },
    /// Issue a leaf cert. `--usage` is control-server | control-client (NEVER mitm-leaf).
    Issue {
        cn: String,
        #[arg(long = "san")]
        sans: Vec<String>,
        #[arg(long)]
        ttl_days: Option<u64>,
        #[arg(long)]
        usage: String,
    },
    /// List the local CA and persisted leaf certs.
    List,
    /// Renew a leaf cert.
    Renew {
        cn: String,
        #[arg(long)]
        apply: bool,
    },
    /// Revoke a leaf cert (destructive; dry-run unless `--apply`).
    Revoke {
        cn: String,
        #[arg(long)]
        apply: bool,
        #[arg(long)]
        confirm: bool,
    },
    /// Wire CA trust into tool env / the system bundle (reversible, owned-file-only).
    Trust {
        targets: Vec<String>,
        /// Root-of-trust: requires `--apply --confirm`.
        #[arg(long)]
        system_bundle: bool,
        #[arg(long)]
        apply: bool,
        #[arg(long)]
        confirm: bool,
    },
}

#[derive(Args, Debug)]
pub struct AuditArgs {
    #[arg(long)]
    pub actor: Option<String>,
    #[arg(long)]
    pub relay: Option<String>,
    #[arg(long)]
    pub since: Option<String>,
    #[arg(long)]
    pub until: Option<String>,
    #[arg(long)]
    pub limit: Option<u32>,
}

#[derive(Args, Debug)]
pub struct RunArgs {
    /// Attach one or more named relays (else inferred from a profile / provider).
    #[arg(long = "relay")]
    pub relays: Vec<String>,
    #[arg(long)]
    pub provider: Option<String>,
    /// Mint a one-off ephemeral bearer for this process.
    #[arg(long)]
    pub ephemeral: bool,
    #[arg(long = "no-profile")]
    pub no_profile: bool,
    #[arg(long)]
    pub profile: Option<String>,
    /// The command to run: `env-ctl run -- <cmd> [args...]`.
    #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
    pub argv: Vec<String>,
}
