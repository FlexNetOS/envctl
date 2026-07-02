# Test strategy — weave (test-coverage dimension, cycle 4)

- Target: **weave** (A2A transport plane / "nervous system"). Snapshot `@4fe2419` (codemap) · RED branch `plan/weave-red-tests`.
- Authored worktree (isolated): `/home/drdave/Desktop/meta/.worktrees/plan-weave-red/weave`
- Scope of authoring: weave-core (the spine; zero internal weave deps — builds standalone, no external escape).
- Read-only on product code; tests are the one additive mutation. Every claim cites file:line.

tests-ran: 3

---

## Coverage shape (existing tests, by reachability — not file presence)

- `git-kb code` reports 809 inferred entrypoints "dominated by `#[test]`/test-harness symbols" (graph §2): weave is heavily tested overall. Existing suites: `weave/tests/{integration,security,prop}.rs` (CLI/E2E + proptest) plus dense `#[cfg(test)]` modules in every weave-core module (`store.rs`, `model.rs`, `sign.rs`, `config.rs`, …) and weave-mcp (`mcp.rs`, `obscura.rs`, `dashboard.rs`).
- The **native `Intent` wire path** IS covered: `weave/tests/integration.rs:3541` `tier2_dedup_keyed_on_intent_id_not_content`, `:3646` `tier2_misaddressed_intent_not_committed`, `weave/tests/security.rs:1388` `signed_intent_failing_verification_is_always_rejected` exercise pull/commit/dedup/sign over the weave-native schema.
- The gap is **interop**, not the native path.

### CLAIM rows

- CLAIM: the A2A v1.0 convergence seam (`Intent` <-> A2A message mapping `to_a2a`/`from_a2a`) has ZERO test caller and ZERO implementation | evidence: no `to_a2a`/`from_a2a`/`message/send`/`AgentCard` symbol exists anywhere in tree (grep over `**/*.rs` excluding new suite: 0 hits); `Intent` (`weave-core/src/model.rs:216`) derives only flat serde, serialized keys = `[id,ts,to,to_host,from,subject,body,sig,idempotency_key,trace_id,priority,ttl]` (observed at runtime) | confidence: high
- CLAIM: the `jsonrpc:"2.0"` strings present in tree are weave's **MCP** protocol, not A2A — distinct standard | evidence: `weave-mcp/src/obscura.rs:212,235`, `weave-mcp/src/mcp.rs:4098,6473` all use MCP methods (`tools/call`, `notifications/initialized`), never A2A `message/send` | confidence: high
- CLAIM: existing `Intent` tests prove the weave-NATIVE Tier-2 contract but none assert any industry-A2A wire shape | evidence: `weave/tests/integration.rs:3541,3646`; `weave/tests/security.rs:1388` (native dedup/sign only) | confidence: high
- CLAIM: `model.rs` is the single highest-blast file (impact 1238, graph §4) yet its contract evolution toward A2A is unguarded — a schema change there with no interop test is high blast-radius risk | evidence: graph/weave.graph.md §4 blast(model.rs)=1238; `Intent` at model.rs:216 | confidence: high
- CLAIM: weave-core defines NO `default` feature (`weave-core/Cargo.toml [features]` lists `sqlite,libsql,sign,llm,surfaces,obscura` with no `default=`), so `cargo test -p weave-core` runs with zero features — the new suite is feature-independent (serde_json is a non-optional dep; `model` is always compiled via `lib.rs`) | evidence: `weave-core/Cargo.toml`; `weave-core/src/lib.rs:1` | confidence: high

---

## Suite authored this cycle (additive RED — compiles, RUNS, FAILS on assertion)

File: `weave-core/tests/a2a_interop.rs` (new; 3 integration tests). Authored to compile against the **existing** `Intent` via serde_json (no unbuilt symbol referenced) so failure is on ASSERTION, not compile — the correct RED. Migrate to drive `to_a2a`/`from_a2a` directly once the adapter lands.

