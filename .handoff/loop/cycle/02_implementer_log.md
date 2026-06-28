# TASK-0078 implementer log — cache-child manifest id validation

Date: 2026-06-28

## Red test

Added fixture coverage for an existing but wrong manifest:

```toml
[[component]]
id = "cache-other"
```

The new test expects `--apply --migrate-cache-child tool` to fail with:

```text
--migrate-cache-child tool: component manifest manifest/components.d/cache-tool.toml does not declare component id cache-tool; review/fix the manifest before migration
```

Red result: the existing code moved the cache child, proving that manifest existence alone was insufficient.

## Changes made

- Added `cache_child_component_id` to derive the canonical component id from the requested cache child name.
- Added `cache_child_component_manifest_declares_id`, a small shell/awk TOML-enough check scoped to `[[component]]` id fields.
- Added a dry-run/apply refusal path when the hinted manifest is present but lacks the expected id.
- Updated approved cache-child migration fixtures to use minimal matching component manifests instead of comment-only placeholders.

## Notes

- This is a strict safety upgrade on top of PR #368; no broad `.cache` migration is introduced.
- Missing-manifest, invalid-name, open-handle, target-collision, and approved migration coverage remain intact.
