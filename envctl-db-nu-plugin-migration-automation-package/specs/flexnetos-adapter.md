# FlexNetOS package adapter

The existing `codex-flexnetos-migration-prompt-package` is bundled under `source/`. Codex must use it as the first external package fixture.

## Adapter duties

- inspect package manifest
- validate required files
- compute package hash
- import artifact contract
- import expected output tree
- create recipe phases from master prompt
- execute/import package outputs as operations
- register artifacts/evidence/link graph
- expose results to `nu_plugin`

## Anti-pattern

Do not implement `FlexNetOS` as a special case in core database tables or plugin commands. Use a target descriptor.
