#!/usr/bin/env nu

# Classify secret surfaces without reading or storing secret values.
# Outputs names, paths, classes, and handling policies only.

def class-from-file [row: record] {
    if $row.file_id == "codex_auth_json" {
        "codex_auth_store"
    } else if $row.domain == "runner" {
        "runner_injected_secret_refs"
    } else if $row.domain == "mcp" {
        "mcp_secret_refs"
    } else if $row.domain == "secrets" {
        "envctl_secretd_refs"
    } else if $row.domain == "database" {
        "database_secret_refs"
    } else if $row.domain == "codex" {
        "codex_sensitive_state"
    } else {
        "sensitive_reference"
    }
}

def read-policy [row: record] {
    if ($row.sensitivity == "secret" or $row.parse_method =~ "path_only") {
        "path_only_do_not_read"
    } else {
        "redacted_reference_only"
    }
}

def file-row [row: record] {
    {
        secret_id: $"file_($row.file_id)"
        surface: "file"
        owner: $row.owner
        domain: $row.domain
        source_path: $row.path
        source_kind: $row.format
        secret_name: ""
        secret_class: (class-from-file $row)
        value_policy: "no_value_stored"
        storage_policy: $row.state_classification
        read_policy: (read-policy $row)
        consumer: $row.runtime_role
        source_reference: "env_files.csv"
        classification_status: "classified"
        notes: $row.notes
    }
}

def name-row [
    secret_id: string
    owner: string
    domain: string
    source_path: string
    secret_name: string
    secret_class: string
    consumer: string
    source_reference: string
    status: string
    notes: string
] {
    {
        secret_id: $secret_id
        surface: "secret_name"
        owner: $owner
        domain: $domain
        source_path: $source_path
        source_kind: "name_only"
        secret_name: $secret_name
        secret_class: $secret_class
        value_policy: "no_value_stored"
        storage_policy: "envctl_secretd_or_external_secret_store"
        read_policy: "name_only"
        consumer: $consumer
        source_reference: $source_reference
        classification_status: $status
        notes: $notes
    }
}

def path-secret-row [
    secret_id: string
    owner: string
    domain: string
    source_path: string
    secret_name: string
    secret_class: string
    consumer: string
    source_reference: string
    status: string
    notes: string
] {
    {
        secret_id: $secret_id
        surface: "secret_path"
        owner: $owner
        domain: $domain
        source_path: $source_path
        source_kind: "path_only"
        secret_name: $secret_name
        secret_class: $secret_class
        value_policy: "no_value_stored"
        storage_policy: "external_secret_store"
        read_policy: "path_only_do_not_read"
        consumer: $consumer
        source_reference: $source_reference
        classification_status: $status
        notes: $notes
    }
}

def env-var-row [row: record] {
    {
        secret_id: $"env_var_($row.var_name)"
        surface: "env_var"
        owner: "envctl"
        domain: "env"
        source_path: $row.source_path
        source_kind: "parsed_env_fixture"
        secret_name: $row.var_name
        secret_class: "env_secret"
        value_policy: "redacted_in_parser_output"
        storage_policy: "fixture_only"
        read_policy: "redacted_reference_only"
        consumer: "parser_validation"
        source_reference: "env_vars_parsed.csv"
        classification_status: "classified"
        notes: "Sensitive env var name from parser fixture; value and raw line are redacted."
    }
}

