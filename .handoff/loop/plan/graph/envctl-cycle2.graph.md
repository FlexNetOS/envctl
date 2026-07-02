# envctl code graph snapshot - cycle 2

Date: 2026-07-02
Worktree: /home/flexnetos/FlexNetOS/src/envctl-plan-autoresearch-20260702
Branch: codex/plan-autoresearch-20260702
Head: 52100614ab4666e6abab52a3292d0149351a9453

## Identity

| Field | Value |
| --- | --- |
| shell root | /home/flexnetos/FlexNetOS/src/envctl-plan-autoresearch-20260702 |
| shell branch | codex/plan-autoresearch-20260702 |
| gitkb stats kb_root | /home/flexnetos/FlexNetOS/src/envctl |
| gitkb doctor branch | master |
| gitkb flows branch | master |

Interpretation: the graph was freshly indexed, but the GitKB metadata points at the main checkout and `master`, not the active worktree branch. This must be treated as a planning evidence gap.

## Counts

| Metric | Value |
| --- | ---: |
| symbols | 4297 |
| Rust symbols | 4279 |
| Python symbols | 18 |
| stats files | 925 |
| doctor symbolized files | 149 |
| extracted call sites | 37408 |
| resolved call edges | 10157 |
| unresolved calls | 27251 |
| stale files | 0 |

## Unresolved calls

| Kind | Count |
| --- | ---: |
| no_match | 12687 |
| skip_list | 12276 |
| ambiguous | 1488 |
| stdlib_allowlist | 718 |
| symbol_forwarding_ambiguous_star | 82 |

## Entrypoint families

| Family | Examples |
| --- | --- |
| CLI binaries | `crates/cli/src/main.rs::main`, `crates/cli/src/bin/meta-env.rs::main` |
| GUI binary | `crates/gui/src/main.rs::main` |
| Secrets binaries | `crates/secretctl/src/main.rs::main`, `crates/secretd/src/main.rs::main` |
| Build/example/script | `crates/secrets-proto/build.rs::main`, `crates/secrets-engine/examples/seed_factor_probe.rs::main`, `scripts/meta-fleet-sync.py::main` |
| Tests | agent-env parity tests, engine tests, secretd e2e and edge hardening tests |

## Flow hotspots

| Rank | Symbol | Nodes | Files | Note |
| ---: | --- | ---: | ---: | --- |
| 1 | `crates/cli/src/main.rs::function::main` | 725 | 68 | primary CLI spine |
| 2 | `crates/agent-env/tests/parity_vs_kasetto.rs::function::parity_hash_dir_vs_independent_framing` | 493 | 65 | test flow ranked like product |
| 3 | `crates/gui/src/main.rs::function::test_app` | 474 | 56 | test helper around GUI surface |
| 4 | `crates/gui/src/main.rs::function::main` | 472 | 56 | GUI runtime spine |
| 5 | `crates/engine/tests/engine.rs` integration tests | 300+ | mixed | test coverage flows dominate top list |

## API and impact observations

- Public API results are dominated by `envctl-agent-env` types and helpers, including MCP settings, command targets, source configs, and driver surfaces.
- `code impact crates/secretd/src/main.rs --depth 3` returned 8 local symbols and no cross-service edge evidence.
- Service edge health is zero across routes, clients, inbound, outbound, and cross-service edges.

## Blind spots

- Branch and root identity mismatch makes graph evidence non-terminal until checked.
- Product and test flows need explicit labels.
- Service/RPC boundaries are not represented in the service-edge graph.
- Dead-code output needs `#[cfg(test)]` awareness before it can drive cleanup decisions.
