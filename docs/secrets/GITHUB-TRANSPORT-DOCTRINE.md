# GitHub transport doctrine (verified)

**TASK-0053.** This note routes the *verified* meta GitHub transport and automation doctrine into
envctl, so envctl's GitHub credential / merge-gate work proceeds from source-grounded truth rather
than stale assumptions or raw GitHub-API output. Every claim below was confirmed by reading the
cited source (file:line) in the live workspace.

## 1. SSH `git` is the repository source of truth

For the FlexNetOS fleet, the authoritative repository transport is **`git` over SSH**, not `gh`:

- `.meta.yaml` configures every project as an SSH remote — `git@github.com:FlexNetOS/<repo>.git`.
  The repo's identity and history resolve through SSH `git`, and `git ls-remote --symref origin HEAD`
  over SSH is the canonical "what does origin actually say" check.
- **`gh` is NOT the git transport truth.** `gh config get git_protocol` reports `https` while the
  real clones are SSH — so the `gh` CLI's own protocol setting must never be treated as the
  repository transport of record. Resolve refs/branches/HEAD via SSH `git`, not via `gh`.

## 2. `gh` / GitHub API is workflow orchestration — advisory until read-back verified

`gh` and the GitHub REST API are the **workflow-orchestration** layer (checks, PR state, merge,
policy). Their mutations are **advisory until re-queried against the underlying truth**:

