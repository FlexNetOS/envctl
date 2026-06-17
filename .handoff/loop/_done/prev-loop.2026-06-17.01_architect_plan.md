# TASK-0026 — `secretctl github-app enroll`  (VERDICT: GO)

Seal the GitHub App credential into the unlocked vault so TASK-0020's `Engine::mint_github_token`
can read it. Write EXACTLY what the mint reads (byte-for-byte names): broker-only secret
`github-app-private-key` (PEM) + meta key `github-app-id`. installation-id is NOT enrolled (it comes
per-mint from `MintGithubReq.installation_id`). Live: app-id=4044997, installation-id=140063898, org FlexNetOS.

## Decisions (resolved from code)
1. **Reuse `Vault.Add` for the PEM** (control.proto:73 `AddSecretReq.broker_only=6` → conv.rs:131 → `secret_put{broker_only}`). **Add ONE new minimal RPC for app-id**: `Engine::put_github_app_id` (lib.rs:1682, writes `put_meta("github-app-id")`, Err(Locked) if locked) exists but is wired to NO RPC — that's the gap.
2. Names (match `mint_github_token` readers verbatim): PEM secret `github-app-private-key` (lib.rs:120 `GITHUB_APP_KEY_NAME`, read via `open_real_key` lib.rs:1742); app-id meta `github-app-id` (lib.rs:122 `GITHUB_APP_ID_META`, read via `get_meta` lib.rs:1749). RECOMMENDED: export these as `pub const` from secrets-engine for secretctl to reference (kills literal-drift). NOT the `{secret_name}.app_id` relay convention — different path, out of scope.
3. PEM input `--private-key <path|->` (stdin via existing `read_stdin_bytes` main.rs:102). Validate client-side BEFORE any write by reusing `envctl_secrets::build_app_jwt` (lib.rs:40 re-export; mint_github.rs:155 parses PKCS#1 or PKCS#8) — bail on Err, NEVER echo PEM bytes. Wrap PEM in `Zeroizing` on read.
4. `--apply` gating (dry-run by default, CF-8): no --apply → validate + preview to STDERR (name, broker_only=true, app-id, installation-id reminder, optional SHA-256 fingerprint = non-secret hash), write nothing. --apply → `Vault.Add{broker_only:true}` (PEM) THEN `Vault.SetGithubAppId{apply:true}` (app-id). PEM first (no orphan app-id on failure).

## Proto delta (secrets-proto/proto/control.proto, service Vault ~line 50/81)
`rpc SetGithubAppId (SetGithubAppIdReq) returns (stream Event);`
`message SetGithubAppIdReq { string app_id = 1; bool apply = 2; }`  (stream Event matches Add/Rm/Rotate; apply=dry-run gate)

## Units (leaf-first; engine UNTOUCHED — seams exist)
1. proto: the RPC + message above (existing prost/tonic build.rs, no new dep).
2. secretd `grpc.rs`: `Vault::set_github_app_id` next to `add` (line 147). apply==false → dry-run Event, mutate nothing. apply==true → `run_streaming` calling `engine.put_github_app_id(&req.app_id)` (Locked→failed_precondition per grpc.rs:332 pattern). Empty app_id → invalid_argument. No secret logging (app-id is non-secret).
3. secretctl `cli.rs`: `Cmd::GithubApp{ #[command(subcommand)] cmd: GithubAppCmd }` + `GithubAppCmd::Enroll{ app_id:String (--app-id), private_key:String (--private-key, path or "-"), apply:bool }`.
4. secretctl `main.rs`: `Cmd::GithubApp` dispatch (~line 191) + `github_app` fn: read+Zeroize PEM (file or `-`), validate via `build_app_jwt` (bail, no byte echo), dry-run preview→stderr, --apply → Add{name "github-app-private-key", provider github, value pem, broker_only:true, overwrite:false} then SetGithubAppId{app_id, apply:true}, drain each Event stream. Fingerprint only if it needs NO new secretctl dep (else omit).
5. tests: secretd handler (empty app_id→invalid_argument; dry-run no-mutate; locked→failed_precondition); secretctl cli parse (mirror MintGithub parse tests main.rs:885; `-` stdin); **ROUND-TRIP e2e (load-bearing)** in secretd tests: init+unlock → enroll (Add broker-only test PKCS#1 key from mint_github.rs:340 + SetGithubAppId "4044997") → `Vault.MintGithub` (mock via ENVCTL_GITHUB_API_BASE) SUCCEEDS reading exactly what enroll wrote; broker-only-refusal test (`secret get github-app-private-key --reveal` REFUSED post-enroll); negatives (non-PEM → no write; locked → failed_precondition, nothing written).

## Invariants
No-C (ZERO new dep; reuse rsa/build_app_jwt + prost/tonic). One rustls ring-only (transport untouched). Engine non-printing/untouched; secretd+secretctl thin; NEVER print PEM/secret bytes (only metadata to stderr). Fail-closed + dry-run-by-default (locked/bad-PEM/missing-arg/no-apply → nothing written). Broker-only PEM (secret get --reveal REFUSES it). Audit metadata-only.

## Sequencing
proto → secretd handler+tests → secretctl cli+dispatch → round-trip/refusal/negative tests → fmt/clippy --workspace -D warnings/test --workspace → no-c.sh + shape.sh.

## Risks
Name drift (mitigate: pub const from engine + round-trip e2e is the gate). installation-id confusion (document: enroll does NOT take it). fingerprint dep creep (drop if it adds a secretctl dep). proto regen blast radius low (single server impl, secretctl sole client).
