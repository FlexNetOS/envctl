---
name: skill-catalog
description: "Discover and activate Envctl's locked Codex capability packs without copying unmanaged skills. Use for skill search, pack selection, provenance, or task-intent activation."
---

# Envctl Skill Catalog

The catalog is always active. It is a compact index, not a copy of every skill
description. Query it through the profile-owned Envctl command:

```text
envctl agent catalog
envctl agent catalog --search <query>
envctl agent catalog --show <skill>
envctl agent catalog --activate-pack <pack> --apply --sync
envctl agent catalog --activate-skill <skill> --apply --sync
envctl agent catalog --activate-intent <intent> --apply --sync
envctl agent catalog --deactivate-pack <pack> --apply --sync
```

Activation writes only the repository-owned `agent-env.active.yaml` projection.
It never reads `$HOME`, archives, caches, `.git/hooks`, or retired FlexNetOS/LifeOS
paths. `--sync` runs `envctl agent lock`, `envctl agent sync --apply`, and
`envctl agent lock --check --locked` through the shared Engine API after an
activation change. Start a fresh Codex session after materialization because
native Codex does not expose a supported in-session skill refresh API.
