# Handoff Packet (latest) — handoff.packet.v2

## 1. North Star
envctl owns and contains the meta environment: every FlexNetOS tool/dotfile/.local/bin resolves inside meta; user-global ($HOME/.local, ~/.claude) holds ONLY symlinks into meta; envctl exports META_ROOT (resolved from the .meta.yaml marker, like meta_core's META_DATA_DIR) so no config hardcodes paths; secrets are held and auto-injected. Heal not harm; never downgrade; never delete (archive).

## 2. State Precedence
Git > .handoff/ledger.db > tasks/*.task.json > active.md > this packet.

## 3. Progress
Done: 51/54.  Tamper-evident events verified: 67.

## 4. Remaining (next safe first)
- [P2] **TASK-0009** — Relocate kasetto + kst (BLOCKED: superseded by Epic C built-in absorption)
- [P1] **TASK-0033** — VPS Profile B (BLOCKED owner-gated): F7 install gate + F8/OI-SM-2 authorizer + OI-SM-3 trusted-time
- [P1] **TASK-0039** — remote-clients-CA lifecycle: mint/<=7d-leaf/renew/revoke + revocation-set propagation for mTLS verifier

## 5. Next Best Task
**TASK-0039** — remote-clients-CA lifecycle: mint/<=7d-leaf/renew/revoke + revocation-set propagation for mTLS verifier
  objective: remote-clients-CA lifecycle: mint/<=7d-leaf/renew/revoke + revocation-set propagation for mTLS verifier

## 6. Resume Commands
```bash
hf resume
hf claim TASK-0039
```

## 7. Machine Summary
```json
{
  "done": [
    "TASK-0001",
    "TASK-0002",
    "TASK-0003",
    "TASK-0004",
    "TASK-0005",
    "TASK-0006",
    "TASK-0007",
    "TASK-0008",
    "TASK-0010",
    "TASK-0011",
    "TASK-0012",
    "TASK-0013",
    "TASK-0014",
    "TASK-0015",
    "TASK-0016",
    "TASK-0017",
    "TASK-0018",
    "TASK-0019",
    "TASK-0020",
    "TASK-0021",
    "TASK-0022",
    "TASK-0023",
    "TASK-0024",
    "TASK-0025",
    "TASK-0026",
    "TASK-0027",
    "TASK-0028",
    "TASK-0029",
    "TASK-0030",
    "TASK-0031-PR2",
    "TASK-0031-PR2C",
    "TASK-0031",
    "TASK-0032",
    "TASK-0034",
    "TASK-0035",
    "TASK-0036",
    "TASK-0037",
    "TASK-0038",
    "TASK-0041",
    "TASK-0042",
    "TASK-0043",
    "TASK-0044",
    "TASK-0045",
    "TASK-0046",
    "TASK-0047",
    "TASK-0048",
    "TASK-0049",
    "TASK-0050",
    "TASK-0051",
    "TASK-0052",
    "TASK-0053"
  ],
  "next_command": "hf claim TASK-0039",
  "next_task_id": "TASK-0039",
  "project": "handoff (Continuity Ledger Kernel)",
  "remaining": [
    "TASK-0009",
    "TASK-0033",
    "TASK-0039"
  ],
  "schema": "handoff.packet.v2",
  "tasks_total": 54,
  "witnessed_events_verified": 67
}
```

## Contract Proof (ADR-0011 — ruvector-verified/Lean)
Active task **TASK-0039** — AgentContract PROVEN via ruvector-verified (3 obligation(s)).
- ✓ `intent:objective` (Eq.refl proof-term #0)
- ✓ `intent:path_scope` (Eq.refl proof-term #1)
- ✓ `intent:acceptance` (Eq.refl proof-term #2)
3 proof-term(s) · proof-hash `4fae6edd4fe50dc5` · binding `0x868602ae2eddac78` · verifier `0x00010000` (lean-agentic 0.1.0).
