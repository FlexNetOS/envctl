# envctl Runbook — ASCII Diagrams

Runbook-grade diagrams. `envctl` is a pure-Rust workstation environment manager
(declarative TOML components wrapping proven bash) **plus** a secrets vault + credential
broker, driven and maintained by an **autonomous agent harness**. It absorbed Kasetto v3.2.0
as its built-in `envctl agent` agent-environment engine. The non-negotiable spine: **no C
library in the trust boundary** (libSQL `remote` only, ring-only rustls, pure-Rust crypto),
one shared non-printing `Engine` drives both CLI and GUI, and destructive ops are fail-closed
+ dry-run by default.

This file has three parts:

- **§1–§10** — the vault / secrets / CLI / agent-env spine (the original ten topics).
- **§11–§16** — the **full picture**: every component, the env-manager data flow, the
  meta-prefix system-depth convergence, the agent-harness automation topology, the continuity
  kernel, and the complete automated-vs-manual control surface.
- Companion stories: [`AGENTIC-STORY.md`](AGENTIC-STORY.md) (how the box builds & maintains
  itself with zero humans) and [`USER-STORY.md`](USER-STORY.md) (how / if / when the human is
  involved — desktop app, CLI, enter/exit points).

Each diagram cites its source `file:section`.

### Automation legend (used throughout §11–§16)

```
  [A]  AUTOMATED          — agent/loop runs it unattended; no human needed
  [A*] AUTOMATED-ELEVATED — runs sudo, but sudo -n is passwordless here → still unattended
  [P]  PREVIEW-BY-DEFAULT — runs dry-run; a human (or RALPH_APPLY) must pass --apply to mutate
  [H]  HUMAN-GATED        — a human MUST trigger it (reboot, live-shell migration, secret reveal)
  [!!] SUPERVISED/CRITICAL— the loop REFUSES to auto-run; writes NEEDS-HUMAN and stops
```

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

`secretctl run -- <cmd>` is a **fork/exec wrapper**, not a shell mutation: it mints a
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
   secretctl run -- claude -p "hi"
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

---

## 11. The full component catalog (every component envctl declares)

envctl declares **79 components** (the committed `manifest/envctl.lock` count) across 14 manifest
files (`manifest/*.toml` + drop-ins in `manifest/components.d/`). Each is a `[[component]]` with
five phase hooks (Detect → Install → Verify → Fix → Remove) and a `requires = [...]` edge set;
the engine runs them in topological order. Below is the complete set, grouped by manifest, with
the automation class of its *install/converge* path (legend at top). `group-*` ids are aggregate
meta-components (install-a-set); `gpu_required` components are skipped on a GPU-less host.

*Source: `manifest/*.toml` + `manifest/components.d/*.toml` (id inventory, verified 2026-06-23);
`manifest/envctl.lock` (79 components); CLAUDE.md (manifest dir defaults to `./manifest`,
override `ENVCTL_MANIFEST_DIR`); docs/ARCHITECTURE.md §3 (component model), §6 (topo order).*

