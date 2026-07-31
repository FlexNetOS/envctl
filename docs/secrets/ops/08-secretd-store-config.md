# env-ctl ops — secretd store backend configuration (OI-1 (a), Phase 1)

**Reads with:** `DESIGN-NOTES.md` (OI-1), `07-ci-supplychain.md` (no-C gate), `ARCHITECTURE.md`
(FS-S4/FS-S7), `crates/secrets-store-libsql/README.md`.

**Scope:** how `secretd` selects and connects its persistence backend. `secretd` runs the engine on
one of two `Store` backends behind the identical `vault::Store` trait:

| Backend | What it is | When |
|---|---|---|
| `inmem` (default) | RAM-only vault; lost on restart | tests/CI, first-run, or before a DB is provisioned |
| `libsql` | the durable libSQL `remote` store, talking HTTP/Hrana to a **loopback** `sqld` | production durability (OI-1 (a)) |

The engine, proto, and CLI never link libSQL; only `secretd` does (and only `secretd` carries the
libSQL dependency stack). The no-C-**library** tenet still holds — `ci/gates/no-c.sh` proves it.

## 1. Configuration

Precedence (highest first): **environment variables > the TOML file > defaults** (`inmem`).

### 1.1 TOML file — `$META_ROOT/.config/env-ctl/secretd.toml` (optional)

```toml
[store]
backend = "libsql"            # "inmem" (default) | "libsql"
url     = "http://127.0.0.1:8080"   # the LOOPBACK sqld (see §2). https/remote is refused (see §3).
# NOTE: the auth token is a credential and is NEVER read from this file — see §1.3.
```

Override the file location with `SECRETD_CONFIG=/path/to/secretd.toml`. A missing file is fine
(defaults apply).

### 1.2 Environment variables (override the file)

| Var | Meaning |
|---|---|
| `SECRETD_STORE_BACKEND` | `inmem` \| `libsql` |
| `SECRETD_LIBSQL_URL` | the loopback sqld URL, e.g. `http://127.0.0.1:8080` |
| `SECRETD_LIBSQL_AUTH_TOKEN` | the libSQL/sqld auth token (JWT), if the server requires one |
| `SECRETD_LIBSQL_AUTH_TOKEN_FILE` | path to a **`0600`** file holding the token (preferred over the inline var) |
| `SECRETD_CONFIG` | override the TOML path |

### 1.3 Auth-token hygiene

The token is a credential, so it is **never** taken from the TOML file. Provide it via
`SECRETD_LIBSQL_AUTH_TOKEN` through the Yazelix-owned profile environment or, preferably, via
`SECRETD_LIBSQL_AUTH_TOKEN_FILE` pointing at a **`0600`** file — a group/other-readable token file is
**refused** (fail-closed). The **config-layer** token copy is held in a zeroizing buffer and never
logged (the config's `Debug` redacts it); note the downstream libSQL client takes a plain `String`
(its public API) and keeps its own non-zeroized copy for the connection's lifetime. The low-level
configuration parser still accepts an empty token for an explicitly started development server,
but the envctl-managed production units and real-server test runner always require JWT auth.

## 2. Standing up a loopback sqld (Profile A — recommended)

`sqld` (a.k.a. `libsql-server`) is run **on loopback**, co-located with `secretd`. Install the
managed components rather than starting an open-auth server by hand:

```sh
yzx launch
```

The Yazelix stack bootstrap starts the pinned server with `--auth-jwt-key-file`, records the exact
process identity, and requires authenticated readiness before dependents start. It then launches
`secretd`, opens the USB-bound vault, and starts downstream tools only after the vault proves both
unlocked state and USB possession. A missing, open-auth, unreachable, JWT-incompatible, or
wrong-owner server therefore fails closed instead of reporting a false-ready process. No manual
migration step is needed.

## 3. Transport: loopback-only, or a loopback TLS terminator for a remote DB

`secretd`'s libSQL client uses a **plaintext** HTTP connector. This is deliberate and gate-clean:
libSQL's `tls` feature would pull a **second** rustls (`hyper-rustls 0.25 → rustls 0.22`) alongside
the workspace's single ring-only `rustls 0.23`, breaking the no-C / single-rustls gate (DESIGN-NOTES
OI-1), and there is no hyper-0.14 `hyper-rustls` on rustls 0.23. Therefore:

- **Accepted:** `http`/`ws` to a **loopback** host (`127.0.0.0/8`, `::1`, `localhost`).
- **Refused (fail-closed):** plaintext to a non-loopback host (FS-S7 — the auth token + metadata +
  write-integrity would cross the network in the clear).
- **Refused with guidance:** `https`/`wss`/`libsql` URLs. For a **remote** DB (Turso, a remote sqld),
  run a **loopback TLS terminator** and point `secretd` at it:

  ```sh
  # e.g. stunnel / spiped / cloudflared, listening on 127.0.0.1:8080 and TLS-forwarding to the remote
  SECRETD_LIBSQL_URL=http://127.0.0.1:8080   # -> terminator -> https://<remote-sqld>
  ```

  This keeps the daemon's dependency graph gate-clean while still encrypting the off-box hop. (A
  future opt-in `remote-tls` build that accepts the second rustls is possible but is NOT enabled —
  it would fail the single-rustls gate by design.)

## 4. Durability & resilience

- **Durability** is the server's responsibility for the remote backend: `sqld` persists each write to
  its WAL (durable by default), and the store's `fsync_barrier` (a `SELECT 1` round-trip after each
  write) confirms the prior statement was applied by the server before success is reported (HF-14). A
  client-side `PRAGMA synchronous=FULL` is not issued (Hrana rejects `PRAGMA`).
- **Hrana stream-expiry:** a libSQL `remote` connection's Hrana stream baton is expired by `sqld`
  after a short idle window. The engine interleaves slow CPU work between store ops — notably argon2id
  during `init_vault` (seconds–tens of seconds) — so the store **reconnects once and retries** on a
  `STREAM_EXPIRED` (the retried statements are idempotent and an expiry means the prior attempt never
  committed). This makes `init_vault`/`unlock` on libSQL transparent to the operator.

## 5. Verification

The libSQL path has real-server coverage through one hermetic runner. It downloads and verifies the
exact pinned sqld release, generates a fresh Rust-native JWT pair for each suite, requires an
unauthenticated SQL request to return `401`, then requires an authenticated `SELECT 1` before running
the ignored tests against fresh databases:

```sh
bash ci/run-live-libsql-tests.sh
```

The runner covers both the Store implementation and the engine-over-libSQL durability E2E
(init/unlock/put/get plus persistence across engine instances). The default `cargo test --workspace`
keeps these `#[ignore]`d (no sqld needed) and stays green; `ci/gates/no-c.sh` confirms the libSQL
stack adds no C **library** and keeps the single ring-only rustls.
