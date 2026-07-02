# ROADMAP rows — grit (cycle 5)

Canonical plan copy: `reports/grit-plan.md`. These rows are the promotable `docs/ROADMAP.md` entries
(written here in the plan dir only — never into grit's tree, owner-wall). One feature/upgrade row per
sequenced item; one Feature-Forge **test-build** row shaped to FF `feature-architect` `## Verification
plan` intake.

## Upgrade rows (target: grit)

| id | target | item | axis | tier | blast (graph) | status | source |
|---|---|---|---|---|---|---|---|
| grit-0a | grit | Remove/relocate stray `.worktrees/Cargo.toml` (or empty `[workspace]` in grit manifest) | quality | APPLY | none (build unblock) | ready | verdicts claim-7 |
| grit-0b | grit | Stable version-independent `Symbol.hash` (fixed pure-Rust algo) | accuracy | APPLY | low (parser, 1 fn) | ready | architecture U-hash |
| grit-1 | grit | `grit reconcile {a,b}` union-step-2 capability (`--lock-conflicts`) | accuracy | PROPOSE/FF | contained (additive) | RED authored | test-strategy FF spec |
| grit-2 | grit | Atomic multi-symbol claim (release granted on terminal bail) | accuracy | APPLY | contained | ready | verdicts claim-4b |
| grit-3 | grit | `enum Backend` deny-unknown (kill `_ => SQLite` silent downgrade) | accuracy | PROPOSE | medium (config+resolve) | ready | governance gov-008 |
| grit-4 | grit | Route lock-availability reads through active LockStore (cloud skew) | accuracy | PROPOSE | medium-high (central reads) | ready | verdicts claim-4a |
| grit-5 | grit | Disambiguate symbol ids (kind/positional discriminator) | accuracy | ADR | highest (system PK + migration) | behind grit-1 | architecture U-id |
| grit-6 | grit | Caller-identity binding on lock ownership | governance | ADR | high (trait + 3 backends) | behind ADR | prompt-arch PA-U4 |
| grit-7 | grit | Parse-each-file-once + single tree glob | speed | APPLY | low-medium (init) | ready | architecture U-speed |
| grit-8 | grit | Call-edge scope resolution (stop `--with-deps` over-locking) | accuracy | PROPOSE | medium (deps) | ready | architecture U-callscope |
| grit-9 | grit | Retire socket Room → `--poll`, or real `grit serve` daemon | quality | PROPOSE | low (room/watch) | ready | verdicts claim-4c |
| grit-10 | grit | grit→weave A2A room-event bridge | quality | PROPOSE | low (additive, fail-open) | ready | rules-policy rpo-A |
| grit-11 | grit | `--json` output mode / MCP machine surface | accuracy | ADR | medium (all verb prints) | behind ADR | prompt-arch PA-U2 |
| grit-12 | grit | Remove dead code (ChildKind; audit get_deps/count_deps) | quality | APPLY | low | ready | verdicts remove-dead-code |
| grit-gov-1 | grit | AGENTS.md hard-rules + `.claude/rules/destructive-commands.md` | governance | PROPOSE | docs/rules | ready | governance gov-001/011 |
| grit-gov-2 | grit | Trim RTK noise + dedup ICM in CLAUDE.md | quality | APPLY | docs | ready | governance gov-002 |
| grit-gov-3 | grit | MSRV (`rust-version`) + `rust-toolchain.toml` pin (≥1.96.0) | accuracy | PROPOSE | build/CI | ready | governance gov-005; trends C13 |
| grit-gov-4 | grit | clippy `--all-targets` + `cargo audit`/`deny` job | accuracy | PROPOSE | CI | ready | governance gov-006/007 |
| grit-gov-5 | grit | Azure key via env / 0600 config perms (secret residency) | governance | SUPERVISED | config save/load | owner-gated | governance gov-009; FL-5 |
| grit-gov-6 | grit | Parameterize release workflows to `${{ github.repository }}` | governance | PROPOSE | CI/release | ready | governance gov-010 |
| grit-cur-1 | grit | Migrate `azure_storage_blobs` 0.21 → `azure_storage_blob` 1.0 + `azure_core` 1.x | accuracy | PROPOSE | cloud backend | ready | trends C10 (GA 2026-05-14) |
| grit-cur-2 | grit | Stage `rusqlite` 0.31 → 0.40; `tree-sitter` 0.25 → 0.26 (+grammars) | quality | PROPOSE | db/parser | ready | trends C9/C11 |
| grit-dc-1 | grit | `openssl-sys`→`rustls` to unlock ARM64 release target | quality | PROPOSE | release matrix | ready | distributed-compute U1 |
| grit-fl-1 | grit | Ignore/remove un-owned root `.worktrees/`; drop stale `.fastembed_cache/` rule; move `tests/*.sh` → `scripts/test/` | quality | APPLY/PROPOSE | layout | ready | filesystem FL-1/2/3 |

## Feature-Forge test-build row (generate + run handoff)

| id | target | test-build | intake shape (FF feature-architect `## Verification plan`) | RED state | source |
|---|---|---|---|---|---|
| grit-ff-1 | grit | Implement `grit reconcile` to GREEN the authored union-step-2 contract | see Verification plan below | 3 RED (unrecognized `reconcile`) | test-strategy FF spec |

### Verification plan (FF intake for grit-ff-1)

- **Test surface (authored, additive):** `tests/union_dedup_contract.rs`, binary-driven via
  `CARGO_BIN_EXE_grit`. Keep additive; do not weaken assertions. 3 tests, currently RED for the right
  reason (`error: unrecognized subcommand 'reconcile'`).
- **Production work to GREEN (engine-first, then thin CLI):**
  1. Engine: a reconcile function over two repo roots — `SymbolIndex::scan_all` each (parser/mod.rs:250),
     join by `id`, partition by `Symbol.hash` (parser/mod.rs:15) into `identical`(auto-merge) /
     `conflicting`(same id, differing hash) / `only_in_a` / `only_in_b`.
  2. CLI: `Reconcile { a, b, lock_conflicts: bool }` variant on `Command` (cli/mod.rs:31-175) + dispatch
     arm in `run` + `cmd_reconcile` printing the partition (conflicts labelled `conflict`) and divergent
     ids in `<file>::<name>` form.
  3. `--lock-conflicts`: route each conflicting symbol through `LockStore::try_lock` (lock_store.rs:29).
- **GREEN acceptance (1:1 with the 3 RED tests):** `grit reconcile --help` exits 0; `grit reconcile
  <A> <B>` exits 0 with stdout containing `parse`/`dedupe` AND `checksum`+`conflict`; `grit reconcile
  --lock-conflicts <A> <B>` exits 0 with stdout containing `core.rs::checksum`.
- **Prerequisites:** grit-0a (phantom-workspace remediation, so `cargo test` builds standalone) and
  grit-0b (stable hash, so the partition key is toolchain-stable).
- **Golden fixtures:** promote the in-test crate fixtures to `tests/fixtures/union/{crate_a,crate_b}/src/core.rs`
  (9 byte-identical helpers + 1 divergent `checksum`) and snapshot reconcile stdout.
- **Routing note:** GREEN proves Route B (grit-inline, OUT of the no-C trust boundary). A pure-Rust
  in-boundary reconciler (Route A) is the ADR-gated alternative (`reports/adr-draft-grit-reconciler.md`).
- **CI gate(s):** `cargo test` (new integration target compiles+runs); fmt/clippy unaffected.
