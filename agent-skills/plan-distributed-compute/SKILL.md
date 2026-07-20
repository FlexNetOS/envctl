---
name: plan-distributed-compute
description: >-
  Plans Rust+Lua distributed compute across user hardware and vendors: workstation, local servers,
  mobile, AI glasses, Raspberry Pi Zero, ESP32, cloud GPUs/serverless, local models, and multi-vendor
  control/data planes.
---

# plan-distributed-compute — Rust/Lua multi-vendor edge/cloud fabric axis

The north star is distributed compute across all owner hardware with Rust as the safety/control plane
and Lua/Luau as a small policy/scripting plane where it is the right fit. Emit
`.handoff/loop/plan/findings/distributed-compute-<T>.md`.

Required sections:
1. **Hardware target matrix** — workstation/GPU, local servers, phones/tablets, AI glasses/wearables,
   Raspberry Pi / Pi Zero class Linux, ESP32/ESP32-S3 class MCUs, and offline/degraded modes.
2. **Language/runtime map** — Rust std/no_std/embedded, Lua/Luau/mlua/Lune/Xedge-style scripting,
   WASM optional sandboxing, and explicit no-C/no-downgrade constraints for envctl trust boundaries.
3. **Vendor mesh** — local models/Ollama or equivalent, OpenAI, Anthropic/Claude via weave where used,
   Cloudflare Workers/Workers AI, Hugging Face, GitHub/Copilot cloud agent, and any project-local
   provider; classify local/cloud responsibility and failover.
4. **Control/data plane** — scheduling, discovery, telemetry, secrets, model routing, OTA/update,
   message bus/A2A, bandwidth/power constraints, and privacy/data-residency policy.
5. **Upgrade rows** — `axis: distributed-compute`, evidence, acceptance, risk, reversibility.
