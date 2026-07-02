# prompt-architecture — TARGET: grit

Dimension: prompt/tool/model/instruction surfaces reviewed AS architecture.
Target repo: `/home/drdave/Desktop/meta/grit` (origin `git@github.com:FlexNetOS/grit.git`, branch `master`).
Scan date: 2026-06-27. Read-only review. Evidence cites `file:line`.

## Verdict (1-line)

grit is an **agent-coordination substrate** ("Coordination layer for parallel AI agents on top of git", `Cargo.toml:5`) whose *own* prompt/instruction surfaces are essentially empty — generic ICM+RTK boilerplate with zero grit-specific guidance — so the very tool built to orchestrate agents does not expose itself to agents through any authoritative surface. The thin prompt surface is itself the headline finding, and it is a defect for a coordination substrate (not a neutral "library, no prompt needed" outcome).

---

## Instruction surfaces (what an agent actually reads)

| Surface | Path | Content | Assessment |
|---|---|---|---|
| CLAUDE.md | `grit/CLAUDE.md:1-160` | `<!-- icm:start -->` ICM block + `<!-- rtk-instructions v2 -->` RTK block. **Nothing about grit's own verbs.** | Auto-injected boilerplate; zero domain guidance |
| AGENTS.md | `grit/AGENTS.md:1-27` | ICM block only (26 lines). | No agent operating contract for grit at all |
| `.claude/` skills/agents | (absent) | No `.claude/`, no `SKILL.md`, no `agents/`, no `prompts/` anywhere in tree | No structured tool/skill surface |
| settings / hooks | (absent) | No `settings.json`, `.meta.yaml`, MCP config | No permission/hook governance plane |
| Agent-facing usage prose | `grit/examples/05-claude-code-integration.sh:13-58` | The ONLY place that tells an agent how to call grit — buried in a `cat <<PROMPT` heredoc inside a shell example | Authoritative guidance is mislocated in an example, not in an instruction surface |
| README "agent" framing | `grit/README.md` (problem/solution/how-it-works) | Human-facing narrative; no machine contract | Marketing, not an agent contract |

Hidden coupling: the example heredoc (`05-claude-code-integration.sh:13`) literally says "Add this to your CLAUDE.md or agent system prompt" — i.e. grit's design assumes a per-consumer copy-paste of its operating instructions, with no canonical source. Any drift between the verbs in `src/cli/mod.rs` and that heredoc is silent.

---

## The real tool surface: grit's CLI verbs (what SHOULD be exposed as tools)

Enumerated from `src/cli/mod.rs:31-175`. These ARE the tools a merge/lock-coordination agent would be granted:

| Verb | Source | Side effect class | Tool shape |
|---|---|---|---|
| `init` | `cli/mod.rs:34,414` | writes `.grit/`, indexes AST, starts notify server, edits `.gitignore` | setup (once) |
| `claim` (`-a -i --ttl --wait --mode --queue --with-deps`) | `cli/mod.rs:36-68,460` | acquires AST lock(s), creates git worktree | **core read/acquire** |
| `assign` (auto-pick free symbol) | `cli/mod.rs:142-163,1203` | locks first free symbol + worktree | core acquire |
| `release` | `cli/mod.rs:70-78,643` | drops locks, promotes queue | core release |
| `done` | `cli/mod.rs:101-106,890` | **rebase + merge worktree + delete branch + release** | **destructive merge** |
| `heartbeat` | `cli/mod.rs:165-174,1285` | refresh TTL (lease keep-alive) | liveness |
| `plan` | `cli/mod.rs:90-99,826` | read-only suggestion of symbols+deps | advisory |
| `status` / `symbols` / `queue list` / `worktree list` / `watch` | `cli/mod.rs:80-88,108-125,1120-1170` | read-only / event stream | observability |
| `session start/status/pr/end` | `cli/mod.rs:183-203,1300-1448` | branch create, **`git push` + PR create**, checkout base | destructive / external |
| `gc` | `cli/mod.rs:127,1190` | delete expired locks | maintenance |
| `config set-s3/set-azure/set-local/show` | `cli/mod.rs:219-249,1452` | writes backend config (incl. **Azure access-key on argv**) | config / secret-bearing |

Backend abstraction: `LockStore` trait with sqlite/s3/azure impls (`cli/mod.rs:388-412`) — model-agnostic, storage-pluggable. This is a sound, already-decided architecture.

---

## tool grants (what a consuming harness must grant, and the gaps)

CLAIM rows (evidence-backed):

- CLAIM PA-G1: grit offers **no machine-readable output contract**. Every verb prints ANSI-colored human prose via `colored::Colorize` (`cli/mod.rs:4`, e.g. `"+".green()`, `"x".red()` at `:564,:593`). The only `serde_json` use is decoding `RoomEvent` off the room socket for `watch` (`cli/mod.rs:1036`), never as command output. Evidence: `grep` for `--json`/`--format` in `cli/mod.rs` returns nothing. → An agent's only "tool" is a raw `Bash` shell-out that must scrape colored stdout, and decide success by exit code + substring matching (the code itself matches on `msg.contains("already exists")` / `"not found"`, `cli/mod.rs:556,920` — string-fragile even internally).
- CLAIM PA-G2: **Destructive verbs need explicit grant scoping that does not exist.** `done` merges + deletes branches (`cli/mod.rs:931-943`), `session pr` runs `git push` + PR creation (`cli/mod.rs:1415`), `session end` checks out the base branch (`cli/mod.rs:1442`). No allowlist/permission profile ships with grit; a consuming repo must hand-author one. Cross-ref: workspace policy `rusty-idd/.claude/rules/meta-destructive-commands.md` forbids unscoped `git` destructive ops, so a grit-driving agent inherits a governance obligation grit gives no help meeting.
- CLAIM PA-G3: **No caller-identity authorization on lock ownership.** `release`/`done` take an arbitrary `-a <agent>` and act on that id with no proof the caller IS that agent (`cmd_release` → `lock_store.release(sym, agent)`, `cli/mod.rs:657`; `cmd_done`, `cli/mod.rs:890`). Any process can release or "done"-merge another agent's locks. Coordination correctness rests on cooperative honesty of agent ids. (Counter-balance: `validate_identifier`, `cli/mod.rs:262-284`, blocks path-traversal/argv-injection in those ids — a real, present governance control, but it authenticates the *string*, not the *caller*.)
- CLAIM PA-G4: **Secret-bearing tool grant.** `config set-azure --access-key <key>` puts a storage key on argv (`cli/mod.rs:236-244,1482`); `set-s3` correctly defers to `AWS_*` env (`cli/mod.rs:1475-1477`). Asymmetric secret handling = grant hazard for an automated agent that logs commands.

