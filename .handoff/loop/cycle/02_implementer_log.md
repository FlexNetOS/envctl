# TASK-0039 — implementation log · STATUS: GREEN

## Result

Implemented the remote-clients CA lifecycle that remained after PR #158:

- Added a DEK-sealed `remote-clients CA` that is distinct from the local MITM CA.
- Rebuilds the remote-clients issuer on unlock and zeroizes it on lock alongside the existing CA.
- Keeps `control_plane_client` leaves at `ttl_days <= 7`; the default client-leaf TTL is now 7 days.
- Persists certificate revocation state in both the in-memory store and libSQL store.
- Implements `Engine::ca_renew` and `Engine::ca_revoke`.
- Wires `Certs.Renew` and `Certs.Revoke` through `secretd`.
- On revoke, disables a matching remote-client registry row and appends revoked certificate
  SHA-256 DER fingerprints to the configured `client_revocations_path` for the PR #158 verifier.

## Deviations

- The existing frozen proto/CLI surface does not return client private key material from
  `Certs.Issue`. This cycle closes the shipped lifecycle surfaces (CA separation, <=7d mint,
  renew, revoke, registry disablement, verifier revocation propagation), but a future enrollment
  packet/export surface would be needed for full out-of-band device provisioning.

## Verification

- `cargo test -p envctl-secrets-engine ca_ -- --nocapture`
- `cargo test -p envctl-secretd append_revoked_client_fingerprints_writes_verifier_format -- --nocapture`
- `cargo test -p envctl-secrets-store-libsql bind_cert_row_shape -- --nocapture`
- `cargo check -p envctl-secretd --features relay-edge`
- `cargo check -p envctl-secretctl`
- `cargo build -p envctl-engine -p envctl`
- `cargo fmt --all -- --check`
- `cargo clippy --workspace -- -D warnings`
- `bash ci/gates/no-c.sh`
- `bash ci/gates/shape.sh`
- `bash ci/gates/enable.sh`
- `bash ci/gates/p7.sh && bash ci/gates/loop-state.sh`
- `cargo test --workspace`

All checks passed locally.
