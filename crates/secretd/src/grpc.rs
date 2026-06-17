//! gRPC service implementations (Vault / Relay / Lock / Audit / Certs) over the engine.
//!
//! Each mutating RPC server-streams `Event`s (bridged from the engine's std-mpsc `SecretEvent`
//! stream via [`crate::audit::run_streaming`]); unary RPCs run the SYNC engine call on
//! `spawn_blocking` and map its result. Security OUTCOMES are committed to the durable hash-chained
//! audit log by the engine BEFORE the RPC returns (HF-14).
//!
//! REVEAL / broker_only invariant: the daemon NEVER re-implements the reveal gate — `Vault.Get`
//! forwards `reveal`/`apply` to `engine.secret_get`, which refuses a broker_only reveal and
//! apply-gates an allowed one. A refusal surfaces as `Status::permission_denied` with an EMPTY
//! value, so the real key never crosses the wire for a broker_only secret.
//!
//! Vault.List/Rm/Rotate, Relay.Create/List, Audit.Query, and GetSecret.meta are now wired to the
//! engine's metadata-read / fail-closed-mutation methods (TASK-0035). The only RPCs still returning
//! `Unimplemented` are ALL of `Certs.*` (CA path — Phase 4+); the engine exposes no public CA-issue
//! path for them and the engine's CA crate surface is untouched here.
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use envctl_secrets::{Engine, EventSink, Unlock};
use envctl_secrets_proto::v1;
use tonic::{Request, Response, Status};
use zeroize::Zeroizing;

use crate::audit::{run_streaming, EventStream};
use crate::conv;

/// Shared daemon-side state the engine does not expose publicly. The engine remains the authority;
/// these are best-effort mirrors for `Lock.Status` (the engine has no public `status()`).
#[derive(Clone, Default)]
pub struct DaemonState {
    /// Mirror of the last successful Unlock/Lock outcome. The engine is the true authority.
    pub unlocked: Arc<AtomicBool>,
    /// Whether the current unlock was achieved via the **USB possession factor** (the Seed), as
    /// opposed to the passphrase. Set on a successful USB unlock, cleared on passphrase unlock and
    /// on lock. Surfaced as `Status.usb_possessed` — a cheap, non-blocking signal ("USB possession
    /// was proven for the active session"); deliberately NOT a live re-probe, which would make
    /// `status` block on a Seed round-trip (and hang when the Seed is absent).
    pub usb_unlocked: Arc<AtomicBool>,
    /// The relay proxy's bound `127.0.0.1:<port>` loopback address, published ONCE by `main` after
    /// `serve_proxy` binds. PR-2b's `Relay.Mint` reads it to fill `MintResp.injection` so the child
    /// is repointed at the proxy. `None` until the proxy has bound (or if it failed to bind).
    pub proxy_addr: Arc<std::sync::OnceLock<std::net::SocketAddr>>,
}

/// Map a `JoinError` from `spawn_blocking` to an internal status.
fn join_err(e: tokio::task::JoinError) -> Status {
    Status::internal(format!("blocking task failed: {e}"))
}

/// Map an engine `anyhow::Error` to a tonic `Status` for the metadata-read / mutation RPCs
/// (TASK-0035). A locked vault is a `failed_precondition` (the operator must unlock first); anything
/// else is `internal`. NEVER echoes a secret — these engine errors carry no key material.
fn engine_status(e: anyhow::Error) -> Status {
    use envctl_secrets::EngineError;
    if let Some(EngineError::Locked) = e.downcast_ref::<EngineError>() {
        return Status::failed_precondition(e.to_string());
    }
    Status::internal(e.to_string())
}

/// Read the USB keyslot keyfile for `partuuid` via the engine's `UsbProbe` seam (the same seam the
/// engine's unlock path uses to PROVE possession). The keyfile is HKDF IKM only — it never crosses
/// the wire and is never persisted (the engine drops it after wrapping the slot).
///
/// Forwards to `RealUsbProbe::keyfile_for`. Built with `--features seed-factor`, that resolves the
/// keyfile from the **Cognitum Seed** (a deterministic, PARTUUID-bound Ed25519 signature) — so the
/// material is identical at init and at unlock for the same `partuuid`. Without the feature the seam
/// returns `None` and we refuse cleanly (fail-closed): a stock daemon has no USB backend, so the
/// operator enrolls the passphrase keyslot and completes USB enrollment with a seed-factor build +
/// the Seed present. `None` also covers a Seed that is unreachable/unpaired at runtime.
fn read_usb_keyfile(partuuid: &str) -> Result<zeroize::Zeroizing<Vec<u8>>, String> {
    use envctl_secrets::{RealUsbProbe, UsbProbe};
    RealUsbProbe.keyfile_for(partuuid).ok_or_else(|| {
        "USB possession not proven: the Cognitum Seed USB factor is unavailable (Seed unreachable / \
         unpaired, or this secretd was not built with --features seed-factor). Enroll the passphrase \
         keyslot as recovery and retry USB enrollment with the Seed present."
            .to_string()
    })
}

// ============================================================================================
// Vault
// ============================================================================================

#[derive(Clone)]
pub struct VaultSvc {
    pub engine: Engine,
}

#[tonic::async_trait]
impl v1::vault_server::Vault for VaultSvc {
    type InitStream = EventStream;
    type AddStream = EventStream;
    type RmStream = EventStream;
    type RotateStream = EventStream;
    type SetGithubAppIdStream = EventStream;

