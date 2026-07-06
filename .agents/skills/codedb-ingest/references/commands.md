# CodeDB and Nu Plugin Commands

Use these commands from the target repo or from the CodeDB repo at `/home/flexnetos/FlexNetOS/src/nu_plugin`.

## Build Runtime Tools

Prefer the packaged Nix runtime when available:

```bash
CODEDB_REPO=/home/flexnetos/FlexNetOS/src/nu_plugin
CODEDB_TOOLS="$(nix build --accept-flake-config --no-write-lock-file --no-link --print-out-paths "$CODEDB_REPO#codedb_runtime_tools")"
CODEDB_BIN="$CODEDB_TOOLS/bin/codedb"
CODEDB_PLUGIN_BIN="$CODEDB_TOOLS/bin/nu_plugin_codedb"
```

If working inside the CodeDB repo with Cargo available:

```bash
cargo build --quiet -p codedb -p nu_plugin_codedb
CODEDB_BIN="$CODEDB_REPO/target/debug/codedb"
CODEDB_PLUGIN_BIN="$CODEDB_REPO/target/debug/nu_plugin_codedb"
```

## Doctor and Smoke

CLI:

```bash
"$CODEDB_BIN" doctor --nu --format json
"$CODEDB_BIN" doctor --yazelix --format json
"$CODEDB_BIN" tables --format json
```

Transient Nu plugin smoke, matching the no-registry-mutation test pattern:

```bash
TEMP_HOME="$(mktemp -d)"
TEMP_PLUGIN_CONFIG="$TEMP_HOME/plugins.msgpackz"
HOME="$TEMP_HOME" \
XDG_CONFIG_HOME="$TEMP_HOME/.config" \
XDG_DATA_HOME="$TEMP_HOME/.local/share" \
XDG_CACHE_HOME="$TEMP_HOME/.cache" \
nu --no-config-file \
  --plugin-config "$TEMP_PLUGIN_CONFIG" \
  --plugins "$CODEDB_PLUGIN_BIN" \
  -c 'codedb tables | to json'
```

Use the same temporary home/plugin config for later `nu` commands in the same run. This mirrors the Yazelix smoke tests and avoids writing a real Nushell plugin registry.

## Repository Semantic Tables

Run these against a target repo. The plugin commands return native Nushell rows; append `| to json` or `| save -f <path>` when an artifact is needed.

```bash
TARGET_REPO=/path/to/repo
TEMP_HOME="$(mktemp -d)"
TEMP_PLUGIN_CONFIG="$TEMP_HOME/plugins.msgpackz"
NU_CODEDB=(
  env
  HOME="$TEMP_HOME"
  XDG_CONFIG_HOME="$TEMP_HOME/.config"
  XDG_DATA_HOME="$TEMP_HOME/.local/share"
  XDG_CACHE_HOME="$TEMP_HOME/.cache"
  nu
  --no-config-file
  --plugin-config "$TEMP_PLUGIN_CONFIG"
  --plugins "$CODEDB_PLUGIN_BIN"
)

"${NU_CODEDB[@]}" -c "codedb scan '$TARGET_REPO' | to json"
"${NU_CODEDB[@]}" -c "codedb fs entries --repo '$TARGET_REPO' | to json"
"${NU_CODEDB[@]}" -c "codedb source files --repo '$TARGET_REPO' | to json"
"${NU_CODEDB[@]}" -c "codedb cargo packages --repo '$TARGET_REPO' | to json"
"${NU_CODEDB[@]}" -c "codedb cargo deps --repo '$TARGET_REPO' | to json"
"${NU_CODEDB[@]}" -c "codedb cargo sources --repo '$TARGET_REPO' | to json"
"${NU_CODEDB[@]}" -c "codedb rust items --repo '$TARGET_REPO' | to json"
"${NU_CODEDB[@]}" -c "codedb rust macros --repo '$TARGET_REPO' | to json"
"${NU_CODEDB[@]}" -c "codedb rust cfg --repo '$TARGET_REPO' | to json"
"${NU_CODEDB[@]}" -c "codedb build scripts --repo '$TARGET_REPO' | to json"
"${NU_CODEDB[@]}" -c "codedb gaps | to json"
"${NU_CODEDB[@]}" -c "codedb validation errors | to json"
"${NU_CODEDB[@]}" -c "codedb schema | to json"
```

The packaged runtime may not include `cargo` on `PATH`. If semantic commands fail with cargo metadata errors, run from a Rust/Cargo development environment and record that limitation as a capture gap in the summary.

## Config and Runtime File Import

After creating an inventory artifact as described in `inventory-contract.md`:

```bash
INVENTORY_JSON=/path/to/codedb-inventory.json
nu --no-config-file \
  --plugin-config "$(mktemp -d)/plugins.msgpackz" \
  --plugins "$CODEDB_PLUGIN_BIN" \
  -c "codedb envctl import inventory '$INVENTORY_JSON' | to json"
```

The import command returns `envctl_yazelix_file_import` rows. Content rows include `content_hash`, `blob_ref`, `structured_status`, `structured_row_count`, and nested `structured_rows` when the parser hint is supported.

## Yazelix Generated Bridge

Yazelix integration should be generated state, not a tracked config edit:

```bash
GENERATED_DIR="$HOME/.local/share/yazelix/initializers/nushell"
"$CODEDB_BIN" generate-yazelix-bridge --out-dir "$GENERATED_DIR" --format json
```

The bridge path is useful for confirming CodeDB CLI/plugin availability in Yazelix-like launches. Do not register the plugin into the real user Nushell plugin registry unless an explicit runtime policy says to do so.
