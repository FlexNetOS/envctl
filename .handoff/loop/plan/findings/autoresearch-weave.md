# Autoresearch findings — weave (cycle 4)

| field | value |
|---|---|
| target | **weave** — A2A transport plane (SQLite-mailbox broker + terminal-pane injector) |
| code root (READ-ONLY) | /home/drdave/Desktop/meta/.worktrees/plan-weave-red/weave |
| snapshot | `@4fe2419` · branch `plan/weave-red-tests` · cycle 4 |
| audited | 2026-06-26 |
| recency window (90d) | 2026-03-28 → 2026-06-26 (from `research/weave.trends.md`) |
| inputs | `reports/codemap-weave.md`, `research/weave.trends.md`, weave `deny.toml`, `.github/workflows/ci.yml`, `scripts/supply_chain_audit.py`, `weave-core/src/store.rs`, `weave-core/src/model.rs`, `weave/src/main.rs` |
| verdict | weave has **NO repo-native code auto-research** (no `.kb/`, no git-kb config, no CI re-index) and **NO time-windowed web auto-research bot** (no dependabot/renovate). It DOES have a strong, fail-closed, **event-driven advisory currency gate** (deny.toml + supply_chain_audit.py + CI `audit`) with a genuine stale-ignore invalidation mechanism, and a runtime heartbeat staleness model that is the runtime analogue of (not a substitute for) research-staleness invalidation. |

---

## 1. Code auto-research

**CLAIM C1 — weave has NO repo-native code-intelligence index; there is no `.kb/`, no git-kb config, and no CI re-index step. [CONFIRMED]**
- `ls .kb` → `No such file or directory`; `find` for `config.toml` under any `kb` path → empty; no `.gitkb`/`gitkb` files.
- The only `git-kb`/`.kb/` string in the whole tree is a stray mention in `.handoff/loop/review_REPOWIRE_LOCAL_SOURCE_PARITY.md` (a harness note), not a wired index.
- `.github/workflows/ci.yml` (8 jobs: fmt, clippy, test, target-smoke, libsql, sign, libsql-sign, surfaces, libsql-surfaces, audit) contains **no indexing / kb / re-index job**.
- Evidence: directory listing of `/home/drdave/Desktop/meta/.worktrees/plan-weave-red/weave`; `.github/workflows/ci.yml:21-185`.

**CLAIM C2 — the code graph that drives this planning loop is built HARNESS-SIDE by the cartographer from `git-kb code`, snapshotted + diffed per cycle — NOT by weave itself. [CONFIRMED]**
- `reports/codemap-weave.md:6` states the map is "Built from `git-kb code` (2722 symbols / 9571 resolved edges / 36 deep source files)" and `graph/weave.metrics.json` is referenced for layering/SCC analysis (`codemap-weave.md:32, 113`).
- So "code auto-research" for weave is the **plan-cartographer's** per-cycle snapshot/diff against `graph/`, external to the target repo. The exact `git-kb code` invocation, snapshot, and delta-vs-previous live in the cartographer's `graph/` outputs, not in weave's tree.
- Entrypoint/public-API, hotspots, dead-code, unresolved calls, and cross-repo impact are therefore covered by the cartographer artifact (`codemap-weave.md` §Entry points, §Crate roles, §Dependency hygiene; the 774 ambiguous resolutions / 3 Tarjan SCCs flagged at `codemap-weave.md:113` are the "unresolved calls" the verifier must confirm). This finding does not re-derive them; it records WHERE code auto-research happens.

**Assessment.** weave is read-only-correct from the harness's view: the planning loop's code auto-research is satisfied by the cartographer graph snapshot/diff. The **gap** is that weave has no *repo-native* code intelligence — a contributor working in weave alone (outside the meta planning loop) gets no callers/callees/impact tooling and no CI guard that the graph stays fresh. For a 2722-symbol, 4-crate workspace this is a real (low-severity) observability gap, addressable without touching the trust boundary (see UPGRADE U1).

## 2. Web auto-research

