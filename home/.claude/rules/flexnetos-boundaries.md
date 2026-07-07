---
paths:
  - "FlexNetOS/**"
---

# FlexNetOS workspace boundaries

- The workspace root `~/FlexNetOS` is a **hollow orchestration area**, deliberately not a git repo. Real work happens in the peer repos under `src/`. Identity note: this workspace was meant to be **lifeos**; `lifeos_foundation_yzx` (Nix profile) carries the true identity.
- `~/FlexNetOS/usr/bin` is **quarantined-pack residue**, not a design surface. Do not add binaries there; do not treat its contents as canonical (each is a refactor target, archive-first). One toolchain owner: the foundation's Nix profile.
- `src/upstream/<owner>/<repo>` are evidence mirrors only — never edit them.
- Runtime state lives under `~/FlexNetOS/var` (log/lib/cache/tmp); archives under `var/lib/codex-runtime-gate/archives`, `var/log/raw`, or `~/.claude/archive`.
- envctl is the environment authority; meta is the workspace/fleet authority; yazelix+Nix own the toolchain. Do not create new owners.
- Runtime beats docs: verify live state before trusting any document, including this one.
