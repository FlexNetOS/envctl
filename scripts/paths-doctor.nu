#!/usr/bin/env nu

use ./meta-paths.nu *

# ONE command for the path law: `check` reports, `fix` repairs, `explain` prints
# the model. Idempotent — running `fix` on a clean host changes nothing.
#
# WHY THIS EXISTS: every previous fix was a detector. check-state-tiers.nu reports
# 42 violations but repairs none, so a human had to hand-drive each repair, every
# time, in every session. This closes that loop: the same rules that detect also
# repair, and the repair is one command.
#
# It NEVER deletes. Everything it retires is moved under
# $META_ROOT/var/archives/paths-doctor/<stamp>/, so a wrong call is recoverable.
#
# Scope is deliberately the live filesystem. Table rows belong to the envctl
# committers (profile-env / codex-env / retired-root / agent-home cutovers) and
# are only *reported* here, never written, because envctl is their sole owner.

const RETIRED_ROOT = "/home/flexnetos/FlexNetOS"
const AGENT_HOMES = ["codex" "claude" "gemini" "copilot"]
const XDG_TOOLS = ["icm" "rtk" "zoxide" "weave" "yazelix" "atc"]
const DESKTOP_IN_FHS = [
    "evolution" "gnome-settings-daemon" "gnome-shell" "gvfs-metadata" "ibus-table"
    "icc" "mimeapps.list" "nautilus" "org.gnome.TextEditor" "pki" "sounds"
    "user-session-migration" "keyrings" "recently-used.xbel"
    "gnome-session@ubuntu.state"
]
const VOLATILE_ROOT = "/run/user/1001/yazelix"

def finding [id: string, status: string, detail: string, fixable: bool] {
    {check: $id, status: $status, detail: $detail, fixable: $fixable}
}

# Every violation the law defines, as data. check and fix consume the SAME list,
# so a rule can never be enforced by one and missed by the other.
def scan [root: string, home: string] {
    let lib = ($root | path join "var" "lib")
    let xdg = ($root | path join "var" "xdg-data")
    let xdgs = ($root | path join "var" "xdg-state")
    mut out = []

    # 1. retired workspace root
    $out = ($out | append (finding "retired_root" (if ($RETIRED_ROOT | path exists) { "violation" } else { "ok" })
        $"($RETIRED_ROOT) must not exist" true))

    # 2. agent shadows in the user home
    for a in $AGENT_HOMES {
        let p = ($home | path join $".($a)")
        $out = ($out | append (finding $"agent_shadow_($a)" (if ($p | path exists) { "violation" } else { "ok" })
            $"($p) is a competing agent home" true))
    }

    # 3. FlexNetOS tool state under ~/.local
    for t in $XDG_TOOLS {
        for sub in ["share" "state"] {
            let p = ($home | path join ".local" $sub $t)
            $out = ($out | append (finding $"dotlocal_($sub)_($t)" (if ($p | path exists) { "violation" } else { "ok" })
                $"($p) must not hold FlexNetOS state" true))
        }
    }

    # 4. XDG tool duplicated into the FHS service tier
    for t in $XDG_TOOLS {
        let p = ($lib | path join $t)
        $out = ($out | append (finding $"tier_dup_($t)" (if ($p | path exists) { "violation" } else { "ok" })
            $"($p) duplicates ($xdg)/($t)" true))
    }

    # 5. desktop data stranded in the FHS service tier
    for d in $DESKTOP_IN_FHS {
        let p = ($lib | path join $d)
        $out = ($out | append (finding $"desktop_in_fhs_($d)" (if ($p | path exists) { "violation" } else { "ok" })
            $"($p) is desktop data in the service tier" true))
    }

    # 6. durable agent state on tmpfs
    for a in $AGENT_HOMES {
        let p = ($VOLATILE_ROOT | path join "profile-runtime" $a)
        $out = ($out | append (finding $"tmpfs_agent_($a)" (if ($p | path exists) { "violation" } else { "ok" })
            $"($p) puts agent state on tmpfs" true))
    }

    # 7. cargo must never resolve under the volatile runtime.
    #
    # SESSION-GATED. A process's environment is fixed at exec time, so if the LIVE
    # session still has CARGO_HOME/CARGO_TARGET_DIR pointing into the tmpfs, moving
    # the directory out from under it breaks every build in this session — the same
    # restart gate the XDG migration hit (meta/AGENTS.md: "a session cannot restart
    # itself"). Report it, name the lever, and let the next session's env do the work.
    let live = [($env.CARGO_HOME? | default "") ($env.CARGO_TARGET_DIR? | default "")]
    for n in ["cargo-home" "cargo-target" "rustup-home"] {
        let p = ($VOLATILE_ROOT | path join "volatile" $n)
        let bound = ($live | any {|v| $v == $p})
        let st = (if not ($p | path exists) { "ok" } else if $bound { "session-gated" } else { "violation" })
        let d = (if $bound {
            $"($p) is still bound by this session's env — config.nu already points at the durable path; clears on session restart"
        } else {
            $"($p) — cargo/rustup must be durable"
        })
        $out = ($out | append (finding $"tmpfs_($n)" $st $d (not $bound)))
    }

    # 8. agent homes must EXIST (report only; creating one is not a repair)
    for a in $AGENT_HOMES {
        let p = ($lib | path join $a)
        $out = ($out | append (finding $"agent_home_present_($a)" (if ($p | path exists) { "ok" } else { "missing" })
            $"($p) should exist" true))
    }

    # 9. table residue — REPORTED ONLY. envctl committers own these rows.
    let tables = ($root | path join "var" "lib" "envctl" "tables")
    if ($tables | path exists) {
        let files = (glob $"($tables)/*.csv")
        let hits = ($files | each {|p| (open --raw $p | split row $RETIRED_ROOT | length) - 1} | math sum)
        $out = ($out | append (finding "tables_retired_root" (if $hits > 0 { "violation" } else { "ok" })
            $"($hits) retired-root occurrences in envctl tables — run retired-root-table-purge.nu" false))
    }

    $out
}