Command: `cargo test -p weave-core --test a2a_interop`
Result: `test result: FAILED. 0 passed; 3 failed; 0 ignored; 0 measured; 0 filtered out` — tests-ran = **3** (>0; not a fail-open exit-0).

| test | criterion | expected RED reason (observed) |
|---|---|---|
| `intent_serializes_to_a2a_message_object` | A2A-1 to_a2a Message-object mapping | `kind` absent — serialized Intent is flat `[id,ts,to,to_host,from,subject,body,sig,...]`, no `kind/role/messageId/parts` (panic at a2a_interop.rs:60) |
| `a2a_message_deserializes_into_intent` | A2A-2 from_a2a inbound parse | `from_value::<Intent>` = `Error("missing field \`id\`")` → `is_ok()` false (panic at a2a_interop.rs:120) |
| `intent_frames_as_a2a_jsonrpc_request` | A2A-3 JSON-RPC 2.0 envelope | `jsonrpc` absent — no `{jsonrpc,method:"message/send",params.message}` framing (panic at a2a_interop.rs:156) |

### UPGRADE rows

- UPGRADE: add integration test for `Intent` -> A2A v1.0 Message mapping (`to_a2a`) | axis: accuracy | rationale: closes the A2A-1 interop gap — proves id->messageId, body->parts[0].text, kind="message"/role present | evidence: `weave-core/tests/a2a_interop.rs:54`; gap CLAIM above (model.rs:216) | blast: guards the cross-vendor send envelope (model.rs blast 1238) | risk: low
- UPGRADE: add integration test for A2A v1.0 Message -> `Intent` parse (`from_a2a`) | axis: accuracy | rationale: closes A2A-2 — proves inbound foreign-A2A messages deserialize into the mailbox with from/to/subject/body recovered | evidence: `weave-core/tests/a2a_interop.rs:104` | blast: guards inbound interop into `Store::send` | risk: low
- UPGRADE: add integration test for the A2A JSON-RPC 2.0 transport envelope | axis: accuracy | rationale: closes A2A-3 — proves outbound framing (`jsonrpc/method/params.message`) distinct from the MCP envelope already in tree | evidence: `weave-core/tests/a2a_interop.rs:151`; MCP-vs-A2A CLAIM | blast: guards the transport-framing layer | risk: low

### Designed (not yet authored — for Feature Forge to add alongside the adapter)

- UPGRADE: add **property/round-trip** test `from_a2a(to_a2a(intent)) == intent` over core fields (id/from/to/subject/body) | axis: accuracy | rationale: the adapter's round-trip invariant; proptest already a dev-dep (`weave-core/Cargo.toml [dev-dependencies] proptest = "1.8.0"`) | evidence: existing `weave/tests/prop.rs` precedent | blast: schema round-trip | risk: low
- UPGRADE: add test that weave's optional `sign` (ed25519) produces an **A2A signed-AgentCard**-shaped signature (reuse `weave-core/src/sign.rs`) | axis: accuracy | rationale: research §A2 — `sign` is the local analogue of A2A v1.0 signed Agent Cards; gate behind `sign` feature | evidence: `weave-core/Cargo.toml sign = [...]`; trends §A2 | blast: cross-org trust-before-interaction | risk: low

---

## traceability (plan-item <-> acceptance-criterion <-> test <-> RED/GREEN)

| plan item | acceptance criterion | test path::name | status |
|---|---|---|---|
| A2A interop: outbound Message mapping | Serialized Intent is an A2A Message (`kind="message"`, `role`, `messageId`=Intent.id, `parts[0].text`=Intent.body) | `weave-core/tests/a2a_interop.rs::intent_serializes_to_a2a_message_object` | **RED** (no `kind`; flat shape) |
| A2A interop: inbound Message parse | An A2A v1.0 Message JSON deserializes into `Intent` with from/to/subject/body recovered | `weave-core/tests/a2a_interop.rs::a2a_message_deserializes_into_intent` | **RED** (`missing field id`) |
| A2A interop: JSON-RPC transport framing | Outbound A2A send framed as `{"jsonrpc":"2.0","method":"message/send","params":{"message":{…}}}` | `weave-core/tests/a2a_interop.rs::intent_frames_as_a2a_jsonrpc_request` | **RED** (no `jsonrpc`) |
| (designed) A2A round-trip invariant | `from_a2a(to_a2a(i))` preserves id/from/to/subject/body | (Feature Forge to author) | not-yet-authored |
| (designed) A2A signed AgentCard | `sign` produces an A2A-card-shaped ed25519 signature | (Feature Forge, `--features sign`) | not-yet-authored |

