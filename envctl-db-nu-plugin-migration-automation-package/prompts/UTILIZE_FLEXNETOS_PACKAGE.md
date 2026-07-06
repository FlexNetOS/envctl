# How to utilize codex-flexnetos-migration-prompt-package

The prior package is the first real migration bundle and the acceptance fixture for envctl database automation.

## Treat it as four things

1. **Artifact contract source**
   - `source/previous-migration-artifact-context.md`
   - `prompts/ARTIFACT_CONTRACT_FULL.md`
   - `expected-output/migration-artifacts-tree.md`

2. **Execution recipe source**
   - `prompts/MASTER_PROMPT.md`
   - `prompts/EXECUTION_STYLE.md`
   - `helpers/background_scan.sh`
   - `prompts/spark_helpers/*.md`

3. **Adapter fixture**
   - The package defines how a bundle provides prompts, helpers, schemas, expected outputs, and source context.
   - envctl must import this metadata into database records.

4. **Regression fixture**
   - The adapter must prove it can run/import the package, ingest artifacts, preserve evidence, and expose status through `nu_plugin`.

## Required envctl commands/API capabilities

Codex must adapt names to the existing repo, but the capability surface is:

```text
envctl migration package inspect <package-path>
envctl migration package import <package-path> --name flexnetos-artifact-contract
envctl migration target add --descriptor <target.yaml>
envctl migration plan --target <target-id> --contract <contract-id> --recipe <recipe-id>
envctl migration run <plan-id> --mode approval-gated
envctl migration events <run-id>
envctl migration artifacts <run-id>
envctl migration replay <run-id> --verify-hashes
envctl migration export <run-id> --format wiki|json|markdown
```

## Required package import mapping

| Prior package file | envctl database record |
|---|---|
| `PACKAGE_MANIFEST.json` | package version, file hashes, provenance |
| `prompts/ARTIFACT_CONTRACT_FULL.md` | artifact contract |
| `expected-output/migration-artifacts-tree.md` | artifact templates / required output tree |
| `prompts/MASTER_PROMPT.md` | recipe / execution plan source |
| `prompts/spark_helpers/*.md` | helper operation definitions |
| `helpers/background_scan.sh` | external collector operation |
| `schemas/*.json` | validation schemas |
| generated `migration-artifacts/**` | artifact records + evidence links |

## Required adapter behavior

- Inspect package manifest and compute hashes.
- Refuse to run if required package files are missing.
- Create a migration contract record.
- Create a recipe record for the package phases.
- Create operation records for shell helper, Spark helper scans, artifact synthesis, linking, validation, and final summary.
- Append events for operation start, stdout/stderr evidence, artifact creation, validation results, and blockers.
- Store raw evidence as immutable files or blobs referenced by hash.
- Convert generated artifact tree into database artifact records.
- Link artifacts using `artifact_id`, file path, link graph, and parent/related edges.
- Expose run through nu_plugin.

## Genericization requirement

Do not bake FlexNetOS into envctl. FlexNetOS should be one target descriptor and one package fixture. The same envctl machinery must run against any target with:

- descriptor
- recipe
- artifact contract
- safety policy
- scanner capability map
- output policy
