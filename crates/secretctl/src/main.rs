//! secretctl — the `env-ctl` CLI. A thin gRPC client over the daemon's Unix socket; it drains the
//! `Event` stream and pretty-prints (or `--json`). Destructive verbs default to dry-run (`--apply`
//! to act, `--confirm` for root-of-trust). The bearer/value printing is owner-only and only on the
//! peercred-gated channel; the real key is never printed (the daemon never sends it).
mod authorizer;
mod cli;
mod render;

use std::io::Read;
use std::path::PathBuf;

use anyhow::Context;
use clap::Parser;
use cli::{CaCmd, Cli, Cmd, GithubAppCmd, MintGithubArgs, RelayCmd, SecretCmd};
use envctl_secrets_proto::v1;
use hyper_util::rt::TokioIo;
use tonic::transport::{Endpoint, Uri};
use tonic::Streaming;

type VaultClient = v1::vault_client::VaultClient<tonic::transport::Channel>;
type RelayClient = v1::relay_client::RelayClient<tonic::transport::Channel>;
type LockClient = v1::lock_client::LockClient<tonic::transport::Channel>;
type AuditClient = v1::audit_client::AuditClient<tonic::transport::Channel>;
type CertsClient = v1::certs_client::CertsClient<tonic::transport::Channel>;

fn main() -> anyhow::Result<()> {
    let args = Cli::parse();
    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .context("building the tokio runtime")?;
    rt.block_on(run(args))
}

/// Resolve the control socket: `--socket` override, else `$XDG_RUNTIME_DIR/env-ctl/secretd.sock`.
/// secretctl does NOT depend on the engine, so this path is recomputed inline (mirrors
/// `Paths::resolve().control_socket()`).
fn socket_path(override_path: &Option<String>) -> anyhow::Result<PathBuf> {
    if let Some(p) = override_path {
        return Ok(PathBuf::from(p));
    }
    let runtime = match std::env::var_os("XDG_RUNTIME_DIR") {
        Some(r) => PathBuf::from(r).join("env-ctl"),
        None => {
            // Fall back to the XDG state dir, matching the engine's Paths::resolve fallback.
            let home = std::env::var_os("HOME")
                .map(PathBuf::from)
                .ok_or_else(|| anyhow::anyhow!("neither XDG_RUNTIME_DIR nor HOME is set"))?;
            std::env::var_os("XDG_STATE_HOME")
                .map(PathBuf::from)
                .unwrap_or_else(|| home.join(".local/state"))
                .join("env-ctl")
        }
    };
    Ok(runtime.join("secretd.sock"))
}

/// Connect a tonic `Channel` to the daemon over the UDS. tonic's `Channel` cannot bind a UDS
/// directly, so we use the classic `service_fn` connector: the URI is ignored; every connection is
/// a fresh `UnixStream` to `sock`, wrapped in `TokioIo` to satisfy hyper's IO traits.
async fn connect(sock: PathBuf) -> anyhow::Result<tonic::transport::Channel> {
    // The scheme/authority are placeholders; the connector ignores them and dials the UDS.
    let endpoint = Endpoint::try_from("http://[::]:0").context("building the endpoint")?;
    let channel = endpoint
        .connect_with_connector(tower::service_fn(move |_: Uri| {
            let sock = sock.clone();
            async move {
                let stream = tokio::net::UnixStream::connect(sock).await?;
                Ok::<_, std::io::Error>(TokioIo::new(stream))
            }
        }))
        .await
        .context("connecting to the daemon socket (is secretd running?)")?;
    Ok(channel)
}

fn provider_to_proto(s: &str) -> i32 {
    let k = match s.to_ascii_lowercase().as_str() {
        "anthropic" => v1::ProviderKind::Anthropic,
        "openai" => v1::ProviderKind::Openai,
        "github" => v1::ProviderKind::Github,
        "generic" => v1::ProviderKind::Generic,
        _ => v1::ProviderKind::Generic,
    };
    k as i32
}

fn mode_to_proto(s: &str) -> i32 {
    let m = match s.to_ascii_lowercase().as_str() {
        "base-url" | "base_url" | "baseurl" => v1::DataPlaneMode::BaseUrlRepoint,
        "proxy" | "mitm" => v1::DataPlaneMode::HttpsProxyMitm,
        "native" | "subtoken" => v1::DataPlaneMode::NativeSubtoken,
        _ => v1::DataPlaneMode::BaseUrlRepoint,
    };
    m as i32
}

fn read_stdin_string() -> anyhow::Result<String> {
    let mut s = String::new();
    std::io::stdin().read_to_string(&mut s)?;
    Ok(s.trim_end_matches(['\n', '\r']).to_string())
}

fn read_stdin_bytes() -> anyhow::Result<Vec<u8>> {
    let mut v = Vec::new();
    std::io::stdin().read_to_end(&mut v)?;
    Ok(v)
}

/// Drain a server-streamed `Event` response, rendering each event.
async fn drain(mut stream: Streaming<v1::Event>, json: bool) -> anyhow::Result<()> {
    while let Some(ev) = stream.message().await? {
        render::render_event(&ev, json);
    }
    Ok(())
}

async fn run(args: Cli) -> anyhow::Result<()> {
    let sock = socket_path(&args.socket)?;
    let json = args.json;

    match args.cmd {
        Cmd::Status => {
            let mut c = LockClient::new(connect(sock).await?);
            let r = c.status(v1::StatusReq {}).await?.into_inner();
            render::render_status(&r, json);
        }
        Cmd::Init {
            passphrase_stdin,
            enroll_usb,
            usb_partuuid,
            apply,
        } => {
            // `--usb-partuuid` is required when enrolling a USB keyslot (the daemon also re-checks,
            // fail-closed). Catch it client-side for a friendlier error.
            if enroll_usb
                && usb_partuuid
                    .as_deref()
                    .map(str::trim)
                    .unwrap_or("")
                    .is_empty()
            {
                anyhow::bail!("--enroll-usb requires --usb-partuuid <UUID>");
            }
            let passphrase = if passphrase_stdin {
                Some(read_stdin_string()?)
            } else {
                None
            };
            let mut c = VaultClient::new(connect(sock).await?);
            let stream = c
                .init(v1::InitReq {
                    passphrase,
                    enroll_usb,
                    usb_partition_uuid: usb_partuuid.unwrap_or_default(),
                    apply,
                })
                .await?
                .into_inner();
            drain(stream, json).await?;
        }
        Cmd::Unlock { passphrase_stdin } => {
            let passphrase = if passphrase_stdin {
                Some(read_stdin_string()?)
            } else {
                None
            };
            let mut c = LockClient::new(connect(sock).await?);
            let stream = c.unlock(v1::UnlockReq { passphrase }).await?.into_inner();
            drain(stream, json).await?;
        }
        Cmd::Lock => {
            let mut c = LockClient::new(connect(sock).await?);
            let stream = c.lock_now(v1::LockReq {}).await?.into_inner();
            drain(stream, json).await?;
        }
        Cmd::Secret { cmd } => secret(cmd, sock, json).await?,
        Cmd::Relay { cmd } => relay(cmd, sock, json).await?,
        Cmd::Ca { cmd } => ca(cmd, sock, json).await?,
        Cmd::Audit(a) => {
            let mut c = AuditClient::new(connect(sock).await?);
            let req = v1::AuditQueryReq {
                actor: a.actor,
                relay: a.relay,
                since: a.since,
                until: a.until,
                limit: a.limit.unwrap_or(0),
            };
            let r = c.query(req).await?.into_inner();
            render::render_audit(&r, json);
        }
        Cmd::Run(a) => run_child_cmd(a, sock, json).await?,
        Cmd::MintGithub(a) => mint_github(a, sock).await?,
        Cmd::GithubApp { cmd } => github_app(cmd, sock, json).await?,
        Cmd::Authorizer { cmd } => authorizer::authorizer(cmd, json).await?,
    }
    Ok(())
}