    /// Vault.Init — genesis: mint the DEK + enroll the passphrase keyslot (and optionally a USB
    /// keyslot) over `Engine::init_vault`. Owner-only (the SO_PEERCRED interceptor already gated the
    /// channel). FAIL-CLOSED + apply-gated:
    ///   * `apply=false` (the default) is a DRY-RUN: it emits a preview of what init WOULD do and
    ///     mutates NOTHING (no DEK, no keyslot, no audit row).
    ///   * `apply=true` runs the real `init_vault`, which itself REFUSES to clobber an existing vault
    ///     (engine guard) and re-validates the Argon2 floor.
    /// The daemon FORCES the hardened Argon2 params server-side ([`conv::forced_argon2_params`]); the
    /// client never supplies KDF params. The optional passphrase is owner-only over the UDS and is
    /// zeroized after `init_vault` derives from it. For a USB keyslot the keyfile is read via the
    /// `UsbProbe` seam by PARTUUID — it is NEVER carried on the wire.
    async fn init(
        &self,
        request: Request<v1::InitReq>,
    ) -> Result<Response<Self::InitStream>, Status> {
        let req = request.into_inner();
        // Validate USB fields at the boundary (enroll_usb REQUIRES a PARTUUID). Fails closed.
        let usb_uuid = conv::init_usb_uuid(&req)?;
        let apply = req.apply;
        // Move the optional passphrase into a Zeroizing buffer immediately; the proto String drops
        // with `req`. A missing passphrase means a USB-only enrollment (no passphrase keyslot is
        // valid only if a USB slot is enrolled — the engine requires at least one factor).
        let passphrase: Zeroizing<String> = Zeroizing::new(req.passphrase.unwrap_or_default());
        let params = conv::forced_argon2_params();

        let stream = run_streaming(self.engine.clone(), move |engine, sink: &EventSink| {
            use envctl_secrets::event::{SecretEvent, Stream};
            // DRY-RUN (the default, CF-8): preview only — mutate nothing.
            if !apply {
                let usb_note = match &usb_uuid {
                    Some(u) => format!(" + a USB keyslot for PARTUUID {u}"),
                    None => String::new(),
                };
                sink.emit(SecretEvent::Log {
                    source: "vault.init".to_string(),
                    stream: Stream::Stdout,
                    line: format!(
                        "DRY-RUN: would initialize a fresh vault (passphrase keyslot{usb_note}; \
                         Argon2id m={} KiB, t={}, p={}). Re-run with --apply to mutate.",
                        params.m_kib, params.t_cost, params.p_lanes
                    ),
                });
                return Ok(());
            }

            // APPLY: read the USB keyfile via the seam (possession is proven cryptographically by the
            // engine when it wraps the slot). The keyfile NEVER crosses the wire.
            let usb_keyfile = match &usb_uuid {
                Some(uuid) => match read_usb_keyfile(uuid) {
                    Ok(kf) => Some(kf),
                    Err(e) => anyhow::bail!(e),
                },
                None => None,
            };

            // The engine refuses to clobber an existing vault and re-validates the Argon2 floor.
            engine.init_vault(passphrase, usb_uuid.clone(), usb_keyfile, params, sink)
        });
        Ok(Response::new(stream))
    }

    async fn add(
        &self,
        request: Request<v1::AddSecretReq>,
    ) -> Result<Response<Self::AddStream>, Status> {
        let req = request.into_inner();
        let meta = conv::add_req_to_meta(&req);
        // Move the value into a Zeroizing buffer; the proto buffer is dropped with `req`.
        let body = Zeroizing::new(req.value);
        let stream = run_streaming(self.engine.clone(), move |engine, sink: &EventSink| {
            engine.secret_put(meta, body, sink)
        });
        Ok(Response::new(stream))
    }

    /// Vault.SetGithubAppId — TASK-0026 (`secretctl github-app enroll`): seal the NON-SECRET GitHub
    /// App id (`github-app-id`) that `mint_github_token` reads. The App PEM is enrolled separately via
    /// `Vault.Add` (broker_only). FAIL-CLOSED + apply-gated, mirroring `init`:
    ///   * an empty `app_id` is a malformed request ⇒ `invalid_argument` (never written).
    ///   * `apply=false` (the default, CF-8) is a DRY-RUN: emit a preview Event and mutate NOTHING.
    ///   * `apply=true` runs `engine.put_github_app_id`, which REFUSES when the vault is Locked
    ///     (no DEK) ⇒ surfaced as `failed_precondition` (the operator must unlock first). The id is
    ///     non-secret (integrity-covered by the header MAC), so it is safe to echo in the Event.
    ///
    /// Without the `provider-github` feature the engine has no enroll path ⇒ `Unimplemented`.
    #[cfg(not(feature = "provider-github"))]
    async fn set_github_app_id(
        &self,
        _request: Request<v1::SetGithubAppIdReq>,
    ) -> Result<Response<Self::SetGithubAppIdStream>, Status> {
        Err(Status::unimplemented(
            "Vault.SetGithubAppId requires the provider-github feature",
        ))
    }

