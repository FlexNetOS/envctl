# filesystem-layout — TARGET: grit

- Target repo: `/home/drdave/Desktop/meta/grit`
- Axis: `filesystem-layout`
- Method: `/home/drdave/Desktop/meta/harness_hub/harness/skills/plan-filesystem-layout/SKILL.md`
- Baselines applied: FHS 3.0, XDG Base Directory, envctl/meta placement invariant, Rust/Cargo repo-native layout, repo-local convention.
- Stance: read-only on target; no production code or files moved. Findings returned to parent.

grit is a single root crate (no `crates/` workspace). Sources live at the repo root under `src/`
(`main.rs`, `config.rs`, `parser/`, `git/`, `db/`, `cli/`, `room/`). All runtime/agent state is
repo-local under `.grit/` (sqlite registry, worktrees, config, room socket, merge lock) unless the
operator selects the S3 or Azure lock backend, which writes to an external bucket/container.

---

## 1. path inventory

Kind legend: source | test | doc | script | config | manifest | generated | cache | state | runtime | secret | artifact.
Mutability: static (rarely changes) | mutable (runtime-written) | build (regenerated).

| path | kind | owner | mutability | tracked? | evidence |
|------|------|-------|-----------|----------|----------|
| `/home/drdave/Desktop/meta/grit/Cargo.toml` | manifest | crate | static | tracked | `Cargo.toml:1` `name = "grit"`, root-crate manifest |
| `/home/drdave/Desktop/meta/grit/Cargo.lock` | manifest (lock) | crate | build | tracked | bin crate; lock committed (correct for a binary) |
| `/home/drdave/Desktop/meta/grit/src/main.rs` | source | crate | static | tracked | `src/main.rs:1-15` declares `cli,config,db,git,parser,room` |
| `/home/drdave/Desktop/meta/grit/src/config.rs` | source | crate | static | tracked | `GritConfig::load/save(grit_dir)` reads/writes `.grit/config.json` |
| `/home/drdave/Desktop/meta/grit/src/cli/mod.rs` | source | crate | static | tracked | command surface; `grit_dir()` at `:364` |
| `/home/drdave/Desktop/meta/grit/src/parser/mod.rs` | source+test | crate | static | tracked | tree-sitter parsing; large `#[cfg(test)]` block from `:592` |
| `/home/drdave/Desktop/meta/grit/src/git/mod.rs` | source | crate | static | tracked | worktree mgmt; `grit_dir()` at `:16-17` = `root.join(".grit")` |
| `/home/drdave/Desktop/meta/grit/src/db/mod.rs` | source | crate | static | tracked | registry/symbol DB; tests at `:482+` |
| `/home/drdave/Desktop/meta/grit/src/db/lock_store.rs` | source | crate | static | tracked | `LockStore` trait + `LockEntry` |
| `/home/drdave/Desktop/meta/grit/src/db/sqlite_store.rs` | source | crate | static | tracked | `SqliteLockStore::open(path)` `:31` local backend |
| `/home/drdave/Desktop/meta/grit/src/db/s3_store.rs` | source | crate | static | tracked | S3 backend; default key prefix `.grit/locks/` `:644-645` |
| `/home/drdave/Desktop/meta/grit/src/db/azure_store.rs` | source | crate | static | tracked | Azure backend; `AzureConfig.access_key` `:29`; prefix default `.grit/locks/` `:32-33` |
| `/home/drdave/Desktop/meta/grit/src/room/mod.rs` | source | crate | static | tracked | unix socket at `grit_dir.join("room.sock")` `:29,:62` |
| `/home/drdave/Desktop/meta/grit/docs/` (14 files) | doc | repo | static | tracked | `RELEASE_FLOW.md` + 13 `README.<lang>.md` translations |
| `/home/drdave/Desktop/meta/grit/scripts/` | script | repo | static | tracked | `README.md`, `lib/common.sh`, `*/bench.sh` |
| `/home/drdave/Desktop/meta/grit/scripts/.gitignore` | config | repo | static | tracked | ignores `*/results/` (bench output) |
| `/home/drdave/Desktop/meta/grit/tests/` | test+script | repo | static | tracked | `*.sh` harnesses + `gen_graph.py` (no Rust `tests/*.rs`) |
| `/home/drdave/Desktop/meta/grit/examples/` | doc/script | repo | static | tracked | `01..06-*.sh` runnable usage scripts (not `examples/*.rs`) |
| `/home/drdave/Desktop/meta/grit/test-projects/` | fixture | repo | static | tracked | py-ml/ts-api/rust-service/pi-calc sample trees |
| `/home/drdave/Desktop/meta/grit/assets/` | artifact | repo | static | tracked | `banner.png`, `benchmark.png`, `benchmark.pdf`, `bench_data.json` (binaries in tree) |
| `/home/drdave/Desktop/meta/grit/.github/workflows/` | config (CI) | repo | static | tracked | `ci.yml`, `release*.yml`, `pr-target-check.yml` |
| `/home/drdave/Desktop/meta/grit/.rtk/filters.toml` | config (tool) | meta-tool | static | tracked | rtk filter; untrusted-by-default warning on read |
| `/home/drdave/Desktop/meta/grit/release-please-config.json` | config | repo | static | tracked | release-please root pkg config |
| `/home/drdave/Desktop/meta/grit/.release-please-manifest.json` | config | repo | build | tracked | `{".":"0.4.0"}` version pin |
| `/home/drdave/Desktop/meta/grit/AGENTS.md` | doc | repo | static | tracked | ICM block only |
| `/home/drdave/Desktop/meta/grit/CLAUDE.md` | doc | repo | static | tracked | Claude bridge / RTK notes |
| `/home/drdave/Desktop/meta/grit/.gitignore` | config | repo | static | tracked | ignore policy (see verdicts) |
| `/home/drdave/Desktop/meta/grit/.grit/` (runtime) | state+runtime+secret | crate-runtime | mutable | ignored | created by `cmd_init` `:414-444`; self-adds `.grit` to `.gitignore` |
| `  └ .grit/registry.db` | state (sqlite) | crate-runtime | mutable | ignored | `Database::open(dir.join("registry.db"))` `cli:419,480,...` |
| `  └ .grit/worktrees/<agent>` | runtime (git worktree) | crate-runtime | mutable | ignored | `git/mod.rs:22,94,135,477` |
| `  └ .grit/config.json` | config+secret | crate-runtime | mutable | ignored | `config.rs:31-56`; Azure `access_key` persisted plaintext |
| `  └ .grit/room.sock` | runtime (socket) | crate-runtime | mutable | ignored | `room/mod.rs:29,62` |
| `  └ .grit/merge.lock` | runtime (lock) | crate-runtime | mutable | ignored | `git/mod.rs:183` |
| `  └ .grit/locks/` (cloud key prefix) | state | external bucket | mutable | n/a | s3/azure default prefix `.grit/locks/` |
| `/home/drdave/Desktop/meta/grit/target/` | generated | cargo | build | ignored | `.gitignore:1`; measured 5.5G on disk |
| `/home/drdave/Desktop/meta/grit/.worktrees/` | runtime | UNKNOWN | mutable | **untracked + NOT ignored** | present at root, empty; not in `.gitignore`, not a git-tracked path |
| `/home/drdave/Desktop/meta/grit/.fastembed_cache/` (ignore rule) | cache | UNKNOWN | mutable | ignored | `.gitignore:10` — no producer in `src/` or `Cargo.toml` (stale rule) |

