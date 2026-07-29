#!/usr/bin/env nu

use ./meta-paths.nu *

# Commit the durable tool-state anchors into envctl's canonical bootstrap table.
# Dry-run is the default; --apply archives the exact prior table, publishes one
# candidate atomically, verifies every committed row, and emits a hash receipt.
#
# Protocol and receipt shape mirror scripts/profile-env-cutover.nu so the two
# cutovers stay auditable the same way.
#
# Why these rows (2026-07-27 session-env incident):
#   ICM_DB           ICM resolves its database only from --db / ICM_DB / the XDG
#                    default -- the [store].path config key is parsed but ignored.
#                    icm-web.service runs under the systemd user-manager env with
#                    no --db flag, so without an explicit anchor a restored
#                    XDG_DATA_HOME silently serves a stale ~/.local/share copy
#                    (5.6M, 2026-07-21) instead of the live table (36M).
#   CARGO_HOME       The prior value named a workspace-local candidate path that
#   CARGO_TARGET_DIR does not exist on this host, while the operative environment
#                    pointed both at /run/user/1001 -- XDG_RUNTIME_DIR, whose tmpfs
#                    budget is shared with the wayland socket, dconf, dbus and
#                    gnome-keyring. A single workspace build reached 30G there and
#                    drove the tmpfs to 84%. Build artifacts are durable, not
#                    volatile, and the authoritative table must say so.
#
# This script deliberately does NOT touch XDG_DATA_HOME / XDG_STATE_HOME /
# XDG_CACHE_HOME. Those rows already name the real user roots, and redirecting
# them at session scope is what re-homes the GNOME keyring, icons, launchers and
# Trash. See scripts/profile-env-cutover.nu for the separate, owner-gated
# proposal to move them to the profile-owned Meta roots.

def fail [message: string] {
    print --stderr $"session env anchors cutover: ($message)"
    exit 1
}

const ANCHORS = [
    {
        name: "ICM_DB"
        value: $"($root)/var/xdg-data/icm/memories.db"
        value_kind: "path"
        owner_table: "env_vars"
        scope: "user"
        precedence: "90"
        sensitivity: "local_state"
        source_ref: "T006"
        generated_target: "bootstrap.nu;bootstrap.sh"
        notes: "ICM durable memory database; explicit anchor because ICM ignores [store].path."
    }
    {
        name: "CARGO_HOME"
        value: $"($root)/var/cache/cargo-home"
        value_kind: "path"
        owner_table: "rust_toolchains"
        scope: "user"
        precedence: "70"
        sensitivity: "local_state"
        source_ref: "T006"
        generated_target: "bootstrap.nu;bootstrap.sh"
        notes: "Durable cargo registry root; must never resolve under XDG_RUNTIME_DIR tmpfs."
    }
    {
        name: "CARGO_TARGET_DIR"
        value: $"($root)/var/cargo-target"
        value_kind: "path"
        owner_table: "rust_toolchains"
        scope: "user"
        precedence: "70"
        sensitivity: "local_state"
        source_ref: "T006"
        generated_target: "bootstrap.nu;bootstrap.sh"
        notes: "Durable cargo build-artifact root; must never resolve under XDG_RUNTIME_DIR tmpfs."
    }
]

def main [
    --meta-root: string = ""
    --timestamp: string = ""
    --apply
] {
    let root = (meta-root $meta_root)
    let tables_root = ($root | path join "var" "lib" "envctl" "tables")
    let table = ($tables_root | path join "bootstrap_env_vars.csv")
    if not ($table | path exists) {
        fail $"canonical table is missing: ($table)"
    }

    let rows = (open $table)
    let columns = ($rows | columns)

    # Every anchor must be expressible in the table's exact schema; a schema drift
    # has to fail closed rather than silently write a ragged row.
    for anchor in $ANCHORS {
        let anchor_columns = ($anchor | columns)
        let missing = ($columns | where {|c| $c not-in $anchor_columns })
        if (($missing | length) != 0) {
            fail $"anchor ($anchor.name) cannot fill columns: ($missing | str join ',')"
        }
    }

    # Secret-ref rows must never be retargeted at generated bootstrap files.
    for anchor in $ANCHORS {
        if ($anchor.sensitivity == "secret_ref") or ($anchor.value_kind == "secret_ref") {
            fail $"anchor ($anchor.name) is a secret ref and cannot target generated files"
        }
    }

    let anchor_names = ($ANCHORS | get name)
    let retained = ($rows | where {|row| $row.name not-in $anchor_names })
    let projected = ($ANCHORS | each {|anchor|
        $columns | reduce --fold {} {|col, acc| $acc | upsert $col ($anchor | get $col) }
    })
    let updated = ($retained ++ $projected | sort-by precedence name)

    let before_hash = (open --raw $table | hash sha256)
    let rendered = ($updated | to csv)
    let after_hash = ($rendered | hash sha256)
    let observed_at = if ($timestamp | is-empty) {
        date now | format date "%Y%m%dT%H%M%S%3fZ"
    } else {
        $timestamp
    }
    let archive = ($root | path join "var" "lib" "envctl" "archives" "session-env-anchors-cutover" $observed_at)

    mut receipt = {
        schema: "envctl.session-env-anchors-cutover.v1"
        observed_at: $observed_at
        applied: $apply
        table: ($table | into string)
        archive: ($archive | into string)
        anchors: ($ANCHORS | select name value)
        xdg_rows_untouched: ["XDG_DATA_HOME" "XDG_STATE_HOME" "XDG_CACHE_HOME" "XDG_CONFIG_HOME"]
        before_sha256: $before_hash
        after_sha256: $after_hash
        verified: false
    }

    if $apply {
        mkdir $archive
        cp $table ($archive | path join "bootstrap_env_vars.csv.before")
        let candidate = ($tables_root | path join "bootstrap_env_vars.csv.envctl-candidate")
        $rendered | save --raw --force $candidate
        mv --force $candidate $table

        let committed = (open $table)
        let anchors_ok = ($ANCHORS | all {|anchor|
            let matches = ($committed | where {|row| $row.name == $anchor.name and $row.value == $anchor.value })
            ($matches | length) == 1
        })
        # The XDG rows are the blast radius of this whole incident: prove the
        # cutover left them exactly as found before accepting the commit.
        let xdg_ok = (["XDG_DATA_HOME" "XDG_STATE_HOME" "XDG_CACHE_HOME" "XDG_CONFIG_HOME"] | all {|name|
            let before = ($rows | where {|row| $row.name == $name })
            let after = ($committed | where {|row| $row.name == $name })
            (($before | length) == 1) and (($after | length) == 1) and (($before | get 0.value) == ($after | get 0.value))
        })
        if not ($anchors_ok and $xdg_ok) {
            cp ($archive | path join "bootstrap_env_vars.csv.before") $table
            fail "post-commit verification failed; prior table restored"
        }
        $receipt = ($receipt | upsert verified true)
        $receipt | to json --indent 2 | save --force ($archive | path join "session-env-anchors-cutover.receipt.json")
    }

    $receipt | to json --indent 2
}