def repair [f: record, root: string, home: string, archive: string] {
    let lib = ($root | path join "var" "lib")
    let xdg = ($root | path join "var" "xdg-data")
    let xdgs = ($root | path join "var" "xdg-state")

    # Retire by MOVING into the archive, never deleting.
    def retire [src: string, slug: string] {
        if ($src | path exists) {
            mkdir $archive
            let dest = ($archive | path join $slug)
            ^mv -T $src $dest
            print $"    archived ($src) -> ($dest)"
        }
    }

    let id = $f.check
    if ($id == "retired_root") {
        if ($RETIRED_ROOT | path exists) {
            let n = (ls -a $RETIRED_ROOT | length)
            if $n == 0 { rmdir $RETIRED_ROOT; print $"    removed empty ($RETIRED_ROOT)" }
            else { retire $RETIRED_ROOT "FlexNetOS-nonempty" }
        }
    } else if ($id | str starts-with "agent_shadow_") {
        let a = ($id | str replace "agent_shadow_" "")
        retire ($home | path join $".($a)") $"home_.($a)"
    } else if ($id | str starts-with "dotlocal_") {
        let rest = ($id | str replace "dotlocal_" "")
        let sub = ($rest | split row "_" | first)
        let tool = ($rest | str replace $"($sub)_" "")
        let src = ($home | path join ".local" $sub $tool)
        let dst = (if $sub == "share" { ($xdg | path join $tool) } else { ($xdgs | path join $tool) })
        if ($src | path exists) {
            mkdir $dst
            ^rsync -a $"($src)/" $"($dst)/"
            retire $src $"dotlocal_($sub)_($tool)"
            print $"    merged into ($dst)"
        }
    } else if ($id | str starts-with "tier_dup_") {
        retire ($lib | path join ($id | str replace "tier_dup_" "")) $"tierdup_($id | str replace 'tier_dup_' '')"
    } else if ($id | str starts-with "desktop_in_fhs_") {
        retire ($lib | path join ($id | str replace "desktop_in_fhs_" "")) $"desktop_($id | str replace 'desktop_in_fhs_' '')"
    } else if ($id | str starts-with "tmpfs_agent_") {
        retire ($VOLATILE_ROOT | path join "profile-runtime" ($id | str replace "tmpfs_agent_" "")) $"tmpfs_($id)"
    } else if ($id | str starts-with "tmpfs_") {
        retire ($VOLATILE_ROOT | path join "volatile" ($id | str replace "tmpfs_" "")) $"volatile_($id)"
    } else if ($id | str starts-with "agent_home_present_") {
        let p = ($lib | path join ($id | str replace "agent_home_present_" ""))
        mkdir $p
        ^chmod 0700 $p
        print $"    created ($p) mode 0700"
    }
}

