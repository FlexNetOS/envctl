# Verification report: TASK-0035 — secretd gRPC surface gaps (Vault/Relay/Audit/read)

## Verdict — **PASS**

Independent cross-boundary verification of the working-tree TASK-0035 changeset (working tree vs
`HEAD`, since the task work is uncommitted on top of the committed TASK-0020 #105). Every
NON-NEGOTIABLE invariant holds; all real gates + cargo checks are green from raw `rtk proxy`
passthrough (verified exit codes, not the implementer's word).

### Changeset scope (TASK-0035 = `git diff HEAD`, 12 files + 1 new test file)
`crates/secrets-engine/src/{lib.rs,vault/store.rs}`, `crates/secrets-store-libsql/src/{schema,store}.rs`,
`crates/secretd/src/{conv,grpc}.rs`, `crates/secretd/tests/e2e.rs`, `crates/secrets-engine/tests/{relay,vault}.rs`,
NEW `crates/secretd/tests/grpc_surface_e2e.rs`, + 3 handoff docs. **No proto file in the changeset.**

## Gate results
| Gate | Command | Exit | Result |
|------|---------|------|--------|
| no-c | `bash ci/gates/no-c.sh` | 0 | **PASS** — `rustls=['0.23.40'] on ring=['0.17.14']; zero aws-lc/openssl/C-SQLite` |
| shape | `bash ci/gates/shape.sh` | 0 | **PASS** — `SHAPE GATE PASS` |

## cargo
| Check | Command | Exit | Result |
|-------|---------|------|--------|
| fmt | `rtk proxy cargo fmt --all -- --check` | 0 | **PASS** |
| clippy | `rtk proxy cargo clippy --workspace -- -D warnings` (exact CI form, `ci.yml:49`) | 0 | **PASS** |
| test | `rtk proxy cargo test -p envctl-secrets-engine -p envctl-secretd` | 0 | **PASS** |

Test counts: secrets-engine lib **127**, relay **17**, vault **15** (+ libsql binding 11/4/6, 1 ignored
live-sqld for `delete_secret`); secretd lib **35** (incl. conv tests), e2e **5**, **grpc_surface_e2e 6**,
proxy_swap 2, self_check 2, native_mint/mint_github 1. **0 failed.**

CI clippy form confirmed at `.github/workflows/ci.yml:48-49` = `cargo clippy --workspace -- -D warnings`
(NO `--all-targets`). The pre-existing `crates/gui/src/main.rs:1997` `doc_lazy_continuation` lint only
fires under `--all-targets`, is NOT in CI's gate, and `gui/` is NOT in the TASK-0035 changeset.
**Not attributable / not blocking** — implementer's claim confirmed.

## Invariant checks
1. **No secret bytes in List/meta/audit/logs — PASS.** `SecretListItem` (lib.rs:195) and `SecretMeta`
   carry only non-secret `SecretRow` fields (name/provider/note/broker_only + version/created_ts) — no
   `nonce`/`ct_tag`/value field exists by construction. `secret_list`/`secret_meta`/`audit_query`
   (lib.rs:859/900/1070) never read or emit plaintext. `secret_rotate` holds `new_value` in
   `Zeroizing` (lib.rs:972; daemon moves the proto buffer into `Zeroizing` immediately in grpc.rs
   rotate). Audit details carry only counts/versions (`would_remove`/`removed`/`version`). Enforced by
   SENTINEL scans: `secret_list_is_metadata_only_*`, the rm/rotate apply audit scans,
   `audit_query_clamps_*`, and the e2e wire-secrecy assertion (now also fed Audit.Query bytes, e2e.rs:355).
2. **Destructive ops fail-closed + dry-run default — PASS.** `secret_rm`/`secret_rotate` refuse on
   locked vault (write Refused audit row + GuardRefused event BEFORE returning → daemon
   `failed_precondition` via `engine_status`, grpc.rs:51). `apply=false` (proto3 default) mutates
   nothing — rm counts via `list_secret_versions`, rotate confirms+drops the Zeroizing value. Daemon
   folds `apply = req.apply && req.confirm` for Vault.Rm (mirrors Relay.Revoke). Refusal paths
   unit+e2e tested: `secret_rm_dry_run_mutates_nothing_apply_removes`, `secret_rm_refuses_when_locked`,
   `secret_rotate_dry_run_then_apply_appends_version`, `secret_rotate_refuses_locked_and_unknown`,
   `vault_rm_dry_run_then_apply_and_empty_arg_refused`, `locked_vault_refuses_list_with_failed_precondition`.
3. **Engine = single sync non-printing library — PASS.** No `println!`/`eprintln!`/`print!`/clap/UI
   and no `async fn`/`.await` added to secrets-engine (diff grep clean). All 7 new methods are sync,
   emit Events + audit rows; secretd handlers are thin spawn_blocking/run_streaming wrappers over the
   engine. Logic landed in the engine, not the daemon.
4. **Broker-only never revealable — PASS.** `secret_get`/reveal path NOT in the diff (the only
   `broker_only` mention in the diff is a test comment). List/meta expose `broker_only` only as a bool
   flag. `ca_key_not_revealable_via_secret_get` and the e2e broker_only sentinel assertion still pass.
5. **delete_secret correctness — PASS.** Trait default `Ok(0)` (store.rs:145, non-breaking). InMemStore
   impl = retain-filter returning the correct removed count, deliberately does NOT rewind the row_id
   high-water (store.rs:373). libSQL impl = `DELETE FROM secrets WHERE name = ?` parameterized, returns
   affected-row count (store.rs:321; SQL const schema.rs:127). In-module `SharedStore` fwd! mock + the
   two integration test `SharedStore`s forward `delete_secret` explicitly → behavior is real in tests.
   Whole-workspace compile + 127/17/15 engine tests prove all Store impls compile.
6. **No proto change — PASS.** `git diff HEAD -- crates/secrets-proto/proto/control.proto` is EMPTY.
   (The proto delta visible vs `develop` belongs to committed TASK-0020 #105 `MintGithub` + G2
   `DataPlaneMode mode` — NOT this task.) **Zero new deps:** no `Cargo.lock`/`Cargo.toml` in the
   changeset; `cargo tree -p envctl-secrets-engine` shows no libsql/sqlite/openssl/aws-lc — engine
   still never links libSQL (the libSQL delete lives in the quarantined `secrets-store-libsql` crate
   behind the Store trait).

## Parity check (front-end reach)
Secrets stack: `secretctl` is the front-end for `secrets-engine` (GUI out of scope per plan §10).
Engine method → daemon RPC → CLI verb:
- `secret_list` → `Vault.List` (grpc.rs list) → `secretctl secret list` (pre-existing verb)
- `secret_rm` → `Vault.Rm` (grpc.rs rm, apply&&confirm) → `secretctl secret rm` (pre-existing)
- `secret_rotate` → `Vault.Rotate` (grpc.rs rotate) → `secretctl secret rotate` (pre-existing)
- `secret_meta` → `GetSecretResp.meta` populated in `Vault.Get` (grpc.rs:182) → `secretctl secret get`
- `relay_create` → `Relay.Create` (grpc.rs create) → `secretctl relay create` (confirmed pre-existing)
- `relay_list` → `Relay.List` (grpc.rs list) → `secretctl relay list` (pre-existing)
- `audit_query` → `Audit.Query` (grpc.rs query, daemon post-filter) → `secretctl audit query` (confirmed)
All in-scope RPCs proto+CLI were pre-wired; this cycle filled the engine+daemon gap. No CLI change
required (plan Step 6 resolved to confirmation only).

## Findings
None blocking. Notes / non-blocking observations:
- **N1 (cosmetic):** `secret_rm` apply emits `SecretEvent::GuardRefused` and `secret_rotate` apply
  emits `SecretEvent::RelayRotated` as removal/rotation notification carriers (no dedicated
  `SecretRemoved`/`SecretRotated` enum variant exists; adding one is an event-enum + `event_to_proto`
  change outside scope). The reason/relay strings carry ONLY name + count — no secret. Authoritative
  record is the durable `secret_removed`/`secret_rotated` audit row. Acceptable; an optional future
  enrichment (already in the implementer's follow-ups). No secrecy/invariant impact.
- **N2 (note):** `policy_to_proto.expires_at` surfaces the relative `policy_ttl_secs` as a string (the
  engine stores relative TTL, not an absolute timestamp). Operator-facing, documented; a future schema
  bump could carry a resolved RFC3339. Not a TASK-0035 defect.
- **N3 (note):** Audit.Query `since`/`until` daemon-side filter uses lexical RFC3339 compare (correct
  for a fixed offset, as the code comments note). Metadata filtering only.
- **N4 (deferred, recorded):** Certs.* / non-mitm `ca_issue` / `secretctl ca` / empty-features
  carve-out correctly deferred to **TASK-0038** (appended to backlog). The 4 Certs.* RPCs remain
  `Status::unimplemented` by design; module-doc updated accordingly.

## Re-test needed
None — all gates and tests green on this changeset. If the cosmetic event-enum enrichment (N1) is
pursued, re-run `cargo test -p envctl-secrets-engine -p envctl-secretd` + `cargo clippy --workspace
-- -D warnings`. Routing for N1, if wanted: rust-implementer (small event-enum + conv change).
