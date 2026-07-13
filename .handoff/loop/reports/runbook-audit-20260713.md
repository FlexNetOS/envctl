# Envctl runbook audit — Yazelix/profile and agent-env ownership

Date: 2026-07-13
Scope: `docs/runbook/**`, checked against `agent-env.yaml`, `agent-env.lock`, the `agent-env-config` skill, active Yazelix profile state, and current envctl source.

## Verdict

The runbook is not an executable source of truth today. Its agent-env command descriptions are mostly useful, but its ownership diagrams and several operational examples predate the Yazelix single-profile contract. Following them would recreate the exact parallel paths this repair is removing.

## Confirmed mismatches

1. `docs/runbook/README.md` and `DIAGRAMS.md` still require the retired six-MCP baseline. Current declared/generated state intentionally contains only remote `exa`; the owning skill explicitly refuses restoration of the five local-launcher entries until each has a profile-owned frontdoor.
2. The quickstart runs `agent doctor --scope global`, while this repository's source of truth is `scope: project`; the global command currently inspects an empty inventory and gives misleading results.
3. `DIAGRAMS.md` §§11/13 says every tool belongs in `$META_ROOT/.toolchains` + `$META_ROOT/usr/bin`, treats `/nix` as removable, and depicts user-global/home/meta wrappers. Current policy makes the one Yazelix Nix profile the binary/runtime owner and treats those paths as shadows.
4. The component inventory is stale (`rustup→nightly`, standalone `rtk`, `kasetto`, meta-prefix Nushell/Yazelix, apt prerequisites). It does not describe profile-owned `yzx`, native RTK, Nu module routing, or the editable-config/generated-runtime split.
5. `agent-env/installation.md` tells operators to build/run unqualified Cargo from a clone and append Bash completion to `~/.bashrc`. It never requires the profile-owned cargo/envctl path or Yazelix-managed Nu input surface.
6. `agent-env/introduction.md` advertises `--dry-run` examples despite the documented envctl contract being preview-by-default, and promotes global config without distinguishing the repo's project-scoped source.
7. The diagrams state that `lock --check` is a CI gate, but no dedicated `manifest/envctl.lock` gate exists in the workflow.
8. Operational secrets guidance still names `/usr/local` CA installation and other host mutations without classifying them as explicit irreducible/external exceptions to the profile model.
9. The runbook does not explain long-lived process PATH generations: an existing Codex session can retain an old immutable profile store prefix even when `~/.nix-profile` has one current element. Relaunch through profile `yzx` is required; this is not evidence of multiple active profiles.
10. There is no release/runtime contract proving that Nu finds the profile-owned `rtk_wrappers.nu`, `^bash` works natively, and no duplicate envctl wrapper is sourced.

## Adoption decision for surfaced skills

- Adopt `env-toolchain-install` and `env-stabilize` as the normative install/drift paths; they match single-profile ownership and fail-closed doctor/lock requirements.
- Adopt `agent-env-config` as the agent config source of truth; it correctly supersedes the six-MCP and JavaScript/ECC drift in older prose.
- Adopt `cross-repo-health` for the final health matrix and `code-research-verify` for every report claim.
- Adopt `feature-forge`/`rust-feature-impl` for engine, CLI, GUI, manifest, and profile-contract repairs.
- Keep planning-only, porting, image, NotebookLM, and GitNexus skills out of this repair because they do not close a verified runtime/test gap.

## Required documentation upgrade

Replace the old meta-prefix convergence diagram with:

```text
editable inputs                         generated/runtime outputs
~/.config/yazelix/**  ── profile yzx ─▶ ~/.local/share/yazelix/**
repo agent-env.yaml   ── envctl agent ▶ repo .claude/.codex projections
Yazelix flake source  ── nix profile ─▶ exactly one lifeos_foundation_yzx element
                                          ├─ bin/yzx (launch frontdoor)
                                          └─ toolbin/{rtk,nu,cargo,...}
```

Then mark `$META_ROOT/.toolchains`, `$META_ROOT/usr/bin`, user-bin shims, repo caches, and immutable old profile generations as non-authoritative migration/shadow surfaces unless an explicitly documented external-data exception applies.

