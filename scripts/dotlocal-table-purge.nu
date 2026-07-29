#!/usr/bin/env nu

use ./meta-paths.nu *

# Purge retired `~/.local` references from the envctl tables.
#
# Dry-run by default; --apply archives every table it touches, publishes each
# candidate atomically, verifies zero residue for the rewritten classes, and
# writes a hash receipt.
#
# The `.local` law is now enforced on disk: migrate-tool-state-off-dotlocal.sh
# finalize removed ~/.local/share/{icm,rtk,yazelix,weave,env-ctl}. These table
# rows are the last references to locations that no longer exist.
#
# MAPPING, per class -- deliberately explicit, because a blanket prefix swap is
# wrong here the same way it was wrong for the retired workspace root:
#
#   ~/.local/share/yazelix/...  -> $META_ROOT/var/xdg-data/yazelix/...
#       Tool state. XDG_DATA_HOME is meta/var/xdg-data, so this is where the
#       tool actually reads and writes now.
#
#   ~/.local/bin/<tool>         -> $META_ROOT/../.nix-profile/bin/<tool>  NO.
#       Left ALONE. These rows record where a shadow launcher was OBSERVED, and
#       the profile is the only installed launcher owner. Rewriting them would
#       assert a profile path was seen when it was not. They are evidence of a
#       violation, not a location to correct -- and the shadows are already gone
#       from disk.
#
#   ~/.local/share/{evolution,nautilus}, ~/.local/.crates*  -> left ALONE.
#       Not FlexNetOS tool state. evolution/nautilus are GNOME's own data under
#       the session-scope XDG root, which by design stays on the real home; the
#       cargo crates files belong to a rustup/cargo layout this project does not
#       own.

const RETIRED_TOOL_PREFIX = "/home/flexnetos/.local/share/yazelix"
const PORTABLE_TOOL_PREFIX = "$META_ROOT/var/xdg-data/yazelix"
const EXCLUDED = ["bootstrap_env_vars.csv"]

def fail [message: string] {
    print --stderr $"dotlocal table purge: ($message)"
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
        let hits = (($raw | split row $RETIRED_TOOL_PREFIX | length) - 1)
        {
            table: ($p | path basename)
            path: $p
            occurrences: $hits
            before_sha256: ($raw | hash sha256)
            after_sha256: (($raw | str replace --all $RETIRED_TOOL_PREFIX $PORTABLE_TOOL_PREFIX) | hash sha256)
        }
    } | where occurrences > 0)

    let observed_at = if ($timestamp | is-empty) {
        date now | format date "%Y%m%dT%H%M%S%3fZ"
    } else {
        $timestamp
    }
    let archive = ($root | path join "var" "lib" "envctl" "archives" "dotlocal-table-purge" $observed_at)

    mut receipt = {
        schema: "envctl.dotlocal-table-purge.v1"
        observed_at: $observed_at
        applied: $apply
        tables_root: ($tables_root | into string)
        rewritten_prefix: $RETIRED_TOOL_PREFIX
        portable_form: $PORTABLE_TOOL_PREFIX
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
            let rewritten = (open --raw $entry.path | str replace --all $RETIRED_TOOL_PREFIX $PORTABLE_TOOL_PREFIX)
            let candidate = $"($entry.path).envctl-candidate"
            $rewritten | save --raw --force $candidate
            mv --force $candidate $entry.path
        }

        let residual = ($candidates | each {|p|
            {table: ($p | path basename), hits: ((open --raw $p | split row $RETIRED_TOOL_PREFIX | length) - 1)}
        } | where hits > 0)

        if not ($residual | is-empty) {
            for entry in $affected {
                cp ($archive | path join $"($entry.table).before") $entry.path
            }
            fail $"post-commit residue remains in ($residual | get table | str join ','); all tables restored"
        }

        $receipt = ($receipt | upsert verified true)
        $receipt | to json --indent 2 | save --force ($archive | path join "dotlocal-table-purge.receipt.json")
    }

    $receipt | to json --indent 2
}
