# Archived profile-runtime cycle artifact

This cycle artifact was current before the independent Doctor cycle was integrated.
It is archived intact so the Doctor cycle can occupy the single active cycle slot.

# 01 — Architect plan: single-profile Yazelix/envctl ownership convergence

Date: 2026-07-13
Verdict: GO
Architect: `profile_ownership_architect`

## Outcome

Repair the one Yazelix-owned Nix profile so it is the complete runtime/toolchain owner, and align envctl diagnostics with that ownership model. This is additive and fail-closed: no user-bin wrappers, second profile, rustup mutation, compiler downgrade, generated-runtime edits, or bypasses.

## Verified failures

- Full envctl musl compilation fails in `ring`/`cc-rs` because only `x86_64-unknown-linux-musl-*` exists; cc-rs requires the conventional `x86_64-linux-musl-*` family.
- The sole profile lacks `cargo-audit` although locked nixpkgs supplies the required 0.22.1.
- `yzx doctor --json` rejects package-owned `yzx-desktop-launch` and `yzx-agent-workspace-launch` helpers and reports unhealthy.
- envctl reports 32 HIGH violations for exact executables exposed by the active Yazelix profile.
- `ci/setup-meta-deps.sh` assumes `.git` is a directory and therefore mishandles linked worktrees.
- CI's nominal MSRV lane accepts newer compilers and does not prove Rust 1.89 compatibility.
- envctl carries a duplicate Nu RTK wrapper instead of sourcing the profile-owned Yazelix Nu module.

## Units

1. Add exact active-profile target classification in `crates/engine/src/detect.rs`; accept only the active profile's lexical paths or store paths whose canonical target exactly equals its exposed command. Continue rejecting user-bin shadows, second profiles, stale store targets, and unverifiable paths.
2. Add RED/GREEN engine tests for profile/store acceptance and every refusal path; update model/drift wording without changing serialized API.
3. Make `ci/setup-meta-deps.sh` Git-aware for linked worktrees and validate/create the minimal sibling workspace at the locked 0.2.25 version. Add a hermetic regression and wire it into `meta-substrates.sh`.
4. Delete envctl's duplicate `home/.config/nushell/rtk-wrappers.nu`; source `~/.nix-profile/nushell/config/rtk_wrappers.nu` from the managed Nu config exactly once and add clean-login/managed-Nu behavior tests.
5. Add `pkgs.cargo-audit` to the Yazelix foundation and both command export inventories.
6. Wrap the existing musl package additively with `x86_64-linux-musl-{gcc,g++,ar,ranlib}` aliases while preserving `x86_64-unknown-linux-musl-*`.
7. Extend package release contracts to execute cargo-audit, compile C/C++, exercise ar/ranlib, and perform a real Cargo-level static Rust build.
8. Strictly validate the two package-owned desktop helpers and governed desktop entries in Yazelix ownership diagnostics.
9. Add an exact Nix-pinned Rust 1.89 compatibility command/lane while retaining latest nightly as the default developer compiler.
10. Reconcile active MSRV/toolchain/ownership instructions and regenerate agent-env projections through `envctl agent sync --apply`.

## Guards

- Build/check the Yazelix package before changing the active profile.
- Upgrade only the existing `lifeos_foundation_yzx` profile element; require exactly one element before and after.
- Never rewrite an unrelated parent Cargo workspace or replace an already valid sibling checkout.
- No `Cargo.lock` or `flake.lock` dependency downgrade is expected.
- No public Engine/CLI/GUI API delta is needed for this cycle.

## Acceptance

- `envctl auto-detect --json` has zero boundary violations for exact active-profile targets while fixtures for user-bin/stale-store/second-profile paths remain HIGH.
- `cargo-audit` and both musl naming families resolve from the one profile.
- `cargo build --release --target x86_64-unknown-linux-musl -p envctl --locked` succeeds with no compiler environment overrides; the binary is static.
- Exact Rust 1.89 checks the workspace while nightly remains the default developer lane.
- `yzx doctor --json` reports healthy.
- `envctl agent lock --check`, all workspace tests, clippy, fmt, every CI gate, no-c, and cargo audit pass.
- The active profile element count remains exactly one.