**CLAIM W1 — weave has NO automated dependency-update bot: no `.github/dependabot.yml`, no `renovate.json`/`.renovaterc`. [CONFIRMED]**
- `ls` for `.github/dependabot.{yml,yaml}`, `renovate.json`, `.renovaterc*` → all `No such file or directory`.
- Grep for `dependabot|renovate` across `*.md`/`*.yml` finds only narrative mentions in `.handoff/` and `CHANGELOG.md`, never a config.
- Consequence: there is **no scheduled, time-windowed web pull** of new crate releases or advisories inside the repo. Crate-currency facts (the 90-day "is `rusqlite`/`reqwest`/`libsql` current" table) are produced HARNESS-SIDE by the trend-researcher (`research/weave.trends.md` §D, crates.io API accessed 2026-06-26), not by weave.

**CLAIM W2 — web auto-research / advisory currency IS enforced, but EVENT-DRIVEN (every CI run), not 90-day-windowed. Three layers. [CONFIRMED]**
1. **`deny.toml`** (WL-044) — `[graph] all-features = true` (deny.toml:27) so the scan sees the libsql remote-TLS crates absent from the default sqlite graph; `[advisories] version = 2` with an explicit, reasoned, time-bounded `ignore = [...]` of exactly 5 RUSTSEC ids (deny.toml:32-44); `[sources] unknown-registry = "deny"` (deny.toml:73).
2. **`scripts/supply_chain_audit.py`** (WL-075) — local mirror of the CI posture (4 checks, below).
3. **CI `audit` job** (`ci.yml:166-185`) — runs `python3 scripts/supply_chain_audit.py --allow-missing-cargo-deny` then `EmbarkStudios/cargo-deny-action@v2` `check advisories`. This is a **blocking required check**.
- The recency/90-day discipline itself is owned by the harness trend-researcher (`research/weave.trends.md:8` window `2026-03-28 → 2026-06-26`; §D re-verifies RUSTSEC-2026-0104 published 2026-04-22 against rustsec.org). The repo gate is "no UN-listed advisory may exist," which runs on every push — a stricter cadence than any 90-day window, but blind to *new releases* that are not yet advisories (that recency gap is what a renovate bot would close; see UPGRADE U2).

**CLAIM W3 — official-docs-first + contradiction checks are satisfied by the source ledger in `research/weave.trends.md`. [CONFIRMED]**
- The trends note carries a full Sources table (`weave.trends.md:212-232`) with publisher + published-date + in-window flags, official sources first (a2a-protocol.org, linuxfoundation.org, rustsec.org, crates.io API), and per-claim "Refute attempt" rows (e.g. §A1, §D advisory-cluster refute). Contradiction check example: the bincode advisory RUSTSEC-2025-0141 is recorded as **eliminated** by WL-044b's libsql feature trim — cross-checked against deny.toml's own NOTE (deny.toml:39-42) and `supply_chain_audit.py` `REMOVED_IGNORES` (line 29). No contradiction between ledger and repo state.

## 3. Cadence + stale-evidence invalidation

**CLAIM S1 — the deny.toml advisory ignore-set is exactly pinned and any drift fails closed. [CONFIRMED]**
- `supply_chain_audit.py:22-28` defines `EXPECTED_IGNORES` = the 5 RUSTSEC ids; `validate_deny_toml` (lines 58-76) fails on **missing** expected ignores, **extra** (unexpected) ignore ids, **reintroduced removed** ids (`REMOVED_IGNORES = {RUSTSEC-2025-0141}`, line 29), and if `all-features = true` is dropped. This is the "no blanket ignore / no silent new ignore" guard — both directions fail closed.

