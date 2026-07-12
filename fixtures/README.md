# envctl db fixtures

Fixture matrices that drive the code-graph db surface (GH#414 / REQ-05x) through
its real engine entry points from integration tests. Fixtures are copied into a
temp workspace before any mutating test runs — the checked-in inputs are never
modified.

## `refactor_matrix/`

Exercises the root-alias refactor + atomic backup-write apply path
(`db_refactor` → `db_atomic::atomic_backup_write`, ARCH08 / MISS07 / NFR08).

The env-token scanner (`db_symbols`) resolves `$VAR` / `${VAR}` references only;
bare-word string literals (e.g. a Rust `"META_ROOT"`) are intentionally not
rewrite targets. A file is a **safe** rewrite only when every `from` occurrence
in it is a replace candidate (its mutable policy is OwnedApply or GuardedApply).

| Path | Kind → policy | Expected under `META_ROOT → LIFE_OS_ROOT` apply |
|------|---------------|--------------------------------------------------|
| `wrappers/wrapper.sh` | shell → OwnedApply (Safe) | rewritten in place; `.bak` holds the original |
| `wrappers/alias.sh`   | shell, `$META_ROOT` + `${LIFEOS_ROOT}` | the `$META_ROOT` token rewrites; the `${LIFEOS_ROOT}` alias (already the `to` side) is left untouched; `.bak` kept |
| `config/paths.toml`   | toml → GuardedApply (NeedsParser) | rewritten via the guarded path; `.bak` kept |
| `src/paths.rs`        | rust, bare-word literal only | no `$`-token occurrence → untouched by refactor (drives the syn symbol pass instead) |
| `secrets/.env`        | protected → Never (Refuse) | **refused**: never modified, no `.bak` |
| `README.md`           | markdown → ReadOnly (ManualReview) | refused: not a safe rewrite target, never modified |

Consumed by `crates/engine/tests/db_refactor_fixtures.rs`.
