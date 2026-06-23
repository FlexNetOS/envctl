# envctl Runbook — ASCII Diagrams

Runbook-grade diagrams for the ten owner-named topics. `envctl` is a pure-Rust
workstation environment manager (declarative TOML components wrapping proven bash)
**plus** a secrets vault + credential broker. It absorbed Kasetto v3.2.0 as its
built-in `envctl agent` agent-environment engine. The non-negotiable spine: **no C
library in the trust boundary** (libSQL `remote` only, ring-only rustls, pure-Rust
crypto), one shared non-printing `Engine` drives both CLI and GUI, and destructive
ops are fail-closed + dry-run by default.

Each diagram cites its source `file:section`.

---

## 1. How the vault works

The vault is an **app-layer AEAD-at-rest** store: every secret body is sealed with
XChaCha20-Poly1305 under a 32-byte Data Encryption Key (DEK). The DEK exists in
**cleartext only inside the `secretd` address space** — it is `Zeroizing`/`ZeroizeOnDrop`,
held in RAM only while `Unlocked`, and wiped on `lock` / USB-pull / drop. The DEK is
never written to disk; instead **keyslots** wrap it: a passphrase slot (KEK via argon2id,
1 GiB×t4×p4 production floor) and a USB/Seed slot (KEK via HKDF-SHA256 over the keyfile).
The at-rest store backend is libSQL's `remote` client talking to a **loopback `sqld`**, so
no C SQLite is ever linked into the trust boundary.

Two planes with different trust models meet at the daemon: the **control plane** is gRPC
over a Unix-domain socket, authorized owner-only by `SO_PEERCRED`; the **data plane** is a
loopback relay proxy authorized per-request by a ≤24h bearer. Everything leaving the TCB
carries only ciphertext or a short-lived bearer.

*Source: docs/secrets/ARCHITECTURE.md §1 (two planes), §5 (key hierarchy), §11 (XDG);
docs/secrets/DESIGN-NOTES.md R9/OI-1 (libSQL remote, no-C); crates/secrets-engine/src/vault/mod.rs;
crates/secrets-engine/src/keyslot.rs (Dek/Kek, Factor, argon2id/HKDF).*

```
                          secretd  =  THE SOLE TCB
   ┌──────────────────────────────────────────────────────────────────────┐
   │   plaintext DEK + real upstream keys exist ONLY here                   │
   │   Zeroizing · mlockall · RLIMIT_CORE=0 · MADV_DONTDUMP                 │
   │                                                                        │
   │   ┌─ KEYSLOTS (wrap the ONE DEK) ─────────────────────────────────┐   │
   │   │  passphrase slot : KEK = argon2id(pass)  ──┐                   │   │
   │   │  usb / seed slot : KEK = HKDF(keyfile)   ──┤ unwrap            │   │
   │   │                                            ▼                   │   │
   │   │                                       ┌─────────┐              │   │
   │   │                                       │  DEK    │ (RAM only)   │   │
   │   │                                       └────┬────┘              │   │
   │   └────────────────────────────────────────── │ ──────────────────┘   │
   │                                                ▼ AEAD seal/open        │
   │   ┌─ vault store (libSQL `remote`, NO C linked) ─────────────────┐     │
   │   │  XChaCha20-Poly1305 ciphertext + non-secret metadata only    │     │
   │   └──────────────────────────────────────────────────────────────┘    │
   └───────┬─────────────────────────────────────────────────┬────────────┘
   CONTROL │ gRPC over UDS                          DATA plane │ loopback relay proxy
   (0700/0600 control.sock)                         (HTTP/HTTPS, ≤24h bearer
    authz: SO_PEERCRED uid==owner                    validated per request,
           ─ owner only ─)                           peer-bound at swap)
           │                                                   │
           ▼                                                   ▼
     CLI / envctl (secretctl)                       same-box semi-trusted clients
                                                              │
                                                    ┌─────────▼──────────┐
                                                    │ loopback sqld      │  ← C SQLite
                                                    │ (separate process) │     lives HERE,
                                                    └────────────────────┘     not in the TCB
```

---

## 2. The flow of data