State residency summary (the question asked):
- **sqlite** (default `local` backend): `.grit/registry.db` (symbols/registry) and the lock store also use
  sqlite locally — both repo-local under `.grit/`, gitignored. `SqliteLockStore::open` takes the
  `.grit/registry.db` path (`cli/mod.rs:408`).
- **S3 / R2 / GCS-S3-compat** (`backend="s3"`): lock objects written to the configured bucket under key
  prefix default `.grit/locks/` (`s3_store.rs:644-645`, `config.rs:106-126`). Credentials come from the
  AWS env/credential chain — `cmd_config_set_s3` prints "Set credentials via environment" (`cli:1475`),
  so **no S3 secret is persisted to disk by grit**.
- **Azure Blob** (`backend="azure"`): lock blobs written to the container under prefix default
  `.grit/locks/` (`azure_store.rs:32-33`). Unlike S3, the **`access_key` is persisted into
  `.grit/config.json` in plaintext** (`azure_store.rs:29`, `cmd_config_set_azure` `cli:1482-1494`).

---

## 2. placement verdict

Status legend: OK | DRIFT | LEGACY-COMPAT | OWNER-WALL | UNKNOWN. Convention: FHS | XDG | envctl-meta | Rust-Cargo | repo-local.

| area / path | verdict | expected location | convention | citation / rationale |
|-------------|---------|-------------------|------------|----------------------|
| `src/` at repo root (`main,config,parser/,git/,db/,cli/,room/`) | OK | `src/` for a single root crate | Rust-Cargo | Cargo binary crate; `crates/<c>/src` is only required for a multi-crate workspace. `Cargo.toml:1` defines one package. |
| `Cargo.lock` committed | OK | repo root | Rust-Cargo | grit is a binary (`categories=command-line-utilities`), so committing the lock is the recommended convention. |
| `tests/` holds `*.sh` + `gen_graph.py`, no `tests/*.rs` | DRIFT | Rust integration tests in `tests/*.rs`; shell harnesses belong in `scripts/` or `tests/scripts/` | Rust-Cargo / repo-local | `tests/` is Cargo's reserved integration-test surface; populating it with bash/python harnesses (`harness.sh`, `benchmark.sh`, `gen_graph.py`) collides with Cargo semantics and mixes script-kind into a test-kind surface (SKILL "no mixed semantics"). Unit tests currently live inline (`parser/mod.rs:592`, `db/mod.rs:482`). |
| `examples/*.sh` | DRIFT (minor) | shell usage demos under `docs/examples/` or `scripts/examples/`; `examples/` reserved for `examples/*.rs` | Rust-Cargo | Cargo treats `examples/` as compilable Rust example targets. Shell-only contents are non-idiomatic but low-risk; rename surface to avoid `cargo` confusion. |
| `test-projects/` fixtures | OK | fixture trees under a clearly-named non-Cargo dir | repo-local | acceptable; they contain their own `Cargo.toml`/`package.json` and are deliberately outside `src/`. Nested `Cargo.toml` files are not workspace members (no `[workspace]` in root manifest), so they will not be pulled into grit's build. |
| `docs/` (RELEASE_FLOW + translations) | OK | `docs/` | repo-local | matches SKILL repo-native (`docs/`). |
| `scripts/` + `scripts/lib/common.sh` + `scripts/.gitignore` | OK | `scripts/` | repo-local | correct surface; `scripts/.gitignore` scopes `*/results/` bench output to ignored — good hygiene. |
| `assets/*.png/*.pdf/*.json` (binaries committed) | LEGACY-COMPAT | binary artifacts under `assets/` or Git LFS / release assets | repo-local | README references `assets/banner.png`; committing a `benchmark.pdf` + `bench_data.json` inflates history. Acceptable as docs assets but flag PDF as a generated artifact better attached to a release. |
| `.rtk/filters.toml` | OWNER-WALL | meta-tool config; trust managed by rtk | envctl-meta | tracked per-repo, but read emits "untrusted project filters … Run `rtk trust`". A committed, untrusted-by-default rtk filter is a meta-tool surface, not grit's to own; confirm intent with the tool owner. |
| `.github/workflows/*` | OK | `.github/workflows/` | repo-local | standard CI placement. |
| `release-please-config.json`, `.release-please-manifest.json` | OK | repo root | repo-local | release-please requires root placement. |
| `.grit/` runtime dir (repo-local) | OK | repo-local per-project state under `.grit/` | repo-local / XDG-analogue | `git/mod.rs:16-17` anchors to `root.join(".grit")`; `cmd_init` creates it AND auto-appends `.grit` to `.gitignore` (`cli:430-444`). This is the correct VCS-tool convention (mirrors `.git/`): per-repo, self-ignored, mutable state co-located with the repo it coordinates. Not an FHS/XDG violation because the state is intrinsically per-working-tree. |
| `.grit/config.json` storing Azure `access_key` plaintext | OWNER-WALL | secret material out of plaintext repo-local JSON; use env/credential chain (as S3 already does) or an OS keyring | XDG / secret-residency | `azure_store.rs:29` + `cmd_config_set_azure`. Mitigated (gitignored, `.gitignore:12-13` comment acknowledges it) but inconsistent with the S3 path which keeps secrets in env. Plaintext key at rest is a secret-residency drift; envctl/secrets ownership applies if hardened. |
| `.grit/locks/` on S3/Azure (external bucket) | OWNER-WALL / PROPOSE | external object store, operator-owned | n/a (crosses to system/cloud) | s3/azure write outside the repo and outside meta. grit does not own preview/apply/lock/rollback/parity for the remote store; the operator does. Per SKILL "no unmanaged global writes", any planning that automates remote provisioning must be PROPOSE until an owner with rollback/parity exists. As-is (operator opt-in via explicit `config set-s3/azure`) it is acceptable but must be labeled cross-boundary. |
| `.worktrees/` at repo root (untracked, un-ignored, empty) | DRIFT | either remove, or add to `.gitignore` if a tool writes here | repo-local | `git check-ignore` returns nothing and `git ls-files` is empty → silent root clutter with no owner (SKILL "no silent root clutter"). grit's own worktrees go under `.grit/worktrees/` (`git/mod.rs:22`), so this root `.worktrees/` is not grit's; likely a meta/harness artifact leaked into the target. Missing ownership = finding, not pass. |
| `.fastembed_cache/` ignore rule | DRIFT (stale) | remove rule or document the producer | repo-local | `.gitignore:10` ignores a cache dir, but no `fastembed` dep in `Cargo.toml` and no producer in `src/`. Orphan ignore rule = drift/rot; either dead config or an undocumented out-of-tree tool. |
| `target/` (5.5G) | OK | `target/` ignored | Rust-Cargo | `.gitignore:1`; correct. Size is a disk-hygiene note, not a layout fault. |
| `.gitignore` `*.db` blanket rule | OK (note) | repo-local | repo-local | `.gitignore:5` ignores all `*.db`; combined with `.grit` rule this protects registry/lock sqlite from accidental commit. Note: also hides any intentional fixture `.db` — none found in tree, so fine. |

