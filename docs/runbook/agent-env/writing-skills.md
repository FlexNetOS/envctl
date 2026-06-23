# Writing Skills

> **Ported from kasetto.dev/docs** (Kasetto v3.2.0, absorbed into envctl `crates/agent-env`).
> Renamed kasetto→`envctl agent`; `kasetto.yaml`→`agent-env.yaml`; mimalloc removed.
> Source: https://www.kasetto.dev/docs/writing-skills. The standalone `kasetto` binary is retired — this is the `envctl agent` surface.

Author skills that envctl can sync.

Skills are just directories with a `SKILL.md` file inside. Here's how to structure yours and what envctl looks for.

## Directory Layout

envctl discovers skills from the source root and from `skills/`:

```
repo-root/
├── SKILL.md                ← discovered as repository name skill
├── my-skill/
│   └── SKILL.md        ← discovered
├── skills/
│   ├── another-skill/
│   │   └── SKILL.md    ← discovered
│   └── third-skill/
│       └── SKILL.md    ← discovered
└── README.md           ← ignored (no SKILL.md)
```

envctl picks up:

- A top-level `SKILL.md` (installed using the repository name, or `sub-dir` basename).
- Any root-level subdirectory containing `SKILL.md`.
- Any `skills/<name>/SKILL.md` subdirectory.

Directory names are used as skill identifiers for folder-based skills.

`SKILL.md` is required. Directories without one are silently skipped.

## SKILL.md Format

`SKILL.md` is a markdown file. YAML frontmatter is optional but gives you control over how the skill appears in `envctl agent list`:

```
---
name: Code Reviewer
description: Reviews pull requests for common issues and style violations.
---

# Code Reviewer

Detailed instructions for the AI agent go here. This is the content that gets
installed into the agent's skill directory.
```

### Frontmatter Fields

| Field | Required | Description |
| --- | --- | --- |
| `name` | no | Display name shown in `envctl agent list` and `envctl agent doctor` |
| `description` | no | Short description carried in the lock + JSON output |

Both are optional. If you skip them, envctl parses the markdown body instead:

- **Name:** First `#` heading in the document, or the directory name if no heading exists.
- **Description:** First non-empty, non-heading paragraph, or `"No description."` if none found.

### Minimal Example

No frontmatter? No problem:

```
# My Skill

You are an expert at doing X. When the user asks you to...
```

envctl uses `"My Skill"` as the display name and the first paragraph as the description.

## Referencing Skills in Config

Reference skills by their directory name in `agent-env.yaml`:

```yaml
skills:
  - source: https://github.com/org/skill-pack
    skills:
      - my-skill           # matches repo-root/my-skill/ or repo-root/skills/my-skill/
      - another-skill
```

Want everything from a source? Use `"*"`:

```yaml
skills:
  - source: ~/Development/my-skills
    skills: "*"
```

## Custom Source Path

If a skill lives somewhere non-standard within the repository, point to it with the `path` field:

```yaml
skills:
  - source: https://github.com/acme/monorepo
    skills:
      - name: my-skill
        path: tools/ai-skills    # looks in tools/ai-skills/my-skill/SKILL.md
```

## Limiting Discovery to a Nested Directory

If your skills live under a nested plugin folder, set `sub-dir` on the source:

```yaml
skills:
  - source: https://github.com/acme/agents
    sub-dir: plugins/swift-apple-expert
    skills: "*"
```

envctl treats `plugins/swift-apple-expert` as the source root for discovery.
