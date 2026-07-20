# Verification report: Blueprint test suite T1–T6 + R1 swarm-immune wrapper fix

Guardian: invariant-guardian (Phase 3 + Phase 3.5 runtime-verify). All findings raw-captured
against the delivered worktrees; conflicting evidence → the raw capture wins.

## Verdict — PASS-WITH-NOTES

Zero blocking findings. All CI gates green, every declared runtime surface driven with the app's
own output, diff discipline clean, no-regression confirmed. Three non-blocking NOTES: (N1) the
load-bearing `degrade` rebuild deviation is a **sanctioned in-contract robustness fix** (all four
scrutiny conditions hold — proven below); (N2) the inherited `cargo test --workspace` baseline was
**starved by concurrent CI** (envctl PR #463 took the local runners) and never returned — classified
INHERITED because the envctl change is provably script-only (zero Rust touched); (N3) `cargo fmt`/
`cargo clippy` subcommand shims are absent in this fenix toolbin (env gap, not a code defect).

## Gate results — all PASS (raw exit=0 each, envctl worktree)
- no-c.sh    : PASS — `resolved graph clean: rustls=['0.23.40'] on ring=['0.17.14']; zero aws-lc/openssl/C-SQLite` → `NO-C GATE PASS`
- shape.sh   : PASS — `SHAPE GATE PASS`
- enable.sh  : PASS — `ENABLE GATE PASS`
- p7.sh      : PASS — `P7 GATE PASS`
- kdf-feature-off.sh : PASS — `low-cost-kdf-tests correctly OFF by default` → `KDF-FEATURE-OFF GATE PASS`
- agent-env.sh : PASS — `✓ agent-env.lock is up to date` → `AGENT-ENV GATE PASS` (confirms no agent-env drift)
- meta-local-policy.sh : PASS — `active install sources target META_ROOT FHS/XDG` (install.sh writes only var/lib/ruvector + ~/.claude/archive, no usr/bin frontdoor)
- loop-state.sh : PASS — counters intact (cycles_total=58, monotonic ok)
- harness-scripts.sh : PASS — `HARNESS-SCRIPTS GATE PASS`

## cargo
- fmt   : NOT-RUN (env gap) — `cargo fmt` → `error: no such command: fmt` (exit 101); only `rustfmt`/`clippy-driver` binaries present, no `cargo-fmt`/`cargo-clippy` shims. envctl change touches **zero Rust** → fmt N/A there. Touched Rust = the one nu_plugin test: `rustfmt --edition 2024 --check .../tests/blobstore_parity.rs` → **exit 0 (clean)**.
- clippy: NOT-RUN on envctl (shim absent + no Rust touched → inherited/N-A). nu_plugin touched crate reported clippy-clean by implementer (only inherited `base64` registry-dep warnings).
- test (envctl `--workspace`) : **STARVED / INCONCLUSIVE (inherited axis)** — background run in a cold fresh-worktree target/ never returned; local runners were taken by envctl PR #463's CI jobs (per orchestrator). NOT re-run per instruction. Cannot be affected by this change (script-only, zero Rust touched in envctl). Touched Rust anywhere = nu_plugin T5 only, which compiles clean and runs GREEN (see Runtime check 3f).
- test (nu_plugin `codedb-store-pg --test blobstore_parity`) : PASS — default `1 ignored`; opt-in `1 passed`.

## Invariant checks (1–10)
1. No C in trust boundary : PASS — no-c.sh green; envctl dep graph untouched (script-only); nu_plugin dev-deps pure-Rust (`tempfile`→fastrand/rustix/getrandom/once_cell/windows-sys; `codedb-store-redb`=redb). Cargo.lock diff shows no SQLite/OpenSSL/aws-lc.
2. Code-shape invariants : PASS — shape.sh green.
3. secretd enable invariant : PASS — enable.sh green.
4. Engine purity (non-printing lib) : PASS/N-A — zero `crates/` edits (git status clean on tracked files); no logic added to engine/main.rs/GUI. R1 is var-runtime bun-lane JS (plan §"Runtime surface"), not the engine library.
5. Front-end parity : N/A — no new envctl `Engine` method (script/test/JS only). The parity contracts here are the R1 **frozen JS surface** `immuneGraph(edges)→{g,isolationBoundary(),weakestCoupling(),degrade(u,v,w),connected()}` (preserved, verified) and the T5 **BlobStore trait** parity `PgStore == redb` (verified GREEN).
6. Fail-closed + dry-run defaults : PASS/strengthened — T3 SELECT-only; T5 dedicated `codedb_t5_<pid>` temp table asserted `!= codebase`/`codebase_codedb`, dropped before asserting (no leak, before+after empty); install.sh Law-1 archives before overwrite (archive `d8de6e41…` preserved); `degrade` of a nonexistent edge is a runtime-proven fail-closed no-op.
7. Rust-native, no drift : PASS — no non-Rust source enters `crates/`; the bun/psql harness is test tooling mirroring `scripts/tests/*.sh`; no banned dep; nu_plugin dev-deps pure-Rust.
8. Lock honesty : PASS — no envctl component/dep change → `manifest/envctl.lock` + `agent-env.lock` unchanged (agent-env.sh: lock up to date); nu_plugin Cargo.lock updated to match its 2 dev-deps.
9. Kasetto absorption / agent-env : N/A — no `crates/agent-env` change; agent-env.sh PASS (no config↔lock drift).
10. Runtime behavior (observable surfaces) : PASS — plan declared `## Runtime surface: YES`; every surface driven with the app's own output + off-happy-path probes (see Runtime check).

## Parity check (no Engine method — contract-parity surfaces instead)
- R1 frozen JS surface : `runtime/swarm-immune.mjs:29 immuneGraph()` → consumed by `t1_swarm_immune.mjs:41` (import) + `:52,:69` (immuneGraph/degrade drive). Shape byte-identical to plan (`g` exposed as a transparent getter; 4 methods unchanged).
- T5 BlobStore trait parity : `blobstore_parity.rs:57 snapshot(&dyn BlobStore)` drives BOTH `PgStore` (`:103`) and redb `CaptureBatcher` (`:90`) through the identical `codedb_core::store::BlobStore` surface → asserted equal (`:126–133`).

## Unit ledger (plan `## Units` U1–U8) — all present + wired
| U# | present (file::symbol) | wired (driver/reach) | evidence |
|----|------------------------|----------------------|----------|
| U1 | runtime/swarm-immune.mjs::immuneGraph + live var/lib/ruvector/swarm-immune.mjs | required by T1; live sha == canonical | sha `971a54f8…` live==canonical; T1 GREEN |
| U2 | t1_swarm_immune.mjs | driven by guardian; RED pre / GREEN post | RED `NumberExpected`→ GREEN exit 0 ×2 |
| U3 | t2_envctl_db_fresh.sh | runs `envctl db --help` | RED exit 1 (`unrecognized subcommand 'db'`) |
| U4 | t3_embedder_wiring.sh | psql SELECT + manifest read (read-only) | RED exit 1 (500/5157 + manifest fallback) |
| U5 | t4_router_discrimination.mjs + fixtures/router_prompts.json | ROUTER_DIR router, parses modelTier | RED exit 1 both modes |
| U6 | t6_musl_static.sh | file(1) on musl target binary | RED exit 1 (target absent) |
| U7 | crates/codedb_store_pg/tests/blobstore_parity.rs::pg_redb_blobstore_parity + 2 dev-deps | `cargo test -p codedb-store-pg --test blobstore_parity` | default 1 ignored; opt-in 1 passed |
| U8 | install.sh | archives→installs→stages; idempotent RED→GREEN mechanism | install ran; archive+live sha verified; idempotent |

## Runtime check — PASS (every declared surface driven; raw captures)

Diff discipline (pre-req):
- envctl worktree `git status --porcelain` → ONLY `?? scripts/tests/blueprint/` + `?? .handoff/loop/cycle/01_architect_plan.md` + `?? 02_implementer_log.md`; `git diff --stat`/`--cached` empty → **no engine/cli/gui/manifest/lock/crates touch**. PASS.
- nu_plugin worktree (branch t5-blobstore-parity) → ` M Cargo.lock  M crates/codedb_store_pg/Cargo.toml  ?? crates/codedb_store_pg/tests/`; diff = +21 Cargo.lock / +7 Cargo.toml (dev-deps only: codedb-store-redb path, tempfile="3"; lock adds tempfile 3.27.0 + fastrand 2.4.1 — both pure-Rust). PASS.

3a. R1/T1 GREEN live — run TWICE (fresh processes, `bash -lc 'cd var/lib/ruvector && bun .../t1_swarm_immune.mjs'`):
```
(run 1 and run 2 — byte-identical, exit=0 each)
PASS pre.connected===true / pre.weakestCoupling===1.0 / pre.boundary {coordinator, review-gate}@1.0
PASS post.connected===true / post.weakestCoupling===0.01 / post.boundary {researcher, merge-resolver}@0.01
T1 GREEN: all assertions passed (isolation boundary shifted coordinator/review-gate@1.0 -> researcher/merge-resolver@0.01)
```
Deterministic across both fresh processes (implementer's 3/3 + guardian's 2/2 = 5/5 GREEN). connected()==true both pre and post.

3b. RED reproduction from the Law-1 archive (read-only throwaway import of the broken wrapper):
```
THREW at immuneGraph: Error: Failed to convert napi value String into rust type `u32`
code: NumberExpected
```
Archive + live untouched (verified read-only).

3c. Install sync:
```
live      swarm-immune.mjs sha = 971a54f8fd4da2275a5557b00f6e2c3227835d4e42da30a67379f7f9c60c497b
canonical swarm-immune.mjs sha = 971a54f8fd4da2275a5557b00f6e2c3227835d4e42da30a67379f7f9c60c497b   (== live)
archived  original       sha  = d8de6e413faec64dacd0f2272906d62da29b5c9167e0e9ebae7888b7718d7152   (== expected)
var/lib/ruvector/tests/ : t1_swarm_immune.mjs, t3_embedder_wiring.sh, t4_router_discrimination.mjs, fixtures/router_prompts.json  (staged)
```

3d. T2/T3/T6 RED (each exit 1, self-describing FAIL line):
```
T2: error: unrecognized subcommand 'db' (exit 2) -> T2 RED
T3: codebase total=5157 minilm_embedded=500; manifest model="agentdb fallback embedder (no local model wired yet)" -> T3 RED   (read-only: 2× SELECT count + manifest read)
T6: musl target binary absent -> T6 RED
```

3e. T4 RED — both ROUTER_DIR modes (each exit 1); runner treats ANY mismatch-vs-fixture as FAIL (t4_router_discrimination.mjs:76):
```
mode 1 (default meta-ruvector, full fixture): 10/10 cases -> ["ABSENT","ABSENT","ABSENT"]  -> T4 RED
mode 2 (meta-ruvector-router-wt, first 3):
  [expect haiku] "fix a typo in the README"          -> ["haiku","haiku","sonnet"]   FAIL (instability)
  [expect haiku] "rename the local variable foo..."  -> ["opus","opus","opus"]       FAIL (wrong tier)
  [expect haiku] "add a trailing newline..."         -> ["sonnet","sonnet","opus"]   FAIL (non-discriminating)
  -> T4 RED (3/3)   — RED holds regardless of which wrong tier appears
```

3f. T5 (nu_plugin worktree):
```
default   : test result: ok. 0 passed; 0 failed; 1 ignored   (default-green preserved)
opt-in    : T5 GREEN: PgStore == redb parity on 5 files (relative_path + sha256); temp table codedb_t5_127082 dropped
            test result: ok. 1 passed; 0 failed
```
Table safety (from source): dedicated `codedb_t5_<pid>`, `assert_ne!` vs `codebase`/`codebase_codedb`, DROP before assert. Production tables unchanged around the run: `SELECT count(*) FROM codebase` = 5157 before AND after; `codebase_codedb` = 4 before AND after; no leaked `codedb_t5_%` table before or after.

Off-happy-path (fail-closed) probe — `degrade` of a nonexistent edge (canonical wrapper):
```
BEFORE {"connected":true,"weakest":1,"boundary":[["b","c",1]]}
AFTER  {"connected":true,"weakest":1,"boundary":[["b","c",1]]}
FAIL-CLOSED OK: nonexistent-edge degrade was a safe no-op (graph unchanged, no throw)
```

## Deviation scrutiny — SANCTIONED (NOTE, not FAIL): `degrade` rebuilds via `MinCut.fromEdges`
The implementer replaced the plan's `deleteEdge`+`insertEdge` degrade body with an edge-list-mutation
+ `fromEdges` rebuild. All four required conditions hold:
- (i) Frozen surface byte-identical in shape — `{g, isolationBoundary(), weakestCoupling(), degrade(u,v,w), connected()}`; `g` is now a transparent getter (property-access reads identically). VERIFIED (runtime/swarm-immune.mjs:45–63).
- (ii) Rebuild semantics = same graph — maintains `numericEdges`, mutates the matching edge's weight in **both** endpoint orders, rebuilds via `fromEdges`; degrade of a nonexistent edge is a runtime-proven fail-closed no-op (graph unchanged, no throw). VERIFIED (probe above).
- (iii) Addon defect is real — I reproduced the native `deleteEdge(1,4)`+`insertEdge(1,4,0.01)` form across 8 fresh processes: threw `Failed to insert edge: Edge already exists: (1, 4)` on **3/8** (runs 1,2,5); the shipped rebuild form is deterministic (T1 GREEN 5/5). VERIFIED.
- (iv) Documented honestly — implementer log Deviations 1–4 + the wrapper header comment (runtime/swarm-immune.mjs:7–23). VERIFIED.
Plan explicitly allowed adaptation on observed staleness at this exact spot → sanctioned robustness deviation.
Deviations 2 (weakestCoupling kept as minCutValue), 3 (router-wt empirically unstable haiku/sonnet, not
"constant-opus" — RED stronger, confirmed by 3e), 4 (postgres already a normal dep, 2 dev-deps met) all verified accurate.

## No-regression — CONFIRMED
- 5 `.rvf.db` agent containers (coordinator/researcher/materializer/review-gate/merge-resolver): all present, mtimes 04:36–09:36 (pre-16:54-install) → untouched by this cycle.
- `reasoningbank.db` (`var/lib/agentdb/reasoningbank.db`, mtime 04:18) → untouched; not in any cycle write path.
- router files (`.../meta-ruvector/.claude/helpers/router.js` 15:43, `.../meta-ruvector-router-wt/...` 06:48) → pre-cycle; T4 reads only.
- `codebase`/`codebase_codedb` (5157/4) + `ruvector.db` (07:35) → unchanged.
- Only cycle mutations: `swarm-immune.mjs` (16:54, Law-1 archived first) + new `tests/` dir (16:54) + T5 temp-table create/drop. (The `runtime/` subdir 17:16/17:19 mtimes are the documented background bun/claude-flow daemon refreshing node_modules/bun.lock/tools — unrelated to this cycle.)

## Lock/manifest — unchanged (envctl worktree)
`manifest/envctl.lock`, `agent-env.lock`, root `Cargo.lock`, `crates/`, `manifest/` — all clean in
`git status --porcelain` (no tracked file modified). agent-env.sh confirms lock up to date.

## Findings
- N1 (severity: info / sanctioned) runtime/swarm-immune.mjs:54 — `degrade` uses rebuild not native delete+insert. All four scrutiny conditions hold; the native pair is nondeterministically broken (reproduced 3/8). Accept as-is. No fix needed.
- N2 (severity: note / inherited) `cargo test --workspace` in envctl was starved by concurrent CI (PR #463 runners) and did not return. Axis = INHERITED (script-only change, zero Rust touched in envctl) → non-blocking. The only touched Rust anywhere (nu_plugin T5) is compile-clean, rustfmt-clean, and GREEN. Suggested: the orchestrator may re-confirm the inherited baseline out-of-band once the runners free up; it does not gate this cycle.
- N3 (severity: note / env) `cargo fmt`/`cargo clippy` cargo-subcommand shims absent in this fenix toolbin (only `rustfmt`/`clippy-driver`). Not a code defect; matches implementer note. Touched Rust verified rustfmt-clean directly.

## Re-test needed
None blocking. Optional inherited-baseline reconfirm (out-of-band, after CI frees the runners):
```
cd /home/flexnetos/lifeos/src/envctl-ff-blueprint-tests-wt && rtk proxy cargo test --workspace
```
All cycle acceptance already observed GREEN/RED-as-designed above.
