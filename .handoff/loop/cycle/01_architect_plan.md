# Cycle artifact — TASK-0077 architect plan (shared GitHub fetch-token resolver)

**Verdict:** GO. Manifest + `assets/scripts/` layer only — no Engine/CLI/GUI/proto change, no deps, no-C trust boundary untouched.

## Unit ledger
| U# | Lives at | Change | Verified by |
|----|----------|--------|-------------|
| U1 | `assets/scripts/envctl-gh-fetch.sh` (NEW; `envctl_gh_token`, `envctl_gh_api`, `envctl_gh_release_download`, `envctl_gh_curl`) | sourceable 3-tier resolver lib (mint → authed gh → unauth), functions-only, diagnostics to stderr | U6 shell fail-through test |
| U2 | `manifest/components.d/epic-h-toolchains.toml` → `llvm-clang` install | repoint the `gh api` JSON-listing fetch to `envctl_gh_api` | `clang --version`/`llvm-config` verify + U-lock |
| U3 | same → `nix-portable`, `yazi`, `helix` install | repoint `gh release download` to `envctl_gh_release_download` | each component verify + U-lock |
| U4 | same → `gh-cli`, `nushell`, `zellij`, `mise`, `ollama`, `libgccjit` install | repoint release/raw `curl` to `envctl_gh_curl` (bearer when available, identical bytes otherwise) | each verify + U-lock |
| U5 | `envctl.lock` | regen content-hashes for edited components | `envctl lock --check` exits 0 |
| U6 | runtime | resolver fail-through (vault locked → gh tier, real fetch) + one live component install through resolver | guardian |

## Resolver design (the contract)
- `envctl_gh_token` → echo bearer to **stdout**, return 0 if available; tiers: (1) `secretctl mint-github --output json` → extract `.token` (jq if present, else `grep -oE` over frozen compact shape) when non-empty; (2) `gh auth status` ok → `gh auth token`; (3) none → nonzero, echo nothing. ANY mint failure (no binary / vault locked / daemon down / malformed JSON) silently falls through.
- `envctl_gh_api <path> [args]` → `gh api` when authed (preserves `--jq`), else `curl` api.github.com with `Authorization: Bearer` only if a token resolved.
- `envctl_gh_release_download --repo R --pattern P --output O` → `gh release download --clobber` when authed, else `/releases/latest` redirect + `envctl_gh_curl`.
- `envctl_gh_curl <url> [args]` → `curl -fsSL` with optional bearer header injected.
- **Fail-open contract:** resolver returns nonzero ONLY when the final attempted tier's actual network fetch fails — NEVER because the isolated token was merely unavailable. All diagnostics to **stderr** (never corrupt `TAG=$(...)` captures). Functions-only (no top-level `set -e`/execution) so `source` into a `set -euo pipefail` hook is safe.

## Runtime surface (guardian must drive)
1. `source assets/scripts/envctl-gh-fetch.sh; envctl_gh_token` (vault locked → echoes gh token or nothing+nonzero, never hangs/errors); `envctl_gh_api 'repos/cli/cli/releases/latest' --jq .tag_name` → real tag on stdout, `mint unavailable, using gh` on stderr, exit 0.
2. One cheap live install through the resolver (`mise` — single static binary) → existing verify passes.

## Risks / gates
Every Epic-H fetch passes through one script; gh+curl tiers are byte-identical to today, only the guarded mint tier is new. Gates: `envctl lock --check` · `no-c.sh` · `shape.sh` · build · clippy -D · fmt --check · workspace tests.
