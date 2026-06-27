# filesystem-layout findings — TARGET=icm (cycle 7)

Axis: `filesystem-layout`. Repo audited: `/home/drdave/Desktop/meta/icm` (read-only).
Frame: meta = ONE converging system; north-star residency = `$META_ROOT` + handoff. icm = the
persistent-memory organ. envctl/meta invariant under test: everything meta uses installs INTO meta
(`meta/.toolchains/`, `$META_ROOT`); no system-depth or user-global installs; global paths
(`~/.local/bin`, `~/.config`) hold only symlinks pointing INTO meta — never real state.

Headline verdict: icm is a clean, idiomatic Cargo workspace (repo-native layout = OK), but its
**entire runtime data plane lives user-global under XDG** (`~/.local/share/icm`, `~/.config/icm`,
`~/.cache/icm`) and its **binary installs user-global to `~/.local/bin`** — neither is meta-owned and
neither is a symlink into meta. This is the central convergence gap: the memory plane's data residency
diverges from handoff's `$META_ROOT` residency. Good news / migration lever: DB, config, and cache are
ALL already redirectable via env/config (`ICM_CONFIG`, `[store] path`, `--db`, `XDG_*` honored by the
`directories` crate), so envctl can point icm at a meta-owned path with **zero icm code change**.

Two concrete tracked-file drifts: a live runtime lock and a generated web build are committed to source.

---

## 1. path inventory

| path | kind | owner | mutability | git | evidence |
|------|------|-------|-----------|-----|----------|
| `Cargo.toml`, `Cargo.lock` | manifest/generated | repo | mutable | tracked | root; `[workspace]` 4 members (`Cargo.toml:1-8`) |
| `crates/icm-core/src` | source | repo | mutable | tracked | core types/embedder; `crates/icm-core/Cargo.toml` |
| `crates/icm-store/src` | source | repo | mutable | tracked | SQLite store; `rusqlite` bundled (`Cargo.toml:19`) |
| `crates/icm-mcp/src` | source | repo | mutable | tracked | MCP stdio server crate |
| `crates/icm-cli/src` | source | repo | mutable | tracked | CLI + hooks + host-injectors; `main.rs` ~8.6k lines |
| `crates/icm-cli/tests` | test | repo | mutable | tracked | repo-native test surface (OK) |
| `crates/icm-cli/web/src` | source (TS/Svelte) | repo | mutable | tracked | dashboard UI subproject |
| `crates/icm-cli/web/dist/` | generated artifact | repo | regenerated | **TRACKED** | 39 files incl. `dist/_app/...`; embedded via `rust-embed = "8"` (`Cargo.toml:74`) despite `web/.gitignore` listing `dist/` |
| `crates/icm-cli/web/bun.lock`, `package.json`, `vite.config.ts` | manifest (bun/vite) | repo | mutable | tracked | polyglot toolchain inside a Rust crate |
| `config/default.toml` | config (template) | repo | static | tracked | shipped defaults + path doc (`config/default.toml:1-12`) |
| `docs/*.md` | doc | repo | mutable | tracked | architecture/features/guide/integrations/product |
| `scripts/bench-*.sh` | script | repo | mutable | tracked | shell benches (repo-native) |
| `scripts/bench-agent-sim.ts`, `bench-quality.ts` | script (TS) | repo | mutable | tracked | polyglot in a Rust meta member |
| `scripts/bench-longmemeval.py` | script (Python) | repo | mutable | tracked | Python in a Rust-native meta member |
| `plugins/opencode-icm.ts` | source (TS) | repo | mutable | tracked | opencode host plugin payload |
| `assets/banner.png` | artifact | repo | static | tracked | README banner |
| `install.sh`, `install.ps1` | script (installer) | repo | static | tracked | downloads GitHub release → `~/.local/bin` (`install.sh:17`) |
| `README.md` + 12× `README_<locale>.md` | doc | repo | mutable | tracked | locale sprawl at root |
| `LICENSE`, `release-please-config.json`, `.release-please-manifest.json` | manifest/doc | repo | static | tracked | release tooling |
| `.rtk/filters.toml` | config | repo | static | tracked | rtk filter config |
| `.github/workflows/*.yml` | config (CI) | repo | mutable | tracked | ci/cd/release |
| `.claude/settings.json`, `.claude/rusty-idd-adapter.md` | config | repo | mutable | tracked | repo-local agent config (OK) |
| `.claude/scheduled_tasks.lock` | runtime/state | tool | **mutable at runtime** | **TRACKED** | live lock: `{"sessionId":...,"pid":51325,"procStart"...,"acquiredAt":1777721218484}` committed to source |
| `target/` | generated | cargo | ephemeral | ignored | `.gitignore:1` `/target` (OK) |
| `*.db`, `*.db-shm`, `*.db-wal` | state | runtime | ephemeral | ignored | `.gitignore:2-4` (OK — keeps DB out of repo) |
| `.env` | secret | user | mutable | ignored | `.gitignore:5` (OK) |
| `.fastembed_cache/` | cache | runtime | ephemeral | ignored | `.gitignore:6` — CWD fallback safety net |

