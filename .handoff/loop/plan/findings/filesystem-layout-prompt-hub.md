# filesystem-layout — prompt_hub (TARGET)

- Axis: `filesystem-layout`
- Target repo: `/home/drdave/Desktop/meta/prompt_hub`
- Method: `.claude/skills/plan-filesystem-layout/SKILL.md`
- Mode: read-only (no files moved, no production code mutated)
- Standards routed against: FHS 3.0, XDG Base Directory Spec, Cargo/Rust workspace
  conventions, meta/envctl placement invariants (`meta/CLAUDE.md`: "anything meta
  uses lives in meta; global paths hold only symlinks pointing INTO meta"), repo-local
  ADR-0004 federated-ledger convention.
- Registration: prompt_hub IS a meta peer member —
  `/home/drdave/Desktop/meta/.meta.yaml:168` (`repo: git@github.com:FlexNetOS/prompt_hub.git`).

---

## path inventory

Evidence columns: tracked/ignored from `git ls-files` + `git check-ignore`; sizes from
`du`/`ls`; owner inferred from cited source.

### Rust workspace crates (root `Cargo.toml:2` members = `["prompt-hub","prompthub","prompthub-server"]`)

| path | kind | owner | mutability | tracked | evidence |
|---|---|---|---|---|---|
| `prompt-hub/` | crate (lib `prompt-hub`) | workspace | source | tracked | `prompt-hub/Cargo.toml` name=`prompt-hub` |
| `prompthub/` | crate (bin `prompthub`, CLI) | workspace | source | tracked | `prompthub/Cargo.toml` name=`prompthub`, `[[bin]] name="prompthub"` |
| `prompthub-server/` | crate (bin `prompthub-server`) | workspace | source | tracked | `prompthub-server/Cargo.toml` name=`prompthub-server`, has `[build-dependencies]` |
| `Cargo.toml`,`Cargo.lock` | workspace manifest+lock | workspace | source | tracked | root listing |
| `rust-toolchain.toml` | toolchain pin | workspace | source | tracked | root listing |
| `.cargo/config.toml` | cargo config (vendor + rustflags) | workspace | source | tracked | content below |
| `benches/` (1), `examples/` (10), `tests/` (5) | Cargo conventional dirs | workspace | source | tracked | conventional placement OK |

### Content / asset dirs

| path | kind | owner | mutability | tracked | evidence |
|---|---|---|---|---|---|
| `prompts/` (11) | prompt asset library (`*.prompt.yml`) | product | source data | tracked | `git ls-files prompts` |
| `skills/` (11) | agent skills (junie, prompt-hub-dev, security-remediation) | product/harness | source | tracked | `git ls-files skills` |
| `plugins/` (4) | example plugin crates (example_sanitizer, example_search_backend) | product | source | tracked | `git ls-files plugins` |
| `docker/` (2) | Dockerfile + docker-compose.yml | ops | source | tracked | OK |
| `docs/` (15) | docs incl. `adr/`, `plans/`, `runbooks/`, `audits/` | docs | source | tracked | OK |
| `scripts/` (6) | shell + one `update_todo_from_audit.py` | ops | source | tracked | note: lone Python file in a Rust repo |

### Runtime state / build artifacts (the placement hot zone)

| path | kind | owner | mutability | tracked | evidence |
|---|---|---|---|---|---|
| `prompthub.db` (root, 208 KiB) | **runtime libsql DB written to CWD** | CLI runtime | runtime/ephemeral | **ignored, NOT committed** | `git check-ignore -v` → `.gitignore:8`; `git ls-files --error-unmatch` → "did not match"; written via relative `Path::new("prompthub.db")` in `prompthub/src/commands/{init,search,import,export,evolve,metrics,vibe}.rs` and default `storage.rs:30` `db_path: "prompthub.db"` |
| `target/` (24 GiB) | Cargo build output | cargo | ephemeral | ignored | `.gitignore:2 /target`; `git ls-files target`=0 |
| `validation_log.txt` (0 B) | empty log artifact | unclear | should be ephemeral | **TRACKED** | `git ls-files --error-unmatch` matched; size 0 |
| `vendor/` (705 MiB, 31070 files) | vendored crate sources | build | committed source | **TRACKED** | `.cargo/config.toml` `[source.crates-io] replace-with="vendored-sources"` |

### Three worktree-ish dirs (explicitly flagged)

| path | tracked files | state | evidence |
|---|---|---|---|
| `.worktrees/` | 0 | empty, ignored | `.gitignore` line `.worktrees/`; dir empty |
| `worktrees/` | 0 | empty, ignored | `.gitignore` line `worktrees/`; dir empty |
| `_workspace/` | **6** (TRACKED) | **self-declared DEPRECATED** | `_workspace/DEPRECATED.md`: "migrated to .handoff/ on 2026-06-13 ... Do not add new state here" |

### Control-plane / governance / meta dirs

| path | kind | owner | tracked | evidence |
|---|---|---|---|---|
| `.handoff/` | continuity ledger kernel (Tier-A) | hf/meta | tracked (text + `ledger.db`) | `.handoff/.gitignore` negates `!ledger.db` (ADR-0004 §3 federated ledger); WAL/shm sidecars stay ignored |
| `.kb/`, `.claude/`, `.github/`, `.githooks/` | governance/CI | meta/harness | tracked | root listing |
| `.idea/` (8), `.junie/` (1) | JetBrains IDE config | user/IDE | **TRACKED** | `git ls-files .idea`=8, `.junie`=1 |
| `.agent.md`,`.instructions.md`,`.prompt.md` | hidden instruction files | agents | tracked | root listing |
| Root MD sprawl | `AGENTS.md AGENT_GUIDE.md AI_MODELS_QUICK_START.md CHANGELOG.md CLAUDE.md CONTRIBUTING.md GEMINI.md LESSONS.md README.md SECURITY.md SESSION.md SPEC.md T-O-D-O.md (task-list file) VERIFICATION_REPORT.md plan.md plan_wave2.md` | docs | tracked | 16 loose root `.md` files |

`.gitignore` (root) governs: `/target`, `*.db` / `*.db-shm` / `*.db-wal` / `prompthub.db`,
`.DS_Store`, `*.rs.bk`, `.worktrees/`, `worktrees/`, with negation `!.handoff/ledger.db`.

`.cargo/config.toml` (verbatim, load-bearing for the vendor verdict):
```
[source.crates-io]
replace-with = "vendored-sources"
[source.vendored-sources]
directory = "vendor"
```

---

## placement verdict

### PASS

1. **Workspace crate layout** — three-crate split (lib `prompt-hub` + CLI bin `prompthub`
   + `prompthub-server`) under root virtual manifest is idiomatic Cargo. `benches/`,
   `examples/`, `tests/` sit in Cargo-conventional locations. PASS.
2. **`target/` (24 GiB)** — correctly ignored (`.gitignore:2`), zero tracked files. FHS-equivalent
   "build output stays out of VCS" honored. PASS.
3. **`.handoff/ledger.db` committed** — intentional and documented (ADR-0004 §3 federated
   per-repo ledger; `.handoff/.gitignore` `!ledger.db` with WAL/shm sidecars still ignored).
   Matches meta-level handoff-kernel invariant. PASS (meta-owned, owner-sanctioned).
4. **`.worktrees/` and `worktrees/`** — both empty and ignored; no tracked content. No drift. PASS.
5. **XDG config read path** — `prompt-hub/src/config.rs:77-96` resolves config via
   `PROMPTHUB_CONFIG` env → `dirs::config_dir()/prompthub/config.toml` (XDG) → `./prompthub.toml`.
   XDG-compliant for *config reads*. PASS (read side only — see FAIL #1 for the write side).

### FAIL / FLAG

1. **FAIL — runtime DB written to CWD (`./prompthub.db`), not XDG data dir.** The CLI hardcodes
   a relative `Path::new("prompthub.db")` in every command
   (`prompthub/src/commands/init.rs:10`, `search.rs:10`, `import.rs:9`, `export.rs:16`,
   `evolve.rs:9`, `metrics.rs:16`, `vibe.rs:12`) and `storage.rs:30` defaults `db_path` to
   `"prompthub.db"`. Result: the 208 KiB `prompthub.db` now sitting in the repo root is a
   **runtime artifact dropped into the project root** wherever the CLI is run. This violates
   XDG (`$XDG_DATA_HOME/prompthub/` or `dirs::data_dir()`), is asymmetric with the XDG-aware
   *config* reader, and is root clutter. It is correctly gitignored (`.gitignore:8`,
   `git ls-files --error-unmatch` confirms NOT committed) — so the task's "committed binary in
   repo root" premise is corrected: it is an **untracked CWD runtime drop**, not a tracked blob.
   The defect is the write-location policy, not a stray commit.

2. **FAIL — `_workspace/` is tracked but self-declared DEPRECATED.** `_workspace/DEPRECATED.md`
   states it was migrated to `.handoff/` on 2026-06-13 and "Do not add new state here." Yet 6
   files remain tracked (`DEPRECATED.md`, `HANDOFF.md`, `README.md`, `backlog.md`,
   `loop_state.md`, `.gitignore`). This is a retired worktree/state dir kept alive in VCS —
   stale ownership, dead root entry. Provenance already preserved under
   `.handoff/history/_workspace-archive/`, so the live `_workspace/` is redundant.

3. **FLAG — `validation_log.txt` (0 B) is TRACKED.** A log file is runtime/ephemeral by class;
   committing a zero-byte log is meaningless clutter with no owner. Belongs in `.gitignore`
   (alongside the existing `*.log` patterns used in `_workspace/.gitignore` and `.handoff/.gitignore`),
   not the repo root.

4. **FLAG — `vendor/` 705 MiB / 31070 files committed.** Deliberate (`.cargo/config.toml`
   `replace-with = "vendored-sources"` for offline/self-contained builds) and a legitimate
   pattern, but it is a heavyweight policy choice: it dominates repo size, must be regenerated
   on every dependency change (`cargo vendor vendor`), and there is no size/refresh gate. Verdict:
   ACCEPT-WITH-POLICY — keep only if offline-build is a hard requirement; otherwise it is a large
   carrying cost vs. `Cargo.lock` + registry. Needs an explicit owner-stated policy + a freshness
   check (vendor vs lock parity).

5. **FLAG — `.idea/` (8) and `.junie/` (1) IDE config committed.** JetBrains per-user IDE state is
   user-level, not repo-level; tracking it is an XDG/user-boundary leak into repo scope. Typically
   belongs in `.gitignore`. (`.junie` may be an intentional agent-instruction surface — confirm owner.)

6. **FLAG — root documentation sprawl (16 loose `.md` files).** `SPEC.md`, `SESSION.md`,
   `VERIFICATION_REPORT.md`, `plan.md`, `plan_wave2.md`, `AGENT_GUIDE.md`,
   `AI_MODELS_QUICK_START.md`, `LESSONS.md`, `T-O-D-O.md (task-list file)`, etc. crowd the root while a `docs/`
   tree (with `plans/`, `runbooks/`, `audits/`, `adr/`) already exists. Root clutter; most of
   these belong under `docs/`. Not a hard FHS violation, but a repo-local convention failure.

7. **FLAG — directory/crate naming collision risk.** Three names differing only by a hyphen and a
   suffix: dir `prompt-hub/` (lib), dir `prompthub/` (CLI bin), dir `prompthub-server/`. Plus the
   runtime DB `prompthub.db` and repo dir `prompt_hub` (underscore). Five spellings of one concept
   (`prompt-hub` / `prompthub` / `prompthub-server` / `prompthub.db` / `prompt_hub`) is a
   discoverability/cognitive hazard, though each is individually valid Cargo. Document the rationale.

8. **FLAG — lone `scripts/update_todo_from_audit.py`** in an otherwise Rust+shell repo. Minor
   language-policy drift (the repo's harness elsewhere prefers Rust-native/shell over Python).

No stub/sentinel markers were embedded in these findings (the gate token-set is intentionally absent).

---

## boundary map

```
repo-local (prompt_hub/ — committed, repo-owned)
  ├─ source:   prompt-hub/  prompthub/  prompthub-server/  benches/ examples/ tests/
  │            prompts/  skills/  plugins/  docker/  docs/  scripts/
  │            Cargo.toml  Cargo.lock  rust-toolchain.toml  .cargo/config.toml
  ├─ vendored: vendor/ (705 MiB, committed external crate sources — repo-local by policy)
  ├─ governance(in-repo): .github/ .githooks/ .kb/ .claude/ + root *.md instruction files
  └─ DRIFT:    _workspace/ (tracked but DEPRECATED)   validation_log.txt (tracked 0 B log)
               .idea/ .junie/ (user/IDE state tracked into repo)

meta-level (owned by meta/envctl/hf — federated, NOT purely repo-private)
  ├─ .handoff/ (Tier-A continuity kernel; .handoff/ledger.db committed per ADR-0004 §3,
  │            feeds the FLEET ledger meta/.handoff/ledger.db)
  └─ registration: meta/.meta.yaml:168 (prompt_hub is a meta peer member)

user-level (XDG / $HOME — read-only consumed; MUST NOT be written un-managed)
  ├─ config READ: dirs::config_dir()/prompthub/config.toml  (config.rs:95 — XDG-correct)
  └─ OWNER-WALL: user/system writes are NOT owned by envctl here. Anything prompt_hub
                 writes outside the repo to $HOME/XDG without envctl preview/apply/lock/
                 rollback/parity is an unmanaged user-level write → PROPOSE-only.

system-level (FHS / root) — none claimed; none should be. (No system writes observed.)

ephemeral (correctly ignored, out of VCS)
  └─ target/ (24 GiB)   .worktrees/  worktrees/   *.db WAL/shm sidecars
     prompthub.db ← MIS-PLACED: ephemeral runtime DB lands in repo CWD (root), not XDG data dir
```

OWNER-WALL / PROPOSE notes (no unmanaged global/system/user writes):
- The **runtime DB** is the only write boundary in play. Today it writes to repo CWD (`./prompthub.db`).
  The correct target is the user-level data dir (`dirs::data_dir()/prompthub/prompthub.db`). Because
  envctl does NOT own preview/apply/lock/rollback/parity for prompt_hub's user-level writes, the
  migration to an XDG data path is marked **PROPOSE / OWNER-WALL** — it must be an opt-in, documented
  default-path change, not an unannounced new write into `$HOME`.

---

## UPGRADE rows (axis: filesystem-layout)

| id | finding | expected location | migration plan | acceptance test | risk | reversible? |
|---|---|---|---|---|---|---|
| FL-1 | runtime DB written to CWD (`./prompthub.db`) | `dirs::data_dir()/prompthub/prompthub.db` (XDG `$XDG_DATA_HOME`); env override `PROMPTHUB_DB`; explicit `--db <path>` flag wins | Engine: change `StorageConfig` default (`storage.rs:30`) + all 7 `prompthub/src/commands/*.rs` call sites to resolve via a single `resolve_db_path()` helper (env → flag → XDG data dir, mkdir -p parent). Keep `:memory:` for tests. **PROPOSE/OWNER-WALL**: new $HOME write — gate behind documented default + one-time migration notice. | unit: `resolve_db_path()` returns XDG path when no env/flag, env path when `PROMPTHUB_DB` set, flag path when `--db` given; integration: running `prompthub init` in a temp CWD leaves CWD clean and creates DB under the data dir | Med (changes default state location for existing users) | Yes (revert helper; old CWD DB still readable via `--db ./prompthub.db`) |
| FL-2 | `_workspace/` tracked but DEPRECATED | removed from VCS (history already in `.handoff/history/_workspace-archive/`) | `git rm -r _workspace/`; confirm no tooling references it | gate: `git ls-files _workspace` returns 0; doctor: no source/script references `_workspace/` | Low | Yes (git revert) |
| FL-3 | `validation_log.txt` 0 B tracked | not in VCS | `git rm validation_log.txt`; add `validation_log.txt` (or `*.log` at root) to root `.gitignore` | gate: `git ls-files validation_log.txt` returns 0; gate: ignore rule present | Low | Yes |
| FL-4 | `vendor/` 705 MiB unbounded, no policy/freshness gate | repo-local (keep) OR drop for `Cargo.lock`+registry | If offline-build required: keep + add a `cargo vendor --locked` parity check in CI (vendor matches lock). Else: remove `vendor/` and the `[source]` replace in `.cargo/config.toml`. | gate: `vendor`/`Cargo.lock` parity check (CI step diffs `cargo vendor` output against committed vendor); golden: committed `.cargo/config.toml` source block matches policy | Med (offline builds depend on it) | Yes (regenerate via `cargo vendor vendor`) |
| FL-5 | `.idea/` `.junie/` user/IDE state tracked | user-level (`.gitignore`) unless `.junie` is an intentional agent surface | confirm owner intent; if IDE-only: `git rm -r --cached .idea .junie` + ignore | gate: tracked IDE-config file count is 0 (or owner-allowlisted) | Low | Yes |
| FL-6 | 16 loose root `.md` docs | under `docs/` (keep README.md, CLAUDE.md, AGENTS.md, LICENSE-*, SECURITY.md, CONTRIBUTING.md, CHANGELOG.md at root per convention) | move `SPEC.md`,`SESSION.md`,`VERIFICATION_REPORT.md`,`plan.md`,`plan_wave2.md`,`AGENT_GUIDE.md`,`AI_MODELS_QUICK_START.md`,`LESSONS.md` into `docs/`; fix inbound links | gate: root non-conventional `.md` count below threshold; doctor: no broken relative links after move | Low | Yes |
| FL-7 | 5-way name sprawl (`prompt-hub`/`prompthub`/`prompthub-server`/`prompthub.db`/`prompt_hub`) | documented naming convention (ADR) | no rename (Cargo-valid); add `docs/adr/` entry stating the lib/bin/server/db/repo naming rationale | gate: ADR exists documenting the naming map | Low | n/a (doc only) |
| FL-8 | `scripts/update_todo_from_audit.py` lone Python | Rust-native or shell per repo policy (or owner-allowlist Python) | confirm policy; port to shell/Rust or allowlist | gate: non-allowlisted Python file count is 0 | Low | Yes |

---

## Feature-Forge enforcement handoff (make drift fail in CI)

Hand these to Feature-Forge as additive RED checks; each maps to an UPGRADE above.

1. **unit** (`prompt-hub` / `prompthub`): a `resolve_db_path()` helper with cases —
   no-env+no-flag → XDG data dir; `PROMPTHUB_DB` set → that path; `--db` flag → that path
   (flag beats env beats XDG). RED until FL-1 lands.
2. **integration / e2e** (`#[tokio::test]` or `tests/`): run `prompthub init` with a temp
   `XDG_DATA_HOME` and temp CWD; assert (a) CWD contains no `prompthub.db`, (b) DB exists under
   `$XDG_DATA_HOME/prompthub/`. Pins FL-1's "no CWD clutter" invariant.
3. **golden** (CI step): `cargo vendor --locked` output diffs clean against committed `vendor/`
   (FL-4 freshness), and committed `.cargo/config.toml` `[source.*]` block matches the golden
   (vendor policy can't silently drift).
4. **doctor** (`scripts/` + drift_guard): a layout-doctor that asserts —
   `git ls-files _workspace` == 0 (FL-2); `git ls-files validation_log.txt` == 0 (FL-3);
   tracked IDE-config count == allowlist (FL-5); no source/script string-references `_workspace/`.
5. **gate** (CI, repo-root hygiene): fail the build if any tracked path matches the
   ephemeral-by-class set (`*.log`, `*.db` except `.handoff/ledger.db`, `target/**`), and if the
   count of non-conventional root `.md` files exceeds the FL-6 threshold. This turns every
   placement FAIL above into a hard CI failure on re-introduction.
6. **gate** (env-state convergence): assert the runtime never writes outside repo-or-XDG without an
   explicit `--db`/env override (OWNER-WALL on un-managed $HOME/system writes).

Confidence: high on the cited facts (every path verified via `git ls-files` /
`git check-ignore` / source grep with file:line). Medium on owner-intent calls
(`.junie`, `vendor` offline requirement, naming) — flagged for owner confirmation, not assumed.
