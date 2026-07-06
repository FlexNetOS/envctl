# Acceptance criteria

Codex may only claim completion when all applicable criteria pass or are explicitly marked blocked with evidence.

## envctl criteria

- [ ] A target descriptor can be parsed and validated.
- [ ] The prior FlexNetOS package can be inspected/imported as a package/artifact contract.
- [ ] A migration recipe can be created from contract phases.
- [ ] A migration run can be created in the database.
- [ ] Operations append events.
- [ ] Evidence records include path/URI, kind, hash, redaction status, and operation link.
- [ ] Artifact records include artifact ID, status, hash, path, evidence, and links.
- [ ] Approval requests block risky operations.
- [ ] Approval decisions are appended as events.
- [ ] Validation results are queryable.
- [ ] Replay can reconstruct or verify a run.
- [ ] Rollback metadata exists for operations that need rollback.
- [ ] Tests cover the above.

## nu_plugin criteria

- [ ] Plugin command signatures are registered according to the actual nu_plugin protocol used by the repo.
- [ ] Read-only commands return structured Nushell records/tables.
- [ ] Mutating commands call envctl through a controlled boundary.
- [ ] Status/timeline/artifacts/approvals/graph/validation commands work against test data.
- [ ] Approval commands append auditable envctl events.
- [ ] Errors are structured and actionable.
- [ ] Tests cover command outputs and failure cases.

## Integration criteria

- [ ] A run created by envctl appears in the plugin.
- [ ] Events appended by envctl appear in the plugin timeline.
- [ ] Plugin approval changes operation state in envctl.
- [ ] Artifact records link back to evidence and can be listed/opened.
- [ ] Replay verification result is visible through plugin.
- [ ] The FlexNetOS-vs-lifeos target descriptor is one fixture, not hardcoded into the engine.

## Documentation/issue criteria

- [ ] envctl issue prompt is updated.
- [ ] nu_plugin issue prompt is updated.
- [ ] Shared protocol issue prompt is updated.
- [ ] The strategy decision and gap analysis are present in docs.