Minimum tool grants a harness must provision: `Bash(grit:*)` on PATH; git worktree + push rights for `done`/`session pr`; `gh`/PR credentials for `session pr`; cloud creds via env for s3/azure backends. None are documented by grit.

---

## model lanes

grit's runtime is **LLM-free** — no model client, no provider SDK, no model selection anywhere in `Cargo.toml:10-66` or `src/`. So the model-lane question is **not internal to grit**; it is a contract grit imposes on the *consuming* multi-agent harness. Lane mapping derived from verb risk:

| Lane | Verbs | Rationale (evidence) |
|---|---|---|
| Mechanical (haiku-class) | `heartbeat`, `status`, `symbols`, `queue list`, `worktree list`, `gc`, `assign` | deterministic, low-blast — pure lease/observability (`cli/mod.rs:1285,730,1190`) |
| Structured (sonnet-class) | `claim`, `release`, `plan`, `session start/status` | scoped acquisition + advisory; reversible (`cli/mod.rs:460,643,826`) |
| Decision/gate (opus-class) | `done`, `session pr`, `session end` | `done` runs rebase+merge that **can fail** (`merge_error` path keeps branch for recovery, `cli/mod.rs:918-983`); merge-conflict adjudication + irreversible push/PR belong on the strongest lane |

UPGRADE rows:

- UPGRADE PA-U1 (axis:accuracy): Publish an authoritative grit **agent operating contract** as the single source of verb guidance — a `SKILL.md` (or canonical `AGENTS.md` section) generated from / checked against `src/cli/mod.rs` — replacing the example-heredoc as the source of truth (fixes PA-G1 scatter + drift). Blast radius: docs/CI only. Risk: low.
- UPGRADE PA-U2 (axis:accuracy): Add a `--json` output mode (or an MCP server) so agents consume a stable contract instead of scraping colored prose (fixes PA-G1). Blast radius: every verb's print path in `cli/mod.rs`; additive flag = backward-safe. Risk: medium (surface size).
- UPGRADE PA-U3 (axis:governance): Ship a recommended permission/model-lane profile (destructive-verb allowlist + lane map above) as a drop-in for consuming harnesses (fixes PA-G2). Blast radius: docs + sample settings. Risk: low.
- UPGRADE PA-U4 (axis:governance): Add caller-identity / token binding to lock ownership so `release`/`done` cannot act on another agent's locks (fixes PA-G3). Blast radius: `LockStore` trait + 3 backends. Risk: high — defer behind an ADR.
- UPGRADE PA-U5 (axis:governance): Accept the Azure key via env (mirror the S3 path) to remove argv secret exposure (fixes PA-G4). Blast radius: `cmd_config_set_azure` + `AzureConfig` load. Risk: low.

Note: an inline deferred-work code comment keyed `#queue-ttl-worktree` (`cli/mod.rs:694-696`) flags that queue-promoted agents get a hardcoded 600s TTL and no worktree — a coordination-fidelity gap adjacent to the tool surface, worth tracking but out of this dimension's scope.

---

## ADR candidates / no-ADR rationale

| ID | Decision | ADR? | Rationale |
|---|---|---|---|
| ADR-cand-1 | **Agent interface contract**: MCP server and/or stable `--json` output as grit's machine surface (drives PA-U2) | **ADR candidate** | Introduces a new public interface + likely new dependency/process model; cross-cutting and hard to reverse — a genuine architecture decision |
| ADR-cand-2 | **Lock-ownership authorization model** (caller identity binding, PA-U4) | **ADR candidate** | Changes the `LockStore` trust model across all 3 backends; security-relevant and high blast-radius |
| ADR-cand-3 | **Canonical agent operating-contract surface** (skill/AGENTS.md as single source, PA-U1) | borderline / **prefer no-ADR** | A documentation-convention fix, reversible, no architecture change — track as a roadmap item, not an ADR, unless it forces codegen from the CLI AST |
| no-ADR-1 | grit stays **model-agnostic / LLM-free** internally | **explicit no-ADR** | Correct by design (`Cargo.toml`, `src/` carry no model client); model-lane policy is the consumer's concern, not grit's — no decision record needed, just document the recommended lane map |
| no-ADR-2 | Pluggable **storage backend** abstraction (sqlite/s3/azure via `LockStore`, `cli/mod.rs:388-412`) | **no-ADR (already decided + stable)** | Architecture is sound and shipped; no open decision |
| no-ADR-3 | Identifier hardening via `validate_identifier` (`cli/mod.rs:262-284`) | **no-ADR** | Existing governance control working as intended; keep |

Confidence: high on instruction-surface emptiness and verb/output evidence (read directly from source); medium on the merge-decision lane mapping (inferred from verb risk, not from a benchmarked failure mode).
