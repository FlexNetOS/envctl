# envctl target descriptor registry

Generated at: `2026-07-04T23:10:14+00:00`
Status: `passed`

## Contract

- Descriptor schema: `schemas/target_descriptor.schema.json` (`2b394af8fea1ab7e88c83b1c50a244a21156d7ba728883e76d343f7aeff9a436`)
- Backing table: `envctl_migration_targets`
- Registry key: stable `target_id` upsert with canonical descriptor JSON and SHA-256 hash

## Target type coverage

| target type | covered |
|---|---|
| `codebase` | yes |
| `data` | yes |
| `infrastructure` | yes |
| `integration` | yes |
| `mixed` | yes |

## Registered descriptors

| target id | type | source | descriptor hash |
|---|---|---|---|
| `flexnetos-vs-lifeos` | `mixed` | `examples/target-descriptors/flexnetos-vs-lifeos.yaml` | `sha256:b3f653f9e9cda7991821687f041cb540ce3e4342bde03ef333aa0e72a6b42384` |
| `replace-with-target-id` | `codebase` | `examples/target-descriptors/generic-codebase.yaml` | `sha256:e79f5c931d50fb8193c678d61ddecc1405c8030ab9528946fcb65a7bde06bff9` |
| `synthetic-data-target` | `data` | `generated:synthetic-data-target` | `sha256:b60cc1a1afc87e27ced7bcd8cd986a858d8bc85fa7509692133faf682e1d6c7d` |
| `synthetic-infrastructure-target` | `infrastructure` | `generated:synthetic-infrastructure-target` | `sha256:0527a67aad464952377710ed804063fb78f929cfb6db7580a33911ca34884f79` |
| `synthetic-integration-target` | `integration` | `generated:synthetic-integration-target` | `sha256:2c0b9fff80948d3964bfb88006d579c33ee3cb26c9dea98478e9085c9ffcb6e6` |

## Runtime checks

- Idempotent upsert kept row count stable: `True`
- Descriptor hashes stayed stable on re-register: `True`
- Invalid descriptor rejected: `True`
- Lookup checks passing: `True`
