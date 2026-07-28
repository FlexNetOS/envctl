# ART-104 Toolchain dependency tree

Generated at: `2026-07-28T11:09:04+00:00`
Status: `complete`
Target root: `/home/flexnetos/FlexNetOS`

## Summary

- Coverage: Tree is generated from target descriptor, package scan, envctl registry evidence, selected manifests, and capped filesystem discovery; node-level statuses identify weaker evidence surfaces.
- Manifest signals scanned: `286` from `10216` visited files.
- Languages detected: `{"javascript-typescript": 2, "rust": 154}`.
- Package manager signals: `{"cargo": 154, "container": 4, "nix": 4, "npm": 2}`.
- Lockfile signals: `{"nix": 4, "npm": 2}`.
- Envctl toolchain components captured: `0`.

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

## CI/CD and Deploy Signals

- GitHub Actions workflow evidence: `src/flexnetos_runner/_work/actions-runner-01-work/_actions/actions/checkout/v6/.github/workflows/check-dist.yml`, `src/flexnetos_runner/_work/actions-runner-01-work/_actions/actions/checkout/v6/.github/workflows/codeql-analysis.yml`, `src/flexnetos_runner/_work/actions-runner-01-work/_actions/actions/checkout/v6/.github/workflows/licensed.yml`, `src/flexnetos_runner/_work/actions-runner-01-work/_actions/actions/checkout/v6/.github/workflows/publish-immutable-actions.yml`, `src/flexnetos_runner/_work/actions-runner-01-work/_actions/actions/checkout/v6/.github/workflows/test.yml`, `src/flexnetos_runner/_work/actions-runner-01-work/_actions/actions/checkout/v6/.github/workflows/update-main-version.yml`, `src/flexnetos_runner/_work/actions-runner-01-work/_actions/actions/checkout/v6/.github/workflows/update-test-ubuntu-git.yml`, `src/flexnetos_runner/_work/actions-runner-01-work/envctl/envctl/.github/workflows/ci.yml`.
- Container evidence: `src/flexnetos_runner/_work/actions-runner-01-work/envctl/envctl/third_party/ruvector-postgres-2.0.5/Dockerfile`, `src/flexnetos_runner/_work/actions-runner-01-work/envctl/envctl/third_party/ruvector-postgres-2.0.5/Dockerfile.prebuilt`, `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/third_party/ruvector-postgres-2.0.5/Dockerfile`, `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/third_party/ruvector-postgres-2.0.5/Dockerfile.prebuilt`.
- Build task evidence: `src/flexnetos_runner/_work/actions-runner-01-work/envctl/envctl/third_party/ruvector-postgres-2.0.5/Makefile`, `src/flexnetos_runner/_work/actions-runner-01-work/envctl/envctl/third_party/tokenizers-0.20.4/Makefile`, `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/third_party/ruvector-postgres-2.0.5/Makefile`, `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/third_party/tokenizers-0.20.4/Makefile`.

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
