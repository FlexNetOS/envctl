#!/usr/bin/env nu

# Validate envctl table state for the v0.1 bootstrap slice.
# This is fail-closed with --strict and writes nothing by itself.

def validation-row [
    validation_id: string
    table_name: string
    row_id: string
    check_name: string
    severity: string
    blocking: string
    status: string
    source_path: string
    message: string
] {
    {
        validation_id: $validation_id
        table_name: $table_name
        row_id: $row_id
        check_name: $check_name
        severity: $severity
        blocking: $blocking
        status: $status
        source_path: $source_path
        message: $message
    }
}

def status-row [
    validation_id: string
    table_name: string
    check_name: string
    ok: bool
    source_path: string
    pass_message: string
    fail_message: string
] {
    if $ok {
        validation-row $validation_id $table_name "" $check_name "info" "false" "ok" $source_path $pass_message
    } else {
        validation-row $validation_id $table_name "" $check_name "error" "true" "error" $source_path $fail_message
    }
}

def read-table [tables_root: string, table_name: string] {
    let path = $"($tables_root)/($table_name).csv"
    if ($path | path exists) {
        open $path
    } else {
        []
    }
}

def table-path [tables_root: string, table_name: string] {
    $"($tables_root)/($table_name).csv"
}

def validate-required-table [tables_root: string, table_name: string, required_columns: list] {
    let path = (table-path $tables_root $table_name)
    if not ($path | path exists) {
        [ (validation-row $"required_table_($table_name)" $table_name "" "table_exists" "error" "true" "error" $path "Required envctl table is missing.") ]
    } else {
        let rows = (open $path)
        let columns = ($rows | columns)
        let missing = ($required_columns | where {|col| $col not-in $columns})
        [
            (status-row $"required_table_($table_name)_exists" $table_name "table_exists" true $path "Required table exists." "Required table is missing.")
            (status-row $"required_table_($table_name)_columns" $table_name "required_columns" (($missing | length) == 0) $path "Required columns are present." $"Missing required columns: ($missing | str join ',').")
            (status-row $"required_table_($table_name)_rows" $table_name "nonempty" (($rows | length) > 0) $path $"Table has ($rows | length) rows." "Required table is empty.")
        ]
    }
}

def validate-required-tables [tables_root: string] {
    let required = [
        {table: "bootstrap_env_vars", columns: ["name", "value_kind", "value", "owner_table", "scope", "precedence", "sensitivity", "generated_target"]}
        {table: "bootstrap_generated_files", columns: ["artifact_id", "path", "role", "source_table", "required_header", "status"]}
        {table: "env_files", columns: ["file_id", "path", "owner", "source_role", "mutability", "sensitivity"]}
        {table: "env_vars_parsed", columns: ["source_path", "var_name", "redacted_value", "sensitive", "parse_status"]}
        {table: "nushell_env_config_parsed", columns: ["source_path", "row_kind", "key", "redacted_value", "owner", "parse_status"]}
        {table: "tool_versions", columns: ["tool_id", "tool_name", "source_path", "source_of_truth", "parse_status"]}
        {table: "secrets", columns: ["secret_id", "secret_name", "value_policy", "read_policy", "classification_status"]}
        {table: "env_precedence", columns: ["precedence_id", "surface", "setting_name", "precedence_rank", "value_policy"]}
        {table: "conflicts", columns: ["conflict_id", "severity", "blocking", "status", "source_refs"]}
        {table: "envctl_tables", columns: ["table_id", "table_name", "durable_role", "source_checksum", "validation_status"]}
        {table: "generated_file_guards", columns: ["guard_id", "artifact_id", "target_path", "silent_overwrite_guard", "validation_status"]}
        {table: "bootstrap_generation_manifest", columns: ["artifact_id", "path", "source_table_checksum", "output_checksum", "header_status", "manual_edits_allowed", "diff_required_before_apply", "secret_policy"]}
    ]
    $required | each {|spec| validate-required-table $tables_root $spec.table $spec.columns} | flatten
}

