# TASK-0078 cache manifest scaffold guardian report

Status: PASS

## Invariants checked

- Existing cache-child status and validation report schemas remain unchanged.
- New scaffold report is read-only and leaves `apply_command` empty.
- Missing manifests receive deterministic escaped TOML stubs and owner-review next actions.
- Existing matching manifests receive no stub and route to review-before-migration.
- Existing wrong/empty manifests receive no stub and route to fix-id-before-migration.
- No broad `.cache` mutation, manifest write, or cache-child apply is performed.

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
rc=0
meta-local audit: PASS warnings=10 changed=0 dot_entries=79
scaffold_lines=85 validation_lines=85
rows=84 missing=84 no_decl=84 scaffold=84 pending=84 stub=84 next=84 nonempty_apply=0
config=0 nested=0 sensitive=0
apply_empty=PASS
```

The runtime command used report-only owner-supervised flags, did not pass `--apply`, and emitted no non-empty scaffold `apply_command` values.
