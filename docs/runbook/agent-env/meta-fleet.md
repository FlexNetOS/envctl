# Meta Fleet Policy

Use this procedure when Meta must apply one reviewed agent-environment baseline
while every peer keeps an independent repository, branch, lockfile, and native
assistant adapter.

## Scope

The participant set is not a filesystem scan. It is:

- the parent Meta repository; and
- every `.meta.yaml` project whose declared `path` begins `src/`.

Discover it from the parent root:

```nu
^meta project list --json
```

Do not include unregistered worktrees, fixtures, vendor trees, or runner
payloads merely because they happen to be below `src/`.

## Policy ownership

Meta owns selection, shared policy, and fleet evidence. Each participant owns a
committed `agent-env.yaml` and `agent-env.lock`; a project config must never
point at envctl's own configuration. A shared skill source should be pinned to
a reviewed Meta commit. Keep MCP ownership explicit and install no fleet MCP
without a per-assistant conflict inventory.

Envctl-specific skills (`agent-env-codex`, `agent-env-config`,
`env-stabilize`, and `env-toolchain-install`) are not a fleet baseline.
`codedb-config-tables` is opt-in after a verified CodeDB/Yazelix use case.

## Preflight

Use the profile-owned RTK and Nu frontdoors:

```nu
~/.nix-profile/toolbin/nu -l -c '^rtk verify; ^rtk gain'
```

The interactive shell may be Nu, while noninteractive CI runners can supply a
different process shell. The policy does not create a second wrapper or Nix
profile; Yazelix remains the runtime owner.

## Preview, apply, and audit

From primary Meta checkouts, fan out an already reviewed command with
`meta exec --include src -- <command>` and audit the root separately. For a
Meta-managed worktree set, use the worktree-aware executor instead:

```nu
^meta git worktree exec <set> -- envctl agent audit --config agent-env.yaml --scope project --json
```

`meta git review` is a pass-through to `git review`; it is not a valid fleet
executor unless that Git subcommand has been separately installed and proved.

Use `envctl agent sync` without `--apply` for a preview. After review, apply
the sync, commit the config and lock in each peer, and run the zero-network,
read-only proof:

```nu
envctl agent audit --config agent-env.yaml --scope project --json
envctl agent lock --check --config agent-env.yaml --scope project --color never
```

The audit fails on config/lock drift, missing or hash-mismatched managed
skills, missing native MCP targets, or duplicate lock owners for an MCP name.
Resolve the owning source/configuration; never hand-edit generated outputs.
