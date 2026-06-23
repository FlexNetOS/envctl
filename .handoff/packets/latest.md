# Handoff Packet (latest) — handoff.packet.v2

## 1. North Star
envctl owns and contains the meta environment: every FlexNetOS tool/dotfile/.local/bin resolves inside meta; user-global ($HOME/.local, ~/.claude) holds ONLY symlinks into meta; envctl exports META_ROOT (resolved from the .meta.yaml marker, like meta_core's META_DATA_DIR) so no config hardcodes paths; secrets are held and auto-injected. Heal not harm; never downgrade; never delete (archive).

## 2. State Precedence
Git > .handoff/ledger.db > tasks/*.task.json > active.md > this packet.

## 3. Progress
Done: 45/53.  Tamper-evident events verified: 10.

## 4. Remaining (next safe first)
- [P2] **TASK-0009** — Relocate kasetto + kst (BLOCKED: superseded by Epic C built-in absorption)
- [P1] **TASK-0019** — fix-secretd: U1 USB-unlock path needs a real RealUsbProbe
- [P2] **TASK-0021** — node-via-bun manifest follow-up: node not-applicable when real node present, or add node-real component
- [P1] **TASK-0022** — agent-web-access Phases 2-3 (Phase 1 n8n-mcp+kasetto wiring merged; live smoke is human-only)
- [P2] **TASK-0029** — portability-links.toml branch fork reconcile (usrlocal-script-links develop/master divergence)
- [P2] **TASK-0031-PR2C** — Parse PROXY-protocol header to key per-IP shed on real client IP behind an L4 front
- [P1] **TASK-0033** — VPS Profile B (BLOCKED owner-gated): F7 install gate + F8/OI-SM-2 authorizer + OI-SM-3 trusted-time
- [P1] **TASK-0039** — remote-clients-CA lifecycle: mint/<=7d-leaf/renew/revoke + revocation-set propagation for mTLS verifier

## 5. Next Best Task
**TASK-0019** — fix-secretd: U1 USB-unlock path needs a real RealUsbProbe
  objective: fix-secretd: U1 USB-unlock path needs a real RealUsbProbe

## 6. Resume Commands
```bash
hf resume
hf claim TASK-0019
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
    "TASK-0020",
    "TASK-0023",
    "TASK-0024",
    "TASK-0025",
    "TASK-0026",
    "TASK-0027",
    "TASK-0028",
    "TASK-0030",
    "TASK-0031-PR2",
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
    "TASK-0052"
  ],
  "next_command": "hf claim TASK-0019",
  "next_task_id": "TASK-0019",
  "project": "handoff (Continuity Ledger Kernel)",
  "remaining": [
    "TASK-0009",
    "TASK-0019",
    "TASK-0021",
    "TASK-0022",
    "TASK-0029",
    "TASK-0031-PR2C",
    "TASK-0033",
    "TASK-0039"
  ],
  "schema": "handoff.packet.v2",
  "tasks_total": 53,
  "witnessed_events_verified": 10
}
```
