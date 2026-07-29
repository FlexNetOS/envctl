#!/usr/bin/env nu

use ./meta-paths.nu *

# Commit the strict profile XDG roots into envctl's canonical bootstrap table.
# Dry-run is the default; --apply archives the exact prior table, publishes one
# candidate atomically, verifies the two owner rows, and emits a hash receipt.

def fail [message: string] {
    print --stderr $"profile env cutover: ($message)"
    exit 1
}

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
    for name in ["XDG_DATA_HOME" "XDG_STATE_HOME"] {
        let matches = ($rows | where {|row| $row.name == $name })
        if (($matches | length) != 1) {
            fail $"expected exactly one ($name) row, found ($matches | length)"
        }
    }
    if (($rows | where {|row| $row.name == "YAZELIX_STATE_DIR" } | length) != 0) {
        fail "YAZELIX_STATE_DIR must remain profile-runtime-owned, not a bootstrap-table row"
    }

    let owned_root = ($root | path join "var" "xdg-data" | into string)
    let owned_state_root = ($root | path join "var" "xdg-state" | into string)
    let updated = ($rows | each {|row|
        if $row.name == "XDG_DATA_HOME" {
            $row | upsert value $owned_root | upsert notes "Profile-owned Meta data root."
        } else if $row.name == "XDG_STATE_HOME" {
            $row
              | upsert value $owned_state_root
              | upsert notes "Profile-owned Meta state root."
        } else {
            $row
        }
    })
    let before_hash = (open --raw $table | hash sha256)
    let rendered = ($updated | to csv)
    let after_hash = ($rendered | hash sha256)
    let observed_at = if ($timestamp | is-empty) {
        date now | format date "%Y%m%dT%H%M%S%3fZ"
    } else {
        $timestamp
    }
    let archive = ($root | path join "var" "lib" "envctl" "archives" "profile-env-cutover" $observed_at)
    mut receipt = {
        schema: "envctl.profile-env-cutover.v1"
        observed_at: $observed_at
        applied: $apply
        table: ($table | into string)
        archive: ($archive | into string)
        xdg_data_home: $owned_root
        xdg_state_home: $owned_state_root
        yazelix_state_owner: "/run/user/1001/yazelix/profile-runtime/yazelix"
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
        let verified = (["XDG_DATA_HOME" "XDG_STATE_HOME"] | all {|name|
            let expected = if $name == "XDG_DATA_HOME" {
              $owned_root
            } else {
              $owned_state_root
            }
            let matches = ($committed | where {|row| $row.name == $name and $row.value == $expected })
            ($matches | length) == 1
        })
        if not $verified {
            cp ($archive | path join "bootstrap_env_vars.csv.before") $table
            fail "post-commit verification failed; prior table restored"
        }
        $receipt = ($receipt | upsert verified true)
        $receipt | to json --indent 2 | save --force ($archive | path join "profile-env-cutover.receipt.json")
    }

    $receipt | to json --indent 2
}
