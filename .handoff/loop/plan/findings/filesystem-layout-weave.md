# filesystem-layout — weave (cycle 4)

Axis: `filesystem-layout`. Target: **weave** A2A session mesh.
Code (READ-ONLY): `/home/drdave/Desktop/meta/.worktrees/plan-weave-red/weave`
Method: `.claude/skills/plan-filesystem-layout/SKILL.md` (FHS 3.0 / XDG Base Directory / Rust-Cargo / envctl-meta / repo-local).
Date: 2026-06-26. Verdict: **DRIFT (contained)** — dep hygiene + repo layout are clean and self-documented; the live concerns are (1) a 9,631-line `main.rs` god-file, (2) the interim 4-crate workspace that CLAUDE.md says must collapse to single-crate, (3) unmanaged user-global runtime writes (store DB / config / memory) with **no ADR** establishing them as an envctl-invariant exemption, and (4) two Python scripts in a Rust-native repo.

Context note: `reports/codemap-weave.md` was specified as read context but **does not exist** in either worktree (`find … -name codemap-weave.md` → 0 hits across `/home/drdave/Desktop/meta/.worktrees`). N/A — codemap absent; this audit was built from direct read-only inventory of the checkout.

Correction to KEY FACTS: `weave/src/main.rs` is **9,631 lines** (`wc -l`), not ~4,489 — the god-file concern is larger than stated.

---

## 1. Path inventory