def main [--json] {
    let tables_root = "/home/flexnetos/FlexNetOS/var/lib/envctl/tables"
    let sensitive_files = (
        open $"($tables_root)/env_files.csv"
        | where sensitivity =~ "secret|sensitive"
        | each {|row| file-row $row}
    )
    let sensitive_env_vars = (
        open $"($tables_root)/env_vars_parsed.csv"
        | where sensitive == true or sensitive == "true"
        | each {|row| env-var-row $row}
    )
    let explicit_names = [
        (name-row env_OPENAI_API_KEY openai ai_provider "env:OPENAI_API_KEY" OPENAI_API_KEY provider_api_token codex T033 expected "OpenAI API token names are classified; Codex ChatGPT auth remains path-only in auth.json.")
        (name-row env_ANTHROPIC_API_KEY anthropic ai_provider "env:ANTHROPIC_API_KEY" ANTHROPIC_API_KEY provider_api_token ai_cli T033 expected "Anthropic API token name for AI CLI and relay surfaces.")
        (name-row env_GH_TOKEN github github "env:GH_TOKEN" GH_TOKEN github_pat gh_cli T033 expected "GitHub CLI token env name; gh hosts file is separately classified path-only if present.")
        (name-row env_GITHUB_TOKEN github github "env:GITHUB_TOKEN" GITHUB_TOKEN github_actions_token runner T033 expected "GitHub Actions injected token name.")
        (path-secret-row gh_hosts_yml github github "/home/flexnetos/.config/gh/hosts.yml" GH_HOSTS_TOKEN github_cli_host_token gh_cli T033 not_found "No gh hosts.yml was found in this workspace scan; if created later it must remain path-only or redacted.")
        (path-secret-row nix_determinate_netrc nix nix "/nix/var/determinate/netrc" NIX_NETRC_TOKEN nix_access_token nix T033 path_only "Determinate Nix netrc exists and is classified path-only; contents were not read.")
        (name-row env_CARGO_REGISTRY_TOKEN cargo rust "env:CARGO_REGISTRY_TOKEN" CARGO_REGISTRY_TOKEN registry_publish_token cargo T033 expected "Crates.io publish token name.")
        (name-row env_NPM_TOKEN npm node "env:NPM_TOKEN" NPM_TOKEN npm_publish_token npm T033 expected "NPM publish token name.")
        (name-row env_NODE_AUTH_TOKEN npm node "env:NODE_AUTH_TOKEN" NODE_AUTH_TOKEN npm_auth_token npm T033 expected "Node/npm auth token name.")
        (name-row env_HOMEBREW_TAP_TOKEN homebrew release "env:HOMEBREW_TAP_TOKEN" HOMEBREW_TAP_TOKEN homebrew_tap_token release T033 expected "Homebrew tap token name from release docs.")
        (name-row env_ORG_REPO_BOOTSTRAP_TOKEN github release "env:ORG_REPO_BOOTSTRAP_TOKEN" ORG_REPO_BOOTSTRAP_TOKEN org_bootstrap_token release T033 expected "One-time GitHub org/repo bootstrap token name.")
        (name-row env_CORE_REPO_PAT github release "env:CORE_REPO_PAT" CORE_REPO_PAT source_release_token release T033 expected "Core source-release token name.")
        (name-row env_SECRETD_TOKEN envctl secrets "env:SECRETD_TOKEN" SECRETD_TOKEN secretd_auth_token secretd T033 expected "secretd auth token name; value belongs only in envctl/secretd vault or a protected runtime injection path.")
        (name-row env_ENVCTL_SEED_TOKEN envctl secrets "env:ENVCTL_SEED_TOKEN" ENVCTL_SEED_TOKEN envctl_seed_token envctl T033 expected "envctl seed token name.")
        (name-row env_ENVCTL_SEED_TOKEN_FILE envctl secrets "env:ENVCTL_SEED_TOKEN_FILE" ENVCTL_SEED_TOKEN_FILE secret_file_ref envctl T033 expected "envctl seed token file reference; path only.")
        (name-row env_LIBSQL_AUTH_TOKEN libsql database "env:LIBSQL_AUTH_TOKEN" LIBSQL_AUTH_TOKEN database_auth_token secretd T033 expected "libSQL/sqld token name; sqld loopback manifest currently uses open local auth but token class is tracked.")
        (name-row env_KACHE_REMOTE_TOKEN kache cache "env:KACHE_REMOTE_TOKEN" KACHE_REMOTE_TOKEN cache_remote_token kache T033 expected "Kache remote cache token name if remote backend is configured later.")
        (name-row env_AWS_ACCESS_KEY_ID aws cache "env:AWS_ACCESS_KEY_ID" AWS_ACCESS_KEY_ID cloud_access_key kache T033 expected "AWS access key name for Kache or other S3-compatible cache backends.")
        (name-row env_AWS_SECRET_ACCESS_KEY aws cache "env:AWS_SECRET_ACCESS_KEY" AWS_SECRET_ACCESS_KEY cloud_secret_key kache T033 expected "AWS secret key name for Kache or other S3-compatible cache backends.")
        (name-row env_GITHUB_APP_PRIVATE_KEY github secrets "env:GITHUB_APP_PRIVATE_KEY" GITHUB_APP_PRIVATE_KEY github_app_private_key envctl_secretd T033 expected "GitHub App private key class; broker-only and never stored in generated artifacts.")
        (name-row env_GITHUB_APP_INSTALLATION_TOKEN github secrets "env:GITHUB_APP_INSTALLATION_TOKEN" GITHUB_APP_INSTALLATION_TOKEN github_app_installation_token envctl_secretd T033 expected "Short-lived GitHub App installation token class.")
    ]

    let rows = ($sensitive_files ++ $sensitive_env_vars ++ $explicit_names)
    if $json {
        $rows | to json
    } else {
        $rows
    }
}