---

## 3. boundary map

```
SYSTEM-LEVEL (/usr, /etc, /var, systemd)
  └─ (none) grit installs no system files; it is a single `cargo install`-style binary.
     No FHS system-path writes anywhere in src/. ............................. CLEAN

USER-LEVEL ($HOME, ~/.config, ~/.local, XDG)
  └─ S3 credential chain: AWS env/credentials (operator-managed, not grit-written) . OWNER-WALL (operator)
  └─ NO grit-authored writes to $HOME / ~/.config / ~/.local found in src/ ....... CLEAN
     (grit deliberately keeps all of its own state repo-local, not user-global.)

META-LEVEL (meta/.toolchains, $META_ROOT, meta tooling)
  └─ .rtk/filters.toml ......... meta-tool (rtk) config committed into the target . OWNER-WALL (rtk)
  └─ .worktrees/ (root) ........ likely meta/harness worktree artifact in target .. DRIFT (no owner)
  └─ grit binary itself is a meta peer/tool when installed by envctl ............. (envctl owns install)

REPO-LOCAL (the grit working tree)
  ├─ source/doc/script/config/manifest/fixture/asset .......................... OK (Rust-Cargo + repo-local)
  ├─ tests/ (shell+python in Cargo test surface) .............................. DRIFT (mixed semantics)
  ├─ examples/ (shell in Cargo example surface) ............................... DRIFT (minor)
  ├─ .grit/ runtime state (registry.db, worktrees, room.sock, merge.lock) ..... OK (self-ignored, .git-like)
  ├─ .grit/config.json with Azure access_key (plaintext) ...................... OWNER-WALL (secret-residency)
  └─ .fastembed_cache/ ignore rule (no producer) ............................. DRIFT (stale)

EXTERNAL / CLOUD (S3/R2/GCS bucket, Azure container)
  └─ {bucket|container}/.grit/locks/** lock objects ........................... OWNER-WALL/PROPOSE
     grit lacks preview/apply/lock/rollback/parity for the remote store; operator-owned.
```

