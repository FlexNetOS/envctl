---
description: Run the nu_plugin export workflow for exporting database content.
argument-hint: DB=<source> TABLE=<name> TARGET=<format-or-path> [MODE=plan|implement]
---

Use the project-local `$nu-plugin-export` skill for `/nu_plugin:export`.

Work from the active `nu_plugin` repository. Export the requested database
content through a Rust-first `nu_plugin`/CodeDB workflow.

Inputs:
- Database/source: `$DB`
- Table/view/blob namespace: `$TABLE`
- Target format or path: `$TARGET`
- Mode: `$MODE`
- Extra arguments: `$ARGUMENTS`

Follow the skill exactly:
- Identify the database, table, view, query, or blob boundary first.
- Declare the export target, ordering, pagination, redaction, checksum, and
  provenance contract before writing.
- Create missing export tables, views, materializers, indexes, manifests, or
  Nu/CLI surfaces when required.
- Keep durable behavior in Rust crates rather than one-off scripts.
- Validate success, missing-surface creation, and safety boundaries.
- If export implementation fails, preserve evidence, create a fresh
  `nu_plugin` worktree from latest `origin/master`, create a detailed GitKB
  task, commit the task first, implement the upgrade, push a PR, and enable or
  request automerge when repository policy permits.
