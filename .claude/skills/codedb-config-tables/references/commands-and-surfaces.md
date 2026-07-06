# Commands and Runtime Surfaces

Use the CodeDB CLI and Nu plugin as table-producing surfaces, not as plain text commands.

## Primary commands

- `codedb scan <path>`: capture filesystem and crate facts from a path into CodeDB rows.
- `codedb export --format json|nuon|csv`: export captured rows in a structured format.
- `codedb schema`: inspect the supported tables and their purpose.
- `codedb tables`: inspect available table names from the Nu plugin surface.
- `codedb gaps`: inspect capture gaps that must be surfaced instead of silently omitted.
- `codedb validation-errors`: inspect validation failures on captured material.
- `codedb doctor --nu`: inspect host Nushell readiness.
- `codedb doctor --yazelix`: inspect Yazelix runtime Nu readiness.
- `codedb generate-yazelix-bridge --out-dir <dir>`: generate Yazelix-owned Nu bridge artifacts without editing tracked Yazelix config.

## Nu plugin expectations

- Prefer Nu-native table output when `nu_plugin_codedb` is available.
- Treat plugin results as records/lists/tables, not log text that needs re-parsing.
- Use `codedb tables`, `codedb gaps`, and `codedb validation-errors` after imports to prove coverage and quality.

## Yazelix runtime lanes

Use three separate lanes and do not blur them:

1. Transient proof mode:
   - launch `nu` with explicit plugin paths
   - use temporary `HOME`, `XDG_CONFIG_HOME`, `XDG_DATA_HOME`, `XDG_CACHE_HOME`
   - pass isolated `--plugin-config`
   - do not mutate a real user plugin registry
2. User registry mode:
   - `plugin add <path-to-nu_plugin_codedb>`
   - `plugin use codedb`
   - only acceptable when the user explicitly wants registry setup, and still prefer temp HOME during tests
3. Yazelix generated bridge mode:
   - use `codedb generate-yazelix-bridge`
   - wire `YAZELIX_CODEDB_BIN` and `YAZELIX_CODEDB_PLUGIN_BIN`
   - source generated initializer/extern artifacts instead of editing tracked Yazelix config
