# verdicts — agentic-os-blueprint (2026-07-09)

Verdict ledger for the blueprint-alignment plan. Evidence chain: (1) 6-auditor layer audit +
2-verifier adversarial pass (workflow `wf_97a0b5a7-fb9`, 8 agents / 299 tool calls), then
(2) a full **manual runtime re-verification** by the orchestrator (no delegation) — live `psql`,
`bun` drives of the var-runtime layer, execution of built binaries (`gguf-proof`, `envctl`,
router, embedder), `ldd`/`xxd`/`strings`/`file(1)`, toolchain `rustlib` listings. Runtime
observation outranks grep; every row cites its capture.

Legend: CONFIRMED = held under adversarial + runtime check · QUALIFIED(cond) = true with the
stated bound · REFUTED = disproven (recorded for the gaps section, never re-admitted).

| id | claim | verdict | evidence (capture / path:line) |
|----|-------|---------|--------------------------------|
| V1 | PostgreSQL 17.10 live, socket-only, ruvector ext active | CONFIRMED | `ps`: `pg17-rw/bin/postgres -D var/lib/postgresql/17 -p 5432 -k <socket>`; `pg_extension` → `ruvector 0.3.0` in `ruvector` + `ruvector_full`; `ruvector_full` has **zero tables** |
| V2 | `codebase` table = one merge corpus, file-granular, fully semantic-embedded | CONFIRMED | psql: counts `5157\|5157\|0` (total\|semantic\|minilm); `block_type=file` 5157/5157; origins A=462/B=4616/resolved=79; prefixes match `src/rusty-idd-unified` (AI_MERGE dir present) |
| V3 | Vector flush to Postgres proven but minimal | CONFIRMED | `episodes(embedding ruvector(1536))` + `episodes_hnsw (ruvector_cosine_ops)`; rows: exactly `1\|researcher`; `var/lib/ruvector/agents/_manifest.json` documents the flush contract |
| V4 | Blueprint MinCut immune system EXISTS on-box; native layer correct; wrapper broken | CONFIRMED | `var/lib/ruvector/swarm-immune.mjs` ("sever a hallucinating/failing agent") over `mincut/ruvector_mincut.node`. Drives: string ids → `napi … NumberExpected`; numeric ids → `minCut() {"value":1,"isExact":true}`, `cutEdges() [{source:0,target:3,weight:1}]` (correct weighted cut); weights honored (0.5-edge→0.5; w=7 triangle→14). 3 wrapper bugs: string ids, `cutEdges`/`isConnected` accessed as properties |
| V5 | Frozen-console + MicroLoRA hot-swap runs live (<1ms) | CONFIRMED | Ran `var/lib/ruvector/gguf-proof/target/release/gguf-proof`: `CONSOLE loaded once: 0.69s (frozen 1.1B Q4)`; `swap+apply 0.013ms / 0.023ms (<1ms claim)`; 2 agents generated; `CYCLE COMPLETE: 1 frozen model, 2 agents, per-request cartridge swap` |
| V6 | (self-audit) "gguf-proof model path no longer exists" | REFUTED | `ls -la ~/.ruvllm/models/` (untruncated): `tinyllama-1.1b-chat-v1.0.Q4_K_M.gguf` present (06:02) beside 3 RuvLTRA GGUFs; `strings` on binary matches source path; src/binary mtimes 07:38:01/07:38:02 |
| V7 | RuvLTRA complexity tier runs live but is non-discriminating | QUALIFIED(calibration) | Worktree `meta-ruvector-router-wt` @ 79c2f91f: BFT-consensus → `opus (0.551)` AND "fix typo in README" → `opus (0.563)`, `backend:"ruvltra-fastgrnn"` — commit's "fix typo → haiku PROVEN" **not reproduced**. Main-checkout router tier-less; its JSON = the live UserPromptSubmit `[INFO] Routing task` banner |
| V8 | Local MiniLM embedder works, discriminates, is unwired | CONFIRMED | Drove export surface: `DIM: 384 \| related: 0.711 \| unrelated: 0.016`; manifest: `"model":"agentdb fallback embedder (no local model wired yet)"`, dimension_observed 1536; `codebase.embedding_minilm` 0/5157 |
| V9 | Deployed envctl binary is stale (missing `envctl db`) | CONFIRMED | live binary: `error: unrecognized subcommand 'db'`; binary mtime 2026-07-07 06:02 < f6659ba 2026-07-08 (GH#414); source `crates/cli/src/main.rs:269,741` carries the verb; `usr/bin/envctl` symlinks the stale binary |
| V10 | musl static capability absent box-locally | QUALIFIED(fleet-CI) | rustup nightly+stable `rustlib/` gnu-only; fenix sysroot `rustlib/` = `etc x86_64-unknown-linux-gnu`; zero musl in all `.cargo/config.toml`/flakes. Fleet exception: meta-ruvector GitHub CI builds musl napi artifacts (build-gnn.yml:38) |
| V11 | Workspace binaries glibc-dynamic; vendor CLIs static | CONFIRMED | `ldd target/release/envctl` → `/usr/lib/x86_64-linux-gnu/libc.so.6`; installed codex = `static-pie linked` (vendor musl tarball) |
| V12 | Export contract NAMES envctl the materializer; `redb_access=forbidden` is transport, not role | CONFIRMED | `nu_plugin/docs/ENVCTL_EXPORT_CONTRACT.md:12,38,43-55`: "envctl remains the materializer for requested files"; checksum-bound `codedb_materialization_targets` rows |
| V13 | Production SHAKE-256 witness-chain consumer exists (per-event, not per-fact) | CONFIRMED | `xxd src/handoff/.handoff/ledger.db.rvf` → `5346 5652` (**SFVR**); `src/handoff/ledger` v2 "append, replay, witness chain" via rvf-crypto |
| V14 | Live agent containers are SQLite, `.rvf` in name only | CONFIRMED | `file agents/*.rvf.db` → "SQLite 3.x database" ×5 (coordinator/materializer/merge-resolver/researcher/review-gate) |
| V15 | ccboard wired as TUI pane; envctl mission-control.kdl undeployed | CONFIRMED | `yazelix/configs/zellij/layouts/flexnetos_agent_workspace.kdl:39-41` `pane name="ccboard" { command ".../libexec/ccboard" }` (axum web mode dormant); `find usr ~/.config -name mission-control.kdl` → none |
| V16 | bun-rewrite enforcement not live; memory ahead of box | CONFIRMED | `~/.claude/hooks/` lacks bun-rewrite.sh; `git show develop:home/.claude/hooks/bun-rewrite.sh` → path does not exist; only worktree commit 1889fb8; MEMORY.md says "HOOK-ENFORCED" |
| V17 | ATAS/ESN code absent; trajectory substrate exists | CONFIRMED | ESN greps empty workspace+crates (nervous.ts `createReservoir` is an empty stub; tests commented out); `ruvector-sona-0.2.1/src/trajectory.rs` = "Lock-free trajectory buffer … non-blocking trajectory recording during inference" |
| V18 | Routing decision already made by the operator (Law 8 mechanism) | CONFIRMED | laws.md Law 8: "Everything runs on Fable **unless the operator says otherwise**"; router source: "Operator directive 2026-07-09: RuvLTRA is installed and proven"; 3 RuvLTRA GGUFs pulled 03:59–04:00 same day |
| V19 | envctl migration engine live; `envctl db` fail-closed code-graph surface in source | CONFIRMED | `envctl migration --help` (event-sourced redb, hash-chained ledgers, R3 approvals, replay verify); main.rs:741 "Read/plan only in the CLI" |
| V20 | Extension↔client version skew | CONFIRMED | installed ext `0.3.0` vs workspace pin `ruvector-postgres 2.0.5`; `var/lib/ruvector/ext/` ships `ruvector--2.0.0.sql` AND a `2.0.0--0.3.0` downgrade script |
| V21 | Postgres unmanaged; foreign-uid pgdata | CONFIRMED | no systemd unit/cron/manifest component (started ad hoc 09:52); `stat var/lib/ruvector/pgdata` → owner UNKNOWN uid, mode 700 |
| V22 | crates.io discipline holds | QUALIFIED(sona ships napi/wasm sources, feature-gated) | Cargo.lock: ruvector-core 2.2.3 / ruvllm 2.3.0 / ruvector-postgres 2.0.5 / ruvector-sona 0.2.1 / rvf-runtime 0.3.0 all `registry+crates.io`; Cargo.toml:120-122 "registries not repo" |
| V23 | SIMD vector math real in pinned crate; zero first-party callers | CONFIRMED | `ruvector-core-2.2.3/src/simd_intrinsics.rs` AVX-512/AVX2+FMA/NEON dispatch; engine `ruvector` feature default OFF; only comment at `engine/src/layout.rs:196` |
| V24 | COW branching real upstream; merge-half absent; unused on box | QUALIFIED(no merge op) | rvf-runtime cow.rs/`branch()` (store.rs:1884, ADR-031); no `fn merge`; zero call sites in envctl/nu_plugin |
| V25 | Subpolynomial min-cut crate real; consumed only via the local .node addon | CONFIRMED | `ruvector-mincut-2.0.6` implements Jin-Sun-Thorup (SODA 2024); absent from envctl Cargo.lock; native addon at `var/lib/ruvector/mincut/` proven by V4 drives |
| V26 | redb↔postgres BlobStore path exists, unmerged | CONFIRMED | branch `codex-codedb-store-pg` @ 4c2fef4 (`crates/codedb_store_pg`, pluggable BlobStore trait); master redb-only |
| V27 | Deterministic capture→store→export→materialize spine proven E2E | CONFIRMED | `nu_plugin/examples/idd_unify_e2e.nu` + `var/idd-unify/e2e/run-20260707-*`; envctl PR #440 migration_db merged |
| V28 | No HTTP UI in envctl workspace; UDS = secretd gRPC only | CONFIRMED | no axum/actix/warp in envctl Cargo.tomls; `secretd/src/main.rs:640 bind_uds`; GUI = in-process egui. (ccboard's axum lives in the vendored crate, wired TUI-only — V15) |
| V29 | `.rvf`→ruvllm adapter link absent in published crates | CONFIRMED | `rg -in 'rvf' ruvllm-2.3.0/src` → 0 matches; gguf-proof builds adapters in-process |
| V30 | Corrected alignment score basis | CONFIRMED | 45 → **57**/100: Layer-4/5 materially better than first-scored (V4, V5, V7, V13); still missing: ATAS (V17), musl (V10), merged PG path (V26), discriminating router (V7) |

Notable REFUTED overclaims (do not re-admit): first-audit "MinCut severing application absent
everywhere" (V4 refutes); "one-frozen-model is a deviation / ruvllm unused" as a *capability*
statement (V5 refutes; the *wiring* gap stands); "contract forbids Hop-3" (V12 refutes the framing);
"tinyllama missing" (V6, self-inflicted truncation); router commit's "fix typo → haiku PROVEN" (V7
refutes live).