Runtime-created paths (NOT in repo; resolved by code — the data plane):

| runtime path (Linux/XDG) | kind | owner | resolver evidence |
|--------------------------|------|-------|-------------------|
| `~/.local/share/icm/memories.db` | state (SQLite memory DB) | user-global | `default_db_path()` → `ProjectDirs::from("dev","icm","icm").data_dir().join("memories.db")` (`crates/icm-cli/src/main.rs:1041-1044`) |
| `~/.local/share/icm/install-manifest.json` | state | user-global | `install_manifest.rs:178-179` (`data_dir().join("install-manifest.json")`) |
| `~/.config/icm/config.toml` | config | user-global | `config_path()` → `config_dir().join("config.toml")` (`config.rs:299-301`) |
| `~/.config/icm/credentials` | secret | user-global | `web.rs:117-118` (`config_dir().join("credentials")`) |
| `~/.cache/icm/models` | cache (embedding models) | user-global | `cache_dir()` → `dirs.cache_dir().join("models")` (`crates/icm-core/src/fastembed_embedder.rs:14-16`) |
| `~/.cache/icm/extract-pending.lock` | runtime (flock) | user-global | `acquire_extract_lock()` → `XDG_CACHE_HOME`/`~/.cache` + `icm` (`main.rs:5183-5188`) |
| `~/.local/bin/icm` | toolchain (binary) | user-global | `install.sh:17` `INSTALL_DIR="${HOME}/.local/bin"` |
| `~/.claude`, `~/.gemini`, `~/.codex`, `~/.copilot` | config (host-injected) | other tools, user-global | `main.rs:3330-3333` via `cli_config_dir(env,default,home)` |
| `~/.config/{Code/User,amp,zed,opencode}` | config (host-injected) | other tools, user-global | `main.rs:3346-3486`, `uninstall/locations.rs:95-100,364-388` |

---

## 2. placement verdict

| surface | verdict | expected location | standard | citation |
|---------|---------|-------------------|----------|----------|
| `crates/<c>/src` + `crates/<c>/tests` | OK | as-is | Rust-Cargo | `Cargo.toml:3-8`; tests dir present |
| Workspace manifest + `Cargo.lock` at root | OK | as-is | Rust-Cargo | `Cargo.toml:1` |
| `config/default.toml` (template, not live config) | OK | as-is | repo-local | `config/default.toml:1-7` documents real XDG path |
| `docs/` | OK | as-is | repo-local | `docs/*.md` |
| `assets/banner.png` | OK | as-is | repo-local | single static asset |
| `target/`, `*.db*`, `.env`, `.fastembed_cache/` ignored | OK | as-is | Rust-Cargo/XDG | `.gitignore:1-6` |
| icm registered as meta peer | OK | as-is | envctl-meta | `/home/drdave/Desktop/meta/.meta.yaml:189-191` (`provides: [icm]`) |
| **`memories.db` → `~/.local/share/icm`** | **OWNER-WALL** | meta-owned data root (e.g. `$META_ROOT/.state/icm/memories.db` or `meta/.toolchains/icm/data/`) symlinked/redirected; bare `~/.local/share` only as symlink INTO meta | envctl-meta (vs XDG default) | `main.rs:1041-1044`; invariant: no user-global real state |
| **`~/.config/icm/{config.toml,credentials}`** | **OWNER-WALL** | meta-owned config root; `~/.config/icm` = symlink into meta | envctl-meta | `config.rs:299-301`, `web.rs:117-118`; `credentials` is a secret written user-global |
| **`~/.cache/icm/models` + `extract-pending.lock`** | **OWNER-WALL** | meta-owned cache root; honors `XDG_CACHE_HOME` so envctl can redirect | envctl-meta | `fastembed_embedder.rs:14-16`, `main.rs:5183-5188` |
| **`install.sh` → `~/.local/bin/icm`** | **PROPOSE** | build/install INTO `meta/.toolchains/icm/bin/`; `~/.local/bin/icm` = symlink into meta (envctl owns preview/apply/lock/rollback/parity) | envctl-meta | `install.sh:17,142,175`; pulls GitHub release, not envctl-managed |
| `install.sh --dir /usr/local/bin` (optional) | OWNER-WALL | never system-depth unless envctl owns it | FHS/envctl-meta | `install.sh:36,134` |
| **host-injection into `~/.claude`/`~/.gemini`/`~/.codex`/`~/.config/*`** | **OWNER-WALL** | writes to other tools' user-global config; acceptable as designed product behavior but unmanaged w.r.t. meta — must be envctl-aware (honors `CLAUDE_CONFIG_DIR` etc. already) | XDG/envctl-meta | `main.rs:3330-3333,3346-3486`; env overrides at `cli_config_dir` (`main.rs:4629`) |
| **`.claude/scheduled_tasks.lock` tracked** | **DRIFT** | gitignore it (runtime lock, never source) | repo-local | live PID/session JSON committed; mixes runtime state into tracked source |
| **`crates/icm-cli/web/dist/` tracked** | **DRIFT (qualified)** | generated build output; either ignore + build in CI before `rust-embed`, or document `dist/` as an intentional vendored-artifact surface | Rust-Cargo/repo-local | tracked despite `web/.gitignore` `dist/`; embedded at compile time via `rust-embed` (`Cargo.toml:74`) — intentional but undocumented + contradicts its own ignore file |
| `scripts/*.py`, `scripts/*.ts` | LEGACY-COMPAT | benches may stay, but Python in a Rust-native meta member is convention drift; prefer Rust/shell or quarantine under `scripts/bench/` with a documented polyglot exception | repo-local | `scripts/bench-longmemeval.py`, `bench-agent-sim.ts`; meta convention is Rust-native, no Python |
| `crates/icm-cli/web` bun/vite/svelte toolchain | LEGACY-COMPAT | dashboard UI is inherently TS; acceptable but is a second toolchain inside the Rust workspace — confine + document | repo-local | `web/bun.lock`, `web/package.json` |
| 13× `README_<locale>.md` at root | UNKNOWN→OK | root locale READMEs are a common GitHub convention; optional move to `docs/i18n/` | repo-local | root listing |