/// Read the App private-key PEM from `--private-key` (a file path, or `-` for stdin) into a
/// `Zeroizing` buffer that is wiped on drop. The bytes never leave this buffer except over the
/// peercred-gated UDS in `AddSecretReq.value`; they are NEVER printed, logged, or echoed.
fn read_pem(source: &str) -> anyhow::Result<zeroize::Zeroizing<Vec<u8>>> {
    let bytes = if source == "-" {
        read_stdin_bytes().context("reading the App private-key PEM from stdin")?
    } else {
        std::fs::read(source)
            .with_context(|| format!("reading the App private-key PEM file '{source}'"))?
    };
    Ok(zeroize::Zeroizing::new(bytes))
}

/// `env-ctl github-app …` dispatch (TASK-0026 `enroll` + TASK-0027 `revoke-token`). Each verb is
/// fail-closed + dry-run by default; neither ever prints a credential.
async fn github_app(cmd: GithubAppCmd, sock: PathBuf, json: bool) -> anyhow::Result<()> {
    match cmd {
        GithubAppCmd::Enroll { .. } => github_app_enroll(cmd, sock, json).await,
        GithubAppCmd::SetAppId { .. } => github_app_set_app_id(cmd, sock, json).await,
        GithubAppCmd::RevokeToken { .. } => github_app_revoke_token(cmd, sock, json).await,
    }
}

/// `env-ctl github-app set-app-id` — persist ONLY the non-secret `github-app-id` meta that
/// `mint_github_token` reads, via `Vault.SetGithubAppId`, WITHOUT re-sealing (or needing) the PEM.
/// This heals an enrollment whose App key is already sealed under `github-app-private-key` but whose
/// id meta is missing (so `mint-github` fails "GitHub App id not enrolled"). The id is non-secret;
/// the daemon refuses when the vault is Locked. Dry-run preview to STDERR unless `--apply`.
async fn github_app_set_app_id(cmd: GithubAppCmd, sock: PathBuf, json: bool) -> anyhow::Result<()> {
    let GithubAppCmd::SetAppId { app_id, apply } = cmd else {
        unreachable!("github_app_set_app_id dispatched a non-SetAppId variant");
    };

    if app_id.trim().is_empty() {
        anyhow::bail!("--app-id must not be empty");
    }

    // DRY-RUN (default, CF-8): preview to STDERR, write nothing. The sealed App key is never touched.
    if !apply {
        eprintln!(
            "DRY-RUN: would persist meta `github-app-id`: {app_id} (writes nothing; the sealed App \
             private key under `github-app-private-key` is left untouched). Re-run with --apply."
        );
        return Ok(());
    }

    // APPLY: the same `Vault.SetGithubAppId` RPC the enroll path uses for its id step — minus the PEM.
    let mut c = VaultClient::new(connect(sock).await?);
    let set = v1::SetGithubAppIdReq {
        app_id: app_id.clone(),
        apply: true,
    };
    let set_stream = c.set_github_app_id(set).await?.into_inner();
    drain(set_stream, json).await?;

    Ok(())
}

/// `secretctl github-app revoke-token` (TASK-0027): early-revoke an outstanding GitHub installation
/// access token via the daemon's `Vault.RevokeGithubToken` RPC. Fail-closed + dry-run by default:
///   1. read the token (file / `-` stdin) into `Zeroizing`, refuse empty;
///   2. `--apply` absent ⇒ print a dry-run preview to STDERR and contact nothing (CF-8);
///   3. `--apply` ⇒ drive the RPC; under `--json` emit `{"revoked":<bool>,"dry_run":<bool>}` to
///      STDOUT, all human text to STDERR.
///
/// The token is NEVER printed in any mode (it lives only in `Zeroizing` and the RPC `bytes` field).
async fn github_app_revoke_token(
    cmd: GithubAppCmd,
    sock: PathBuf,
    json: bool,
) -> anyhow::Result<()> {
    let GithubAppCmd::RevokeToken {
        token,
        installation_id,
        apply,
    } = cmd
    else {
        unreachable!("github_app_revoke_token dispatched a non-RevokeToken variant");
    };

    // (1) Read the token into a Zeroizing buffer (wiped on drop). NEVER printed.
    let token_bytes = read_token(&token)?;
    if token_bytes.iter().all(|b| b.is_ascii_whitespace()) {
        anyhow::bail!("--token must not be empty");
    }

    // (2) DRY-RUN (default, CF-8): preview to STDERR, contact nothing.
    if !apply {
        eprintln!(
            "DRY-RUN: would early-revoke the supplied GitHub installation token via \
             DELETE /installation/token (contacts nothing). Re-run with --apply.\n  \
             - the token is sent ONLY as the revoke request's bearer; it is never printed."
        );
        if json {
            println!(
                "{}",
                serde_json::json!({ "revoked": false, "dry_run": true })
            );
        }
        return Ok(());
    }

    // (3) APPLY: drive the RPC. The token crosses ONLY as the RPC `bytes` field.
    let mut c = VaultClient::new(connect(sock).await?);
    let resp = c
        .revoke_github_token(v1::RevokeGithubTokenReq {
            token: token_bytes.to_vec(),
            apply: true,
            installation_id: installation_id.unwrap_or(0),
        })
        .await?
        .into_inner();
    let revoked = resp.count_revoked > 0;
    if json {
        println!(
            "{}",
            serde_json::json!({ "revoked": revoked, "dry_run": resp.dry_run })
        );
    } else {
        eprintln!("revoked: {} (dry_run: {})", revoked, resp.dry_run);
    }
    Ok(())
}

/// Read the installation token from `--token` (a file path, or `-` for stdin) into a `Zeroizing`
/// buffer wiped on drop. The bytes are NEVER printed; they cross ONLY as the RPC `bytes` field. A
/// trailing newline (common when piping) is trimmed.
fn read_token(source: &str) -> anyhow::Result<zeroize::Zeroizing<Vec<u8>>> {
    let mut bytes = if source == "-" {
        read_stdin_bytes().context("reading the installation token from stdin")?
    } else {
        std::fs::read(source)
            .with_context(|| format!("reading the installation token file '{source}'"))?
    };
    while bytes.last().is_some_and(|b| *b == b'\n' || *b == b'\r') {
        bytes.pop();
    }
    Ok(zeroize::Zeroizing::new(bytes))
}

