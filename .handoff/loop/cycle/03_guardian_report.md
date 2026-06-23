# Verification report: TASK-0075b — relocate Cognitum Seed Device-CA pin off /usr/local to meta path

## Verdict — PASS-WITH-NOTES
Independently verified from the diff at commit `97f79ca` (branch `seed-ca-meta-relocate`) and by
running the real gates. The change does exactly what it claims: pure-manifest relocation of the
Seed Device-CA pin onto `%h/Desktop/meta/.toolchains/secrets/ca/cognitum-ca.crt`, zero Rust change.
Sole reason it is PASS-WITH-NOTES (not clean PASS): the write-refusal off-happy-path could not be
exercised here because `systemd-run --user` (systemd 259 user instance) does not enforce
filesystem-namespace read-only directives — the real `env-ctl.service` is a SYSTEM unit where it
will. The crux runtime check (sandbox READ reachability) PASSED. No blocking findings.

## Gate results
- `ci/gates/no-c.sh` : **PASS** — `resolved graph clean: rustls=['0.23.40'] on ring=['0.17.14']; zero aws-lc/openssl/C-SQLite` → `NO-C GATE PASS`
- `ci/gates/shape.sh` : **PASS** — `SHAPE GATE PASS`
- `ci/gates/enable.sh` : **PASS** — `ENABLE GATE PASS` (env-ctl.toml `enable = true` intact, line 206)

## cargo
- `cargo run -p envctl -- lock --check` : **PASS** — `✓ envctl.lock matches the manifest (78 components)` (exit 0)
- fmt / clippy / test : **N/A** — zero Rust/Cargo files changed on the branch (`git diff origin/develop..HEAD` = 3 manifests + 2 handoff docs only). No source to fmt/clippy/test; no inherited red introduced.

## Invariant checks
1. No C in trust boundary — **PASS** — no Cargo.toml/Cargo.lock change on branch; no-c.sh PASS from resolved graph.
2. Code-shape — **PASS** — shape.sh PASS.
3. secretd enable — **PASS** — enable.sh PASS; `enable = true` present (env-ctl.toml:206).
4. Engine purity (non-printing/sync) — **PASS** — no engine/ change; `git diff origin/develop..HEAD` shows no `src/` files.
5. Front-end parity — **N/A** — no Engine method added (manifest-only change).
6. Fail-closed + dry-run — **PASS** — worker still additive: absent-Seed `exit 0` (lines 78,143,176); idempotent `cmp -s` (84,147,180); `install -d "$(dirname "$DST")"` auto-create (88,183); REMOVE block intentionally KEEPS the pinned CA (lines 215-218, 228-230). `destructive = false`.
7. Rust-native, no drift — **PASS** — no non-Rust source/package files added; no dep change.
8. Lock honesty — **PASS** — envctl.lock regenerated; `lock --check` exit 0; the two changed `content_hash`es (env-ctl, cognitum-seed-trust) match.
9. Kasetto/agent-env — **N/A** — no `crates/agent-env` change.
10. Runtime behavior — see Runtime check below (architect declared a `## Runtime surface`).

## Parity check
N/A — no `Engine` method added. seam.rs::ca_path() (the consumer) is untouched and already reads
`ENVCTL_SEED_CA` (crates/secrets-engine/src/seam.rs:113-116) — verified by read.

## Unit ledger
| U# | present | wired | evidence (file:line) |
|----|---------|-------|----------------------|
| U1 `Environment=ENVCTL_SEED_CA=meta path` | YES | YES — systemd injects env → seam.rs reads it | manifest/env-ctl.toml:227 (set) → crates/secrets-engine/src/seam.rs:114 (reads `ENVCTL_SEED_CA`) |
| U2 `ReadOnlyPaths=meta CA dir` (NOT ReadWritePaths) | YES | YES — in secretd `[Service]` | manifest/env-ctl.toml:242 |
| U3 worker DST default → meta path (3 DSTs + verify probe + description + header + remove comments) | YES | YES — `${ENVCTL_SEED_CA:-…meta…}` override intact; no `/usr/local` CA default remains | cognitum-seed-trust.toml:82,146,179 (DST); :43 (desc); :5,16,30,228 (comments) |
| U4 regenerate lock | YES | YES — lock --check exit 0 | manifest/envctl.lock:56,94 (content_hash updated) |
| U5 seam.rs unchanged | YES (unchanged) | YES — already reads `ENVCTL_SEED_CA` | crates/secrets-engine/src/seam.rs:113-116 (not in branch diff) |

