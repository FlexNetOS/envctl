# TASK-0027 (G2) — GitHub installation-token early-revoke (DELETE /installation/token) · VERDICT: GO

Additive `DELETE /installation/token` early-revoke through the EXISTING `HttpTransport` seam (no new dep),
exposed as a new `RevokeGithubToken` RPC + `secretctl github-app revoke-token` verb; `mint-github` frozen
contract untouched. GitHub's DELETE authenticates with **the installation token itself** as the bearer (204
on success) — NOT the App-JWT — and the daemon does NOT persist minted tokens. So: explicit-token verb is
the primary kill-switch; the `relay_revoke` tie-in is a documented best-effort revoke of the relay's
last engine-minted NATIVE token (the one path where the engine still holds the token in-process).

## Target repos
envctl (single). secrets-engine: mint_github.rs, lib.rs, event.rs; secrets-proto: control.proto; secretd:
grpc.rs; secretctl: cli.rs, main.rs. Tests: engine units (mint_github.rs + lib.rs) + secretd
native_mint_e2e.rs. 7 modules single-repo → sequential single-crew.

## Design choice (resolved, GO)
GitHub `DELETE /installation/token` needs the TOKEN VALUE as bearer (confirmed vs GitHub REST docs, 204 No
Content). Daemon mints into Zeroizing and hands off once (no token-at-rest store, by design). →
**Primary = explicit-token verb** `secretctl github-app revoke-token --token <tok|-> --apply` →
RevokeGithubToken RPC → `engine.revoke_github_token(token, apply, api_base, sink)`; works for ANY outstanding
token because the holder supplies it. **relay_revoke tie-in = best-effort, native-plane only**: the
NativeSubtoken `resolve_injection` path is where the ENGINE mints in-process; retain that relay's last token
(in-memory Zeroizing, never persisted, cleared on lock/clear_provider) and on `relay_revoke(apply=true)` fire
a best-effort DELETE before clearing. A handed-off/rotated token or a BaseUrlRepoint/MitM relay has nothing
to auto-revoke → relay_revoke's existing bearer/policy revocation stays authoritative, 1h expiry is the
backstop. NOT full token-tracking (worse at-rest posture; larger change) — that's an Out-of-scope follow-up.

## Engine API delta
mint_github.rs (model on mint request ctor ~229-260; revoke needs NO App-JWT):
  `pub fn build_revoke_request(api_base, user_agent, installation_token: &[u8]) -> HttpRequest`
    (method DELETE, url {base}/installation/token, Authorization: Bearer <tok>, Accept/X-GitHub-Api-Version/
     User-Agent, empty body). Token only in the auth header; NEVER {:?}-log the request (add comment).
  `pub fn revoke_installation_token<T: HttpTransport>(transport,api_base,user_agent,installation_token:&[u8])
     -> Result<(),MintError>` — 204 ⇒ Ok; transport err / non-204 ⇒ Err (snippet ≤200, no token).
