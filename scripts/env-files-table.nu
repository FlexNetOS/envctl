#!/usr/bin/env nu

# Read the env_files CSV inventory as a typed Nushell table.
# This command is intentionally read-only and performs no file discovery itself.
def main [
    table_path: path
    --domain: string
    --owner: string
    --sensitivity: string
    --state-classification: string
    --runtime-role: string
    --json
] {
    let rows = (
        open $table_path
        | upsert exists {|row| ($row.exists | into string) == "true" }
    )

    mut filtered = $rows

    if $domain != null {
        $filtered = ($filtered | where domain == $domain)
    }
    if $owner != null {
        $filtered = ($filtered | where owner == $owner)
    }
    if $sensitivity != null {
        $filtered = ($filtered | where sensitivity == $sensitivity)
    }
    if $state_classification != null {
        $filtered = ($filtered | where state_classification == $state_classification)
    }
    if $runtime_role != null {
        $filtered = ($filtered | where runtime_role == $runtime_role)
    }

    if $json {
        $filtered | to json
    } else {
        $filtered
    }
}