All 5 rows present AND wired. No miss.

## Runtime check — PASS (read crux) + NOTE (write-refusal not exercisable here)
Surface declared: env injection + sandbox readability of the relocated path under the daemon's
exact hardening (`ProtectHome=read-only` + `ProtectSystem=strict`). `systemd-run --user` IS
available (systemd 259, user manager `running`).

- **Crux (sandbox READ):** wrote a dummy PEM to `~/Desktop/meta/.toolchains/secrets/ca/cognitum-ca.crt`,
  then `systemd-run --user --pty --wait -p ProtectHome=read-only -p ProtectSystem=strict cat <meta CA>`
  → printed the PEM, `status=0/SUCCESS`. **PASS** — proves U1+U2: the relocated path the daemon will
  read via `ENVCTL_SEED_CA` IS reachable under the daemon's sandbox.
- **Off-happy-path (write must be refused — NOT exercisable here, NOTE):** attempted an append to the
  trust root under `ProtectHome=read-only` (+`ReadOnlyPaths`) via `systemd-run --user` — the write
  **succeeded**. Investigated: `ProtectHome=read-only` does not enforce on the systemd 259 **user**
  instance (filesystem-namespace sandboxing requires the privileged system instance). This is a known
  systemd limitation, NOT a defect in the change. The real `env-ctl.service` is a SYSTEM unit (uses
  `%t`=/run, `LimitMEMLOCK=infinity`, system store ordering), where `ReadOnlyPaths` WILL be enforced.
  The `ReadOnlyPaths`-not-`ReadWritePaths` choice (env-ctl.toml:242 + comment 240-241) is correct by
  inspection; runtime write-refusal can only be confirmed on the box with the system unit installed.
  Test CA cleaned up after.

## Diff / shell-safety findings
- **Diff matches claims (a):** seam.rs / all Rust UNTOUCHED — `git diff origin/develop..HEAD` lists no `src/`.
- **Diff matches claims (b):** env-ctl.toml adds `Environment=ENVCTL_SEED_CA=%h/...` (227) + `ReadOnlyPaths=%h/Desktop/meta/.toolchains/secrets/ca` (242) to secretd `[Service]`. Correct `%h` specifier.
- **Diff matches claims (c):** cognitum-seed-trust.toml DST default → meta path in install worker (82), fix worker (179), verify probe (146), description (43), header doc (5,16,30), remove comments (228). `${ENVCTL_SEED_CA:-…}` override form intact everywhere; **no stray `/usr/local` CA *default* remains**.
- **`/usr/local` residuals are CORRECT:** the only remaining `/usr/local` strings are the worker BINARY path (`/usr/local/sbin/cognitum-seed-trust-refresh`) and prose — none is a CA pin DST. Relocating the worker binary was out of scope (task = relocate the CA pin).
- **Shell-safety (the bit that bit a prior cycle): PASS.** Read every `set -euo pipefail` block (install 57-120, verify 128-154, fix 160-213, remove 222-233). NO naked prose — every non-command line begins with `#`. `${USER:-}` guarded (72,137,171); `${META_ROOT:-…}` guarded (82,146,179); `${COGNITUM_TRUST_DIR:-}` guarded (68,133,167). Quoted heredocs (`<<'WORKER'`/`'UNIT'`/`'RULE'`) write literal file content (not outer-shell-executed); their `#` lines are valid comments anyway.
- **auto-detect:** both TOMLs parse, no panic/error; `cognitum-seed-trust … wired`, `env-ctl … [healthy] wired`.

## Non-blocking note
- `seam.rs::ca_path()` fallback (used only when `ENVCTL_SEED_CA` is UNSET) is still the literal
  `/usr/local/share/ca-certificates/cognitum-ca.crt` (seam.rs:115). Harmless and out of scope: the
  unit always sets `ENVCTL_SEED_CA`, so the fallback never fires; changing it would be a Rust change
  the architect explicitly scoped OUT (U5 = no Rust). Recorded for the record, not a finding.

## Re-test needed
None blocking. To confirm the write-refusal off-happy-path on the actual box (after the system unit
is installed):
```
sudo systemctl daemon-reload && sudo systemctl restart env-ctl.service
systemctl show env-ctl.service -p Environment        # expect ENVCTL_SEED_CA=…meta…/cognitum-ca.crt
sudo -u <daemon-user> bash -c 'echo x >> ~/Desktop/meta/.toolchains/secrets/ca/cognitum-ca.crt'  # under the system sandbox → must be refused
```
