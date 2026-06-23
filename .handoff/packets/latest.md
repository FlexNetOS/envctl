# Handoff Packet (latest) — handoff.packet.v2

> Compiled by `hf fleet render envctl` from the FLEET ledger (meta/.handoff) + this repo's git-text capsule/cards. Not rendered from a per-repo ledger (ADR-0004 §3).

## 1. North Star (envctl)
envctl owns and contains the meta environment: every FlexNetOS tool/dotfile/.local/bin resolves inside meta; user-global ($HOME/.local, ~/.claude) holds ONLY symlinks into meta; envctl exports META_ROOT (resolved from the .meta.yaml marker, like meta_core's META_DATA_DIR) so no config hardcodes paths; secrets are held and auto-injected. Heal not harm; never downgrade; never delete (archive).

## 2. State Precedence
Git > FLEET ledger (meta/.handoff/ledger.db) > tasks/*.task.json > this packet.

## 3. Progress
Done: 50/54.  FLEET tamper-evident events verified: 1037.

## 4. Remaining
- [P2] **TASK-0009** — Relocate kasetto + kst (BLOCKED: superseded by Epic C built-in absorption)
- [P1] **TASK-0033** — VPS Profile B (BLOCKED owner-gated): F7 install gate + F8/OI-SM-2 authorizer + OI-SM-3 trusted-time
- [P1] **TASK-0039** — remote-clients-CA lifecycle: mint/<=7d-leaf/renew/revoke + revocation-set propagation for mTLS verifier
- [P0] **TASK-0053** — Route verified GitHub transport doctrine into envctl

