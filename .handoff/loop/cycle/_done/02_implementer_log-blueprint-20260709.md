STATUS: GREEN

# Implementation log: Blueprint test suite T1–T6 + R1 swarm-immune wrapper fix

Sequential single-crew. envctl worktree `/home/flexnetos/lifeos/src/envctl-ff-blueprint-tests-wt`
(branch `ff-blueprint-tests`) — additive files under `scripts/tests/blueprint/` only, no
Cargo/Rust/manifest/lock change. nu_plugin worktree
`/home/flexnetos/lifeos/src/nu_plugin-t5-parity-wt` (branch `t5-blobstore-parity` off master @
`532a461`) — one `#[ignore]`d differential test + 2 dev-deps. **Nothing committed anywhere; both
trees left dirty for the orchestrator.**

## Changes
- `scripts/tests/blueprint/runtime/swarm-immune.mjs` (NEW): canonical R1 fix of the ruvector MinCut
  wrapper — names→u32 map before `fromEdges`; `cutEdges()`/`isConnected()` called as METHODS;
  `degrade()` rebuilds via `fromEdges` (see Deviations). Frozen surface preserved.
- `scripts/tests/blueprint/t1_swarm_immune.mjs` (NEW): U2 bun integration test; drives production by
  absolute path (`SWARM_IMMUNE` override), pre/post-degrade min-cut assertions, unordered endpoint
  set + float epsilon.
- `scripts/tests/blueprint/t2_envctl_db_fresh.sh` (NEW): U3 `envctl db --help` smoke.
- `scripts/tests/blueprint/t3_embedder_wiring.sh` (NEW): U4 read-only psql SELECT (codebase MiniLM
  coverage) + agentdb manifest model read.
- `scripts/tests/blueprint/t4_router_discrimination.mjs` (NEW): U5 golden-fixture router runner
  (`ROUTER_DIR` env, parses `modelTier`, 3 stability runs, absent=FAIL). Full 10-case fixture by
  default; `T4_LIMIT` caps only for fast RED capture.
- `scripts/tests/blueprint/fixtures/router_prompts.json` (NEW): the exact 10-case golden fixture.
- `scripts/tests/blueprint/t6_musl_static.sh` (NEW): U6 `file(1)` static-musl build smoke.
- `scripts/tests/blueprint/install.sh` (NEW): U8 installer — Law-1 archive of the live wrapper →
  install fix → stage bun/psql tests to `var/lib/ruvector/tests/`. Idempotent; targets
  `var/lib/ruvector/` + `~/.claude/archive/` only.
- var-runtime (NOT git): `/home/flexnetos/lifeos/var/lib/ruvector/swarm-immune.mjs` replaced by U8
  (broken original Law-1 archived); tests staged under `var/lib/ruvector/tests/`.
- nu_plugin `crates/codedb_store_pg/Cargo.toml` (MOD): `[dev-dependencies]` += `codedb-store-redb`
  (path), `tempfile = "3"`.
- nu_plugin `crates/codedb_store_pg/tests/blobstore_parity.rs` (NEW): U7 differential parity test.
- nu_plugin `Cargo.lock` (MOD, auto): `tempfile v3.27.0` + `fastrand v2.4.1` added by the dev-dep.

## Engine API (parity contracts — no envctl Engine changed; this is script/JS/Rust-test only)
- **Frozen JS surface (R1)** — preserved exactly:
  `immuneGraph(edges) -> { g, isolationBoundary(), weakestCoupling(), degrade(u,v,w), connected() }`.
  `g` is now a getter (returns the live graph after a rebuild-degrade); the four methods are
  unchanged in signature/semantics. Names are the string agent labels; internally mapped to u32.
- **BlobStore parity (T5)** — `codedb_core::store::BlobStore`: `PgStore` (PostgreSQL) must equal the
  `CaptureBatcher` (redb) reference on `list_source_files()` → set of `(relative_path, sha256)` and
  `captured_paths()` for identical input. Both compute `format!("{:x}", Sha256::digest(bytes))`.

## Tests added
- **T1** `pre/post-degrade` swarm-immune assertions — proves the fixed wrapper connects the graph,
  reports `weakestCoupling` 1.0→0.01, and shifts the isolation boundary
  `{coordinator,review-gate}@1.0 → {researcher,merge-resolver}@0.01` (RED `NumberExpected` pre-fix).
- **T2** — proves `envctl db` is unwired now (RED); GREEN when R2 lands the verb.
- **T3** — proves MiniLM embedder unwired now (partial coverage + manifest "fallback"); GREEN after R3.
- **T4** — proves router non-discrimination now (tier-absent default; unstable router-wt); GREEN after R4.
- **T5** `pg_redb_blobstore_parity` — proves the PostgreSQL backend is byte-for-byte (path+sha256)
  parity with the redb reference; `#[ignore]`d + `CODEDB_PG_TEST=1`-guarded, dedicated
  `codedb_t5_<pid>` temp table dropped at end.