/// `env-ctl github-app enroll` (TASK-0026): seal the GitHub App credential so `mint-github` can mint
/// installation tokens. Fail-closed + dry-run by default:
///   1. read the PEM (file or stdin) into `Zeroizing`;
///   2. VALIDATE it client-side BEFORE any write by building a throwaway App JWT — a non-PEM / bad
///      key bails here, so a malformed credential never touches the vault (and the PEM is never
///      echoed; only the validation error string is shown);
///   3. `--apply` absent ⇒ print a dry-run preview to STDERR and write nothing (CF-8);
///   4. `--apply` ⇒ `Vault.Add{ broker_only=true, overwrite=false }` (the PEM) THEN
///      `Vault.SetGithubAppId{ apply=true }` (the id) — PEM first so a failure leaves no orphan id.
///
/// The secret name + meta key are the engine's `pub const`s read by the mint, so they can never
/// drift. installation-id is NOT enrolled (it is supplied per mint).
async fn github_app_enroll(cmd: GithubAppCmd, sock: PathBuf, json: bool) -> anyhow::Result<()> {
    let GithubAppCmd::Enroll {
        app_id,
        private_key,
        apply,
    } = cmd
    else {
        unreachable!("github_app_enroll dispatched a non-Enroll variant");
    };

    if app_id.trim().is_empty() {
        anyhow::bail!("--app-id must not be empty");
    }

    // (1) Read the PEM into a Zeroizing buffer (wiped on drop). Never printed.
    let pem = read_pem(&private_key)?;

    // (2) Validate the PEM client-side BEFORE any write: a throwaway App JWT proves the key parses
    // (PKCS#1 or PKCS#8) and signs. On Err we bail WITHOUT echoing a single PEM byte. The JWT is
    // discarded immediately (never sent anywhere). `MAX_JWT_TTL_SECS` is the engine's own ceiling.
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);
    envctl_secrets::build_app_jwt(&app_id, now, envctl_secrets::MAX_JWT_TTL_SECS, &pem).map_err(
        |e| anyhow::anyhow!("the supplied --private-key is not a usable GitHub App RSA PEM: {e}"),
    )?;

    // The secret name + meta key are the engine consts the mint reader uses — referenced verbatim so
    // the enroll writer can NEVER drift from `mint_github_token`'s reader.
    let key_name = envctl_secrets::GITHUB_APP_KEY_NAME;

    // (3) DRY-RUN (default, CF-8): preview to STDERR (so a piped stdout consumer sees nothing), write
    // nothing. NEVER prints the PEM — only metadata.
    if !apply {
        eprintln!(
            "DRY-RUN: would enroll the GitHub App credential (writes nothing). Re-run with --apply.\n  \
             - secret `{key_name}`: the App private key, sealed broker_only=true (un-revealable: \
             `secret get --reveal` will refuse it)\n  \
             - meta `github-app-id`: {app_id}\n  \
             note: installation-id is supplied per-mint (NOT enrolled here)."
        );
        return Ok(());
    }

    // (4) APPLY: seal the PEM FIRST (no orphan app-id if this fails), then persist the App id.
    let mut c = VaultClient::new(connect(sock).await?);

    let add = v1::AddSecretReq {
        name: key_name.to_string(),
        provider: v1::ProviderKind::Github as i32,
        value: pem.to_vec(),
        note: "GitHub App private key (TASK-0026 github-app enroll)".to_string(),
        overwrite: false,
        broker_only: true,
    };
    let add_stream = c.add(add).await?.into_inner();
    drain(add_stream, json).await?;

    let set = v1::SetGithubAppIdReq {
        app_id: app_id.clone(),
        apply: true,
    };
    let set_stream = c.set_github_app_id(set).await?.into_inner();
    drain(set_stream, json).await?;

    Ok(())
}

/// TASK-0020 — the FROZEN `mint-github` consumer-contract surface. Calls `Vault.MintGithub` over the
/// daemon UDS and prints EXACTLY the compact two-field JSON `{"token":"...","expires_at_unix":<i64>}`
/// to **stdout** — nothing else. Any human/diagnostic output goes to stderr so the consumer's
/// `serde_json::from_slice` over stdout never sees a stray byte. `--output json` is the only format.
async fn mint_github(a: MintGithubArgs, sock: PathBuf) -> anyhow::Result<()> {
    if a.output != "json" {
        anyhow::bail!("--output must be 'json' (the only supported mint-github format)");
    }
    // The CLI carries `repository_ids` verbatim as strings (the daemon parses them to u64 and rejects
    // a non-numeric id at the boundary). `permissions` pass through verbatim (`name:access`).
    let req = v1::MintGithubReq {
        installation_id: a.installation_id,
        repository_ids: a.repository_ids,
        permissions: a.permissions,
        ttl_secs: a.ttl_secs,
    };
    let mut c = VaultClient::new(connect(sock).await?);
    let resp = c.mint_github(req).await?.into_inner();
    // Emit EXACTLY the frozen two-field shape, compactly, to stdout. We build the JSON `Value`
    // explicitly (NOT from the proto struct) so the contract is pinned to these two fields even if
    // `MintGithubResp` grows a field later. `serde_json` escapes the token string correctly.
    let out = serde_json::json!({
        "token": resp.token,
        "expires_at_unix": resp.expires_at_unix,
    });
    // `println!` writes the compact JSON + a trailing newline to STDOUT only. No other stdout writes.
    println!("{}", serde_json::to_string(&out)?);
    Ok(())
}

async fn secret(cmd: SecretCmd, sock: PathBuf, json: bool) -> anyhow::Result<()> {
    let mut c = VaultClient::new(connect(sock).await?);
    match cmd {
        SecretCmd::Add {
            name,
            provider,
            value_stdin,
            note,
            overwrite,
            broker_only,
        } => {
            let value = if value_stdin {
                read_stdin_bytes()?
            } else {
                anyhow::bail!("secret add requires --value-stdin");
            };
            let req = v1::AddSecretReq {
                name,
                provider: provider_to_proto(&provider),
                value,
                note: note.unwrap_or_default(),
                overwrite,
                broker_only,
            };
            let stream = c.add(req).await?.into_inner();
            drain(stream, json).await?;
        }
        SecretCmd::Get {
            name,
            reveal,
            apply,
            confirm,
        } => {
            let req = v1::GetSecretReq {
                name,
                reveal,
                apply,
                confirm,
            };
            let r = c.get(req).await?.into_inner();
            render::render_get(&r, json);
        }
        SecretCmd::List { provider } => {
            let req = v1::ListSecretReq {
                provider: provider.as_deref().map(provider_to_proto),
            };
            let r = c.list(req).await?.into_inner();
            for item in &r.items {
                if json {
                    println!(
                        "{}",
                        serde_json::json!({ "name": item.name, "version": item.version,
                            "broker_only": item.broker_only })
                    );
                } else {
                    println!(
                        "{} v{} broker_only={}",
                        item.name, item.version, item.broker_only
                    );
                }
            }
        }
        SecretCmd::Rm {
            name,
            apply,
            confirm,
        } => {
            let req = v1::RmSecretReq {
                name,
                apply,
                confirm,
            };
            let stream = c.rm(req).await?.into_inner();
            drain(stream, json).await?;
        }
        SecretCmd::Rotate {
            name,
            value_stdin,
            apply,
        } => {
            let new_value = if value_stdin {
                read_stdin_bytes()?
            } else {
                Vec::new()
            };
            let req = v1::RotateReq {
                name,
                new_value,
                apply,
            };
            let stream = c.rotate(req).await?.into_inner();
            drain(stream, json).await?;
        }
    }
    Ok(())
}

async fn relay(cmd: RelayCmd, sock: PathBuf, json: bool) -> anyhow::Result<()> {
    let mut c = RelayClient::new(connect(sock).await?);
    match cmd {
        RelayCmd::Create {
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
            let policy = v1::RelayPolicy {
                name,
                secret_name: secret,
                provider: provider_to_proto(&provider),
                mode: mode_to_proto(&mode),
                host_allow: hosts,
                path_allow: paths,
                method_allow: methods,
                expires_at: expires.unwrap_or_default(),
                rate_per_min: rate.unwrap_or(0),
                quota_total: quota.unwrap_or(0),
                enabled: !disabled,
                ephemeral: false,
                upstream_base: upstream_base.unwrap_or_default(),
            };
            let stream = c
                .create(v1::CreateRelayReq {
                    policy: Some(policy),
                })
                .await?
                .into_inner();
            drain(stream, json).await?;
        }
        RelayCmd::Revoke {
            name,
            apply,
            confirm,
        } => {
            let r = c
                .revoke(v1::RevokeRelayReq {
                    name,
                    apply,
                    confirm,
                })
                .await?
                .into_inner();
            render::render_revoke(&r, json);
        }
        RelayCmd::RevokeToken { token_id, apply } => {
            let r = c
                .revoke_bearer(v1::RevokeBearerReq { token_id, apply })
                .await?
                .into_inner();
            render::render_revoke(&r, json);
        }
        RelayCmd::List { all } => {
            let r = c
                .list(v1::ListRelayReq {
                    include_revoked: all,
                })
                .await?
                .into_inner();
            for item in &r.items {
                if json {
                    println!(
                        "{}",
                        serde_json::json!({ "name": item.name, "enabled": item.enabled })
                    );
                } else {
                    println!("{} enabled={}", item.name, item.enabled);
                }
            }
        }
        RelayCmd::Mint {
            name,
            ttl,
            mode,
            provider,
            repos,
            perms,
        } => {
            // TTL string -> seconds (default 0 => engine clamps against policy + the 24h ceiling).
            let ttl_secs = match ttl {
                Some(s) => s
                    .parse::<u64>()
                    .with_context(|| format!("invalid --ttl '{s}' (expected seconds)"))?,
                None => 0,
            };
            let req = mint_req_for_relay_mint(name, ttl_secs, mode, provider, repos, perms);
            let r = c.mint(req).await?.into_inner();
            render::render_mint(&r, json);
        }
    }
    Ok(())
}