    #[cfg(feature = "provider-github")]
    async fn set_github_app_id(
        &self,
        request: Request<v1::SetGithubAppIdReq>,
    ) -> Result<Response<Self::SetGithubAppIdStream>, Status> {
        use envctl_secrets::event::{SecretEvent, Stream};
        let req = request.into_inner();
        let app_id = req.app_id.trim().to_string();
        // Boundary validation: an empty id is malformed — refuse BEFORE any mutation (fail-closed).
        if app_id.is_empty() {
            return Err(Status::invalid_argument("app_id must not be empty"));
        }

        // DRY-RUN (the default, CF-8): preview only — mutate nothing (no meta write, no audit). No
        // engine call is made, so this can never fail closed; emit one Log Event and finish.
        if !req.apply {
            let stream = run_streaming(self.engine.clone(), move |_engine, sink: &EventSink| {
                sink.emit(SecretEvent::Log {
                    source: "vault.set_github_app_id".to_string(),
                    stream: Stream::Stdout,
                    line: format!(
                        "DRY-RUN: would enroll GitHub App id '{app_id}' (meta key `github-app-id`). \
                         Re-run with --apply to mutate."
                    ),
                });
                Ok(())
            });
            return Ok(Response::new(stream));
        }

        // APPLY: run the SYNC engine write on spawn_blocking. The engine REFUSES when the vault is
        // Locked (no DEK) ⇒ `Err(Locked)`, which we classify to `failed_precondition` (the operator
        // must unlock first) — NOT `internal`, matching the unary `map_mint_github_err` discipline.
        // The id is non-secret (header-MAC integrity-covered), so it is safe to echo in the Event.
        let engine = self.engine.clone();
        let id_for_log = app_id.clone();
        tokio::task::spawn_blocking(move || engine.put_github_app_id(&app_id))
            .await
            .map_err(join_err)?
            .map_err(map_set_app_id_err)?;

        // On success, ship a one-item success stream (a single Log Event) so the client drains it
        // exactly like Add/Init. `run_streaming` here makes no further engine mutation.
        let stream = run_streaming(self.engine.clone(), move |_engine, sink: &EventSink| {
            sink.emit(SecretEvent::Log {
                source: "vault.set_github_app_id".to_string(),
                stream: Stream::Stdout,
                line: format!("enrolled GitHub App id '{id_for_log}' (meta key `github-app-id`)"),
            });
            Ok(())
        });
        Ok(Response::new(stream))
    }

    async fn get(
        &self,
        request: Request<v1::GetSecretReq>,
    ) -> Result<Response<v1::GetSecretResp>, Status> {
        let req = request.into_inner();
        let name = req.name.clone();
        let reveal = req.reveal;
        // `confirm` is an EXTRA control-plane belt: a reveal that is not BOTH applied AND confirmed is
        // treated as a dry-run (apply=false passed through), so the engine never reveals. We still
        // forward `apply` truthfully so the ENGINE stays the authority on the reveal gate.
        let apply = req.apply && req.confirm;
        let engine = self.engine.clone();
        let meta_name = name.clone();
        let res = tokio::task::spawn_blocking(move || {
            let sink = EventSink::null();
            // Read the NON-SECRET metadata first (un-audited; no value) so the response carries
            // `meta` regardless of whether the value is revealed. A meta read failure (e.g. locked)
            // is non-fatal here — `secret_get` below is the authority and reports the real error.
            let meta = engine.secret_meta(&meta_name).ok().flatten();
            let got = engine.secret_get(&name, reveal, apply, &sink);
            (meta, got)
        })
        .await
        .map_err(join_err)?;
        let (meta, res) = res;

        match res {
            Ok(value) => {
                // The engine bails (Err) for BOTH a broker_only reveal and a `reveal && !apply`
                // dry-run, so reaching `Ok` with `reveal == true` means an APPLIED, allowed reveal
                // (apply was folded above and enforced by the engine). `revealed` therefore tracks
                // `reveal` directly — NOT value-emptiness — so a genuinely zero-length secret still
                // reports `revealed = true` on a successful reveal. The value is forwarded as-is on a
                // reveal and is the engine's empty buffer on a non-reveal (metadata-only) read.
                let revealed = reveal;
                // Populate the NON-SECRET metadata from `engine.secret_meta` (TASK-0035). `None`
                // (unknown secret / a meta-read race) leaves `meta: None` — honest, never fabricated.
                Ok(Response::new(v1::GetSecretResp {
                    meta: meta.map(conv::secret_meta_to_proto),
                    value: if revealed { value.to_vec() } else { Vec::new() },
                    revealed,
                }))
            }
            // A refusal (broker_only / apply-not-set) or any engine error: the real key NEVER crosses
            // the wire — the value is empty and the status carries no key material.
            Err(e) => Err(Status::permission_denied(e.to_string())),
        }
    }

    /// Vault.List — METADATA-ONLY list of stored secrets (TASK-0035), optionally filtered to one
    /// provider. Fail-closed: a locked vault is `failed_precondition` (the engine gates on unlock).
    /// NEVER returns a value/ciphertext — `engine.secret_list` yields `SecretListItem` (non-secret
    /// fields only), mapped to proto `SecretMeta` via `conv::secret_list_item_to_proto`.
    async fn list(
        &self,
        request: Request<v1::ListSecretReq>,
    ) -> Result<Response<v1::ListSecretResp>, Status> {
        let req = request.into_inner();
        let provider = req.provider.map(conv::provider_from_proto);
        let engine = self.engine.clone();
        let items = tokio::task::spawn_blocking(move || {
            let sink = EventSink::null();
            engine.secret_list(provider, &sink)
        })
        .await
        .map_err(join_err)?
        .map_err(engine_status)?;
        Ok(Response::new(v1::ListSecretResp {
            items: items
                .into_iter()
                .map(conv::secret_list_item_to_proto)
                .collect(),
        }))
    }