def validate-bootstrap-env [tables_root: string] {
    let table_name = "bootstrap_env_vars"
    let path = (table-path $tables_root $table_name)
    let rows = (read-table $tables_root $table_name)
    let names = if ($rows | length) == 0 { [] } else { $rows | get name }
    let required_names = ["FLEXNETOS_WORKSPACE", "FLEXNETOS_SRC", "META_ROOT", "ENVCTL_ROOT", "ENVCTL_TABLE_ROOT", "FLEXNETOS_VAR", "FXRUN_STATE_DIR"]
    let missing = ($required_names | where {|name| $name not-in $names})
    let secret_generated = (
        $rows
        | where value_kind == "secret_ref" or sensitivity == "secret_ref"
        | where generated_target =~ "bootstrap"
    )
    [
        (status-row "bootstrap_required_env_names" $table_name "required_env_names" (($missing | length) == 0) $path "Required bootstrap env names are present." $"Missing required bootstrap env names: ($missing | str join ',').")
        (status-row "bootstrap_secret_refs_not_generated" $table_name "secret_refs_not_generated" (($secret_generated | length) == 0) $path "Secret-ref rows are not targeted at generated bootstrap files." "Secret-ref rows are targeted at generated bootstrap files.")
    ]
}

def validate-precedence [tables_root: string] {
    let precedence_path = (table-path $tables_root "env_precedence")
    let conflicts_path = (table-path $tables_root "conflicts")
    let precedence = (read-table $tables_root "env_precedence")
    let conflicts = (read-table $tables_root "conflicts")
    let surfaces = if ($precedence | length) == 0 { [] } else { $precedence | get surface | uniq }
    let required_surfaces = ["codex", "envctl_secretd", "kache", "rtk", "nix", "yazelix", "runner_fxrun", "database"]
    let missing = ($required_surfaces | where {|surface| $surface not-in $surfaces})
    let blocking_errors = ($conflicts | where severity == "error" and blocking == "true")
    [
        (status-row "precedence_required_surfaces" "env_precedence" "required_surfaces" (($missing | length) == 0) $precedence_path "Required precedence surfaces are present." $"Missing precedence surfaces: ($missing | str join ',').")
        (status-row "conflicts_no_blocking_errors" "conflicts" "no_blocking_errors" (($blocking_errors | length) == 0) $conflicts_path "No blocking error conflicts are present." $"Blocking error conflicts present: ($blocking_errors | length).")
    ]
}

def contains-any [text: string, patterns: list] {
    $patterns | any {|pattern| $text =~ $pattern}
}

def validate-secret-hygiene [tables_root: string, generated_dir: string] {
    let patterns = [
        "sk-[A-Za-z0-9]{20,}"
        "ghp_[A-Za-z0-9]{20,}"
        "github_pat_[A-Za-z0-9_]{20,}"
        "xox[baprs]-[A-Za-z0-9-]{20,}"
        "AKIA[0-9A-Z]{16}"
        "-----BEGIN (RSA |OPENSSH |EC |DSA |)?PRIVATE KEY-----"
    ]
    let secrets = (read-table $tables_root "secrets")
    let secret_columns = if ($secrets | length) == 0 { [] } else { $secrets | columns }
    let secrets_has_value = "value" in $secret_columns
    let files = [
        (table-path $tables_root "bootstrap_env_vars")
        (table-path $tables_root "secrets")
        (table-path $tables_root "env_precedence")
        (table-path $tables_root "conflicts")
        $"($generated_dir)/bootstrap.nu"
        $"($generated_dir)/bootstrap.sh"
    ]
    let leaks = (
        $files
        | where {|path| $path | path exists}
        | each {|path|
            let text = (open --raw $path)
            if (contains-any $text $patterns) { $path } else { null }
        }
        | compact
    )
    let nu_path = $"($generated_dir)/bootstrap.nu"
    let sh_path = $"($generated_dir)/bootstrap.sh"
    let nu_text = if ($nu_path | path exists) { open --raw $nu_path } else { "" }
    let sh_text = if ($sh_path | path exists) { open --raw $sh_path } else { "" }
    [
        (status-row "secrets_no_value_column" "secrets" "no_value_column" (not $secrets_has_value) (table-path $tables_root "secrets") "Secrets table has no value column." "Secrets table has a value column.")
        (status-row "raw_secret_pattern_scan" "all" "raw_secret_patterns" (($leaks | length) == 0) $tables_root "No raw secret-like values found in validated tables or generated bootstrap files." $"Raw secret-like values found in: ($leaks | str join ',').")
        (status-row "bootstrap_no_secret_refs" "bootstrap_generation_manifest" "bootstrap_no_secret_refs" (not ($nu_text =~ "secret:" or $sh_text =~ "secret:" or $nu_text =~ "OPENAI_API_KEY" or $sh_text =~ "OPENAI_API_KEY" or $nu_text =~ "GITHUB_TOKEN" or $sh_text =~ "GITHUB_TOKEN")) $generated_dir "Generated bootstrap files contain no secret refs or provider token names." "Generated bootstrap files contain secret refs or provider token names.")
    ]
}