async fn ca(cmd: CaCmd, sock: PathBuf, json: bool) -> anyhow::Result<()> {
    let mut c = CertsClient::new(connect(sock).await?);
    match cmd {
        CaCmd::Init { apply } => {
            let stream = c
                .ca_init(v1::CaInitReq {
                    apply,
                    confirm: false,
                })
                .await?
                .into_inner();
            drain(stream, json).await?;
        }
        CaCmd::Issue {
            cn,
            sans,
            ttl_days,
            usage,
        } => {
            let stream = c
                .issue(v1::IssueLeafReq {
                    cn,
                    sans,
                    ttl_days: ttl_days.unwrap_or(0),
                    usage,
                })
                .await?
                .into_inner();
            drain(stream, json).await?;
        }
        CaCmd::List => {
            let r = c.list(v1::ListCertReq {}).await?.into_inner();
            render::render_certs(&r, json);
        }
        CaCmd::Rotate { apply, confirm } => {
            let stream = c
                .ca_rotate(v1::CaRotateReq { apply, confirm })
                .await?
                .into_inner();
            drain(stream, json).await?;
        }
        CaCmd::Renew { cn, apply } => {
            let stream = c.renew(v1::RenewLeafReq { cn, apply }).await?.into_inner();
            drain(stream, json).await?;
        }
        CaCmd::Revoke { cn, apply, confirm } => {
            let stream = c
                .revoke(v1::RevokeLeafReq { cn, apply, confirm })
                .await?
                .into_inner();
            drain(stream, json).await?;
        }
        CaCmd::Trust {
            targets,
            system_bundle,
            apply,
            confirm,
        } => {
            let stream = c
                .trust_apply(v1::TrustReq {
                    targets,
                    system_bundle,
                    apply,
                    confirm,
                })
                .await?
                .into_inner();
            drain(stream, json).await?;
        }
    }
    Ok(())
}

// ---- `env-ctl run` (PR-2b): mint a bearer + run the child with the daemon-built injection --------

/// Map the daemon's proto `ResolvedInjection` back into the engine's `inject::ResolvedInjection`. The
/// DAEMON is authoritative: it built the env delta (the bearer-only child env) via the engine's
/// `injection_template` and shipped the resolved shape over the peercred-gated UDS. This is a pure
/// field-for-field transcription — secretctl re-derives NO env/key logic; all of that stays in the
/// engine. The empty-string `proxy_url`/`base_url` proto sentinel maps back to `None`.
fn injection_from_proto(p: &v1::ResolvedInjection) -> envctl_secrets::inject::ResolvedInjection {
    use envctl_secrets::broker::Provider;
    use envctl_secrets::inject::DataPlaneMode;
    let provider = match v1::ProviderKind::try_from(p.provider).unwrap_or(v1::ProviderKind::Generic)
    {
        v1::ProviderKind::Anthropic => Provider::Anthropic,
        v1::ProviderKind::Openai => Provider::Openai,
        v1::ProviderKind::Github => Provider::Github,
        v1::ProviderKind::Generic | v1::ProviderKind::ProviderUnspecified => Provider::Generic,
    };
    let mode =
        match v1::DataPlaneMode::try_from(p.mode).unwrap_or(v1::DataPlaneMode::BaseUrlRepoint) {
            v1::DataPlaneMode::HttpsProxyMitm => DataPlaneMode::HttpsProxyMitm,
            v1::DataPlaneMode::NativeSubtoken => DataPlaneMode::NativeSubtoken,
            v1::DataPlaneMode::BaseUrlRepoint | v1::DataPlaneMode::ModeUnspecified => {
                DataPlaneMode::BaseUrlRepoint
            }
        };
    let opt = |s: &str| (!s.is_empty()).then(|| s.to_string());
    envctl_secrets::inject::ResolvedInjection {
        provider,
        mode,
        env: p.env.clone().into_iter().collect(),
        ca_env_keys: p.ca_env_keys.clone(),
        proxy_url: opt(&p.proxy_url),
        base_url: opt(&p.base_url),
    }
}

/// Build the `MintReq` for an explicit `env-ctl relay mint` (pure; unit-tested). `mode` selects the
/// data plane (default base-url); `provider` defaults to generic. For a NATIVE GitHub mint with no
/// explicit `--perm`, the least-privilege default `["checks:write"]` is supplied so a native mint
/// never silently requests the installation's full default scope. `repos`/`perms` scope the native
/// mint (ignored by the non-native planes). The native ttl rides on `ttl_secs` (advisory; GitHub
/// fixes the ~1h installation-token TTL regardless).
fn mint_req_for_relay_mint(
    name: String,
    ttl_secs: u64,
    mode: Option<String>,
    provider: Option<String>,
    repos: Vec<String>,
    perms: Vec<String>,
) -> v1::MintReq {
    let mode_proto = mode
        .as_deref()
        .map(mode_to_proto)
        .unwrap_or(v1::DataPlaneMode::BaseUrlRepoint as i32);
    let provider_str = provider.as_deref().unwrap_or("generic");
    let provider_proto = provider_to_proto(provider_str);
    let is_native_github = mode_proto == v1::DataPlaneMode::NativeSubtoken as i32
        && provider_proto == v1::ProviderKind::Github as i32;
    // Least-privilege default for a native GitHub mint: `checks:write` (the merge-gate use case)
    // unless the operator passed explicit `--perm`s. Empty perms otherwise ⇒ full installation scope.
    let perms = if is_native_github && perms.is_empty() {
        vec!["checks:write".to_string()]
    } else {
        perms
    };
    v1::MintReq {
        relay: name,
        ephemeral: false,
        provider: provider_proto,
        ttl_secs,
        client_pid: 0,
        mode: mode_proto,
        repos,
        perms,
    }
}

/// Build the `MintReq` for an `env-ctl run` from its args (pure; unit-tested). The relay name is the
/// first `--relay` (else the provider name, else "default"); the provider defaults to generic
/// (default-deny in the engine). `client_pid = 0` selects uid-primary binding (OQ1); `ttl_secs = 0`
/// lets the engine clamp against the policy + the 24h ceiling.
fn mint_req_for_run(a: &cli::RunArgs) -> v1::MintReq {
    let provider = a.provider.as_deref().unwrap_or("generic");
    let relay = a
        .relays
        .first()
        .cloned()
        .or_else(|| a.provider.clone())
        .unwrap_or_else(|| "default".to_string());
    v1::MintReq {
        relay,
        ephemeral: a.ephemeral,
        provider: provider_to_proto(provider),
        ttl_secs: 0,
        client_pid: 0,
        // `env-ctl run` uses the daemon's default data plane (base-url repoint); native scope is only
        // selected via the explicit `relay mint --mode native` path.
        mode: v1::DataPlaneMode::BaseUrlRepoint as i32,
        repos: Vec::new(),
        perms: Vec::new(),
    }
}