- **T6** — proves no static-musl build now (RED); GREEN after R9.

## Build/test status (exact commands + RAW captures)

### (a) live wrapper sha (no drift — matches intake `d8de6e…7152`)
```
$ sha256sum /home/flexnetos/lifeos/var/lib/ruvector/swarm-immune.mjs
d8de6e413faec64dacd0f2272906d62da29b5c9167e0e9ebae7888b7718d7152  …/swarm-immune.mjs
```

### (c) T1 RED — against the STILL-BROKEN live wrapper  [PASS: correct RED]
```
$ bash -lc 'cd /home/flexnetos/lifeos/var/lib/ruvector && bun …/t1_swarm_immune.mjs'
T1 swarm-immune wrapper — under test: /home/flexnetos/lifeos/var/lib/ruvector/swarm-immune.mjs
11 |   const g = MinCut.fromEdges(edges);
                        ^
error: Failed to convert napi value String into rust type `u32`
 code: "NumberExpected"
      at immuneGraph (/home/flexnetos/lifeos/var/lib/ruvector/swarm-immune.mjs:11:20)
exit=1
```

### (d) install.sh — Law-1 archive → install → stage  [PASS]
```
$ bash …/blueprint/install.sh
== install.sh: R1 swarm-immune fix + blueprint test staging ==
archived live wrapper -> /home/flexnetos/.claude/archive/20260709T215424Z/var-lib-ruvector/swarm-immune.mjs (Law-1)
installed R1 fix -> /home/flexnetos/lifeos/var/lib/ruvector/swarm-immune.mjs
staged bun/psql tests -> /home/flexnetos/lifeos/var/lib/ruvector/tests/ (t1, t4, t3 + fixtures/router_prompts.json)
install.sh complete
exit=0
$ sha256sum <live> <canonical>
971a54f8fd4da2275a5557b00f6e2c3227835d4e42da30a67379f7f9c60c497b  <live>
971a54f8fd4da2275a5557b00f6e2c3227835d4e42da30a67379f7f9c60c497b  <canonical>   # live == canonical
$ sha256sum <archived original>
d8de6e413faec64dacd0f2272906d62da29b5c9167e0e9ebae7888b7718d7152  <archived>    # broken original preserved
```

### (e) T1 GREEN — against the INSTALLED fix, 3 fresh processes  [PASS, deterministic]
```
$ for i in 1 2 3; do bun …/t1_swarm_immune.mjs; done   # each run:
PASS pre.connected===true
PASS pre.weakestCoupling===1.0
PASS pre.boundary endpoints === {coordinator, review-gate}
PASS pre.boundary weight === 1.0
PASS post.connected===true
PASS post.weakestCoupling===0.01
PASS post.boundary endpoints === {researcher, merge-resolver}
PASS post.boundary weight === 0.01
T1 GREEN: all assertions passed (… coordinator/review-gate@1.0 -> researcher/merge-resolver@0.01)
exit=0   (×3, identical)
```
Also GREEN via the canonical copy (guardian alt path): `SWARM_IMMUNE=…/runtime/swarm-immune.mjs bun t1…` → `T1 GREEN … exit=0`.

### install.sh idempotency  [PASS]
```
$ bash …/install.sh          # re-run
live wrapper already matches the fix — no archive needed (idempotent)
…
archive dirs before=1 after=1 (idempotent — no new archive)
exit=0
```

### T2 RED  [PASS: correct RED]
```
$ bash …/t2_envctl_db_fresh.sh
--- output (exit=2) ---
error: unrecognized subcommand 'db'
  tip: a similar subcommand exists: 'dashboard'
FAIL: 'envctl db --help' exit=2 (unrecognized subcommand until R2 wires the db verb)
T2 RED
exit=1
```

### T3 RED  [PASS: correct RED]
```
$ bash …/t3_embedder_wiring.sh
codebase: total=5157  minilm_embedded=500
FAIL: only 500 / 5157 codebase rows are MiniLM-embedded (embedder not fully wired until R3)
manifest model = "agentdb fallback embedder (no local model wired yet)"
FAIL: manifest embedder model is still the fallback (real model not wired until R3)
T3 RED (expected until R3)
exit=1
```

