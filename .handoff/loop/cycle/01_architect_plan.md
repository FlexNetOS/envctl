# TASK-0017 — component manifest extends · VERDICT: GO

## Trigger Check

TASK-0017 asks for kasetto-style `extends` composition for envctl component manifests. The agent-env
crate already contains the reusable no-downgrade model: strip `extends`, load parents recursively,
merge before deserialization, and fail closed on cycles/depth overflow. The envctl component
manifest loader currently parses each `*.toml` file directly into `ManifestFile`.

## Design

Add a local-only manifest composition layer in `Registry::load`:

- Parse each manifest file as raw `toml::Value`.
- Accept `extends = "parent.toml"` or `extends = ["base.toml", "team.toml"]`.
- Resolve relative parent paths from the child manifest's directory.
- Load parents before the child.
- Refuse cycles and chains deeper than 8.
- Merge `[[component]]` arrays by component `id`.
- For the same component `id`, deep-merge the component table so a child can inherit parent hooks and
  override only selected fields.

## Target Repos

Single repo: envctl. Sequential single-crew path.

Touched surfaces:

- `crates/engine/src/model.rs`
- `crates/engine/tests/engine.rs`
- `docs/ARCHITECTURE.md`
- `docs/KASETTO-FEATURES.md`
- `docs/ROADMAP.md`

## Non-Goals

- No remote manifest URLs; TASK-0017 has `allows_network=false`, and envctl component manifests are
  local TOML files.
- No new dependency.
- No change to the existing `[[component]]` schema.
