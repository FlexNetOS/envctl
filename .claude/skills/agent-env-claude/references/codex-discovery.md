# Codex-sibling discovery — adopted reference (claude half)

Source: the polished codex prompt's source ledger + controllers (commit 41c5214), adapted for
`/agent-env-claude`. Compact contracts live in the prompt/SKILL.md; this file carries depth.

## CLI probe matrix (claude-adapted; capture real output, profile frontdoors first)

| Probe area | Command | Interpretation |
|---|---|---|
| yazelix ownership | `yzx inspect --json` | must show profile install owner, launcher, runtime dir, update command |
| yazelix state | `yzx status --versions` | generated-state freshness + versions via the owner frontdoor |
| non-UI exec | `yzx env`, `yzx env --no-shell`, `yzx run <argv…>` | profile-owned execution without entering the UI |
| rtk non-mutating | `rtk init --show`; `rtk --help` | inspect rtk wiring without writing; `rtk run` is a raw `sh -c` executor — deliberate use only |
| envctl surface | `./target/debug/envctl --help` (or `cargo run -p envctl --`) | meta workspace env manager; full verb set below |
| hardware gate | `envctl auto-detect --json` | parse for GPU/driver/toolkit/CDI/Rust-GPU/PyTorch/linker evidence before any GPU-aware decision |
| GitKB | `rtk git-kb list --path context/ --json` | inspection route; writable KB = explicit task |
| Grit | `rtk grit status` | absent `.grit` = recorded gap, never implicit init |
| ICM | `ICM_READONLY=1 rtk icm wake-up --max-tokens 200` | absent DB = recorded gap, never implicit init |

A successful probe can still reveal unrelated dirty state in another checkout: that evidence
goes in the proof ledger; it is NOT permission to mutate outside the requested owner surface.

## envctl full surface

Verbs: `auto-detect · install · auto-fix · reset · add-repo · graph · lock · doctor · migrate ·
dashboard · agent · secret`. Destructive verbs preview-by-default. `envctl agent` command
family: `init · add · remove · sync · lock · list · doctor · clean` — preserve the whole family
(add/remove are preview-by-default config edits; `list` inventories; `doctor` reports agent-env
health; `clean` removes tracked stale assets; sync tracks and removes ONLY what the lock says it
installed — it never adopts unrelated MCP servers or skills). Fleet note: `meta git`-adapted
verbs are clone/commit/update/setup-ssh/snapshot/worktree + status passthrough; fleet sync is
safer than raw `meta exec` pull/push loops.

## Substrate command families (inventory — init uses inspection verbs only)

- **git-kb**: initialize/doctor/fsck/repair/info · create/show/list/search/rm/set/assign/mv/
  templates · link/unlink/reorder/graph/board/view · checkout/status/diff/commit/uncommit/
  stash/reset · `code` (index/callers/callees/impact/doctor/query).
- **grit**: init · claim · release · status · symbols · plan · done · watch · worktree · queue ·
  gc · session · config · assign · reconcile · heartbeat. Parallel code work = claim/heartbeat/
  release with worktree isolation.
- **icm**: store/remember · recall · list · forget · update · health · facts/feedback/
  transcripts/sessions · wake-up · context · save-project · hooks · cloud · MCP serve.
- **weave**: setup/uninstall/provider-switch · register/attach/peers/scan/sessions/connect ·
  send/inbox/export/backup/restore · ask/answer/ack/asks/ask-many · job create/list/show/claim/
  dispatch/update/result/cancel · orchestrator claim/status · describe/status/daemon ·
  notify/delivery/inject · spawn/kill · mcp · outbox/pull · web · key/audit · dashboards.
  A missing frontdoor is a gap — never invent commands from memory.

## Yazelix root-variable contract

`YAZELIX_CONFIG_DIR` = editable config root; `YAZELIX_STATE_DIR` = generated state;
`YAZELIX_RUNTIME_DIR` = shipped runtime assets. Never substitute `YAZELIX_DIR` as canonical
ownership. Editable input: `~/.config/yazelix/` (+ managed override sidecars). Generated proof
(never hand-edit): `~/.local/share/yazelix/`. Managed shell hooks: `shell_bash.sh`,
`shell_zsh.zsh`, `shell_fish.fish`, `shell_nu.nu`, host-owned `shell_xonsh.xsh`.

## Authority split (decision/receipt)

Tracked policy/config/tests are durable authority. Ignored state, ledgers, and runtime reports
are receipts — proof of execution, never completion authority by themselves. A runtime receipt
alone is not "done".
