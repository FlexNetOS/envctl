# Migration / Adoption Engine

`envctl migrate` is the upgrade-only bridge from the current workstation state to
the canonical meta-hosted install topology:

```text
$META_ROOT/{usr/bin,usr/lib,usr/share,etc,var/lib,var/cache,var/log,var/tmp,opt} plus XDG meta-home roots
```

The command exists so operators do **not** hand-edit paths. envctl inventories
old or ad hoc install locations, reports the canonical replacement, materializes
safe meta-local structure, and refuses destructive cleanup until adoption is
proven.

## Non-negotiable rules

- **Meta-root first:** envctl-owned payloads belong under `$META_ROOT`
  using the canonical FHS/XDG layout (`usr`, `etc`, `var`, `opt`, and meta-home XDG roots),
  not `/usr/local`, `/opt`, or a real-home local tree.
- **No blind rebuilds:** Codex/agent assets are adopted/preserved in place. This
  includes `agent-env.yaml`, `agent-env.lock`, `.codex/config.toml`, `.mcp.json`,
  and the ejected harness mirrors under `.Codex/` / `.agents/`.
- **Never downgrade shared meta substrates:** `loop_lib` and
  `meta_plugin_protocol` are protected meta peer dependencies. If envctl needs a
  better API, upgrade the shared substrate and consume it; do not bypass or
  remove it.
- **Purge is strict upgrade-only:** no legacy directory is removed unless a
  typed migration candidate has already been adopted into the canonical
  replacement and ledgered. The initial implementation deliberately refuses
  purge attempts while it builds the evidence trail.

## Verbs

| command | writes? | purpose |
|---|---:|---|
| `envctl migrate scan` | no | Inventory canonical layout, manifest path debt, protected agent assets, and protected meta substrates. |
| `envctl migrate plan` | no | Same report, labeled as a migration plan for automation/front-ends. |
| `envctl migrate apply` | no | Preview the safe apply set. |
| `envctl migrate apply --apply` | yes | Create missing canonical META_ROOT FHS/XDG directories and append a migration ledger entry. |
| `envctl migrate verify` | no | CI/automation gate. Exits non-zero while unresolved migration debt remains. |
| `envctl migrate purge` | no | Explain purge protections and required evidence. |
| `envctl migrate purge --apply --confirm` | guarded | Refuses until the ledger contains verified adoption/parity evidence for a typed legacy candidate. |

All verbs emit `envctl.migration.report.v1` with `--json`.

## Scopes and filters

Use `--scope` to limit the report:

```bash
envctl migrate scan --scope layout
envctl migrate scan --scope component-registry
envctl migrate scan --scope agent-assets --scope meta-substrates
```

Use `--component <id>` to focus manifest debt for specific components:

```bash
envctl migrate plan --component bun --component rust
```

## Ledger and archive roots

The engine reports these roots in every JSON/human report:

- ledger: `$META_ROOT/var/lib/envctl/migrations/ledger.jsonl`
- future archive root: `$META_ROOT/var/lib/envctl/legacy-archives`

The ledger is append-only JSONL. `apply --apply` records the safe materialization
step. Purge attempts that reach the destructive guard also ledger the refusal so
operators have an audit trail for why legacy compatibility roots were preserved.

## Current migration classification

The first implementation classifies manifest references conservatively:

- `.toolchains` / `ENVCTL_LEGACY_TOOLCHAINS` → manifest must move to
  `MetaLayout` / canonical FHS/XDG paths; `.toolchains` itself remains a
  protected compatibility root.
- legacy real-home local spellings → user-global install debt; adopt into
  `$META_ROOT` FHS/XDG paths.
- `/usr/local` / `/opt/` → high-risk system/global path; report only until a
  component-specific adoption plan proves ownership and safety.

This keeps existing working installs available while envctl upgrades the path
authority. It intentionally avoids the dangerous “delete old paths and hope”
failure mode.
