# Agent interop posture

How `grit` interoperates with the fleet's agent-communication and execution planes, and the named
bridges to build.

---

## grit

grit is the **code-contention authority** (it gates who may edit which AST symbol); it is not a
message bus. Its interop posture is defined by what it emits (lock-lifecycle events) and what consumes
them. Both grit and `weave` are tagged `orchestration` in `.meta.yaml:184-200` and are complementary
planes — grit arbitrates writes on code, weave arbitrates messages between sessions.

| Plane | grit today | Posture / named bridge |
|---|---|---|
| **weave** (cross-session A2A session mesh) | none in `src/` — no direct grit→weave bridge | **Build the grit→weave bridge** (roadmap #10, rpo-A): forward room `Released`/`AgentDone` events as weave nudges so the next queued agent's *session* is pinged. Additive, fail-open like the existing `notify`. Live proof: the weave A2A round-trip this cycle (envctl asked rusty-idd to verify the front-door plan; rusty-idd corrected; envctl shipped prompt_hub PR #182). |
| **mcp** | none — grit ships no MCP server; no `.mcp.json`/`.codex` MCP surface | **Candidate (ADR):** expose grit verbs as an `mcp` server (or a stable `--json` mode) so agents consume a machine contract instead of scraping ANSI-colored stdout (PA-U2/PA-G1, roadmap #11). Today an agent's only "tool" is `Bash(grit:*)` + exit-code/substring scraping — string-fragile even internally. |
| **ACP** (agent-client execution protocol) | not implemented | grit has no `ACP` surface; it is driven as a CLI. If a consuming harness speaks `ACP`, grit sits *beneath* it as the contention gate — the harness's `ACP` execution requests must first pass grit's `claim`. No grit-side `ACP` work is warranted until the `--json`/MCP contract (above) lands. |
| **A2A** (agent-to-agent) | indirect only — agents observe each other through the shared lock truth + the `room` socket pub/sub (Claimed/Released/AgentDone) | grit's local `A2A` analogue is `room.sock`; the distributed analogue is Azure Event Grid / S3 notifications (poll). The cross-session `A2A` transport is weave (see the bridge above). grit's emission is non-blocking/fail-open, so it never stalls a foreground agent. |
| **GitHub cloud agent** | partial — `session pr` runs `git push` + PR create; release workflows exist (but hardcode `rtk-ai/grit`) | A **GitHub cloud agent** can open PRs via `session pr` and coordinate over a cloud lock backend (S3/Azure). Two interop fixes required: parameterize the release/homebrew workflows to `${{ github.repository }}` (gov-010) so the FlexNetOS fork's path doesn't target upstream; and gate the push/PR destructive seam (`[!!]`). Caller-identity binding (roadmap #6) is needed before an untrusted cloud agent can be allowed to `release`/`done` others' locks. |

### Interop summary

grit's authoritative interop today is: **emit** lock events on `room`/Event-Grid, **consume** nothing
from other planes (no recall-informed merge). The two highest-value interop upgrades are (1) the
grit→weave A2A bridge (makes coordination cross-session, not just cross-process) and (2) a machine
contract (`--json`/`mcp`) so any agent — local, container, remote-vm, or GitHub cloud agent — drives
grit without prose-scraping. Both are additive and fail-open; the cloud-agent path additionally needs
caller-identity authorization (ADR) before it is safe across a trust boundary.
