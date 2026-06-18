# Handoff Packet (latest) — handoff.packet.v2

> Compiled by `hf fleet render envctl` from the FLEET ledger (meta/.handoff) + this repo's git-text capsule/cards. Not rendered from a per-repo ledger (ADR-0004 §3).

## 1. North Star (envctl)
envctl owns and contains the meta environment: every FlexNetOS tool/dotfile/.local/bin resolves inside meta; user-global ($HOME/.local, ~/.claude) holds ONLY symlinks into meta; envctl exports META_ROOT (resolved from the .meta.yaml marker, like meta_core's META_DATA_DIR) so no config hardcodes paths; secrets are held and auto-injected. Heal not harm; never downgrade; never delete (archive).

## 2. State Precedence
Git > FLEET ledger (meta/.handoff/ledger.db) > tasks/*.task.json > this packet.

## 3. Progress
Done: 34/53.  FLEET tamper-evident events verified: 840.

## 4. Remaining
- [P2] **TASK-0007** — envctl doctor boundary-refusal on out-of-meta FlexNetOS install; idempotent ~/.local/bin symlink regen
- [P2] **TASK-0008** — Relocate meta-mcp into meta/meta_mcp (first proof of the relocation procedure)
- [P2] **TASK-0009** — Relocate kasetto + kst (BLOCKED: superseded by Epic C built-in absorption)
- [P1] **TASK-0015** — Provisioning fidelity — verbatim skill copy; 5 command-format + 4 MCP-merge additive transforms
- [P1] **TASK-0016** — Lock unification — fold agent assets into envctl.lock (SHA-256 section)
- [P1] **TASK-0017** — Adopt kasetto extends config composition for envctl component manifests
- [P1] **TASK-0019** — fix-secretd: U1 USB-unlock path needs a real RealUsbProbe
- [P2] **TASK-0021** — node-via-bun manifest follow-up: node not-applicable when real node present, or add node-real component
- [P1] **TASK-0022** — agent-web-access Phases 2-3 (Phase 1 n8n-mcp+kasetto wiring merged; live smoke is human-only)
- [P2] **TASK-0029** — portability-links.toml branch fork reconcile (usrlocal-script-links develop/master divergence)
- [P2] **TASK-0031-PR2c** — Parse PROXY-protocol header to key per-IP shed on real client IP behind an L4 front
- [P1] **TASK-0033** — VPS Profile B (BLOCKED owner-gated): F7 install gate + F8/OI-SM-2 authorizer + OI-SM-3 trusted-time
- [P1] **TASK-0034** — Hardening tail: F10 tonic pin + cargo-audit CI, F11 MSRV check, F18 audit-fsync
- [P1] **TASK-0035** — secretd gRPC surface gaps: Vault List/Rm/Rotate, Relay Create/List, Audit Query, Get meta
- [P1] **TASK-0037** — Phase-7 verify-don't-rebuild: confirm secrets verbs on envctl + install secretd component; fix stale ROADMAP
- [P1] **TASK-0038** — secretd Certs.* service + non-mitm ca_issue + secretctl ca (deferred from TASK-0035, Phase 4+)
- [P1] **TASK-0039** — remote-clients-CA lifecycle: mint/<=7d-leaf/renew/revoke + revocation-set propagation for mTLS verifier
- [P1] **TASK-0044** — Pick-time dependency authority via the hf kernel: mint envctl backlog into fleet-scoped handoff.task.v1 cards
- [P2] **TASK-0052** — Full eject/package forge-loop into harness_hub (packaged-harness shape; doctrine override; capstone; cross-repo)