```
 base.toml ─ language/runtime floor ───────────────────────────────────────────────┐
   [A]  nerd-fonts · bun · node-via-bun · node-real · rustup(→nightly) · rtk         │
 apt-base.toml ─ host prerequisites (apt; some sudo) ───────────────────────────────┤
   [A*] ghostty · podman · keepassxc · virt-stack · libssl-dev · meta-base-sanity    │
 dev-tools.toml ─ dev CLIs ─────────────────────────────────────────────────────────┤
   [A]  gh · vite · wasmer · cargo-nextest · uv                                       │
 components.d/just.toml :  [A] just                                                   │
 components.d/epic-h-toolchains.toml ─ META-PREFIX toolchains (the no-system-depth set)┤
   [A]  gh-cli · wild-linker · kache · nushell · zellij · mise · ollama · llvm-clang  │
        · libgccjit · nix-portable · yazi · helix · huggingface-cli                   │
 gpu.toml ─ NVIDIA / CUDA / rust-GPU (gpu_required) ────────────────────────────────┤
   [A*] nvidia-cuda-repo   [floor] nvidia-open(kernel)   [A] cuda-toolkit(meta-prefix)│
   [A]  rust-nightly-cuda-oxide · cuda-oxide · pytorch-venv · gpu-verify-scripts      │
   [A*] nvidia-container-toolkit            (aggregate) group-gpu-stack               │
 ai-clis.toml ─ agent CLIs ─────────────────────────────────────────────────────────┤
   [A]  claude-code-cli · codex-cli · gemini-cli · kimi-cli · devin-cli   group-ai-clis│
 agent-env.toml :  [A] kasetto   (the absorbed agent-env engine component)            │
 SECRETS STACK ─ env-ctl.toml/secretd.toml/sqld.toml + seed ────────────────────────┤
   [A]  env-ctl(builds+installs secretd/secretctl→.toolchains) · secretd · sqld       │
   [A*] cognitum-seed-net · cognitum-seed-trust · cognitum-seed-autounlock (udev/units)│
 nix-yazelix.toml ─ interactive shell (nix-sourced; being de-nixed) ────────────────┤
   [A]  nix · nix-yazelix-cache · home-manager · yazelix · yazelix-config             │
   [A]  yazelix-desktop · yazelix-shell · ghostty-default-terminal  group-nix-yazelix │
 dashboard.toml :  [A] dashboard (zellij mission-control)                             │
 desktop-app.toml :  [A] desktop-app (envctl-gui .desktop entry)                      │
 n8n-mcp.toml :  [A] n8n-mcp     grit.toml : [A] grit     rusty-idd.toml : [A] rusty-idd│
 components.d/portability-links.toml ─ ~/.config, $META_ROOT/usr/bin, /usr/local symlinks ─┤
   [A]  home-config-links · home-bin-links · rtk-config-links · meta-tool-links       │
   [A]  claude-global-links · usrlocal-script-links            group-portability      │
 boot-repair.toml ─ recovery (DESTRUCTIVE; not in the normal converge path) ─────────┤
   [P/!!] boot-repair-diagnose · -dev · -rename-pro · -finalize   group-boot-repair   │
 ───────────────────────────────────────────────────────────────────────────────────┘
   IRREDUCIBLE FLOOR (meta never owns; detect/verify only): nvidia-open KERNEL module
   + libcuda.so  (see §13).   Everything else resolves inside meta → $META_ROOT/.toolchains.
```

---

## 12. The env-manager data flow (detect → converge → lock)

The whole env-manager is one loop: **read reality, diff against the declared state, converge,
re-hash the lock.** `auto-detect` is read-only and returns an `EnvReport`; `install` is the only
*additive-by-default* mutator (it ACTS — it is **not** dry-run); `auto-fix` / `reset` / `add-repo`
are **dry-run by default** and need `--apply`. Every component's Verify hook returns `0=healthy`;
the committed `envctl.lock` is the OS-invariant content hash that makes a converged box
reproducible (and `lock --check` is the CI no-drift gate). A failing hook is recorded in
`RunSummary.fail[]` and the run continues — never an abort.

*Source: crates/cli/src/main.rs (Cmd verbs); crates/engine/src/lib.rs (Engine, Event stream,
RunSummary); docs/ARCHITECTURE.md §5 (verb→phase), §6 (topo/Kahn), §12 (dry-run safety);
CLAUDE.md (install applies by default; auto-fix/reset/add-repo preview).*