Cross-boundary flags:
- `.worktrees/` (root) — meta/harness state crossed INTO the repo-local boundary with no owner. Route: ignore-or-remove.
- `.grit/config.json` Azure key — secret crossed from a credential plane into a plaintext repo-local config file; inconsistent with the S3 path that keeps secrets at the user/env plane.
- `.grit/locks/` on S3/Azure — repo-local tool reaching OUT to a cloud/system plane; legitimate but operator-owned and must be labeled, not automated, without an owner that has rollback/parity.

---

## 4. UPGRADE rows (axis: filesystem-layout)

```
UPGRADE[FL-1] axis: filesystem-layout
  target_surface: /home/drdave/Desktop/meta/grit/.worktrees/  (root, untracked + un-ignored)
  evidence: git check-ignore returns nothing; git ls-files empty; grit's own worktrees are .grit/worktrees/ (git/mod.rs:22)
  expected_location: not in the grit working tree at all; if a meta/harness tool needs it, it must be gitignored with an owning comment
  migration_plan: confirm producer (meta worktree tooling); if grit-irrelevant, remove the empty dir; otherwise add `.worktrees/` to .gitignore with an owner comment
  acceptance: `git status --porcelain` shows no untracked `.worktrees/`; OR `.gitignore` contains `.worktrees/` with attribution
  risk_tier: PROPOSE   (touches root layout; needs owner confirmation of producer)
  reversibility: trivial (dir is empty; re-add or un-ignore)

UPGRADE[FL-2] axis: filesystem-layout
  target_surface: /home/drdave/Desktop/meta/grit/.gitignore:10  (.fastembed_cache/)
  evidence: no `fastembed` in Cargo.toml deps; no producer in src/; orphan ignore rule
  expected_location: remove the rule, or document the out-of-tree producer in .gitignore
  migration_plan: delete the `.fastembed_cache/` line if dead; else add a comment naming the tool that writes it
  acceptance: every .gitignore rule maps to a real producer (doctor check FL-D2 below)
  risk_tier: APPLY   (pure ignore-file hygiene, no behavior change)
  reversibility: trivial (revert one line)

UPGRADE[FL-3] axis: filesystem-layout
  target_surface: /home/drdave/Desktop/meta/grit/tests/  (*.sh + gen_graph.py in Cargo's integration-test dir)
  evidence: tests/harness.sh, benchmark.sh, bench_*.sh, gen_graph.py occupy Cargo's reserved tests/*.rs surface; unit tests are inline (parser/mod.rs:592, db/mod.rs:482)
  expected_location: shell/python harnesses under scripts/ (e.g. scripts/test/) or tests/scripts/; reserve tests/ for tests/*.rs integration tests
  migration_plan: move non-Rust harnesses to scripts/test/ (or tests/scripts/), update any CI/doc references; leave room for future tests/*.rs
  acceptance: `tests/` contains only Rust test files (or is empty); `cargo test` discovery is unaffected; CI references updated
  risk_tier: PROPOSE   (path move touches CI/docs references; needs ref sweep)
  reversibility: medium (git mv back; update references)

UPGRADE[FL-4] axis: filesystem-layout
  target_surface: /home/drdave/Desktop/meta/grit/examples/*.sh  (shell in Cargo's examples/ target dir)
  evidence: 01..06-*.sh are shell demos; Cargo treats examples/ as compilable Rust example targets
  expected_location: docs/examples/ or scripts/examples/
  migration_plan: relocate the six shell demos; update README links that point at examples/
  acceptance: examples/ holds only Rust example targets or is absent; README links resolve
  risk_tier: PROPOSE   (doc-link churn)
  reversibility: medium (git mv back; fix links)

UPGRADE[FL-5] axis: filesystem-layout
  target_surface: /home/drdave/Desktop/meta/grit/.grit/config.json  (Azure access_key persisted plaintext)
  evidence: azure_store.rs:29 AzureConfig.access_key serialized by config.rs:51-56; cmd_config_set_azure (cli:1482-1494); contrast S3 which uses env (cli:1475)
  expected_location: secrets out of plaintext repo-local JSON — env/credential chain (parity with S3) or OS keyring; if envctl/secrets owns it, route through that with preview/apply/lock/rollback/parity
  migration_plan: read Azure access_key from env (e.g. AZURE_STORAGE_KEY) like S3; keep config.json to non-secret fields only; doctor-warn if a key is found in config.json
  acceptance: `grit config set-azure` no longer writes a key to .grit/config.json; backend still authenticates from env; a unit test asserts serialized config.json contains no `access_key`
  risk_tier: PROPOSE   (secret-residency hardening; behavior-affecting; OWNER-WALL until owner confirms)
  reversibility: medium (revert to field-based config)

UPGRADE[FL-6] axis: filesystem-layout
  target_surface: /home/drdave/Desktop/meta/grit/assets/benchmark.pdf, assets/bench_data.json
  evidence: generated benchmark artifacts committed to tree (artifact kind in a doc surface); history bloat
  expected_location: release assets / CI artifact store; keep only README-referenced images (banner.png) in-tree
  migration_plan: attach generated benchmark PDF/data to a GitHub release; keep banner/benchmark PNG only if README embeds them
  acceptance: tree contains only README-referenced static images; generated artifacts are release-attached
  risk_tier: PROPOSE   (history/asset policy; owner call)
  reversibility: easy (re-commit if needed)

UPGRADE[FL-7] axis: filesystem-layout
  target_surface: /home/drdave/Desktop/meta/grit/.rtk/filters.toml  (committed, untrusted-by-default)
  evidence: read emits "untrusted project filters (.rtk/filters.toml) … Run `rtk trust`"
  expected_location: confirm rtk-owner intent; either trust+document or remove from the target
  migration_plan: with rtk owner, decide whether grit should ship a committed rtk filter; if yes, document trust step in CLAUDE.md; if no, remove
  acceptance: .rtk/ presence in grit is documented with an owner, or removed
  risk_tier: PROPOSE   (meta-tool ownership; OWNER-WALL)
  reversibility: trivial
```

