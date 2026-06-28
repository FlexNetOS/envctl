# TASK-0078 cache manifest id readiness implementer log

## Red

Focused TDD started by adding fixture expectations for `--owner-supervised-cache-child-component-manifest-validation PATH`; the first direct probe failed with `unknown argument: --owner-supervised-cache-child-component-manifest-validation`.

## Change

- Added the new read-only validation report flag to `scripts/audit-meta-local-paths.sh`.
- Reused existing cache component-key/id helpers and `cache_child_component_manifest_declares_id`.
- Preserved the 12-column manifest-status report unchanged.
- Added a separate 14-column validation TSV with `expected_component_id` and `manifest_declares_expected_id`.
- Documented the status + validation review sequence in the ADR and envctl-home README.

## Green

Focused test and gate evidence:

```text
bash -n scripts/audit-meta-local-paths.sh scripts/tests/test-meta-local-path-audit.sh
bash scripts/tests/test-meta-local-path-audit.sh
# test-meta-local-path-audit: PASS
git diff --check
bash ci/gates/meta-local-policy.sh
bash ci/gates/harness-scripts.sh
bash ci/gates/p7.sh
# all PASS
```

Runtime non-mutating evidence on 2026-06-28:

```text
rc=0
85 cache-validation.tsv
validation_rows=84 manifest_exists_yes=0 manifest_declares_expected_id_yes=0 create_next_action=84 nonempty_apply=0
```

No `--apply` was used and no live cache-child state was moved.