- **`gh` mutations can silently succeed.** `.github_org/architecture/map/01-meta-control-plane.md`
  records this as ground truth ("**gh mutations can succeed silently — always re-query**", method
  lessons at `:167`/`:475`; the `FlexNetOS/ruflo` fork was a "SILENT SUCCESS again — stdout empty,
  verified by org re-query" at `:136`). Therefore: after any `gh`/API mutation, **re-query** the
  result against git refs, PR state (`gh pr view <PR> --json state,mergeStateStatus`), and the
  required-check set before trusting it. A raw connector/API response is not success on its own.
- **Always assert owner/repo — the wrong-CWD hazard.**
  `.github_org/architecture/plan/2026-06-17-deep-review-upgrade-plan.md:56` records a concrete
  policy-applier hazard: `apply-github-policies.py` `--apply` mutates "whatever repo `gh repo view`
  resolves from CWD, with **no assertion** it equals the intended `FlexNetOS/<name>`" — a
  wrong-clone / `gh` default can PUT policies to the *wrong repo*. The fix (and the rule for any
  envctl path that shells `gh`): require `--owner/--repo` or assert the resolved slug before any
  mutating call. Never rely on the ambient CWD to pick the target repo.

## 3. Envctl owns the scoped, broker-only GitHub App token path

Envctl is the credential broker. The App private key lives in envctl's vault; envctl exchanges the
App-JWT for a short-lived, per-repository, per-permission **installation token** and hands only that
scoped token out. The downstream consumer contract is **frozen** (TASK-0020) and must stay
byte-stable, or any change updates `flexnetos_github_app` consumers in the same cycle:

- **CLI:** `secretctl mint-github --installation-id <id> --repository-ids <id[,id…]>
  --permissions <name:access[,…]> --ttl-secs <n> --output json` →
  `{"token": "<scoped>", "expires_at_unix": <u64>}` (compact, exactly two fields).
- **RPC:** `Vault.MintGithub(MintGithubReq) returns MintGithubResp { token, expires_at_unix }`.
- **Token discipline (AC4):** tokens are **broker-only, scoped, short-lived, and never logged**.
  The engine wraps the App PEM and minted token in `Zeroizing` (single owner, wiped on drop), the
  token enters only the `Authorization` bearer header, and mint/revoke error snippets are truncated
  and token-free (`crates/secrets-engine/src/mint_github.rs`). Mint is gated on a vault unlock +
  USB-possession; audit/event bodies carry metadata only, never the secret.
- **Mutating App ops are `--apply`-gated dry-runs by default** (CF-8): `secretctl github-app enroll`
  and `secretctl github-app revoke-token` preview unless `--apply` is passed
  (`crates/secretctl/src/main.rs`).

### POLICY_DRIFT_TOKEN — uses the existing `mint-github` path (no new surface)

Strict `.github` policy-drift reads (branch protection, rulesets, environments, repo settings) need
a token the default `GITHUB_TOKEN` can't provide. That token is the **existing** `mint-github` path
with the strict scope `administration:write,metadata:read` — **no new envctl surface is required**.
The live consumer is `.github_org/scripts/rotate-policy-drift-token.sh`:

- `:37` `INSTALLATION_ID="${POLICY_DRIFT_INSTALLATION_ID:-140063898}"`
- `:38` `TTL_SECS="${POLICY_DRIFT_TTL_SECS:-3600}"`
- `:39` `PERMS="administration:write,metadata:read"`
- `:90-95` shells `secretctl mint-github --installation-id … --repository-ids … --permissions
  "${PERMS}" --ttl-secs … --output json` (token text never printed; mint errors redacted).

The engine's `build_token_request_body` (`crates/secrets-engine/src/mint_github.rs:342`) already
serializes an arbitrary `name:access` scope into the GitHub permission map, so this scope produces
`{"administration":"write","metadata":"read"}` with no code change. The
`policy_drift_permissions_scope_serializes` regression test in `crates/secretctl/src/main.rs` pins
both ends: the scope parses through the real `mint-github` clap surface, and the *real* engine
serializer emits exactly that map (parse-level / fake-transport only — never a real credential, never
a logged token).

## 4. Merge-gate cross-check (consumer expectations)

The downstream merge gate (`../flexnetos_github_app/crates/app-core/src/merge_gate.rs`) defines how a
verdict reaches GitHub — and envctl's token model is built to serve it without ever holding a broad
merge token:

- The App posts its gatekeeper verdict as a GitHub **check-run** wired as a *required* status check
  (`MergeGate::post_verdict`), and arms GitHub-native auto-merge **only after** the verdict is green
  (`ensure_armable` returns `Ok` only for `Conclusion::Success`; `merge_gate.rs:66-74`).
- It is **never** a native `github-actions[bot]` APPROVE — a bot APPROVE silently satisfies
  branch-protection required-reviews and defeats the gate (gh-aw #25439; cross-referenced at
  `.github_org/architecture/map/01-meta-control-plane.md:116`/`:467`).
- `UnwiredMergeGate` **fails closed** until the real check-runs/auto-merge REST client lands
  (`merge_gate.rs:81-88`). The minting seam likewise fails closed (`UnwiredMinter` →
  `MintError::NotWired`; `../flexnetos_github_app/crates/app-core/src/mint.rs:95-99`) and never falls
  back to a plaintext PAT.

Consequently, **agents hold no broad merge token, never native-APPROVE their own PRs, and never
force-merge red checks.** They emit verdicts as DATA; the merge is an out-of-band gate.

**Output-shape cross-check.** The consumer's `parse_mint_output`
(`../flexnetos_github_app/crates/app-core/src/mint.rs:131-143`) deserializes envctl's stdout into
`struct Out { token: String, expires_at_unix: u64 }` — exactly the two-field shape envctl emits, and
exactly what the secretctl differential contract tests pin.

## 5. Handoff continuity wording

Envctl handoff continuity uses the **redb-backed ledger plus deterministic JSONL export** — it is
**NOT SQLite**. Any doc, packet, or status describing the continuity substrate must say
redb + deterministic JSONL export, never SQLite.

## Verification rule (applies to all GitHub automation)

Verification uses **SSH-backed git refs** (`git ls-remote --symref origin HEAD`) plus a **`gh`
re-query** of PR state and required checks. **No raw API mutation is accepted as success without
read-back.** Use the API only through controlled `gh`/App paths with explicit owner/repo selection,
least privilege, short TTLs, and read-back verification; SSH `git` remains the repository truth.