lib.rs:
  `#[cfg(feature="provider-github")] pub fn revoke_github_token(&self, token: Zeroizing<Vec<u8>>, apply: bool,
     api_base: Option<String>, sink: &EventSink) -> anyhow::Result<bool>` — gate on vault Unlocked
     (EngineError::Locked matches mint's auth floor); apply=false ⇒ dry-run audit {apply:false} + return false,
     no egress; apply=true ⇒ revoke via self.inner.github_transport; Ok(204) ⇒ SecretEvent::GithubTokenRevoked
     + ok audit, return true; transport/non-204 ⇒ Err (no false success). Token never in audit/event/Err.
  native_token_cache: `Mutex<HashMap<String /*relay_id*/, Zeroizing<Vec<u8>>>>` populated in
     resolve_injection NativeSubtoken success branch (~2218+); cleared on lock()/clear_provider().
  relay_revoke tie-in (~1418): after existing policy+bearer revocation, if cache has relay_id → best-effort
     revoke_installation_token, success ⇒ GithubTokenRevoked{outcome:"revoked"}, failure ⇒ audit
     {outcome:"best_effort_failed"} SWALLOWED (relay revoke still returns bearer count); clear entry.
event.rs: `GithubTokenRevoked { installation_id: Option<u64>, outcome: String }` (never the token; outcome ∈
  "revoked"/"dry_run"/"best_effort_failed"). installation_id Option (explicit verb may not know it).
CLI+GUI: secretctl drives the RPC; engine method is the single shared entry so GUI parity (later) won't
  diverge. No GUI change in TASK-0027 (mint-github not yet in GUI) — noted Out of scope.

## Proto + gRPC delta
control.proto (additive RPC on service Vault, next to MintGithub):
  `rpc RevokeGithubToken (RevokeGithubTokenReq) returns (RevokeResp);`
  `message RevokeGithubTokenReq { bytes token = 1; bool apply = 2; uint64 installation_id = 3; }`
  Reuse existing RevokeResp{count_revoked,dry_run} (count ∈ {0,1}) — drains like Relay.Revoke. token is
  `bytes` (byte-exact, no accidental Display). Additive → wire round-trip drift test stays green.
grpc.rs handler (model on revoke/mint_github ~388/531): reject empty token (invalid_argument); token →
  Zeroizing; api_base from ENVCTL_GITHUB_API_BASE (same as mint); installation_id (req!=0).then_some;
  spawn_blocking engine.revoke_github_token; map via existing map_mint_github_err (Locked→failed_precondition,
  transport/non-204→unavailable). non-feature companion ⇒ Status::unimplemented.

## CLI delta
cli.rs GithubAppCmd::RevokeToken{ token:String(--token, `-`=stdin), installation_id:Option<u64>, apply:bool }.
main.rs handler: read token (file/`-`/stdin into Zeroizing, refuse empty); no --apply ⇒ stderr dry-run
  preview, no egress; --apply ⇒ Vault.RevokeGithubToken{token,apply,installation_id}, drain RevokeResp; --json
  ⇒ {"revoked":<bool>,"dry_run":<bool>} to stdout, human text to stderr. Token NEVER printed.

## relay_revoke tie-in (best-effort, native-plane only)
1. NativeSubtoken success → store Zeroizing(token) keyed by relay_id (replace prior; never persisted; cleared
   on lock()/clear_provider). 2. relay_revoke(apply=true): after policy+bearer revoke (1443-1461),
   cache.remove(relay_id) → best-effort revoke_installation_token; success⇒revoked event+audit, failure⇒
   best_effort_failed audit, SWALLOW err (relay still revoked at policy/bearer; worst case token lives ≤1h
   = today's behavior). 3. dry-run apply=false: no egress, count only. 4. cleared on lock()/clear_provider
   (fail-closed: locked vault holds no live token bytes).

## Dep decision (no-C proof)
ZERO new deps. Reuses mint_github::HttpTransport (seam), secretd DaemonHttpTransport (existing reqwest/rustls-
on-ring), Zeroizing, tonic/prost (proto regen only), clap. No sqlite/openssl/aws-lc; one rustls(ring) remains.
no-c.sh operates on resolved cargo metadata → graph unchanged → unaffected (run it anyway). DELETE has no body
→ no new serializer.

## Fail-closed matrix
dry-run default (apply=false) → no egress, dry_run audit+event, return false. transport error → Err →
unavailable (never success). non-204 (401/404/422) → Err(MintError::Other "{status} ... {snippet≤200}", no
token) → unavailable/permission_denied. locked vault → Err(Locked) → failed_precondition. missing App cred →
N/A (revoke needs no App key; bearer = token). empty/blank token → invalid_argument, no egress. relay tie-in
failure → best_effort_failed audit, swallowed, relay revoke still returns bearer count.

## Tests (all CI-offline: FakeTransport / loopback mock, never real GitHub)
mint_github.rs units: revoke_builds_correct_delete_request (method/url/headers/empty body);
revoke_204_is_success; revoke_non_204_is_failure (401, assert no token in err); revoke_transport_error_is_failure;
revoke_token_is_zeroized (token only in auth header, not in MintError Display).
lib.rs units (reuse unlocked_engine_with_transport ~4334): revoke_github_token_dry_run_no_egress;
revoke_github_token_apply_204 (GithubTokenRevoked{revoked}, metadata-only audit); _locked_vault_fails_closed;
relay_revoke_native_tie_in_best_effort (DELETE fired; 500 variant ⇒ relay_revoke STILL returns bearer count).
secretctl units (clap round-trip): revoke_token parses + defaults dry-run; accepts `-` stdin; requires token.
secretd e2e (native_mint_e2e.rs loopback + ENVCTL_GITHUB_API_BASE ~419-595): revoke over wire 204 ⇒
{count_revoked:1,dry_run:false}; dry-run contacts nothing; locked vault ⇒ failed_precondition.

## Sequencing (leaf-first)
1 mint_github.rs build_revoke_request + revoke_installation_token + units. 2 event.rs GithubTokenRevoked.
3 lib.rs revoke_github_token + native_token_cache + relay_revoke tie-in + lock/clear clearing + units.
4 control.proto RevokeGithubToken RPC + msg (regen). 5 grpc.rs handler (+non-feature unimplemented), reuse
map_mint_github_err. 6 secretctl cli.rs+main.rs RevokeToken + clap tests. 7 native_mint_e2e.rs.
8 fmt + clippy --workspace -Dwarnings (rtk proxy) + 4 CI gates (no-c proves graph unchanged).

## Invariants (each checkable)
no-C: zero new deps, cargo metadata unchanged → no-c.sh green; grep diff for new Cargo.toml dep (none).
engine single non-printing: request ctor + 204/non-204 + tie-in in engine via HttpTransport seam (env-free),
secretd supplies transport+RPC, secretctl thin; no println! in engine; emits SecretEvent. fail-closed/dry-run:
apply default false previews w/o egress, non-204/transport→Err never false success (tests guard). no secret in
logs: token Zeroizing only in auth header, audit/event metadata-only, map_mint_github_err echoes no secret,
no {:?} of revoke request (zeroize test guards). frozen-contract: mint-github flag/JSON + MintGithub*
untouched; revoke is additive RPC/verb/event (mint argv + proto drift tests stay green).

## Risks
GHES api-base: revoke must read ENVCTL_GITHUB_API_BASE the SAME way as mint (thread api_base identically;
e2e covers via loopback). token-via-argv leak: default-recommend `--token -` stdin (same as secret add
--value-stdin). best-effort over-promise: doc-comment + --help state plainly it revokes only the relay's last
engine-minted native token; explicit verb is the kill-switch for handed-off tokens.

## Out of scope (follow-up)
GUI revoke parity (when GUI gains mint-github) · blanket auto-revoke-on-relay-revoke for handed-off tokens
(needs persisted encrypted registry + GC, worse at-rest posture) · bulk all-tokens revoke (no GitHub endpoint).
