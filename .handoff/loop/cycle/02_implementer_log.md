# Implementation log: TASK-0035 — secretd gRPC surface gaps (Vault/Relay/Audit/read)

Status: **GREEN** — engine-first, zero new deps, no proto change. All required gates clean.

## Changes
- `crates/secrets-engine/src/vault/store.rs`: added `Store::delete_secret(name) -> Result<u32>` trait
  method with default body `Ok(0)` (non-breaking); real `InMemStore` impl (retain-filter, returns
  count removed; does NOT rewind the row_id high-water).
- `crates/secrets-store-libsql/src/schema.rs`: added `DELETE_SECRET` SQL const
  (`DELETE FROM secrets WHERE name = ?`).
- `crates/secrets-store-libsql/src/store.rs`: real `delete_secret` impl via `conn.execute` (returns
  affected-row count). Straightforward — same pattern as the other write methods.
- `crates/secrets-engine/src/lib.rs`: new `SecretListItem` struct (non-secret fields + version +
  created_ts, `#[derive(Debug)]`); `#[derive(Debug)]` on `SecretMeta`; 7 new public engine methods
  (below) + inline `#[cfg(test)]` tests (9 new tests); added `delete_secret` to the in-module
  `SharedStore` fwd! mock.
- `crates/secrets-engine/tests/relay.rs`, `crates/secrets-engine/tests/vault.rs`: added `delete_secret`
  forward to each test `SharedStore` impl (so they forward to the real InMemStore, not the `Ok(0)`
  default).
- `crates/secretd/src/conv.rs`: added `method_str`, `secret_meta_to_proto`, `secret_list_item_to_proto`
  (→ `v1::SecretMeta`), `policy_to_proto` (engine `RelayPolicy` → `v1::RelayPolicy`); imported
  `SecretListItem`; 4 new inline tests.
- `crates/secretd/src/grpc.rs`: replaced the 6 `Status::unimplemented` bodies (Vault.List/Rm/Rotate,
  Relay.Create/List, Audit.Query) with spawn_blocking / run_streaming engine calls mapped via conv;
  populated `GetSecretResp.meta` via `engine.secret_meta`; added `engine_status` error mapper
  (Locked→failed_precondition, else internal); removed the now-stale `#[allow(dead_code)]` on
  `AuditSvc.engine`; updated module-doc unimplemented list (Certs.* only). Rm folds
  `apply = req.apply && req.confirm` (mirrors Relay.Revoke); empty-name → invalid_argument.
- `crates/secretd/tests/e2e.rs`: updated 2 Phase-6 assertions that the new behavior invalidates —
  step 4c now asserts `meta` IS populated (was `meta.is_none()`); step 4h now asserts Audit.Query
  RETURNS rows (was Unimplemented), and feeds those bytes into the wire-secrecy sentinel scan.
- `crates/secretd/tests/grpc_surface_e2e.rs`: **NEW** — 6 tokio e2e tests over the real server stack.
- `.handoff/loop/backlog.md`: marked TASK-0035 `[~]` (in review) with the API delta; appended
  **TASK-0038** for the deferred Certs.* / non-mitm ca_issue / secretctl ca / empty-features carve-out.

## Engine API delta (the parity contract — all sync, non-printing)
- `secret_list(provider: Option<Provider>, sink) -> Vec<SecretListItem>` — metadata-only; gates on
  unlock (Locked when `dek().is_none()`); optional provider filter; no audit row.
- `secret_meta(name) -> Option<SecretMeta>` — non-secret metadata; gates on unlock; UN-audited (Get
  already audits, avoids double-row).
- `secret_rm(name, apply, sink) -> u32` — DESTRUCTIVE; locked-refusal; `apply=false` counts
  would-remove via `list_secret_versions` (mutates nothing); `apply=true` removes via
  `Store::delete_secret`, audits `secret_removed` Ok, emits event. No secret bytes.
- `secret_rotate(name, new_value: Zeroizing<Vec<u8>>, apply, sink)` — carry-forward
  provider/note/broker_only from latest + `secret_put`; locked/unknown refusal; apply-gated dry-run;
  audits `secret_rotated`. No secret bytes.
- `relay_list(include_revoked, sink) -> Vec<RelayPolicy>` — filters revoked unless flag.
- `relay_create(policy, sink) -> i64` — additive via `save_relay_policy(RelayPolicyRow{id:0,..})`;
  audits `relay_created`.