**CLAIM S2 — STALE-ADVISORY INVALIDATION: the libsql-pinned-rustls block self-invalidates when upstream unblocks. [CONFIRMED — this is the core stale-evidence mechanism]**
- `check_libsql_tree_tracks_tls` (`supply_chain_audit.py:92-117`): runs `cargo tree -i rustls-webpki --locked --no-default-features --features libsql`. It PASSES only while `rustls-webpki v0.102` is still present in the libsql graph. If rustls-webpki **disappears** from the libsql tree it returns **fail** with detail "rustls-webpki disappeared from libsql graph; verify libsql patched TLS and **remove stale deny.toml ignores**" (lines 107-112).
- This is exactly the WL-044b removal trigger documented in `deny.toml:11-15`: "remove each id below the moment libsql adopts the rustls 0.23 stack." The gate therefore *forces* invalidation of the now-**stale** ignore rows the instant the upstream pin lifts — the ignores cannot rot silently into permanent suppressions. `research/weave.trends.md:167-170` corroborates the upstream-block (patched rustls-webpki ≥0.103 needs rustls 0.23 / hyper-rustls 0.27 which libsql still pins to 0.25).
- `check_default_tree_clean` (lines 79-89) is the dual guard: it asserts the **default sqlite** graph has ZERO rustls-webpki, so the advisory budget can never silently leak into the default build.

**CLAIM S3 — per-cycle / batch / resume cadence is HARNESS-side, not repo-side. [CONFIRMED]**
- Repo cadence: the `audit` job + supply_chain_audit checks fire on **every** push/PR (`ci.yml:7-8`) — event-driven, continuous, no time window. cargo-deny re-resolves the advisory DB each run, so a newly-published advisory against any in-tree crate fails the next CI run automatically (the "stale evidence detected by re-resolution" path).
- Loop cadence: per-cycle code-graph refresh (cartographer `graph/` snapshot+diff, C2) and per-cycle 90-day web refresh (trend-researcher `research/weave.trends.md`, W1/W3) are owned by the plan-loop, with the source ledger (`weave.trends.md` §Sources) as the invalidation surface (carried-in-window vs flagged-older labels, e.g. serde/ed25519-dalek "older release, still current because unsuperseded" at `weave.trends.md:146-148`).

**CLAIM S4 — the daemon heartbeat is a RUNTIME staleness model, the runtime analogue of research-staleness invalidation; it is N/A as code/web auto-research. [CONFIRMED]**
- "Constant runtime auto-research" for a transport binary maps onto weave's **presence/liveness** subsystem, not onto evidence research. `weave-core/src/store.rs:41,45` define `ONLINE_TTL_SECS = 900` and `PRESENCE_TTL_SECS = 30`; `weave-core/src/model.rs:1684-1697` classifies `Live` (fresh heartbeat ≤30s), `Likely` (no heartbeat but `last_seen` within 900s TTL), `Offline`. The daemon "writes a heartbeat every 15 s" (`weave/src/main.rs:1082`).
- The staleness verdict is computed by `peer_stale_reason` (`weave/src/main.rs:3324-3341`): when `!is_online_at(p.last_seen, now)` it returns `"heartbeat_stale"` — i.e. peer-state evidence past its TTL is **invalidated** and the peer is reported stale (`is_online_at` = `now - last_seen <= 900`, `store.rs:1059-1060`).
- **Relevance:** this is the same SHAPE as auto-research staleness (TTL → invalidate → re-fetch), applied to live transport facts rather than to web/code evidence. It is a useful in-repo *precedent* for a research-staleness TTL, but it does **NOT** refresh code or web research, so for the autoresearch axis it is **N/A — runtime peer-liveness cadence, not code/web evidence refresh** (it neither re-indexes the graph nor re-pulls advisories/releases).

## 4. Upgrade rows (axis: autoresearch)