### T4 RED — mode 1 (default ROUTER_DIR = meta-ruvector, tier-ABSENT)  [PASS: correct RED]
```
$ bun …/t4_router_discrimination.mjs      # cases=10
FAIL [expect haiku] "fix a typo in the README" -> ["ABSENT","ABSENT","ABSENT"]
… (all 10 ABSENT) …
T4 RED: 10/10 cases failed
exit=1
```
### T4 RED — mode 2 (ROUTER_DIR = meta-ruvector-router-wt, NON-discriminating)  [PASS: correct RED]
```
$ ROUTER_DIR=…/meta-ruvector-router-wt bun …/t4_router_discrimination.mjs
FAIL [expect haiku] "update the copyright year in the license header" -> ["opus","opus","opus"]
FAIL [expect opus]  "design a Byzantine fault-tolerant consensus protocol …" -> ["haiku","haiku","haiku"]
… (unstable haiku/sonnet/opus, no complexity correlation) …
T4 RED: 10/10 cases failed
exit=1
```

### T6 RED  [PASS: correct RED]
```
$ bash …/t6_musl_static.sh
expected binary: …/target/x86_64-unknown-linux-musl/release/envctl
FAIL: musl target binary absent (add target x86_64-unknown-linux-musl and build --release after R9)
T6 RED
exit=1
```

### T5 compile-only (0 run — #[ignore]d)  [PASS]
```
$ cd nu_plugin-t5-parity-wt && rtk proxy cargo test -p codedb-store-pg --test blobstore_parity
   Compiling codedb-store-pg v0.1.0 (…/crates/codedb_store_pg)
    Finished `test` profile … in 5.45s
running 1 test
test pg_redb_blobstore_parity ... ignored, drives the live PostgreSQL cluster; opt in with CODEDB_PG_TEST=1 -- --ignored
test result: ok. 0 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out
exit=0
```

### T5 GREEN — live opt-in differential parity  [PASS]
```
$ CODEDB_PG_TEST=1 rtk proxy cargo test -p codedb-store-pg --test blobstore_parity -- --ignored --nocapture
running 1 test
T5 GREEN: PgStore == redb parity on 5 files (relative_path + sha256); temp table codedb_t5_4130543 dropped
test pg_redb_blobstore_parity ... ok
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
exit=0
```
Temp-table leak check + skip-guard:
```
$ psql -tAc "select table_name … like 'codedb_t5_%'"      -> (empty; no leak)
$ rtk proxy cargo test … -- --ignored --nocapture          # WITHOUT CODEDB_PG_TEST
SKIP: CODEDB_PG_TEST != 1 — set it to run the live PostgreSQL/redb differential parity test
test result: ok. 1 passed; 0 failed …
exit=0
```

### fmt / clippy on the touched crate  [PASS — touched code clean]
`cargo-fmt`/`cargo-clippy` subcommand shims are absent in this fenix toolbin; `rustfmt` +
`clippy-driver` binaries are present, so I drove them directly (clippy via the same
`RUSTC_WORKSPACE_WRAPPER=clippy-driver` mechanism `cargo clippy` uses).
```
$ rustfmt --edition 2024 --check …/tests/blobstore_parity.rs   # after in-place format
recheck exit=0     (clean)
$ RUSTC_WORKSPACE_WRAPPER=clippy-driver cargo check -p codedb-store-pg --tests
warning: … base64 (lib) generated 2 warnings         # INHERITED third-party dep only
clippy(check) exit=0     # zero warnings on codedb-store-pg / blobstore_parity.rs
```
Classification: the only clippy/rustc warnings are from the `base64` **registry dependency**
(`mismatched_lifetime_syntaxes`, `unexpected_cfgs` under rustc 1.93.1) — inherited, present on
master; not from touched code. No broad `#[allow]` added.

### envctl gates most affected by the new scripts  [PASS]
```
$ bash ci/gates/meta-local-policy.sh   -> "active install sources target META_ROOT FHS/XDG…"  exit=0
$ bash ci/gates/harness-scripts.sh     -> "HARNESS-SCRIPTS GATE PASS"                          exit=0
```
(install.sh writes only `var/lib/ruvector/` + `~/.claude/archive/` — no usr/bin frontdoor; the new
`blueprint/` subdir is inert to the fixed-list harness-scripts gate.)

### git status (nothing committed)
```
envctl:    ?? scripts/tests/blueprint/            (+ orchestrator's .handoff/…/01_architect_plan.md)
nu_plugin:  M Cargo.lock   M crates/codedb_store_pg/Cargo.toml   ?? crates/codedb_store_pg/tests/
```

