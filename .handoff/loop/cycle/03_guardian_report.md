# Verification report: TASK-0031 PR-1 — F2 remote relay-edge listener (in-process TLS + DPoP/EKM)

## Verdict — **PASS**

Independent, adversarial verification of the new default-OFF `relay-edge` HTTPS edge in
`crates/secretd/src/edge/{mod,dpop,tls,listener}.rs` (+ additive engine seam) against every
NON-NEGOTIABLE invariant and the real CI gates. All gates green, both feature states clean,
every security invariant confirmed against the actual code (not the implementer's claims).
Zero blocking findings. Two non-blocking notes recorded below.

> Note on the PR-1 delta surface: the edge work is **uncommitted in the worktree** (all three
> branch refs point at the same commit `755ebb2`). The correct delta is therefore *working-tree
> vs HEAD*, NOT vs `develop` — a `develop` diff is polluted by already-merged Epic-C/TASK-0030
> crates (`baby-mimalloc`, `envctl-agent-env`, `tar`, `filetime`, `xattr`). I verified against
> HEAD throughout. (`baby-mimalloc` is a pure-Rust allocator, NOT the C `libmimalloc-sys`, and is
> not reachable from the secretd trust boundary — see gate results.)

## Gate results
| Gate | Command | Exit | Result |
|------|---------|------|--------|
| no-c | `bash ci/gates/no-c.sh` | 0 | **PASS** — `rustls=['0.23.40'] on ring=['0.17.14']; zero aws-lc/openssl/C-SQLite` |
| shape | `bash ci/gates/shape.sh` | 0 | **PASS** — FS-S25/REQ-SEC-11 edge greps armed + scanning `crates/secretd/src/edge` |
| enable | `bash ci/gates/enable.sh` | 0 | **PASS** |
| p7 | `bash ci/gates/p7.sh` | 0 | **PASS** |

## cargo
| Check | Command | Exit | Result |
|-------|---------|------|--------|
| fmt | `cargo fmt --all -- --check` | 0 | **PASS** |
| clippy OFF | `cargo clippy --workspace -- -D warnings` | 0 | **PASS** |
| clippy ON | `cargo clippy --workspace --features relay-edge -- -D warnings` | 0 | **PASS** |
| test OFF | `cargo test -p envctl-secretd` | 0 | **PASS** — secretd-lib 31, e2e 5, mitm 1, native_mint 11, proxy_swap 2, self_check 2; `edge_e2e` runs **0** tests (cfg-gated absent), edge module not compiled |
| test ON | `cargo test -p envctl-secrets-engine -p envctl-secretd --features relay-edge` | 0 | **PASS** — **17** dpop unit vectors all pass; `edge_e2e::edge_dpop_swap_accepts_and_rejects` PASS; engine `remote_no_dpop_fails_closed` PASS; `relay_swap_remote_unverified_dpop_denied_no_dpop` PASS; secretd-lib 52, engine-lib 129, relay 18, vault 15. **0 failed** |

## Invariant checks (independently confirmed against source)
1. **No-C / one rustls ring-only** — PASS. Cargo.lock working-tree-vs-HEAD adds **ZERO `[[package]]`** entries; the only delta is 4 dependency *edges* on `envctl-secretd` (`base64`, `rcgen`, `ring`, `sha2`) — all crates already in the graph. `cargo tree -p envctl-secretd --features relay-edge -e normal` → 0 banned crates (aws-lc/openssl/native-tls/mimalloc); single `rustls` package = 0.23.40 (lockfile has exactly one `[[package]] name="rustls"`). An edge adding a TLS server pulled **no** aws-lc-rs and **no** second rustls.
2. **decide() truly untouched + fail-closed reaches the engine** — PASS. `git diff HEAD -- crates/secrets-engine/src/broker/decide.rs` and `git diff develop -- …decide.rs` are **both empty**. `broker/jti.rs` untouched (consumed read-only). Engine test `tests/relay.rs::relay_swap_remote_unverified_dpop_denied_no_dpop` (lines 724-786) drives `relay_swap` with `remote: Some(RemotePeer{dpop_verified:false})` → asserts `Denied(RemoteNoDPoP)` AND `cap.0.lock().is_none()` (**real key NEVER fetched**). The edge cannot manufacture an Allow by skipping a check — the engine re-asserts (`broker::decide::tests::remote_no_dpop_fails_closed` also passes).
3. **EKM binding (FS-S20) is real, not cosmetic** — PASS. `dpop.rs:138` rejects `EkmUncomputable` *before touching the proof*; `dpop.rs:243-252` requires the proof's `ekm` claim present AND byte-equal to the connection EKM (`EkmMissing`/`EkmMismatch` otherwise). `listener.rs:132-134` reads EKM via the **real** rustls 0.23 API `tls_stream.get_ref().1.export_keying_material(out, EKM_LABEL, None)` (not a stub); `None` ⇒ `dpop_verified` never set. `listener.rs:273-278` maps the three `Ekm*` rejects → **403**. Vectors `uncomputable_ekm_rejected_failclosed`, `ekm_mismatch_rejected`, `ekm_claim_absent_rejected` all pass; e2e binds the *client-side* EKM into the proof and gets 200 (symmetric RFC 5705 export proven end-to-end).
4. **relay-tls ONLY, never MITM CA (FS-S25)** — PASS. `RelayTlsConfig::load_from_dir` (`tls.rs:38`) reads ONLY `relay_tls_dir/{cert,key}.pem`, imports no MITM-CA type, has no fallback; missing dir/key/empty cert all fail closed (4 tests pass). Symbol grep over `crates/secretd/src/edge/` for `mitm_ca|local_ca|MitmCertResolver|issue_leaf|ResolvesServerCert` → **0 code references** (only doc-comment prose mentions "MITM CA"). `shape.sh:27-34` arms the FS-S25/FS-S18 symbol grep over `EDGE_SRC` and passes. `relay_tls_dir()` = `config/relay-tls` (sibling of `secretd.toml`), explicitly NOT the MITM-CA `data` dir (unit-tested, `paths.rs:77-95`).
5. **Fail-closed completeness** — PASS. `listener.rs::verify_remote_presentation` traces every reject BEFORE `swap_and_respond`: missing/empty DPoP→401 (256), missing bearer→401 (264), `Ekm*`→403 / all other DPoP rejects→401 (273-279), **poisoned `Mutex<JtiReplayStore>` → 401, NOT `.unwrap()`** (285-288), replayed/drift jti→401 (290-300), no proof client_id→401 (308-311), unknown/disabled/revoked client OR store error → fail-closed 401 (316-325), proven-jkt ≠ registered-jkt → 401 (318-321). `RemotePeer{dpop_verified:true}` is constructed ONLY after all of EKM-bound + DPoP-verified + jti-fresh + client-registered+enabled + jkt-match. `InternalRefused→503` mapping in the swap core. **`awk`-scan (excluding `#[cfg(test)]`) finds ZERO `.unwrap()`/`.expect()`/`panic!`/`unreachable!`/`[idx]` on attacker-reachable input across the whole edge tree.**
6. **No secret bytes in logs** — PASS. Grep of all `tracing::*!` macros in `edge/` for `bearer|dpop|proof|ekm|api_key|secret|private_key|token` → **0 matches**. Log lines carry only `peer`/`status`/`error`-display/`client_id`. The engine emits the secret-free durable audit row.
7. **Engine purity / non-printing** — PASS. No `println!`/`eprintln!`/`print!`/`stdout` in `edge/` (only a doc-comment *saying* "no `println!`"). MINT + DECIDE stay in `relay_swap`/`decide()`; the edge does I/O + proof verification only, enforces no policy (upstream host/path ride headers and are re-fenced by `decide()`'s allowlist). `EgressReq.remote` is purely additive — all existing constructors set `None` (compile-proven; local-plane behavior in `swap_and_respond` unchanged when `remote.is_none()`).
8. **Off-by-default** — PASS. `crates/secretd/Cargo.toml:17` `default = ["mitm-ca", "provider-github"]` — `relay-edge` is **not** in defaults. The whole edge module/config/startup is `#[cfg(feature="relay-edge")]`-gated; `[edge].enabled` defaults `false`; `bind_addr` required only when enabled (fail-closed); absent `[edge]` block ⇒ no bind. Feature-OFF test run confirms `edge_e2e` compiles to 0 tests and the module isn't built. Cert-load/bind failure is FATAL only when the operator explicitly enabled the edge.

## Parity check
This is a secretd daemon network surface (no CLI/GUI front-end) — front-end parity N/A. The
relevant parity is **proxy↔edge plane parity**: both drive the SAME `proxy::swap_and_respond`
core, so the two planes cannot diverge in how they reach `relay_swap`/`decide()`.
- `Engine::relay_swap` (`crates/secrets-engine/src/lib.rs:~1267`) ← local proxy `proxy.rs::swap_and_respond` (remote=`None`) AND edge `listener.rs:223` `swap_and_respond(.., Some(rp))`.
- `Engine::load_remote_client` (`lib.rs`, additive read accessor) ← edge `listener.rs:316`.
- `Paths::relay_tls_dir()` (`paths.rs:67`) ← edge `mod.rs:54` / `tls.rs::load_from_dir`.

## Findings
None blocking. Two non-blocking notes:
- **NOTE (informational, not a defect):** the worktree's PR-1 changes are **uncommitted**. The orchestrator must commit working-tree state before merge; a `develop` diff will mislead (it folds in unrelated Epic-C crates). Verification above used HEAD-relative diffs and is sound.
- **NOTE (scope, already documented):** identity for the registry lookup is taken from the proof's `client_id` claim (`listener.rs:308`), which `dpop.rs` documents as "not trusted for identity." This is defensible here because the registered client's `dpop_jkt` is re-asserted equal to the *proven* key (`listener.rs:318`) and `decide()` clause 11a independently re-binds the bearer's own `client_id` — a forged claim cannot escalate. Worth keeping in mind for PR-2's mTLS/nonce hardening. No change required for PR-1.

## Re-test needed
None — PASS as delivered. If the orchestrator amends anything before merge, re-run the changed
surface:
```bash
bash ci/gates/no-c.sh && bash ci/gates/shape.sh && bash ci/gates/enable.sh && bash ci/gates/p7.sh
cargo fmt --all -- --check
cargo clippy --workspace -- -D warnings
cargo clippy --workspace --features relay-edge -- -D warnings
cargo test -p envctl-secretd                                              # feature OFF
cargo test -p envctl-secrets-engine -p envctl-secretd --features relay-edge   # feature ON
```