/// `env-ctl run -- <cmd> [args...]`: mint a peer-bound ephemeral bearer, then spawn the child with the
/// daemon-built env injection overlaid (the bearer + base-url/proxy repoint) — the real key NEVER
/// enters the child. Engine-driven: secretctl is a thin driver that mints over gRPC, then calls
/// `Engine::run_child` in-process, draining its `Event`s to the existing renderer. The process exits
/// with the child's true exit code.
///
/// Peer-binding (OQ1): we mint with `client_pid = 0` (uid-primary binding). The relay decision
/// (`broker::decide`) enforces the bound uid at swap time, and the PR-2a proxy resolves the request's
/// peer uid (not pid) from the loopback connection; the child runs as the same uid as secretctl, so
/// the uid binding holds with no exec gymnastics. (decide's pid check only fires for a non-None bound
/// pid, and the PR-2a proxy sends `peer_pid: None`, so binding a pid would deny the swap.)
async fn run_child_cmd(a: cli::RunArgs, sock: PathBuf, json: bool) -> anyhow::Result<()> {
    // Fail-closed: an empty argv has no program to run (the engine also refuses, but catch it early
    // for a friendlier error and to avoid an unnecessary mint).
    if a.argv.is_empty() {
        anyhow::bail!(
            "`env-ctl run` requires a command: env-ctl run [--relay R] -- <cmd> [args...]"
        );
    }

    // Mint a peer-bound ephemeral bearer + receive the daemon-built injection (PR-2b populates it).
    let mut c = RelayClient::new(connect(sock).await?);
    let resp = c.mint(mint_req_for_run(&a)).await?.into_inner();

    // FAIL-CLOSED: without a populated injection (e.g. the daemon's relay proxy never bound) we have
    // no proxy to repoint the child at — refuse rather than spawn with a half-built env.
    let proto_injection = resp.injection.ok_or_else(|| {
        anyhow::anyhow!(
            "the daemon returned no child-env injection (is the relay proxy listening?); refusing to \
             run the child without a repointed, bearer-only env"
        )
    })?;
    let injection = injection_from_proto(&proto_injection);

    // Build the engine plan. The bearer is uid-bound, so no pid hint is needed (OQ1).
    let plan = envctl_secrets::inject::ChildEnvPlan {
        injection,
        child_pid_hint: None,
    };

    // Drive the engine in-process. `run_child` overlays ONLY the injection env (bearer, never the real
    // key) onto the inherited parent env, streams the child's stdout/stderr as `Event`s, and returns
    // the child's true exit code. We render those events through the same renderer the rest of the CLI
    // uses, then exit with the child's code.
    let (sink, rx) = envctl_secrets::EventSink::channel();

    // Open an in-process engine over the real seams. The engine is non-printing — it emits Events that
    // we render below. (run_child needs no vault/USB; it only spawns the child with the overlay env.)
    let paths = envctl_secrets::paths::Paths::resolve().context("resolving engine paths")?;
    let engine = envctl_secrets::Engine::open(paths).context("opening the in-process engine")?;

    let argv = a.argv.clone();
    // run_child is a SYNC, blocking call (it waits on the child); run it off the async reactor.
    let render_handle = std::thread::spawn(move || {
        for ev in rx {
            render_secret_event(&ev, json);
        }
    });
    let code = tokio::task::spawn_blocking(move || engine.run_child(plan, argv, &sink))
        .await
        .context("joining the child-run task")?
        .context("running the child")?;
    // The sink dropped when `run_child` returned; the render thread drains the rest and exits.
    let _ = render_handle.join();

    // Exit with the child's true exit code (POSIX 128+signal already folded by the engine).
    std::process::exit(code);
}

