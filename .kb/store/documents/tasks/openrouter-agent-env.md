---
id: 019f5f5d-2abe-7202-89ab-9fa63f8326a5
slug: tasks/openrouter-agent-env
title: "Add OpenRouter to centralized agent-env"
type: task
status: active
priority: high
---

## Goal

Add OpenRouter as a centrally controlled Codex provider in envctl's agent-env engine while preserving independent repository configuration and Yazelix/Nix profile ownership.

## Acceptance criteria

- [ ] Meta central control can project and sync the Codex provider/model configuration across every managed peer repository without destroying repo-local independence.
- [ ] `tencent/hy3:free` is a first-class Codex model selectable through `/model`.
- [ ] The active binary, runtime, and agent configuration are delivered through the single Yazelix Nix profile; no user-bin or generated-runtime edit shadows remain.
- [ ] Nushell is the mandatory primary execution lane and fallback shells are explicitly gated.
- [ ] TDD covers config parsing/rendering, lock determinism, fleet synchronization, and runtime behavior; the wide relevant test/gate suite passes.
- [ ] The user receives the exact envctl secrets-engine command for loading the OpenRouter key without exposing the secret.
- [ ] A live Codex run using the requested Hunyuan model succeeds and is captured as runtime proof.

## Evidence required

- Source/lock/projection diffs and model/provider parsing tests.
- Idempotent `envctl agent sync` preview/apply/no-op proof across the fleet.
- Yazelix `status`/`inspect`/`doctor` plus frontdoor and Nushell routing proof.
- Live OpenRouter request response from the profile-owned Codex CLI.
