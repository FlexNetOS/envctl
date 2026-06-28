# Risk policy — planning targets

Machine-shaped risk policy per target. `risk_tier` ∈ {APPLY, PROPOSE, SUPERVISED, OWNER-ONLY}.
Trust-boundary classes: trust-boundary · secrets · destructive · provider · model · cross-repo.
Built only from CONFIRMED/QUALIFIED + feasibility-passed rows.

## prompt-hub

risk_policy: prompt-hub is the Front-Door intent STORE; its mutations are governance-of-intent, so the
default posture is fail-closed and additive-only (Upgrade Only / No Downgrades). No production code is
mutated by planning; every change below lands as a feature-gated/additive PR through the repo's
push→PR→green-merge gate. Relaxations are forbidden — strength changes are additive only.

| id | surface | class | risk_tier | rule | acceptance | evidence |
|---|---|---|---|---|---|---|
| RP-1 | 4 AI workflows interpolate untrusted PR diff into a `run:` shell/JSON body | trust-boundary | SUPERVISED | PR-controlled content MUST pass via `env:` and be read as `"$VAR"` / built with `jq --arg`; never spliced into a `run:` body. A PR diff containing `";id;"` produces no shell execution. | no `${{ steps.*.outputs.* }}` inside any `run:` body in `external-ai-apis.yml`/`ai-safety-deployment.yml`/`ai-code-review.yml`/`multi-model-evaluation.yml`; matches the already-remediated `audit_sync.yml` pattern | verdicts.md VERDICT 4; governance gov-002 |
| RP-2 | plaintext API-key / secret CI surface (ANTHROPIC_API_KEY, DEVIN_API_KEY over curl; GitHub Models GITHUB_TOKEN) | secrets | SUPERVISED | provider secrets injected only via `env:` (never inline in a command); model ids pinned/centralized to dated identifiers; no third-party secret where GITHUB_TOKEN suffices | `external-ai-apis.yml` keeps keys in `env:`; AI-workflow model ids are pinned + centralized so one edit updates all | prompt-architecture §2; governance gov-008/gov-012 |
| RP-3 | plugin dynamic native-code loading (libloading + inventory, catch_unwind) | trust-boundary | SUPERVISED | loaded `.so` objects are OUTSIDE the crate `#![forbid(unsafe_code)]` guarantee; the boundary MUST be documented and the load path treated as untrusted-code execution | ADR-0007 amended to state the native-code trust boundary; a doc/test asserts the guarantee does not extend to loaded objects | prompt-architecture §2 (plugin grant) |
| RP-4 | GoalArtifact envelope wire format → rusty-idd | cross-repo | SUPERVISED | the envelope field set MUST be derived from rusty-idd's ACTUAL consumer schema (`rusty-idd/.handoff/loop/plan/`), NOT invented; an unbound envelope must NOT land as canonical | step 0 reads the consumer schema; the RED assertions (schema_version, provenance.sources, produced_by, target, artifact_kind) are reconciled before GREEN is the canonical gate | verdicts.md UPGRADE A/B/H conditions; architecture-prompt-hub.md OPEN QUESTIONS |
| RP-5 | `git push *` / `gh pr merge *` allowed un-gated at the Claude layer (no `.claude/settings.json`) | destructive | PROPOSE | add a `permissions.deny` (or PreToolUse hook) blocking `git push --force`, `git reset --hard`, `git clean -fdx`, `rm -rf` on repo paths — mirroring `rules.toml [commands.blocked]`; narrow push to `--force-with-lease`-safe | a committed `.claude/settings.json` denies the destructive set; additive only (no relaxation) | governance gov-005 |
| RP-6 | multi-vendor provider egress (OpenAI/Anthropic/Google/Custom + local_llm) | provider | PROPOSE | provider calls are opt-in per config; no silent provider downgrade; cross-vendor fan-out (eval harness) is an explicit opt-in subcommand bounded by cost | provider egress documented per config; differential eval is opt-in with recorded cost/latency | distributed-compute CLAIM-7, UPGRADE-3; rules-policy P3 |
| RP-7 | model-lane drift (gpt-4o vs claude-opus-4-8 vs free-text anthropic; claude/claude-opus CI alias) | model | PROPOSE | canonical model-id registry; encode no-downgrade; pin dated/explicit ids so a single edit updates all lanes | a model-lane policy doc + registry reconciles the three lanes; CI ids are dated/pinned | prompt-architecture §3; governance gov-012 |
| RP-8 | DB store path forks by CWD (only `init` honors `--path`) | destructive | PROPOSE | resolve a single canonical store path (`--db`/`$PROMPTHUB_DB`/HubConfig/XDG); default MUST preserve today's resolution (no-downgrade) so existing invocations don't silently re-target | `init --db X; add; list` all hit the same store; default path unchanged for existing users | verdicts.md UPGRADE D; filesystem FL-1 |
| RP-9 | preserve the libsql/prometheus feature-trims (dodge RUSTSEC) | provider | APPLY | the trims that drop the advised rustls-webpki-0.102 / protobuf chains MUST NOT silently regress | a `cargo deny`/regression guard fails if libsql/prometheus default features are re-enabled | verdicts.md VERDICT 7; trends advisories |

owner-wall: RP-1..RP-4 are SUPERVISED — they cross a trust/secret/cross-repo boundary and require human
verification before merge. RP-5..RP-9 are additive PROPOSE/APPLY and never weaken an existing guard.
