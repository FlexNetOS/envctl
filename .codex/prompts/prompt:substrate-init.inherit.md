# SUBSTRATE INIT — inheritable block for the Codex harness

Source: `.claude/prompts/prompt:claude-code-agent-env-ultraplan.prompt.md` (SUBSTRATE INIT
CONTRACT). The codex v3 prompt has zero substrate wiring — no rtk, git-kb, grit, icm, meta, or
weave anywhere in its 2,491 lines. Include this section verbatim in any codex harness prompt.

Sanctioned one-shot (five substrates): `yzx agent init` → preview; `yzx agent init --apply` →
mutate. Fail-closed pre-check requires `git-kb grit icm meta rtk git` all on PATH; steps in
order — GitKB (`git-kb verify --full --json` / `git-kb init --no-verify` + codex scaffold), Grit
(`grit -r <repo> init`), ICM (`icm init --mode cli --force`), Meta
(`rtk meta exec --include <repo> -- git status --short --branch`), RTK (`rtk init --global --codex`); a
failing step aborts the remainder; never enables hooks/plugins or rewrites git commands;
`--meta-root` defaults `$META_ROOT` else `/home/flexnetos/meta`. Weave is NOT covered; wire it
per the row below. Independent per-row verification is still required:

| Substrate | Floor | Verify (raw output required) | Command substitution it enforces |
|---|---|---|---|
| rtk | 0.43.0 | `rtk --version && rtk gain` | `git/cargo/gh/docker/… X` → `rtk X`; cross-repo `rtk meta git …`; raw: `rtk proxy` |
| meta | 0.2.22 | `rtk meta project list` | `cd <repo> && cmd` → `rtk meta exec --include <repo> -- cmd`; workspace-wide git → `rtk meta git status` |
| git-kb | 0.2.12 | `git-kb code doctor --json` | grep-for-callers/defs → `git-kb code callers/symbols --json` (AST, not text) |
| grit | 0.6.4 | `grit status` in-repo | "I'll be careful" parallel edits → `grit init/claim/release` file::symbol locks |
| icm | 0.10.57 | `icm --version` + store/recall smoke | remember/recall → `icm store` / `icm recall` |
| weave | build from `meta/src/weave` if absent | `weave scan --json` | ad-hoc cross-session files/polling → `weave send/notify/ask` |

WEAVE WIRING (WL-084): `weave setup --provider codex` (maps to Codex's single `notify` hook) —
idempotent, additive, atomic, read-back verified. Session identity is AUTOMATIC — never invent
one: resolution `--from/--me/--name` > `$WEAVE_SESSION` > cwd basename; every peer carries a
stable `sess_<16-hex>` handle. Acceptance: `weave scan --json` shows this session's row.

Init discipline: verify each binary resolves under `/nix/store/` or `~/.nix-profile` (the nix
profile `lifeos-foundation-yzx` owns every toolchain binary), run its health verb, record
`pass/fail/unsupported/gap` per row with raw output, and only then rely on it. A missing
substrate is a `gap` work item, not a stop.
