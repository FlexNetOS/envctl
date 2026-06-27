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
