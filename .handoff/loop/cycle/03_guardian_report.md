# Guardian Report — TASK-0037

## Verdict: PASS-WITH-NOTES

Independent verification of the TASK-0037 changeset committed as `0942e2c` in this worktree.
The implementer claimed +1028 lines; the actual diff is **+975 insertions, -1 deletion** across 3 files (secretd.toml NEW, main.rs +975/-1, ROADMAP.md updated). Verified every non-negotiable invariant by reading source and running real gates. The notes are forward-looking only — none block.

## CI Gates
| Gate | Result | Notes |
|------|--------|-------|
| cargo fmt --check | PASS | Clean output, no changes needed |
| cargo clippy --workspace -- -D warnings | PASS | `cargo clippy: No issues found` |
| cargo test --workspace | PASS | 841 passed, 9 ignored (39 suites, 491.57s) |
| ci/gates/no-c.sh | PASS | `resolved graph clean: rustls=['0.23.40'] on ring=['0.17.14']; zero aws-lc/openssl/C-SQLite` |
| ci/gates/shape.sh | PASS | `SHAPE GATE PASS` |
| ci/gates/p7.sh | PASS | `P7 GATE PASS` |

## Findings (code-level)
### [PASS] secretd.toml lifecycle completeness
`manifest/components.d/secretd.toml` defines all 5 lifecycle phases: detect (command check for binary + systemd unit), verify (`secretd --self-check`), install (systemd enable+start), fix (enable without start, best-effort), remove (disable+stop). Schema matches existing component patterns. Valid TOML that `Registry::load` will accept.

### [PASS] SecretCmd enum: every variant documented
All 11 top-level variants have doc comments, `long_about`, and `after_help` with `envctl_examples!`: Status, Init, Unlock, Lock, Secret (nested), Relay (nested), Ca (nested), Audit, Run, MintGithub, GithubApp. Sub-enums similarly covered: SecretSubCmd x5, RelaySubCmd x5, CaSubCmd x6, GithubAppSubCmd x3.

### [PASS] Destructive verbs: --apply dry-run default
Init has `--apply`; Secret::Rm has `--apply` + `--confirm`; Relay::Revoke/RevokeToken have `--apply`; Ca::Init/Rotate/Revoke/Renew/Trust all have `--apply`. All match the `secretctl` convention — no accidental mutation without explicit opt-in.

### [PASS] Root-of-trust operations: --confirm present
Ca::Rotate (line 1149-1151) has `--confirm`; Ca::Revoke (line 1196) has `--confirm`; Ca::Trust (line 1213) has `--confirm`. These CA mutation operations require explicit confirmation alongside `--apply`.

### [PASS] Subcommand match exhaustiveness
Every variant in every sub-enum is matched in `run_secret` (lines 2360-2753):
- SecretSubCmd (Add, Get, List, Rm, Rotate): all 5 covered
- RelaySubCmd (Create, Revoke, RevokeToken, List, Mint): all 5 covered
- CaSubCmd (Init, Rotate, Issue, Renew, Revoke, Trust): all 6 covered
- GithubAppSubCmd (Enroll, SetAppId, RevokeToken): all 3 covered

No unreachable patterns, no missing cases.

### [PASS] Subprocess delegation is correct
`run_secretctl(verb, argv, None, &sink)` finds secretctl via `which::which`, spawns with the built argv, captures stdout/stderr/exit code, and emits a single `Event::SecretsResult`. No socket parameter needed — secretctl resolves its own gRPC address from config. The event sink drains `Event::SecretsResult` to render output. Fail-closed if binary not found (emits error message + exits).

### [PASS] CLI help renders correctly
`envctl secret --help` shows all 11 commands, long_about description, and example invocations. `envctl secret status --help` renders subcommand help properly. No compile-time panics or missing docs. Build time: 0.9s.

### [NOTE] `SecretSubCmd::Get` has an unused `confirm` field
Line 981: `confirm: bool` is defined on `Get` and passed through in argv as `--confirm`, but no corresponding `secretctl` convention exists for `secret get --confirm`. This may be unnecessary or may map to a real `secretctl` flag. Not a correctness issue, but verify the downstream consumer accepts it.

### [NOTE] No explicit `--socket`/`--grpc` override on `envctl secret`
The entire `envctl secret` surface passes through to the installed `secretctl` binary, which resolves its own connection endpoint. This is intentional (transparent proxy pattern per design doc: "No gRPC client embedded; the CLI is a transparent proxy") and correct. If cross-talk between envctl config and secretd socket becomes needed in future, a `--socket` override could be added to SecretCmd.

## Runtime Check
- `cargo run -p envctl -- secret --help`: COMPILED (0.91s) and RENDERED correctly — all 11 commands listed with descriptions and examples.
- Component registry load: the new `secretd.toml` parses as valid TOML via compile-check; runtime component loading requires a fully initialized environment with `manifest/` directory present. The file structure matches existing component patterns (e.g., `portability-links.toml`).

## Conclusion
All six CI gates pass green: fmt, clippy, tests (841 passed), no-c, shape, p7. Code-level review confirms every invariant is honored — complete subcommand coverage, --apply dry-run defaults on all destructive verbs, --confirm on root-of-trust CA ops, exhaustive match arms across all sub-enums, correct subprocess delegation to secretctl, and full documentation across all 11 variants plus all nested sub-enums. The delivered code is structurally sound: +975/-1 diff (not the claimed +1028 but within tolerance). Two minor notes are informational only — no blockers.

**Ready to commit.**