A request enters `secretd` over the gRPC control plane (UDS) or the loopback data plane.
Vault/CRUD and `mint-github` go through the control plane; credential **swaps** ride the
data plane through `relay_swap`. Every path that touches a secret requires the vault to be
`Unlocked` (DEK present); a `Locked` vault answers metadata only and refuses reveals/mints.
The store sees only ciphertext, so a request is: gRPC → engine `decide()`/CRUD → DEK
decrypt/encrypt → libSQL `remote` → loopback `sqld`. Security outcomes are written to the
**durable, hash-chained `audit_log` before the RPC returns** (not on the lossy event stream).

The lock/unlock state machine has three states; the DEK is in RAM **only** in `Unlocked`.
`lock` is the true panic-stop: it zeroizes the DEK and the in-RAM CA issuer.

*Source: crates/secretd/src/grpc.rs (Vault/Relay/MintGithub RPCs); crates/secrets-engine/src/error.rs
(VaultState: Locked / LockedNeedPassphrase / Unlocked); crates/secrets-engine/src/vault/mod.rs;
docs/secrets/ARCHITECTURE.md §1/§3 (durable audit before return).*

```
   client ──gRPC/UDS──▶ secretd ──▶ engine decide()/CRUD ──▶ DEK ──▶ libSQL remote ──▶ sqld
   (secretctl)            │            (default-deny)       en/decrypt   (ciphertext)   (loopback)
                          │
                          └──▶ durable hash-chained append_audit()  (BEFORE the RPC returns)


   LOCK / UNLOCK STATE MACHINE  (DEK in RAM only when Unlocked)
   ┌──────────┐   unlock: USB-first   ┌──────────────┐
   │  Locked  │──────────────────────▶│  Unlocked    │
   │ (no DEK) │   else passphrase     │  { dek }     │
   └────┬─────┘                       └──────┬───────┘
        │ USB absent + USB slot only         │  lock / USB-pull / idle / drop
        ▼                                     │     → zeroize DEK + CA issuer
   ┌────────────────────┐                     ▼
   │ LockedNeedPassphrase│◀───────────────  Locked
   └────────────────────┘
```

---

## 3. The secret relay engine + secret injection flow

`envctl secret run -- <cmd>` is a **fork/exec wrapper**, not a shell mutation: it mints a
≤24h, peer-bound relay bearer for the child, asks the daemon for a provider-shaped env delta
(`ResolvedInjection`), overlays only those keys onto a clone of the parent env, and `execvp`s
the child. **The real key never enters the child env, argv, shell history, logs, or git** — the
child holds only a bearer, and the **real key is swapped in only at the proxy's egress** inside
the `decide()==Allow` branch (`relay_swap → DaemonUpstream::send`, `Zeroizing`).

Two injection styles cover both client classes (the auto-inject seam): **base-URL repoint**
(Claude/OpenAI SDKs get `*_BASE_URL`=loopback proxy + `*_API_KEY`=bearer) and **HTTPS_PROXY /
CONNECT MITM** for hardcoded-host tools (`git`/`curl`) — the proxy MITM-terminates their TLS
with a vault-backed per-host leaf trusted via the injected CA bundle, reads plaintext, and
swaps at egress to the canonical upstream host. The relay edge uses in-process ring-only TLS.

