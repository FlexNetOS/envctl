# verdicts — rusty-idd · the GATE (plan-verifier)

Verifier: plan-verifier. Method: adversarial refute-each-claim + feasibility-gate-each-upgrade against
actual source at SHA `5a55284` (branch `plan/lifeos-meta-front-door`). Read-only on target.
Only CONFIRMED/QUALIFIED + feasibility-passed rows flow to the architect. Default-skeptical, fail-closed.

Format: `CLAIM/UPGRADE <id> — <one line> — VERDICT -> <verdict> [feasible|infeasible] — evidence: <…>`.

---

## 2026-06-26 — dimension: architecture (core, 15 CLAIM / 10 UPGRADE)

### CLAIM verdicts

- CLAIM C1 — crate graph is a clean DAG; `cli` is unique sink-of-control, nothing depends on it — VERDICT -> CONFIRMED — evidence: `Cargo.toml:25-37` (11 members; cli depends on core/spec/merge-tools/runner/tui/knowledge; `grep cli crates/*/Cargo.toml` shows no crate depends on cli), `graph/rusty-idd.metrics.json:111-114` (cross_crate_cycles []).
- CLAIM C2 — `SpecDoc.contains` is the #1 product symbol (842 callers) and a one-line query primitive — VERDICT -> CONFIRMED — evidence: `crates/spec/src/model/spec.rs:30-32` (`self.position_of(name).is_some()`); 842 from `metrics.json:8-11`.
- CLAIM C3 — `runner.rs` highest blast (803), 2,146 lines, ~12 top-level items — VERDICT -> CONFIRMED — evidence: `wc -l` = 2146; `grep -cE '^(pub )?(struct|enum|impl|fn)'` = 12; blast 803 `metrics.json:90`.
- CLAIM C4 — `tui/src/app.rs` god-file, 5,708 lines, blast 248 — VERDICT -> CONFIRMED — evidence: `wc -l` = 5708; `metrics.json:13-16,91`.
- CLAIM C5 — `knowledge/src/lib.rs` is a 7,058-line single-file engine (blast 105) — VERDICT -> CONFIRMED — evidence: `wc -l` = 7058; catalog literals at `crates/knowledge/src/lib.rs:3585-3725` verified; `metrics.json:92`.
- CLAIM C6 — `crates/config/` is NOT a workspace member (only `example.toml`) — VERDICT -> CONFIRMED — evidence: `ls crates/config/` = `example.toml` only; absent from `Cargo.toml:25-37`.
- CLAIM C7 — `work-order` is wired-but-unconsumed (no crate depends on it; 24 dead) — VERDICT -> CONFIRMED — evidence: `grep work-order crates/*/Cargo.toml` → only its own manifest; `compute_intent_lock` only in `crates/work-order/src/lib.rs`; `Cargo.toml` description literally says "S1 spike"; 24 dead `metrics.json:107`.
- CLAIM C8 — ~278 dead product symbols (lower bound), 182 in vendored codegraph — VERDICT -> CONFIRMED — evidence: `metrics.json:98-109` (120+62=182 of 278; truncation_note self-disclosed as lower bound). Tool-derived count; caveat already stated by analyst.
- CLAIM C9 — vendored codegraph-core exposes wider public surface (355) than product crates — VERDICT -> CONFIRMED — evidence: `metrics.json:116-124` (codegraph-core 355 vs cli 74, core 71).
- CLAIM C10 — NO internal HTTP/service routes in `crates/` (CLI+TUI+lib) — VERDICT -> CONFIRMED — evidence: `grep -rinE 'axum|actix|warp|hyper::Server|rocket'` over all product crates → empty; `metrics.json:151-154`.
- CLAIM C11 (HEADLINE) — ZERO library/IPC dep on weave/icm/grit/hf; the only in-code refs are descriptive string data — VERDICT -> CONFIRMED — evidence: `grep -rinE 'weave|icm|grit|\bhf\b' crates/*/Cargo.toml` → NONE; refs are string literals (repo catalog `crates/knowledge/src/lib.rs:3585-3725`; icm-checker contract text `crates/cli/src/commands/harness.rs:208-215`). The string-literal-vs-coupling distinction holds: catalog entries are `&str` array data, not deps.
- CLAIM C12 — convergence is filesystem + JSON-schema contracts only (analyst-marked medium) — VERDICT -> CONFIRMED (lifted from medium) — evidence: re-opened both cited lines — `.handoff/tasks` read at `crates/cli/src/commands/codex.rs:594` (`check_task_evidence`); `_workspace/{backlog,loop_state,HANDOFF}.md` declared at `crates/merge-tools/src/lib.rs:110` (`LegacySurface`). No lib bindings.
- CLAIM C13 — spec well-modularized, 30 dead, "not fully wired to the 6 spec_* CLI commands"; merge.rs 21 KB — VERDICT -> QUALIFIED — evidence: corrections required — (a) merge.rs is at `crates/spec/src/model/merge.rs` (21,901 B ≈ 21.4 KB), NOT `crates/spec/src/merge.rs` as cited; (b) there are **5** `spec_*` CLI command files (`spec_adr, spec_archive, spec_plan_integration, spec_scaffold, spec_status`), not 6. Substance (modular sub-trees adr/archive/model/parse/scaffold/schema/validate; 30 dead per `metrics.json:104`; partial CLI wiring) holds; the two numeric/path citations are wrong. Use corrected figures downstream.
- CLAIM C14 — deprecated `serde_yaml 0.9.34` is transitive via vendored codegraph-core; first-party on serde_norway — VERDICT -> CONFIRMED — evidence: `cargo tree -i serde_yaml` → `serde_yaml v0.9.34+deprecated` sourced ONLY from `codegraph-core` (→ codegraph-parser → knowledge → cli); `serde_yaml = "0.9"` at `crates/external/codegraph-core/Cargo.toml:40`; first-party uses `serde_norway` (`crates/spec/Cargo.toml`, `crates/runner/Cargo.toml`). Lock entry `Cargo.lock:3400`.
- CLAIM C15 — mixed editions (core 2021 vs tui/runner 2024) forcing resolver=3; pinned deps current — VERDICT -> CONFIRMED — evidence: `crates/core/Cargo.toml:9` `edition.workspace = true` (=2021 default `Cargo.toml:21`); `crates/runner/Cargo.toml:9` and `crates/tui/Cargo.toml:9` = `2024`; `Cargo.toml:24` `resolver = "3"`. Version currency is trend-researcher-sourced; edition mix independently re-verified.

