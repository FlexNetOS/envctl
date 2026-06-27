# envctl Runbook

The operator's entry point to **envctl** — a pure-Rust, GPU-aware workstation environment
manager **and** secrets vault. envctl has two halves over one shared `Engine`:

- **env-manager** — declarative TOML *components* (detect → install → verify → fix → remove)
  that bring the box to a declared state. Verbs: `auto-detect, install, doctor, auto-fix,
  reset, add-repo, graph, registry, lock, env, dashboard, self, completions`.
- **secrets stack** — a pure-Rust gRPC vault + credential broker (`secretctl …`, daemon
  `secretd`): AEAD-at-rest, ≤24h peer-bound relay bearers, GitHub-App token minting, a local CA.
- **agent-env** — the absorbed **Kasetto v3.2.0** engine (`envctl agent …`): declarative
  skills / MCP servers / slash-commands sync. The standalone `kasetto` binary is retired;
  mimalloc was removed (no-C trust boundary).

> **No-C trust boundary:** no SQLite/OpenSSL/aws-lc is *linked*. Store = libSQL `remote` only;
> crypto = pure-Rust (ring, blake3, chacha20poly1305, argon2). One rustls, ring-only.

---

## Map of this runbook

| Area | Where |
|------|-------|
| **ASCII diagrams** — §1–§10 vault/secrets/CLI/agent-env spine; §11–§16 every component, env-manager data flow, meta-prefix convergence, agent-harness topology, continuity kernel, full automated-vs-manual control surface | [`DIAGRAMS.md`](DIAGRAMS.md) |
| **Agentic story** (how the box builds & maintains itself with zero humans; the crew, the loop, the gates, self-improvement) | [`AGENTIC-STORY.md`](AGENTIC-STORY.md) |
| **User story** (how/if/when the human is involved; desktop app, CLI, dashboard, communication flow, enter/exit points, the five human walls) | [`USER-STORY.md`](USER-STORY.md) |
| **agent-env user docs** (ported from kasetto.dev/docs, renamed) | [`agent-env/`](agent-env/) — installation, configuration, commands, agents, authentication, security, ci, faq, cookbook, how-sync-works, sync-flow, slash-commands, writing-skills |
| Env-manager architecture | [`../ARCHITECTURE.md`](../ARCHITECTURE.md) |
| Secrets-stack design corpus | [`../secrets/`](../secrets/) — ARCHITECTURE, DESIGN-NOTES, SERVER-MODE, THREAT-MODEL, AUTO-INJECT-STATUS, GITHUB-TRANSPORT-DOCTRINE |
| Absorbed Kasetto feature catalog | [`../KASETTO-FEATURES.md`](../KASETTO-FEATURES.md) |
| Key ADRs | [`../adr-seed-usb-possession-factor.md`](../adr-seed-usb-possession-factor.md), [`../adr-install-locations-and-local-state.md`](../adr-install-locations-and-local-state.md) |

---

## Operational procedures

### Vault — status / unlock / lock

The vault is **locked** after every daemon (re)start or reboot (the DEK is RAM-only and
zeroized on lock — this is correct security posture, **not** an empty/uninitialized vault).
`secret_count=0` and "would init fresh" in some outputs are cosmetic — **never run
`secretctl init` on an existing vault** (it refuses against the stored header MAC).

```bash
secretctl status                 # locked | unlocked, usb_possessed, relay/secret counts
secretctl unlock                 # USB-first (Cognitum Seed); passphrase if USB absent
secretctl unlock --passphrase-stdin   # scripted passphrase unlock
secretctl lock                   # zeroize the DEK + CA issuer in RAM (true panic-stop)
```

**Unlock factors (the "way in"):**
1. **USB / Cognitum Seed (intended "plugged-in = access"):** secretd asks the Seed's custody
   API (`https://169.254.42.1:8443/api/v1/custody/sign`, `Bearer <seed-token>`) to Ed25519-sign
   the context `envctl/usb-kek/v1/{partition_uuid}`; the signature → HKDF → KEK →
   AEAD-unwraps the slot's `wrapped_dek`. Reachability persists across reboot via the
   `cognitum-seed-net` NetworkManager profile (host `169.254.42.2/24` on the Seed cdc_ncm link).
2. **Passphrase (argon2id keyslot):** the fallback when the USB factor is absent.

> ⚠️ **Most common USB-unlock failure — a STALE PINNED CA (proven 2026-06-23):** the daemon
> validates the Seed's TLS **strictly against the pinned Cognitum Device CA**
> (`/usr/local/share/ca-certificates/cognitum-ca.crt`; override `ENVCTL_SEED_CA`). When the Seed
> **rotates its Device CA**, the host pin goes stale, the TLS handshake fails **fail-closed and
> silently** (no log), so custody/sign fails → `usb_possessed=false` → `secretctl unlock` returns a
> generic `Internal: "unlock failed"`. It is **NOT** a broken keyslot and the passphrase is **not**
> required. **Fix = re-pin from the physically-present USB trust anchor:**
> `bash /run/media/drdave/COGNITUM/trust/install-trust.sh` (copies `trust/cognitum-ca.pem` → the
> system pin + `update-ca-certificates`), restart `env-ctl.service`, then `secretctl unlock` (USB)
> succeeds with no custom config. Diagnose by comparing pubkeys: the host pin must equal the live
> Seed-presented CA — `openssl s_client -connect 169.254.42.1:8443 -showcerts` vs
> `openssl x509 -in <pin> -pubkey`. (Codification to auto-refresh on rotation: backlog TASK-0075.)
>
> **Note:** `secretctl status` shows `secret_count=0` even on a populated vault (cosmetic). Revealing
> a non-broker secret needs all three flags: `--reveal --apply --confirm`. The Seed itself holds **no
> vault material** — it is a Pi Zero 2 W providing only Ed25519 custody-sign (admin via
> `ssh genesis@169.254.42.1`).

