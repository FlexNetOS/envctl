#!/usr/bin/env nu

# Commit the durable Codex state roots into envctl's canonical bootstrap table.
# Dry-run is the default; --apply archives the exact prior table, publishes one
# candidate atomically, verifies every owner row, and emits a hash receipt.
#
# Sibling of profile-env-cutover.nu and the sole authoritative committer for the
# CODEX_* rows. The generated bootstrap artifacts are projections refreshed by
# scripts/generate-bootstrap.nu; they are never a competing authority.
#
# Why this exists: the rows pointed at /home/flexnetos/FlexNetOS/var/lib/codex, a
# retired workspace root. Yazelix's nushell/agent/profile_frontdoor.nu rejects a
# competing owner by comparing CODEX_HOME as a RAW STRING (not realpath) and
# hard-exits on mismatch, so if the bootstrap env were ever regenerated with the
# retired value it would make `codex` unlaunchable and every non-dry-run
# `fxrun forge-loop run` fail. CODEX_HOME must therefore stay byte-identical to
# that frontdoor's STATE_HOME and to DEFAULT_CODEX_HOME in flexnetos_runner's
# crates/runner-cli/src/forge_loop.rs.

# The byte-for-byte contract the yazelix agent frontdoor enforces.
const FRONTDOOR_CODEX_STATE_HOME = "/home/flexnetos/meta/var/lib/codex"

def fail [message: string] {
    print --stderr $"codex env cutover: ($message)"
    exit 1
}

def main [
    --meta-root: path = "/home/flexnetos/meta"
    --timestamp: string = ""
    --apply
] {
    let tables_root = ($meta_root | path join "var" "lib" "envctl" "tables")
    let table = ($tables_root | path join "bootstrap_env_vars.csv")
    if not ($table | path exists) {
        fail $"canonical table is missing: ($table)"
    }

    let owned = {
        CODEX_HOME: ($meta_root | path join "var" "lib" "codex" | into string)
        CODEX_SQLITE_HOME: ($meta_root | path join "var" "lib" "codex" "sqlite" | into string)
        CODEX_LOG_DIR: ($meta_root | path join "var" "log" "codex" | into string)
    }
    let owned_notes = {
        CODEX_HOME: "Durable Codex state root in the Meta payload; byte-identical to the yazelix agent frontdoor STATE_HOME. Auth files remain untracked."
        CODEX_SQLITE_HOME: "Codex SQLite state root under the durable Meta payload."
        CODEX_LOG_DIR: "Codex log target under the durable Meta payload."
    }
    let names = ($owned | columns)

    # Refuse unless the committed CODEX_HOME is exactly what the frontdoor compares
    # against. A trailing slash or a symlink alias here is a live outage.
    if ($owned | get CODEX_HOME) != $FRONTDOOR_CODEX_STATE_HOME {
        fail $"CODEX_HOME ($owned | get CODEX_HOME) is not byte-identical to the frontdoor owner ($FRONTDOOR_CODEX_STATE_HOME)"
    }

    let rows = (open $table)
    for name in $names {
        let matches = ($rows | where {|row| $row.name == $name })
        if (($matches | length) != 1) {
            fail $"expected exactly one ($name) row, found ($matches | length)"
        }
    }

    let prior = ($names | reduce --fold {} {|name, acc|
        $acc | upsert $name ($rows | where {|row| $row.name == $name } | first | get value)
    })

    let updated = ($rows | each {|row|
        if $row.name in $names {
            $row
              | upsert value ($owned | get $row.name)
              | upsert notes ($owned_notes | get $row.name)
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
    let archive = ($meta_root | path join "var" "lib" "envctl" "archives" "codex-env-cutover" $observed_at)
    mut receipt = {
        schema: "envctl.codex-env-cutover.v1"
        observed_at: $observed_at
        applied: $apply
        table: ($table | into string)
        archive: ($archive | into string)
        frontdoor_state_home: $FRONTDOOR_CODEX_STATE_HOME
        prior_values: $prior
        committed_values: $owned
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
        let verified = ($names | all {|name|
            let expected = ($owned | get $name)
            let matches = ($committed | where {|row| $row.name == $name and $row.value == $expected })
            ($matches | length) == 1
        })
        if not $verified {
            cp ($archive | path join "bootstrap_env_vars.csv.before") $table
            fail "post-commit verification failed; prior table restored"
        }
        $receipt = ($receipt | upsert verified true)
        $receipt | to json --indent 2 | save --force ($archive | path join "codex-env-cutover.receipt.json")
    }

    $receipt | to json --indent 2
}