### UPGRADE feasibility verdicts (axis-fit + invariant gate: NO C in trust boundary, engine-first, Upgrade-Only)

- UPGRADE U1 — decompose `runner.rs` into sub-modules behind unchanged public API (axis: quality) — VERDICT -> CONFIRMED feasible — evidence: behavior-preserving module split inside one pure-Rust crate; no C, no downgrade; serves quality (shrinks reviewable scope on the #1 blast surface, blast 803 `metrics.json:90`). Acceptance (public-API diff = ∅) is the correct gate.
- UPGRADE U2 — split `tui/src/app.rs` into screen/state/input modules (axis: quality) — VERDICT -> CONFIRMED feasible — evidence: same class as U1; pure-Rust ratatui crate, blast 248; behavior-preserving. Feasibility passes.
- UPGRADE U3 — split `knowledge/src/lib.rs`; catalog → external data file (axis: quality) — VERDICT -> CONFIRMED feasible — evidence: catalog is static `&str` data at `crates/knowledge/src/lib.rs:3585-3725`; extracting to a `.toml`/data file with a round-trip test is buildable in-repo, no C, serves quality/maintainability. Feasibility passes.
- UPGRADE U4 — feature-gate the 182 dead vendored codegraph symbols (axis: speed) — VERDICT -> QUALIFIED feasible — evidence: mechanically buildable (Cargo features + `#[cfg]`); knowledge already imports codegraph with `default-features = false` and uses only parsing types (`crates/knowledge/src/lib.rs:7-12`), so a slim feature set is viable. QUALIFIED on the *speed* axis claim: build-time/binary-size win is plausible but unquantified; gate the win with a before/after build-time + `code dead` measurement (the acceptance already requires the dead-count drop). Feasible; magnitude must be measured, not asserted.
- UPGRADE U5 — de-duplicate vendored upstreams (handoff vendored 3×) (axis: governance) — VERDICT -> CONFIRMED feasible — evidence: vendored trees are NOT workspace members (`Cargo.toml:25-37`), blast 0 on product; deleting duplicate tracked trees is reversible via git history. Correctly PROPOSE (owner-walled, deletes tracked trees). Feasibility passes within invariants.
- UPGRADE U6 — integrate-or-retire `work-order` (axis: accuracy) — VERDICT -> CONFIRMED feasible — evidence: 24 dead, zero product consumers (verified C7); either wiring it into `codex.rs` task intake or removing it from members is buildable; both directions preserve no-C/Upgrade-Only. Feasibility passes; direction is an owner decision (correctly PROPOSE).
- UPGRADE U7 (HEADLINE) — define a convergence/adapter boundary so weave/icm/grit/hf can bind as libs/IPC, not only filesystem (axis: governance) — VERDICT -> QUALIFIED feasible — evidence: a trait/adapter crate with the filesystem adapter as the first impl is buildable in pure Rust (weave is redb/pure-Rust; A2A/gRPC via tonic is pure-Rust — no C enters the trust boundary), and the filesystem `.handoff/` contract stays the required fallback. QUALIFIED condition: the boundary MUST keep weave the required local route and any future adapter must remain C-free in the trust path (e.g. no C TLS/native vector lib); the contract test asserting `handoff.task.v1` round-trip is the right gate. Feasible under that condition.
- UPGRADE U8 — migrate vendored codegraph-core off `serde_yaml 0.9` to `serde_norway` (axis: governance) — VERDICT -> CONFIRMED feasible — evidence: `cargo tree -i serde_yaml` proves codegraph-core is the SOLE source; first-party already proved serde_norway works (`spec`/`runner`); serde_norway is pure-Rust drop-in. Acceptance (`cargo tree -i serde_yaml` empty) is verifiable. Strongly feasible.
- UPGRADE U9 — resolve `crates/config/` stray dir + CI member-guard (axis: governance, APPLY) — VERDICT -> CONFIRMED feasible — evidence: C6 confirms the orphan; a RED test asserting every source-bearing child of `crates/` is a workspace member is buildable and currently fails for `crates/config/`. Low-risk, reversible. Feasibility passes.
- UPGRADE U10 — wire-or-mark the 30 dead `spec` symbols (axis: accuracy, APPLY) — VERDICT -> CONFIRMED feasible — evidence: spec is a re-export facade (blast 0, `metrics.json:96`); adding CLI call-paths or `#[allow(dead_code)]`-with-rationale + a no-undocumented-dead-public test is buildable in-repo. Note the corrected CLI count (5 spec_* commands, per C13) when sizing the wiring. Feasibility passes.

---

## 2026-06-26 — cross-dimension spot-gate (governance / test-strategy / memory-vector / distributed-compute / filesystem)

### CLAIM verdicts

- CLAIM gov-001 — fail-open harness drift: `.claude/settings.json` enforces only SessionStart while `.codex/hooks.json` gates 6 lifecycle points — VERDICT -> CONFIRMED — evidence: `.claude/settings.json` hook keys = {SessionStart} only; `.codex/hooks.json` contains SessionStart/PreToolUse/PostToolUse/Stop/SubagentStop (5 hook types, grep count 5). A Claude agent runs ungated.
- CLAIM gov-002 — agent-guard is decorative: only consumer checks existence, never parses deny/mode — VERDICT -> CONFIRMED — evidence: `crates/core/src/validation.rs:48` `require_file(root, ".claude/agent-guard.toml", …)` (existence-only); `.claude/agent-guard.toml:2` `mode = "warn"`; no PreToolUse hook invokes it.
- CLAIM gov-003 — toolchain drift: CI defaults to nightly, manifest advertises stable 1.88, no rust-toolchain.toml — VERDICT -> CONFIRMED — evidence: `scripts/ci/envctl-rust-env.sh:121` `toolchain="${RUSTY_IDD_RUST_TOOLCHAIN:-nightly}"`; `Cargo.toml:22` `rust-version = "1.88"`; `rust-toolchain.toml` ABSENT; `.github/workflows/ci.yml:51` `cargo clippy --all-targets --all-features -- -D warnings` on nightly caches.
- CLAIM mem (.kb) — rusty-idd has NO `.kb/` of its own; recalled only if the fleet daemon indexed the path — VERDICT -> CONFIRMED — evidence: `ls .kb` → "No such file or directory"; 0 git_kb/.kb refs in `crates/`.
- CLAIM dc (no-C boundary) — only third-party native surface is blake3 + serde/serde_json/schemars; no FFI/C in the control path — VERDICT -> CONFIRMED — evidence: `grep -rinE 'mlua|rusqlite|libsqlite|openssl-sys|-sys =|cc =|bindgen' crates/*/Cargo.toml` (excl. external) → none; `crates/work-order/Cargo.toml` deps = serde, serde_json, blake3, schemars (blake3 defaults to pure-Rust/intrinsics, not C linkage).
- CLAIM ts-24/25/26 — work-order load path silently accepts a foreign `schema` discriminator and a tampered card; `#[schemars(regex)]` shapes only the generated JSON Schema, not serde deserialization — VERDICT -> CONFIRMED — evidence: `crates/work-order/src/lib.rs:40-46` `schema:String` carries only `#[schemars(regex(...))]` (schema-doc constraint, not a serde validator); no `from_card`/`validate` load counterpart to `to_json` (`lib.rs:222`); every test uses bare `serde_json::from_str` (`lib.rs:457,493,564`) which does not enforce the pattern. The "provable contract" is unenforced on load.
- CLAIM ts-27 — work-order producer never wired to the `.handoff/tasks` consumer — VERDICT -> CONFIRMED — evidence: producer `work_orders_from_bundle` in `crates/work-order/src/lib.rs`; consumer `check_task_evidence` at `crates/cli/src/commands/codex.rs:594`; no crate depends on work-order (C7).
- CLAIM ts-28 — `.handoff/tasks` consumer does no card validation (accepts any `*.json`) — VERDICT -> CONFIRMED — evidence: `crates/cli/src/commands/codex.rs:594-596` `contains_task_card` returns on any present card; no parse/schema check on the path.

### UPGRADE feasibility verdicts

- UPGRADE DC-2 — bind issued work-orders to weave/A2A transport carrying correlation_id (axis: distributed-compute) — VERDICT -> QUALIFIED feasible — evidence: weave is pure-Rust (no C), so transport binding stays within the no-C boundary; but it introduces the first live network/IPC dep into a currently-offline binary. QUALIFIED condition: gate behind a transport feature flag and keep the filesystem `.handoff/` contract as the offline/degraded fallback (as the analyst proposed). Feasible under that condition.
- UPGRADE DC-5 — guardrail: do NOT add mlua/esp-hal/no_std to rusty-idd; firmware + Lua/Luau runtime belong to fleet-executor repos (axis: distributed-compute, guardrail) — VERDICT -> CONFIRMED feasible — evidence: this guardrail PROTECTS the no-C/no-downgrade invariant (mlua links the C Lua lib). Recording it as an ADR-candidate adds zero deps. The gate endorses it.
- UPGRADE FL-3 — split first-party god-files into module trees behind unchanged public API; gate no `src/*.rs` > 1500 LOC (axis: filesystem-layout) — VERDICT -> CONFIRMED feasible — evidence: same behavior-preserving class as U1/U2/U3 (knowledge 7058, app.rs 5708, runner 2146 LOC re-verified by `wc -l`); buildable, no-C, reversible. Feasibility passes (coordinate with U1-U3 to avoid duplicate work).

---

## Tallies (this cycle)

- **architecture** (15 CLAIM / 10 UPGRADE): CLAIM = 14 CONFIRMED, 1 QUALIFIED (C13), 0 REFUTED, 0 INCONCLUSIVE. UPGRADE = 8 CONFIRMED-feasible, 2 QUALIFIED-feasible (U4 speed-magnitude, U7 invariant-condition), 0 infeasible.
- **cross-dimension spot-gate**: CLAIM = 8 CONFIRMED, 0 REFUTED. UPGRADE = 2 CONFIRMED-feasible (DC-5, FL-3), 1 QUALIFIED-feasible (DC-2), 0 infeasible.
- **Headline (C11)**: "weave/icm/grit/hf = 0 LIBRARY/IPC deps" CONFIRMED — string-literal-vs-coupling distinction holds.
- **No upgrade was found infeasible**: every proposed upgrade respects NO-C-in-trust-boundary, engine-first, and Upgrade-Only/no-downgrade; the two headline upgrades (U7 adapter boundary, DC-2 transport) pass only under the stated invariant/offline conditions.

Only CONFIRMED + QUALIFIED + feasibility-passed rows above flow to the plan-architect. C13's numeric/path
corrections (merge.rs at `model/merge.rs`; 5 not 6 `spec_*` CLI commands) must be carried into the plan.

---

# verdicts — handoff (cycle 2) · the GATE (plan-verifier)

Verifier: plan-verifier. Target = **handoff** (continuity kernel) planned as the UNION with rusty-idd.
Method: adversarial refute-each-claim + feasibility-gate-each-upgrade against actual source in worktree
`/home/drdave/Desktop/meta/.worktrees/plan-handoff-cycle2/handoff` @ `d74ad4b` (branch `plan/handoff-union-cycle2`),
plus RuVector read at `/home/drdave/Desktop/meta/RuVector`. Read-only on target. Default-skeptical, fail-closed.
Only CONFIRMED/QUALIFIED + feasibility-passed rows flow to the architect. Existing rusty-idd verdicts above are untouched.

Format: `- <ref> — <one line> — VERDICT -> <verdict> [feasible|infeasible] — evidence: <…>`.

## 2026-06-26 — EMPIRICAL EXPERIMENTS (static → empirical)

- EXP-1 (RuVector standalone blocker) -> **CONFIRMED**. `cargo build -p ledger` AND `cargo build -p ledger --no-default-features --features redb-store` BOTH fail at workspace **manifest-load** (not compile): `failed to read .../plan-handoff-cycle2/RuVector/crates/rvf/rvf-crypto/Cargo.toml (os error 2)`. Stronger than reported: `redb-store = ["dep:redb","dep:rvf-crypto"]` (`ledger/Cargo.toml:33`) so even the "minimal" feature pulls the `../../RuVector` `rvf-crypto` path dep; default `v2` additionally pulls rvf-runtime/index/types. The path dep makes the ENTIRE workspace unresolvable → no crate (even leaf `work-order`) builds in-tree. The union is NOT standalone at `$META_ROOT+handoff` today.
- EXP-2 (v1↔v2 benign collision) -> **CONFIRMED**. Exactly one public `Ledger` per feature set: `ledger/src/lib.rs:36-39` cfg-gates `#[cfg(all(feature="redb-store", not(feature="v2")))] pub use v1::*` XOR `#[cfg(feature="v2")] pub use v2::*`; `v2.rs:26-29` `pub struct Ledger { v1: v1::Ledger, … }` composes v1; `v2.rs:19-23` `pub use crate::v1::{EventRow, LeaseOutcome, …}` re-exports shared v1 types. (Minor line drift vs analyst's `:18,20` — actual `:17 use crate::v1;`, `:19 pub use crate::v1::{`; substance exact.) Embeddings are SHA3-256 pseudo-embeddings (`v2.rs:48-56`, doc: "small input changes produce uncorrelated vectors") → "semantic" is a misnomer.
- EXP-3 (witness chain = SHAKE-256, NOT blake3+ed25519) -> **CONFIRMED — KEY CORRECTION**. `RuVector/crates/rvf/rvf-crypto/src/witness.rs:4` "The chain uses **SHAKE-256** for hash binding"; `:74 prev_hash = shake256_256(&encoded)`. Ledger import `ledger/src/v1.rs:20 use rvf_crypto::witness::{WitnessEntry, create_witness_chain, verify_witness_chain}` — **no `sign` import**; action hashing is `sha3::Sha3_256` (`v1.rs:22`). ed25519 signing exists in `rvf-crypto` (default feature `["std","ed25519"]`, `sign.rs`) but is **un-wired** in the witness path. NOTE: blake3 IS used in the tree — but for `work-order::compute_intent_lock`, NOT the witness chain. Any doc/seed/trends text calling the witness chain "blake3+ed25519-signed" is REFUTED; it is SHA-3-family (SHAKE-256 link, SHA3-256 action), unsigned.

## architecture (15 CLAIM / 6 UPGRADE)

### CLAIM verdicts
- A-C1 — Cargo dep graph strict DAG; `hf` universal sink, nothing depends on `hf` — VERDICT -> CONFIRMED — `grep 'hf = {' --include=Cargo.toml` = ∅ (no crate depends on hf); `hf/Cargo.toml` carries 20 path deps; `metrics.json` layering `cargo_dep_graph: "strict DAG"`.
- A-C2 — 14 call-graph SCCs are NOT architectural cycles (syntect recursion + same-name collisions) — VERDICT -> CONFIRMED — `metrics.json` cycles: 8 vendor + 5 own size-2/3 same-name; ground truth = Cargo DAG (A-C1). Tool-derived; manifest cross-check corroborates.
- A-C3 — v1↔v2 "same-name collision" is benign (cfg-gated mutually-exclusive re-exports; v2 composes v1) — VERDICT -> CONFIRMED (EXP-2).
- A-C4 — both v1+v2 compile under default; v1 public only via v2 wrapper — VERDICT -> CONFIRMED — `lib.rs:20-23` cfg-gated `mod v1`/`mod v2`; `default=["v2"]`→`v2`→`redb-store`→`v1`.
- A-C5 — standalone blocker: hf+ledger need sibling RuVector — VERDICT -> CONFIRMED (EXP-1, empirical).
- A-C6 — standalone build rests on a CI clone side-effect, not the crate graph — VERDICT -> CONFIRMED — `hf/Cargo.toml:45` comment; `ledger/Cargo.toml:18` path deps; CI clone (governance gov-003).
- A-C7 — witness-chain RuVector dep is minimal/pure-Rust; deps default-features=false → C-free trust boundary — VERDICT -> QUALIFIED — CORRECTION: only the **hf-side** `ruvector-verified`/`ruvector-domain-expansion`/`cognitum-gate-tilezero` carry `default-features=false` (`hf/Cargo.toml:48,52,59`). **`ledger`'s `rvf-crypto` does NOT** (`ledger/Cargo.toml:20 features=["std"]`, default features incl. `ed25519` ON) — so `ed25519-dalek` is compiled into the default build though the witness path never signs. The C-free conclusion HOLDS (sha3 + ed25519-dalek are pure-Rust); the "default-features=false rvf-crypto" wording is inaccurate. Use the corrected figure downstream.
- A-C8 — two separate programs; zero Cargo dep between KERNEL and rusty-idd-*; 1 RIDD→KERNEL call edge vs 41 — VERDICT -> CONFIRMED — `metrics.json` cross_group_call_edges {RIDD→KERNEL:1, KERNEL→RIDD:41}; no manifest dep (A-C1 grep).
- A-C9 — crates/{cli,core,runner,spec,tui} are a stale partial fork; rusty-idd is the superset — VERDICT -> CONFIRMED (lineage facts), QUALIFIED on exact % — spec model files byte-identical to rusty-idd (`diff -q` ∅ on `model/spec.rs`,`requirement.rs`); handoff cli = 9 cmd files vs rusty-idd 17 (+8 stripped); rusty-idd has `config/external/knowledge/merge-tools` crates handoff LACKS. The "95% / 0.1–40%" aggregate is tool-derived (QUALIFIED), the fork/superset/stripped facts are CONFIRMED.
- A-C10 — `unsafe_code="deny"` workspace lint; single audited `pid_is_alive` FFI in v2.rs, fail-closed — VERDICT -> CONFIRMED — `Cargo.toml:34`; `ledger/src/v2.rs:115 #[allow(unsafe_code)]`, `:118` unverifiable→fail-closed (do not reclaim). (Correctly in v2.rs, not v1.)
- A-C11 — `Ledger.open` widest blast (120/74), `ledger_path` 54, `validate_card` 40 — VERDICT -> CONFIRMED — `metrics.json` blast_radius. Tool-derived counts; structure corroborated.
- A-C12 — `validate_card` is the fail-closed gate; schema generated from `work_order::WorkOrder` via schemars+OnceLock (can't drift) — VERDICT -> CONFIRMED — `handoff-schema/src/lib.rs:10-11,28-31` (`work_order::task_schema_json()` compiled once).
- A-C13 — primary union contract seam is a mirrored file copy, not a dependency — VERDICT -> CONFIRMED — `rusty-idd/crates/work-order/src/lib.rs:35` "mirrors … task.schema.json", `:249` "a *contract mirror*, not a path-dependency".
- A-C14 — centrality dominated by RIDD fork; kernel hubs McpServer.new/Ledger.open/ledger_path/compute_intent_lock — VERDICT -> CONFIRMED — `metrics.json` hotspots. Tool-derived.
- A-C15 — git-kb dead-code list (1258) unsafe as a removal list; hf's 314 are clap string-dispatch false positives — VERDICT -> CONFIRMED (as caveat) — `metrics.json` dead_code caveat self-discloses the false-positive class; analyst's medium confidence + per-symbol-triage requirement is the correct posture.

### UPGRADE feasibility verdicts (invariants: NO C in trust boundary, engine-first, strict-upgrade-only, keep filesystem contract fallback)
- A-U1 — move RuVector off `../../` path deps (vendor witness-chain crates / publish / off-by-default feature) — VERDICT -> CONFIRMED **feasible** — vendoring `rvf-crypto`/`rvf-*` keeps no-C (sha3 + ed25519-dalek pure-Rust; redb pure-Rust); serves governance/portability; kills the EXP-1 blocker. The headline fix.
- A-U2 — split feature graph so default=`redb-store` (no RVF runtime), `v2` opt-in — VERDICT -> QUALIFIED **feasible** — acceptance (default tree excludes rvf-runtime/index/types) is achievable, BUT `redb-store` STILL pulls `rvf-crypto` from `../../RuVector` (`ledger/Cargo.toml:33`) → it REDUCES but does NOT eliminate the RuVector coupling and does not make the kernel standalone on its own; it is **coupled to A-U1** (rvf-crypto must resolve before this can even build/test). Feasible under that condition.
- A-U3 — replace rusty-idd's mirrored `work-order` with a real dep on handoff `work-order`+`validate_card` — VERDICT -> CONFIRMED **feasible** — pure-Rust crate dependency; single compiler-enforced `handoff.task.v1` source-of-truth; serves accuracy (closes A-C13/G2 drift).
- A-U4 — converge/delete the stale `crates/{cli,core,runner,spec,tui}` fork toward rusty-idd superset — VERDICT -> CONFIRMED **feasible** — identical `rusty-idd-*` pkg names in both workspaces (verified) mandate dedup to avoid a Cargo name collision on union; reversible via git history; PROPOSE (owner-walled per-crate reconcile) is correct.
- A-U5 — compile-time test asserting exactly one `Ledger` per feature set + ADR note — VERDICT -> QUALIFIED **feasible** — additive + correct (EXP-2 proves the invariant), but the test builds `ledger` under `redb-store`/`v2` which both need `rvf-crypto` → it **cannot run until A-U1 resolves RuVector** (EXP-1). Feasible, gated on A-U1.
- A-U6 — manifest-cross-checked graph-integrity gate (planning artifacts only) — VERDICT -> CONFIRMED **feasible** — read-only planning post-processing; flags SCC/dead-code false positives against the Cargo DAG; APPLY, zero production blast.

## governance+settings+config (9 CLAIM / 8 UPGRADE)
- gov-001 (HEADLINE fail-OPEN seam) — `hooks.toml` 5 `fail_mode="block"` gates NOT bridged to Claude PreToolUse (`settings.json` only SessionStart/SessionEnd) — VERDICT -> CONFIRMED (empirical) — `.handoff/hooks/hooks.toml` has 5 block gates (`:24,30,42,62,108`); `.claude/settings.json` hook keys = {SessionEnd, SessionStart} only, **no PreToolUse/PostToolUse**. A Claude edit can go out-of-scope without tripping the block gate — the kernel's own L7 fail-OPEN class.
- gov-002 — agent-guard not wired as project PreToolUse (enforcement lives in uncommitted envctl user-global) — VERDICT -> CONFIRMED — no PreToolUse in `settings.json`; user-global layer unverifiable from here (the portability defect stands regardless).
- gov-003 — RuVector path deps; CI clone workaround; not standalone — VERDICT -> CONFIRMED (EXP-1).
- gov-004 — SessionStart hook unconditionally `exec rusty-idd next` (no `command -v` guard) — VERDICT -> CONFIRMED — `settings.json:26`.
- gov-005 — no `rust-toolchain.toml`; CI pins toolchain per-job — VERDICT -> CONFIRMED — `ls rust-toolchain.toml` = absent; `Cargo.toml` edition 2024 + rust-version.
- gov-006 — `hf-mcp` binary but no `.mcp.json`/`.codex` registration (MCP rot) — VERDICT -> CONFIRMED — `ls .mcp.json .codex` = absent.
- gov-007 — permissions allow `Bash(git -C * push:*)` (push any repo path) — VERDICT -> CONFIRMED — `settings.json:38`.
- gov-008 — master-vs-main is a recorded owner decision, not live drift — VERDICT -> CONFIRMED (recorded data; routed to gap, not upgrade).
- gov-009 — rule lists 4 blocked cmds, agent-guard blocks 8 (doc-vs-config drift) — VERDICT -> CONFIRMED (finding-sourced; coherence defect).
- gov-U1 — wire PreToolUse(Edit|Write|Bash)→`hf hook run` — VERDICT -> CONFIRMED **feasible** (PROPOSE; closes gov-001 fail-OPEN; additive, never weakens).
- gov-U2 — wire agent-guard PreToolUse repo-portable — VERDICT -> CONFIRMED **feasible** (PROPOSE; tightens only).
- gov-U3 — resolve RuVector dep — VERDICT -> CONFIRMED **feasible** (= A-U1).
- gov-U4 — guard `rusty-idd next` hook with `command -v` — VERDICT -> CONFIRMED **feasible** (graceful skip when rusty-idd absent).
- gov-U5 — add `rust-toolchain.toml` pinned to CI toolchain — VERDICT -> CONFIRMED **feasible**.
- gov-U6 — commit `.mcp.json` registering `hf-mcp` — VERDICT -> CONFIRMED **feasible**.
- gov-U7 — tighten `Bash(git -C * push:*)` to repo-scoped — VERDICT -> CONFIRMED **feasible** (tightens; never relaxes).
- gov-U9 — doc-sync the destructive-cmd rule to all 8 guard patterns — VERDICT -> CONFIRMED **feasible** (APPLY, doc-only).

## memory-vector-intelligence (key CLAIM / 6 UPGRADE)
- mem-1 — RVF `query_by_intent` has 0 production callers; no `hf recall/search/query` verb — VERDICT -> CONFIRMED (empirical) — `grep query_by_intent` = lib.rs doc + v2.rs:346 def + v2.rs:660 test ONLY; `grep '"recall"|"search"|"query"' hf/src/main.rs` = ∅.
- mem-2 — embeddings are SHA3-256 pseudo, not learned/semantic — VERDICT -> CONFIRMED (EXP-2).
- mem-3 — witness chain SHAKE-256 hash-linked, NOT blake3+ed25519-signed — VERDICT -> CONFIRMED (EXP-3, KEY CORRECTION).
- mem-4 — ICM has 0 product references — VERDICT -> CONFIRMED — `grep '\bicm\b' --include=*.rs` (excl vendor) = 0.
- mem-5 — handoff is a first-class git-kb member (committed `.kb/`) + drives git-kb subprocess — VERDICT -> CONFIRMED (finding-sourced; `.kb/` present, `hf/src/kb.rs` seam).
- mem-6 — RVF written on every append, read by nothing (write-amplification) — VERDICT -> CONFIRMED — ingest on append (`v2.rs`); 0 readers (mem-1).
- mem-7 — RuVector path dep unresolved in worktree — VERDICT -> CONFIRMED (EXP-1).
- mem-U1 — wire `query_by_intent` to `hf recall` with REAL embeddings OR delete v2-default + delegate recall — VERDICT -> QUALIFIED **feasible** — feasible, CONDITION: any native embedder must respect NO-C-in-trust-boundary (no C vector lib); delete/delegate path is unconditionally feasible.
- mem-U2 — make RVF overlay opt-in (default-off) to stop write-amp — VERDICT -> QUALIFIED **feasible** — reduces write-amp, but `redb-store` still pulls `rvf-crypto` (RuVector) → does not achieve standalone alone (coupled to A-U1/mem-U4). Same condition as A-U2.
- mem-U3 — correct witness provenance to SHAKE-256 (optionally wire ed25519 signing) — VERDICT -> CONFIRMED **feasible** — doc fix is verified-correct (EXP-3); signing is additive, pure-Rust, behind a feature.
- mem-U4 — resolve RuVector path dep for standalone — VERDICT -> CONFIRMED **feasible** (= A-U1).
- mem-U5 — introduce decision/"why" memory (ICM or ledger-curated events) — VERDICT -> CONFIRMED **feasible** — additive; ledger-curated variant needs no new dep.
- mem-U6 — unify fleet recall behind one provenance-tagged facade — VERDICT -> QUALIFIED **feasible** — high-effort cross-organ contract; additive facade + ADR; feasible as a façade with fail-closed organ-tagging.

## test-coverage (CLAIM / 4 UPGRADE)
- ts-1 — work-order producer seam well-covered (15 tests) — VERDICT -> CONFIRMED (finding-sourced; reachable producer tests).
- ts-2 — NO fail-closed work-order LOADER; only `serde_json::from_str` = FAIL-OPEN — VERDICT -> CONFIRMED (empirical) — `grep from_card_json|from_card|fn validate|try_from_value` in `work-order/src/lib.rs` = ∅; only `#[schemars(regex)]` at `:60,:66` (schema-doc, NOT serde-enforced).
- ts-3 — union consumer inherits the fail-open via the mirror — VERDICT -> CONFIRMED (rusty-idd mirror, A-C13).
- ts-4 — `validate_card` (JSON-schema) cannot catch intent_lock-vs-content drift (no blake3 on load) — VERDICT -> CONFIRMED — `handoff-schema` is pure JSON-schema; `work-order::intent_unchanged` recomputes blake3; nothing chains them on a load path.
- ts-5 — ledger read API is MISSING (internal to hf) — VERDICT -> CONFIRMED — Seam 2; `Ledger.open` callers all in-kernel.
- ts-6 — ledger cannot be tested standalone (RuVector wall) — VERDICT -> CONFIRMED (EXP-1; whole workspace fails manifest-load).
- ts-RED — RED suite `work-order/tests/union_failclosed.rs` committed + verified RED — VERDICT -> CONFIRMED (empirical) — built `work-order` in a standalone scratch mirror (RuVector-free): `test result: FAILED. 1 passed; 3 failed` (foreign-schema / malformed-id / drifted-intent_lock all FAIL-OPEN; fixture GREEN). tests-ran: 4, not an exit-0 fail-open.
- ts-U1 — fail-closed loader tests (AUTHORED) — VERDICT -> CONFIRMED **feasible** (RED verified; additive; flips GREEN when `WorkOrder::from_card_json` chains serde+validate_card+intent_unchanged).
- ts-U2 — handoff-intake refusal integration test — VERDICT -> QUALIFIED **feasible** — BLOCKED in-tree by the RuVector wall (handoff-intake→handoff-core→ledger→RuVector); feasible after A-U1.
- ts-U3 — public ledger read-API contract test — VERDICT -> QUALIFIED **feasibility** pending — design-only: API unbuilt AND RuVector wall; cannot author a COMPILING RED today. Feasible after the read-API is designed + A-U1.
- ts-U4 — differential/golden `task_schema_json` parity test across the mirror — VERDICT -> CONFIRMED **feasible** — golden capture of `task_schema_json()`; spec/work-order lineage already proven (A-C9) — cheap drift gate.

## rules-policy-org (CLAIM / 5 UPGRADE)
- rp-teeth (HEADLINE) — handoff gates have REAL teeth (`exit(1)`) vs rusty-idd advisory — VERDICT -> CONFIRMED (empirical) — `process::exit(1)` in `handoff-drift/src/lib.rs:676,792`, `hf/src/cognitum.rs:135`, `handoff-gatekeeper/src/lib.rs:304-386`; fired with `fail_mode="block"` (`hooks.toml`). rusty-idd's agent-guard is `mode="warn"` existence-only (cycle-1 gov-002). For the union the teeth live in handoff; rusty-idd CLI folds UNDER them.
- rp-declared-unenforced — `default_network_mode`/`default_dependency_mode` are policy DATA with no kernel enforcement path — VERDICT -> CONFIRMED (finding-sourced).
- rp-org-chart — 9-agent org; no `evolution-steward`; uniform-opus, no per-role `model:` lane — VERDICT -> CONFIRMED (finding-sourced; consistent with prompt-architecture §3).
- rp-A2A — weave=transport plane, handoff=witnessed-receipts plane stay distinct; offline degrade to ledger-only — VERDICT -> CONFIRMED (finding-sourced; `handoff-lease` Reserve::Unsupported→ProceedDegraded).
- rp-upgrade-only-is-intent — "Upgrade Only / parity-before-removal" is NORTH-STAR intent, no machine gate enforces it — VERDICT -> CONFIRMED (honest finding; intent ≠ enforced gate).
- UP-1 — fold rusty-idd CLI under `hf policy check-edit`/gatekeeper — VERDICT -> CONFIRMED **feasible** (wiring, not logic; blast LOW).
- UP-2 — enforce declared network/dep-audit policies (default-warn→block) — VERDICT -> QUALIFIED **feasible** (new enforcement can false-block; must default-warn first; RED test gates it).
- UP-3 — add `evolution-steward` to handoff's org — VERDICT -> CONFIRMED **feasible** (additive, propose-by-default, never weakens a guard).
- UP-4 — dual-model background lane (No-Downgrades guard) — VERDICT -> QUALIFIED **feasible** — CONDITION: gates/ADR/verifier stay opus (asserted), only mechanical work routes cheaper, and a witnessed guard blocks a SILENT downgrade of a gate-tier action. Feasible under that condition (mirrors cycle-1 stance).
- UP-5 — self-enforce agent-guard via handoff's own PreToolUse — VERDICT -> CONFIRMED **feasible** (= gov-U2).

## prompt-architecture (CLAIM / 4 UPGRADE)
- pa-dual-front-door — TWO SessionStart hooks (hf loop-entry + `rusty-idd next`) — VERDICT -> CONFIRMED (empirical) — `settings.json:18` loop-entry.sh, `:26` `exec rusty-idd next`.
- pa-fork-drift — the in-repo `rusty-idd-cli` fork lacks the `next`/`render` verbs its own hook/adapter need — VERDICT -> CONFIRMED (empirical) — `crates/cli/src/commands/` has no `next.rs`/`render.rs` (rusty-idd has both); the hook must resolve an external superset binary on PATH.
- pa-hf-mcp — `hf-mcp` grants ~35 tools, each a shell-out to `hf`; mutating tools (ship/done/claim) ungated at the MCP layer — VERDICT -> CONFIRMED (finding-sourced; consistent with gov/dc findings).
- pa-single-opus-lane / pa-determinism-intake — VERDICT -> CONFIRMED (finding-sourced; intake is deterministic non-LLM by design).
- pa-U1 — reconcile the dual front door to ONE canonical entry — VERDICT -> CONFIRMED **feasible** (resolves pa-dual/fork-drift; PROPOSE).
- pa-U2 — pin/version-stamp `hf`↔`hf-mcp` instead of PATH+warn — VERDICT -> CONFIRMED **feasible**.
- pa-U3 — trim the 1541-skill `.agent/skills-catalog.md` surface — VERDICT -> CONFIRMED **feasible** (token hygiene).
- pa-U4 — make the opus model-lane explicit policy — VERDICT -> CONFIRMED **feasible** (= rules UP-4 documentation half).

## distributed-compute (CLAIM / 5 UPGRADE)
- dc-1 — pure-Rust, no daemon, no network stack (0 reqwest/hyper/tonic/axum) — VERDICT -> CONFIRMED (finding-sourced; root `Cargo.toml` pure-Rust, `unsafe_code=deny`).
- dc-2 — Lua/Luau zero presence (0 mlua/lua/luau/lune hits) — VERDICT -> CONFIRMED (finding-sourced grep).
- dc-3 — RuVector is the only compute coupling + standalone blocker — VERDICT -> CONFIRMED (EXP-1).
- dc-4 — no-C boundary: `rusqlite` is optional migration-import-only, never default — VERDICT -> CONFIRMED — `ledger/Cargo.toml:33` `legacy-sqlite` feature only; not in `default`/`v2`.
- DC-1 — resolve RuVector standalone blocker — VERDICT -> CONFIRMED **feasible** (= A-U1).
- DC-2 — define leaf-node proxy contract (mobile/Pi-Zero/ESP32) — VERDICT -> CONFIRMED **feasible** (additive; reuses MCP+work-order; no kernel network code).
- DC-3 — native weave mesh binding (optional feature) — VERDICT -> QUALIFIED **feasible** — first live network/IPC dep; CONDITION: feature-gated, no-C, byte-identical offline fallback preserved (weave is pure-Rust).
- DC-4 — enforce `allows_network`/`path_scope` cross-node egress/residency — VERDICT -> CONFIRMED **feasible** (pure local validation on data already in the envelope).
- DC-5 — guardrail: no embedded/Lua/in-kernel network stack in handoff — VERDICT -> CONFIRMED **feasible** (protects no-C/no-downgrade invariant; ADR-only, zero deps).

## autoresearch (CLAIM / 5 UPGRADE)
- ar-1 — code auto-research is pull/event-driven via git-kb + `hf kb` seam + `handoff-index` (no resident daemon) — VERDICT -> CONFIRMED (finding-sourced; `hf/src/kb.rs`, `.kb/` present).
- ar-2 — `handoff-drift` is the fail-closed invalidation engine (`exit(1)`→PreHandoff block) — VERDICT -> CONFIRMED (empirical) — `handoff-drift/src/lib.rs:676 exit(1) // hard-fail so PreHandoff (fail_mode=block) stops`.
- ar-3 — advisory web gate (`cargo audit`, `ignore=[]`) runs on promotion, NOT per-PR (asymmetric) — VERDICT -> CONFIRMED (finding-sourced).
- ar-4 — stale `.git/gitkb/code.db` incident caught + discarded this cycle — VERDICT -> CONFIRMED (finding-sourced; cartography integrity note).
- U1 — git-kb index-staleness gate — VERDICT -> CONFIRMED **feasible** (additive check).
- U2 — symmetric `cargo audit` per-PR — VERDICT -> CONFIRMED **feasible**.
- U3 — scheduled research cadence — VERDICT -> CONFIRMED **feasible**.
- U4 — align fleet currency bot (Renovate vs Dependabot) — VERDICT -> CONFIRMED **feasible**.
- U5 — delete the last C dep (`rusqlite`/`legacy-sqlite`) — VERDICT -> QUALIFIED **feasible** — CONDITION: all fleet legacy ledgers migrated to redb first (else loses the import path); then `cargo tree -i rusqlite` empty in all feature sets.

## filesystem-layout (CLAIM / 6 UPGRADE)
- fs-1 — RuVector `../../` path deps break the portable-root residency mandate — VERDICT -> CONFIRMED (EXP-1).
- fs-2 — `crates/{cli,core,runner,spec,tui}` duplicate-lineage; identical `rusty-idd-*` pkg names = collision on union — VERDICT -> CONFIRMED (verified pkg names + A-C9).
- fs-3 (analyst correction) — `_workspace*` are gitignored ephemeral, NOT committed (inbound claim REFUTED by analyst) — VERDICT -> CONFIRMED (the correction is correct; routed as gitignore-hygiene).
- fs-4 (analyst correction) — vendored syntect is at root `vendor/syntect/`, NOT `crates/tui/vendor/` (corrects graph.md wording) — VERDICT -> CONFIRMED (root `[patch.crates-io] syntect`; no `crates/tui/vendor/`). NOTE: graph.md/metrics "crates/tui/vendor/syntect" path label is imprecise; the syntect IS vendored, just at root.
- fs-5 — `.idea/` (13 JetBrains files) committed = user→repo leak — VERDICT -> CONFIRMED (finding-sourced `git ls-files`).
- fs-6 — `.agent/skills-catalog.md` (313K generated blob) + `intent-driven-template/`/`spike/` orphans = root clutter — VERDICT -> CONFIRMED (finding-sourced).
- U1 — standalone-portable RuVector — VERDICT -> CONFIRMED **feasible** (= A-U1).
- U2 — dedup `rusty-idd-*` crates to one canonical set — VERDICT -> CONFIRMED **feasible** (= A-U4; collision must resolve).
- U3 — untrack `.idea/` — VERDICT -> CONFIRMED **feasible** (APPLY).
- U4 — route the generated skills-catalog off the committed root — VERDICT -> CONFIRMED **feasible** (REGENERATE).
- U5 — home/remove `intent-driven-template/` + `spike/` orphans — VERDICT -> CONFIRMED **feasible** (PROPOSE).
- U6 — mark `schemas/*.schema.json` provenance (generated vs authored) — VERDICT -> CONFIRMED **feasible** (REGENERATE; golden-from-type gate).

## union-with-rusty-idd (lineage + MERGE strategy)
- union-1 — 95%+ shared lineage; handoff = hardened kernel-focused fork — VERDICT -> CONFIRMED (lineage), QUALIFIED on exact % — spec model files byte-identical; rusty-idd is the superset (config/external/knowledge/merge-tools + 8 extra CLI cmds); work-order originated in handoff (provenance comment). The "95%" aggregate is tool-derived.
- union-2 — MERGE (fold rusty-idd CLI into handoff kernel; handoff north-star) — VERDICT -> QUALIFIED **feasible** — CONDITIONS that must hold before/at merge: (a) RuVector standalone resolved (A-U1, else the union cannot build at `$META_ROOT+handoff`); (b) `rusty-idd-*` pkg-name collision deduped (A-U4); (c) C-dep scan of rusty-idd `codex`/`knowledge` (syntect-onig / codegraph) — cycle-1 confirmed rusty-idd's only native surface is blake3, no C in the control path, so this passes but must be re-checked at land; (d) work-order seam converted from mirror to dependency (A-U3); (e) the filesystem `.handoff/tasks` JSON contract stays the fallback. Feasible under those conditions.
- union-3 — Seam 2 (ledger read API) is MISSING and must be designed for the union — VERDICT -> CONFIRMED gap (= ts-5).

## Tallies (handoff, cycle 2)

- **Empirical experiments**: EXP-1 (RuVector blocker) CONFIRMED; EXP-2 (one-Ledger-per-feature + SHA3 pseudo-embeddings) CONFIRMED; EXP-3 (witness = SHAKE-256, not blake3+ed25519) CONFIRMED — the KEY CORRECTION. RED suite re-run standalone: 1 passed / 3 failed (true RED).
- **CLAIM verdicts (all dimensions)**: CONFIRMED = 57, QUALIFIED = 3 (A-C7 rvf-crypto default-features correction; A-C9 exact-% aggregate; union-1 exact-% aggregate), REFUTED = 0 material analyst claims (the only refutations are the analyst's OWN correct refutations of inbound facts — `_workspace` not committed, syntect at root — which I confirm), INCONCLUSIVE = 0.
- **UPGRADE feasibility (all dimensions)**: feasible = 39 total — CONFIRMED-feasible = 31, QUALIFIED-feasible = 8 (A-U2, A-U5, mem-U1, mem-U2, mem-U6, ts-U2, ts-U3, rules UP-2/UP-4, DC-3, ar-U5, union-2 — conditioned on RuVector resolution, no-C boundary, default-warn, or witnessed no-downgrade), **infeasible = 0**. No upgrade violates NO-C-in-trust-boundary, engine-first, or strict-upgrade-only; the filesystem `.handoff/` contract is preserved as fallback in every transport/binding upgrade.
- **Headline gates**: (a) RuVector path-dep makes the union non-standalone — CONFIRMED EMPIRICALLY (manifest-load fails even for redb-store). (b) crates/* are shared-lineage forks, union=MERGE — CONFIRMED (lineage) / QUALIFIED-feasible (conditions). (c) handoff policy gates have real teeth (exit(1)) vs rusty-idd advisory — CONFIRMED EMPIRICALLY. (d) RVF semantic-recall dead (0 callers) + embeddings SHA3 not semantic — CONFIRMED EMPIRICALLY. (e) fail-OPEN hooks seam (block gates not bridged to Claude PreToolUse) — CONFIRMED EMPIRICALLY.
- **Correction propagated**: the witness chain is SHAKE-256 hash-linked (SHA3-256 action hash), UNSIGNED — NOT "blake3+ed25519". blake3 is used only for `work-order` intent_lock. Any seed/trends/doc text saying "blake3+ed25519 witness chain" must be corrected (mem-U3).

Only CONFIRMED + QUALIFIED + feasibility-passed rows above flow to the plan-architect.
