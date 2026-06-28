# TASK-0078 managed config deep-diff summary guardian report

Status: PASS (PR #372)

## Invariants checked

- Existing managed-config conflict, conflict-summary, and deep-status reports remain read-only.
- New deep-diff summary report is read-only and leaves `apply_command` empty.
- The report emits aggregate counts only, not nested relative paths or file contents.
- Deep-identical rows route to `review-then-bridge-identical-managed-config-child`.
- Non-identical rows route to `owner-review-real-home-config-child-deep-diff-before-bridge`.
- No `--bridge-managed-config-child` or `--bridge-identical-managed-config-child` apply path is
  executed during runtime evidence.

## Verification evidence

```text
bash -n scripts/audit-meta-local-paths.sh scripts/tests/test-meta-local-path-audit.sh
bash scripts/tests/test-meta-local-path-audit.sh
# test-meta-local-path-audit: PASS
git diff --check
bash ci/gates/meta-local-policy.sh
# meta-local-policy: active install sources target META_ROOT FHS/XDG; only the single real-home .local bridge is allowed
bash ci/gates/harness-scripts.sh
# HARNESS-SCRIPTS GATE PASS
bash ci/gates/p7.sh
# P7 GATE PASS
```

## Runtime evidence

Live non-mutating audit against `/home/drdave/Desktop/meta` and `/home/drdave`:

```text
deep_status_lines=6
deep_diff_summary_lines=6
conflict_summary_lines=6
deep_diff_rows=5
bad_nf=0
bad_apply=0
differing_files_sum=0
type_conflicts_sum=15
real_only_entries_sum=23
managed_only_entries_sum=0
deep_identical[no]=5
next_action[owner-review-real-home-config-child-deep-diff-before-bridge]=5
child_names=ghostty,kasetto,nushell,systemd,yazelix
```

The runtime command used report-only owner-supervised flags, did not pass `--apply`, emitted no
non-empty `apply_command` values, and performed no live managed-config bridge/archive mutation.