---

## 3. boundary map

```
repo-local (tracked, /home/drdave/Desktop/meta/icm)
  crates/{icm-core,icm-store,icm-mcp,icm-cli}/src   OK  Rust-Cargo
  crates/icm-cli/{tests, web/src}                   OK / web=TS subproject
  config/default.toml (TEMPLATE only)               OK
  docs/, assets/, scripts/, plugins/, README*       OK / scripts polyglot drift
  install.sh|ps1, Cargo.*, release-please*          OK
  .claude/scheduled_tasks.lock  <<< runtime lock leaked into source (DRIFT)
  crates/icm-cli/web/dist/      <<< generated build leaked into source (DRIFT-q)

meta-level ($META_ROOT) — north-star residency
  .meta.yaml registers icm (peer, provides:[icm])  OK
  >>> GAP: NO icm data/config/cache/binary lands here. icm has zero
      $META_ROOT / meta/.toolchains awareness (grep META_ROOT/.toolchains
      in crates/*/src + install.sh = 0 hits).

user-level (XDG, ~) — where icm ACTUALLY lives (WRONG boundary for meta)
  ~/.local/bin/icm                       binary (should be symlink INTO meta)
  ~/.local/share/icm/memories.db         the memory plane (should be meta-owned)
  ~/.local/share/icm/install-manifest.json
  ~/.config/icm/config.toml + credentials (secret!) 
  ~/.cache/icm/models + extract-pending.lock
  ~/.claude ~/.gemini ~/.codex ~/.copilot ~/.config/{Code,amp,zed,opencode}
      = host-injection targets (other tools' user-global config)

system-level (/usr, /etc, /var)
  none by default. install.sh --dir /usr/local/bin = optional, OWNER-WALL.
```

Wrong-boundary crossings (vs the envctl/meta invariant): the entire user-level block holds **real
state**, not symlinks into meta. Per the invariant the binary should sit in `meta/.toolchains/icm/`
with `~/.local/bin/icm` as a symlink into it, and the data/config/cache roots should resolve to a
meta-owned location. Convergence note: handoff keeps continuity state under `$META_ROOT`; icm keeps
memory state under `~`. For "ONE converging system" the two planes should share a residency root so a
clone of meta carries both the handoff ledger AND the memory DB.

---

## 4. UPGRADE rows — axis: filesystem-layout

