# Architecture Plan — OpenRouter Agent-Environment Convergence

## Verdict

GO. Existing OpenRouter provider profiles, policy, Rust probe, and internal
catalog are reusable. The active/projected model catalog and installed runtime
are incomplete, and the probe accepts responses too loosely.

Authenticated generation is NO-GO until an operator-owned OpenRouter key is
loaded without exposing it.

## Ownership

- Meta enumerates the fleet and owns shared policy/evidence.
- Each repo independently owns any committed `agent-env.yaml` and
  `agent-env.lock`.
- Repos without those files inherit the central Codex runtime; they are not
  falsely reported as independently synced.
- Envctl remains the per-repo lock/sync/audit engine.
- Yazelix/Nix owns the Codex binary frontdoor; `codex-global-baseline`
  projects tracked profiles and the catalog into the active home.
- Nushell is the primary fleet/runtime command surface. Existing Bash lifecycle
  hooks remain compatibility gates; no new Python/JS or shell control plane.

## Minimal implementation

1. Add a visible `tencent/hy3:free` entry to the tracked active catalog and
   keep its contract in parity with the harness catalog.
2. Correct current official metadata: 262144-token context, high/low reasoning,
   and the 2026-07-21 free-route expiry.
3. Harden the Rust OpenRouter probe to require target-model discovery and a
   parsed completed Responses result with non-empty output, while redacting all
   credential-bearing data.
4. Add failing-first Rust tests for absent target, API error/empty output, and a
   valid completed response. Add tracked-catalog/profile contract coverage.
5. Add a read-only Nushell fleet verifier that inventories Meta plus all
   declared projects, previews/audits repos with independent agent-env state,
   and classifies the remainder as central-runtime inheritance.
6. Apply only through the existing envctl component owner, then prove active
   mode-0600 projections, profile-owned Codex 0.144.0, and interactive
   `/model` visibility.
7. If a key is available, run a unique-marker HY3 generation through the
   OpenRouter profile and persist only redacted proof.

## Impact and test gates

- `openrouter_model_catalog_summary`: one direct production caller.
- `openrouter_probe_value`: two direct callers (shim and supervised runner).
- Whole `codex-harness/src/lib.rs`: HIGH file-level blast radius; edits stay
  localized to OpenRouter symbols.
- Runtime-contract and agent-audit engine code remain unchanged.
- Run focused harness tests first, then harness workspace tests, envctl
  agent-env tests, Codex baseline/runtime gates, no-C/meta-substrate gates,
  format/clippy, lock checks, fleet preview/audit, installed-runtime proof, and
  GitKB change detection.

## Archive

Pre-edit snapshots are under
`/tmp/envctl-openrouter-agent-env-20260714/`.
