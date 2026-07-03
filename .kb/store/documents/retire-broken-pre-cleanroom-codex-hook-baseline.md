---
id: 019f2a52-3abb-79a3-b941-73f974623260
slug: retire-broken-pre-cleanroom-codex-hook-baseline
title: "Retire broken pre-cleanroom Codex hook baseline"
type: incident
status: draft
priority: medium
---

## Root Cause

`manifest/components.d/codex-global-baseline.toml` still modeled the
pre-cleanroom Codex hook bundle as active desired state. It required generated
`hooks.json`, enabled `[features].hooks = true`, wrote lifecycle hooks with the
old `with-meta-env.sh` helper, and verified that helper.

That made manual hook purges non-durable: any envctl detect/install/verify
repair path could reintroduce the corrupted hook state.

## Repair

The envctl baseline now:

- Accepts `LIFEOS_ROOT` first, with `META_ROOT` retained only as a compatibility
  fallback.
- Removes active lifecycle hook generation and pre-cleanroom hook-helper checks.
- Pins `[features].hooks = false` until a clean-room lifecycle gate is rebuilt.
- Unlinks stale generated `~/.codex/hooks.json` during install.
- Adds a regression guard in
  `scripts/tests/test-flexnetos-codex-runtime-gate.sh`.

Hooks remain mandatory for the control plane. The removed bundle was the
corrupted pre-cleanroom implementation, not the future clean-room hook policy.

## Evidence

Archive before mutation:

```text
/home/flexnetos/FlexNetOS/var/lib/codex-runtime-gate/archives/envctl-pre-cleanroom-hook-generator-before-20260703T233100Z.tar.gz
sha256: 2f3122f94d847314886fe1999b8cf6da07c8104b2c651f421b25ac6ff9502ec5
```

Proof commands:

```text
bash scripts/tests/test-flexnetos-codex-runtime-gate.sh
PASS: FlexNetOS Codex runtime gate is archived, inactive, and generator-disabled

/home/flexnetos/FlexNetOS/usr/bin/envctl lock --check
envctl.lock matches the manifest (97 components)

codex features list
hooks stable false
```