| Path | Kind | Owner | Mutability | Tracked/Ignored | Evidence |
|------|------|-------|-----------|-----------------|----------|
| `Cargo.toml` (root) | manifest | repo | static | tracked | virtual workspace, `members = ["weave-core","weave-inject","weave-mcp","weave"]`, `resolver = "2"` |
| `Cargo.lock` | manifest/generated | cargo | regenerated | tracked (78.7K) | binary-producing workspace ⇒ lock is correctly committed |
| `rustfmt.toml`, `deny.toml` | config | repo | static | tracked | fmt + cargo-deny gate config at root (Rust-Cargo idiom) |
| `weave-core/` | source crate | repo | static | tracked | `src/{lib,config,model,store,store_libsql,memory,session,archive,export,llm,sign,webpolicy,testenv}.rs`; member at repo root (NO `crates/` subdir) |
| `weave-inject/` | source crate | repo | static | tracked | `src/{lib,inject}.rs` |
| `weave-mcp/` | source crate | repo | static | tracked | `src/{lib,mcp,http,dashboard,obscura}.rs` |
| `weave/` (bin) | source crate | repo | static | tracked | `src/{main,setup,session,provider_switch,backup,git,harness,slack,telegram,testenv}.rs`, `[[bin]] path = "src/main.rs"` |
| `weave/benches/weave_bench.rs` | bench | repo | static | tracked | `[[bench]] harness=false` (correct Cargo bench surface) |
| `weave/tests/{integration,prop,security}.rs` | test | repo | static | tracked | per-crate `tests/` (correct Cargo integration-test surface) |
| `docs/` | doc | repo | static | tracked | `ARCHITECTURE-GRAPHS.md`, `DIRECTORY-TREE.md`, `OPERATIONS.md`, `SECURITY.md`, `TESTING.md` (78.7K), `ROADMAP-v0.2/v0.3.md`, `SPEC-*.md`, `REPOWIRE-PARITY.md`, `MULTI-SURFACE-PARITY.md`, `FORMAT-session-export.md` |
| `scripts/supply_chain_audit.py` | script | repo | static | tracked | Python in a Rust-native repo (`git ls-files scripts/`) |
| `scripts/target_smoke.py` | script | repo | static | tracked | Python (24K) |
| `goal/repowire-local-source-parity-audit.md` | doc | repo | static | tracked | single goal/intent doc |
| `.github/workflows/` | config (CI) | repo | static | tracked | CI surface |
| `ARCHITECTURE.md` (126.1K) | doc | repo | static | tracked | root doc, very large |
| `CHANGELOG.md` (100.3K) | doc | repo | static | tracked | root doc, very large |
| `README.md` (67.7K) | doc | repo | static | tracked | root doc, very large |
| `CLAUDE.md` (16.1K), `CONTRIBUTING.md`, `LESSONS-LEARNED.md`, `LICENSE-*` | doc | repo | static | tracked | standard root docs |
| `.handoff/` | sidecar (control plane) | hf kernel | runtime+static mix | mixed | `context/ decisions/ hooks/ loop/ packets/ policies/ tasks/`, `policy.toml`, `README.md`; ADRs under `decisions/ADR-000{1..5}-*.md` |
| `.handoff/ledger.db`, `**/ledger.db`, `*.rvf`, `*.rvf.lock`, `ledger.redb.tmp`, `*.db-wal/-shm/.bak` | state/runtime | hf kernel | ephemeral | **ignored** | `.gitignore` (ADR-0004 §3: per-repo `.handoff` carries no binary ledger) |
| `.handoff/locks/`, `.handoff/run/`, `.handoff/workspaces/main/scratch/` | runtime/lock | hf kernel | ephemeral | **ignored** | `.gitignore` |
| `.claude/` | sidecar (agent cfg) | agent tooling | auto-generated | tracked | `agents/ commands/ homunculus/ skills/`, `ecc-tools.json`, `identity.json` |
| `.codex/` | sidecar (agent cfg) | agent tooling | auto-generated | tracked | `agents/`, `AGENTS.md`, `config.toml` |
| `.agents/` | sidecar (agent cfg) | agent tooling | auto-generated | tracked | `skills/` |
| `target/`, `/.weave-test/` | generated/cache | cargo/tests | ephemeral | **ignored** | `.gitignore` |
| `cc-switch-main.zip` | artifact (audit archive) | external | static | **ignored** ("do not delete") | `.gitignore`; not present in checkout (planned audit evidence) |
| **`~/.local/share/weave/messages.db`** (`$XDG_DATA_HOME/weave/messages.db`) | **state (the broker DB)** | weave runtime | runtime | not in repo (USER-level) | `config.rs:1006-1022` `default_db_path()`; override via `WEAVE_DB` (`config.rs:1086`) / config `db` field (`config.rs:1953-1958`) |
| **`~/.config/weave/config.toml`** (`$XDG_CONFIG_HOME/weave/config.toml`) | **config** | weave runtime | runtime | not in repo (USER-level) | `config.rs:1024-1031` `config_dir()`, `config_path()` |
| **`~/.config/weave/memory/{global,project,persona,orchestrator}/*.md`** | **state (agent memory)** | weave runtime | runtime | not in repo (USER-level) | `memory.rs:414-416` `config_memory_dir()` → `config_dir().join("memory")`; writers `memory_write` create parent dirs |

---

## 2. Placement verdicts

