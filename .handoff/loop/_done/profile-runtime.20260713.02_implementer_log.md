# Archived profile-runtime cycle artifact

This cycle artifact was current before the independent Doctor cycle was integrated.
It is archived intact so the Doctor cycle can occupy the single active cycle slot.

# 02 — Implementer log: envctl profile ownership convergence

Date: 2026-07-13
Implementer: `envctl_profile_implementer`
Verdict: GREEN for assigned focused scope

## Engine boundary classifier

- Added an exact active-profile ownership proof for meta tool PATH entries.
- Accepted frontdoors are limited to lexical
  `~/.nix-profile/{bin,toolbin}/<tool>` and the command path in each current
  profile directory's canonical Nix-store exposure.
- The ultimate binary target alone is deliberately insufficient. A TDD
  regression gives the current and an old foundation generation the same raw
  package target and proves that only the current generation passes.
- User-bin shadows, second-profile paths, old foundation generations, raw
  package paths, stale store paths, missing exposures, and non-store targets
  remain fail-closed.
- Updated boundary model/drift wording without changing the serialized API.

RED evidence:

```text
active_profile_ownership_accepts_only_lexical_and_current_store_frontdoors
FAILED: old-foundation/toolbin/meta was accepted when only ultimate-target
equality was checked
```

GREEN evidence:

```text
cargo test -p envctl-engine active_profile_ownership --locked
3 passed; 0 failed

cargo test -p envctl-engine detect::tests --locked
12 passed; 0 failed

cargo clippy -p envctl-engine --lib --locked -- -D warnings
PASS
```

## Shared substrate worktree setup

- `ci/setup-meta-deps.sh` now recognizes a valid linked worktree whose `.git`
  is a file, preserves its checkout/HEAD, and refuses to replace an occupied
  non-repository sibling path.
- The parent workspace is generated only when absent. An existing parent must
  contain both substrate members and match the equal `loop_lib` /
  `meta_plugin_protocol` versions locked by envctl; incompatible parents are
  refused without modification.
- Added `scripts/tests/test-setup-meta-deps.sh` and wired it into the
  `meta-substrates` gate.

RED evidence: the previous implementation treated the linked-worktree `.git`
file as absent and entered its destructive clone path.

GREEN evidence:

```text
test-setup-meta-deps: PASS
META-SUBSTRATES GATE PASS
```

## Nushell / RTK ownership

- Ported the canonical envctl cleanup exactly: removed the duplicate
  `home/.config/nushell/rtk-wrappers.nu`, made standalone login Nu import the
  profile-owned `rtk_wrappers.nu` once, removed the duplicate Yazelix user-hook
  import, and removed the retired portability-link footprint.
- Added a hermetic behavioral test that proves login Nu routes Cargo through
  the profile module, native `^bash` works, and a Yazelix-managed config plus
  the editable user hook still routes exactly once.
- Wired the test into `yazelix-codex-runtime.sh`.

RED evidence: the new ownership test initially failed on the duplicate envctl
module.

GREEN evidence:

```text
test-nushell-rtk-ownership: PASS
PASS: Yazelix/Codex ownership source gate
ok - active Claude harness owners use the Meta root
```

## Verification discipline

All implementer shell commands were proxied through the explicit current
profile RTK path with the current profile `toolbin` and `bin` prepended to
`PATH`. Full workspace builds/tests were intentionally left to the guardian so
this focused cycle did not contend with the parallel Yazelix/package work.
