# Odysseus adoption (QUALIFY / sandbox)

`manifest/odysseus.toml` adopts **Odysseus** (`github.com/pewdiepie-archdaemon/odysseus`, a
self-hosted AI workspace) as a **pinned, sandboxed, reversible** service managed by envctl. This
doc records *why it is shaped the way it is* and what must clear before it could be promoted past
a sandbox.

## Verdict & provenance

rusty-idd's plan-loop evaluated Odysseus as a candidate LifeOS front door and returned
**QUALIFY / sandbox-adopt** (weave msg #178; source:
`rusty-idd/.handoff/loop/plan/reports/odysseus-front-door-evaluation.md`):

> Do not make it the canonical LifeOS front door yet. The strict-upgrade path is a **reversible
> envctl-managed service behind LifeOS** — not a source merge, not a direct replacement.
> AGPL-3.0 ⇒ **no source merge** until a license decision. Bind **127.0.0.1**. **Gate** the
> Docker-socket and privileged seams.

This component is the envctl realization of that verdict.

## AGPL stance (no source merge)

Odysseus is **AGPL-3.0-or-later**. We run it as a **separate process boundary** (its own
containers) from a **pinned upstream clone** — we never vendor its source into the envctl Rust
workspace, and the clone lives under managed local state (`$META_ROOT/var/lib/odysseus/src`,
not a workspace member, not committed). No code is merged. A license decision is a prerequisite
for any deeper integration (see promotion gates).

## Engine: rootless Podman, not rootless Docker (why)

The container engine is the box's existing **meta-local rootless Podman** (`apt-base.toml`
`podman`, installed under `$META_ROOT/.toolchains/podman`). Odysseus's `docker-compose.yml` is
run via **`podman compose`** (Docker Compose v2 semantics over Podman).

We deliberately did **not** install rootless Docker, even though Odysseus is Docker-native:

- Rootless Docker on **Ubuntu 26.04** fails to start because
  `kernel.apparmor_restrict_unprivileged_userns=1` denies `rootlesskit`'s `fork/exec
  /proc/self/exe`. The only fix is a **system AppArmor `userns` profile** for the rootlesskit
  binary — a system-depth, security-mitigation change. That conflicts with the meta
  *no-system-depth* doctrine and was (correctly) blocked by the agent's safety classifier.
- Rootless **Podman** uses setuid `newuidmap`/`newgidmap` (not a `/proc/self/exe` re-exec), so it
  runs containers with **zero system changes** — verified on this box. It is already a
  first-class envctl component, so this reuses already-built, policy-compliant infrastructure.

Podman exposes a **Docker-API-compatible socket** (`podman.socket` → `$XDG_RUNTIME_DIR/podman/
podman.sock`); Cookbook's model-serving seam talks to *that*, never the root daemon.

## Hardening (the gates, enforced by the component)

The generated `src/compose.meta.yml` override + `src/.env` enforce:

| Gate | Mechanism |
|------|-----------|
| Loopback only | `APP_BIND=127.0.0.1`; `verify` fails if any container publishes off `127.0.0.1` |
| Auth on | `AUTH_ENABLED=true`, `LOCALHOST_BYPASS=false` |
| Side services internal-only | SearXNG/ntfy host ports dropped (`ports: !reset null`) — per Odysseus SECURITY.md, also avoids the `sqld:8080` clash |
| Docker-socket redirected | compose `!override` replaces the hardcoded `/var/run/docker.sock` source with the rootless **Podman** socket — never the root daemon |
| No committed secrets | `.env` is generated locally (gitignored upstream); provider keys come from the envctl secret stack (`secretctl`) at runtime, not from a committed file |
| Reversible | data/logs are `data_paths` (preserved on `remove`; only `reset --purge` deletes); `remove` runs `podman compose down` + drops the clone |
| Pinned | clone is `checkout --detach <SHA>` — never floating `dev`/`latest` (`ODYSSEUS_REF`) |
| Re-run idempotent | `install` creates missing data/log volume dirs, but does not chmod/chown existing data/log trees; rootless Podman shifted-ID state remains authoritative |

## Operate it (via envctl)

```bash
envctl auto-detect | grep odysseus      # state (declared / installed / healthy)
envctl install odysseus                 # clone (pinned) + generate config + podman compose up -d --build
envctl verify  odysseus                 # container running + http://127.0.0.1:7000 + loopback-only assert
envctl reset   odysseus --apply         # podman compose down + drop clone; KEEPS data
envctl reset   odysseus --apply --purge # also delete data/ + logs/ (irreversible)
```

First `install` builds the Odysseus image + pulls chromadb/searxng/ntfy — minutes. The admin
password is auto-generated and printed in the container logs:
`podman logs $(podman ps --filter name=odysseus -q | head -1)`.

Re-running `envctl install odysseus` is safe after the containers have written state: the
component only creates missing volume directories and deliberately leaves existing `data/` and
`logs/` ownership/mode alone. If a missing child has to be created below a rootless-Podman-owned
parent, the installer uses `podman unshare` rather than any system-depth `chown`/`chmod` repair.

## Promotion gates (must clear before Odysseus becomes a default LifeOS `/ai` surface)

Odysseus stays **sandbox/QUALIFY** until ALL of:

- [ ] **License review** — AGPL-3.0 network-copyleft posture decided for LifeOS integration.
- [ ] **Reproducible dependency lock** — pin the compose image digests (chromadb/searxng/ntfy) +
      the Odysseus build base, not floating tags.
- [ ] **Secret scan** — confirm no `.env`/data/auth/token leakage (run upstream SECURITY.md's
      `git check-ignore` + secret-grep against the clone).
- [ ] **Tool-privilege audit** — the high-risk agent tools (shell, Python, file, email, MCP,
      model-serving) restricted to admins; demo/test users removed.
- [ ] **Backup/restore test** — `data/` snapshot + restore proven.
- [ ] **Rollback test** — `reset --apply` returns the box to pre-adoption state.
- [ ] **LifeOS UX fit** — see downstream seam below.

## Downstream seam (not built here): LifeOS `/ai`

LifeOS (Tauri/Vue, `meta/lifeos`) already has a pluggable AI provider enum + keyring/env lookup
(`ai_complete`/`ai_provider_set`, `<app_data_dir>/ai.json`). Routing `/workspace/ai` to the local
Odysseus is a **new provider variant** pointing at `http://127.0.0.1:7000` with a bearer from
`secretctl`. That belongs in a separate LifeOS change after the promotion gates clear; this
component only stands the service up behind the front door.