| Surface | Verdict | Expected location | Standard | Citation / rationale |
|---------|---------|-------------------|----------|----------------------|
| Crate members at repo root (no `crates/`) | **OK** | repo-root members OR `crates/<c>/` | Rust-Cargo | Both are accepted Cargo layouts; root `Cargo.toml` lists all 4 members; self-documented in `docs/DIRECTORY-TREE.md` ("High-level ownership graph"). No drift. |
| `src/` / `tests/` / `benches/` per crate | **OK** | `<crate>/src`, `<crate>/tests`, `<crate>/benches` | Rust-Cargo | `weave/tests/{integration,prop,security}.rs`, `weave/benches/weave_bench.rs` are canonical Cargo surfaces. |
| `Cargo.lock` committed | **OK** | repo root | Rust-Cargo | Workspace produces a binary (`[[bin]] weave`) ⇒ lock must be tracked. Present (78.7K). |
| Path dependencies | **OK (positive)** | intra-repo `../<crate>` only | Rust-Cargo / envctl-meta | All path deps are sibling crates (`weave-core = { path = "../weave-core" }`, etc., in all 4 `Cargo.toml`). **NO repo-escaping path deps** — a clean contrast to handoff's RuVector/envctl escapes. External deps are git (`fnx-classes/algorithms/runtime` from `github.com/Dicklesworthstone/franken_networkx`), not local paths. |
| `weave/src/main.rs` (9,631 lines) | **DRIFT** | decomposed: CLI dispatch in `main.rs`, subcommand logic in sibling modules (`session.rs`/`setup.rs`/`harness.rs` already exist) | repo-local / Rust-Cargo | A 9.6K-line bin entrypoint is a god-file: it is the workspace's #2 largest source file after `store.rs` (11,315). The repo already practices module extraction (10 sibling modules in `weave/src/`), so the entrypoint should be a thin dispatcher. Mixed-semantics concentration of CLI + hooks + bridges in one file. |
| Interim 4-crate workspace | **LEGACY-COMPAT** | single crate (deferred goal) | repo-local | `CLAUDE.md:17` + `ARCHITECTURE.md` ("`WL-001 workspace split`… **Single-crate remains the goal** — the workspace is kept as an *interim* state and will be collapsed back **after the meta workspace is aligned**; the `backup/*` tags on origin are retained for that collapse… do **not** add further crates"). The split is documented + owner-acknowledged as temporary ⇒ LEGACY-COMPAT, not OK; canonical target (single crate) and migration trigger (meta-workspace alignment) are both named. |
| `~/.local/share/weave/messages.db` (store / broker) | **OWNER-WALL** | XDG data dir is FHS/XDG-correct for a user CLI; but unmanaged w.r.t. envctl | XDG vs envctl-meta | XDG-default placement is *correct standard practice* (`config.rs:1006-1022`) and the DB is intentionally a machine-wide cross-session broker. **However**: it is an unmanaged user-global write under the meta invariant (global paths should hold only symlinks INTO meta), and **no ADR** (`/.handoff/decisions/ADR-000{1..5}`) documents it as an intentional user-scoped exemption. envctl does not own its preview/apply, lock, rollback, or parity. ⇒ OWNER-WALL/PROPOSE. |
| `~/.config/weave/config.toml` | **OWNER-WALL** | XDG config dir (correct) but unmanaged | XDG vs envctl-meta | Same as above (`config.rs:1024-1031`). User-global write, no envctl ownership, no exemption ADR. |
| `~/.config/weave/memory/**/*.md` | **OWNER-WALL** | XDG config (arguably should be `$XDG_DATA_HOME`/state, not config) | XDG vs envctl-meta | `memory.rs:414-416` roots agent memory under the **config** dir. XDG semantics: mutable, regenerable agent memory is *data/state*, not config — placing it under `$XDG_CONFIG_HOME` mixes config and state semantics. Plus the same unmanaged-user-write wall. |
| `scripts/*.py` (2 files) | **DRIFT** | Rust-native equivalent (e.g. `xtask`/cargo bin) or explicitly-documented exception | repo-local / meta Rust-native | `supply_chain_audit.py`, `target_smoke.py` are Python in a repo whose invariant is "one dependency-light **Rust** binary" (`CLAUDE.md:49`) within a meta workspace whose hub standard is Rust-native. `CLAUDE.md:49` does anticipate "files in other languages" from external tooling, which *qualifies* this — but does not name these two scripts; their owner/replacement is unstated. Missing ownership = finding, not pass. |
| Root docs >60K (`ARCHITECTURE.md` 126K, `CHANGELOG.md` 100K, `README.md` 67K) | **DRIFT (minor)** | `docs/` (where `TESTING.md`/`SECURITY.md`/`OPERATIONS.md` already live) | repo-local | The repo already routes large docs into `docs/`; three oversized docs remain at root. Not a correctness issue, but inconsistent with the repo's own `docs/` convention and inflates root clutter. README at root is standard; `ARCHITECTURE.md`/`CHANGELOG.md` at root are conventional too, so this is the weakest finding — flagged for size/consistency only. |
| `.handoff/` ignore rules | **OK** | binary ledger + locks + run + scratch ignored | repo-local (ADR-0004) | `.gitignore` correctly excludes `ledger.db`, `*.rvf[.lock]`, `ledger.redb.tmp`, WAL/SHM/bak, `locks/`, `run/`, `workspaces/main/scratch/`. State/runtime kept out of git as required. |
| `.claude/` + `.codex/` + `.agents/` | **OK (noted)** | three parallel agent-tooling sidecars | repo-local | `CLAUDE.md:49` documents these as auto-generated/auto-pushed control-plane bundles. Owner + purpose are named ⇒ OK. Noted: triple agent-config surface is a governance-axis concern (out of scope here), not a filesystem-layout drift. |
| `target/`, `/.weave-test/` ignored | **OK** | cargo/test cache | Rust-Cargo | Generated cache correctly ignored. |
| Logical "leases" | **OK (clarification)** | inside the store DB, not on-disk lockfiles | repo-local | `model.rs:1312-1359` (`lease_resource_valid`/`lease_path_normalize`/`lease_path_conflicts`) are *logical* resource leases persisted in the broker DB ("the DB is the broker"). They are **not** filesystem lockfiles. The only on-disk locks are the hf kernel's `.handoff/locks/` (ignored). `memory.rs` `FS_LOCK` is an in-process `Mutex`, not a file lock. ⇒ no unmanaged on-disk lock surface from weave runtime. |

