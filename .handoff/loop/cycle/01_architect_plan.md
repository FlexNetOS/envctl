# Feature Forge architect plan — full verification repair

VERDICT: GO

All eight triggering claims hold at `0ff8229`; manifest drift also contains a source-policy defect that must be fixed before relocking. No human design decision remains. Work is ordered A2: Yazelix must own GitNexus through its active profile before envctl may archive the real-home shadow. No PR may merge until both repo guardians pass.

## Target repos

| Repo | Independent lanes | Dependency |
|---|---:|---|
| envctl | 5 | offline agent lock; weave harness; profile/real-home boundary; toolchain/verifier provisioning; reviewed manifest lock |
| yazelix | 1 | package and expose GitNexus 1.6.9 before envctl shadow cleanup |

## Engine/API and invariants

- Preserve `Engine::agent_lock(AgentLockSpec, &EventSink) -> AgentLockOutcome`, `Event::AgentLockChecked`, and existing JSON/exit behavior. `check + Locked` must perform a real zero-network audit, never `prev.clone()` self-diff.
- Preserve `EnvReport`/`MetaBoundaryReport` schemas; add internal active-profile provenance so only exact current-generation profile `bin`/`toolbin` targets are accepted. Arbitrary or stale `/nix/store` paths remain foreign.
- Keep agent SHA-256 `agent-env.lock` separate from FNV-1a `manifest/envctl.lock`.
- No Rust dependency/TLS graph change; external GitNexus/cargo-audit remain outside the product link graph. Engine stays sync/non-printing. Existing CLI/GUI parity remains through the same Engine APIs.
- Preserve Yazelix's real-home `.local/state/nix/profiles` chain. Unknown user-bin entries are inventoried and refused, never deleted. Known shadows are archived only after an exact healthy replacement exists.

## Ordered units

### Lane A — agent lock, failing first

1. Add local content/config add/remove/missing-source plus command/MCP drift tests and a network-fetch sentinel. Replace the existing test that explicitly expects self-diff after removing the source.
2. Add `envctl-agent-env` zero-network audit logic. Rehash local sources without network; for remote sources require the existing pinned lock to satisfy configured identities/selectors. Missing pins/assets fail closed.
3. Complete lock comparison for skills and non-skill assets: additions/removals, hashes, revisions, source identity, destination, scope, kind/name.
4. Wire `Engine::agent_lock` locked check to the audit. Preserve no-write behavior, typed drift, and nonzero error when zero-network satisfaction is impossible.
5. Keep `ci/gates/agent-env.sh` on locked semantics and add a drift counterexample proving the gate fails.

### Lane B — Yazelix GitNexus owner, then envctl cleanup

6. In Yazelix package GitNexus 1.6.9 through the existing release/runtime package pattern. Expose it in active runtime `toolbin`, profile `bin`, public outputs, and release checks. Exact `gitnexus --version` must pass with no real-home user-bin frontdoor.
7. Only after step 6 is guardian-green, redesign envctl's real-home audit and `home-local-single-link`: never create/relink whole `~/.local`; prove the profile chain; dynamically inventory every `~/.local/bin` entry; refuse unknowns; archive GitNexus only after the exact profile or owned `$META_ROOT/usr/bin` replacement passes.
8. Rewrite `test-meta-local-path-audit.sh` to preserve `.local` inode/type and the Nix profile through default/apply modes, reject malicious/broken profile targets, block missing replacements, test known GitNexus cleanup and unknown-entry refusal. Make `meta-local-policy.sh` execute/enforce this contract instead of exempting contradictory code.

### Lane C — profile provenance and envctl frontdoor

9. Add `ActiveProfileProvenance` helpers in `detect.rs`: derive current-generation exact command targets from `ENVCTL_REAL_HOME`/`HOME` only after `~/.nix-profile` resolves through `.local/state/nix/profiles/profile`.
10. Teach `meta_boundary_report_for` to accept exact current-profile paths/direct store targets and reject arbitrary same-named store binaries, stale generations, broken profiles, foreign files, and meta frontdoor symlinks. Include profile-owned `rtk`, `git-kb`, and `gitnexus` where relevant.
11. Add an envctl CLI owner component: stage the release binary privately, atomically install a regular marked `$META_ROOT/usr/bin/envctl` wrapper, verify/idempotently fix it, and remove only owned artifacts. Add lifecycle tests. Do not route boundary findings to retired no-op `meta-tool-links`.