### Vault — GitHub App token mint (admin control plane)

```bash
secretctl mint-github --installation-id <ID> --ttl-secs <SECS> --output json   # → {"token":"…","expires_at_unix":<i64>}  (FROZEN contract; --installation-id, --ttl-secs, --output all REQUIRED)
secretctl github-app enroll --app-id <APP_ID> --private-key <app.pem|-> --apply  # seal the App PEM + persist app-id (one-time; --app-id & --private-key REQUIRED)
secretctl github-app set-app-id --app-id <APP_ID> --apply                        # heal a missing github-app-id meta (PEM untouched)
secretctl github-app revoke-token …                                             # early-revoke an outstanding installation token
```
`mint-github` opens the vault-sealed `github-app-private-key` (app-id `4044997`) against the live
DEK, builds an RS256 App-JWT, and exchanges it for an installation access token. **Gated on the
vault being unlocked.**

### Vault — secrets, relay injection, CA, audit

```bash
secretctl secret <add|get|list|rm|rotate> …   # stored-secret CRUD (get --reveal --apply to reveal; audited, refused if broker-only)
secretctl run -- <cmd>                  # run <cmd> with relay creds injected into the CHILD only
secretctl relay …                       # relay policies + mint ≤24h peer-bound bearers
secretctl ca …                          # local CA: issue/revoke leaf certs, trust wiring
secretctl audit …                       # query the tamper-evident (hash-chained) audit log
```
The **auto-inject seam** keeps the real key in the daemon: child tools get a bearer (base-url
repoint *or* `HTTPS_PROXY`/MITM) and the real credential never leaves `secretd`. See
[`../secrets/AUTO-INJECT-STATUS.md`](../secrets/AUTO-INJECT-STATUS.md).

### agent-env — skills / MCP / commands sync (absorbed Kasetto)

```bash
envctl agent sync --apply                   # reconcile installed assets with agent-env.yaml
envctl agent add <url> --skill <name> --apply
envctl agent list --kind skills
envctl agent lock --check                   # CI gate: agent-env.yaml ↔ agent-env.lock no-drift
envctl agent doctor --scope global
```
Config = `agent-env.yaml` → `agent-env.lock` (content-hash). MCP baseline kept identical across
Claude (`.mcp.json`) + Codex (`.codex/config.toml`): github, context7, exa, memory, playwright,
sequential-thinking. Full surface: [`agent-env/`](agent-env/) (ported kasetto.dev/docs).

### Env-manager — bring the box to its declared state

```bash
envctl auto-detect [--json]                 # read-only inventory: host, GPU, tools, components
envctl install <components…>                # additive + idempotent (--dry-run to preview)
envctl doctor                               # health: writability, toolchains, sudo, UEFI, GPU
envctl auto-fix [--apply]                   # repair broken components (DRY-RUN default)
envctl reset <component> [--apply] [--purge] # remove + unwire (fail-closed, dry-run default)
envctl lock [--check]                       # envctl.lock content-hash of every component
envctl env [--json]                         # emit $META_ROOT + toolchain env exports
```

> **Safety:** destructive verbs (`reset` / `auto-fix` / `self uninstall`) are **PREVIEW by
> default** and fail-closed — they refuse unless safety is proven and you pass `--apply`/`--purge`.

### Fleet sync — clean-only pull/push across the meta workspace

```bash
python3 scripts/meta-fleet-sync.py --fetch --json
python3 scripts/meta-fleet-sync.py --apply
```

`scripts/meta-fleet-sync.py` is the safe alternative to raw `meta exec -- git pull/push` when the
workspace has many sibling repos. It only fast-forwards clean behind-only checkouts, only pushes
clean ahead-only checkouts, recognizes linked worktrees during fetch, ignores nested managed
worktree paths when judging the root checkout, and fails closed if any fetch fails.

### Desktop app

`envctl-gui` drives the **same shared `Engine`** as the CLI (they cannot diverge). See the
desktop diagram in [`DIAGRAMS.md`](DIAGRAMS.md).

---

## Provenance

This runbook's `agent-env/` pages are ported from **kasetto.dev/docs** (Kasetto v3.2.0, absorbed
into `crates/agent-env`), renamed `kasetto`→`envctl agent` / `kasetto.yaml`→`agent-env.yaml`,
mimalloc removed. The diagrams and procedures are authored from the envctl source + the
`docs/secrets/` corpus + a verified vault/Seed walk (2026-06-23).
