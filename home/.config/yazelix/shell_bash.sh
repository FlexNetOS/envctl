# Yazelix-managed Bash hook
# Add Bash-only commands for Yazelix sessions here

# === rtk (Rust Token Killer) auto-routing — BEGIN =========================
# Routes interactive dev-tool commands through `rtk <cmd>` for token-optimized
# output in yazelix bash panes (all meta repos). Aliases only expand in
# interactive bash — scripts and Claude Code's non-interactive `bash -c` tool
# are unaffected (Claude already routes via its own rtk PreToolUse hook).
# Escape hatch: prefix with `\` or `command`, e.g. `\git log` for raw git.
# Skipped on purpose: ls/find/grep/tree/wc (coreutils raw output expected).
if command -v rtk >/dev/null 2>&1; then
  for _rtk_cmd in git gh glab gt cargo go pnpm npm npx tsc prettier jest \
    vitest playwright prisma pip pytest ruff mypy rake rubocop rspec dotnet \
    gradlew golangci-lint docker kubectl aws psql curl wget; do
    alias "$_rtk_cmd"="rtk $_rtk_cmd"
  done
  unset _rtk_cmd
fi
# === rtk (Rust Token Killer) auto-routing — END ===========================

# === rtk monitor pane (live coverage + savings) ===========================
# `rtk-mon` opens it on demand; it also auto-opens ONCE per zellij session.
# Opt out: export RTK_MONITOR_AUTOSTART=0 before bash starts.
# Owner ruling 2026-07-07: no META_ROOT/LIFEOS_ROOT wiring (the old block here
# exported a /home/drdave/Desktop/meta fallback — pre-clean-room debris). The
# tool resolves from PATH (the Nix profile owns runtime); absent tool = no-op.
alias rtk-mon='command -v rtk-monitor >/dev/null 2>&1 && zellij run --name rtk --direction down -- rtk-monitor'
if [ -n "${ZELLIJ_SESSION_NAME:-}" ] && [ "${RTK_MONITOR_AUTOSTART:-1}" != "0" ] \
   && command -v rtk-monitor >/dev/null 2>&1; then
  _rtk_marker="/tmp/rtk-monitor-${ZELLIJ_SESSION_NAME}.lock"
  if [ ! -e "$_rtk_marker" ]; then
    : > "$_rtk_marker"
    zellij run --name rtk --direction down -- rtk-monitor >/dev/null 2>&1 || true
  fi
  unset _rtk_marker
fi
# =========================================================================