    /// Vault.Rm — DESTRUCTIVE removal of every version, fail-closed + dry-run by default (TASK-0035).
    /// `apply = req.apply && req.confirm` (mirrors `Relay.Revoke`): an apply without confirm DOWNGRADES
    /// to a dry-run. The engine refuses on a locked vault, counts the would-remove on a dry-run, and
    /// removes + audits on apply. Streams the engine's `SecretEvent`s; NEVER logs a secret byte.
    async fn rm(
        &self,
        request: Request<v1::RmSecretReq>,
    ) -> Result<Response<Self::RmStream>, Status> {
        let req = request.into_inner();
        if req.name.trim().is_empty() {
            return Err(Status::invalid_argument(
                "Vault.Rm requires a non-empty name",
            ));
        }
        // Root-of-trust destructive verb: apply REQUIRES confirm (apply && confirm).
        let apply = req.apply && req.confirm;
        let name = req.name;
        let stream = run_streaming(self.engine.clone(), move |engine, sink: &EventSink| {
            engine.secret_rm(&name, apply, sink).map(|_| ())
        });
        Ok(Response::new(stream))
    }

    /// Vault.Rotate — append a fresh sealed version carrying the current meta forward, fail-closed +
    /// dry-run by default (TASK-0035). The engine refuses on a locked vault or an unknown secret, and
    /// the new value is held in `Zeroizing`. Streams the engine's `SecretEvent`s; NEVER logs a value.
    async fn rotate(
        &self,
        request: Request<v1::RotateReq>,
    ) -> Result<Response<Self::RotateStream>, Status> {
        let req = request.into_inner();
        if req.name.trim().is_empty() {
            return Err(Status::invalid_argument(
                "Vault.Rotate requires a non-empty name",
            ));
        }
        let apply = req.apply;
        let name = req.name;
        // Move the new value into a Zeroizing buffer immediately; the proto buffer drops with `req`.
        let new_value = Zeroizing::new(req.new_value);
        let stream = run_streaming(self.engine.clone(), move |engine, sink: &EventSink| {
            engine.secret_rotate(&name, new_value, apply, sink)
        });
        Ok(Response::new(stream))
    }

    /// Vault.MintGithub — TASK-0020 FROZEN consumer contract. Per-call GitHub App installation-token
    /// mint: the engine builds a fresh `GitHubAppMint` from the vault-sealed App key for the
    /// request's `installation_id`, exchanges the App-JWT for a scoped installation token, and
    /// returns `{token, expires_at_unix}`. Read-only (no apply/confirm) — the gate is the unlocked
    /// vault (USB-possession-floored). The minted token is NEVER logged here; it crosses the wire
    /// ONLY in `MintGithubResp.token` and is materialized as a `String` exactly once.
    ///
    /// Without the `provider-github` feature the daemon has no mint path ⇒ `Unimplemented`.
    #[cfg(not(feature = "provider-github"))]
    async fn mint_github(
        &self,
        _request: Request<v1::MintGithubReq>,
    ) -> Result<Response<v1::MintGithubResp>, Status> {
        Err(Status::unimplemented(
            "Vault.MintGithub requires the provider-github feature",
        ))
    }

    #[cfg(feature = "provider-github")]
    async fn mint_github(
        &self,
        request: Request<v1::MintGithubReq>,
    ) -> Result<Response<v1::MintGithubResp>, Status> {
        let req = request.into_inner();

        // Parse the repeated `repository_ids` STRINGS into u64 at the boundary — a non-numeric id is
        // a malformed request, rejected (never forwarded so it can't become a doomed GitHub call).
        // The closure maps to a small `String` (not a large `Status`) to satisfy `result_large_err`.
        let repository_ids: Vec<u64> = req
            .repository_ids
            .iter()
            .map(|s| {
                s.trim()
                    .parse::<u64>()
                    .map_err(|_| format!("repository_ids: '{s}' is not a numeric id"))
            })
            .collect::<Result<Vec<u64>, String>>()
            .map_err(Status::invalid_argument)?;

        // `ttl_secs` is proto `int64` (already an `i64`); bound-check defensively (the engine treats
        // it as advisory — GitHub fixes the installation-token lifetime ~1h). A negative ttl is a
        // malformed request.
        let ttl_secs = req.ttl_secs;
        if ttl_secs < 0 {
            return Err(Status::invalid_argument("ttl_secs must be non-negative"));
        }

        // The REST base override (GHES / e2e mock) is read HERE, in the daemon — the engine lib stays
        // env-free (same discipline as the relay-native `rebuild_github_provider`). Default ⇒ real
        // GitHub. An empty value is ignored.
        let api_base = std::env::var("ENVCTL_GITHUB_API_BASE")
            .ok()
            .filter(|b| !b.trim().is_empty());
        let params = envctl_secrets::GithubMintParams {
            installation_id: req.installation_id,
            repository_ids,
            permissions: req.permissions,
            ttl_secs,
            api_base,
        };
        let engine = self.engine.clone();
        let scoped = tokio::task::spawn_blocking(move || {
            let sink = EventSink::null();
            engine.mint_github_token(params, &sink)
        })
        .await
        .map_err(join_err)?
        .map_err(map_mint_github_err)?;

        // Materialize the token as a String EXACTLY here (the engine kept it `Zeroizing`). Defensive
        // re-check (the engine already rejects these): a non-positive expiry or an empty token is a
        // fail-closed refusal, never a fabricated success.
        if scoped.expires_at <= 0 {
            return Err(Status::permission_denied(
                "mint produced a non-positive expiry",
            ));
        }
        let token = String::from_utf8_lossy(&scoped.token).into_owned();
        if token.is_empty() {
            return Err(Status::permission_denied("mint produced an empty token"));
        }
        Ok(Response::new(v1::MintGithubResp {
            token,
            expires_at_unix: scoped.expires_at,
        }))
    }
}

