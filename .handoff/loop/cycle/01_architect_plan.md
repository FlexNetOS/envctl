# Architect plan — TASK-0075 cognitum-seed-trust (cycle: seed-ca-trust)

**Verdict: GO. Pure-manifest, single new component file + lock sync.**

## Decisive finding
`ENVCTL_SEED_CA` override ALREADY EXISTS — `crates/secrets-engine/src/seam.rs:113-116`:
`ca_path()` = env `ENVCTL_SEED_CA` || `/usr/local/share/ca-certificates/cognitum-ca.crt`.
So NO daemon/engine code change. PR-1 pins to the `/usr/local` path the daemon reads today
(a meta-path pin the daemon ignores would be useless). Meta-path relocate (set `ENVCTL_SEED_CA`
in the secretd unit) is deferred → TASK-0075b follow-up.

## On-box facts
- USB anchor `/run/media/drdave/COGNITUM/trust/`: `cognitum-ca.pem` (786B) = source; `.crt` identical; `install-trust.sh` does `sudo cp cognitum-ca.pem /usr/local/share/ca-certificates/cognitum-ca.crt && sudo update-ca-certificates`.
- Current host pin MATCHES the USB (owner already re-pinned manually) → verify will report MATCH.
- `manifest/envctl.lock` enumerates components by FNV-1a `content_hash`; regenerate via `envctl lock`.
- Template = `manifest/cognitum-seed-net.toml` (top-level sibling, NM-profile worker + oneshot unit + cdc_ncm udev rule, additive, absent-Seed=no-op, needs_sudo).

## Unit ledger
| U# | unit | what | why |
|----|------|------|-----|
| U1 | `manifest/cognitum-seed-trust.toml` `[[component]]` | new component, `destructive=false`, `requires=[]` | codify the manual re-pin |
| U2 | `[component.detect]` | worker+unit+udev present (artifacts predicate, NOT Seed-reachable) | drift never nudges reinstall when Seed unplugged |
| U3 | `[component.install]` worker `/usr/local/sbin/cognitum-seed-trust-refresh` | locate COGNITUM mount, cp `trust/cognitum-ca.pem`->pin path, `update-ca-certificates`; absent->exit 0 | the auto re-pin |
| U3b | install -> oneshot unit + `99-cognitum-seed-trust.rules` (cdc_ncm trigger) | run on boot + Seed hotplug | "plugged in = access" without manual action |
| U4 | `[component.verify]` | artifacts hard (fail-closed) + non-fatal byte-compare pin vs anchor (MATCH/STALE) | task's stated verify; honest absent-Seed no-op |
| U5 | `[component.fix]` | idempotent re-provision + re-pin now | self-heal |
| U6 | `[component.remove]` | drop unit/udev/worker; KEEP pinned CA | removal never breaks a working unlock |
| U7 | `manifest/envctl.lock` | add `[components.cognitum-seed-trust]` via `envctl lock` | reproducible-state honesty |

## Runtime surface (guardian drives — Seed mounted at /run/media/drdave/COGNITUM)
1. `cargo run -p envctl -- auto-detect` -> `cognitum-seed-trust` appears, no manifest parse error.
2. With Seed mounted: the verify hook's non-fatal probe reports pin==anchor MATCH (byte compare).
3. Simulated absent-Seed (`COGNITUM_TRUST_DIR` -> nonexistent): worker exits 0 with explicit no-op msg; never fails the box.

## Guards
- absent-Seed = clean no-op (exit 0) in every hook — never fail the box when unplugged.
- additive: cp the CA + update-ca-certificates; never removes other CAs; never scripts the passphrase; never reveals a secret.
- no-C: pure manifest, no Cargo deps; verify compares cert by raw bytes (`cmp -s`), no openssl needed.
- needs_sudo on install/fix/remove (root-owned `/usr/local` + `/etc` artifacts).
- no-system-depth tradeoff is explicit & deferred (TASK-0075b).
