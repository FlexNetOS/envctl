# Yazelix Proof Patterns

Use these patterns when the request mentions Yazelix, generated runtime state, or Nu plugin loading behavior.

## Generated bridge contract

`codedb generate-yazelix-bridge --out-dir <dir>` writes:

- `codedb_init.nu`
- `codedb_extern.nu`
- `codedb_bridge_manifest.json`

It does not:

- edit tracked Yazelix `config.nu`
- run `plugin add`
- take over ownership of Yazelix startup files

Treat generated files as bridge artifacts, not as source-of-truth configuration.

## Ready-state smoke pattern

The existing Yazelix-enabled smoke path shows the intended proof lane:

1. build `codedb` and `nu_plugin_codedb`
2. create temporary `HOME` and XDG roots
3. create a Yazelix-like `~/.local/share/yazelix/initializers/nushell/` directory
4. generate the bridge into that directory
5. launch `nu --no-config-file` against a tiny probe script
6. provide:
   - `IN_YAZELIX_SHELL=1`
   - `YAZELIX_RUNTIME_DIR=<temp-runtime>`
   - `YAZELIX_CODEDB_BIN=<codedb-bin>`
   - `YAZELIX_CODEDB_PLUGIN_BIN=<plugin-bin>`
7. prove:
   - bridge mode is generated state
   - CLI/plugin status are `available`
   - no real plugin registry was created

## File-import rows to look for

The Nu plugin exposes Yazelix-oriented file import rows including:

- `envctl_yazelix_file_import`
- `envctl_yazelix_file_structured_rows`

These rows carry:

- owner and source-of-truth class
- file kind and parser hint
- content hash and byte length
- blob reference
- import safety and reproduction policy
- structured-row readiness and row counts

Use these rows when the user wants runtime-owned config/settings/env/files represented as database tables with both semantics and exact-byte provenance.