| id | upgrade | evidence | acceptance | risk | reversibility |
|---|---|---|---|---|---|
| U1 | Add repo-native code-intelligence freshness signal: a CI step (or pre-commit) that runs `git-kb code` over the 4 crates and fails if the committed graph snapshot is stale vs HEAD (mirrors the harness cartographer, but in-repo). | C1 (no `.kb/`, no CI re-index), C2 (graph is harness-side only) | CI job exists; editing a public symbol without refreshing the snapshot fails the check; default build untouched. | low — additive CI only; no trust-boundary code; git-kb has no C dep. | high — delete the job + snapshot. |
| U2 | Close the *new-release* recency blind spot: add a scheduled `renovate.json` (or `dependabot.yml`) so crate updates (e.g. the owed `rusqlite 0.40.0→0.40.1`, `weave.trends.md:144`) are surfaced by a bot, not only by the 90-day human trend pass. | W1 (no bot), W2 (gate is advisory-only, blind to non-advisory releases), `weave.trends.md` §D currency table | a scheduled PR-raising bot config lands; a stale pin produces an automated PR; CI `audit` still gates merges. | low–med — bot PRs add noise; backends are mutually-exclusive features so test matrix must stay green (`ci.yml` libsql/sign columns already cover this). | high — remove config file. |
| U3 | Document the daemon-heartbeat TTL as the canonical staleness-invalidation precedent and (optionally) add an explicit TTL to harness research artifacts (`weave.trends.md` recency window) so stale evidence auto-flags the same way `heartbeat_stale` does. | S2 (advisory self-invalidation), S4 (runtime TTL model: `store.rs:41,45`, `main.rs:3337`) | a stated TTL on the trends note; an out-of-window source is flagged exactly as `peer_stale_reason` flags `heartbeat_stale`. | low — docs/convention only. | high — revert doc. |

## 5. Gate handoff — tests proving stale-evidence checks fail closed

These EXIST in-repo and are the fail-closed proof for the advisory axis (the autoresearch gate weave already enforces):

- **`scripts/supply_chain_audit.py --self-test`** (`self_test`, lines 149-157): asserts the RUSTSEC-id parser, the exact `EXPECTED_IGNORES` set, and that the removed bincode id parses — a unit guard that the ignore-set logic itself can't silently break.
- **CI negative test (documented invariant)** — `ci.yml:175` / `deny.toml:18-19`: "any advisory NOT in that explicit list fails this job — proven by the negative test (dropping an id → `error[vulnerability]`, exit 1)." Dropping any of the 5 ids from deny.toml makes `cargo deny check advisories` fail; adding an un-listed advisory likewise fails. Both directions fail closed.
- **`check_libsql_tree_tracks_tls`** (lines 92-117) is itself the stale-evidence test: it FAILS when the ignored advisories become stale (rustls-webpki gone from the libsql graph), forcing their removal — i.e. "missing stale-evidence check" cannot pass silently.
- **`check_default_tree_clean`** (lines 79-89): FAILS if rustls-webpki ever leaks into the default graph — proving the advisory budget stays confined.

**Gap for the loop to add (RED handoff):** there is currently **no** test that the *code graph snapshot* is fresh (U1) and **no** test that crate pins are not stale beyond the recency window (U2). A RED test for U1 would assert `git-kb code` over `weave-core/weave-inject/weave-mcp/weave` produces a symbol/edge count matching the committed snapshot (fail on drift); a RED test for U2 would assert no direct dependency is >1 minor behind its crates.io latest without a documented waiver. Both are additive and read-only on production code.

---

### Required markers (gate)
- **code auto-research** / **git-kb**: §1 — weave has no repo-native git-kb index; code auto-research is the harness cartographer's per-cycle `git-kb code` snapshot/diff (C1, C2).
- **web auto-research** / **90-day** / **recency**: §2 — no dependabot/renovate; web auto-research = deny.toml + supply_chain_audit.py + CI audit (event-driven) plus the harness 90-day recency window in `research/weave.trends.md` (W1–W3).
- **stale** / **invalidate**: §3 — the libsql-pinned-rustls block self-invalidates via `check_libsql_tree_tracks_tls` (S2); the daemon `heartbeat_stale` TTL invalidation is the runtime analogue (S4).

### Confidence
HIGH on the advisory gate, deny.toml mechanics, daemon TTL (all from cited repo source). HIGH on the absence findings (.kb/, dependabot/renovate, CI re-index — exhaustive ls/grep). MEDIUM on the U2 release-recency blind-spot framing (depends on owner appetite for bot noise vs the existing human 90-day pass).