---

## 3. Boundary map

```text
SYSTEM-LEVEL  ( /usr, /etc, /var, systemd )
  └─ (none) — weave writes nothing to system depth.  VERIFIED CLEAN.
     No /usr/local, /etc, /var, or systemd writes in config.rs/store.rs/memory.rs.

USER-LEVEL  ( $HOME, XDG )                                    << BOUNDARY CONCERN
  ├─ ~/.local/share/weave/messages.db   state  (broker DB)   [OWNER-WALL]
  ├─ ~/.config/weave/config.toml        config               [OWNER-WALL]
  └─ ~/.config/weave/memory/**/*.md     state  (mis-rooted in config) [OWNER-WALL]
     Redirectable via WEAVE_DB / XDG_DATA_HOME / XDG_CONFIG_HOME / config `db`.
     XDG-correct as a standard; unmanaged w.r.t. envctl; NO exemption ADR.

META-LEVEL  ( meta/.toolchains, $META_ROOT, .handoff fleet )
  └─ .handoff/ control plane (ledger ignored; ADRs/policies tracked)  [OK]
     weave is a registered hf fleet member; harness state stays in-repo+ignored.

REPO-LOCAL  ( the checkout )
  ├─ weave-core / weave-inject / weave-mcp / weave   Rust source   [OK]
  ├─ docs/ scripts/ goal/ .github/                                  [OK / DRIFT(py)]
  ├─ .claude/ .codex/ .agents/  agent sidecars (auto-gen)          [OK]
  └─ root docs (ARCHITECTURE/CHANGELOG/README oversized)           [DRIFT minor]
```

Boundary crossings flagged: the **user-level** trio crosses out of repo/meta boundaries into `$HOME` without envctl ownership or an exemption ADR. Everything else stays within its correct boundary. No system-level crossing exists.

---

## 4. UPGRADE rows (axis: filesystem-layout)