```
UPGRADE[icm-fs-1] axis: filesystem-layout
  target_surface: ~/.local/share/icm/memories.db (the memory plane)
  evidence: crates/icm-cli/src/main.rs:1041-1044 default_db_path() -> ProjectDirs data_dir;
            no meta/$META_ROOT awareness (grep META_ROOT in crates/*/src = 0 hits)
  expected_location: meta-owned data root, e.g. $META_ROOT/.state/icm/memories.db (or
                     meta/.toolchains/icm/data/memories.db), with ~/.local/share/icm a symlink INTO meta
  migration_plan: envctl owns it. (a) envctl resolves a meta data root; (b) set ICM_CONFIG to a
                  meta-owned config.toml whose [store] path points there (config.rs:34, main.rs open_store
                  honors --db/cfg), OR pass --db; (c) one-time move of existing DB + manifest with envctl
                  preview/apply + lock + rollback; (d) parity: row-count + checksum of memories table
                  before/after. NO icm code change required — override surfaces already exist.
  acceptance: with envctl env active, `icm config show` + DB open resolve under $META_ROOT; a fresh meta
              clone on another host finds the same DB via the meta-owned path; nothing real written to
              ~/.local/share except a symlink into meta.
  risk_tier: PROPOSE
  reversibility: full — unset env/config restores XDG default; DB file is movable; backup before move.

UPGRADE[icm-fs-2] axis: filesystem-layout
  target_surface: ~/.config/icm/config.toml AND ~/.config/icm/credentials (secret)
  evidence: config.rs:299-301 (config.toml), web.rs:117-118 (credentials) -> ProjectDirs config_dir
  expected_location: meta-owned config root; ~/.config/icm = symlink into meta; credentials never
                     user-global plaintext outside meta's secret boundary
  migration_plan: ICM_CONFIG already overrides config path (config.rs:292). envctl sets ICM_CONFIG to a
                  meta-owned file; for credentials, redirect config_dir via meta-owned HOME/XDG shim or
                  add an env override (small code add) — flag to Feature-Forge as the one place lacking an
                  env escape hatch. envctl preview/apply/lock/rollback.
  acceptance: icm config + credentials resolve under meta; no plaintext secret lands in bare ~/.config.
  risk_tier: PROPOSE
  reversibility: full for config (env); credentials needs the env-override code add to be fully reversible.

UPGRADE[icm-fs-3] axis: filesystem-layout
  target_surface: ~/.cache/icm/models + ~/.cache/icm/extract-pending.lock
  evidence: fastembed_embedder.rs:14-16 (models), main.rs:5183-5188 (lock; already honors XDG_CACHE_HOME)
  expected_location: meta-owned cache root; envctl sets XDG_CACHE_HOME into meta
  migration_plan: lock already honors XDG_CACHE_HOME (zero code change); models uses directories
                  cache_dir which also follows XDG_CACHE_HOME on Linux -> envctl sets XDG_CACHE_HOME to a
                  meta-owned cache dir. Cache is regenerable so no parity move needed (REGENERATE).
  acceptance: with XDG_CACHE_HOME set into meta, models + lock land under meta; bare ~/.cache/icm empty.
  risk_tier: REGENERATE
  reversibility: full — cache is rebuildable from model downloads.

UPGRADE[icm-fs-4] axis: filesystem-layout
  target_surface: install.sh -> ~/.local/bin/icm (binary residency)
  evidence: install.sh:17,142,175 INSTALL_DIR=$HOME/.local/bin; no meta/.toolchains path
  expected_location: meta/.toolchains/icm/bin/icm built/installed by envctl; ~/.local/bin/icm = symlink INTO meta
  migration_plan: envctl owns install (build from this repo or pull pinned release INTO meta/.toolchains),
                  then symlink ~/.local/bin/icm -> meta. install.sh stays as the standalone/non-meta path.
                  envctl provides preview/apply/lock(pinned version)/rollback/parity(--version match).
  acceptance: `which icm` resolves to a symlink into meta; meta clone on a new host provisions icm via
              envctl with no curl|sh user-global install.
  risk_tier: PROPOSE
  reversibility: full — remove symlink, re-run install.sh for the standalone XDG layout.

UPGRADE[icm-fs-5] axis: filesystem-layout
  target_surface: .claude/scheduled_tasks.lock (tracked runtime lock)
  evidence: git ls-files shows it tracked; content is live PID/session JSON
            {"sessionId":...,"pid":51325,...,"acquiredAt":1777721218484}
  expected_location: not in git at all — runtime lock belongs in ignored runtime/cache surface
  migration_plan: add `.claude/scheduled_tasks.lock` (or `.claude/*.lock`) to .gitignore and
                  `git rm --cached` it. Pure repo hygiene; no production code.
  acceptance: file untracked; `git status` clean after a scheduled run writes the lock.
  risk_tier: APPLY
  reversibility: trivial — re-add if some tool truly needs it committed (it does not).

UPGRADE[icm-fs-6] axis: filesystem-layout
  target_surface: crates/icm-cli/web/dist/ (generated build, tracked, contradicts web/.gitignore)
  evidence: 39 tracked files under dist/; web/.gitignore lists dist/; rust-embed=8 (Cargo.toml:74)
            embeds it at compile time
  expected_location: either (a) keep tracked but DOCUMENT it as a vendored build artifact + remove `dist/`
                     from web/.gitignore so the repo is self-consistent; or (b) gitignore dist/ and build
                     the web bundle in CI/build.rs before rust-embed runs.
  migration_plan: decision for the architect. Lowest-risk: option (a) — drop the stale ignore line and add
                  a one-line note in docs/architecture.md. Cleaner: option (b) needs a bun build step in CI
                  (adds toolchain dependency to the build).
  acceptance: repo no longer both ignores and tracks dist/; build reproduces the embedded assets.
  risk_tier: PROPOSE
  reversibility: full (option a is doc-only).

UPGRADE[icm-fs-7] axis: filesystem-layout
  target_surface: scripts/bench-longmemeval.py (+ scripts/*.ts) — polyglot in a Rust-native meta member
  evidence: scripts/bench-longmemeval.py, bench-agent-sim.ts, bench-quality.ts tracked; meta convention is
            Rust-native (no Python — hub-registry-sync rule)
  expected_location: quarantine under scripts/bench/ with a documented polyglot exception, or port the
                     Python bench to Rust/shell.
  migration_plan: low priority; benches are not core. Document the exception in docs or move under
                  scripts/bench/. No production-code impact.
  acceptance: polyglot scripts are isolated + documented, or ported; meta Rust-native convention holds.
  risk_tier: PROPOSE
  reversibility: full.
```

---

## 5. Feature-Forge enforcement handoff (make drift FAIL in CI)

| check | kind | target | makes-fail-on |
|-------|------|--------|---------------|
| `gitignore_excludes_runtime_locks` | gate (script) | `.claude/*.lock` not tracked (`git ls-files | grep -q '\.lock$'` → fail) | re-committing a runtime lock (fs-5) |
| `web_dist_consistency` | gate | `dist/` must NOT be both in `web/.gitignore` AND tracked | self-contradictory ignore/track state (fs-6) |
| `data_residency_env_redirect` | integration test | run `icm` with `ICM_CONFIG` + `[store] path` set into a temp meta-owned dir; assert DB + config resolve there and NOT under `~/.local/share` / `~/.config` | regression of env-override capability that envctl relies on (fs-1, fs-2) |
| `cache_honors_xdg` | unit/integration | set `XDG_CACHE_HOME`; assert `cache_dir()` (fastembed_embedder.rs) + extract lock resolve under it | a future change hardcoding `~/.cache` (fs-3) |
| `no_unmanaged_user_global_default` | doctor (`icm doctor`/golden) | emit + snapshot the resolved data/config/cache/bin paths; golden-diff flags any new user-global write surface | new unmanaged global write path added silently |
| `host_injection_honors_env` | unit | already partly covered (`cli_config_dir_tests` main.rs:7961) — extend to assert every injector path honors its `*_CONFIG_DIR`/`*_HOME` override | an injector hardcoding `~/.claude` etc. (OWNER-WALL surfaces) |
| `installer_target_is_overridable` | gate (shellcheck/test) | assert `install.sh` `INSTALL_DIR` is overridable (`--dir`) so envctl can target meta | installer hardcoding a fixed user/system path (fs-4) |

Existing test seam to build on: `crates/icm-cli/src/main.rs:7961` `cli_config_dir_tests` already proves
env-override of host-config dirs — extend that pattern to the data/config/cache resolvers, which
currently have NO env-override test guarding the envctl redirect lever.

---

## Notes / non-findings (N/A)
- System-level (`/usr`,`/etc`,`/var`) writes: N/A — none by default; only optional `install.sh --dir`.
- Source/test layout: N/A as a finding — `crates/<c>/src` + `crates/<c>/tests` are idiomatic Cargo (OK).
- `.gitignore` for `target`/`*.db*`/`.env`/cache: N/A as a finding — correct (keeps state out of repo).
- Confidence: HIGH on residency facts (resolvers read directly at cited lines); MEDIUM on the web `dist/`
  intent (inferred from `rust-embed` + the ignore/track contradiction; architect to confirm the chosen option).
