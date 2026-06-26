# Plan architecture loop upgrade research — distributed Rust/Lua edge/cloud fabric (2026-06-26)

Scope: upgrade Planning Engineer / plan-loop so every plan covers persistent memory/vector
intelligence, aggressive code+web auto-research, policy/org/A2A, Rust+Lua runtime choices,
distributed compute across owner hardware, and multi-vendor local+cloud execution.

## Current references used

- A2A official project: https://github.com/a2aproject/A2A — A2A targets cross-framework,
  cross-vendor agent collaboration.
- MCP official spec repo: https://github.com/modelcontextprotocol/modelcontextprotocol — MCP provides
  the tool/data protocol layer; its roadmap emphasizes conformance test suites and SDK tiers.
- Cloudflare Agents docs / Workers AI / Dynamic Workers: https://developers.cloudflare.com/docs-for-agents/,
  https://developers.cloudflare.com/workers-ai/, https://blog.cloudflare.com/dynamic-workers/ — current
  cloud/serverless agent surface and sandboxed isolate direction.
- Espressif Rust portal / esp-hal: https://developer.espressif.com/tags/rust/ and
  https://developer.espressif.com/blog/2025/02/rust-esp-hal-beta/ — vendor-backed Rust SDK direction
  for ESP32-class devices.
- Lune: https://github.com/lune-org/lune — Rust-built async Luau runtime.
- mlua: https://github.com/mlua-rs/mlua — Rust Lua/Luau bindings for embedded scripting.
- Raspberry Pi official site: https://www.raspberrypi.com/ — stable hardware family for local edge.

## Decisions encoded into the loop

1. Planning is now memory/vector-first: ICM, `.handoff`, source ledgers, GitKB, vector/RAG if present,
   and cold-start recall are mandatory planning axes.
2. Planning is auto-research-first: every cycle refreshes code graph intelligence and web/vendor docs;
   stale evidence invalidates recommendations.
3. Policy/org is first-class: Upgrade Only, No Downgrades, automation-first, human replacement where
   safe, explicit supervised boundaries, org chart, background agents, weave/A2A/MCP comms.
4. The runtime north star is Rust+Lua: Rust for safety/control/data/embedded; Lua/Luau for portable
   script/policy runtime only where it strengthens deployability without weakening trust boundaries.
5. Distributed compute target matrix includes workstation/GPU, local servers, mobile, AI glasses,
   Raspberry Pi/Pi Zero class Linux, ESP32/ESP32-S3 class MCU, local models, and multiple cloud/vendor
   providers working together.