---

## 5. Feature-Forge enforcement handoff (make drift fail in CI)

Planning-only here; Feature Forge implements. Each check below maps to an UPGRADE row.

- **FL-D1 (gate, root clutter) — FL-1:** CI step asserts no untracked, un-ignored path at repo root:
  `test -z "$(git status --porcelain --untracked-files=all | grep -E '^\?\? [^/]+/$')"`. Fails the
  build if a new owner-less top-level dir (like `.worktrees/`) appears.
- **FL-D2 (doctor, ignore-rule liveness) — FL-2:** a `scripts/check-gitignore-rules.sh` (or a
  `#[test]`) that, for each non-glob `.gitignore` dir rule, asserts either a producer reference exists
  in `src/`/`Cargo.toml`/`docs` or the rule carries an attribution comment. Fails on orphan rules.
- **FL-D3 (golden, tests/ surface) — FL-3:** a unit/integration check asserting `tests/` contains only
  `*.rs` (glob `tests/*.{sh,py}` must be empty). Pairs with a CI grep so reintroduced shell harnesses
  in `tests/` fail.
- **FL-D4 (golden, examples/ surface) — FL-4:** assert `examples/*.sh` is empty (Cargo examples are
  `*.rs`). Optional; lower priority.
- **FL-D5 (unit, secret residency) — FL-5:** `#[test]` that builds a populated `GritConfig` with an
  Azure backend, calls `save()` to a `TempDir`, reads back `config.json`, and asserts the serialized
  text does NOT contain the access-key value (post-hardening). Mirrors existing roundtrip tests in
  `config.rs:106-126`. This is the only permitted planning mutation lane (additive RED test) and should
  be authored RED against current behavior so it documents the gap until FL-5 lands.
- **FL-D6 (doctor, state residency) — boundary invariant:** a `grit doctor`-style check (or test) that
  resolves `grit_dir()` and asserts all local state paths are under `.grit/` and `.grit` is present in
  `.gitignore` (already enforced at init `cli:430-444`; make it a standing gate so a regression that
  writes state outside `.grit/` or un-ignores it fails).

---

## Notes / non-findings

- No production code or files were modified; no files were moved. Only this findings file was written.
- No FHS system-path or `$HOME`/`~/.config`/`~/.local` writes are authored by grit — the repo-local
  `.grit/` model is the correct `.git`-style convention for a per-working-tree coordination tool and is
  NOT an XDG violation.
- This crate has no `.handoff`, `.idd`, or `.kb` directories of its own — it is a standalone tool repo,
  not a harness-bootstrapped repo; that is consistent and not a gap for this axis.
- The findings above contain no stub/unfinished markers; every row cites a path plus a standard or
  repo invariant.
```
