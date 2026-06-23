# Implementation log: TASK-0077 — shared GitHub fetch-token resolver for Epic-H installs

GREEN. Manifest + shell-asset only; no Rust source changed (only `envctl.lock` regen via the CLI). No deps, no-C trust boundary untouched.

## Changes
- `assets/scripts/envctl-gh-fetch.sh` (NEW, +x): sourceable 3-tier resolver lib — functions only, no top-level `set -e`/execution, all diagnostics to stderr.
- `manifest/components.d/epic-h-toolchains.toml`: repointed all 10 GitHub fetch sites to source the resolver and call the matching wrapper (every flag/pattern/output path preserved).
- `manifest/envctl.lock`: regenerated content-hashes for the 10 edited components (79 components total).

## Engine API
None — no Engine/CLI/GUI/proto change. The "contract" is the shell resolver surface:
- `envctl_gh_token` → bearer to stdout + exit 0 if available; tiers (1) `secretctl mint-github` [gated on `ENVCTL_GH_INSTALLATION_ID`], (2) authed `gh auth token`, (3) none → exit 1. Any failure falls through silently; diagnostics to stderr.
- `envctl_gh_api <path> [args…]` → `gh api` when authed (preserves `--jq`), else `curl` api.github.com with optional bearer.
- `envctl_gh_release_download --repo R --pattern P --output O` → `gh release download --clobber` when authed, else `/releases/latest` redirect + bearer curl.
- `envctl_gh_curl <url> [args…]` → `curl -fsSL` with optional `Authorization: Bearer` header.
- Helper `_envctl_gh_authed` selects the gh-vs-curl branch so each component's existing structure is preserved byte-identically.

## Repoint map (per unit)
- U2 `llvm-clang` (gh api JSON listing): gh branch → `envctl_gh_api` (preserves `--jq`); kept the component's OWN unauth grep-based curl fallback, now routed through `envctl_gh_curl`. Asset download → `envctl_gh_curl`.
- U3 `gh release download` sites:
  - `nix-portable`, `yazi`: gh branch → `envctl_gh_release_download`; unauth redirect+curl fallback → `envctl_gh_curl`.
  - `helix`: gh branch → `envctl_gh_release_download` (pattern is a version-agnostic GLOB, passed straight to `gh release download`); unauth branch builds the concrete URL and is routed through `envctl_gh_curl` (NOT flattened into the wrapper, which treats `--pattern` as a literal filename).
- U4 release/raw curl sites: `gh-cli`, `nushell`, `zellij`, `mise`, `ollama`, `libgccjit` → `envctl_gh_curl`. `libgccjit`'s raw.githubusercontent.com `libgccjit.version` fetch also routed through `envctl_gh_curl`.
- gpu.toml audit: its only github.com hits are `nvidia.github.io` keyring/container-toolkit URLs (apt GPG key + repo), NOT api.github.com / releases — LEFT UNCHANGED, as the plan directed.
- The `curl -fsSLI -o /dev/null` redirect-HEAD tag-resolution probes stay plain curl (github.com HEAD, not rate-limited API; each precedes a tokened asset fetch).

## Tests added
No automated tests (shell-asset change; the components are not unit-tested). Verified by live runtime exercise (below) per the plan's Runtime surface.

## Build/test status (commands run + result)
- `bash -n assets/scripts/envctl-gh-fetch.sh` → SYNTAX OK
- `shellcheck assets/scripts/envctl-gh-fetch.sh` → SHELLCHECK OK (clean)
- `source … && envctl_gh_token` → exit=0, token_len=40 (gh tier; no installation-id so mint tier cleanly skipped — no hang/error)
- `source … && envctl_gh_api 'repos/cli/cli/releases/latest' --jq .tag_name` → `v2.95.0` on stdout, exit 0, diagnostics only on stderr (captured-stdout==`v2.95.0`, no contamination)
- `envctl_gh_curl 'https://api.github.com/repos/cli/cli/releases/latest'` (unauth path) → fetched, `"tag_name": "v2.95.0"`, exit 0
- LIVE install fetch (mise, sourced from META_ROOT path as a component hook does) → fetched `mise v2026.6.12`, ELF runs (`mise --version` OK)
- LIVE `envctl_gh_release_download` gh-authed branch (yazi literal pattern) → downloaded 11,394,377-byte zip, exit 0
- `cargo run -p envctl -- lock --check` BEFORE regen → exit 1, flagged exactly the 10 edited components
- `cargo run -p envctl -- lock` → wrote manifest/envctl.lock (79 components)
- `cargo run -p envctl -- lock --check` AFTER regen → exit 0
- `bash ci/gates/no-c.sh` → NO-C GATE PASS (rustls=0.23.40 on ring=0.17.14; zero aws-lc/openssl/C-SQLite)
- `bash ci/gates/shape.sh` → SHAPE GATE PASS
- `cargo build -p envctl-engine -p envctl` → Finished, exit 0

## Deviations
- **secretctl mint-github requires `--installation-id` (frozen contract, TASK-0020); it cannot be invented.** Per the plan ("do not invent an installation-id; the mint tier may simply not succeed and MUST fall through cleanly"), the mint tier is gated on an operator-supplied `ENVCTL_GH_INSTALLATION_ID` env var (TTL via `ENVCTL_GH_TTL_SECS`, default 3600). When unset → mint tier is skipped and the resolver falls through to gh (verified). This keeps the mint tier WIRED and ready when an operator provides an installation-id, while never fabricating one. NOT a design change — a fail-open gate exactly as the plan's contract allows.
- `helix` gh `--pattern` is a glob — its unauth fallback was NOT flattened into `envctl_gh_release_download`. Instead helix's own gh/curl branch structure is preserved, each side routed through the resolver. Honors "preserve the component's own fallback when the fetch logic is subtle."

## Handoff notes (for the guardian)
- **No-regression claim:** the gh and curl tiers are byte-for-byte behavior-identical to today — the resolver ONLY optionally prepends `Authorization: Bearer` when a token is available; it never changes WHAT is fetched, only the rate-limit bucket. Verify by diffing the old inline fetch logic against the wrapper bodies.
- The resolver is functions-only with NO top-level `set -e`/execution → safe to `source` into the components' `set -euo pipefail` install hooks. Confirm there's no stray top-level statement.
- All diagnostics go to **stderr**; stdout carries only return values. The critical no-corruption case is `TAG=$(envctl_gh_api … --jq …)` — verified the captured stdout is exactly `v2.95.0` with the `mint unavailable, using gh` / `gh unavailable …` lines on stderr only.
- `envctl.lock` lives at `manifest/envctl.lock` (not repo root). `lock --check` exits 0.
- gpu.toml's github hits are `nvidia.github.io` (keyring/container) — intentionally untouched; not in scope.
- No grit / parallel mode used (sequential single-implementer).
