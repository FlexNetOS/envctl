# ADR — Meta-located tools + envctl-owned portability seam

**Status:** proposed (2026-06-12) · **Owner:** envctl · **Derived from:** runaway-session
containment mission (2026-06-12), owner architecture direction, ADR-0006 (meta-portability).

## Context

The owner invariant: **every tool, dotfile, local data tree, `lib`, and `bin` resolves
inside `meta`**. The real-home local tree is a single bridge to `$META_ROOT/.local`; no
per-tool host-home link farm is allowed, and no active config may hardcode one machine's
absolute meta path. Live audit (2026-06-12) found the box far from this: real binaries and
manager prefixes lived outside meta, and three hardcoded `/home/drdave/Desktop/meta` paths
were present in the canonical `settings.json`.

Two hard facts shape the design:

1. **Meta-root is discoverable via the `.meta.yaml` marker** (walk up from cwd, like git's
   `.git`) — `engine::dashboard::locate_meta_file`. `meta_core` already uses an env-var override
   for its data dir (`META_DATA_DIR`). There is no `META_ROOT` env var yet.
2. **Claude Code `settings.json` field expansion is NOT uniform** (verified via claude-code-guide
   against the docs): shell-executed fields (`statusLine.command`, `hooks[].command`) expand
   `$VAR`/`${VAR}` (shell does it) and `${CLAUDE_PROJECT_DIR}`; but **`extraKnownMarketplaces[].source.path`
   is read literally by Claude — it does NOT expand `$META_ROOT`** (only `${CLAUDE_PROJECT_DIR}`
   and `~`). `${CLAUDE_PROJECT_DIR}` is the *current* project, not meta-root, so it is wrong for a
   global/meta-anchored path.

## Decision

**envctl owns the portability seam.** It discovers meta-root, exports it, sets toolchain install
prefixes into meta, materializes configs that cannot self-expand, and maintains the
`$META_ROOT/.local/bin` frontdoor tree plus the single real-home local bridge — idempotently,
never-downgrade, archive-first.

1. **`META_ROOT` from the marker, exported by `envctl env`** (SHIPPED, `feat/envctl-env`):
   `envctl env` walks to `.meta.yaml` and emits `export META_ROOT=…`/`META_FILE=…`
   (`eval "$(envctl env)"`). This is the single resolution source; everything else references it.

2. **Config references — three mechanisms by field capability:**
   - **Tool invocations** (`icm hook …`, `weave hook …`) → **bare name via PATH** (DONE,
     envctl `2bf6a28`). Portable by construction.
   - **Shell-expanded path prefixes** (`statusLine.command`) → **`$META_ROOT/…`** (shell expands
     once `envctl env` is sourced into the session env).
   - **Claude-literal path prefixes** (`extraKnownMarketplaces[].source.path`) → **envctl
     MATERIALIZES** from `META_ROOT`. The committed source holds a `${META_ROOT}` token; envctl
     renders the live file with the resolved absolute path per machine (re-rendered if meta moves).

3. **Two tool categories, two relocation mechanisms** (both exposed through `$META_ROOT/.local/bin`):
   - **FlexNetOS-built tools** (rtk, agent-env, meta-mcp, icm, vox, weave, **and gitkb — adopted**,
     since gitkb is meta's foundation): built in their meta repo; `$META_ROOT/.local/bin/<tool>`
     points at the meta-hosted build output. **Always latest:** if the installed copy is newer than
     the meta source, bring **meta UP** to that version and build (never downgrade, never "migrate") —
     then swap the install with a meta-hosted frontdoor.
   - **Third-party toolchains** (uv, node-via-bun, bun, mise, cargo): NOT vendored as repos.
     Redirect each manager's install prefix INTO meta via its native env var, owned by envctl:
     `BUN_INSTALL=$META_ROOT/.toolchains/.bun`, `MISE_DATA_DIR=$META_ROOT/.toolchains/mise`,
     `CARGO_HOME=$META_ROOT/.toolchains/cargo`, `UV_TOOL_DIR`/`UV_PYTHON_INSTALL_DIR=$META_ROOT/.toolchains/uv`.
     Installs land physically in meta and are exposed through `$META_ROOT/.local/bin`; "latest" is
     the managers' own `upgrade`. node is installed via bun/mise into that prefix.

4. **envctl responsibilities (the env-ownership build-out):** export `META_ROOT` + the toolchain
   prefixes + meta tool-dir PATH into the session env (the shell/nushell env envctl owns);
   materialize the home-tree `settings.json` literal paths from `META_ROOT`; idempotently
   maintain `$META_ROOT/.local/bin` and the single real-home bridge; **refuse** (doctor/boundary)
   when a real FlexNetOS install is found outside meta. All idempotent, never-delete (archive),
   never-downgrade.

## Consequences

- The 3 hardcoded `settings.json` refs heal cleanly: statusline → `$META_ROOT/…`; the 2
  marketplace paths → envctl-materialized `${META_ROOT}` tokens.
- No repo-per-vendor-tool explosion; toolchains stay in meta via native prefixes.
- gitkb is reclassified from "external" to **adopted foundation** (bring latest into meta).
- Prereq for the relocation loop: the env-export + materialization must land before the live
  bridge/frontdoor swaps (so configs resolve through `META_ROOT`).

## Research / Cross-References

- **Claude settings expansion** (verified 2026-06-12, claude-code-guide vs official docs):
  `statusLine.command`/`hooks[].command` = shell-expanded (`$VAR`, `${CLAUDE_PROJECT_DIR}`);
  `extraKnownMarketplaces[].source.path` = literal (no `$VAR`; only `${CLAUDE_PROJECT_DIR}`/`~`).
- **Toolchain install-dir env vars** (web research 2026-06-12): uv — `UV_TOOL_DIR`,
  `UV_INSTALL_DIR`, `UV_PYTHON_INSTALL_DIR`, `UV_PYTHON_BIN_DIR`/`XDG_BIN_HOME`
  (docs.astral.sh/uv/reference/{storage,environment,installer}); mise — `MISE_DATA_DIR` /
  `XDG_DATA_HOME` (mise.jdx.dev/configuration/settings); cargo — `CARGO_HOME`; bun — `BUN_INSTALL`.
- **Meta resolution precedent:** `meta_cli/src/context.rs` (`.meta.yaml` walk-up),
  `meta_core/src/data_dir.rs` (`META_DATA_DIR` override), `engine::dashboard::locate_meta_file`.
- ADR-0006 (meta-portability home tree), runaway-containment mission (ICM recursion fix +
  settings de-hardcoding `2bf6a28`), `envctl env` (`feat/envctl-env`).
- **Bun path note:** envctl uses `$META_ROOT/.toolchains/.bun` rather than
  `$META_ROOT/.toolchains/bun` because Bun honors `BUN_INSTALL`, and Codex's
  installer/updater recognizes Bun-managed installs by the `.bun/install/global`
  path segment.
- **Open follow-up:** decide gitkb adoption mechanism (vendor crate vs `.meta.yaml` project).
