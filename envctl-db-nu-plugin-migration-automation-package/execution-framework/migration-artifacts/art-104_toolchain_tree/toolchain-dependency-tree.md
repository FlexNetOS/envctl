# ART-104 Toolchain dependency tree

Generated at: `2026-07-04T23:22:31+00:00`
Status: `complete`
Target root: `/home/flexnetos/FlexNetOS`

## Summary

- Coverage: Tree is generated from target descriptor, package scan, envctl registry evidence, selected manifests, and capped filesystem discovery; node-level statuses identify weaker evidence surfaces.
- Manifest signals scanned: `1229` from `20001` visited files.
- Languages detected: `{"javascript-typescript": 176, "python": 42, "rust": 608}`.
- Package manager signals: `{"bun": 6, "cargo": 608, "container": 59, "nix": 3, "npm": 5, "pep517": 4, "pip": 38}`.
- Lockfile signals: `{"bun": 4, "nix": 3, "npm": 4}`.
- Envctl toolchain components captured: `12`.

## Dependency Tree

```text
FlexNetOS toolchain
|-- envctl component manifests
|   |-- Bun runtime
|   |   |-- bun / bunx
|   |   `-- npm / pnpm / yarn / npx frontdoors
|   |-- Rust toolchain
|   |   |-- cargo
|   |   |-- rustc
|   |   |-- clang + wild linker
|   |   `-- kache RUSTC_WRAPPER
|   |-- GitHub CLI
|   |-- Podman container runtime
|   `-- Codex / agent frontdoors
|-- Nix flake development shells
|-- GitHub Actions CI
|   |-- rustfmt
|   |-- clippy
|   |-- MSRV cargo check
|   |-- cargo audit
|   `-- test gates
`-- Deploy/control frontdoors
    |-- envctl
    |-- meta
    `-- git-kb
```

## Nodes

| id | kind | status | depends on | evidence |
|---|---|---|---|---|
| `root:flexnetos-toolchain` | root | partial | `provisioning:envctl-manifest`, `compiler:rust`, `runtime:bun`, `runtime:nix`, `ci:github-actions`, `deploy:frontdoors` | `generated/envctl_target_registry.json`, `generated/package_scan.json` |
| `provisioning:envctl-manifest` | toolchain-provisioner | evidenced | `runtime:nix`, `package-manager:cargo`, `package-manager:bun` | `src/envctl/manifest/base.toml`, `src/envctl/manifest/components.d/epic-h-toolchains.toml`, `src/envctl/manifest/apt-base.toml`, `src/envctl/manifest/ai-clis.toml` |
| `compiler:rust` | compiler | evidenced | `package-manager:cargo`, `linker:clang-wild`, `cache:kache` | `src/envctl/Cargo.toml`, `src/envctl/.github/workflows/ci.yml` |
| `package-manager:cargo` | package-manager | evidenced | `compiler:rust` | `src/Cargo.toml`, `src/envctl/Cargo.lock` |
| `linker:clang-wild` | linker | evidenced | `compiler:rust` | `src/envctl/manifest/components.d/epic-h-toolchains.toml` |
| `cache:kache` | compiler-cache | evidenced | `compiler:rust` | `src/envctl/manifest/components.d/epic-h-toolchains.toml`, `usr/bin/kache-rustc-wrapper` |
| `runtime:bun` | runtime-package-manager | evidenced |  | `src/envctl/manifest/base.toml`, `usr/bin/bun`, `usr/bin/bunx` |
| `runtime:node-via-bun` | runtime | evidenced | `runtime:bun` | `src/envctl/manifest/base.toml` |
| `package-manager:js-frontdoors` | package-manager | evidenced | `runtime:bun` | `src/envctl/manifest/base.toml` |
| `runtime:nix` | runtime | evidenced |  | `src/yazelix/flake.nix`, `src/nu_plugin/flake.nix`, `src/envctl/.github/workflows/ci.yml` |
| `container:podman-docker` | container | partial | `deploy:frontdoors` | `src/envctl/manifest/apt-base.toml` |
| `ci:github-actions` | ci-cd | evidenced | `compiler:rust`, `package-manager:cargo`, `sdk:github-cli` | `src/envctl/.github/workflows/ci.yml`, `src/envctl/.github/workflows/sync-master.yml` |
| `sdk:github-cli` | sdk-cli | evidenced | `deploy:frontdoors` | `src/envctl/manifest/components.d/epic-h-toolchains.toml` |
| `deploy:frontdoors` | deploy-runtime | evidenced |  | `usr/bin/envctl`, `usr/bin/meta`, `usr/bin/git-kb` |
| `runtime:python` | runtime | evidenced | `package-manager:pip-pep517` | `execution-framework/scripts/verify_envctl_db_schema.py`, `pyproject.toml` |
| `package-manager:pip-pep517` | package-manager | partial | `runtime:python` | `pyproject.toml`, `requirements.txt` |

