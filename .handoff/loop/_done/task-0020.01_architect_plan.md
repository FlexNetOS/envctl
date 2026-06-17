# TASK-0020-COMPLETE — FROZEN `mint-github` consumer-contract surface

VERDICT: GO. Base = `origin/g2-native-mint` (714b187) which ALREADY has the G2 primitive +
`crates/secretd/src/transport.rs` (`DaemonHttpTransport`) + `Engine::resolve_injection`. **Correction
to the architect's premise:** DaemonHttpTransport is PRESENT on this base — REUSE it, do not rebuild.

## Frozen contract (do NOT change; `flexnetos_github_app/crates/app-core/src/mint.rs` shells it)
`secretctl mint-github --installation-id <N> [--repository-ids a,b] [--permissions name:access,...] --ttl-secs <T> --output json`
→ stdout EXACTLY `{"token":"<tok>","expires_at_unix":<i64 epoch>}` (consumer struct: `Out{token:String, expires_at_unix:u64}`).
RPC: `rpc MintGithub(MintGithubReq) returns (MintGithubResp)` on `service Vault`;
`MintGithubReq{ uint64 installation_id=1; repeated string repository_ids=2; repeated string permissions=3; int64 ttl_secs=4; }`
`MintGithubResp{ string token=1; int64 expires_at_unix=2; }`.

## Design decisions (resolved → GO)
1. **repository_ids vs repositories:** GitHub `POST /app/installations/{id}/access_tokens` accepts both, MUTUALLY EXCLUSIVE. Contract passes numeric IDs → emit `"repository_ids":[ints]`. Add `repo_ids: Vec<u64>` to `MintRequest`; teach `build_token_request_body` to emit `repository_ids` when present; keep the existing `repositories` (names) path for other callers. This RPC populates ONLY repo_ids (sending both → 422).
2. **Per-call mint (not the installed provider):** installation_id is request-supplied → handler reads the sealed App key + builds a per-call `GitHubAppMint::new(app_id, req.installation_id, pem, clock, transport)`. The relay-provider/NativeSubtoken path (G2) is NOT used here.
3. **Secret names:** flat TASK-0020 convention — `github-app-private-key` (broker-only PEM), `github-app-id` (=4044997). installation_id from the REQUEST. If secrets not sealed → fail closed with an "App key not enrolled — run `secretctl github-app enroll` (TASK-0026)" message.
4. **expires_at_unix:** `ScopedToken.expires_at` is already an i64 epoch (engine converts GitHub RFC3339 via `.timestamp()` in `parse_token_response`). Emit as JSON number, NOT a string. Defensive negative-check.

## Engine API delta (logic in the engine — non-printing, seam-pure)
`Engine::mint_github_token(&self, params: GithubMintParams, sink: &EventSink) -> anyhow::Result<ScopedToken>`
with `GithubMintParams{ installation_id:u64, repository_ids:Vec<u64>, permissions:Vec<String>, ttl_secs:i64 }`.
Steps (mirror `open_mitm_ca_key` lib.rs:~535): vault read lock, require Unlocked (locked → `EngineError::Locked`, fail-closed); open `github-app-private-key` broker-only directly against the live DEK (NOT `secret_get`) → `Zeroizing` PEM; read `github-app-id`; build per-call `GitHubAppMint` with the engine clock + the HttpTransport seam; `mint_scoped`; emit METADATA-ONLY audit (installation_id, repo/perm counts, expires_at — never the token); return ScopedToken (Zeroizing).
**HttpTransport seam:** prefer an Engine field `github_transport: Box<dyn HttpTransport>` set in `with_seams` (daemon supplies `DaemonHttpTransport`, tests a fake; default errors → non-daemon builds fail closed). Reuse the EXISTING `DaemonHttpTransport` from transport.rs.

## Units (leaf-first)
1. proto: add the RPC + 2 messages to `service Vault` (unary, no apply/confirm field).
2. engine: `MintRequest.repo_ids` + `build_token_request_body` `repository_ids` branch + `mint_github_token` + the `github_transport` seam in `with_seams` (update the 3 `with_seams` callers: open_with_store, secretd `engine_with_daemon_seams`, test helper) + unit tests.
3. secretd: wire `engine_with_daemon_seams` to pass `DaemonHttpTransport` as `github_transport`; `mint_github` handler in grpc.rs (`i64::try_from(ttl)` bound check; parse repository_ids strings→u64, reject malformed; `spawn_blocking(engine.mint_github_token)`; map errors → Status: locked→failed_precondition, transport→unavailable, denial→permission_denied; build resp; NEVER log token).
4. secretctl: `mint-github` subcommand, EXACT frozen flags; `--output json` prints ONLY the two-field JSON (compact) to stdout; all logs to stderr.
5. DIFFERENTIAL contract test: serialize `MintGithubResp` via the CLI path, deserialize with the consumer's `Out{token:String, expires_at_unix:u64}` shape (read `flexnetos_github_app/crates/app-core/src/mint.rs` for exact names); assert argv matches `app-core::build_argv`.

## Invariants
No-C: reuse existing DaemonHttpTransport reqwest/rustls-ring + frozen webpki roots, NO new dep (`ci/gates/no-c.sh` green). One rustls ring-only. Engine non-printing (audit Events, logic in engine; only secretctl prints, only the frozen JSON). Fail-closed: locked vault / absent key / transport error / malformed input / empty token → error, never a fabricated/plaintext token; read-only RPC so no --apply (vault-unlock + USB possession is the gate). Token Zeroizing until the final stdout write; audit metadata-only.

## Sequencing
proto → engine (+seam, callers) → secretd transport-wire + handler → secretctl subcommand → differential test → `cargo fmt`/`clippy --workspace -D warnings`/`test` + 4 `ci/gates/*.sh`.

## Risks
repository_ids exclusivity (only one); repository_ids wire = repeated string parsed→u64 (reject non-numeric at handler boundary); stdout purity (only the JSON on stdout or the consumer's `serde_json::from_slice` breaks); expires_at i64→u64 safe (epoch positive, defensive check).
