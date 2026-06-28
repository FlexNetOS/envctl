# TASK-0078 cache manifest scaffold plan

## Target

Add a read-only owner-supervised cache-child component manifest scaffold report that extends the
validated cache-manifest state with deterministic owner-review TOML stubs for missing component
manifests.

## Contract

New flag: `--owner-supervised-cache-child-component-manifest-scaffold PATH`.

TSV columns:

```text
dot_entry child_name child_path type canonical_target component_key expected_component_id cache_scope manifest_hint manifest_exists manifest_declares_expected_id scaffold_kind scaffold_status manifest_stub supervision next_action apply_command
```

Routing:

- missing manifest: `manifest_exists=no`, `manifest_declares_expected_id=no`,
  `scaffold_kind=component-manifest-minimal`, `scaffold_status=stub-needs-owner-review`, a
  deterministic escaped TOML `manifest_stub`, and
  `next_action=owner-review-cache-component-manifest-scaffold`
- existing manifest with expected `[[component]] id = "cache-<component>"`:
  `scaffold_kind=none`, `scaffold_status=existing-manifest-declares-expected-id`, empty
  `manifest_stub`, and `next_action=review-existing-cache-component-manifest-before-migration`
- existing manifest with no/wrong id: `scaffold_kind=none`,
  `scaffold_status=existing-manifest-id-mismatch`, empty `manifest_stub`, and
  `next_action=fix-cache-component-manifest-id-before-migration`
- `apply_command` stays empty; report-only, no manifest writes and no cache migration.

## Verification plan

- Red: focused fixture test fails because the new flag is unknown.
- Green: lock the 17-column schema plus missing/valid/wrong manifest branches, escaped stub
  content, direct-child exclusions, and scaffold-only mode.
- Runtime: live non-mutating audit should emit the current cache-child rows, all missing manifests,
  all deterministic stubs, all empty apply commands.
