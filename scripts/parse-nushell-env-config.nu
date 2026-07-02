#!/usr/bin/env nu

# Parse Nushell env/config fragments into inspectable table rows.
# Nushell's own IDE parser is used for file-level syntax/AST evidence; this
# command then records stable high-level rows and marks unsupported constructs
# as raw fallback rows for later manual review.

def sensitive-text [text: string] {
    ($text | str upcase) =~ '(TOKEN|SECRET|PASSWORD|PASSWD|API_KEY|ACCESS_KEY|AUTH|COOKIE|PRIVATE_KEY|CREDENTIAL)'
}

def redacted-row [
    source_path: string
    line_no: int
    row_kind: string
    key
    value
    value_kind: string
    owner: string
    source_role: string
    runtime_role: string
    precedence_scope: string
    needs_review: bool
    sensitive: bool
    parse_status: string
    ide_check_exit_code: int
    ide_message_count: int
    ast_token_count: int
    raw_line: string
] {
    {
        source_path: $source_path
        line_no: $line_no
        end_line_no: $line_no
        row_kind: $row_kind
        key: $key
        value: (if $sensitive { null } else { $value })
        redacted_value: (if $sensitive { "<redacted>" } else { $value })
        value_kind: $value_kind
        owner: $owner
        source_role: $source_role
        runtime_role: $runtime_role
        precedence_scope: $precedence_scope
        needs_review: $needs_review
        sensitive: $sensitive
        parse_status: $parse_status
        ide_check_exit_code: $ide_check_exit_code
        ide_message_count: $ide_message_count
        ast_token_count: $ast_token_count
        raw_line: (if $sensitive { "<redacted>" } else { $raw_line })
    }
}

def ide-evidence [file_path: path] {
    let check = (^nu --ide-check 100 $file_path | complete)
    let ast = (^nu --ide-ast $file_path | complete)
    let ast_count = if ($ast.exit_code == 0 and (($ast.stdout | str trim) != "")) {
        $ast.stdout | from json | length
    } else {
        0
    }
    {
        ide_check_exit_code: $check.exit_code
        ide_message_count: (($check.stdout | lines | where ($it | str trim) != "" | length) + ($check.stderr | lines | where ($it | str trim) != "" | length))
        ast_token_count: $ast_count
    }
}

def first-match [line: string pattern: string] {
    let matches = ($line | parse -r $pattern)
    if (($matches | length) == 0) {
        null
    } else {
        $matches | first
    }
}

def parse-nu-line [
    source_path: string
    line_no: int
    raw_line: string
    owner: string
    source_role: string
    runtime_role: string
    precedence_scope: string
    ide_check_exit_code: int
    ide_message_count: int
    ast_token_count: int
] {
    let trimmed = ($raw_line | str trim)

    if $trimmed == "" {
        null
    } else if ($trimmed | str starts-with "#") {
        null
    } else if ($trimmed in ["{" "}"]) {
        redacted-row $source_path $line_no block_delimiter null $trimmed nu_syntax $owner $source_role $runtime_role $precedence_scope false false ok $ide_check_exit_code $ide_message_count $ast_token_count $trimmed
    } else {
        let source = (first-match $trimmed '^\s*source\s+(?P<value>.+?)\s*$')
        let const_decl = (first-match $trimmed '^\s*const\s+(?P<key>[A-Za-z_][A-Za-z0-9_-]*)\s*=\s*(?P<value>.+)$')
        let let_decl = (first-match $trimmed '^\s*let\s+(?P<key>[A-Za-z_][A-Za-z0-9_-]*)\s*=\s*(?P<value>.+)$')
        let env_assign = (first-match $trimmed '^\s*\$env\.(?P<key>[A-Za-z_][A-Za-z0-9_]*)\s*=\s*(?P<value>.+)$')
        let def_decl = (first-match $trimmed '^\s*def(?:\s+--wrapped)?\s+(?P<key>[A-Za-z0-9_-]+)\s*(?P<value>.*)$')
        let guard = (first-match $trimmed '^\s*if\s+(?P<value>.+)$')

        if $source != null {
            redacted-row $source_path $line_no source null $source.value path $owner $source_role $runtime_role $precedence_scope false false ok $ide_check_exit_code $ide_message_count $ast_token_count $trimmed
        } else if $const_decl != null {
            let sensitive = ((sensitive-text $const_decl.key) or (sensitive-text $const_decl.value))
            redacted-row $source_path $line_no const $const_decl.key $const_decl.value nu_expr $owner $source_role $runtime_role $precedence_scope false $sensitive ok $ide_check_exit_code $ide_message_count $ast_token_count $trimmed
        } else if $let_decl != null {
            let sensitive = ((sensitive-text $let_decl.key) or (sensitive-text $let_decl.value))
            redacted-row $source_path $line_no let $let_decl.key $let_decl.value nu_expr $owner $source_role $runtime_role $precedence_scope false $sensitive ok $ide_check_exit_code $ide_message_count $ast_token_count $trimmed
        } else if $env_assign != null {
            let sensitive = ((sensitive-text $env_assign.key) or (sensitive-text $env_assign.value))
            redacted-row $source_path $line_no env_assignment $env_assign.key $env_assign.value nu_expr $owner $source_role $runtime_role $precedence_scope false $sensitive ok $ide_check_exit_code $ide_message_count $ast_token_count $trimmed
        } else if $def_decl != null {
            let sensitive = ((sensitive-text $def_decl.key) or (sensitive-text $def_decl.value))
            redacted-row $source_path $line_no def $def_decl.key $def_decl.value nu_def $owner $source_role $runtime_role $precedence_scope false $sensitive ok $ide_check_exit_code $ide_message_count $ast_token_count $trimmed
        } else if $guard != null {
            let sensitive = (sensitive-text $guard.value)
            redacted-row $source_path $line_no guard null $guard.value nu_condition $owner $source_role $runtime_role $precedence_scope false $sensitive ok $ide_check_exit_code $ide_message_count $ast_token_count $trimmed
        } else {
            let sensitive = (sensitive-text $trimmed)
            redacted-row $source_path $line_no raw null (if $sensitive { null } else { $trimmed }) raw $owner $source_role $runtime_role $precedence_scope true $sensitive raw_fallback $ide_check_exit_code $ide_message_count $ast_token_count $trimmed
        }
    }
}

def main [
    file_path: path
    --owner: string = "unknown"
    --source-role: string = "original"
    --runtime-role: string = "nushell_config"
    --precedence-scope: string = "nushell_config"
    --json
] {
    let source_path = ($file_path | path expand)
    let evidence = (ide-evidence $file_path)
    let parsed_rows = (
        open --raw $file_path
        | lines
        | enumerate
        | each {|row|
            parse-nu-line $source_path ($row.index + 1) $row.item $owner $source_role $runtime_role $precedence_scope $evidence.ide_check_exit_code $evidence.ide_message_count $evidence.ast_token_count
        }
        | where $it != null
    )
    let summary = (redacted-row $source_path 0 file_summary null $"active_rows=(($parsed_rows | length))" summary $owner $source_role $runtime_role $precedence_scope false false (if (($parsed_rows | length) == 0) { "no_active_rows" } else { "ok" }) $evidence.ide_check_exit_code $evidence.ide_message_count $evidence.ast_token_count "")
    let rows = ([$summary] ++ $parsed_rows)

    if $json {
        $rows | to json
    } else {
        $rows
    }
}