```
UPGRADE[WV-FSL-1] axis: filesystem-layout
  target_surface: weave/src/main.rs (9,631 lines, bin entrypoint)
  evidence: wc -l weave/src/main.rs = 9631; #2 largest in workspace; 10 sibling
            modules already exist in weave/src (proven extraction pattern)
  expected_location: thin dispatcher in main.rs; subcommand groups in sibling
            modules (e.g. cli_inbox.rs, cli_send.rs, cli_lease.rs, cli_setup
            already partly in setup.rs/session.rs/harness.rs)
  migration_plan: Feature Forge extracts cohesive command groups out of main.rs
            into siblings, leaving main.rs as clap dispatch + wiring; no behavior
            change; one group per PR to bound blast radius.
  acceptance: `wc -l weave/src/main.rs` below an agreed cap (e.g. <2000); all
            existing weave/tests/{integration,prop,security}.rs pass unchanged;
            `cargo build`/`clippy` clean.
  risk_tier: PROPOSE   (large refactor of the binary entrypoint; behavior-preserving)
  reversibility: high (pure code move; git revert restores; no on-disk format change)

UPGRADE[WV-FSL-2] axis: filesystem-layout
  target_surface: 4-crate interim workspace (weave-core/inject/mcp/weave)
  evidence: CLAUDE.md:17 + ARCHITECTURE.md — "single-crate remains the goal …
            interim … collapsed back after the meta workspace is aligned;
            backup/* tags retained"
  expected_location: single `weave` crate (deferred canonical target)
  migration_plan: DO NOT collapse now — gate is "meta workspace aligned". Record a
            tracked decision (ADR under .handoff/decisions/) capturing the trigger
            condition, the backup/* collapse tags, and the "do not add further
            crates" rule so the interim state is governed, not drifting.
  acceptance: an ADR exists naming the single-crate target, the collapse trigger,
            and the freeze on new crates; `Cargo.toml` members count == 4 (a gate
            test asserts no 5th member is added before collapse).
  risk_tier: PROPOSE   (collapse deferred; this row only governs the interim state)
  reversibility: high (documentation + a no-new-crate gate; no code move yet)

UPGRADE[WV-FSL-3] axis: filesystem-layout
  target_surface: user-global runtime writes — ~/.local/share/weave/messages.db,
            ~/.config/weave/config.toml, ~/.config/weave/memory/**
  evidence: config.rs:1006-1031 (default_db_path/config_dir), config.rs:1086
            (WEAVE_DB), memory.rs:414-416 (memory under config dir); NO ADR among
            .handoff/decisions/ADR-000{1..5} documents user-level residency
  expected_location: EITHER (a) an ADR explicitly classifying weave as a
            user-scoped machine-wide broker EXEMPT from the meta-residency
            invariant (XDG paths are correct for that role and the broker must be
            shared across worktrees) — preferred given "the DB is the broker";
            OR (b) envctl owns provisioning: store/config under meta with
            XDG_DATA_HOME/XDG_CONFIG_HOME/WEAVE_DB pointed into meta and global
            paths holding only symlinks INTO meta.
  migration_plan: write the exemption ADR (option a) OR add an envctl component
            with preview/apply, lock, rollback, parity (option b). Until one
            lands, this surface stays OWNER-WALL.
  acceptance: ADR present and linked from CLAUDE.md "Where things live"; OR envctl
            apply/preview test shows redirected residency + rollback parity.
  risk_tier: PROPOSE   (owner decision required — touches the broker rendezvous model)
  reversibility: high for (a) doc-only; medium for (b) (env redirection, reversible)

UPGRADE[WV-FSL-4] axis: filesystem-layout
  target_surface: ~/.config/weave/memory/** rooted under XDG_CONFIG_HOME
  evidence: memory.rs:414-416 config_memory_dir() = config_dir().join("memory");
            contents are mutable, regenerable agent memory (data/state, not config)
  expected_location: $XDG_DATA_HOME/weave/memory/ (state semantics) — co-located
            with messages.db, not under the config tree
  migration_plan: Feature Forge moves memory root to the data dir with a one-time
            read-fallback from the old config-dir location; documented in
            docs/OPERATIONS.md.
  acceptance: new writes land under $XDG_DATA_HOME/weave/memory; a golden test
            asserts config_memory_dir() resolves under the data dir; legacy path
            still readable during a deprecation window.
  risk_tier: PROPOSE   (changes a user-visible path; needs migration/fallback)
  reversibility: medium (path move with read-fallback; reversible by config)

UPGRADE[WV-FSL-5] axis: filesystem-layout
  target_surface: scripts/supply_chain_audit.py, scripts/target_smoke.py
  evidence: git ls-files scripts/ (both tracked Python); CLAUDE.md:49 anticipates
            non-Rust artifacts but does not name/own these two
  expected_location: Rust-native (cargo xtask / a small bin / CI step in Rust) OR
            an explicit, named exception in CLAUDE.md/CONTRIBUTING.md with an owner
  migration_plan: decide per script — port to xtask if it is a dev gate, or record
            it as an intentional external-tooling artifact with an owner and a
            no-runtime-Python guarantee.
  acceptance: either no `*.py` under scripts/ (a gate test asserts this), or each
            .py is enumerated as an owned exception in CONTRIBUTING.md.
  risk_tier: PROPOSE   (owner intent unknown — could be load-bearing CI)
  reversibility: high (port or document; no product-code impact)

UPGRADE[WV-FSL-6] axis: filesystem-layout
  target_surface: oversized root docs ARCHITECTURE.md (126K), CHANGELOG.md (100K)
  evidence: ls -la root; docs/ already holds TESTING.md/SECURITY.md/OPERATIONS.md
  expected_location: docs/ (consistent with the repo's own doc-routing convention)
  migration_plan: optional — relocate to docs/ and leave a root pointer, OR accept
            root placement as conventional (README/ARCHITECTURE/CHANGELOG at root
            is common). Lowest priority.
  acceptance: docs placement is internally consistent (decision recorded once).
  risk_tier: PROPOSE (cosmetic/consistency)
  reversibility: high (doc move + pointer)
```