def main [
    command?: string = "check"
    --meta-root: string = ""
    --json
] {
    let root = (meta-root $meta_root)
    let home = ($env.HOME | default "/home/flexnetos")

    if $command == "explain" {
        print "PATH LAW — one tier per kind of state"
        print ""
        print $"  ($root)/var/lib/<agent>      agent homes, pinned by each frontdoor STATE_HOME"
        print $"  ($root)/var/lib/<service>    FHS service state: postgresql, redb, agentdb, codedb"
        print $"  ($root)/var/xdg-data/<tool>  XDG-following tools, via XDG_DATA_HOME"
        print $"  ($root)/var/xdg-state/<tool> XDG-following state, via XDG_STATE_HOME"
        print $"  ($root)/var/cache/cargo-home cargo registry - DURABLE, never under /run/user"
        print ""
        print "  ~/.local             NEVER holds FlexNetOS state"
        print "  ~/.<agent>           NEVER — the profile frontdoor owns the agent home"
        print "  /home/flexnetos/FlexNetOS   retired, must not exist"
        print "  /run/user/**         volatile only; no agent state, no cargo"
        print ""
        print "  Toolchain is fenix, not rustup. Build inputs come from a nix"
        print "  derivation's buildInputs, never the user profile."
        print ""
        print "  Before 'fixing' any env/path issue: diff against upstream/main."
        print "  Most are fork-invented, and the fix is deletion, not repointing."
        return
    }

    let found = (scan $root $home)
    let bad = ($found | where status in ["violation" "missing"])
    let gated = ($found | where status == "session-gated")

    if $command == "check" {
        if $json {
            print ({checks: ($found | length), violations: ($bad | length), session_gated: ($gated | length), findings: $found} | to json --indent 2)
        } else {
            print $"paths-doctor: (($found | length) - ($bad | length) - ($gated | length))/($found | length) clean"
            for x in $bad { print $"  ($x.status | str upcase)  ($x.check): ($x.detail)" }
            for x in $gated { print $"  SESSION-GATED  ($x.check): ($x.detail)" }
            if (($bad | is-empty) and ($gated | is-empty)) { print "  no violations" }
        }
        # Session-gated findings are not failures — the source is already correct
        # and only a restart can retire them. Failing on them would make the check
        # permanently red and train everyone to ignore it.
        if not ($bad | is-empty) { exit 1 }
        return
    }

    if $command == "fix" {
        let stamp = (date now | format date "%Y%m%dT%H%M%SZ")
        let archive = ($root | path join "var" "archives" "paths-doctor" $stamp)
        print $"paths-doctor fix — archive: ($archive)"
        let fixable = ($bad | where fixable == true)
        let reported = ($bad | where fixable == false)
        if ($fixable | is-empty) {
            print "  nothing to repair"
        } else {
            for f in $fixable {
                print $"  ($f.check)"
                repair $f $root $home $archive
            }
        }
        for r in $reported { print $"  REPORT-ONLY ($r.check): ($r.detail)" }
        for g in $gated { print $"  SESSION-GATED ($g.check): ($g.detail)" }

        print ""
        let after = (scan $root $home | where status in ["violation" "missing"] | where fixable == true)
        print $"  post-fix fixable violations: ($after | length)"
        for a in $after { print $"    STILL ($a.check): ($a.detail)" }
        if not ($after | is-empty) { exit 1 }
        return
    }

    print --stderr $"unknown command: ($command) — use check | fix | explain"
    exit 2
}