- `audit_query(since_seq, limit, sink) -> Vec<AuditRecord>` — clamps limit ≤1000.
- New trait method `Store::delete_secret(name) -> Result<u32>` (default `Ok(0)`).
- New public struct `SecretListItem` (name/provider/note/broker_only/version/created_ts) + Debug on
  both `SecretMeta` and `SecretListItem`.

secretctl: `audit query` and `relay create` client verbs already exist and are correct — NO change
needed (Step 6 was a confirmation; the architect's "confirm or add" resolved to confirm).

## Tests added
Engine inline (`lib.rs`):
- `secret_list_is_metadata_only_and_provider_filtered` — both items, version=1, provider filter, no
  plaintext in serialized output.
- `secret_list_and_meta_refuse_when_locked` — both reads return EngineError::Locked when locked.
- `secret_meta_returns_non_secret_fields` — fields + None for unknown.
- `secret_rm_dry_run_mutates_nothing_apply_removes` — dry-run keeps row; apply removes + audit Ok +
  no plaintext in audit.
- `secret_rm_refuses_when_locked` — Locked + audit Refused row.
- `secret_rotate_dry_run_then_apply_appends_version` — dry-run no new version; apply appends v2,
  broker_only carried forward, audit Ok, no plaintext in audit.
- `secret_rotate_refuses_locked_and_unknown`.
- `relay_create_persists_and_list_filters_revoked` — create + audit; list excludes revoked, includes
  with flag.
- `audit_query_clamps_limit_and_returns_rows` — usize::MAX clamped ≤1000; rows present; no plaintext.

conv inline (`conv.rs`):
- `secret_list_item_to_proto_is_metadata_only`, `secret_meta_to_proto_carries_non_secret_fields`,
  `policy_to_proto_echoes_method_allow_and_mode`, `policy_to_proto_base_url_carries_upstream`.

secretd e2e (`grpc_surface_e2e.rs`, `#[tokio::test]`, real server stack):
- `vault_list_is_metadata_only_and_provider_filtered` (asserts no value byte leaks).
- `vault_rm_dry_run_then_apply_and_empty_arg_refused` (empty→InvalidArgument; dry-run no mutate;
  apply removes).
- `vault_rotate_appends_version_and_refuses_unknown` (broker_only carry-forward; unknown→stream err;
  empty→InvalidArgument).
- `relay_create_then_list_filters_revoked` (missing policy→InvalidArgument; method echo; revoke
  filter).
- `audit_query_returns_rows_and_get_meta_populated` (Get.meta populated; audit returns rows; no leak).
- `locked_vault_refuses_list_with_failed_precondition` (Locked→FailedPrecondition).

## Build/test status (run from the worktree, via `rtk proxy` to avoid corrupting exit codes)
- `cargo fmt --all --check` → PASS (exit 0).
- `cargo clippy --workspace -- -D warnings` (the repo's exact CI clippy invocation, `.github/
  workflows/ci.yml:49`) → PASS (exit 0).
- `cargo clippy -p envctl-secrets-engine -p envctl-secretd -p envctl-secrets-store-libsql
  --all-targets -- -D warnings` (my touched crates, stricter) → PASS (exit 0).
- `cargo test -p envctl-secrets-engine -p envctl-secrets-store-libsql` → PASS (105 lib + relay 17 +
  vault 15 + libsql binding 11/4/6 + 7 ignored live-sqld). exit 0.
- `cargo test -p envctl-secretd` → PASS (exit 0): lib 35 (incl. 6 conv), e2e 6, grpc_surface_e2e 6,
  proxy_swap 2, self_check 2, native_mint/mint_github happy-path (default features =
  mitm-ca+provider-github).
- `bash ci/gates/no-c.sh` → PASS (rustls=0.23.40 on ring; zero aws-lc/openssl/C-SQLite; engine still
  never links libsql).
- `bash ci/gates/shape.sh` → PASS.

### Baseline note (pre-existing drift, NOT mine)
`cargo clippy --workspace --all-targets -- -D warnings` fails on
`crates/gui/src/main.rs:1997` (`clippy::doc_lazy_continuation` in GUI test-doc). GUI is NOT in my
changeset (`git diff --name-only` confirms). The repo's CI clippy does **not** use `--all-targets`
(it runs `cargo clippy --workspace -- -D warnings`), so this lint is not in the gate and not
attributable to TASK-0035. Surfaced only because `--all-targets` lints test code.

## Deviations
- **Step 1 libSQL delete_secret:** implemented the REAL `DELETE FROM secrets WHERE name = ?` (the
  plan's "if straightforward" branch — it was), not the default-stub. Like ALL libSQL store CRUD it
  is unit-tested only at the serialization layer; the live mutation path is exercised by the
  `#[ignore]`d integration tests against a live sqld (existing crate convention), so it ran as
  "0 passed; 7 ignored" here. It compiles + type-checks green.
- **Step 6 secretctl:** the `audit query` and `relay create` client verbs already existed and were
  correct — no code change (confirmation only, per the plan's "confirm/add").
- **Debug derives:** added `#[derive(Debug)]` to `SecretMeta` and `SecretListItem` (needed by
  `.unwrap_err()` in the locked-refusal tests). Both carry non-secret metadata only, so Debug is
  safe (no value/nonce/ct_tag field exists to print).
- **secret_rm event:** reused `SecretEvent::GuardRefused` as the post-removal notification carrier
  (no `SecretRemoved` variant exists in the event enum, and adding one is an enum change outside the
  plan's "emit SecretEvent" intent). The durable audit row (`secret_removed`/Ok) is the authoritative
  record; the event is cosmetic. Guardian: flag if a dedicated `SecretRemoved` variant is wanted —
  that's an event-enum + conv `event_to_proto` change (a small follow-up, not required for parity).
- **secret_rotate event:** reused `SecretEvent::RelayRotated{relay:name, expires_at:""}` similarly
  (no `SecretRotated` variant). `secret_put` also emits `SecretWritten` on apply, so the rotation is
  observable; the `secret_rotated`/Ok audit row is authoritative.

## Follow-ups (feed the wrap-up backlog reconcile)
- **TASK-0038** appended to backlog: Certs.* service + non-mitm ca_issue + secretctl ca + empty
  `provider-openai`/libsql `embedded` features — Phase 4+ (deferred from TASK-0035).
- Pre-existing GUI clippy `--all-targets` drift (`gui/src/main.rs:1997`) — not in CI gate; could be a
  one-line cleanup in a separate hygiene pass.
- Optional event-enum enrichment: dedicated `SecretEvent::SecretRemoved` / `SecretRotated` variants +
  proto twins, if the control stream should surface these distinctly (currently surfaced via
  Audit.Query, which is the design intent).
- `policy_to_proto.expires_at` surfaces the relative `policy_ttl_secs` as a string (the engine stores
  relative TTL, not an absolute timestamp); a future schema bump could carry a resolved RFC3339.

## Handoff notes (for the invariant-guardian — targeted checks)
- **Destructive fail-closed:** `secret_rm`/`secret_rotate` refuse when locked (verify
  `secret_rm_refuses_when_locked`, `secret_rotate_refuses_locked_and_unknown`,
  `locked_vault_refuses_list_with_failed_precondition`) and are dry-run by default (proto3 default
  `apply=false`). Rm additionally folds `apply && confirm` at the daemon (mirrors Relay.Revoke).
- **No secret bytes leak:** the no-leak assertions live in
  `secret_list_is_metadata_only_and_provider_filtered`, the two rm/rotate apply tests (audit scan),
  `audit_query_clamps_limit_and_returns_rows`, and the e2e SENTINEL scans in
  `vault_list_*`/`audit_query_*` + the existing e2e wire-secrecy assertion (now also fed Audit.Query
  bytes). `SecretListItem`/`SecretMeta` have NO nonce/ct_tag/value field by construction.
- **Reveal gate untouched:** I did not modify `secret_get`'s reveal/broker_only path. List/meta
  expose `broker_only` as a bool flag only; `ca_key_not_revealable_via_secret_get` still passes.
- **No-C / one-rustls:** zero new deps; `no-c.sh` PASS; engine still never links libsql
  (the libSQL delete is in the quarantined `secrets-store-libsql` crate, behind the Store trait).
- **Engine non-printing/sync:** all 7 methods are sync, emit events + audit rows, no println!/clap.
- **delete_secret default body:** the `Ok(0)` default keeps every existing Store impl compiling; the
  two test `SharedStore`s + the in-module mock all forward it explicitly so behavior is real in tests.
