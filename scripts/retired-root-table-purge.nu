#!/usr/bin/env nu

use ./meta-paths.nu *

# Purge the retired /home/flexnetos/FlexNetOS root from every envctl table.
#
# Dry-run by default; --apply archives every table it touches, publishes each
# candidate atomically, verifies zero residue, and writes one hash receipt.
#
# WHY A REWRITE IS LEGITIMATE HERE: the generator scripts no longer emit the
# retired root (verified -- they derive from meta-paths.nu), so these rows are
# stale historical output, not something a re-run would reproduce. Where a table
# CAN be re-derived (the *_parser evidence and *_parsed staging tables) that is
# the better fix and is left to its own parser; this committer only removes a
# root that no longer exists on disk from values that still name it.
#
# TARGET FORM: `$META_ROOT/...`, not an absolute path. That is the existing table
# convention (see the secretd_toml row in env_precedence.csv) and it is what keeps
# the tables portable -- no user's home is pinned into declarative state.
#
# EXCLUDED: bootstrap_env_vars.csv. generate-bootstrap.nu renders those values
# VERBATIM into shell exports and performs no variable expansion, so a
# `$META_ROOT/...` value there would emit a literal, unexpanded string. That table
# is already free of the retired root and must stay absolute.

const RETIRED_ROOT = "/home/flexnetos/FlexNetOS"
const PORTABLE = "$META_ROOT"
const EXCLUDED = ["bootstrap_env_vars.csv"]

def fail [message: string] {
    print --stderr $"retired root table purge: ($message)"
    exit 1
}

def main [
    --meta-root: string = ""
    --timestamp: string = ""
    --apply
] {
    let root = (meta-root $meta_root)
    let tables_root = ($root | path join "var" "lib" "envctl" "tables")
    if not ($tables_root | path exists) { fail $"tables root is missing: ($tables_root)" }

    let candidates = (
        glob $"($tables_root)/*.csv"
        | where {|p| ($p | path basename) not-in $EXCLUDED }
        | sort
    )

    let affected = ($candidates | each {|p|
        let raw = (open --raw $p)
        let hits = (($raw | split row $RETIRED_ROOT | length) - 1)
        {
            table: ($p | path basename)
            path: $p
            occurrences: $hits
            before_sha256: ($raw | hash sha256)
            after_sha256: (($raw | str replace --all $RETIRED_ROOT $PORTABLE) | hash sha256)
        }
    } | where occurrences > 0)

    let observed_at = if ($timestamp | is-empty) {
        date now | format date "%Y%m%dT%H%M%S%3fZ"
    } else {
        $timestamp
    }
    let archive = ($root | path join "var" "lib" "envctl" "archives" "retired-root-table-purge" $observed_at)

    mut receipt = {
        schema: "envctl.retired-root-table-purge.v1"
        observed_at: $observed_at
        applied: $apply
        tables_root: ($tables_root | into string)
        retired_root: $RETIRED_ROOT
        portable_form: $PORTABLE
        excluded: $EXCLUDED
        archive: ($archive | into string)
        tables_affected: ($affected | length)
        occurrences_total: ($affected | get occurrences | math sum | default 0)
        tables: $affected
        verified: false
    }

    if $apply {
        mkdir $archive
        for entry in $affected {
            cp $entry.path ($archive | path join $"($entry.table).before")
            let rewritten = (open --raw $entry.path | str replace --all $RETIRED_ROOT $PORTABLE)
            let candidate = $"($entry.path).envctl-candidate"
            $rewritten | save --raw --force $candidate
            mv --force $candidate $entry.path
        }

        # Fail closed: no table outside the exclusion list may still name the root.
        let residual = ($candidates | each {|p|
            {table: ($p | path basename), hits: ((open --raw $p | split row $RETIRED_ROOT | length) - 1)}
        } | where hits > 0)

        if not ($residual | is-empty) {
            for entry in $affected {
                cp ($archive | path join $"($entry.table).before") $entry.path
            }
            fail $"post-commit residue remains in ($residual | get table | str join ','); all tables restored"
        }

        $receipt = ($receipt | upsert verified true)
        $receipt | to json --indent 2 | save --force ($archive | path join "retired-root-table-purge.receipt.json")
    }

    $receipt | to json --indent 2
}
