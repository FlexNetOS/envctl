---
name: plan-filesystem-layout
description: >-
  Map and gate file/folder organization for a planning target against standard OS layout principles
  (FHS/XDG), repo-native conventions, and envctl/meta placement invariants. ALWAYS use when planning
  architecture, packaging, installation, runtime state, configs, caches, logs, generated artifacts, or
  repository layout. Produces cited filesystem-layout CLAIM/UPGRADE rows for planning-engineer.
---

# plan-filesystem-layout — standard OS file/folder organization axis

Use this method to make file/folder organization a first-class planning dimension instead of an
implicit afterthought. It maps the target's on-disk surfaces, classifies each path by purpose, checks
that placement against the relevant standard or repo convention, and produces enforceable upgrade rows.

## Baselines to apply

- **Standard OS layout:** FHS 3.0 for Unix-like system paths: static host config belongs under config
  surfaces, variable state under state surfaces, cache under cache surfaces, logs under log surfaces,
  runtime/lock/socket material under runtime surfaces, executables under bin/libexec-style surfaces,
  and architecture-independent data under share-style surfaces.
- **User/app layout:** XDG Base Directory concepts for user-scoped config/data/state/cache/runtime.
- **envctl/meta invariant:** envctl installs tools into meta (`meta/.toolchains/`, `$META_ROOT`) and
  must not create unmanaged system-depth or user-global installs. Existing legacy locations may remain
  only as explicitly documented compatibility/migration surfaces with a Rust/meta-native replacement
  and parity proof.
- **Repo-native layout:** Rust source in `crates/<crate>/src`, integration tests in
  `crates/<crate>/tests`, docs in `docs/`, manifests in `manifest/`, scripts in `scripts/`, fixtures
  under test/fixture surfaces, generated/lock/cache artifacts under generated or ignored surfaces.

## Required map

Write `findings/filesystem-layout-<T>.md` with:

1. **Path inventory table** — path, kind (`source|test|doc|script|config|manifest|generated|cache|log|state|runtime|secret|toolchain|artifact`), owner, mutability, git-tracked/ignored, and evidence.
2. **Placement verdict table** — `OK|DRIFT|LEGACY-COMPAT|OWNER-WALL|UNKNOWN`, expected location,
   standard/convention used (`FHS|XDG|envctl-meta|Rust-Cargo|repo-local`), and citation.
3. **Boundary map** — repo-local vs meta-level vs user-level vs system-level; identify anything that
   crosses the wrong boundary.
4. **Upgrade rows** — `UPGRADE[...] axis: filesystem-layout` carrying `target_surface`, `evidence`,
   `expected_location`, `migration_plan`, `acceptance`, `risk_tier: APPLY|PROPOSE|REGENERATE`, and
   `reversibility`.
5. **Test/enforcement handoff** — exact checks Feature Forge should add (unit/golden/doctor/gate) so
   layout drift fails in CI instead of recurring.

## Enforcement rules

- **No silent root clutter.** New top-level files/dirs need an owner, purpose, and route into an
  accepted surface; otherwise mark `DRIFT`.
- **No mixed semantics.** Do not mix cache/state/config/log/runtime/generated files in source/doc/test
  directories unless the repo already documents that surface and the verifier confirms it.
- **No unmanaged global writes.** `/usr/local`, `$HOME`, `~/.config`, `~/.local`, `/etc`, `/var`, and
  systemd surfaces are PROPOSE/OWNER-WALL unless envctl owns the component and has an apply/preview,
  lock, rollback, and parity story.
- **Compatibility is not ownership.** A legacy path can be `LEGACY-COMPAT`, but the plan must name the
  canonical target path and migration/removal gate.
- **Read-only planning.** The planning loop maps and specifies tests/gates; Feature Forge implements
  moves or writes production code. The only permitted planning mutation remains additive RED tests.

## Output quality bar

Every finding cites a path and either a standard/convention or repo invariant. A generic "clean up
folders" recommendation is invalid; it must name the exact path, expected surface, migration path,
acceptance test, and reversibility.
