# test-strategy-icm (cycle 7)

dimension: test-coverage
target: icm (4-crate workspace: icm-core / icm-store / icm-mcp / icm-cli)
red-worktree: /home/drdave/Desktop/meta/.worktrees/plan-icm-red/icm @ branch plan/icm-red-tests
commit: 258667eb97e40f0a92b6b4131584a5e1844fa265
red-test-file: /home/drdave/Desktop/meta/.worktrees/plan-icm-red/icm/crates/icm-store/tests/recency_decay_red.rs
tests-ran: 5

## Verdict (1-3 lines)
Coverage of the store is broad on CRUD/search/dedup/decay-mechanics but the
highest-value CONVERGENCE capability — recency-aware (time-decaying) importance —
is absent and untested. Authored a 5-test additive RED suite that pins the
recency contract; all 5 ran and are RED for the right reason (decay is time-blind:
every memory ends at weight 0.95 regardless of `last_accessed`). Suite size: 5.

## Convergence contract chosen
**#1 Dynamic, time-aware (recency / Ebbinghaus) importance & decay.**
Selected over the bind-as-data envelope (#4) because it is the most
evidence-grounded and yields BEHAVIORAL RED tests against the *existing* public
API (no nonexistent type referenced -> no compile-error false-RED). For the
lifeos meta front-door union (icm <-> handoff <-> rusty-idd) a memory surfaced
into a handoff capsule must be recency-weighted; today it is not.

## Existing coverage (CLAIM rows — reachability, not file presence)
- CLAIM: decay MECHANICS are tested but only for importance/access_count axes, never the time axis | evidence: crates/icm-store/src/store.rs:3596 `test_apply_decay`, :3609 `test_apply_decay_caps_access_count_amplification`, :4574 `test_decay_bulk`, :5545 `test_apply_decay_with_aggressive_factor`, :5890 `test_apply_decay_clears_cache` all drive `apply_decay` (src/store.rs:1267) which reads only `importance` + `MIN(access_count,5)` | confidence: high
- CLAIM: `SqliteStore::apply_decay` is time-blind — its UPDATE never references `last_accessed` or `created_at` | evidence: src/store.rs:1290-1305 SQL `weight = weight * (1.0 - (1.0-?1)*CASE importance .. / (1.0+MIN(access_count,5)*0.1)) WHERE importance != 'critical'` | confidence: high
- CLAIM: `maybe_auto_decay` (hotspot, the daily auto path) calls the same flat `apply_decay(0.95)` and has NO test asserting recency weighting | evidence: src/store.rs:130-151 -> :147 `self.apply_decay(0.95)`; no test references `maybe_auto_decay` recency | confidence: high
- CLAIM: recall ranking (`search_by_keywords`, `search_fts`) orders strictly `ORDER BY weight DESC` with no recency tie-break, untested for recency | evidence: src/store.rs:1045 and :1079; tests `test_search_fts`/`test_search_by_keywords` (src/store.rs:3435,:3453) assert membership/limits, not recency order | confidence: high
- CLAIM: `Memory.last_accessed` / `created_at` are persisted faithfully by `store` and `update` (so the contract is testable today) | evidence: store_inner INSERT src/store.rs:789, update src/store.rs:971, validate_and_normalize (src/store.rs:677) does NOT touch timestamps | confidence: high

## Coverage gaps (ranked, highest-risk first)
- GAP: hotspot `apply_decay` (decay engine, called by daily `maybe_auto_decay`) has zero tests on the recency/elapsed-time dimension — the exact convergence capability the front-door union needs. | blast: every memory's rank over time
- GAP: hotspot `SqliteStore.get` (blast 183) returns memories whose weight never reflects staleness; no end-to-end test that a stale memory ranks below a fresh one. | blast: every recall consumer (mcp recall, cli recall, handoff capsule)
- GAP: no recency FLOOR test — a just-accessed memory is decayed identically to an ancient one. | blast: actively-used memories are punished equally with abandoned ones

## Designed suite (UPGRADE rows — the RED tests authored this cycle)
- UPGRADE: add integration test for recency decay (stale > fresh weight loss) | axis: accuracy | rationale: closes the time-axis decay gap | evidence: src/store.rs:1267 apply_decay time-blind | blast: guards decay engine + all recall ranking | risk: low
- UPGRADE: add integration test for monotonic decay vs staleness | axis: accuracy | rationale: pins decay as a function of elapsed time | evidence: src/store.rs:1290 SQL ignores last_accessed | blast: guards decay engine | risk: low
- UPGRADE: add integration test for recall re-rank (fresh out-ranks staler-but-higher-weight) | axis: accuracy | rationale: ties recency into the weight-DESC recall path | evidence: src/store.rs:1045 search_by_keywords | blast: guards recall ordering used by mcp/cli/handoff | risk: low
- UPGRADE: add integration test for fresh-memory recency floor (negligible decay when last_accessed≈now) | axis: accuracy | rationale: a recently-used memory must not decay like an abandoned one | evidence: src/store.rs:147 apply_decay(0.95) flat | blast: guards active-memory survival | risk: low
- UPGRADE: add integration test for stale-memory magnitude (~400d untouched decays substantially in one pass) | axis: accuracy | rationale: decay magnitude must reflect accrued elapsed time | evidence: src/store.rs:1290 flat factor | blast: guards pruning/ranking of abandoned memories | risk: low

## traceability  (plan item ↔ acceptance criterion ↔ test ↔ RED|GREEN)
| contract / claim | acceptance criterion | test (crates/icm-store/tests/recency_decay_red.rs) | status |
|---|---|---|---|
| recency decay core | stale (365d) loses more weight than fresh under one decay pass | `decay_stale_memory_loses_more_weight_than_fresh` | RED |
| recency decay monotonic | weight after decay strictly decreases with staleness (0/30/180d) | `decay_magnitude_is_monotonic_in_staleness` | RED |
| recall re-rank by recency | fresh out-ranks a 2y-stale memory of higher initial weight after decay | `recall_ranks_fresh_above_stale_after_decay` | RED |
| recency floor | memory accessed now barely decays (weight > 0.99) in one pass | `fresh_memory_decay_is_negligible` | RED |
| decay magnitude vs elapsed time | ~400d-untouched memory decays below 0.7 in one pass | `very_stale_memory_decays_substantially` | RED |

## RED run evidence (P8 count verification)
- command: `cargo test -p icm-store --test recency_decay_red`
- tests-ran: 5  (exit-0-with-zero-tests would be a FAIL; this ran 5)
- result: FAILED — 0 passed; 5 failed; 0 ignored
- expected RED failure reason: decay/recall is time-blind. Observed panics confirm the RIGHT reason (not compile/API error):
  - `stale=0.95, fresh=0.95` (equal — recency ignored)
  - `0d=0.95, 30d=0.95, 180d=0.95` (no monotonicity)
  - `got top topic = rank-stale` (no recency re-rank)
  - `got weight=0.95` for a now-accessed memory (no floor)
  - `got weight=0.95` for a 400d-stale memory (no magnitude scaling)
- clippy: `cargo clippy -p icm-store --tests --all-targets -- -D warnings` -> "No issues found" (CI-safe; CI runs `clippy --workspace --all-targets -- -D warnings`)
- additive-only: single new file `crates/icm-store/tests/recency_decay_red.rs`; no product-code edits; no committed *.db; chrono is already an icm-store dependency so no manifest change was needed.

## FF test-build spec  (Feature-Forge handoff: GREEN implementation that flips each RED test)
Surface to change (engine-first, icm-store): `crates/icm-store/src/store.rs`.
Make decay a function of elapsed wall-clock time since `last_accessed`, layered
on the existing importance/access_count multipliers (do NOT drop those — No
Downgrade).

- Cases / GREEN intent (one per RED test):
  - `decay_stale_memory_loses_more_weight_than_fresh`: extend the `apply_decay`
    SQL (src/store.rs:1290) so the effective decay multiplies by an
    elapsed-time term, e.g. `weight = weight * recency_factor(last_accessed, now)`
    where older `last_accessed` -> smaller factor. Add a sibling method
    `apply_recency_decay(now)` (or pass `now` into the existing path via
    `maybe_auto_decay`) so the daily path becomes time-aware.
  - `decay_magnitude_is_monotonic_in_staleness`: recency_factor must be strictly
    decreasing in `(now - last_accessed)` (e.g. exponential `exp(-Δdays/τ)` with
    a tunable half-life τ, defaulted in config).
  - `recall_ranks_fresh_above_stale_after_decay`: no recall change required IF
    decay updates persisted weight before `search_by_keywords`/`search_fts`
    (both order by weight DESC). Confirm `maybe_auto_decay` runs (or decay is
    applied) before recall, OR add a recency tie/boost term in the recall ORDER
    BY. The test asserts via the existing weight-DESC path, so persisting a
    recency-decayed weight is sufficient.
  - `fresh_memory_decay_is_negligible`: recency_factor(Δ≈0) ≈ 1.0 (floor) so a
    just-accessed memory keeps weight > 0.99 in one pass. Pick τ so the per-pass
    floor holds for Δ=0.
  - `very_stale_memory_decays_substantially`: recency_factor(Δ≈400d) drives
    weight < 0.7 in a single pass (τ on the order of weeks–months, not years).
- Differential / golden fixtures to capture: snapshot `(last_accessed_age_days,
  importance, access_count) -> post-decay weight` for a grid (ages {0,30,180,
  365,400}, importance {high,medium,low}) as a golden table so the recency curve
  is behavior-pinned and `critical` is still skipped (existing invariant at
  src/store.rs:1302 `WHERE importance != 'critical'`).
- Coverage target: recency dimension of `apply_decay` + `maybe_auto_decay` +
  the recall ordering path reachable from `get`/`search_*` — all currently 0%
  on the time axis.
- CI gate(s) touched: `cargo test --workspace` (new integration test target
  `recency_decay_red`) and `cargo clippy --workspace --all-targets -- -D warnings`.
- Invariant guardrails for the implementer: keep `critical` decay-exempt; keep
  the access_count anti-gaming cap (MIN(access_count,5), Audit #185 H7); No-C
  trust boundary (pure SQL/Rust); recency term must be config-tunable, not a
  magic constant; do NOT modify the RED test file to make it pass (tests are
  additive-only / acceptance-frozen).
