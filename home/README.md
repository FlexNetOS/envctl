# `home/` — reviewed non-secret configuration inputs

This tree contains first-party, non-secret configuration inputs that envctl may
project. It is not an installed-runtime root and it does not own agent runtime
configuration.

## Runtime ownership

- `/home/flexnetos/.nix-profile` is the sole installed-runtime and PATH owner.
- `/home/flexnetos/.config/yazelix/` contains editable Yazelix inputs.
- `/home/flexnetos/meta/var/lib/codex` is the Codex state and active-config root.
- `/home/flexnetos/meta/var/lib/claude` is the Claude state and active-config root.
- `/home/flexnetos/meta/var/lib/yazelix/runtime/xdg` is the sole FlexNetOS XDG
  runtime root.
- Agent configuration sources ship in the canonical `FlexNetOS/yazelix`
  profile package. Envctl validates those sources and materializations; it does
  not create a home projection or switch the installed profile.

## Rules

1. Never commit credentials, tokens, keyring material, or runtime sessions.
2. Durable envctl state belongs under `$META_ROOT/var/lib/envctl`; regenerable
   caches belong under `$META_ROOT/var/cache/envctl`; temporary files belong
   under `$META_ROOT/var/tmp` or the process runtime directory.
3. Every projected file has one declared source and a verification gate.
4. Existing foreign files are preserved or archived through the owning
   component before a projection is written.
5. Generated runtime output is proof only. Edit the profile/config source and
   rebuild through the canonical Yazelix repository.

The strict ownership gate is `ci/gates/strict-profile-owner.sh`.