## Deviations
1. **`degrade()` uses REBUILD, not `deleteEdge`+`insertEdge` (the plan's exact wrapper body).**
   Forced by a proven, nondeterministic bug in the native addon. RAW evidence: the plan's
   `deleteEdge(1,4)` then `insertEdge(1,4,…)` — which IS the exact T1 degrade (researcher=1,
   merge-resolver=4) — throws `Failed to insert edge: Edge already exists: (1, 4)` on a
   process-hash-seed-dependent ~2/3 of fresh runs, and when it throws the graph is left inconsistent
   (`cutEdges()` shows the new edge but `minCut().value`/`minCutValue` read 0 while `isConnected()`
   is still true). Reversed-order insert is *also* nondeterministic. `MinCut.fromEdges` is the one
   reliable primitive: rebuild-degrade produced the correct, deterministic result in 6/6 isolated
   fresh processes and T1 GREEN in 3/3. **The frozen surface and the T1 assertion math are
   unchanged** — only the internal mechanism of `degrade` differs. This is an implementer-level
   robustness fix within the frozen contract (the plan itself flagged robustness uncertainty at this
   exact spot), not a design/contract change. `g` is exposed as a getter so a caller reading `.g`
   after a degrade sees the rebuilt graph, not a stale handle.
2. **`weakestCoupling` kept as `g.minCutValue` (no fallback needed).** The plan's sanctioned fallback
   `g.minCut().value` is for a *stale* `minCutValue` post-degrade. With rebuild-degrade, `minCutValue`
   reads correctly (`0.01`) every run (RAW: probe + T1 GREEN ×3), so the fallback was unnecessary and
   not used.
3. **T4 router-wt constant is empirically `haiku`/`sonnet` (unstable), not the plan's "constant-opus
   V7".** The RuvLTRA/agentdb classifier returns wildly varying tiers with no complexity correlation
   (RAW above: "Byzantine consensus"→haiku×3; trivial "copyright year"→opus×3). The RED is *stronger*
   than a constant-opus would be — the 3-stability-run design catches both tier-absence and
   instability. Only the descriptive label differs; the test behaves as specified (RED now).
4. **`postgres` used directly in the T5 test as a normal dep, not a 3rd dev-dep.** codedb-store-pg
   already declares `postgres = "0.19"` in `[dependencies]`, and a package's normal deps are in scope
   for its integration tests — confirmed by the clean compile. So the DROP-table admin client needs
   no extra dev-dep; the plan's "2 dev-deps" (codedb-store-redb, tempfile) is met exactly.

## Handoff notes (for the invariant-guardian)
- **R1 rebuild-degrade is the key thing to scrutinize** (Deviation 1). The native addon
  (`…/mincut/ruvector_mincut.node`, a compiled `.node` — out of scope to modify) is nondeterministic
  on `insertEdge` after `deleteEdge`. To reproduce the RED yourself: run the plan's delete+insert
  form in a loop of fresh `bun` processes and you'll see intermittent `Edge already exists`. The
  shipped rebuild form is deterministic — re-run `t1_swarm_immune.mjs` any number of times; it is
  GREEN every time. `SWARM_IMMUNE=<path>` drives either the live installed wrapper (default) or the
  canonical `runtime/swarm-immune.mjs`; both GREEN.
- **T1 RED→GREEN is the U1+U2 acceptance** and is fully reproducible: the live wrapper was Law-1
  archived to `/home/flexnetos/.claude/archive/20260709T215424Z/var-lib-ruvector/swarm-immune.mjs`
  (sha `d8de6e…7152`, the broken original) before install; the live path is now the fix (sha
  `971a54f8…`). `install.sh` is idempotent (cmp-guarded archive).
- **T3 has a moving RED anchor.** The MiniLM count is climbing (250→500 observed across two runs — a
  background re-embed job is live). The **stable RED anchor is the manifest `model` = "…fallback…"**;
  even if the count reaches 5157 before you re-run, T3 stays RED on the manifest condition until R3
  updates it. Both are read-only.
- **T5 is fail-closed by design**: `#[ignore]`d AND `CODEDB_PG_TEST=1`-guarded (double gate); default
  `cargo test` never touches the socket. It uses a dedicated `codedb_t5_<pid>` table (asserted `!=`
  `codebase`/`codebase_codedb`), dropped before the parity assert (verified no leak). Re-run:
  `CODEDB_PG_TEST=1 rtk proxy cargo test -p codedb-store-pg --test blobstore_parity -- --ignored --nocapture`.
- **No-C boundary**: nu_plugin dev-deps are pure-Rust (redb 2.6.3, tempfile→fastrand); no
  SQLite/OpenSSL/aws-lc pulled. envctl dep graph is untouched (script-only) — `no-c.sh` unaffected.
- **Gate scope**: envctl change is additive scripts only; I ran the two gates most likely to interact
  with new `install -m` scripts (meta-local-policy, harness-scripts) — both PASS. The full gate suite
  + fmt/clippy/test on untouched envctl Rust is inherited-green and yours to confirm.
- **fmt/clippy toolchain gap**: this fenix toolbin lacks the `cargo-fmt`/`cargo-clippy` cargo shims
  (only `rustfmt`/`clippy-driver` binaries). I used those directly. If you standardize on
  `cargo fmt`/`cargo clippy`, note they will error `no such command` here — not a code defect.
- **Parallel mode**: N/A. Sequential single-crew; no grit wave, no symbols claimed/released. Both
  worktrees left dirty; the orchestrator owns all commits/merges/PRs after your PASS.
