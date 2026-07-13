# Yazelix-managed Nushell hook
# Add Nushell-only commands for Yazelix sessions here

# n8n docker boot helpers (added by envctl) — work inside yazelix/nushell
# Owner ruling 2026-07-07: no META_ROOT/LIFEOS_ROOT wiring — tools resolve
# from PATH (the Nix profile owns runtime); absent tool = clear error from nu.
def n8n-up [...rest] { ^n8n-up ...$rest }
def n8n-down [...rest] { ^n8n-down ...$rest }

# RTK command routing is owned by Yazelix's packaged managed Nushell config
# (`nushell/config/rtk_wrappers.nu`). Do not duplicate those defs in this
# editable user hook.

# === legacy workspace /usr mirror on PATH ================================
# meta-usr-path.nu is GUARDED on $env.META_ROOT, which session wiring no longer
# sets (owner ruling 2026-07-07: no META_ROOT/LIFEOS_ROOT) — so this source is
# an inert no-op kept for the migration sweep; runtime ownership = Nix profile.
source ../nushell/meta-usr-path.nu
# =========================================================================

# === rtk monitor pane (live coverage + savings) ==========================
# `rtk-mon` opens it on demand; it also auto-opens ONCE per zellij session.
# Opt out: set $env.RTK_MONITOR_AUTOSTART = "0" before nu starts.
def rtk-mon [] { if (which rtk-monitor | is-not-empty) { ^zellij run --name rtk --direction down -- rtk-monitor } }
if ("ZELLIJ_SESSION_NAME" in $env) and (($env.RTK_MONITOR_AUTOSTART? | default "1") != "0") and (which rtk-monitor | is-not-empty) {
    let marker = $"/tmp/rtk-monitor-($env.ZELLIJ_SESSION_NAME).lock"
    if not ($marker | path exists) {
        touch $marker
        do { ^zellij run --name rtk --direction down -- rtk-monitor } | complete | ignore
    }
}
# =========================================================================