/// Map a `mint_github_token` failure to a tonic `Status` WITHOUT echoing any secret (the token never
/// reaches an error path — it lives only in the success `ScopedToken`). Classify fail-closed:
///   * a locked vault ⇒ `failed_precondition` (the operator must unlock first);
///   * a transport / GitHub-HTTP error ⇒ `unavailable` (retryable, upstream/egress fault);
///   * everything else (absent key, broker denial, empty token, malformed) ⇒ `permission_denied`.
#[cfg(feature = "provider-github")]
fn map_mint_github_err(e: anyhow::Error) -> Status {
    use envctl_secrets::EngineError;
    if let Some(EngineError::Locked) = e.downcast_ref::<EngineError>() {
        return Status::failed_precondition(e.to_string());
    }
    let msg = e.to_string();
    // The engine wraps a `MintError` as "github mint failed: ...". A transport/HTTP fault is a
    // retryable upstream condition; surface it as `unavailable`. (The message carries no secret —
    // the engine's `DaemonHttpTransport` maps every reqwest error to a FIXED key-free string.)
    let lower = msg.to_ascii_lowercase();
    if lower.contains("transport") || lower.contains("github returned") {
        return Status::unavailable(msg);
    }
    Status::permission_denied(msg)
}

/// Map a `put_github_app_id` failure to a tonic `Status` (TASK-0026). A locked vault ⇒
/// `failed_precondition` (unlock first); any other write error (store fault) ⇒ `internal`. The App id
/// is non-secret, so the message carries no key material.
#[cfg(feature = "provider-github")]
fn map_set_app_id_err(e: anyhow::Error) -> Status {
    use envctl_secrets::EngineError;
    if let Some(EngineError::Locked) = e.downcast_ref::<EngineError>() {
        return Status::failed_precondition(e.to_string());
    }
    Status::internal(e.to_string())
}

// ============================================================================================
// Relay
// ============================================================================================

#[derive(Clone)]
pub struct RelaySvc {
    pub engine: Engine,
    pub state: DaemonState,
}

#[tonic::async_trait]
impl v1::relay_server::Relay for RelaySvc {
    type CreateStream = EventStream;

    /// Relay.Create — ADDITIVE named-policy create via `engine.relay_create` (TASK-0035). Non-
    /// destructive (a policy carries no secret, only a `secret_name` reference). Unknown methods in
    /// `method_allow` are rejected at the boundary (`invalid_argument`, default-deny). Streams the
    /// engine's `SecretEvent`s (a `relay_created` audit row is written by the engine).
    async fn create(
        &self,
        request: Request<v1::CreateRelayReq>,
    ) -> Result<Response<Self::CreateStream>, Status> {
        let req = request.into_inner();
        let proto_policy = req
            .policy
            .ok_or_else(|| Status::invalid_argument("Relay.Create requires a policy"))?;
        if proto_policy.name.trim().is_empty() {
            return Err(Status::invalid_argument(
                "Relay.Create requires a non-empty policy name",
            ));
        }
        // Build the engine policy at the boundary (rejects unknown methods, default-deny) BEFORE the
        // blocking closure so a malformed policy fails fast as `invalid_argument`.
        let policy = conv::policy_from_proto(&proto_policy)?;
        let stream = run_streaming(self.engine.clone(), move |engine, sink: &EventSink| {
            engine.relay_create(policy, sink).map(|_| ())
        });
        Ok(Response::new(stream))
    }

    async fn revoke(
        &self,
        request: Request<v1::RevokeRelayReq>,
    ) -> Result<Response<v1::RevokeResp>, Status> {
        let req = request.into_inner();
        // Root-of-trust verb: an `apply` without `confirm` DOWNGRADES to a dry-run.
        let apply = req.apply && req.confirm;
        let name = req.name;
        let engine = self.engine.clone();
        let n = tokio::task::spawn_blocking(move || {
            let sink = EventSink::null();
            engine.relay_revoke(&name, apply, &sink)
        })
        .await
        .map_err(join_err)?
        .map_err(|e| Status::internal(e.to_string()))?;
        Ok(Response::new(v1::RevokeResp {
            count_revoked: n,
            dry_run: !apply,
        }))
    }

    async fn revoke_bearer(
        &self,
        request: Request<v1::RevokeBearerReq>,
    ) -> Result<Response<v1::RevokeResp>, Status> {
        let req = request.into_inner();
        let apply = req.apply;
        let token_id = req.token_id;
        let engine = self.engine.clone();
        let n = tokio::task::spawn_blocking(move || {
            let sink = EventSink::null();
            engine.relay_revoke_bearer(&token_id, apply, &sink)
        })
        .await
        .map_err(join_err)?
        .map_err(|e| Status::internal(e.to_string()))?;
        Ok(Response::new(v1::RevokeResp {
            count_revoked: n,
            dry_run: !apply,
        }))
    }

    /// Relay.List — list stored relay policies (TASK-0035), filtering revoked unless
    /// `include_revoked`. Read path; the policy carries no secret (only a `secret_name` reference).
    async fn list(
        &self,
        request: Request<v1::ListRelayReq>,
    ) -> Result<Response<v1::ListRelayResp>, Status> {
        let include_revoked = request.into_inner().include_revoked;
        let engine = self.engine.clone();
        let policies = tokio::task::spawn_blocking(move || {
            let sink = EventSink::null();
            engine.relay_list(include_revoked, &sink)
        })
        .await
        .map_err(join_err)?
        .map_err(engine_status)?;
        Ok(Response::new(v1::ListRelayResp {
            items: policies.into_iter().map(conv::policy_to_proto).collect(),
        }))
    }