def validate-generated-bootstrap [tables_root: string] {
    let manifest_path = (table-path $tables_root "bootstrap_generation_manifest")
    let manifest = (read-table $tables_root "bootstrap_generation_manifest")
    let required_artifacts = ["bootstrap_nu", "bootstrap_sh"]
    let artifact_ids = if ($manifest | length) == 0 { [] } else { $manifest | get artifact_id }
    let missing = ($required_artifacts | where {|artifact_id| $artifact_id not-in $artifact_ids})
    let rows = (
        $manifest
        | each {|row|
            let exists = ($row.path | path exists)
            let text = if $exists { open --raw $row.path } else { "" }
            let checksum_ok = if $exists { ((open --raw $row.path | hash sha256) == $row.output_checksum) } else { false }
            let header_ok = (
                $text =~ "generated by envctl"
                and $text =~ "source table: bootstrap_env_vars.csv"
                and $text =~ $row.source_table_checksum
                and $text =~ "do not edit directly; update envctl table instead"
            )
            [
                (status-row $"generated_bootstrap_($row.artifact_id)_exists" "bootstrap_generation_manifest" "generated_file_exists" $exists $row.path "Generated bootstrap file exists." "Generated bootstrap file is missing.")
                (status-row $"generated_bootstrap_($row.artifact_id)_checksum" "bootstrap_generation_manifest" "output_checksum" $checksum_ok $row.path "Generated bootstrap checksum matches manifest." "Generated bootstrap checksum does not match manifest.")
                (status-row $"generated_bootstrap_($row.artifact_id)_header" "bootstrap_generation_manifest" "provenance_header" $header_ok $row.path "Generated bootstrap provenance header is present." "Generated bootstrap provenance header is missing or incomplete.")
                (status-row $"generated_bootstrap_($row.artifact_id)_manual_gate" "bootstrap_generation_manifest" "manual_gate" ($row.manual_edits_allowed == "false" and $row.diff_required_before_apply == "true") $row.path "Generated bootstrap requires diff/manual gate before apply." "Generated bootstrap manifest does not require diff/manual gate.")
            ]
        }
        | flatten
    )
    [
        (status-row "bootstrap_manifest_required_artifacts" "bootstrap_generation_manifest" "required_artifacts" (($missing | length) == 0) $manifest_path "Bootstrap generation manifest includes required artifacts." $"Bootstrap generation manifest missing artifacts: ($missing | str join ',').")
    ] ++ $rows
}

def validate-ownership [tables_root: string] {
    let env_files_path = (table-path $tables_root "env_files")
    let guards_path = (table-path $tables_root "generated_file_guards")
    let env_files = (read-table $tables_root "env_files")
    let guards = (read-table $tables_root "generated_file_guards")
    let unowned = ($env_files | where owner == "" or owner == "unknown")
    let failed_guards = ($guards | where silent_overwrite_guard == "fail" or validation_status == "error")
    [
        (status-row "env_files_no_unowned_config" "env_files" "no_unowned_config_files" (($unowned | length) == 0) $env_files_path "No unowned config files are present in env_files." $"Unowned config file rows present: ($unowned | length).")
        (status-row "generated_guards_no_failed_overwrite" "generated_file_guards" "no_failed_overwrite_guards" (($failed_guards | length) == 0) $guards_path "Generated-file guards have no failed overwrite rows." $"Generated-file guard failures present: ($failed_guards | length).")
    ]
}

def validation-rows [
    tables_root: string = "/home/flexnetos/meta/var/lib/envctl/tables"
    generated_dir: string = "/home/flexnetos/meta/artifacts/generated/T036"
] {
    let required_rows = (validate-required-tables $tables_root)
    let bootstrap_rows = (validate-bootstrap-env $tables_root)
    let precedence_rows = (validate-precedence $tables_root)
    let secret_rows = (validate-secret-hygiene $tables_root $generated_dir)
    let generated_rows = (validate-generated-bootstrap $tables_root)
    let ownership_rows = (validate-ownership $tables_root)
    $required_rows ++ $bootstrap_rows ++ $precedence_rows ++ $secret_rows ++ $generated_rows ++ $ownership_rows
}

def main [
    --json
    --strict
    --tables-root: string = "/home/flexnetos/meta/var/lib/envctl/tables"
    --generated-dir: string = "/home/flexnetos/meta/artifacts/generated/T036"
] {
    let rows = (validation-rows $tables_root $generated_dir)
    let blocking_errors = ($rows | where status == "error" and blocking == "true")
    if $strict and (($blocking_errors | length) > 0) {
        exit 1
    }
    if $json {
        $rows | to json
    } else {
        $rows
    }
}
