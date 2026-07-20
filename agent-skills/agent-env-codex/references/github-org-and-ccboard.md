# GitHub organization and ccboard integration

Use this reference when `/agent-env-codex` touches GitHub administration, repository delivery, ccboard, Claude endpoints, or Codex session observability.

## GitHub control-plane truth

- Require SSH for repository fetch/push and verify `git@github.com:FlexNetOS/<repo>.git` through the Meta SSH setup.
- Do not claim SSH can configure organization settings. Use authenticated `gh`, REST, GraphQL, or the GitHub UI for administration.
- Never widen auth scopes, change permissions, or expose a secret to make an audit pass. Record the exact denied endpoint/scope and continue every non-mutating check.
- Inventory before mutation; archive/export non-secret metadata; calculate a diff; apply only requested drift; read back the result.
- Never print secret values. Inventory only names, visibility, selected repositories, timestamps, and policy metadata.

Official anchors: [organization settings](https://docs.github.com/en/organizations/managing-organization-settings), [Actions permissions](https://docs.github.com/en/organizations/managing-organization-settings/disabling-or-limiting-github-actions-for-your-organization), [Actions policies](https://docs.github.com/en/organizations/managing-organization-settings/actions-policies/about-actions-policies), [organization rulesets](https://docs.github.com/en/organizations/managing-organization-settings/managing-rulesets-for-repositories-in-your-organization), [custom properties](https://docs.github.com/en/organizations/managing-organization-settings/managing-custom-properties-for-repositories-in-your-organization), [security at scale](https://docs.github.com/en/code-security/concepts/security-at-scale/organization-security), [Code Quality](https://docs.github.com/en/code-security/how-tos/maintain-quality-code/enable-code-quality), [SSH certificate authorities](https://docs.github.com/en/enterprise-cloud@latest/organizations/managing-git-access-to-your-organizations-repositories/about-ssh-certificate-authorities), and [fork synchronization](https://docs.github.com/en/pull-requests/collaborating-with-pull-requests/working-with-forks/syncing-a-fork).

## FlexNetOS organization surface matrix

| Surface | Inventory/configuration contract |
| --- | --- |
| General settings | Repository creation/visibility/fork policies, base permissions, 2FA, commit signoff, member and outside-collaborator controls. |
| Actions, workflows, runners, policies | Allowed actions, SHA pinning, workflow token permissions, PR approval behavior, fork controls, runner groups/labels, artifact retention, and workflow execution protections. Require Linux; reject macOS/Windows jobs. |
| Rules and rulesets | Organization and repository rulesets, default-branch targets, required checks/reviews/signatures, force-push/deletion bans, merge methods, and bypass actors. Never weaken to unblock a PR. |
| Secrets, variables, environments | Names/visibility/selected-repository metadata only; environment protection and deployment policy. Never read or print values. |
| Sandboxes and hosted compute networking | Codespaces, hosted-runner networking, larger-runner/private-network policy, image provenance, isolation, retention, and access. |
| Pages and Packages | Publication/source/custom-domain policy; package visibility, retention, provenance, deletion, and repository linkage. |
| Issues, Projects, Discussions | Feature enablement, creation/deletion permissions, templates/forms, labels, project base permissions/visibility, moderation, and discussion creation policy. |
| Webhooks and deploy keys | Metadata, target scope, event set, active state, TLS validation, least privilege, rotation owner, and stale-key removal. Never expose webhook secrets or private keys. |
| GitHub Apps | Installation inventory, repository selection, granted permissions/events, token issuer owner, rotation, and stale installation removal. |
| Code security and quality | Dependency graph, Dependabot, secret scanning/push protection, code scanning/CodeQL, security configurations, Code Quality, coverage, and ruleset thresholds. |
| Codespaces | Access, machine/retention/prebuild policy, secrets metadata, repository access, and network controls. |
| Custom properties | Schema, required/default/allowed values, repository assignments, and use as ruleset/policy targets. |

Absence of an API result is not proof a surface is empty. Distinguish `[]` from `403`, `404`, plan unavailability, and missing OAuth scope. Never refresh scopes or change an `Allow` setting merely to complete the inventory.

## Repository and fork delivery

1. Use a Meta-managed task worktree and RTK/Meta Git routing.
2. Verify `origin` uses FlexNetOS SSH. For a fork, verify `upstream` points to the parent.
3. Fetch/prune both remotes. Merge upstream into a clean protected trunk so local commits remain; never force-sync or hard-reset.
4. Preserve and test both upstream and local capabilities when resolving conflicts.
5. Commit every intended file, push over SSH, open/update the PR, and enable auto-merge.
6. Wait for required Linux checks and the merged state.
7. Fast-forward clean `main`/`master`/`develop`, then archive and remove every merged non-trunk task branch/worktree locally and remotely.

## ccboard and Claude/Codex implementation path

Yazelix already owns the installed ccboard pane at `configs/zellij/layouts/flexnetos_agent_workspace.kdl`, and `packaging/runtime_release_contracts.nix` proves `__YAZELIX_RUNTIME_DIR__/libexec/ccboard`. Do not edit generated runtime under `~/.local/share/yazelix`.

Claude's current path is the compatibility model:

```text
~/.claude settings/projects JSONL
        |
        +-> ccboard DataStore + FileWatcher + live process/hook monitor
        |
        +-> envctl home/.claude/settings.json(.tmpl)
              +-> ccbrain-session-start.sh
              +-> ccbrain-session-stop.sh -> ~/.ccboard/insights.db
        |
        +-> codex-harness-claude-bridge -> supervised runner/provider receipt
```

Codex is partially wired, not absent:

- `ccboard-core/src/parsers/codex.rs` scans `~/.codex/sessions/YYYY/MM/DD/*.jsonl`.
- `DataStore::scan_third_party_sessions` inserts Codex metadata at startup.
- `SourceTool::Codex` and the TUI source badge already distinguish Codex sessions.

Do not replace those capabilities. Complete the missing path in the ccboard source owner:

1. Parse Codex rollout events into real session ID, cwd/project, timestamps, model, messages, token/cost/tool usage, parent/subagent relationships, and content instead of line-count/date approximations.
2. Add configurable Codex home/session roots; do not derive Codex ownership from `CCBOARD_CLAUDE_HOME`.
3. Extend the watcher to Codex date/session directories and update the store incrementally.
4. Extend session-content, search/cache, activity, analytics, and export paths without regressing Claude.
5. Add Codex live-process/session correlation and a hook/event adapter only where Codex exposes a stable source; otherwise report the unsupported live field explicitly.
6. Expose source filtering and Codex detail consistently in TUI and web/API.
7. Test Claude-only, Codex-only, mixed, missing-root, malformed-rollout, live-update, and large-session cases.
8. Deliver the ccboard change in its own Meta-managed worktree/PR, rebuild Yazelix through its source/profile owner, and prove the active `yzx` layout launches the upgraded profile-owned ccboard.

The envctl harness owns Codex/Claude routing receipts and managed hook templates; ccboard owns session ingestion and presentation; Yazelix owns packaged runtime delivery. Do not collapse these three owners into one repo.