```
                        ┌─────────────────────── envctl.lock (committed) ──────────────┐
                        │     OS-invariant content hash of every component (79)         │
                        │     lock --check = CI no-drift gate  [A]                      │
                        └───────────────▲──────────────────────────────┬───────────────┘
                                        │ re-hash after converge        │ compare
   reality (the box)                    │                               ▼
   ┌──────────────┐  [A] auto-detect   ┌┴───────────────┐   diff    ┌──────────────────┐
   │ host · GPU · │ ───(read-only)───▶ │   EnvReport    │ ────────▶ │ declared state    │
   │ tools · pkgs │                    │ present/healthy │           │ (manifest/*.toml) │
   └──────────────┘                    └────────────────┘           └────────┬─────────┘
        ▲                                                                     │ per component, topo order
        │ converged                                                          ▼
        │                         ┌─ Detect ─▶ present? ─no─▶ [A] Install (ADDITIVE, acts) ─┐
        │                         │     │ yes                                               │
        └─────────────────────────┤     ▼                                                   │
          [A] install (acts)       │  [A] Verify (0=healthy) ─bad─▶ [P] auto-fix (--apply) ─┘
          [P] auto-fix --apply     │                                                         │
          [P] reset --apply        └─ remove path: [P] reset (dry-run) ──--apply--▶ unwire   │
                                                                          [!!] reset --all --confirm
   Event stream (NDJSON with --json) drains to CLI stdout / GUI Live-Logs  ── never println! in engine
```

---

## 13. Meta-prefix convergence — the no-system-depth model (Epic H)

**Owner doctrine:** meta and its peers use **NO system-depth installs** (`apt /usr`,
`/usr/local`, nix `/nix`, kernel). Every system-depth tool has an upstream repo and must be
either (a) installed at `$META_ROOT/.toolchains/<x>` (tarball / `cargo install --root` / runfile
`--toolkitpath`) with a `$META_ROOT/usr/bin` symlink, (b) cloned+added as a `.meta.yaml` peer, or (c)
— only if *physically* irreducible — declared a `system:` host-prerequisite (detect/verify only).
The convergence is itself a set of `[A]` components; the only genuinely irreducible item is the
NVIDIA **kernel module**. `/nix` is **reducible** (nix-portable) but its live removal is `[!!]`.

*Source: docs/adr-install-locations-and-local-state.md §System-depth convergence; CLAUDE.md
(rust-native; agent-env-managed); backlog Epic H (TASK-0054..0077); memory
[[no-system-depth-installs]]; verified on-box 2026-06-23 (nvidia-595-open, /nix 14G, libcuda 595).*

```
   SYSTEM DEPTH (being eliminated)                 META PREFIX (the target)
   ───────────────────────────────                ─────────────────────────────────────────
   apt /usr/bin · /usr/local/bin                  $META_ROOT/.toolchains/<tool>/  +  $META_ROOT/usr/bin/<tool>
   /nix/store (14G, multi-user daemon)            $HOME/.nix-portable  (bwrap, no root)   [A] shipped
                                                  └ live /nix removal + yazelix repoint    [!!] TASK-0067
   ~/.cargo  ~/.rustup  (user-global)             $META_ROOT/.toolchains/{cargo,rustup}
   apt cuda-toolkit-13-3                           .toolchains/cuda (runfile --toolkitpath) [A] done
                                                   └ apt remove cuda-toolkit-13-3           [H] owner sudo

   ── IRREDUCIBLE FLOOR (meta NEVER owns — detect/verify only) ──────────────────────────────
   nvidia-open KERNEL module   loaded by the running kernel (/lib/modules, DKMS/MOK);
                               MANDATORY for RTX 5090 / Blackwell (proprietary unsupported)
   libcuda.so (user-mode drv)  version-locked to the module; every Rust GPU layer rides on it
        │
        ├── [A] cuda-oxide   (Rust → PTX compiler; author kernels in Rust, no C++)
        └── [A] cudarc       (host-side launch; links ONLY libcuda — no CUDA toolkit C needed)
            ⇒ once both are in use, the system CUDA toolkit becomes REMOVABLE
   driver 595→610 bump        [H] needs a REBOOT — held to the very end of the loop
```

---

## 14. The agent-harness automation topology (the construction crew)

