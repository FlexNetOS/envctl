---
description: Run the nu_plugin import workflow for importing files into databases.
argument-hint: FILE=<path> DB=<target> [TABLE=<name>] [MODE=plan|implement]
---

Use the project-local `$nu-plugin-import` skill for `/nu_plugin:import`.

Work from the active `nu_plugin` repository. Import the requested file into the
requested database using Rust-first `nu_plugin`/CodeDB workflow.

Inputs:
- File: `$FILE`
- Database: `$DB`
- Table: `$TABLE`
- Mode: `$MODE`
- Extra arguments: `$ARGUMENTS`

Follow the skill exactly:
- Inspect the file and target database first.
- Create the right tables, indexes, migrations, provenance rows, audit rows, and
  validation surfaces when they do not already exist.
- Keep durable behavior in Rust crates rather than one-off scripts.
- Validate success and missing-table creation.
- If import implementation fails, preserve evidence, create a fresh
  `nu_plugin` worktree from latest `origin/master`, create a detailed GitKB
  task, commit the task first, implement the upgrade, push a PR, and enable or
  request automerge when repository policy permits.
