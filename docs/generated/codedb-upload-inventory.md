# CodeDB Upload Inventory

Date: 2026-07-02
Source manifest: `/home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json`

## What this inventory means

The `codedb_file_imports` ingestion path in envctl reads a generated file-target
manifest and turns each row into `envctl_yazelix_file_import` records. The live
source of truth for "what files must be uploaded to the envctl database" is the
Yazelix-generated manifest above.

The canonical exact list is therefore already present in:

- `/home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json`

Use the manifest directly when you need every row exactly as CodeDB ingests it.
Use the filtered `content_blob` subset when you need only paths whose bytes are
uploaded as content blobs.

This repo also carries generated plain-text path lists produced by
`scripts/export-codedb-upload-lists.sh`:

- `docs/generated/codedb-import-targets.txt`
- `docs/generated/codedb-content-blob-targets.txt`
- `docs/generated/codedb-metadata-only-targets.txt`
- `docs/generated/codedb-semantic-coverage.md`

The three text files describe list membership only. The semantic/table contract
carried by the Nu plugin and envctl catalog scan is documented separately in
`docs/generated/codedb-semantic-coverage.md`.

There are two distinct surfaces:

1. `codedb_file_imports` / `envctl_yazelix_file_import`
   - driven only by `yazelix_file_target_inventory.json`
   - includes both full content-blob imports and metadata-only targets
2. direct envctl catalog scanning
   - separate path for control-plane/config discovery
   - not the same as the Yazelix file-target import manifest

## Current row counts from the live manifest

- total rows: `3549`
- content-blob uploads: `1909`
- metadata-only targets: `1640`

By `source_of_truth_class`:

- `envctl_control_surface`: `1039`
- `repo_source`: `802`
- `real_home_runtime_state`: `1335`
- `nix_store_package_output`: `366`
- `real_home_user_config`: `5`
- `real_home_desktop_entry`: `2`

Content-blob rows by class:

- `envctl_control_surface`: `1039`
- `repo_source`: `802`
- `real_home_runtime_state`: `68`

Metadata-only rows by class:

- `real_home_runtime_state`: `1267`
- `nix_store_package_output`: `366`
- `real_home_user_config`: `5`
- `real_home_desktop_entry`: `2`

## Exact extraction commands

Note: use plain `jq` for exact row counting and extraction here. The `rtk`
wrapper is useful for interactive inspection, but its formatting layer can
distort line-oriented count pipelines such as `... | wc -l`.

Full content-blob file list:

```bash
jq -r '[.[] | select(.import_mode=="content_blob") | .absolute_path] | sort[]' \
  /home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json
```

Full metadata-only target list:

```bash
jq -r '[.[] | select(.import_mode=="metadata_only") | .absolute_path] | sort[]' \
  /home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json
```

Full database-import target list:

```bash
jq -r '[.[] | .absolute_path] | sort[]' \
  /home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json
```

Full envctl-owned content-blob list:

```bash
jq -r '[.[] | select(.import_mode=="content_blob" and .source_of_truth_class=="envctl_control_surface") | .absolute_path] | sort[]' \
  /home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json
```

Full Yazelix repo-source content-blob list:

```bash
jq -r '[.[] | select(.import_mode=="content_blob" and .source_of_truth_class=="repo_source") | .absolute_path] | sort[]' \
  /home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json
```

Full runtime-generated content-blob list:

```bash
jq -r '[.[] | select(.import_mode=="content_blob" and .source_of_truth_class=="real_home_runtime_state") | .absolute_path] | sort[]' \
  /home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json
```

## Representative examples

Representative `envctl_control_surface` content-blob paths:

- `/home/flexnetos/FlexNetOS/src/envctl/.agents/rusty-idd-adapter.md`
- `/home/flexnetos/FlexNetOS/src/envctl/.agents/skills/feature-forge/SKILL.md`
- `/home/flexnetos/FlexNetOS/src/envctl/.agents/skills/planning-engineer/scripts/plan-weave-dispatch.sh`

Representative `repo_source` content-blob paths:

- `/home/flexnetos/FlexNetOS/src/yazelix/.beads/.br_history/issues.20260701_151758_339571239.jsonl`
- `/home/flexnetos/FlexNetOS/src/yazelix/.beads/.br_history/issues.20260701_151758_339571239.jsonl.meta.json`

Representative runtime-generated content-blob paths:

- `/home/flexnetos/.local/share/yazelix/configs/helix/config.toml`
- `/home/flexnetos/.local/share/yazelix/configs/yazi/init.lua`
- `/home/flexnetos/.local/share/yazelix/configs/yazi/keymap.toml`

Representative metadata-only targets:

- `/home/flexnetos/.config/yazelix/settings.jsonc`
- `/home/flexnetos/.config/yazelix/shell_nu.nu`
- `/home/flexnetos/.local/share/applications/com.yazelix.Yazelix.Mars.desktop`
- `/home/flexnetos/.local/share/yazelix/logs/startup_handoff/latest.json`

## How to interpret "must be uploaded"

If "uploaded" means bytes are stored as blobs, use the `content_blob` subset
only (`1909` current files).

If "uploaded" means all file targets that become CodeDB import rows, use the
entire manifest (`3549` current rows), with metadata-only rows carried as target
records rather than byte-for-byte blob ingestion.
