# ADR: owner-managed Codex skill catalog and capability packs

## Decision

Envctl keeps the compact catalog definition in `agent-skills/skill-catalog/`
and every complete skill tree in `agent-skills/capability-packs/`, including the
former hand-authored harness skills. `agent-env.active.yaml` is a generated projection: only `core` and
explicitly selected packs are copied into `.codex/skills` and `.agents/skills`
by the existing `envctl agent lock` → `sync --apply` owner lifecycle.

`skill-catalog` is the only always-active helper. Its compact catalog file is
inside its own locked skill tree, so lock refreshes attest the pack definitions,
aliases, task-intent mappings, and active-pack selection without advertising
every complete description at startup.

## Evidence

The installed Codex CLI exposes plugin and feature commands but no supported
skill-profile, enable/disable, lazy-discovery, or in-session skill-refresh
command. [OpenAI's official Codex use-case documentation](https://developers.openai.com/codex/use-cases/)
describes skills as reusable workflows, not an activation API. Therefore Envctl
uses materialized native discovery roots and documents a fresh Codex session
after activation.

The pre-change Envctl discovery projections contained 47 `SKILL.md` files, 41
unique names, and 10,464 UTF-8 bytes in frontmatter `description` values.
With only `core` materialized, the two discovery roots contain 10 files, five
unique names, and 2,734 description bytes: a 74% reduction in advertised
description payload. The catalog indexes 59 skills: 41 Envctl pack-owned
skills, the catalog helper, and 17 immutable GitKB skills whose tracked
canonical owner is `.kb/skills/`. Those immutable skills are materialized only
when the selected `git` pack explicitly requests them.

| Discovery/owner surface | Classification | Contract |
| --- | --- | --- |
| Codex-provided system roots | Immutable system skills | Not copied or made Envctl-owned. |
| Home-level roots | User-managed runtime skills | Out of this repository's source-of-truth and never scanned as a fallback. |
| `agent-skills/capability-packs/` | Envctl canonical owner | Complete mutable skill trees, retained whether active or inactive. |
| `.kb/skills/` | Immutable project-local GitKB owner | Explicit allowlisted catalog entries; no symlink fallback. |
| `.codex/skills/`, `.agents/skills/` | Generated discovery projections | Exactly `core` plus explicitly active packs; regenerated only by `envctl agent`. |

The installed client does not expose a machine-readable skill-truncation
status. Envctl therefore proves the reduction through the exact active-root
inventory above and requires a fresh session after owner materialization; it
does not claim an unsupported programmatic probe of the client's warning.

This is an Envctl implementation choice, not a claim that Codex has native
lazy discovery. It preserves the normal agent-env lock, no-C boundary, and
engine-owned command API.

## Safety contract

The catalog resolves only direct repository-local canonical paths. It rejects
unknown packs, missing `SKILL.md`, ambiguous aliases/intents, invalid names,
and a missing `core` pack. It never scans home directories, caches, backup or
archive paths, Git hooks, generated runtime state, or retired FlexNetOS/LifeOS
locations. `core.hooksPath` remains unrelated: it governs native Git hooks,
not Codex or Claude lifecycle hooks.

`envctl agent catalog --activate-pack <pack> --apply --sync` is the owner
operation: it updates the canonical active-pack declaration, rewrites the
generated projection, and runs lock → sync → locked verification through the
shared Engine API. `--activate-skill` and `--activate-intent` create a
deterministic temporary explicit pack; `--deactivate-pack` removes it or a
selected optional pack, while refusing to remove `core`.