### Lane D — harness, toolchains, and verifier provisioning

12. Validate explicit `WEAVE_BIN` as executable before lookup/attach. Tests set an invalid override and require exit 10/`NEEDS-HUMAN` independent of live host weave. Synchronize authored/ejected dispatcher and test copies; keep harness gate wiring.
13. Add a pinned cargo-audit 0.22.1 component using meta-owned `CARGO_HOME`, a regular `$META_ROOT/usr/bin/cargo-audit` wrapper, and full detect/install/verify/fix/remove/idempotence tests. Keep hosted CI installation and product graph unchanged.
14. Upgrade `rustup` component to provision exact 1.89.0 alongside nightly while nightly stays default. Restore exact `cargo +1.89.0 check --workspace --locked` CI proof and add a toolchain-contract gate/test that rejects minimum-version-only nightly checks.
15. Update AGENTS.md, CLAUDE.md, README and MSRV/ops/audit docs to one contract: exact stable 1.89 MSRV lane, latest nightly default development lane.

### Lane E — manifest correctness and lock

16. Convert every postgres-ruvector path/description from `/home/flexnetos/lifeos/var/...` to `${META_ROOT:?}/var/...`; preserve socket-only startup and stop-without-delete removal. Add a fake `psql`/`pg_ctl` fixture proving META_ROOT relativity, idempotence, flags, and no deletion.
17. Review the intentional `codex-global-baseline` PR #481 delta and every new/changed component before relocking.
18. Mechanically regenerate `manifest/envctl.lock` once. Expected delta is limited to reviewed codex/postgres/base/portability plus new envctl/cargo-audit components. `Cargo.lock` remains unchanged.
19. Add `manifest-lock.sh`, workflow wiring and a hermetic test covering changed/added/removed components. Immediate `envctl lock --check --json` must be clean.

## Required focused tests

- `cargo test -p envctl-agent-env`
- `cargo test -p envctl-engine agent_lock`
- `cargo test -p envctl-engine meta_boundary`
- `cargo test -p envctl --test agent`
- `bash scripts/tests/test-meta-local-path-audit.sh`
- `bash scripts/tests/test-plan-weave-dispatch.sh`
- new cargo-audit/envctl-frontdoor/toolchain/postgres/manifest-lock fixtures
- Yazelix GitNexus package/runtime release checks

## Guardian matrix

- exact `cargo +1.89.0 check --workspace --locked`
- `cargo fmt --all -- --check`
- `cargo clippy --workspace -- -D warnings`
- `cargo test --workspace --features envctl-secrets-engine/low-cost-kdf-tests`
- all repository CI gates, including no-c/meta-substrates/shape/agent-env/meta-local/yazelix-runtime/cargo-audit/harness plus new toolchain/manifest-lock gates
- Yazelix build/checks and cross-repo all-green barrier

## Runtime surfaces

1. Mutate a temporary local agent source; `agent lock --check --locked --json` must exit nonzero, report drift, make no fetch/write.
2. Invalid explicit `WEAVE_BIN` must return controlled `NEEDS-HUMAN` without attach.
3. New Yazelix generation must expose profile-owned GitNexus 1.6.9.
4. Portability preview/apply must preserve real-home `.local`/profile and archive only the proven GitNexus shadow.
5. `envctl doctor --json` must accept current profile tools, reject negative fixtures, and cease reporting envctl's frontdoor symlink after owner-component apply.
6. Cargo-audit component preview/apply then exact version and audit gate.
7. Exact 1.89 check passes while `rustup default` remains nightly.
8. Live postgres detect/verify only; mutating lifecycle is exercised on fixtures unless separately authorized.
9. `envctl lock --check --json` exits zero with empty drift.

## Open questions

None.
