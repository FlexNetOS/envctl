# Installing envctl (agent-env)

> **Ported from kasetto.dev/docs** (Kasetto v3.2.0, absorbed into envctl `crates/agent-env`).
> Renamed kasetto→`envctl agent`; `kasetto.yaml`→`agent-env.yaml`; mimalloc removed.
> Source: https://www.kasetto.dev/docs/installation. The standalone `kasetto` binary is retired — this is the `envctl agent` surface.

**envctl note (distribution rewritten):** Kasetto shipped as a standalone, self-updating binary
distributed via `curl | sh`, Homebrew, Scoop, prebuilt GitHub-Release binaries, and `cargo install`.
**None of that applies to envctl.** The agent-env engine was absorbed into `crates/agent-env` and is
compiled *into the `envctl` binary* — the standalone `kasetto`/`kst` binaries were retired (TASK-0018).
There is no separate install channel, no `curl | sh`, no Homebrew tap, and no `envctl self update` of
an agent-env binary. You get the agent-env surface by building `envctl` from the meta Cargo workspace.
The *config/usage* guidance below is preserved faithfully.

## How envctl is installed

envctl is a pure-Rust Cargo workspace inside the **meta** multi-repo workspace. Build the CLI (which
includes the agent-env engine) from source:

```
cargo build -p envctl-engine -p envctl       # engine + CLI, zero system deps
```

Run the agent surface directly from the workspace:

```
cargo run -p envctl -- agent --help
```

In a provisioned workstation, `envctl` is placed on `PATH` by the envctl `agent` + `env-ctl`
components in the meta workspace (see the `env-toolchain-install` flow), so `envctl agent …` is
available without a manual install step.

### From source (standalone clone)

```
git clone git@github.com:FlexNetOS/envctl && cd envctl
cargo build -p envctl
```

The resulting `target/release/envctl` (or the meta-provisioned binary) carries the full
`envctl agent` subcommand surface.

## Upgrading

There is **no `envctl self update` for the agent-env engine** — it ships inside `envctl`. To upgrade,
pull and rebuild from the meta workspace:

```
git pull
cargo build -p envctl
```

(envctl *does* have a top-level `envctl self` command family, but it governs the envctl binary's own
lifecycle, not a separate agent-env binary.)

## Shell autocompletion

On a Yazelix workstation, Nu is the configured interactive shell and the active command
frontdoor is the single profile-owned Nix profile. Do not append an envctl block to
`~/.bashrc` or create a user-bin wrapper. Change the Yazelix user-config input when a shell
integration is genuinely required, then regenerate the runtime output through its owner.

For another supported shell, generate completions only after confirming that shell is the active
owner of its configuration; keep the generated completion in that shell's native configuration
surface rather than adding an unrelated Bash startup block.

## Next steps

See the [configuration reference](./configuration.md) for the `agent-env.yaml` schema, or jump to
[commands](./commands.md) for the full `envctl agent` verb reference.