/// Render an engine `SecretEvent` (the in-process variant from `run_child`) to the TTY or as NDJSON.
/// Mirrors `render::render_event`, which renders the PROTO twin; here the events come straight from
/// the engine, so we map the variants we expect from a child run (`Log`, `ChildExited`,
/// `RunFinished`, `GuardRefused`). NEVER prints a secret (the engine's events carry none).
fn render_secret_event(ev: &envctl_secrets::SecretEvent, json: bool) {
    // Reuse the proto renderer by converting the engine event to its proto twin (the single mapping
    // already under test in secretd's `conv::event_to_proto`). We inline the minimal subset here so
    // secretctl does not depend on secretd. Variants without a proto twin are dropped.
    use envctl_secrets::event::{SecretEvent, Stream};
    let line_json = |v: serde_json::Value| println!("{v}");
    match ev {
        SecretEvent::Log {
            source,
            stream,
            line,
        } => {
            let s = matches!(stream, Stream::Stderr);
            if json {
                line_json(serde_json::json!({
                    "type": "log", "source": source, "stream": if s {1} else {0}, "line": line
                }));
            } else {
                let label = if s { "stderr" } else { "stdout" };
                println!("\x1b[36m[{source}:{label}] {line}\x1b[0m");
            }
        }
        SecretEvent::ChildExited { code } => {
            if json {
                line_json(serde_json::json!({ "type": "child_exited", "code": code }));
            } else {
                println!("\x1b[36mchild exited: {code}\x1b[0m");
            }
        }
        SecretEvent::RunFinished { summary } => {
            if json {
                line_json(serde_json::json!({
                    "type": "run_finished", "failed": summary.failed, "refused": summary.refused
                }));
            } else {
                println!(
                    "\x1b[32mrun finished (failed: {}, refused: {})\x1b[0m",
                    summary.failed.len(),
                    summary.refused.len()
                );
            }
        }
        SecretEvent::GuardRefused { subject, reason } => {
            if json {
                line_json(serde_json::json!({
                    "type": "guard_refused", "subject": subject, "reason": reason
                }));
            } else {
                println!("\x1b[33mrefused: {subject} ({reason})\x1b[0m");
            }
        }
        // Other variants are not produced by `run_child`; ignore them.
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run_args(relays: Vec<&str>, provider: Option<&str>, ephemeral: bool) -> cli::RunArgs {
        cli::RunArgs {
            relays: relays.into_iter().map(str::to_string).collect(),
            provider: provider.map(str::to_string),
            ephemeral,
            no_profile: false,
            profile: None,
            argv: vec!["printenv".to_string(), "ANTHROPIC_API_KEY".to_string()],
        }
    }

    #[test]
    fn mint_req_uses_uid_primary_binding_and_relay_from_flag() {
        // OQ1: client_pid is always 0 (uid-primary binding); the relay name comes from --relay.
        let req = mint_req_for_run(&run_args(vec!["claude-main"], Some("anthropic"), true));
        assert_eq!(req.client_pid, 0, "must mint with client_pid=0 (uid-bound)");
        assert_eq!(req.relay, "claude-main");
        assert_eq!(req.provider, v1::ProviderKind::Anthropic as i32);
        assert!(req.ephemeral);
        assert_eq!(req.ttl_secs, 0, "ttl=0 lets the engine clamp");
    }

    #[test]
    fn mint_req_falls_back_to_provider_then_default_relay() {
        // No --relay, but --provider given => relay = provider name.
        let req = mint_req_for_run(&run_args(vec![], Some("openai"), false));
        assert_eq!(req.relay, "openai");
        assert_eq!(req.provider, v1::ProviderKind::Openai as i32);
        // Neither --relay nor --provider => relay "default", provider generic.
        let req = mint_req_for_run(&run_args(vec![], None, false));
        assert_eq!(req.relay, "default");
        assert_eq!(req.provider, v1::ProviderKind::Generic as i32);
        assert_eq!(req.client_pid, 0);
    }

    #[test]
    fn mint_req_for_github_native_sets_mode_and_scope() {
        // `--mode native --provider github` with no `--perm` ⇒ least-privilege default checks:write.
        let req = mint_req_for_relay_mint(
            "gh".to_string(),
            0,
            Some("native".to_string()),
            Some("github".to_string()),
            vec!["meta".to_string()],
            vec![],
        );
        assert_eq!(req.mode, v1::DataPlaneMode::NativeSubtoken as i32);
        assert_eq!(req.provider, v1::ProviderKind::Github as i32);
        assert_eq!(req.repos, vec!["meta".to_string()]);
        assert_eq!(req.perms, vec!["checks:write".to_string()], "default perm");

        // Explicit `--perm`s override the default.
        let req = mint_req_for_relay_mint(
            "gh".to_string(),
            0,
            Some("native".to_string()),
            Some("github".to_string()),
            vec![],
            vec!["contents:read".to_string()],
        );
        assert_eq!(req.perms, vec!["contents:read".to_string()]);

        // A non-native default mint keeps mode=base-url + provider=generic + no scope (pre-G2 shape).
        let req = mint_req_for_relay_mint("r".to_string(), 0, None, None, vec![], vec![]);
        assert_eq!(req.mode, v1::DataPlaneMode::BaseUrlRepoint as i32);
        assert_eq!(req.provider, v1::ProviderKind::Generic as i32);
        assert!(
            req.perms.is_empty(),
            "no default perm for a non-native mint"
        );
    }

    #[test]
    fn injection_from_proto_reconstructs_engine_plan() {
        // The daemon ships the resolved injection; secretctl transcribes it into the engine type that
        // `ChildEnvPlan` carries. The bearer-only env, mode, provider, and base_url survive intact; the
        // empty-string proxy_url sentinel maps back to None.
        const BEARER: &str = "bearer-abc";
        const BASE: &str = "http://127.0.0.1:9000";
        let mut env = std::collections::HashMap::new();
        env.insert("ANTHROPIC_BASE_URL".to_string(), BASE.to_string());
        env.insert("ANTHROPIC_API_KEY".to_string(), BEARER.to_string());
        let proto = v1::ResolvedInjection {
            provider: v1::ProviderKind::Anthropic as i32,
            mode: v1::DataPlaneMode::BaseUrlRepoint as i32,
            env,
            ca_env_keys: vec![],
            proxy_url: String::new(), // sentinel -> None
            base_url: BASE.to_string(),
        };
        let eng = injection_from_proto(&proto);
        use envctl_secrets::broker::Provider;
        use envctl_secrets::inject::DataPlaneMode;
        assert_eq!(eng.provider, Provider::Anthropic);
        assert_eq!(eng.mode, DataPlaneMode::BaseUrlRepoint);
        assert_eq!(
            eng.env.get("ANTHROPIC_API_KEY").map(String::as_str),
            Some(BEARER)
        );
        assert_eq!(
            eng.env.get("ANTHROPIC_BASE_URL").map(String::as_str),
            Some(BASE)
        );
        assert_eq!(eng.base_url.as_deref(), Some(BASE));
        assert!(eng.proxy_url.is_none(), "empty proxy_url must map to None");
        assert!(eng.ca_env_keys.is_empty());

        // Build the plan the same way run_child_cmd does; the bearer is uid-bound, so no pid hint.
        let plan = envctl_secrets::inject::ChildEnvPlan {
            injection: eng,
            child_pid_hint: None,
        };
        assert!(plan.child_pid_hint.is_none());
        assert_eq!(
            plan.injection
                .env
                .get("ANTHROPIC_API_KEY")
                .map(String::as_str),
            Some(BEARER)
        );
    }

    // ===== TASK-0020 DIFFERENTIAL CONTRACT TEST ==============================================
    //
    // The consumer `flexnetos_github_app/crates/app-core/src/mint.rs` (a DIFFERENT repo) BUILDS the
    // argv it shells and PARSES our stdout. These two helpers REPLICATE its frozen shapes VERBATIM
    // (copied from that file). The tests below prove our `mint-github` clap surface parses exactly the
    // argv it emits, and our stdout deserializes exactly into its `Out` struct — so the contract can
    // never silently drift apart even though the two crates don't share a dependency.

    /// VERBATIM copy of `app-core::mint::build_argv` (the consumer's frozen argv builder). If our CLI
    /// surface diverges from this, `mint_github_argv_round_trips_through_clap` fails.
    fn consumer_build_argv(
        program: &str,
        installation_id: u64,
        repository_ids: &[u64],
        permissions: &[(&str, &str)], // (name, "read"|"write")
        ttl_secs: u64,
    ) -> Vec<String> {
        let mut argv = vec![
            program.to_string(),
            "mint-github".to_string(),
            "--installation-id".to_string(),
            installation_id.to_string(),
            "--ttl-secs".to_string(),
            ttl_secs.to_string(),
            "--output".to_string(),
            "json".to_string(),
        ];
        if !repository_ids.is_empty() {
            argv.push("--repository-ids".to_string());
            argv.push(
                repository_ids
                    .iter()
                    .map(u64::to_string)
                    .collect::<Vec<_>>()
                    .join(","),
            );
        }
        if !permissions.is_empty() {
            argv.push("--permissions".to_string());
            argv.push(
                permissions
                    .iter()
                    .map(|(n, a)| format!("{n}:{a}"))
                    .collect::<Vec<_>>()
                    .join(","),
            );
        }
        argv
    }

    /// Mirrors the consumer's `Out` (`app-core::mint::parse_mint_output`): the exact field NAMES +
    /// TYPES it deserializes our stdout into (`token: String`, `expires_at_unix: u64`). We replicate
    /// its `serde_json::from_slice` + typed extraction WITHOUT a serde-derive dep — a missing field,
    /// a wrong type (e.g. a STRING `expires_at_unix`), or a non-object is a hard parse failure here,
    /// exactly as it would be for the consumer's derived `Out`.
    struct ConsumerOut {
        token: String,
        expires_at_unix: u64,
    }

    impl ConsumerOut {
        /// `serde_json::from_slice` into the consumer's typed shape, fail-closed on any mismatch.
        fn from_slice(bytes: &[u8]) -> Result<Self, String> {
            let v: serde_json::Value =
                serde_json::from_slice(bytes).map_err(|e| format!("malformed: {e}"))?;
            let obj = v.as_object().ok_or("not a JSON object")?;
            let token = obj
                .get("token")
                .and_then(|t| t.as_str())
                .ok_or("token missing / not a string")?
                .to_string();
            // `as_u64` REQUIRES a JSON number (a string `expires_at_unix` would be `None`) — the same
            // discipline the consumer's `expires_at_unix: u64` field enforces.
            let expires_at_unix = obj
                .get("expires_at_unix")
                .and_then(|e| e.as_u64())
                .ok_or("expires_at_unix missing / not a u64 number")?;
            Ok(Self {
                token,
                expires_at_unix,
            })
        }
    }

    // ===== TASK-0053: POLICY_DRIFT_TOKEN permission-scope regression =========================
    //
    // The live consumer `.github_org/scripts/rotate-policy-drift-token.sh:39,91-95` rotates the
    // strict-policy-drift token via `secretctl mint-github --permissions
    // administration:write,metadata:read`. This pins BOTH ends of that path against silent drift:
    //   (1) the scope parses through our real `mint-github` clap surface into `MintGithubArgs`, and
    //   (2) the ENGINE's real request-body serializer (`GitHubAppMint::mint_scoped`, which calls the
    //       private `build_token_request_body`) emits the exact GitHub permission MAP
    //       `{"administration":"write","metadata":"read"}`.
    // It drives the ACTUAL engine serializer through a capturing transport (no reimplementation, no
    // network), uses a TEST-ONLY throwaway key, and asserts ONLY on the request-body permission map —
    // it NEVER logs/prints a token and uses NO real credential (AC2/AC3/AC4).

    /// TEST-ONLY throwaway 1024-bit RSA key (weak BY DESIGN; NEVER a real credential). It only has to
    /// RS256-sign the App-JWT so `mint_scoped` reaches the body-builder; the canned 201 below means no
    /// network and no real token. PKCS#8 form (`BEGIN PRIVATE KEY`), accepted by `build_app_jwt`.
    const POLICY_DRIFT_TEST_KEY_PEM: &str = r#"-----BEGIN PRIVATE KEY-----
MIICdgIBADANBgkqhkiG9w0BAQEFAASCAmAwggJcAgEAAoGBAJhBIdXf46YQy/2f
jy4tlYwwVPTlFzl8YDRRlocnrKvNNYR5Ngj5rs+ULgY4dBXi8ZX7bmKkzkkmZcp0
7asRcOa6frSXH1/HmE0Glby5ccgVtW0FpwLP1SPT9iWoivYud3xnTsf+27gbGys6
3iQ4JnMNArV4GkTGJWDhHBqceHRtAgMBAAECgYBIEw0hYcsyYeEvPslY4ttYccjF
5W0JGYexPK41bOKgsZQUEg0yUoAeY9clurO5aKVUiqHGsJ22oyasoI2h3a/DzrM6
MNDoMAJyP9mMfwqgSD4BjN44R0NcK1DzHWoLV3c4dmmUOjcc7GeLdySIjs5QgV7G
AeWy1i5DWQ1xepDrQQJBAMi9bSq25j+Xka+kAsqr4mcF4peJ+l+m2/OYSztpRsiT
s8GsxA0MAy1a7KVOp9ADM3TXDoEiJF4daKCMTTWmzacCQQDCKtO8D4vgG+kiTNre
V8rbdPjYywB4qp8oNoWtt1TQradDhSVQ6LuiEA5F8sXJQLIyQHOZyzLKxzKuAWP6
Z7fLAkAq33IqVkfUux1tYt0Jxi4jjLk5XkmwFiYR36vps3Ffs1QIAEsa8j7Xd/zk
zWi/338k7C133P/hbeyDpZNz6v0vAkEAsf/w+4aFBH6RyxAJ1atGHMmvF4+Cbxx7
q7HP+uEGsAeCPzPgcbvpxzhQ3W8iQs08jzTmxSay+ZKDs2Ey9mv+4QJALSKKsEbb
k+VHrssB9kiF920GQUWgbhQ3rMRpRN9OAa5NKY8GjvCOj3pjJr1/H+J0vqYPnWqz
42vWSRYw7ZnIRA==
-----END PRIVATE KEY-----"#;

    #[test]
    fn policy_drift_permissions_scope_serializes() {
        use envctl_secrets::broker::Provider;
        use envctl_secrets::mint_github::{
            GitHubAppMint, HttpRequest, HttpResponse, HttpTransport, TransportError,
        };
        use envctl_secrets::seam::MintRequest;
        use envctl_secrets::{ProviderMint, SystemClock};
        use std::sync::Mutex;

        // (1) The POLICY_DRIFT scope parses through the REAL `mint-github` clap surface. We build the
        //     argv exactly as the consumer does (`consumer_build_argv`), so this is the wire path the
        //     rotate script drives, not a hand-rolled vec.
        let argv = consumer_build_argv(
            "secretctl",
            140_063_898, // POLICY_DRIFT_INSTALLATION_ID default (rotate-policy-drift-token.sh:37)
            &[1],
            &[("administration", "write"), ("metadata", "read")],
            3600,
        );
        let cli = Cli::try_parse_from(&argv).expect("policy-drift argv parses");
        let a = match cli.cmd {
            Cmd::MintGithub(a) => a,
            other => panic!("expected MintGithub, got {other:?}"),
        };
        assert_eq!(
            a.permissions,
            vec![
                "administration:write".to_string(),
                "metadata:read".to_string()
            ],
            "the strict-policy-drift scope is carried verbatim as name:access tokens"
        );

        // (2) Drive the ENGINE's real serializer with those parsed perms. A capturing transport
        //     (canned 201, no network) lets `mint_scoped` run `build_token_request_body` and hand us
        //     the exact wire body. No token is ever logged; the throwaway key only signs the JWT, and
        //     the canned response is never validated (the body is what we inspect).
        struct CapturingTransport {
            seen: Mutex<Option<HttpRequest>>,
        }
        impl HttpTransport for CapturingTransport {
            fn execute(&self, req: &HttpRequest) -> Result<HttpResponse, TransportError> {
                *self.seen.lock().unwrap() = Some(req.clone());
                // Canned GitHub 201 — a dummy (non-secret) token literal so the mint completes
                // offline. This value is asserted on NOWHERE; only the request body is inspected.
                Ok(HttpResponse {
                    status: 201,
                    body: br#"{"token":"x","expires_at":"2026-06-12T23:00:00Z"}"#.to_vec(),
                })
            }
        }

        let transport = CapturingTransport {
            seen: Mutex::new(None),
        };
        let minter = GitHubAppMint::new(
            "42",
            a.installation_id,
            zeroize::Zeroizing::new(POLICY_DRIFT_TEST_KEY_PEM.as_bytes().to_vec()),
            SystemClock,
            &transport,
        )
        .with_api_base("https://gh.test");

        let repo_ids: Vec<u64> = a
            .repository_ids
            .iter()
            .map(|s| s.parse::<u64>().expect("numeric repo id"))
            .collect();
        let req = MintRequest {
            provider: Provider::Github,
            repos: vec![],
            repo_ids,
            perms: a.permissions.clone(),
            ttl_secs: a.ttl_secs as i64,
        };
        minter
            .mint_scoped(&req)
            .expect("mint reaches the body builder");

        // The engine's real `build_token_request_body` MUST have serialized the scope as the GitHub
        // permission MAP `{"administration":"write","metadata":"read"}` (string access values).
        let sent = transport
            .seen
            .lock()
            .unwrap()
            .clone()
            .expect("a request was built");
        let body: serde_json::Value =
            serde_json::from_slice(&sent.body).expect("request body is JSON");
        let perms = body
            .get("permissions")
            .and_then(|p| p.as_object())
            .expect("permissions is a JSON object map");
        assert_eq!(perms.len(), 2, "exactly the two policy-drift scopes");
        assert_eq!(
            perms.get("administration").and_then(|v| v.as_str()),
            Some("write"),
            "administration:write maps to a string access value"
        );
        assert_eq!(
            perms.get("metadata").and_then(|v| v.as_str()),
            Some("read"),
            "metadata:read maps to a string access value"
        );
        // The serialized map equals the exact GitHub-API shape, byte-for-byte by key/value.
        assert_eq!(
            body["permissions"],
            serde_json::json!({ "administration": "write", "metadata": "read" })
        );
    }

    // ===== TASK-0026: `github-app enroll` clap surface =======================================

    #[test]
    fn github_app_enroll_parses_app_id_keypath_and_apply() {
        let argv = [
            "secretctl",
            "github-app",
            "enroll",
            "--app-id",
            "4044997",
            "--private-key",
            "/tmp/app.pem",
            "--apply",
        ];
        let cli = Cli::try_parse_from(argv).expect("enroll argv parses");
        let cmd = match cli.cmd {
            Cmd::GithubApp { cmd } => cmd,
            other => panic!("expected GithubApp, got {other:?}"),
        };
        let GithubAppCmd::Enroll {
            app_id,
            private_key,
            apply,
        } = cmd
        else {
            panic!("expected Enroll");
        };
        assert_eq!(app_id, "4044997");
        assert_eq!(private_key, "/tmp/app.pem");
        assert!(apply, "--apply was passed");
    }

    #[test]
    fn github_app_enroll_defaults_to_dry_run_and_accepts_stdin_dash() {
        // No `--apply` ⇒ dry-run by default (CF-8); `--private-key -` selects stdin.
        let argv = [
            "secretctl",
            "github-app",
            "enroll",
            "--app-id",
            "4044997",
            "--private-key",
            "-",
        ];
        let cli = Cli::try_parse_from(argv).expect("enroll argv parses (stdin, dry-run)");
        let cmd = match cli.cmd {
            Cmd::GithubApp { cmd } => cmd,
            other => panic!("expected GithubApp, got {other:?}"),
        };
        let GithubAppCmd::Enroll {
            private_key, apply, ..
        } = cmd
        else {
            panic!("expected Enroll");
        };
        assert_eq!(private_key, "-", "`-` selects stdin");
        assert!(!apply, "apply defaults to false (dry-run)");
    }

    #[test]
    fn github_app_enroll_requires_app_id_and_private_key() {
        // Missing `--private-key` is a hard clap error (both flags are required).
        let argv = ["secretctl", "github-app", "enroll", "--app-id", "4044997"];
        assert!(
            Cli::try_parse_from(argv).is_err(),
            "missing --private-key must fail to parse"
        );
        let argv = ["secretctl", "github-app", "enroll", "--private-key", "-"];
        assert!(
            Cli::try_parse_from(argv).is_err(),
            "missing --app-id must fail to parse"
        );
    }

    #[test]
    fn github_app_set_app_id_parses_app_id_and_apply() {
        // The meta-only enrollment-heal verb: takes ONLY --app-id (no PEM) + an optional --apply.
        let argv = [
            "secretctl",
            "github-app",
            "set-app-id",
            "--app-id",
            "4044997",
            "--apply",
        ];
        let cli = Cli::try_parse_from(argv).expect("set-app-id argv parses");
        let cmd = match cli.cmd {
            Cmd::GithubApp { cmd } => cmd,
            other => panic!("expected GithubApp, got {other:?}"),
        };
        let GithubAppCmd::SetAppId { app_id, apply } = cmd else {
            panic!("expected SetAppId");
        };
        assert_eq!(app_id, "4044997");
        assert!(apply, "--apply was passed");
    }

    #[test]
    fn github_app_set_app_id_defaults_to_dry_run_and_requires_app_id() {
        // No `--apply` ⇒ dry-run by default (CF-8); the PEM is NOT a parameter of this verb.
        let argv = [
            "secretctl",
            "github-app",
            "set-app-id",
            "--app-id",
            "4044997",
        ];
        let cli = Cli::try_parse_from(argv).expect("set-app-id argv parses (dry-run)");
        let GithubAppCmd::SetAppId { apply, .. } = (match cli.cmd {
            Cmd::GithubApp { cmd } => cmd,
            other => panic!("expected GithubApp, got {other:?}"),
        }) else {
            panic!("expected SetAppId");
        };
        assert!(!apply, "apply defaults to false (dry-run)");
        // Missing --app-id is a hard clap error.
        let argv = ["secretctl", "github-app", "set-app-id"];
        assert!(
            Cli::try_parse_from(argv).is_err(),
            "missing --app-id must fail to parse"
        );
    }

    #[test]
    fn github_app_revoke_token_parses_token_installation_and_apply() {
        let argv = [
            "secretctl",
            "github-app",
            "revoke-token",
            "--token",
            "/tmp/tok",
            "--installation-id",
            "12345",
            "--apply",
        ];
        let cli = Cli::try_parse_from(argv).expect("revoke-token argv parses");
        let cmd = match cli.cmd {
            Cmd::GithubApp { cmd } => cmd,
            other => panic!("expected GithubApp, got {other:?}"),
        };
        let GithubAppCmd::RevokeToken {
            token,
            installation_id,
            apply,
        } = cmd
        else {
            panic!("expected RevokeToken");
        };
        assert_eq!(token, "/tmp/tok");
        assert_eq!(installation_id, Some(12345));
        assert!(apply, "--apply was passed");
    }

    #[test]
    fn github_app_revoke_token_defaults_to_dry_run_and_accepts_stdin_dash() {
        // No `--apply` ⇒ dry-run by default (CF-8); `--token -` selects stdin; installation-id optional.
        let argv = ["secretctl", "github-app", "revoke-token", "--token", "-"];
        let cli = Cli::try_parse_from(argv).expect("revoke-token argv parses (stdin, dry-run)");
        let cmd = match cli.cmd {
            Cmd::GithubApp { cmd } => cmd,
            other => panic!("expected GithubApp, got {other:?}"),
        };
        let GithubAppCmd::RevokeToken {
            token,
            installation_id,
            apply,
        } = cmd
        else {
            panic!("expected RevokeToken");
        };
        assert_eq!(token, "-", "`-` selects stdin");
        assert_eq!(installation_id, None, "installation-id is optional");
        assert!(!apply, "apply defaults to false (dry-run)");
    }

    #[test]
    fn github_app_revoke_token_requires_token() {
        // Missing `--token` is a hard clap error (it is required).
        let argv = ["secretctl", "github-app", "revoke-token", "--apply"];
        assert!(
            Cli::try_parse_from(argv).is_err(),
            "missing --token must fail to parse"
        );
    }

    #[test]
    fn mint_github_argv_round_trips_through_clap() {
        // The consumer's argv (with both optional scopes) must parse into our subcommand 1:1.
        let argv = consumer_build_argv(
            "secretctl",
            99,
            &[10, 20],
            &[("checks", "write"), ("contents", "read")],
            3600,
        );
        let cli = Cli::try_parse_from(&argv).expect("consumer argv parses");
        let a = match cli.cmd {
            Cmd::MintGithub(a) => a,
            other => panic!("expected MintGithub, got {other:?}"),
        };
        assert_eq!(a.installation_id, 99);
        assert_eq!(a.ttl_secs, 3600);
        assert_eq!(a.output, "json");
        // `--repository-ids 10,20` comma-splits to ["10","20"] (the daemon parses these to u64).
        assert_eq!(a.repository_ids, vec!["10".to_string(), "20".to_string()]);
        // `--permissions` is forwarded VERBATIM as name:access tokens.
        assert_eq!(
            a.permissions,
            vec!["checks:write".to_string(), "contents:read".to_string()]
        );

        // And the request we build carries the parsed scope (ids parse cleanly; perms verbatim).
        let ids: Vec<u64> = a
            .repository_ids
            .iter()
            .map(|s| s.parse::<u64>().unwrap())
            .collect();
        assert_eq!(ids, vec![10, 20]);
    }

    #[test]
    fn mint_github_argv_round_trips_without_optional_scopes() {
        // No `--repository-ids` / `--permissions` ⇒ empty vecs (installation default scope).
        let argv = consumer_build_argv("secretctl", 4044997, &[], &[], 600);
        let cli = Cli::try_parse_from(&argv).expect("minimal consumer argv parses");
        let a = match cli.cmd {
            Cmd::MintGithub(a) => a,
            other => panic!("expected MintGithub, got {other:?}"),
        };
        assert_eq!(a.installation_id, 4_044_997);
        assert_eq!(a.ttl_secs, 600);
        assert!(a.repository_ids.is_empty());
        assert!(a.permissions.is_empty());
    }

    #[test]
    fn stdout_json_deserializes_into_consumer_out_shape() {
        // Build our stdout EXACTLY as `mint_github` does, then deserialize with the consumer's `Out`.
        let resp = v1::MintGithubResp {
            token: "ghs_frozen_contract_token".to_string(),
            expires_at_unix: 1_700_000_000,
        };
        let out_value = serde_json::json!({
            "token": resp.token,
            "expires_at_unix": resp.expires_at_unix,
        });
        let stdout = serde_json::to_string(&out_value).unwrap();

        // The consumer reads stdout with `serde_json::from_slice` — exercise that exact path.
        let parsed = ConsumerOut::from_slice(stdout.as_bytes())
            .expect("our stdout deserializes into the consumer's Out shape");
        assert_eq!(parsed.token, "ghs_frozen_contract_token");
        assert_eq!(parsed.expires_at_unix, 1_700_000_000u64);

        // Contract hardening: stdout is COMPACT (no pretty whitespace) and carries ONLY the two keys —
        // a stray field/line would break the consumer's `from_slice`.
        assert!(
            !stdout.contains('\n') && !stdout.contains("  "),
            "stdout JSON must be compact: {stdout}"
        );
        let raw: serde_json::Value = serde_json::from_str(&stdout).unwrap();
        let obj = raw.as_object().unwrap();
        assert_eq!(obj.len(), 2, "exactly two fields: token + expires_at_unix");
        assert!(obj.contains_key("token") && obj.contains_key("expires_at_unix"));
        // `expires_at_unix` is a JSON NUMBER, not a string (the consumer's `u64` requires it).
        assert!(obj["expires_at_unix"].is_number());
    }
}
