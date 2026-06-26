---
name: handoff-sync
description: "Build & install the `hf` handoff KERNEL and seed the Tier-A `.handoff` layer/ledger for envctl (Epic A: handoff full-sync). ALWAYS use when asked to: 'build hf', 'sync the handoff layer', 'make .handoff tier-A', 'resume handoff full-sync', install/bring-up the continuity kernel, or wire the witnessed ledger. Drives: build `hf` from `meta/handoff`, `hf init`/`hf seed`, render policy/hooks/policies/active/packets/skills, mint task cards via `hf task mint`, and add the `p7-conformance` CI gate. DISAMBIGUATION vs session-relay: handoff-sync BUILDS/INSTALLS the kernel and SEEDS the Tier-A layer (one-time bring-up of the substrate); session-relay does PER-LOOP checkpoint/handoff ON TOP of an already-built kernel. If `hf` is not yet on PATH, this skill is the prerequisite; once it lands, the loops auto-upgrade to the witnessed ledger."
---

# Handoff Sync (build the `hf` kernel + seed Tier-A `.handoff`)

You bring up the **continuity substrate** for envctl: build & install the `hf` kernel binary, make
the per-repo witnessed ledger/export and fleet rollup reside correctly, seed the envctl `.handoff` Tier-A layer, mint
the backlog as schema-valid task cards, and gate the whole thing with a `p7-conformance` CI check.
This is the **one-time bring-up** that every later checkpoint/handoff (see `session-relay`,
`forge-loop`) rides on. After this skill lands, the loops detect `hf` on PATH and auto-upgrade from
hand-written `.handoff/loop/HANDOFF.md` to the witnessed ledger.

> **Not session-relay.** `handoff-sync` = *build the kernel + seed the Tier-A layer/ledger*.
> `session-relay` = *per-loop checkpoint + handoff* on top of the already-built kernel. Run
> handoff-sync once (Epic A); run session-relay every cycle-budget boundary.

> **Already built — don't rebuild from scratch.** The `hf` kernel already exists in `meta/handoff`
> (registered `.meta.yaml` project FlexNetOS/handoff). Build/install the existing kernel and then
> use the current CLI surface reported by `hf --help`; do not rely on stale verb inventories. As of
> ADR-0018/HFTASK-0067 the important sync verbs are `hf import` (rebuild local cache from committed
> JSONL), `hf handoff` (render active/packet), `hf export` (write committed JSONL), `hf sync`/`hf
> fleet status` (roll up/view fleet state), and `hf gitignore --check|--repair` (canonical
> durability policy).

Backlog map: this skill executes Epic A → **TASK-0001** (build/install hf) → **TASK-0002** (seed
Tier-A layer + mint cards) → **TASK-0003** (p7-conformance gate).

---

## P0 — LEDGER-RESIDENCY GUARD (gate everything on this)

**Source of truth.** Before changing this ledger/p7 doctrine, verify against `meta/handoff` source and ADRs, especially `docs/adr-0004-fleet-handoff-rollout.md` and `docs/adr-0018-full-auto-agentic-operation.md`. Do not trust older envctl harness prose; it carried a stale central-only ledger doctrine.

**The hazard.** The shipped `hf` resolves a **CWD-relative** handoff dir (`const HF = ".handoff"`)
and therefore a CWD-relative runtime cache at `.handoff/ledger.db` (`hf/src/main.rs::ledger_path()` =
`.handoff/ledger.db`). The committed per-repo continuity truth is `.handoff/ledger.events.jsonl`
plus rendered text artifacts. Binary `ledger.db`/RVF files are cache/runtime state unless the handoff
kernel explicitly changes the committed wire contract.

**The rule (fail-closed).** Every `hf` call must update the intended continuity surface: envctl's
per-repo state when operating on envctl, and the fleet view when rendering fleet state from
the member repo. After ledger-touching commands, run `hf export` so the committed `.handoff/ledger.events.jsonl`
stays in sync, and keep binary db/RVF caches ignored. Run `hf sync` only when rolling member events
up to the fleet ledger. Do not fork truth by committing stale/generated binary caches.

```bash
META_ROOT="$(cd "$(git rev-parse --show-toplevel)/.." && pwd)"   # or your known meta root
[ -d "$META_ROOT/.handoff" ] || { echo "FAIL: no $META_ROOT/.handoff"; exit 1; }
# Per-repo updates: run hf in the member repo that owns the state, then `hf export`.
# Fleet rollup/view: run `hf sync` / `hf fleet status` when explicitly coordinating across repos.
```