The box is built and maintained by a **self-perpetuating agent loop** (the *Ralph* pattern), not
by hand. `forge-loop` reads a durable backlog, runs one **Feature Forge** cycle per item
(architect → implementer → guardian), opens one PR, and ticks the item **only when the PR is
MERGED**. `env-install-loop` is the same loop pointed at provisioning (doctor/install/auto-fix).
`auto-provision` is the external runner that spawns a *fresh* `claude -p` per cycle for truly
unattended, set-and-forget operation. Parallel **qwen3.6** background sessions do cheap drafting
legwork; the **opus** orchestrator owns every gate (only it commits/merges, and only after the
guardian PASSes). Mutating agents work in isolated git worktrees so parallel cycles never collide.

*Source: .claude/skills/{forge-loop,feature-forge,env-install-loop,auto-provision,session-relay};
.claude/agents/{feature-architect,rust-implementer,invariant-guardian,handoff-kernel-engineer,
continuity-steward,evolution-steward,build-health-auditor}; CLAUDE.md "Harness: Feature Forge";
memory [[forge-loop-epic-g]], [[agenticos-consolidation-loop]].*

```
   .handoff/loop/backlog.md  (the loop's MEMORY — durable on disk, not in chat)
        │  [A] pick top unblocked item  (hf resume / markdown fallback)
        ▼
   ┌──────────────────────── one Feature Forge cycle  [A] ───────────────────────────┐
   │  feature-architect ─▶ rust-implementer ─▶ invariant-guardian                     │
   │  (read-only plan)     (mutates worktree)   (runs CI gates + cargo + runtime-verify)│
   │        ▲                    │                       │ PASS / PASS-WITH-NOTES        │
   │        │ qwen3.6 drafts ────┘ (legwork; opus gates) │ FAIL → mark [!] blocked,     │
   │        │ (parallel bg sessions)                     │        route to next item    │
   └────────┼───────────────────────────────────────────┼──────────────────────────────┘
            │                                            ▼
            │                              [A] commit ─▶ PR ─▶ arm gh auto-merge --squash
            │                                            │
            │                              [A] TICK-ON-MERGED: gh pr view <N> == MERGED
            │                                            │   └ armed-not-merged → leave [~], re-poll next session
            ▼                                            ▼
   every wrap_every(5) cycles:                  cycle_budget reached →
   [A] BATCH BOUNDARY                           [A] HAND OFF (session-relay) → see §15
     reaper (reap merged worktrees, FF trunk)
     wrap-up reconcile (status-truth, MERGED-gated)
     evolution-steward retro (LESSONS.md / proposed-upgrades.md)

   STOP CONDITIONS (no re-fire):  STOP sentinel [H] · NEEDS-HUMAN [!!] · DONE (confirmed+swept)
```

---

## 15. The continuity / handoff kernel (zero-loss across sessions)

The loop survives context rot and token burn because **all its truth lives in durable files**, not
in conversation memory. The `hf` kernel (built from `meta/handoff`) witnesses every claim /
checkpoint / done into a ledger; `.handoff/loop/` holds the human-readable views (backlog,
loop_state, per-cycle artifacts) and the sentinels. At a cycle-budget boundary the
`continuity-steward` writes a cold-start `HANDOFF.md`, announces over the **weave** bus, and the
successor session resumes with zero loss. **ICM** is the cross-session semantic memory (decisions,
preferences, resolved errors). Hooks (Stop / PreCompact) auto-checkpoint and drop a
`WRAP-UP-OWED` marker so a missed boundary is caught fail-closed at the next resume.

*Source: .claude/skills/{session-relay,session-relay-resume,session-relay-wrap-up,handoff-sync};
.claude/agents/{continuity-steward,handoff-kernel-engineer}; .claude/hooks/hf-checkpoint.sh;
CLAUDE.md (ADR-0004 ledger model, state precedence); memory [[harness-bootstrap]].*

