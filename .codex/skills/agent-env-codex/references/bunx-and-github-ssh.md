# Bun/Bunx and GitHub SSH contract

Apply this contract to every skill loaded, generated, reviewed, or executed by `/agent-env-codex`.

## Bun-owned JavaScript execution

Yazelix/Nix owns the active JavaScript runtime. Resolve `bun` and `bunx` from the profile tool surface and never create a global package-manager shadow.

| Legacy recipe | Required FlexNetOS recipe |
| --- | --- |
| `npm install` | `bun install` |
| `npm ci` | `bun install --frozen-lockfile` |
| `npm install --save-dev <pkg>` | `bun add --dev <pkg>` |
| `npm run <script>` / `npm test` | `bun run <script>` |
| `npm update` | `bun update` |
| `npm audit` | `bun audit` |
| `npm version <level>` | `bun pm version <level>` |
| `npm pkg get <key>` | `bun pm pkg get <key>` |
| `npm publish` | `bun publish` |
| `npx <package> ...` | `bunx <package> ...` |

Examples include `bunx ruv-swarm/claude-flow@alpha`, `bunx ruv-swarm ...`, and `bunx claude-flow@alpha ...`.

Terms such as “npm registry,” `.npmrc`, `NPM_TOKEN`, and an upstream package's `npm/` source directory may remain when they name an external protocol, file, secret name, or source path. They are not permission to execute `npm` or `npx`.

Run `scripts/check-bun-command-policy.py <envctl-root>` before delivery. Fix every executable `npm`/`npx` finding in every Markdown skill owner; do not rely only on a runtime rewrite hook.

## Personal and FlexNetOS organization SSH proof

Do not read `~/.ssh` private keys or print credentials. Prove identity and authorization through behavior:

```text
ssh -T -o BatchMode=yes git@github.com
gh api user --jq .login
gh config get git_protocol --host github.com
gh api user/memberships/orgs/FlexNetOS
rtk meta git setup-ssh
rtk meta exec --include envctl -- git ls-remote git@github.com:FlexNetOS/envctl.git HEAD
```

Required interpretation:

1. The SSH greeting and `gh api user` identify `drdave-flexnetos`.
2. GitHub CLI protocol is `ssh`.
3. Organization membership is `active`; record the returned role without changing it.
4. The SSH `ls-remote` proves the authenticated principal can read the FlexNetOS repository.
5. A successful personal greeting without membership and repository proof is incomplete.
6. Organization settings mutation still uses `gh`, REST, GraphQL, or GitHub UI; SSH does not configure Actions, policies, secrets, rulesets, Apps, or other organization settings.