If a future `hf` adds `--ledger <path>` or a `HANDOFF_LEDGER`/`HANDOFF_DIR` env var, prefer that
(explicit > implicit) so commands cannot accidentally write the wrong continuity surface.

**Fail-closed check (run BEFORE and AFTER any hf invocation that could write):**

```bash
# The committed ledger export is allowed; binary db caches must not be unintentionally committed.
git ls-files .handoff | grep -q 'ledger\.events\.jsonl' || { echo "FAIL: missing committed ledger export"; exit 1; }
git check-ignore -q .handoff/ledger.db 2>/dev/null || { echo "FAIL: ledger.db cache is not ignored"; exit 1; }
git ls-files .handoff | grep -Eq '\.(db|db-wal|db-shm|rvf|rvf\.lock)$|(^|/)ledger\.db$' && { echo "FAIL: binary ledger/cache tracked in git"; exit 1; }
hf gitignore --check
```

If the guard fails, **do not proceed** — the hf-aware branch in the loops stays disabled and the
harness falls back to hand-written `.handoff/loop/HANDOFF.md`. A stale export or tracked binary cache is worse than no
ledger. (Mirror this guard wherever a loop gates its `hf checkpoint`/`hf handoff` calls.)

---

## Step 1 — Build & install the `hf` kernel (TASK-0001)

Relocate per the env-ownership procedure (build → archive any existing → symlink into meta →
verify). Build the **existing** kernel; do not regenerate it.

1. **Build release.** From the kernel repo:
   ```bash
   ( cd "$META_ROOT/handoff" && cargo build --release -p hf )   # adjust -p to the actual bin crate
   ```
2. **Archive any existing binary.** If an `hf` is already installed, move it aside (timestamped)
   rather than clobbering, so a bad build is reversible.
3. **Install onto PATH (symlink into meta).** Symlink the freshly-built
   `$META_ROOT/handoff/target/release/hf` to the meta-owned bin dir on PATH (the same place the
   other meta tools are exposed by bare name — do **not** hardcode `~/.local`; follow how meta
   exposes its binaries). Symlink, don't copy, so rebuilds propagate.
4. **Verify install:** `hf --help` (or bare `hf` prints the usage line) resolves to the new binary;
   `command -v hf` points at the meta-owned symlink.

Acceptance: `hf` is on PATH, runs, and the residency guard passes from `$META_ROOT`.

## Step 2 — Seed the envctl Tier-A `.handoff` layer (TASK-0002)

> **Run verbs against the surface they own** (residency guard). Envctl-member verbs update envctl's
> per-repo JSONL ledger export and rendered text. Fleet verbs roll up or view cross-repo state. Binary
> `ledger.db` caches remain ignored unless the handoff kernel changes that contract.

1. **Init / seed the ledger & layout.**
   ```bash
   ( hf init )    # creates ledger + tasks/ + packets/ + context/ (idempotent)
   ( hf import || true )  # rebuild local cache from committed JSONL when present
   ( hf seed )    # seeds kernel HFTASK-#### cards into the ledger
   ```
   `hf init` creates the schema/dirs; `hf seed` writes the kernel's own bring-up cards.
2. **Render the Tier-A surface — NEVER hand-write packets.** The kernel **renders**
   `packets/latest.md` and `active.md` from the witnessed ledger (`hf handoff` writes both). Render,
   don't author:
   ```bash
   ( hf handoff )  # renders .handoff/packets/latest.md + .handoff/active.md (handoff.packet.v2)
   ( hf export )   # writes committed .handoff/ledger.events.jsonl from the local ledger cache
   ```
   Author/maintain only the **policy text** layer (`policy.toml`, `hooks/`, `policies/`, `skills/`)
   — the declarative inputs — and let the kernel render the **state** layer (`active.md`,
   `packets/latest.md`). A hand-edited packet is a conformance failure (the p7 gate catches it).
3. **Commit the portable continuity layer.** The per-repo `.handoff` committed to git includes the
   JSONL ledger export plus rendered text (`policy.toml`, `hooks/`, `policies/`, `active.md`,
   `packets/latest.md`, `skills/`, `tasks/*.task.json`). Binary `ledger.db`/RVF caches remain ignored
   unless the handoff kernel changes that committed contract. Re-run the residency fail-closed check
   after seeding.

