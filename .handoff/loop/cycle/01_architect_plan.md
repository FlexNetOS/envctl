# Architect plan — TASK-0075b (relocate Seed Device-CA pin off /usr/local to a meta path)

VERDICT: GO. Pure-manifest, zero Rust change (the `ENVCTL_SEED_CA` override seam already exists at
`crates/secrets-engine/src/seam.rs:113-116`). Biggest risk (systemd sandbox reachability) is already
mitigated: the secretd unit uses `ProtectHome=read-only`, which makes `/home/.../meta` readable to the
daemon; the daemon only READS the CA. A belt-and-suspenders `ReadOnlyPaths=` is added to survive any
future `ProtectHome=tmpfs` hardening.

## Chosen meta path
`%h/Desktop/meta/.toolchains/secrets/ca/cognitum-ca.crt` in the unit (systemd `%h` specifier, matching
the existing `ExecStart=%h/Desktop/meta/.toolchains/secrets/bin/secretd` idiom); worker default
`${META_ROOT:-$HOME/Desktop/meta}/.toolchains/secrets/ca/cognitum-ca.crt`. Co-locates with the secretd
binaries already installed under `.toolchains/secrets/bin`. Distinct from the daemon's own relay CA in
the vault data dir (`~/.local/share/env-ctl/ca`).

## Unit ledger (completeness contract)
| U# | Change | File::location | Wiring |
|----|--------|----------------|--------|
| U1 | `Environment=ENVCTL_SEED_CA=%h/Desktop/meta/.toolchains/secrets/ca/cognitum-ca.crt` | `manifest/env-ctl.toml` secretd `[Service]` | systemd injects env → `seam.rs::ca_path()` reads it |
| U2 | `ReadOnlyPaths=%h/Desktop/meta/.toolchains/secrets/ca` | same `[Service]` | re-exposes CA dir read-only (future-proof vs ProtectHome tightening). NOT ReadWritePaths — daemon must never overwrite its trust root |
| U3 | Worker DST default → meta path in BOTH WORKER heredocs (install+fix), the verify probe DST, the `[[component]]` description, and header comments | `manifest/cognitum-seed-trust.toml` | `${ENVCTL_SEED_CA:-…}` override still honored; `install -d "$(dirname "$DST")"` auto-creates `…/ca/`; idempotence + absent-Seed no-op preserved |
| U4 | Regenerate lock (env-ctl + cognitum-seed-trust content_hash) | `manifest/envctl.lock` | `envctl lock`; CI `lock --check` |
| U5 | (no Rust) assert `seam.rs::ca_path()` unchanged | `crates/secrets-engine/src/seam.rs:113` | already reads `ENVCTL_SEED_CA` |

## Invariants: all PASS (no C; one rustls ring-only; engine untouched; fail-closed/additive; frozen-roots — only file LOCATION changes, still pins ONLY the Cognitum CA).

## Runtime surface (guardian drives — NO physical Seed needed)
1. Env injection: place a dummy PEM at the meta path, restart the unit, `systemctl --user show env-ctl.service -p Environment` must list `ENVCTL_SEED_CA=…meta…/cognitum-ca.crt`.
2. Sandbox readability (crux): `systemd-run --user -p ProtectHome=read-only -p ProtectSystem=strict cat <meta CA path>` must print the PEM.
3. `/usr/local` no longer the source: Environment shows the meta path; worker verify probe (with `COGNITUM_TRUST_DIR` fixture) compares against the meta DST.

## Verification: cargo run -p envctl -- auto-detect parses both TOMLs; `envctl lock` regen + `lock --check` exit 0; run ci/gates/{enable,no-c,shape}.sh green.
