#!/usr/bin/env nu

use ./meta-paths.nu *

# Fail-closed check for the state-tier law. Read-only; --strict exits 1 on any
# blocking violation. Run it the way validate-env-tables.nu is run.
#
# THE LAW (previously undocumented, which is why it kept regressing):
#
#   $META_ROOT/var/lib/<agent>       agent homes, pinned EXPLICITLY by each
#                                    profile frontdoor's STATE_HOME (codex, claude)
#   $META_ROOT/var/lib/<service>     FHS service state (postgresql, redb, agentdb,
#                                    codedb, envctl, ...)
#   $META_ROOT/var/xdg-data/<tool>   XDG-following tools, via XDG_DATA_HOME
#                                    (icm, rtk, zoxide, weave, yazelix)
#   $META_ROOT/var/xdg-state/<tool>  XDG-following state, via XDG_STATE_HOME
#
#   ~/.local                         NEVER. No FlexNetOS tool state, ever.
#   /home/flexnetos/FlexNetOS        NEVER. Retired root, removed.
#   /run/user/**                     NEVER. No FlexNetOS runtime or state.
#
# WHY THE SPLIT IS NOT REDUNDANT: XDG_DATA_HOME is the only lever that moves a
# spec-following tool, and it drags GNOME with it. Pointing it at var/lib -- the
# FHS service tier -- is what put GNOME's keyring, icons and Trash next to
# PostgreSQL on 2026-07-27. var/xdg-data exists to keep XDG-shaped data out of
# the FHS tier. Session scope deliberately stays on the real home; only the
# per-process profile frontdoors redirect.

const RETIRED_ROOT = "/home/flexnetos/FlexNetOS"
const AGENT_HOMES = ["codex" "claude"]
const XDG_TOOLS = ["icm" "rtk" "zoxide" "weave" "yazelix" "atc"]
# Home-owned agent directories. The profile frontdoor is the sole owner and
# exports CODEX_HOME / CLAUDE_CONFIG_DIR into the Meta payload; a directory here
# is a competing owner that tools silently fall back to.
const RETIRED_AGENT_SHADOWS = [".codex" ".claude"]
const DESKTOP_NAMES = [
    "evolution" "gnome-settings-daemon" "gnome-shell" "gvfs-metadata" "ibus-table"
    "icc" "mimeapps.list" "nautilus" "org.gnome.TextEditor" "pki" "sounds"
    "user-session-migration" "keyrings" "recently-used.xbel"
    "gnome-session@ubuntu.state"
]

def finding [id: string, severity: string, blocking: string, status: string, message: string] {
    {check: $id, severity: $severity, blocking: $blocking, status: $status, message: $message}
}

def main [--json, --strict, --meta-root: string = ""] {
    let root = (meta-root $meta_root)
    let lib = ($root | path join "var" "lib")
    let xdg = ($root | path join "var" "xdg-data")
    let tables = ($root | path join "var" "lib" "envctl" "tables")
    let home = ($env.HOME | default "/home/flexnetos")

    mut out = []

    # 1. the retired root must not exist, anywhere
    $out = ($out | append (
        finding "retired_root_absent" "error" "true"
            (if ($RETIRED_ROOT | path exists) { "error" } else { "pass" })
            $"($RETIRED_ROOT) must not exist"
    ))

    # 2. no FlexNetOS tool state under ~/.local
    for t in $XDG_TOOLS {
        for sub in ["share" "state"] {
            let p = ($home | path join ".local" $sub $t)
            $out = ($out | append (
                finding $"dotlocal_absent_($sub)_($t)" "error" "true"
                    (if ($p | path exists) { "error" } else { "pass" })
                    $"($p) must not exist"
            ))
        }
    }

    # 3. an XDG tool must live in xdg-data, not the FHS tier
    for t in $XDG_TOOLS {
        let in_lib = (($lib | path join $t) | path exists)
        $out = ($out | append (
            finding $"tier_no_lib_copy_($t)" "error" "true"
                (if $in_lib { "error" } else { "pass" })
                $"($lib)/($t) duplicates the XDG tier; the live copy is ($xdg)/($t)"
        ))
    }

    # 4. agent homes stay pinned in var/lib
    for a in $AGENT_HOMES {
        let p = ($lib | path join $a)
        $out = ($out | append (
            finding $"agent_home_present_($a)" "error" "true"
                (if ($p | path exists) { "pass" } else { "error" })
                $"($p) is the frontdoor-pinned agent home and must exist"
        ))
    }

    # 5. desktop data must not sit in the FHS service tier
    for d in $DESKTOP_NAMES {
        let p = ($lib | path join $d)
        $out = ($out | append (
            finding $"desktop_not_in_fhs_($d)" "error" "true"
                (if ($p | path exists) { "error" } else { "pass" })
                $"($p) is desktop data in the FHS service tier"
        ))
    }

    # 6. tables must name neither retired location
    if ($tables | path exists) {
        let files = (glob $"($tables)/*.csv")
        let retired_hits = ($files | each {|p| (open --raw $p | split row $RETIRED_ROOT | length) - 1} | math sum)
        $out = ($out | append (
            finding "tables_no_retired_root" "error" "true"
                (if $retired_hits > 0 { "error" } else { "pass" })
                $"($retired_hits) retired-root occurrences across envctl tables"
        ))
        let dotlocal_tool_hits = ($files | each {|p|
            let retired = ([$home ".local" "share" "yazelix"] | path join)
            (open --raw $p | split row $retired | length) - 1
        } | math sum)
        $out = ($out | append (
            finding "tables_no_dotlocal_tool_state" "error" "true"
                (if $dotlocal_tool_hits > 0 { "error" } else { "pass" })
                $"($dotlocal_tool_hits) retired .local tool-state occurrences across envctl tables"
        ))
    }

    # 7. no durable agent state on the volatile runtime
    for a in $AGENT_HOMES {
        let p = ("/home/flexnetos/meta/var/lib/yazelix/runtime/profile-runtime" | path join $a)
        $out = ($out | append (
            finding $"no_tmpfs_agent_state_($a)" "error" "true"
                (if ($p | path exists) { "error" } else { "pass" })
                $"($p) puts durable agent state on tmpfs"
        ))
    }

    # 8. no home-owned agent shadow beside the profile-owned home
    for s in $RETIRED_AGENT_SHADOWS {
        let p = ($home | path join $s)
        $out = ($out | append (
            finding $"no_agent_shadow_($s)" "error" "true"
                (if ($p | path exists) { "error" } else { "pass" })
                $"($p) is a competing agent home; the profile frontdoor owns it"
        ))
    }

    let failures = ($out | where status == "error")
    if $json {
        print ({checks: ($out | length), failures: ($failures | length), findings: $out} | to json --indent 2)
    } else {
        print $"state-tier check: (($out | length) - ($failures | length))/($out | length) pass"
        for x in $failures { print $"  FAIL ($x.check): ($x.message)" }
    }
    if $strict and (($failures | length) > 0) { exit 1 }
}
