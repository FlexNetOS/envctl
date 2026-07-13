# Cross-repo health findings — 2026-07-13

Failures-only matrix captured from the isolated envctl/Yazelix worktree set. Rows remain open until the final all-green rerun supplies replacement evidence.

| Surface | Result | Verified failure | Required repair |
| --- | --- | --- | --- |
| Full envctl musl release build | FAIL | `ring`/`cc-rs` searches for `x86_64-linux-musl-gcc`; the profile exports only `x86_64-unknown-linux-musl-gcc` | Add package-owned conventional musl aliases and a Cargo-level static-build contract |
| Cargo audit gate | FAIL | `cargo-audit` is absent from the sole active Yazelix profile | Package locked nixpkgs `cargo-audit` 0.22.1 in the existing foundation element |
| Exact MSRV | FAIL | CI and the active profile use a newer nightly and only assert compiler `>=1.89`; Rust 1.89 is not executed | Add a Nix-pinned exact 1.89 compatibility lane while retaining nightly default |
| Yazelix doctor | FAIL | JSON reports `healthy:false` because package-owned desktop launcher helpers are rejected as stale | Strictly recognize and validate the two profile-owned helper forms |
| Envctl ownership detection | FAIL | `auto-detect` emits 32 HIGH findings for commands canonically owned by the sole active profile | Add fail-closed exact-target active-profile classification in the engine |
| Envctl manifest lock | FAIL | `envctl lock --check` reports changed `codex-global-baseline` and added `postgres-ruvector`; no CI gate runs it | Review/regenerate the manifest lock and add a dedicated gate |
| Top-level envctl doctor | FAIL | A documented read-only command writes/removes a filesystem probe, uses CLI-local logic, reports stale roots, and returns success for unhealthy state | Move the report to the engine, use metadata-only checks, define nonzero unhealthy exit, and cover both frontends |
| DB frontend parity | FAIL | `db_parity.rs` calls the same engine method twice; GUI intentionally has no DB screen | Build a real GUI DB surface or remove the false parity claim only after equivalent capability exists |
| Remote libSQL integration | BLOCKED/FAIL | Seven ignored tests exist, but profile-owned `sqld` is absent; docs claim five | Package a reproducible test server, execute all seven, and correct docs |
| P7 durability gate | DEGRADED | Profile-owned `hf` is absent, so the gate runs a materially weaker fallback | Package `hf` or make the canonical check available; remove silent reduced coverage |
| Profile ownership of test tools | FAIL | Active `file`, `sqlite3`, and `cc` resolve under `/usr/bin`; hook coverage silently skips sqlite | Package required tools in the single foundation and make tests fail closed |
| Nu RTK projection | FAIL | origin/develop retains a duplicate envctl wrapper instead of sourcing Yazelix's profile-owned Nu module | Land canonical cleanup and clean-login/managed-Nu behavior tests |
| Active docs/contracts | FAIL | 1.88/stable, apt-based GUI setup, stale MCP-six policy, and five-test libSQL claims conflict with current sources | Reconcile owning instructions and regenerate agent-env projections |

Already-green baseline evidence before repairs: locked workspace build, default workspace tests, low-cost KDF workspace tests, provider/relay-edge feature lanes, fmt, clippy, agent-env lock/sync/doctor, registry, isolated `secretd --self-check`, and every static gate except cargo-audit.

