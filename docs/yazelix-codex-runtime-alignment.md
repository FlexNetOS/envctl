# Yazelix/Codex Runtime Alignment Ledger

Status: strict source contract. The installed-runtime cutover is owned only by
the canonical `FlexNetOS/yazelix` repository after its change is merged and a
clean checkout exactly matches `origin/main`.

## Authority and paths

| role | authority/path |
|---|---|
| Canonical source repository | `git@github.com:FlexNetOS/yazelix.git` |
| Editable Yazelix input | `/home/flexnetos/.config/yazelix/` |
| Sole installed-runtime selector | `/home/flexnetos/.nix-profile` |
| Yazelix frontdoor | `/home/flexnetos/.nix-profile/bin/yzx` |
| Codex frontdoor | `/home/flexnetos/.nix-profile/bin/codex` |
| Claude frontdoor | `/home/flexnetos/.nix-profile/bin/claude` |
| RTK frontdoor | `/home/flexnetos/.nix-profile/bin/rtk` |
| Codex state and active config | `/home/flexnetos/meta/var/lib/codex` |
| Claude state and active config | `/home/flexnetos/meta/var/lib/claude` |
| Yazelix XDG runtime | `/home/flexnetos/meta/var/lib/yazelix/runtime/xdg` |

Raw Nix store targets and generated runtime files are proof, never editable or
parallel ownership surfaces. Change the owning Yazelix source/config input and
rebuild through the profile owner.

## Cutover boundary

Envctl validates profile identity, store identity, config-input presence, and
Yazelix-owned materialization. It must not run `nix build`, mutate a profile
generation, switch the selector, or install an agent CLI. Its install/fix
compatibility phases are read-only validation.

A cutover is authorized only when all of the following are true:

1. The source checkout remote is exactly `git@github.com:FlexNetOS/yazelix.git`.
2. The checkout branch is `main`, its tree is clean, and `HEAD == origin/main`.
3. The strict-profile source and build checks pass from that checkout.
4. The candidate profile output is verified before the selector changes.
5. The resulting profile contains matching `bin` and `toolbin` frontdoors for
   Yazelix, Codex, Claude, and RTK, all resolving into the immutable store.

## Verification

Use `command -v`, `readlink -f`, and version commands through the profile. Run
`ci/gates/strict-profile-owner.sh` to reject maintained references or tracked
home projections that would recreate an alternate runtime owner.
