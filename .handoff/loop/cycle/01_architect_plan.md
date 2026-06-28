# TASK-0078 cache manifest id readiness plan

## Target

Add a read-only owner-supervised cache-child component manifest validation report that preserves the existing manifest-status schema while surfacing the exact component-id contract required by `--migrate-cache-child NAME`.

## Contract

New flag: `--owner-supervised-cache-child-component-manifest-validation PATH`.

TSV columns:

```text
dot_entry child_name child_path type canonical_target component_key expected_component_id cache_scope manifest_hint manifest_exists manifest_declares_expected_id supervision next_action apply_command
```

Routing:

- missing manifest: `manifest_exists=no`, `manifest_declares_expected_id=no`, `next_action=create-cache-component-manifest-before-migration`
- existing manifest with expected `[[component]] id = "cache-<component>"`: `yes/yes`, `next_action=review-existing-cache-component-manifest-before-migration`
- existing manifest with no/wrong id: `manifest_exists=yes`, `manifest_declares_expected_id=no`, `next_action=fix-cache-component-manifest-id-before-migration`
- `apply_command` stays empty; report-only, no cache migration.

## Verification plan

- Red: focused fixture test fails because the new flag is unknown.
- Green: lock the 14-column schema plus missing/valid/wrong manifest branches and validation-only mode.
- Runtime: live non-mutating audit should emit 84 current cache-child rows, all missing manifests, all empty apply commands.