```
   STATE PRECEDENCE (agents never re-rank):
   Git  >  .handoff/ledger.db (witnessed)  >  tasks/*.task.json  >  active.md  >  HANDOFF.md / backlog.md

   ┌─ DURABLE STATE  .handoff/loop/ ──────────────┐     ┌─ CROSS-SESSION ────────────────────┐
   │ backlog.md       (item checklist; the view)  │     │ ICM      decisions / prefs / errors  │
   │ loop_state.md    cycles_*/budget/wrap_every  │     │ weave    cross-agent bus (to:all)    │
   │ cycle/01_..03_   architect/impl/guardian     │     │ HANDOFF.md  cold-start packet        │
   │ SENTINELS  STOP[H] · NEEDS-HUMAN[!!] · DONE  │     └─────────────────────────────────────┘
   │            · WRAP-UP-OWED (hook-dropped)     │
   └───────────────────┬──────────────────────────┘
        [A] hf claim ──┤ checkpoint --auto ──┤ done --pr <N> ──▶ hf handoff (re-render packet)
                       ▼
   session N  ──cycle_budget──▶  [A] continuity-steward writes HANDOFF + weave-announce
                                       │  (in-session cron is session-only → real unattended = auto-provision)
                                       ▼
   session N+1  [A] session-relay-resume: read HANDOFF · reap worktrees · run any WRAP-UP-OWED ·
                    re-poll armed PRs · reset cycles_this_session=0 ·  continue the backlog
```

---

## 16. The complete control surface — every enter/exit point, automated vs not

Two enter-classes meet the box: **agents** (the loop, unattended) and the **human owner** (rare,
high-trust). Almost everything is `[A]`/`[A*]` automated. The human is required only at five kinds
of point: a **reboot**, a **live-shell migration** (`/nix`), a **secret reveal/unlock passphrase**,
an **owner-sudo cleanup**, and an **owner approval verdict** for a queued `[!!]` decision. The
loop never crosses those lines itself — it writes a sentinel and stops. This diagram is the bridge
to [`USER-STORY.md`](USER-STORY.md).

*Source: CLAUDE.md (dashboard panes default to shell; `[!!]` SUPERVISED refusal; owner-gated
items); docs/runbook/README.md (vault unlock/reveal flags); backlog Epic H (owner-sudo +
TASK-0067 + driver reboot); memory [[no-system-depth-installs]], [[cognitum-seed-usb-unlock]].*

```
   ENTER POINTS                          AUTOMATED?    WHO / WHEN
   ─────────────────────────────────────────────────────────────────────────────────────
   agent loop (forge/env-install)        [A]           the harness, unattended, self-pacing
   auto-provision (fresh claude -p/cycle)[A]           set-and-forget overnight runs
   envctl <verb> (CLI)                   [A]/[P]       human OR agent; mutate gated by --apply
   envctl-gui (desktop app, egui)        [H-driven]    human clicks; same Engine as CLI (§9)
   zellij mission-control dashboard      shell default human; Claude only via envctl-open-claude
   secretctl unlock / lock               [A] USB / [H] passphrase   USB possession = auto; pass = human
   secretctl secret get --reveal         [H] --reveal --apply --confirm   human, audited
   ! secretctl unlock (in-session)       [H]           the secure owner path for passphrase unlock
   ─────────────────────────────────────────────────────────────────────────────────────
   EXIT / HUMAN-WALL POINTS              CLASS         WHY
   ─────────────────────────────────────────────────────────────────────────────────────
   driver 595→610 bump                   [H] reboot    kernel module reload needs a reboot
   /nix removal + yazelix repoint        [!!] TASK-0067 touches the owner's LIVE interactive shell
   apt remove cuda-toolkit/mold/gh       [H] sudo      pure cleanup; meta already shadows on PATH
   reset --all / boot-repair destructive [!!]/[P]      fail-closed; --confirm / human only
   queued approval verdict               [H] owner     handoff-steward surfaces; owner decides
   NEEDS-HUMAN / STOP sentinel           [!!]/[H]      loop refuses + stops; human resumes
   ─────────────────────────────────────────────────────────────────────────────────────
   COMMUNICATION BACK TO THE HUMAN:  vox (spoken summary, piper/en) · weave (bus) · ICM (memory)
   · PR descriptions · HANDOFF.md · the sentinel files · GUI Live-Logs / dashboard panes
```
