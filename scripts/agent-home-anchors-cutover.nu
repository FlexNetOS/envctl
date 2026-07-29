#!/usr/bin/env nu

use ./meta-paths.nu *

# Commit the remaining agent-home anchors into the canonical bootstrap table.
# Dry-run by default; --apply archives the exact prior table, publishes one
# candidate atomically, verifies every added row, and writes a hash receipt.
#
# Sibling of session-env-anchors-cutover.nu, profile-env-cutover.nu,
# codex-env-cutover.nu and retired-root-cutover.nu.
#
# WHY: CODEX_HOME and CLAUDE_CONFIG_DIR are already anchored, so those two agents
# resolve into the Meta payload. GEMINI_CONFIG_DIR and COPILOT_HOME were never
# anchored, so both still fall back to $HOME/.gemini and $HOME/.copilot -- exactly
# the user-global layout the path law forbids. `icm doctor` now reports that
# fallback explicitly; these rows remove it.
#
# The agent-home tier is META_ROOT/var/lib/<agent>, matching codex and claude,
# because these are agent homes pinned by name rather than XDG-following tool
# state (which belongs under var/xdg-data).

def anchors [root: string] {
    [
        {
            name: "GEMINI_CONFIG_DIR"
            value: $"($root)/var/lib/gemini"
            value_kind: "path"
            owner_table: "agent_homes"
            scope: "user"
            precedence: "60"
            sensitivity: "local_state"
            source_ref: "T006"
            generated_target: "bootstrap.nu;bootstrap.sh"
            notes: "Gemini CLI agent home in the Meta payload; never the user-global ~/.gemini."
        }
        {
            name: "COPILOT_HOME"
            value: $"($root)/var/lib/copilot"
            value_kind: "path"
            owner_table: "agent_homes"
            scope: "user"
            precedence: "60"
            sensitivity: "local_state"
            source_ref: "T006"
            generated_target: "bootstrap.nu;bootstrap.sh"
            notes: "Copilot CLI agent home in the Meta payload; never the user-global ~/.copilot."
        }
    ]
}

def fail [message: string] {
    print --stderr $"agent home anchors cutover: ($message)"
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
    if not ($table | path exists) { fail $"canonical table is missing: ($table)" }

    let rows = (open $table)
    let wanted = (anchors $root)
    let columns = ($rows | columns)

    # Every anchor must match the table's exact column set, or `to csv` would
    # silently reshape the file.
    for a in $wanted {
        let missing = ($columns | where {|c| $c not-in ($a | columns)})
        if not ($missing | is-empty) {
            fail $"anchor ($a.name) is missing column(s): ($missing | str join ',')"
        }
    }

    # No anchor may land on a user-global path.
    for a in $wanted {
        if ($a.value | str starts-with $"($env.HOME)/.") {
            fail $"($a.name) targets a user-global path: ($a.value)"
        }
    }

    let existing = ($wanted | where {|a| ($rows | where name == $a.name | length) > 0} | get name)
    let to_add = ($wanted | where {|a| ($rows | where name == $a.name | length) == 0})

    let updated = (
        $rows
        | each {|row|
            let hit = ($wanted | where name == $row.name)
            if ($hit | is-empty) { $row } else { $hit | first | select ...$columns }
        }
        | append ($to_add | each {|a| $a | select ...$columns })
        | sort-by precedence name
    )

    let before_hash = (open --raw $table | hash sha256)
    let rendered = ($updated | to csv)
    let after_hash = ($rendered | hash sha256)
    let observed_at = if ($timestamp | is-empty) {
        date now | format date "%Y%m%dT%H%M%S%3fZ"
    } else { $timestamp }
    let archive = ($root | path join "var" "lib" "envctl" "archives" "agent-home-anchors-cutover" $observed_at)

    mut receipt = {
        schema: "envctl.agent-home-anchors-cutover.v1"
        observed_at: $observed_at
        applied: $apply
        table: ($table | into string)
        archive: ($archive | into string)
        anchors: ($wanted | select name value)
        already_present: $existing
        added: ($to_add | get name)
        rows_before: ($rows | length)
        rows_after: ($updated | length)
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
        let ok = ($wanted | all {|a|
            (($committed | where {|r| $r.name == $a.name and $r.value == $a.value }) | length) == 1
        })
        if not $ok {
            cp ($archive | path join "bootstrap_env_vars.csv.before") $table
            fail "post-commit verification failed; prior table restored"
        }
        $receipt = ($receipt | upsert verified true)
        $receipt | to json --indent 2 | save --force ($archive | path join "agent-home-anchors-cutover.receipt.json")
    }

    $receipt | to json --indent 2
}
