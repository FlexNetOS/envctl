# Cross-Repo Reference Map — OpenRouter Agent Environment

| Source contract | Consumer/landing | Impact | Compatibility |
| --- | --- | --- | --- |
| Meta project inventory | Nu fleet verifier | All declared repos are classified; only repos with config+lock run per-repo sync/audit | Additive, read-only |
| Envctl `agent sync/audit` | Independent repo `agent-env.yaml` + lock | Per-repo generated agent assets | Existing config/lock schema unchanged |
| Central Codex catalog/profiles | Yazelix-owned active `~/.codex` | Every repo-launched Codex session | Existing baseline projection; no project provider config |
| OpenRouter probe contract | Harness shim and supervised runner | Two direct callers | JSON proof is additive and secret-redacted |

No shared Rust protocol or substrate signature changes are planned. A repo
without independent agent-env state remains centrally controlled through the
active Codex runtime and is reported as inherited, never as a successful
per-repo sync.