---

## 5. Feature-Forge enforcement handoff (make drift fail in CI)

| Check | Kind | Asserts | Where |
|-------|------|---------|-------|
| `main_rs_line_cap` | gate (CI) | `wc -l weave/src/main.rs` ≤ cap (start at current, ratchet down per WV-FSL-1) | CI step / `xtask` |
| `no_new_crate` | gate (CI) | root `Cargo.toml` `members` length == 4 (freezes the interim split per WV-FSL-2) | CI step parsing Cargo.toml |
| `no_repo_escaping_path_dep` | unit/gate | every `path = "…"` dep in all 4 `Cargo.toml` is a sibling `../weave-*` (preserves the clean-dep-hygiene positive) | unit test over manifests |
| `db_path_is_xdg` | unit (golden) | `default_db_path()` == `$XDG_DATA_HOME/weave/messages.db` and honors `WEAVE_DB`/config `db` override | weave-core unit (extend existing config tests) |
| `config_dir_is_xdg` | unit (golden) | `config_dir()`/`config_path()` resolve under `$XDG_CONFIG_HOME/weave` | weave-core unit |
| `memory_dir_is_state` | unit | after WV-FSL-4, `config_memory_dir()` resolves under `$XDG_DATA_HOME` not `$XDG_CONFIG_HOME` | weave-core unit |
| `no_system_writes` | doctor/test | no write path resolves under `/usr`,`/etc`,`/var`,systemd (grep-gate over store.rs/config.rs/memory.rs create_dir_all/File::create targets) | `weave doctor` extension + CI grep |
| `residency_adr_present` | gate | an ADR in `.handoff/decisions/` documents weave's user-level residency exemption OR envctl ownership (closes WV-FSL-3 OWNER-WALL) | CI existence+link check |
| `no_untracked_root_clutter` | gate | no new top-level file/dir without an owner entry in `docs/DIRECTORY-TREE.md` legend | CI diff check against DIRECTORY-TREE.md |
| `scripts_rust_native` | gate | no `*.py` under `scripts/` unless listed as an owned exception (WV-FSL-5) | CI step |

`weave doctor` already exists and is the natural home for residency/path assertions (`default_db_path` was explicitly exposed "so diagnostics (`weave doctor`)… can warn when the *resolved* db_path points somewhere other than this well-known store" — `config.rs:1010-1012`); extend it with the system-write and XDG-residency checks above so drift surfaces both in CI and at runtime.

---

## Laws compliance

Read-only: no production code or files mutated; only this findings doc written. Fail-closed: every OWNER-WALL/DRIFT defaults to PROPOSE pending owner/envctl decision. Every row cites a path + standard/convention. No stub/unfinished tokens; absent inputs marked `N/A — <why>` (codemap report; residency ADR).