## Step 3 — Mint the backlog as task cards (TASK-0002, cont.)

**Cards are minted by the kernel, never hand-written.** Use `hf task mint` (shipped form:
`hf task mint --from-kb <SLUG>`) to emit `handoff.task.v1` cards into `.handoff/tasks/*.task.json`.
Each card MUST conform to `schemas/task.schema.json` (`$id: handoff.task.v1`):

- Required: `schema` (const `handoff.task.v1`), `id` (`^TASK-[0-9]{4,}$`), `title`, `status`
  (`backlog|active|claimed|blocked|checkpointed|review|done`), `priority` (`P0..P3`), `objective`,
  `path_scope` (≥1), `acceptance_criteria` (≥1), `test_commands`.
- Optional: `dependencies`, `blocked_by`, `allows_network`, `allows_dependency_addition`.

> **Kernel seed cards are `HFTASK-####`, not `TASK-####`, and carry replay fields**
> (`correlation_id`, `role`, `intent_lock`). These are the ledger's own bring-up cards — **omitting
> the replay fields breaks ledger replay** (the witness chain can't re-derive status). Mint kernel
> cards via `hf seed`; mint the envctl/Epic backlog cards (`TASK-0001..`) via `hf task mint` so they
> carry the same envelope. **Never hand-edit a card's status** — status is replayed from the ledger;
> hand edits desync from witnessed truth (use `hf sync-cards` to re-derive cards from the ledger).

Map the consolidated Epic A items onto cards: **TASK-0001** (build/install hf), **TASK-0002** (seed
Tier-A + mint cards), **TASK-0003** (p7-conformance gate).

## Step 4 — Add the `p7-conformance` CI gate (TASK-0003)

Add a CI gate (`ci/gates/p7-conformance.sh`, wired into the workflow as a **required status check**)
that proves the Tier-A layer is kernel-conformant and fail-closed:

1. **Schema validation.** Validate every `.handoff/tasks/*.task.json` against
   `schemas/task.schema.json` (handoff.task.v1); validate the seeded capsule against its schema and
   `policy.toml` against the policy schema. Any non-conforming card/capsule fails the gate.
2. **Packet conformance.** `( hf import || true; hf resume --json )` MUST emit a
   `"schema": "handoff.packet.v2"` object with `next_task_id`/`next_command`; `packets/latest.md`
   and `active.md` MUST carry the hf-rendered v2 header and must **never** be harness-edited
   (compare against a fresh `hf import && hf handoff && hf export` render — drift = a hand edit = FAIL).
3. **Residency invariant (fail-closed).** Assert the committed ledger export is present when expected,
   and any binary `ledger.db` cache under `envctl/.handoff` is either ignored or intentionally tracked
   by a newer handoff contract.
   A tracked binary ledger/cache fails the gate hard; a gitignored local per-repo ledger cache is legitimate (ADR-0004/ADR-0018).

Acceptance: the gate is green on a correctly-seeded tree and red on (a) a non-schema card,
(b) a hand-edited packet, or (c) an unignored binary `ledger.db` cache.

---

## Done criteria (Epic A complete)
- `hf` on PATH, verified; residency guard passes in the member repo and fleet status remains healthy.
- `.handoff` Tier-A portable layer seeded & committed (JSONL/rendered text; binary caches ignored); `packets/latest.md`
  + `active.md` are hf-rendered, not hand-written.
- Backlog minted as `handoff.task.v1` cards (kernel seed cards `HFTASK-####` carry replay fields).
- `p7-conformance` gate added and green (schema + packet-v2 + residency).
- The loops' hf-aware branch (see `continuity-steward`, `session-relay`, `forge-loop`) is now
  unlocked: canonical checkpoint = `hf checkpoint`, canonical packet = `hf handoff`.

## Error handling
- **Residency guard fails** (a binary `.handoff/ledger.db` cache is unignored or unintentionally tracked): stop; fix ignore/tracking
  it, re-run from `$META_ROOT`; repair via `hf gitignore --repair`, re-export/import, and keep the loops on fallback until clean.
- **`hf` build fails:** keep the archived prior binary; do not symlink a broken build; the harness
  stays on the hand-written HANDOFF path (no regression).
- **A packet was hand-edited:** re-render via `hf handoff` (never patch the markdown); the p7 gate
  must pass against the fresh render.
