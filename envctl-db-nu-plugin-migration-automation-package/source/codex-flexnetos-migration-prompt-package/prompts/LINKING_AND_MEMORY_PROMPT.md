# Linking and Persistent Memory Prompt

## Goal

Build a persistent memory layer so future Codex migration tasks can understand the repository without rediscovery.

## Required files

```text
migration-artifacts/MIGRATION_MEMORY.md
migration-artifacts/index.md
migration-artifacts/artifact-manifest.json
migration-artifacts/artifact-manifest.md
migration-artifacts/evidence-register.md
migration-artifacts/link-graph.md
migration-artifacts/wiki-home.md
migration-artifacts/_meta/scan-runs.jsonl
migration-artifacts/_meta/artifact-status.tsv
```

## Wiki-style linking rules

- Every artifact must have a stable relative Markdown link from `migration-artifacts/index.md`.
- Every artifact must include a backlink to `../index.md` or `../../index.md`.
- Use relative links only; no absolute local file links in final Markdown.
- Every graph source file must link to its rendered SVG/PNG if generated.
- Every finding must link to evidence in `evidence-register.md` or cite a raw scan file path.

## Persistent memory sections

`MIGRATION_MEMORY.md` must include:

1. Project identity
2. Proven purpose of FlexNetOS
3. Relationship to lifeos
4. Current-state architecture summary
5. Runtime and deployment summary
6. Data and integration summary
7. Toolchain summary
8. Known risks and blockers
9. Artifact map
10. Open questions
11. Commands used to create memory
12. Last verified timestamp

## Link validation

Run the link/index helper after artifacts are written:

```bash
python3 <prompt-package>/helpers/artifact_manifest.py migration-artifacts
python3 <prompt-package>/helpers/make_wiki_index.py migration-artifacts
```

If links break, fix the links before finalizing.