*Source: see [[secrets-auto-inject-seam]]; docs/secrets/AUTO-INJECT-STATUS.md (5 PRs #51/#58/#60/#63/#69);
docs/secrets/ARCHITECTURE.md §6 (swap modes), §9 (auto-inject); crates/secrets-engine/src/inject.rs;
crates/secretd/src/proxy.rs (DaemonUpstream, mod mitm); crates/secretctl/src/main.rs (Cmd::Run).*

```
   envctl secret run -- claude -p "hi"
        │  1. mint ≤24h peer-bound bearer (child-pid scoped)
        │  2. ask daemon for ResolvedInjection (provider env delta)
        ▼
   ┌──────────── child process (only ever holds the BEARER) ───────────┐
   │  ANTHROPIC_BASE_URL = http://127.0.0.1:<port>   (repoint mode)     │
   │  ANTHROPIC_API_KEY  = <≤24h bearer>                                │
   │   ── or ── HTTPS_PROXY = 127.0.0.1:<port> + injected CA bundle     │
   └───────────────────────────┬──────────────────────────────────────┘
           bearer + request    │
                               ▼
   ┌──── secretd loopback relay proxy ─────────────────────────────────┐
   │  decide()  default-deny ──▶ Allow ?                                │
   │      │ no                        │ yes                             │
   │      ▼                           ▼                                 │
   │   403 + durable      fetch REAL key (Zeroizing) ── swap at egress  │
   │   audited deny       │  re-originate verified TLS (webpki-roots)   │
   └──────────────────────┼────────────────────────────────────────────┘
                          ▼  real key NEVER returns to the child
                 real upstream (api.anthropic.com — host-allowlisted)
```

---

## 4. The secret rotations

Four rotation lifecycles live in the vault. **DEK rotation/rekey** is full O(all-secrets)
re-encryption under one atomic, resumable (`rotation_in_progress`) transaction: every
ciphertext row is decrypted with the OLD DEK and re-sealed with the NEW DEK + fresh
nonce/AAD bound to the new generation, every keyslot's `wrapped_dek` is rewritten, the
generation in `meta` is advanced, and only then is the old DEK dropped. (Passphrase rotation
is the cheap one-blob keyslot rewrite — no re-seal.) **Relay bearers** are always ≤24h
(`MAX_BEARER_TTL_SECS=86400`, single `clamp_ttl` choke point) and are re-minted on demand,
gated on USB possession re-checked at swap time. **GitHub App installation tokens** are minted
per-call from the sealed PEM (RS256 App-JWT → exchange), short-lived, and early-revocable. The
**local CA** issues short-validity leaves (MITM ≤24h relay-bound; control/remote-client leaves
≤7d/≤90d) revoked via relay-disable + cache evict; short TTL replaces CRL/OCSP.

*Source: docs/secrets/ARCHITECTURE.md §5 (DEK rotation = full re-seal), §7 (24h TTL + clamp_ttl),
§8 (CA short validity, revoke=relay-disable); docs/secrets/GITHUB-TRANSPORT-DOCTRINE.md §3 (mint/revoke);
crates/secrets-engine/src/mint_github.rs; schema.rs (meta: dek_generation; cert_revocations).*

```
   DEK rekey (full re-seal, atomic+resumable)        Relay bearer (≤24h)
   ┌───────────────────────────────────────┐         mint ─▶ live ─▶ expire(≤24h) ─▶ re-mint
   │ rotation_in_progress = 1               │              ▲                 │ (USB re-checked
   │ for each row: dec(OLD) → enc(NEW,gen+1)│              └── revoke / lock ─┘    at swap)
   │ rewrite every keyslot.wrapped_dek      │
   │ meta.dek_generation = gen+1            │         GitHub App installation token
   │ drop OLD DEK  (last)                   │         sealed PEM → RS256 App-JWT → install-token
   └───────────────────────────────────────┘              ─▶ short-lived ─▶ expire / early-revoke

   CA leaf (short validity, no CRL)
   issue (relay-bound) ─▶ valid ─▶ relay disable → cache evict + no re-mint  ≡  revoked
                                   (decide() still runs per-request → 403)
```

---

## 5. The way in and out of the vault

**Unlock is USB-first.** `secretd` POSTs to the Cognitum Seed custody-sign endpoint
(`https://169.254.42.1:8443/api/v1/custody/sign`) with the USB-minted seed-token bearer; the
Seed's Ed25519 device key (private key never leaves the Pi Zero 2 W) **deterministically signs**
the PARTUUID-bound context `envctl/usb-kek/v1/{partition_uuid}`. The 64 signature bytes are the
IKM → HKDF-SHA256 → KEK_usb → AEAD-unwrap the USB slot's `wrapped_dek` → live DEK. The Seed
**holds no vault material** — it only signs; the wrapped DEK and sealed secrets live in the libSQL
store. If USB possession is unproven, unlock falls back to the **passphrase slot** (argon2id →
KEK_pp → unwrap). **Out:** `lock` zeroizes the DEK and the in-RAM CA issuer (the true panic stop);
USB-pull auto-relocks after a short drain grace. Reboot persistence is reachability-only — the
`cognitum-seed-net` NM profile re-asserts host `169.254.42.2/24`; the DEK never persists unlocked.

*Source: docs/secrets/ARCHITECTURE.md §5 (USB-KEK via HKDF, passphrase via argon2id), §7 (USB-pull
auto-relock); crates/secrets-engine/src/seam.rs (seed_factor: custody/sign, pinned Cognitum CA,
seed-token bearer, deterministic Ed25519); [[cognitum-seed-usb-unlock]]; keyslot.rs (kek_from_usb).*

```
   ── WAY IN (unlock) — USB-first ──────────────────────────────────────────────
   secretd ──POST custody/sign (seed-token bearer)──▶ Cognitum Seed (Pi Zero 2 W)
     │   over USB link-local, pinned Cognitum CA           │ Ed25519 device key
     │                                                     │ (private key never leaves)
     │◀── 64 sig bytes  (context: envctl/usb-kek/v1/{partition_uuid}) ◀──┘
     ▼
   HKDF-SHA256(sig) ─▶ KEK_usb ─┐
                                ├─▶ AEAD-unwrap  wrapped_dek  ─▶  DEK (RAM)  ─▶  Unlocked
   passphrase ─▶ argon2id ─▶ KEK_pp ─┘  (fallback path when USB possession unproven)

   keyslot → KEK → DEK unwrap chain:
        unlock factor ──▶ KEK ──▶ unwrap(wrapped_dek) ──▶ DEK ──▶ per-record AEAD open

   ── WAY OUT (lock) ───────────────────────────────────────────────────────────
   lock / USB-pull(+grace) / idle  ──▶  zeroize(DEK) + zeroize(CA issuer)  ──▶  Locked
   reboot: cognitum-seed-net re-asserts 169.254.42.2/24 (reachability only; DEK NOT persisted)
```

---

## 6. What content is stored in the vault

The libSQL store holds a small, fixed schema — all ciphertext + non-secret metadata, no
plaintext, no DEK in any column. `secrets` rows carry an encrypted body (`nonce`+`ct_tag`),
versioned, with a `broker_only` flag (broker-only secrets refuse plaintext reveal). The
GitHub App credential set lives here: the **private key is a sealed PEM, broker-only**, beside
the app-id / installation-id / client-id / webhook-secret and other provider keys (e.g.
`brain-api-key`). `keyslots` hold the wrapped DEK per factor; `audit_log` is **hash-chained**
(`prev_hash`/`row_hash`) and tamper-evident; `certs`/`cert_revocations` are the local CA;
`relay_policies`/`relay_bearers`/`remote_clients` back the broker; `meta` is the KV for
`vault.header_mac`, `vault.dek_generation`, `github-app-id`, etc.

*Source: crates/secrets-store-libsql/src/schema.rs (DDL: meta, secrets, keyslots, audit_log,
relay_policies, relay_bearers, remote_clients, certs, cert_revocations);
docs/secrets/GITHUB-TRANSPORT-DOCTRINE.md §3 (sealed PEM broker-only).*

```
   libSQL store (loopback sqld) — ciphertext + metadata ONLY
   ┌─────────────────────────────────────────────────────────────────────────┐
   │ secrets        row_id│name│version│provider│broker_only│dek_gen│nonce│ct_tag│
   │   • github-app-private-key   [sealed PEM, broker_only=1]                  │
   │   • github-app-id · -installation-id · -client-id · -webhook-secret       │
   │   • brain-api-key  · (other provider keys)                                │
   ├─────────────────────────────────────────────────────────────────────────┤
   │ keyslots       factor│kdf_json│salt│usb_partition_uuid│wrap_nonce│wrapped_dek│
   ├─────────────────────────────────────────────────────────────────────────┤
   │ audit_log      seq│ts│event_type│subject│outcome│ prev_hash → row_hash      │  ← hash-chained,
   ├─────────────────────────────────────────────────────────────────────────┤     tamper-evident
   │ certs / cert_revocations          (local CA leaves + revocation set)       │
   │ relay_policies / relay_bearers    (broker policy + ≤24h bearers)           │
   │ remote_clients                    (DPoP jkt, hardware_bound — Phase 8)     │
   │ meta (KV)      vault.header_mac · vault.dek_generation · github-app-id     │
   └─────────────────────────────────────────────────────────────────────────┘
```

---

## 7. envctl CLI use, features, flow

The `envctl` binary is a thin clap front-end over the shared `Engine`. Top-level verbs:
`auto-detect`, `install`, `doctor`, `auto-fix`, `reset`, `add-repo`, `graph`, `registry`,
`lock`, `env`, `dashboard`, `agent`, `secret`, `self`, `completions`. The env-manager core is a
declarative component lifecycle: each TOML component holds the proven bash in five phase hooks
**Detect → Install → Verify → Fix → Remove**, run in deterministic topological order (Kahn,
ties by declaration order). `auto-detect` is read-only (returns `EnvReport`); `install` is
additive (acts by default); `auto-fix`/`reset`/`add-repo` are **dry-run by default** and need
`--apply` (and `reset --all` needs `--confirm`). A failing hook is recorded and the run continues
(best-effort `RunSummary.fail[]`), never an abort.

*Source: crates/cli/src/main.rs (enum Cmd: AutoDetect/Install/Doctor/AutoFix/Reset/AddRepo/Graph/
Registry/Lock/Env/Dashboard/Agent/Secret/Self/Completions); docs/ARCHITECTURE.md §3 (Component model),
§5 (verb→phase mapping), §6 (topo order), §12 (safety/dry-run).*

```
   envctl <verb>  (thin clap → shared Engine; drains Event stream, --json = NDJSON)
   ┌───────────────────────────────────────────────────────────────────────────┐
   │ auto-detect → EnvReport (read-only)      add-repo  → 9-stage build-from-src │
   │ install     → additive (acts)            graph/registry → inspect manifest  │
   │ doctor      → diagnostics                lock      → committed envctl.lock   │
   │ auto-fix    → repair  (dry-run default)  env       → env vars               │
   │ reset       → remove  (dry-run default)  dashboard → zellij mission-control │
   │ agent       → Kasetto agent-env engine   secret    → vault / relay / ca / run│
   │ self · completions                                                          │
   └───────────────────────────────────────────────────────────────────────────┘

   Component lifecycle (proven bash wrapped per phase; topo order):
   ┌────────┐  ┌─────────┐  ┌────────┐  ┌──────┐  ┌────────┐
   │ Detect │─▶│ Install │─▶│ Verify │─▶│ Fix  │─▶│ Remove │   guards refuse on ambiguity;
   └────────┘  └─────────┘  └────────┘  └──────┘  └────────┘   destructive ⇒ PREVIEW, need --apply
   read-only    acts        0=healthy    dry-run     dry-run
```

---

## 8. provider/model envctl integration

envctl wires AI providers/models on two axes. **Config baseline:** the `envctl agent`
agent-env engine provisions an identical MCP baseline across Claude (`.mcp.json`) and Codex
(`.codex/config.toml`) — `github`, `context7`, `exa`, `memory`, `playwright`,
`sequential-thinking` — using additive, never-clobber MCP merge so user-authored servers/secrets
survive. **Credentials:** provider API keys are served by the vault auto-inject seam (diagram 3)
— the child gets a ≤24h bearer + base-URL repoint (or HTTPS_PROXY/MITM), and the real key is
swapped in only at egress to the host-allowlisted upstream. **Local model serving:** components
manage local engines (ollama / shimmy / ruvllm) so models can run on-box; their keys (where
needed) flow through the same seam.

*Source: envctl/CLAUDE.md (MCP baseline identical across .mcp.json + .codex/config.toml);
docs/KASETTO-FEATURES.md §8 (per-agent transforms + additive MCP merge), §6 (agent presets);
docs/secrets/AUTO-INJECT-STATUS.md (provider key auto-inject); [[secrets-auto-inject-seam]].*

```
   ┌─ CONFIG (envctl agent — Kasetto engine) ────────────────────────────────┐
   │ MCP baseline → additive merge, identical on both targets:               │
   │   .mcp.json (Claude)      ◀──┐                                          │
   │   .codex/config.toml (Codex)◀─┴─ github · context7 · exa · memory ·     │
   │                                  playwright · sequential-thinking        │
   └─────────────────────────────────────────────────────────────────────────┘
   ┌─ CREDENTIALS (vault auto-inject seam) ──────────────────────────────────┐
   │ provider key  ──held in vault──▶  child gets ≤24h bearer + base-URL/proxy│
   │ real key swapped in at egress only ─▶ api.anthropic.com / api.openai.com │
   └─────────────────────────────────────────────────────────────────────────┘
   ┌─ LOCAL MODELS (components) ─────────────────────────────────────────────┐
   │ ollama · shimmy · ruvllm   (on-box serving; keys via the same seam)      │
   └─────────────────────────────────────────────────────────────────────────┘
```

---

## 9. envctl desktop app

The `envctl-gui` crate is a native egui/eframe app (no web, no WebView). The decisive
architectural rule: **one shared `Engine` library drives both the CLI and the GUI, so the two
front-ends can never diverge** — all logic lives in the engine; the GUI is a thin renderer over
the identical detect/install/secret API. The engine never prints; it emits a structured `Event`
stream. The GUI runs the engine on a single serial worker thread and drains events via `try_recv`
in `update()`, calling `request_repaint()` so the UI thread never blocks (telemetry runs on its own
~1s cadence). Screens (Dashboard, Components, Add Repo, Live Logs, Settings/Manifest) issue the
same `EngineCommand`s the CLI verbs do.

*Source: docs/ARCHITECTURE.md §2 (one engine, two front-ends), §9 (Event stream), §10 (GUI threading
+ screens); envctl/CLAUDE.md (engine is the single shared library, non-printing).*

```
                 ┌──────────────────────────────┐
                 │  envctl-engine  (THE library) │  sync · non-printing · emits Event
                 │  detect / install / secret …  │
                 └───────┬──────────────┬─────────┘
       identical API     │              │   identical API
              ┌──────────▼───┐    ┌─────▼─────────────────────────────┐
              │ envctl (CLI) │    │ envctl-gui (eframe/egui, native)   │
              │ drains Event │    │ worker thread runs Engine          │
              │ on main thrd │    │ update(): try_recv + request_repaint│
              └──────────────┘    │ screens: Dashboard · Components ·  │
                                  │   Add Repo · Live Logs · Settings  │
                                  └────────────────────────────────────┘
              CLI and GUI CANNOT diverge — both drive the same Engine
```

---

## 10. envctl binary + features from Kasetto

Kasetto v3.2.0 was absorbed into envctl as `crates/agent-env` + `Engine::agent_*`, surfaced as
the `envctl agent` verb (subverbs sync / add / remove / lock / list / clean; the 11 Kasetto verbs
folded to envctl's 6). The full feature set ships inside the one `envctl` binary: declarative YAML
config, the committed lock with OS-invariant SHA-256 content hashing + diff (zero network when
hashes match), `extends` composition, global/project scope, the three asset kinds (skills /
commands / MCPs), per-agent native-format command transforms + **additive never-clobber MCP merge**,
the sync/apply pipeline with `--dry-run`, `--json` structured output, `doctor`, and self-update.
Invariants held through absorption: **no C in the trust boundary** (so Kasetto's `mimalloc`
allocator was dropped), the engine stays non-printing (Kasetto's `ui.rs`/`banner.rs`/`process::exit`
dropped — front-ends own rendering), destructive verbs fail-closed + dry-run by default. The external
`kasetto`/`kst` binary is retired; parity was 102 verified / 0 pending / 13 intentional front-end.

*Source: docs/KASETTO-FEATURES.md §0 (absorbed into crates/agent-env + Engine::agent_*; mimalloc dropped),
§2 (lock + content hash), §3 (extends), §8 (transforms + MCP merge), §9/§10 (sync + --json);
[[kasetto-absorption-rust-port]]; envctl/CLAUDE.md (agent-env-managed, no hand-edit).*

```
   external kasetto / kst  ──RETIRED──▶  absorbed into the ONE envctl binary
   ┌──────────────────────────────────────────────────────────────────────────┐
   │  envctl agent {sync · add · remove · lock · list · clean}                  │
   │     │ crates/agent-env (18-module pure-Rust port) + Engine::agent_*        │
   │     ▼                                                                      │
   │  declarative agent-env.yaml ──▶ resolve sources ──▶ hash-diff vs lock      │
   │     │                                   │ hashes match → ZERO network      │
   │     │ extends compose                   ▼                                  │
   │     │ global/project scope        per-agent transforms (skills/commands)   │
   │     │                             + MCP merge (ADDITIVE, never clobber)    │
   │     ▼                                                                      │
   │  agent-env.lock (OS-invariant SHA-256, content-hashed)   --json · doctor   │
   └──────────────────────────────────────────────────────────────────────────┘
   Invariants kept: NO C in trust boundary (mimalloc DROPPED) · engine non-printing
   (ui/banner/process::exit dropped) · destructive = fail-closed + dry-run default
```
