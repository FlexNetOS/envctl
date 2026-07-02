# Risk policy — planning-engineer

Per-target `risk_policy` for the planning loop. Each section states the risk tiers and the
SUPERVISED `[!!]`/`[H]` boundary that must be human-gated and fail-closed. Tiers: APPLY (low,
self-contained, no contract/secret/trust-boundary touch) · PROPOSE (touches a contract, central
helper, or secret-adjacent config) · SUPERVISED (secrets / destructive / trust-boundary / provider).

---

## grit

`risk_policy` for `grit` (v0.4.0, `/home/drdave/Desktop/meta/grit`). grit is an advisory symbol-lock +
git-worktree coordinator that ships cloud-credential code and destructive git verbs. Three risk classes
are SUPERVISED — never auto-applied, always human-gated, always fail-closed.

### SUPERVISED boundary (the `[!!]` / `[H]` set)

| Risk class | What | Evidence | Required control (fail-closed) |
|---|---|---|---|
| **secrets** | Azure access key written **plaintext** to `.grit/config.json` at default (group/world-readable) perms; the S3 path correctly uses the AWS env chain — handling is asymmetric. Also `config set-azure --access-key` puts the key on **argv**. | azure_store.rs:29 (`pub access_key: String`, serde Serialize, no skip); config.rs:51-56 (`save()` via `std::fs::write`, no mode); cli/mod.rs:1482-1494 vs S3 env at :1475 | SUPERVISED: harden via env-parity-with-S3 (e.g. `AZURE_STORAGE_KEY`) or 0600 perms + keyref; never store the key verbatim; doctor-warn if a key is found in `config.json`. Owner-walled (envctl/secrets owns the residency if hardened). |
| **destructive** | `done` runs rebase + `git merge --no-ff` + branch delete + release; `session pr` runs `git push` + PR create; `session end` checks out the base branch. No allowlist/permission profile ships; no destructive-command guard is propagated from parent meta into grit. | git/mod.rs:221-253; cli/mod.rs:931-943 (done), :1415 (session pr push), :1442 (session end) | SUPERVISED `[!!]`: a `.claude/rules/destructive-commands.md` mirroring parent meta (forbid `git reset --hard`, `git clean -fd`, force-push w/o lease) MUST exist; merge/push/PR verbs are human-gated; the existing fail-closed merge-refusal on a dirty main worktree (README.md:272-275) is kept, never weakened. |
| **trust-boundary (no-C)** | grit's own substrate is NOT no-C: `rusqlite` uses `bundled` SQLite (C) and all 14 tree-sitter grammars are C. grit therefore CANNOT be the in-trust-boundary union engine. | verdicts.md FEASIBILITY (no-C invariant); Cargo.toml:15 (rusqlite bundled), grammar deps | SUPERVISED: any reconciler that must live INSIDE handoff's no-C trust boundary MUST be pure-Rust (Route A); grit is usable only as an OUT-of-boundary coordination tool (Route B). The "grit as-is = in-boundary engine" framing is REFUTED and must not be smuggled back in. |

### Additional fail-closed nits (PROPOSE, not SUPERVISED)

- **provider/model:** grit is LLM-free internally (no model client in `Cargo.toml`/`src`); model-lane
  policy is the *consuming* harness's concern, not grit's — no silent provider/model downgrade is
  possible inside grit. The lane map (mechanical/structured/decision-gate) is advisory for callers.
- **silent backend downgrade:** the `_ => SQLite` catch-all routes a typo'd backend (`"azur"`/`"s33"`)
  to local locking with no error (cli/mod.rs:407) — a No-Downgrade violation; fix with `enum Backend`
  deny-unknown (roadmap #3). PROPOSE (touches the config parse contract).
- **partial-lock leak:** a non-atomic multi-symbol claim leaks the granted subset on terminal `bail!`
  (cli/mod.rs:612-616) — APPLY fix (mirror the retry-path release at :626-628).

### Doctrine

- No SUPERVISED item is auto-applied or applied mid-cycle; each is owner-gated and reversible.
- No control above is ever weakened — every change STRENGTHENS a gate (the destructive guard and the
  fail-closed merge-refusal are additive).
- Apply-time re-verification required for web-sourced advisories (Rust ≥ 1.96.0 for Cargo
  CVE-2026-5223/5222; Azure GA 2026-05-14) before they gate the build.
