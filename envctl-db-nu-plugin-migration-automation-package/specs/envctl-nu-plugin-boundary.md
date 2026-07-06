# envctl / nu_plugin boundary

## Rule

`nu_plugin` must not independently mutate migration truth. It either calls envctl CLI/API/library methods or uses sanctioned transaction APIs.

## Boundary options Codex may choose after repo inspection

1. Invoke envctl CLI and parse JSON output.
2. Link a shared Rust crate used by envctl and plugin.
3. Use local IPC/socket if envctl already has a daemon/service pattern.
4. Use direct DB connection only if envctl already exposes safe transaction APIs and plugin authorization.

## Recommended default

Start with envctl JSON CLI/API boundary because it keeps ownership clear and avoids duplicating DB access logic in the plugin.