Commit (RED suite, branch `plan/weave-red-tests`, NOT pushed): `b7f466f485dc1e5bce00d2892a843dad9a24d8f7`

---

## FF test-build spec (GREEN handoff for Feature Forge)

Intake shape: `feature-architect ## Verification plan`. The RED suite below is committed and FAILING — Feature Forge implements the A2A adapter until it goes GREEN, additively (never remove the SQLite-mailbox transport; A2A is a strict upgrade, research §A1/§B1).

- **Test surface (exists, additive only):**
  - `weave-core/tests/a2a_interop.rs` — committed RED suite (3 cases). On adapter landing, migrate the 3 cases to drive `Intent::to_a2a()` / `Intent::from_a2a()` directly (currently assert wire-shape to stay compiling without the unbuilt fns).
  - `weave-core/src/model.rs` (or a new `weave-core/src/a2a.rs`) — home for `to_a2a`/`from_a2a` + the A2A `Message`/`JsonRpcRequest` types. NB `model.rs` blast = 1238; keep the adapter additive (new fns/module), do not re-derive `Intent`'s existing serde (the native Tier-2 path at `weave/tests/integration.rs:3541` must stay GREEN).
- **Concrete cases (one bullet each — symbol/flow + assertion):**
  - `to_a2a`: `Intent{id:7,from:"alice",to:"bob",subject:Some("status"),body:"build is green"}` -> A2A Message with `kind=="message"`, `messageId=="7"`, `parts[0].text=="build is green"`, a `role`. (a2a_interop.rs:54)
  - `from_a2a`: A2A Message JSON (role/parts/messageId + metadata.from/to/subject) -> `Intent` with `.from=="alice"`, `.to=="bob"`, `.body=="build is green"`, `.subject==Some("status")`. (a2a_interop.rs:104)
  - JSON-RPC envelope: outbound send frames as `{"jsonrpc":"2.0","method":"message/send","params":{"message":{…}}}`. (a2a_interop.rs:151)
  - (add) round-trip property: `from_a2a(to_a2a(i))` preserves id/from/to/subject/body (proptest, dev-dep present).
  - (add, `--features sign`) AgentCard signature shape over `sign.rs`.
- **Differential/golden fixtures to capture:** one golden A2A v1.0 `message/send` JSON-RPC request fixture (validate against the published A2A v1.0 schema, research §A1) — diff the adapter's output against it. Behavior-preserving guard: the native Tier-2 dedup/commit goldens (`integration.rs:3541/3646`) must remain unchanged (adapter is additive, not a re-shape of `Intent`'s native serde).
- **Coverage target:** the 2 contract-bearing adapter symbols (`to_a2a`, `from_a2a`) each reached by >=1 test; the A2A-1/2/3 criteria all GREEN; native Tier-2 + sign suites still GREEN (no regression).
- **CI gates touched:** `cargo test -p weave-core` (new integration test binary `a2a_interop`); `cargo fmt --check` + `cargo clippy` (preflight subset); if AgentCard case added, the `sign`-feature build lane.

---

## Verdict (for the verifier)

weave is well-tested on its NATIVE wire path but has ZERO coverage and ZERO implementation of the A2A v1.0 interop seam — the cycle's primary convergence finding. Authored an additive 3-test RED suite (`weave-core/tests/a2a_interop.rs`) encoding the to_a2a / from_a2a / JSON-RPC-envelope criteria; tests-ran=3, all RED on assertion (not compile). Committed `b7f466f` on `plan/weave-red-tests` (unpushed). Suite size: 3 authored + 2 designed for Feature Forge.
