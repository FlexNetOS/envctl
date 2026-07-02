#!/usr/bin/env nu

# Parse dotenv/shell-style KEY=VALUE files into structured rows.
# The parser is intentionally conservative: unsupported lines become rows with
# parse_status rather than being guessed or silently dropped.

def sensitive-name [name: string] {
    ($name | str upcase) =~ '(TOKEN|SECRET|PASSWORD|PASSWD|API_KEY|ACCESS_KEY|AUTH|COOKIE|PRIVATE_KEY|CREDENTIAL)'
}

def strip-inline-comment [raw: string quoted: bool] {
    if $quoted {
        $raw
    } else {
        $raw | str replace -r '\s+#.*$' '' | str trim
    }
}

def unsafe-value [raw: string value: string] {
    ($raw =~ '\$\(|`|<\(') or ($value =~ '\$\(|`|<\(')
}

def parse-value [raw: string] {
    let trimmed = ($raw | str trim)
    let is_single = (($trimmed | str starts-with "'") and ($trimmed | str ends-with "'") and (($trimmed | str length) >= 2))
    let is_double = (($trimmed | str starts-with '"') and ($trimmed | str ends-with '"') and (($trimmed | str length) >= 2))
    let quoted = ($is_single or $is_double)
    let body = if $quoted {
        $trimmed | str substring 1..-2
    } else {
        strip-inline-comment $trimmed false
    }
    {
        value: $body
        quoted: $quoted
    }
}

def parse-logical-line [source_path: string row: record] {
    let text = ($row.text | str trim)
    let line_no = $row.line_no
    let end_line_no = ($row.end_line_no? | default $line_no)
    let multiline = ($row.multiline? | default false)
    let unterminated = ($row.unterminated? | default false)

    if $text == "" {
        null
    } else if ($text | str starts-with "#") {
        null
    } else if $unterminated {
        {
            source_path: $source_path
            line_no: $line_no
            end_line_no: $end_line_no
            var_name: null
            value: null
            redacted_value: null
            sensitive: false
            quoted: false
            exported: false
            multiline: true
            variable_expansion: false
            duplicate_index: 0
            parse_status: "unterminated_multiline"
            unsafe_pattern: true
            raw_line: $text
        }
    } else {
        let parsed = ($text | parse -r '(?s)^(?P<exported>export\s+)?(?P<name>[A-Za-z_][A-Za-z0-9_]*)=(?P<raw>.*)$')
        if (($parsed | length) == 0) {
            {
                source_path: $source_path
                line_no: $line_no
                end_line_no: $end_line_no
                var_name: null
                value: null
                redacted_value: null
                sensitive: false
                quoted: false
                exported: false
                multiline: $multiline
                variable_expansion: false
                duplicate_index: 0
                parse_status: "invalid_assignment"
                unsafe_pattern: true
                raw_line: $text
            }
        } else {
            let hit = ($parsed | first)
            let name = $hit.name
            let raw_value = $hit.raw
            let value_info = (parse-value $raw_value)
            let sensitive = (sensitive-name $name)
            let unsafe = (unsafe-value $raw_value $value_info.value)
            let variable_expansion = ($value_info.value =~ '(\$\{?[A-Za-z_][A-Za-z0-9_]*\}?)')
            {
                source_path: $source_path
                line_no: $line_no
                end_line_no: $end_line_no
                var_name: $name
                value: (if ($sensitive or $unsafe) { null } else { $value_info.value })
                redacted_value: (if $sensitive { "<redacted>" } else if $unsafe { "<unsafe>" } else { $value_info.value })
                sensitive: $sensitive
                quoted: $value_info.quoted
                exported: ((($hit.exported? | default "") | str trim) != "")
                multiline: $multiline
                variable_expansion: $variable_expansion
                duplicate_index: 0
                parse_status: (if $unsafe { "unsafe_assignment" } else { "ok" })
                unsafe_pattern: $unsafe
                raw_line: (if $sensitive { "<redacted>" } else if $unsafe { "<unsafe>" } else { $text })
            }
        }
    }
}

def build-logical-lines [file_path: path] {
    mut out = []
    mut active = false
    mut start_line = 0
    mut buffer = ""
    mut physical_count = 0

    for row in (open --raw $file_path | lines | enumerate) {
        let line_no = ($row.index + 1)
        let physical = $row.item
        let continues = (($physical | str trim) =~ '\\$')
        let segment = if $continues {
            $physical | str replace -r '\\$' ''
        } else {
            $physical
        }

        if not $active {
            $active = true
            $start_line = $line_no
            $buffer = $segment
            $physical_count = 1
        } else {
            $buffer = $"($buffer)\n($segment)"
            $physical_count = ($physical_count + 1)
        }

        if not $continues {
            $out = ($out ++ [{
                line_no: $start_line
                end_line_no: $line_no
                text: $buffer
                multiline: ($physical_count > 1)
                unterminated: false
            }])
            $active = false
            $start_line = 0
            $buffer = ""
            $physical_count = 0
        }
    }

    if $active {
        $out = ($out ++ [{
            line_no: $start_line
            end_line_no: ($start_line + $physical_count - 1)
            text: $buffer
            multiline: true
            unterminated: true
        }])
    }

    $out
}

def add-duplicate-index [rows: list] {
    mut counts = {}
    mut out = []

    for row in $rows {
        if $row.var_name == null {
            $out = ($out ++ [$row])
        } else {
            let key = $row.var_name
            let previous = ($counts | get -o $key | default 0)
            let next = ($previous + 1)
            $counts = ($counts | upsert $key $next)
            let next_row = ($row | upsert duplicate_index $next)
            $out = ($out ++ [$next_row])
        }
    }

    $out
}

def main [
    file_path: path
    --json
] {
    let source_path = ($file_path | path expand)
    let parsed_rows = (
        build-logical-lines $file_path
        | each {|row|
            parse-logical-line $source_path ({
                line_no: $row.line_no
                end_line_no: $row.end_line_no
                text: $row.text
                multiline: $row.multiline
                unterminated: $row.unterminated
            })
        }
        | where $it != null
    )
    let rows = (add-duplicate-index $parsed_rows)

    if $json {
        $rows | to json
    } else {
        $rows
    }
}
