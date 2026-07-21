# Portable Codex log placement plan

## Decision

The portable/confined all-in-one app keeps Codex logs out of the copied private
state bundle and writes them to an app-owned log root:

```text
<portable-root>/var/log/codex/
  active/
    logs_2.sqlite
    logs_2.sqlite-shm
    logs_2.sqlite-wal
    history.jsonl
  sessions/
  shell-snapshots/
  execution-reports/
  exports/
```

The portable launcher must set or translate runtime paths so log writes never
fall back to `/run/user/1001/yazelix/profile-runtime/codex`:

```text
CODEX_HOME=<portable-root>/data/codex-home
FLEXNETOS_CODEX_LOG_ROOT=<portable-root>/var/log/codex
XDG_STATE_HOME=<portable-root>/var/state
XDG_CACHE_HOME=<portable-root>/var/cache
```

If Codex does not expose a first-class log directory override for every log
surface, the app wrapper owns the copy/sync boundary:

1. Run Codex against the confined `CODEX_HOME`.
2. Keep active logs under `<portable-root>/var/log/codex`.
3. Export redacted support bundles from `exports/`.
4. Never commit logs to envctl source.

## Platform mapping

| Mode | Log root |
| --- | --- |
| Portable Linux/AppImage-style | `<portable-root>/var/log/codex` |
| Installed Linux | `${XDG_STATE_HOME:-$HOME/var/lib}/flexnetos/codex/logs` |
| macOS | `$HOME/Library/Logs/FlexNetOS/Codex` |
| Windows | `%LOCALAPPDATA%\\FlexNetOS\\Codex\\Logs` |

## Current live log-like surfaces

These live `/run/user/1001/yazelix/profile-runtime/codex` paths are treated as logs or log-adjacent transcript
state and are not copied into `private-codex-state/data`:

```text
logs_2.sqlite
logs_2.sqlite-shm
logs_2.sqlite-wal
history.jsonl
sessions/
shell_snapshots/
execution-reports/
```

## Retention and export

- Default local retention: 30 days or explicit app-configured size ceiling.
- Rotate SQLite/WAL and JSONL logs before portable bundle export.
- Export support bundles only through a redaction pass.
- Keep raw logs local to the app-owned log root unless an operator explicitly
  exports them.

## Research notes

- The XDG Base Directory spec defines `XDG_STATE_HOME` as persistent state and
  explicitly lists logs/history as state data.
- AppImage portable mode demonstrates the portable-root principle by allowing
  data/config directories alongside the application and overriding `$HOME` or
  `$XDG_CONFIG_HOME`.
- Apple documents `Library/Logs` as the user-visible location for log files.
- Microsoft documents `%LOCALAPPDATA%` as the per-user local application data
  root; the Windows portable/installed mapping uses that root for logs.
