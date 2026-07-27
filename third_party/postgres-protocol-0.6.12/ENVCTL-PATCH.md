# envctl patch: postgres-protocol 0.6.12

This directory is the exact `postgres-protocol` 0.6.12 crates.io source with
one dependency-feature hardening change. No Rust source or wire-protocol logic
is modified.

- Crates.io archive checksum:
  `08808e3c483c46e999108051c78334f473d5adb59d78bb80a1268c7e6aa6c514`
- Upstream repository: `https://github.com/rust-postgres/rust-postgres`
- Upstream source revision recorded by the crate:
  `76062c9b242da6aada065c021aa3083d0922f7d2`
- License: `MIT OR Apache-2.0` (`LICENSE-MIT`, `LICENSE-APACHE`)

## Maintained change

`sha2` 0.11 is declared with `default-features = false` in both the normalized
and original manifests. `postgres-protocol` uses fixed-size SHA-256/HMAC
operations for authentication and does not use SHA-2's optional allocation
surface.

Without this patch, Cargo feature unification enables SHA-2 `default,alloc`
through the PostgreSQL client and adds those features to envctl's separately
audited fixed-width GitHub RS256 signer. The fail-closed `ci/gates/no-c.sh`
gate correctly rejects that drift. The patch keeps current
`postgres`/`tokio-postgres`/`postgres-protocol` versions and the complete
PostgreSQL authentication capability; it does not downgrade dependencies or
weaken the signer gate.

## Verification

From the envctl repository root:

```bash
cargo test -p envctl-commit-worker --features pg-integration --locked
cargo tree --locked -i sha2@0.11.0 -e features
bash ci/gates/no-c.sh
```

The reverse feature tree must show `sha2 0.11` with `oid` only, the committer
tests must pass against disposable PostgreSQL, and the no-C/single-backend gate
must pass.
