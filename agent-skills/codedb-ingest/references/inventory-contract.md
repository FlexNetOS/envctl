# Yazelix File Inventory Contract

Use this contract when transforming configs, settings, environments, generated runtime files, and arbitrary files into CodeDB table rows through:

```nu
codedb envctl import inventory <inventory.json>
```

The inventory is a JSON array. Each object becomes one `envctl_yazelix_file_import` row.

## Required Fields

```json
{
  "target_id": "stable-logical-id",
  "absolute_path": "/absolute/path/to/target",
  "normalized_logical_path": "repo_source:relative/path.toml",
  "owner": "envctl",
  "source_of_truth_class": "repo_source",
  "file_kind": "regular_file",
  "parser_hint": "toml",
  "safety_policy": "source_content_import_allowed",
  "reproduction_policy": "git_checkout",
  "import_mode": "content_blob"
}
```

## Field Guidance

- `target_id`: stable ID for the row. Include owner and normalized path when possible.
- `absolute_path`: path to inspect. Directories, sockets, package outputs, and missing paths are metadata-only.
- `normalized_logical_path`: source class plus a stable relative/logical path, for example `repo_source:manifest/dashboard.toml` or `real_home_runtime_state:var/lib/yazelix/...`.
- `owner`: `envctl`, `yazelix`, `meta`, `nix`, `codex`, `user`, or another explicit owner.
- `source_of_truth_class`: choose values such as `repo_source`, `generated_runtime_state`, `real_home_user_config`, `real_home_runtime_state`, `meta_fhs_state`, `nix_store_package_output`, `environment_snapshot`, `proof_artifact`, or `external_observation`.
- `file_kind`: `regular_file`, `directory`, `symlink`, `package_output`, `environment_snapshot`, `proof_log`, `binary`, `socket`, or a more specific local kind.
- `parser_hint`: tells CodeDB how to create structured rows from content blobs.
- `safety_policy`: policy text that explains whether content may be hashed/structured or only observed as metadata.
- `reproduction_policy`: how to reproduce the target, such as `git_checkout`, `envctl_install`, `generated_by_yazelix`, `nix_realise`, `observed_runtime_state_only`, or `local_observation_only`.
- `import_mode`: `content_blob` or `metadata_only`.

## Parser Hints

Structured rows are currently emitted for UTF-8 content with these hints:

- JSON-like: `json`, `jsonc`
- Text/config: `toml`, `nix`, `kdl`, `nu`, `lua`, `yaml`, `yml`, `markdown`, `desktop`, `service`, `shell`, `conf`, `terminal_conf`, `plain_config`

JSON/JSONC is flattened into `json_value` rows. Text/config formats are split into comment/entry/line rows using conservative line parsing. Unsupported hints still produce blob metadata for `content_blob` files but no structured rows.

## Import Modes

Use `content_blob` when all of these are true:

- The target is a regular file.
- The content is expected to be non-secret or safely redacted by policy.
- Capturing a SHA-256 blob ref and structured rows helps materialization, reconciliation, or proof.

Use `metadata_only` for:

- Runtime caches, logs, sessions, sockets, FIFOs, package outputs, large binaries, secrets, key material, credential stores, unknown files, and paths with unclear policy.
- Real-home user state unless the task explicitly asks for content capture and policy permits it.
- Any target whose content is not needed to prove the table mapping.

## Output Row Semantics

`codedb envctl import inventory` returns:

- `table = envctl_yazelix_file_import`
- `row_id`
- `target_id`
- `logical_owner`
- `absolute_path`
- `normalized_path`
- `source_of_truth_class`
- `file_kind`
- `parser_hint`
- `content_hash`
- `byte_length`
- `blob_ref`
- `import_safety_policy`
- `reproduction_policy`
- `import_mode`
- `import_status`
- `skip_reason`
- `structured_table = envctl_yazelix_file_structured_rows`
- `structured_status`
- `structured_row_count`
- `structured_rows`
- `last_observed`
- `provenance = yazelix_file_target_inventory`

`content_blob` regular files get `content_hash` and `blob_ref = sha256:<hash>`. `metadata_only` rows intentionally leave content fields empty and use `skip_reason` to preserve the policy reason.

## Minimum Inventory Targets

When the user asks for "all config, settings, environments, and all files", build inventory rows for:

- Tracked repo configs: TOML, YAML, JSON/JSONC, Nix, KDL, Nu, Lua, Markdown, desktop/service files, shell/config files, manifests, locks, and generated declarations.
- Envctl workspace state under `$META_ROOT/etc`, `$META_ROOT/usr/share`, `$META_ROOT/var/lib`, `$META_ROOT/var/cache`, `$META_ROOT/var/log`, and `$META_ROOT/var/tmp`, using metadata-only for volatile or sensitive state.
- Yazelix source and generated runtime state, especially generated Nushell initializers and local share runtime files.
- Environment snapshots requested by the user. Store them as generated JSON files with redacted values, then import that JSON as `content_blob` only if redaction is complete.
- Proof artifacts and logs as metadata-only or redacted/hash-only blobs.

Do not silently omit a class. If a class cannot be imported safely, add metadata-only rows and state the reason in the summary.