    async fn mint(&self, request: Request<v1::MintReq>) -> Result<Response<v1::MintResp>, Status> {
        // The peer uid is the connect-time-frozen owner uid (peercred-gated channel); the peer pid
        // is the client-supplied target pid (advisory peer binding for `env-ctl run` ephemerals).
        let peer_uid = request
            .extensions()
            .get::<tonic::transport::server::UdsConnectInfo>()
            .and_then(|i| i.peer_cred)
            .map(|c| c.uid());
        let req = request.into_inner();

        // Reject a TTL that does not fit i64 at the boundary (the engine clamps to <=24h, but must
        // not be handed a wrapping value).
        let ttl_secs: i64 = i64::try_from(req.ttl_secs)
            .map_err(|_| Status::invalid_argument("ttl_secs does not fit i64"))?;
        let peer_pid = (req.client_pid != 0).then_some(req.client_pid);

        // Phase 6: no public policy load, so synthesize the policy from the request. `relay_mint`
        // persists it as a side effect and mints a <=24h, USB-gated, peer-bound bearer against it.
        let spec = conv::mint_req_to_policy(&req);
        // Capture the provider + data-plane mode + native scope BEFORE the spec is moved into the
        // blocking closure; they shape the child env injection built once the bearer is minted.
        let provider = spec.provider;
        let mode = conv::dataplane_mode_from_swap(&spec.swap);
        let relay_name = spec.relay_id.clone();
        // Native sub-token scope (repos / perms / ttl) carried on the SwapMode (U4); empty for the
        // non-native planes. Defaults: empty perms ⇒ the installation's full default scope.
        let (native_repos, native_perms, native_ttl) = match &spec.swap {
            envctl_secrets::broker::SwapMode::NativeSubToken {
                ttl_secs,
                repos,
                perms,
            } => (repos.clone(), perms.clone(), *ttl_secs),
            _ => (Vec::new(), Vec::new(), 0),
        };
        let engine = self.engine.clone();

        let bearer = tokio::task::spawn_blocking(move || {
            let sink = EventSink::null();
            engine.relay_mint(spec, ttl_secs, peer_uid, peer_pid, &sink)
        })
        .await
        .map_err(join_err)?
        .map_err(|e| Status::permission_denied(e.to_string()))?;

        // Build the child env injection through the engine's `resolve_injection` — the SINGLE place
        // the native-subtoken decision lives (mint vs proxy-swap vs refuse). For the NATIVE plane the
        // minted token is injected directly (no loopback proxy), so we do not require `proxy_addr`.
        // For the proxy/base planes we still FAIL-CLOSED on an unbound proxy (ship `injection: None`),
        // exactly as before, so the client refuses to spawn rather than hand the child a half-built env.
        use envctl_secrets::inject::DataPlaneMode;
        let injection = if matches!(mode, DataPlaneMode::NativeSubtoken) {
            // Native: no proxy repoint. `resolve_injection` mints the scoped token (or falls back /
            // refuses). `Ok(None)` ⇒ the mint was refused (durable Refused row already written by the
            // engine) — ship `injection: None` so the client refuses to spawn (no token emitted).
            let engine = self.engine.clone();
            let bearer_raw = bearer.raw.to_string();
            // The loopback proxy URL is only needed if the native mint is UNSUPPORTED and the engine
            // falls back to the proxy-swap shape. A successful native mint ignores it (the token is
            // injected directly). If the proxy hasn't bound, the fallback ships an empty proxy (the
            // relay bearer still rides in the provider key var) — native success is unaffected.
            let proxy_url = self
                .state
                .proxy_addr
                .get()
                .map(|addr| format!("http://{addr}"))
                .unwrap_or_default();
            let resolved = tokio::task::spawn_blocking(move || {
                let sink = EventSink::null();
                engine.resolve_injection(
                    provider,
                    &relay_name,
                    &bearer_raw,
                    &proxy_url,
                    "",
                    DataPlaneMode::NativeSubtoken,
                    native_repos,
                    native_perms,
                    native_ttl,
                    &sink,
                )
            })
            .await
            .map_err(join_err)?
            .map_err(|e| Status::internal(e.to_string()))?;
            resolved.as_ref().map(conv::injection_to_proto)
        } else {
            match self.state.proxy_addr.get() {
                Some(addr) => {
                    let proxy_url = format!("http://{addr}");
                    // `ca_pem_path` is empty for the BaseUrlRepoint plane (no MITM CA). For the
                    // HttpsProxyMitm plane (PR-3b) the child MUST trust the engine-minted local CA, so
                    // we materialize the public CA bundle. FAIL-CLOSED: an uninitialized CA errors and
                    // we refuse the mint rather than ship a MITM injection the child can't validate.
                    let ca_pem_path = ca_pem_path_for_mode(&self.engine, mode)?;
                    let engine = self.engine.clone();
                    let bearer_raw = bearer.raw.to_string();
                    let resolved = tokio::task::spawn_blocking(move || {
                        let sink = EventSink::null();
                        engine.resolve_injection(
                            provider,
                            &relay_name,
                            &bearer_raw,
                            &proxy_url,
                            &ca_pem_path,
                            mode,
                            Vec::new(),
                            Vec::new(),
                            0,
                            &sink,
                        )
                    })
                    .await
                    .map_err(join_err)?
                    .map_err(|e| Status::internal(e.to_string()))?;
                    resolved.as_ref().map(conv::injection_to_proto)
                }
                None => None,
            }
        };

        // The raw bearer goes to the OWNER only (peercred-gated UDS); the REAL key is NEVER here.
        Ok(Response::new(v1::MintResp {
            bearer: bearer.raw.to_string(),
            expires_at: bearer.expires_at,
            injection,
            token_id: bearer.token_id,
        }))
    }
}