## Frontdoors

| command | status | kind | probe |
|---|---|---|---|
| `envctl` | present | symlink | envctl 0.1.0 |
| `meta` | present | file | meta 0.2.22 |
| `git-kb` | present | symlink | git-kb 0.2.12 |
| `bun` | present | file | 1.3.14 |
| `bunx` | present | file | 1.3.14 |
| `kache-rustc-wrapper` | present | file | kache 0.8.0 |

## CI/CD and Deploy Signals

- GitHub Actions workflow evidence: `src/cuda-oxide/.github/workflows/book.yml`, `src/cuda-oxide/.github/workflows/cargo-deny.yml`, `src/cuda-oxide/.github/workflows/ci.yml`, `src/cuda-oxide/.github/workflows/clippy.yml`, `src/cuda-oxide/.github/workflows/cuda-bindings-guard.yml`, `src/cuda-oxide/.github/workflows/differential-drive.yml`, `src/cuda-oxide/.github/workflows/docs.yml`, `src/cuda-oxide/.github/workflows/examples-compile.yml`.
- Container evidence: `src/cuda-oxide/.devcontainer/Dockerfile`, `src/hermes-agent/Dockerfile`, `src/hermes-agent/docker-compose.windows.yml`, `src/hermes-agent/docker-compose.yml`, `src/meta-ruvector/.devcontainer/Dockerfile`, `src/meta-ruvector/benchmarks/Dockerfile`, `src/meta-ruvector/crates/mcp-brain-server/Dockerfile`, `src/meta-ruvector/crates/mcp-brain-server/Dockerfile.minimal`.
- Build task evidence: `src/cuda-oxide/Justfile`, `src/cuda-oxide/cuda-oxide-book/Makefile`, `src/meta-ruvector/crates/ruvector-postgres/Makefile`, `src/meta-ruvector/crates/ruvix/aarch64-boot/Makefile`, `src/meta-ruvector/crates/rvm/Makefile`, `src/meta-ruvector/examples/ruvLLM/esp32-flash/Makefile`, `src/meta-ruvector/examples/scipix/Makefile`, `src/teri/.worktrees/issue-86-source-wires/.worktrees/external-sources/inferrs/Makefile`.

## Compatibility Risks

### Rust MSRV drift

- Evidence: src/envctl/Cargo.toml and CI enforce rust-version/MSRV 1.88
- Mitigation: Keep cargo check on +1.88.0 and avoid dependency upgrades that raise rust-version.

### Host package-manager fallback

- Evidence: src/envctl/manifest/base.toml wraps npm/pnpm/yarn through Bun
- Mitigation: Resolve package-manager commands through $META_ROOT/usr/bin frontdoors.

### Linker/cache wiring differs between local and clean CI

- Evidence: wild and kache wiring live in envctl manifests; GitHub CI uses clean hosted runners.
- Mitigation: Record local linker/cache as migration prerequisites, while treating CI as the clean acceptance surface.

### Container engine path drift

- Evidence: Podman is installed via envctl apt-base component; Dockerfiles and compose files exist across target root.
- Mitigation: Use meta-owned podman frontdoor when container build/deploy artifacts are exercised.

## Evidence Files

- `docs/CONTRACT_MANIFEST.md`
- `generated/envctl_migration_db_model.json`
- `generated/envctl_target_registry.json`
- `generated/package_scan.json`
- `src/envctl/.github/workflows/ci.yml`
- `src/envctl/.github/workflows/sync-master.yml`
- `src/envctl/Cargo.toml`
- `src/envctl/manifest/ai-clis.toml`
- `src/envctl/manifest/apt-base.toml`
- `src/envctl/manifest/base.toml`
- `src/envctl/manifest/components.d/epic-h-toolchains.toml`