/// Resolve the child-trust `ca_pem_path` for a data-plane `mode`. The `BaseUrlRepoint` /
/// `NativeSubtoken` planes do NOT terminate TLS, so the child uses its normal OS roots and the path
/// is empty. The `HttpsProxyMitm` plane terminates the child's TLS with an engine-minted leaf, so
/// the child MUST trust the engine's local CA: we materialize the public CA bundle and return its
/// path. FAIL-CLOSED: an uninitialized CA errors (`failed_precondition`) — we never ship a MITM
/// injection whose child can't validate the leaf.
// Boxing `tonic::Status` is non-idiomatic at the gRPC boundary (mirrors conv.rs's module allow); a
// `failed_precondition` here is the documented fail-closed path, so the large-Err is intentional.
#[allow(clippy::result_large_err)]
fn ca_pem_path_for_mode(
    engine: &Engine,
    mode: envctl_secrets::inject::DataPlaneMode,
) -> Result<String, Status> {
    use envctl_secrets::inject::DataPlaneMode;
    match mode {
        DataPlaneMode::HttpsProxyMitm => {
            #[cfg(feature = "mitm-ca")]
            {
                let path = engine
                    .ca_pem_path()
                    .map_err(|_| Status::failed_precondition("MITM CA not initialized"))?;
                Ok(path.to_string_lossy().into_owned())
            }
            #[cfg(not(feature = "mitm-ca"))]
            {
                let _ = engine;
                Err(Status::failed_precondition(
                    "MITM data plane requires the mitm-ca feature",
                ))
            }
        }
        DataPlaneMode::BaseUrlRepoint | DataPlaneMode::NativeSubtoken => Ok(String::new()),
    }
}

// ============================================================================================
// Lock
// ============================================================================================

#[derive(Clone)]
pub struct LockSvc {
    pub engine: Engine,
    pub state: DaemonState,
}

#[tonic::async_trait]
impl v1::lock_server::Lock for LockSvc {
    type UnlockStream = EventStream;
    type LockNowStream = EventStream;

    async fn status(
        &self,
        _request: Request<v1::StatusReq>,
    ) -> Result<Response<v1::StatusResp>, Status> {
        // PARTIAL within the public-API constraint: `unlocked` mirrors the last Unlock/Lock outcome
        // and `usb_possessed` mirrors whether that unlock used the USB possession factor (cheap,
        // non-blocking — see `DaemonState::usb_unlocked`); `active_relays`/`secret_count` still have
        // no public query path and are reported best-effort (0). The engine is the authority.
        Ok(Response::new(v1::StatusResp {
            unlocked: self.state.unlocked.load(Ordering::SeqCst),
            usb_possessed: self.state.usb_unlocked.load(Ordering::SeqCst),
            active_relays: 0,
            secret_count: 0,
        }))
    }

    async fn unlock(
        &self,
        request: Request<v1::UnlockReq>,
    ) -> Result<Response<Self::UnlockStream>, Status> {
        let req = request.into_inner();
        // Wrap the passphrase in Zeroizing immediately; the proto String is dropped with `req`.
        let unlock = match req.passphrase {
            Some(pp) => Unlock::Passphrase(Zeroizing::new(pp)),
            None => Unlock::Usb,
        };
        let was_usb = matches!(unlock, Unlock::Usb);
        let flag = self.state.unlocked.clone();
        let usb_flag = self.state.usb_unlocked.clone();
        let stream = run_streaming(self.engine.clone(), move |engine, sink: &EventSink| {
            let r = engine.unlock(unlock, sink);
            if r.is_ok() {
                flag.store(true, Ordering::SeqCst);
                usb_flag.store(was_usb, Ordering::SeqCst);
                // G2 (DD-1, Option A): late-bind the native sub-token minter now that the App
                // credential can be unsealed from the now-unlocked vault. A failure here is
                // NON-FATAL to the unlock (the vault is usable; native mint simply stays
                // unavailable and the relay falls back to the proxy-swap path) — mirrors the
                // `rebuild_ca_if_initialized` precedent.
                #[cfg(feature = "provider-github")]
                rebuild_github_provider(engine);
            }
            r.map(|_state| ())
        });
        Ok(Response::new(stream))
    }

    async fn lock_now(
        &self,
        _request: Request<v1::LockReq>,
    ) -> Result<Response<Self::LockNowStream>, Status> {
        let flag = self.state.unlocked.clone();
        let usb_flag = self.state.usb_unlocked.clone();
        let stream = run_streaming(self.engine.clone(), move |engine, sink: &EventSink| {
            // Drop the native minter (and its Zeroizing App PEM) BEFORE the engine locks. The
            // engine's `lock()` also clears it (defense-in-depth), but clearing here keeps the
            // daemon's lock path explicit.
            engine.clear_provider();
            let r = engine.lock(sink);
            if r.is_ok() {
                flag.store(false, Ordering::SeqCst);
                usb_flag.store(false, Ordering::SeqCst);
            }
            r
        });
        Ok(Response::new(stream))
    }
}

/// Late-bind the GitHub App native sub-token minter from the unlocked vault (DD-1). Reads the App
/// PEM + `app_id`/`installation_id` for the well-known App-credential secret (the relay policy's
/// `secret_name`; configured via `ENVCTL_GITHUB_APP_SECRET`, default `github_app`) and installs a
/// `GitHubAppMint` carrying the daemon's `DaemonHttpTransport` (frozen webpki-roots reqwest client).
///
/// Non-fatal by contract: any failure (no credential enrolled, malformed meta, locked) leaves the
/// engine's `NoMint` in place so the relay falls back to the proxy-swap path. NEVER logs the PEM.
/// The REST base is overridable via `ENVCTL_GITHUB_API_BASE` (GHES / e2e mock) — default real GitHub.
#[cfg(feature = "provider-github")]
fn rebuild_github_provider(engine: &Engine) {
    let secret_name =
        std::env::var("ENVCTL_GITHUB_APP_SECRET").unwrap_or_else(|_| "github_app".to_string());
    let cred = match engine.app_credential_pem(&secret_name) {
        Ok(Some(c)) => c,
        Ok(None) => return, // no App credential enrolled — keep NoMint.
        Err(_) => return,   // locked / read error — keep NoMint (fail-closed).
    };
    let (pem, app_id, installation_id) = cred;
    let mut minter = envctl_secrets::mint_github::GitHubAppMint::new(
        app_id,
        installation_id,
        pem,
        envctl_secrets::seam::SystemClock,
        crate::transport::DaemonHttpTransport::new(),
    );
    if let Ok(base) = std::env::var("ENVCTL_GITHUB_API_BASE") {
        if !base.is_empty() {
            minter = minter.with_api_base(base);
        }
    }
    engine.install_provider(Box::new(minter));
}

// ============================================================================================
// Audit
// ============================================================================================

#[derive(Clone)]
pub struct AuditSvc {
    pub engine: Engine,
}

#[tonic::async_trait]
impl v1::audit_server::Audit for AuditSvc {
    /// Audit.Query — read a window of the durable hash-chained audit log (TASK-0035). The engine
    /// returns rows with `seq > 0` up to a CLAMPED limit (<=1000, enforced in `engine.audit_query`);
    /// the DAEMON then post-filters by `actor`/`relay`/`since`/`until` on the mapped `AuditEntry`
    /// fields. `AuditRecord`s carry NO secret bytes (the engine never writes a value into a detail).
    async fn query(
        &self,
        request: Request<v1::AuditQueryReq>,
    ) -> Result<Response<v1::AuditQueryResp>, Status> {
        let req = request.into_inner();
        // The engine clamps `limit` to <=1000; `limit == 0` means "no caller cap" — pass the engine
        // ceiling so a 0 still returns up to the clamped maximum.
        let limit = if req.limit == 0 {
            1000
        } else {
            req.limit as usize
        };
        let engine = self.engine.clone();
        let records = tokio::task::spawn_blocking(move || {
            let sink = EventSink::null();
            engine.audit_query(0, limit, &sink)
        })
        .await
        .map_err(join_err)?
        .map_err(engine_status)?;

        // Daemon-side post-filter (the engine signature stays minimal). Each predicate is applied
        // only when its proto field is present (`Some`/non-empty). `since`/`until` compare on the
        // RFC3339 `at` string lexically (RFC3339 is lexicographically ordered for a fixed offset).
        let actor = req.actor.filter(|s| !s.is_empty());
        let relay = req.relay.filter(|s| !s.is_empty());
        let since = req.since.filter(|s| !s.is_empty());
        let until = req.until.filter(|s| !s.is_empty());
        let entries: Vec<v1::AuditEntry> = records
            .into_iter()
            .map(conv::audit_to_entry)
            .filter(|e| actor.as_deref().map_or(true, |a| e.actor == a))
            .filter(|e| relay.as_deref().map_or(true, |r| e.relay == r))
            .filter(|e| since.as_deref().map_or(true, |s| e.at.as_str() >= s))
            .filter(|e| until.as_deref().map_or(true, |u| e.at.as_str() <= u))
            .collect();
        Ok(Response::new(v1::AuditQueryResp { entries }))
    }
}

// ============================================================================================
// Certs (all Unimplemented — Phase 4+)
// ============================================================================================

#[derive(Clone)]
pub struct CertsSvc {
    // Held for when the CA path (ca_issue etc.) is wired (Phase 4+: all Unimplemented).
    #[allow(dead_code)]
    pub engine: Engine,
}

#[tonic::async_trait]
impl v1::certs_server::Certs for CertsSvc {
    type CaInitStream = EventStream;
    type CaRotateStream = EventStream;
    type IssueStream = EventStream;
    type RenewStream = EventStream;
    type RevokeStream = EventStream;
    type TrustApplyStream = EventStream;

    async fn ca_init(
        &self,
        _request: Request<v1::CaInitReq>,
    ) -> Result<Response<Self::CaInitStream>, Status> {
        Err(Status::unimplemented("Certs.CaInit is Phase 4+"))
    }
    async fn ca_rotate(
        &self,
        _request: Request<v1::CaRotateReq>,
    ) -> Result<Response<Self::CaRotateStream>, Status> {
        Err(Status::unimplemented("Certs.CaRotate is Phase 4+"))
    }
    async fn issue(
        &self,
        _request: Request<v1::IssueLeafReq>,
    ) -> Result<Response<Self::IssueStream>, Status> {
        Err(Status::unimplemented("Certs.Issue is Phase 4+"))
    }
    async fn renew(
        &self,
        _request: Request<v1::RenewLeafReq>,
    ) -> Result<Response<Self::RenewStream>, Status> {
        Err(Status::unimplemented("Certs.Renew is Phase 4+"))
    }
    async fn revoke(
        &self,
        _request: Request<v1::RevokeLeafReq>,
    ) -> Result<Response<Self::RevokeStream>, Status> {
        Err(Status::unimplemented("Certs.Revoke is Phase 4+"))
    }
    async fn trust_apply(
        &self,
        _request: Request<v1::TrustReq>,
    ) -> Result<Response<Self::TrustApplyStream>, Status> {
        Err(Status::unimplemented("Certs.TrustApply is Phase 4+"))
    }
    async fn list(
        &self,
        _request: Request<v1::ListCertReq>,
    ) -> Result<Response<v1::ListCertResp>, Status> {
        Err(Status::unimplemented("Certs.List is Phase 4+"))
    }
}
