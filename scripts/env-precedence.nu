#!/usr/bin/env nu

# Resolve environment/config precedence into explicit tables.
# The script reads only existing envctl tables and emits policy rows plus
# deterministic conflict records; it never reads credential stores.

def precedence-row [
    precedence_id: string
    surface: string
    owner: string
    domain: string
    setting_name: string
    source_path: string
    source_table: string
    source_row_ref: string
    precedence_rank: int
    precedence_tier: string
    value_policy: string
    selected_policy: string
    conflict_policy: string
    manual_gate: string
    source_reference: string
    notes: string
] {
    {
        precedence_id: $precedence_id
        surface: $surface
        owner: $owner
        domain: $domain
        setting_name: $setting_name
        source_path: $source_path
        source_table: $source_table
        source_row_ref: $source_row_ref
        precedence_rank: $precedence_rank
        precedence_tier: $precedence_tier
        value_policy: $value_policy
        selected_policy: $selected_policy
        conflict_policy: $conflict_policy
        manual_gate: $manual_gate
        source_reference: $source_reference
        notes: $notes
    }
}

def conflict-row [
    conflict_id: string
    surface: string
    owner: string
    domain: string
    setting_name: string
    conflict_kind: string
    severity: string
    blocking: string
    status: string
    source_refs: string
    selected_source: string
    resolution_policy: string
    manual_gate: string
    source_reference: string
    notes: string
] {
    {
        conflict_id: $conflict_id
        surface: $surface
        owner: $owner
        domain: $domain
        setting_name: $setting_name
        conflict_kind: $conflict_kind
        severity: $severity
        blocking: $blocking
        status: $status
        source_refs: $source_refs
        selected_source: $selected_source
        resolution_policy: $resolution_policy
        manual_gate: $manual_gate
        source_reference: $source_reference
        notes: $notes
    }
}

def precedence-rows [] {
    [
        (precedence-row "generic_process_env" "env" "envctl" "env" "*" "process:environment" "runtime_env" "*" 10 "runtime_env" "redacted_or_name_only_for_sensitive_values" "Process environment wins over explicit config and envctl table defaults unless a tool-specific exception narrows it." "Duplicate names and incompatible source classes emit conflicts.csv rows." "no" "T034" "Base precedence model from T034.")
        (precedence-row "generic_explicit_config" "config" "envctl" "env" "*" "explicit:config_file" "env_files.csv" "*" 20 "explicit_config" "redacted_or_path_only_for_sensitive_refs" "Explicit config files win over envctl table defaults when the tool documents a file layer." "Multiple explicit configs for one setting emit conflicts.csv rows unless a documented merge strategy exists." "review_if_multiple" "T034" "Second tier for user or repo config.")
        (precedence-row "generic_envctl_defaults" "table" "envctl" "env" "*" "/home/flexnetos/FlexNetOS/var/lib/envctl/tables" "envctl_tables" "*" 40 "table_defaults" "no_value_stored_for_secrets" "Envctl canonical/default table rows fill only absent runtime and explicit config settings." "Generated defaults must not silently override hand-authored files." "review_before_generation" "T034" "Default tier for table-first generation.")
        (precedence-row "generic_generated_files" "generated_file" "envctl" "env" "*" "generated:declared_targets" "bootstrap_generated_files.csv" "*" 50 "generated_output" "rendered_from_validated_tables" "Generated files are replaceable outputs and never source of truth." "Generated output must include provenance and diff before apply." "required" "T034" "Output tier from envctl table doctrine.")

        (precedence-row "codex_auth_store" "codex" "codex" "codex" "auth.json" "/run/user/1001/yazelix/profile-runtime/codex/auth.json" "secrets.csv" "file_codex_auth_json" 15 "credential_store_exception" "path_only_do_not_read" "Codex ChatGPT auth is path-only and is not parsed into envctl value tables." "If API-token env names are also present, report auth-mode ambiguity instead of merging values." "required" "T033,T034" "Credential-store exception for Codex.")
        (precedence-row "codex_provider_api_env" "codex" "openai" "ai_provider" "OPENAI_API_KEY" "env:OPENAI_API_KEY" "secrets.csv" "env_OPENAI_API_KEY" 10 "runtime_env" "name_only_no_value_stored" "OPENAI_API_KEY is a provider API token name only when an API-mode workflow explicitly opts in." "Does not supersede Codex ChatGPT auth.json; mode switch requires manual gate." "required_before_api_mode" "T033,T034" "Provider-token exception for Codex API work.")
        (precedence-row "codex_global_config" "codex" "codex" "codex" "codex_config" "/run/user/1001/yazelix/profile-runtime/codex/config.toml" "env_files.csv" "codex_config" 20 "explicit_config" "redacted_reference_only" "Global Codex config is an explicit config layer after runtime env and credential-store policy." "Project config conflicts must be reported rather than silently merged." "review" "T028,T034" "Codex global config precedence.")
        (precedence-row "codex_project_config" "codex" "envctl_codex" "codex" "project_codex_config" "/home/flexnetos/FlexNetOS/src/envctl/.codex/config.toml" "env_files.csv" "envctl_codex_project_config" 22 "project_config" "redacted_reference_only" "Project Codex config scopes repo-local behavior and must not mutate global auth state." "Global/project setting collisions require inspectable rows." "review" "T028,T034" "Codex project config precedence.")
        (precedence-row "codex_state_table" "codex" "envctl" "codex" "state_locations" "/home/flexnetos/FlexNetOS/var/lib/envctl/tables/codex_state_locations.csv" "env_files.csv" "envctl_table_codex_state" 40 "table_defaults" "path_only_or_redacted_reference" "Codex state locations are envctl table references, not raw state content." "Generated Codex fragments must not copy state database contents." "required" "T028,T034" "Codex state table precedence.")

        (precedence-row "secretd_config_env" "envctl_secretd" "envctl_secrets" "secrets" "SECRETD_CONFIG" "env:SECRETD_CONFIG" "docs/secrets/ops/08-secretd-store-config.md" "section_1_2" 10 "runtime_env" "path_only" "SECRETD_CONFIG overrides the default TOML path." "Missing or unreadable configured TOML path must become a validation/conflict row." "review" "T034" "secretd config path override.")
        (precedence-row "secretd_token_file_env" "envctl_secretd" "envctl" "secrets" "SECRETD_LIBSQL_AUTH_TOKEN_FILE" "env:SECRETD_LIBSQL_AUTH_TOKEN_FILE" "docs/secrets/ops/08-secretd-store-config.md" "section_1_3" 10 "runtime_env_secret_file" "name_only_path_only_no_value_stored" "Token-file env wins over inline token for libSQL auth when both are configured." "If inline and token-file are both present, emit a warning conflict and select token file." "required" "T034" "Token-file path is preferred; contents are never read by this script.")
        (precedence-row "secretd_token_env" "envctl_secretd" "envctl" "secrets" "SECRETD_LIBSQL_AUTH_TOKEN" "env:SECRETD_LIBSQL_AUTH_TOKEN" "docs/secrets/ops/08-secretd-store-config.md" "section_1_2" 11 "runtime_env_secret" "name_only_no_value_stored" "Inline secretd token is accepted only as protected runtime injection." "Raw token values must never enter tables/logs; token-file conflicts select the file path." "required" "T033,T034" "Inline token class tracked without value storage.")
        (precedence-row "secretd_store_backend_env" "envctl_secretd" "envctl" "secrets" "SECRETD_STORE_BACKEND" "env:SECRETD_STORE_BACKEND" "docs/secrets/ops/08-secretd-store-config.md" "section_1_2" 10 "runtime_env" "non_secret_value" "Environment backend setting overrides TOML and defaults." "Invalid backend emits validation/conflict row." "review" "T034" "Documented highest-precedence secretd backend setting.")
        (precedence-row "secretd_toml" "envctl_secretd" "envctl_secrets" "secrets" "secretd.toml" "$META_ROOT/.config/env-ctl/secretd.toml" "docs/secrets/ops/08-secretd-store-config.md" "section_1_1" 20 "explicit_config" "no_token_in_file" "TOML provides backend and URL only when runtime env does not override." "Token material in TOML is invalid and must be reported." "required" "T034" "secretd file layer from ops docs.")
        (precedence-row "secretd_defaults" "envctl_secretd" "envctl_secrets" "secrets" "store.defaults" "default:inmem" "docs/secrets/ops/08-secretd-store-config.md" "section_1" 40 "tool_defaults" "no_secret_value" "Defaults apply only when env and TOML are absent." "Defaults must be explicit table rows before generation." "no" "T034" "secretd in-memory default.")

        (precedence-row "database_libsql_url_env" "database" "libsql" "database" "SECRETD_LIBSQL_URL" "env:SECRETD_LIBSQL_URL" "docs/secrets/ops/08-secretd-store-config.md" "section_1_2" 10 "runtime_env" "non_secret_loopback_url_only" "Runtime URL overrides TOML and must resolve to loopback unless a terminator policy row is added." "Non-loopback plaintext URLs are refused and reported." "required" "T034" "DB URL precedence for secretd libSQL.")
        (precedence-row "database_libsql_token_env" "database" "libsql" "database" "LIBSQL_AUTH_TOKEN" "env:LIBSQL_AUTH_TOKEN" "secrets.csv" "env_LIBSQL_AUTH_TOKEN" 10 "runtime_env_secret" "name_only_no_value_stored" "Database token names are tracked as secret references only." "sqld loopback open-auth is allowed only when explicitly documented as local." "required" "T033,T034" "DB token classification from T033.")
        (precedence-row "database_sqld_manifest" "database" "envctl" "database" "sqld" "/home/flexnetos/FlexNetOS/src/envctl/manifest/sqld.toml" "env_files.csv" "envctl_manifest_sqld" 30 "repo_manifest" "redacted_reference_only" "sqld manifest is a repo source for database component generation after runtime env and explicit config." "Sensitive refs in manifest remain references only." "review" "T028,T034" "Database manifest precedence.")
        (precedence-row "database_loopback_default" "database" "envctl_secrets" "database" "sqld_loopback_auth" "docs/secrets/ops/08-secretd-store-config.md" "policy_doc" "section_2" 40 "tool_defaults" "no_secret_value" "Loopback sqld may run without token only for local open-auth development or first-run profile." "Any non-loopback DB requires protected token or terminator policy." "required" "T034" "Database open-auth exception.")

        (precedence-row "kache_remote_token_env" "kache" "kache" "cache" "KACHE_REMOTE_TOKEN" "env:KACHE_REMOTE_TOKEN" "secrets.csv" "env_KACHE_REMOTE_TOKEN" 10 "runtime_env_secret" "name_only_no_value_stored" "Remote-cache token is runtime injection only and wins over cache config placeholders." "Raw cache credentials must never be written to Cargo config." "required" "T033,T034" "Kache remote-token precedence.")
        (precedence-row "kache_aws_access_env" "kache" "aws" "cache" "AWS_ACCESS_KEY_ID" "env:AWS_ACCESS_KEY_ID" "secrets.csv" "env_AWS_ACCESS_KEY_ID" 10 "runtime_env_secret" "name_only_no_value_stored" "Cloud cache access-key name is runtime injection only." "Pairing with secret key must be treated as a secret bundle." "required" "T033,T034" "Kache cloud access-key precedence.")
        (precedence-row "kache_aws_secret_env" "kache" "aws" "cache" "AWS_SECRET_ACCESS_KEY" "env:AWS_SECRET_ACCESS_KEY" "secrets.csv" "env_AWS_SECRET_ACCESS_KEY" 10 "runtime_env_secret" "name_only_no_value_stored" "Cloud cache secret-key name is runtime injection only." "Pairing with access key must be treated as a secret bundle." "required" "T033,T034" "Kache cloud secret-key precedence.")
        (precedence-row "kache_cargo_config" "kache" "kache" "cache" "rustc-wrapper" "/home/flexnetos/.cargo/config.toml" "tool_versions.csv" "kache" 20 "explicit_config" "public_config_ref" "Cargo rustc-wrapper config selects Kache after runtime secret injection policy." "Generated Cargo config must diff and preserve manual review." "review" "T032,T034" "Kache active cargo config.")
        (precedence-row "kache_upstream_toolchain" "kache" "kache" "cache" "upstream_toolchain" "/home/flexnetos/FlexNetOS/src/upstream/kunobi-ninja/kache/rust-toolchain.toml" "tool_versions.csv" "kache_rust_toolchain" 30 "repo_config" "public_config_ref" "Upstream Kache build toolchain applies only inside the upstream checkout." "Does not override envctl/runner repo toolchains." "no" "T032,T034" "Kache upstream config boundary.")

        (precedence-row "rtk_config_file" "rtk" "rtk" "rtk" "rtk_config" "/home/flexnetos/.config/rtk/config.toml" "tool_versions.csv" "rtk" 20 "explicit_config" "redacted_reference_only" "RTK config file owns telemetry and failure-tee behavior after runtime env." "RTK summaries never replace raw failure logs for gate evidence." "review" "T032,T034" "RTK config precedence.")
        (precedence-row "rtk_raw_logs" "rtk" "rtk" "rtk" "raw_failure_logs" "/home/flexnetos/FlexNetOS/var/log/raw" "release-baseline.json" "log_roots" 15 "evidence_log" "raw_log_reference_only" "Raw logs are preserved as evidence before any RTK summary is used." "Compressed summaries cannot be the only failure artifact." "required" "T001,T034" "RTK exception for raw failure evidence.")
        (precedence-row "rtk_table_defaults" "rtk" "envctl" "rtk" "rtk_integration_config" "/home/flexnetos/FlexNetOS/var/lib/envctl/tables/rtk_integration_config.csv" "env_files.csv" "envctl_table_rtk_config" 40 "table_defaults" "public_config_ref" "Envctl RTK integration rows fill only absent config values." "Direct config edits must be backported to tables before durable generation." "review_before_generation" "T028,T034" "RTK table default layer.")

        (precedence-row "nix_custom_conf" "nix" "nix" "nix" "nix.custom.conf" "/etc/nix/nix.custom.conf" "tool_versions.csv" "yazelix_cachix_cache" 20 "explicit_config" "public_config_ref" "User/operator Nix changes belong in nix.custom.conf and win over generated Determinate base for local additions." "Conflicts with generated nix.conf must be reported; do not edit Determinate file directly." "required" "T032,T034" "Nix editable layer.")
        (precedence-row "nix_determinate_conf" "nix" "nix" "nix" "nix.conf" "/etc/nix/nix.conf" "tool_versions.csv" "nix_determinate" 30 "system_generated" "public_config_ref" "Determinate-generated nix.conf is imported evidence, not the editable source for user policy." "User modifications are redirected to nix.custom.conf." "required" "T032,T034" "Determinate Nix exception.")
        (precedence-row "nix_profile_tools" "nix" "nix" "nix" "nix_profile" "command:nix profile list --json" "tool_versions.csv" "nushell,yazelix" 25 "profile_state" "public_config_ref" "Nix profile state explains installed user tools after config but before table defaults." "Missing PATH exposure emits validation/conflict rows rather than assuming availability." "review" "T032,T034" "Nix profile tool layer.")
        (precedence-row "nix_netrc_secret_path" "nix" "nix" "nix" "NIX_NETRC_TOKEN" "/nix/var/determinate/netrc" "secrets.csv" "nix_determinate_netrc" 10 "external_secret_store" "path_only_do_not_read" "Determinate netrc is external credential state and must remain path-only." "Never parse or copy contents into envctl tables." "required" "T033,T034" "Nix access-token path-only exception.")

        (precedence-row "yazelix_settings_file" "yazelix" "yazelix" "yazelix" "settings.jsonc" "/home/flexnetos/.config/yazelix/settings.jsonc" "tool_versions.csv" "yazelix_settings" 20 "explicit_config" "public_config_ref" "User Yazelix settings are reviewed editable input; installed runtime configuration belongs to the profile." "Envctl overlays must not overwrite without diff/manual review." "review" "T032,T034" "Yazelix runtime settings input.")
        (precedence-row "yazelix_shell_nu_user" "yazelix" "yazelix" "yazelix" "shell_nu.nu" "/home/flexnetos/.config/yazelix/shell_nu.nu" "nushell_env_config_parsed.csv" "file_summary:/home/flexnetos/.config/yazelix/shell_nu.nu" 20 "explicit_config" "public_or_redacted_rows" "User Yazelix shell fragment is imported even when it has no active rows." "Generated fragments must preserve empty/comment-only user files." "review" "T031,T034" "Yazelix user shell fragment.")
        (precedence-row "yazelix_envctl_template" "yazelix" "envctl" "yazelix" "nix-yazelix manifest" "/home/flexnetos/meta/src/envctl/manifest/nix-yazelix.toml" "env_files.csv" "envctl_manifest_nix_yazelix" 30 "repo_manifest" "public_config_ref" "Envctl Yazelix manifest records validation intent; only merged Yazelix origin/main owns installation." "Generated projections require explicit diff before apply." "review" "T028,T034" "Yazelix manifest layer.")
        (precedence-row "yazelix_nushell_path_template" "yazelix" "envctl" "shell" "PATH" "/home/flexnetos/meta/src/envctl/home/.config/nushell/profile-path.nu" "nushell_env_config_parsed.csv" "env_assignment:PATH" 30 "repo_template" "public_config_ref" "Envctl Nushell validation input replaces inherited PATH with the current profile toolbin/bin pair." "Any other command owner is a fail-closed conflict." "required" "T031,T034" "Strict Nushell/Yazelix profile PATH layer.")

        (precedence-row "runner_dispatch_key_env" "runner_fxrun" "flexnetos_runner" "runner" "FXRUN_DISPATCH_KEY" "env:FXRUN_DISPATCH_KEY" "runner-dispatch/src/main.rs" "serve_requires_FXRUN_DISPATCH_KEY" 10 "runtime_env_secret" "name_only_no_value_stored" "Runner dispatch key is injected from envctl vault and required to serve." "Missing key in serve mode is fail-closed; raw values are redacted." "required" "T033,T034" "Runner frame-auth secret precedence.")
        (precedence-row "runner_injected_secrets_env" "runner_fxrun" "flexnetos_runner" "runner" "FXRUN_INJECT_SECRETS" "env:FXRUN_INJECT_SECRETS" "runner-dispatch/src/main.rs" "resolve_injected_secrets" 10 "runtime_env_secret_refs" "name_list_only_no_values" "Runner relays only named secrets from its environment to kernel children and registers values with redactor." "Injected values never land in audit logs or tables." "required" "T034" "Runner secret relay precedence.")
        (precedence-row "runner_governor_env" "runner_fxrun" "flexnetos_runner" "runner" "FXRUN_*_POLICY" "env:FXRUN_LOOP_WINDOW,FXRUN_DISPATCH_BUDGET,FXRUN_RATE_MAX,FXRUN_SCAN_BLOCK_SEVERITY" "runner-dispatch/src/main.rs" "runtime_policy_env" 10 "runtime_env" "non_secret_policy_values" "Runtime FXRUN policy envs override default runner admission behavior." "Invalid policy env values fail closed or become validation rows." "review" "T034" "Runner guardrail env precedence.")
        (precedence-row "runner_kernel_cmd_env" "runner_fxrun" "flexnetos_runner" "runner" "FXRUN_KERNEL_CMD_*" "env:FXRUN_KERNEL_CMD_LOOP,FXRUN_KERNEL_CMD_ATC,FXRUN_KERNEL_CMD_HF,FXRUN_KERNEL_CMD_WEAVE" "runner-dispatch/src/main.rs" "kernel_command_env" 10 "runtime_env" "path_reference_only" "Kernel command envs enable real kernel execution and override dry-run invoker defaults." "Command paths require allowlist/provenance checks before production generation." "required" "T034" "Runner kernel command precedence.")
        (precedence-row "runner_event_log_env" "runner_fxrun" "flexnetos_runner" "runner" "FXRUN_EVENT_LOG" "env:FXRUN_EVENT_LOG,FXRUN_POLICY_LOG" "runner-dispatch/src/main.rs" "event_log_env" 10 "runtime_env" "path_reference_only" "Runner audit-log env paths override null sink and must point at approved log roots." "Secrets in log details are redacted before write." "required" "T034" "Runner log path precedence.")
        (precedence-row "runner_systemd_env" "runner_fxrun" "flexnetos_runner" "runner" "systemd_runner_env" "/etc/systemd/system/actions.runner.*.service*" "env_files.csv" "systemd_runner_*" 20 "service_env_config" "redacted_reference_only" "Systemd runner service/drop-in env is explicit config after runtime process env and before envctl defaults." "Sensitive refs require redacted parse and manual review." "required" "T028,T034" "Runner service env layer.")
        (precedence-row "runner_workspace_manifest" "runner_fxrun" "flexnetos_runner" "runner" "runner_workspace" "/home/flexnetos/FlexNetOS/src/flexnetos_runner/Cargo.toml" "tool_versions.csv" "flexnetos_runner_workspace" 30 "repo_config" "public_config_ref" "Runner workspace manifest provides build/release version data, not runtime secret policy." "Runtime FXRUN envs remain authoritative for serve-time behavior." "review" "T032,T034" "Runner workspace config layer.")

        (precedence-row "wild_opt_in_cargo_config" "rust" "wild" "rust" "wild_linker" "/home/flexnetos/FlexNetOS/etc/rust/wild/cargo-wild-x86_64-unknown-linux-gnu.toml" "tool_versions.csv" "wild" 30 "workspace_config" "public_config_ref" "Wild linker config is opt-in and must not override default Cargo builds silently." "Opt-in profile must be explicit in generated build env." "review" "T032,T034" "Wild linker boundary used by Rust build env generation.")
        (precedence-row "rust_repo_toolchains" "rust" "envctl" "rust" "rust-toolchain.toml" "/home/flexnetos/FlexNetOS/src/*/rust-toolchain.toml" "tool_versions.csv" "envctl_rust_toolchain,runner_rust_toolchain,meta_rust_toolchain" 30 "repo_config" "public_config_ref" "Repo toolchains beat ambient cargo/rustc PATH observations for builds in that repo." "Ambient toolchain absence emits conflict/warning rows until login/bootstrap validates PATH." "review" "T032,T034" "Rust toolchain precedence for later build tasks.")
    ]
}

def duplicate-env-conflicts [env_rows: list] {
    $env_rows
    | where parse_status == "ok"
    | group-by var_name
    | transpose setting_name rows
    | where {|group| ($group.rows | length) > 1}
    | each {|group|
        let sorted = ($group.rows | sort-by source_path line_no end_line_no duplicate_index)
        let refs = (
            $sorted
            | each {|row| $"($row.source_path):($row.line_no)-($row.end_line_no)#duplicate_index=($row.duplicate_index)"}
            | str join ";"
        )
        conflict-row $"env_duplicate_($group.setting_name)" "env" "envctl" "env" $group.setting_name "duplicate_env_var" "warning" "false" "fixture_detected" $refs "" "Manual resolution is required before this duplicate can be promoted into canonical production env; T030 fixture evidence remains non-blocking." "required_for_production" "env_vars_parsed.csv" "Duplicate env var parser evidence proves deterministic conflict reporting without silent last-writer-wins."
    }
}

def path-merge-conflicts [nu_rows: list] {
    $nu_rows
    | where row_kind == "env_assignment"
    | where key in ["PATH" "LD_LIBRARY_PATH"]
    | sort-by key source_path line_no
    | each {|row|
        conflict-row $"path_merge_($row.key)" "path" $row.owner "shell" $row.key "path_order_merge" "info" "false" "inspectable" $"($row.source_path):($row.line_no)#($row.precedence_scope)" "existing process env plus envctl Nushell merge expression" "Preserve prior env entries, apply declared prepend/append/uniq behavior, and validate before generation." "review" "nushell_env_config_parsed.csv" "PATH-like mutation has explicit source and merge policy."
    }
}

def curated-conflicts [] {
    [
        (conflict-row "codex_auth_mode_ambiguity" "codex" "codex" "codex" "auth_mode" "credential_store_vs_provider_env" "warning" "false" "policy_guard" "secrets.csv:file_codex_auth_json;secrets.csv:env_OPENAI_API_KEY" "auth.json for ChatGPT login; OPENAI_API_KEY only for explicit API mode" "Manual gate required before switching Codex auth mode or generating provider-token env." "required" "T033,T034" "Prevents auth.json and provider API key names from being silently conflated.")
        (conflict-row "secretd_token_file_vs_inline" "envctl_secretd" "envctl" "secrets" "SECRETD_LIBSQL_AUTH_TOKEN" "secret_file_preferred_over_inline_env" "warning" "false" "policy_guard" "env:SECRETD_LIBSQL_AUTH_TOKEN_FILE;env:SECRETD_LIBSQL_AUTH_TOKEN" "SECRETD_LIBSQL_AUTH_TOKEN_FILE" "Select token file when both references exist; never store either value." "required" "T034" "Documented token hygiene preference for secretd.")
        (conflict-row "nix_generated_vs_custom_layer" "nix" "nix" "nix" "nix_config" "generated_base_with_custom_overlay" "info" "false" "policy_guard" "tool_versions.csv:nix_determinate;tool_versions.csv:yazelix_cachix_cache" "/etc/nix/nix.custom.conf for user additions" "Do not hand-edit Determinate-generated nix.conf; add reviewed policy to nix.custom.conf." "required" "T032,T034" "Keeps generated system config and user overlay separate.")
        (conflict-row "runner_sensitive_env_refs" "runner_fxrun" "flexnetos_runner" "runner" "FXRUN_*" "runtime_secret_refs_in_service_env" "warning" "false" "policy_guard" "env_files.csv:systemd_runner_*;runner-dispatch/src/main.rs:FXRUN_DISPATCH_KEY" "envctl vault or protected service env refs only" "Parse service env as redacted references and require manual review before generated runner env." "required" "T033,T034" "Runner services may carry sensitive refs and must stay value-free in tables.")
        (conflict-row "ambient_rust_toolchain_missing" "rust" "envctl" "rust" "cargo_rustc" "ambient_path_missing_but_repo_toolchains_present" "warning" "false" "observed" "tool_versions.csv:cargo_ambient;tool_versions.csv:rustc_ambient;tool_versions.csv:envctl_rust_toolchain" "repo rust-toolchain plus Nix/profile bootstrap" "Login/bootstrap generation must validate cargo and rustc before build tasks rely on them." "review" "T032,T034" "Prevents assuming ambient cargo/rustc are available in non-login commands.")
    ]
}

def conflict-rows [] {
    let tables_root = "/home/flexnetos/FlexNetOS/var/lib/envctl/tables"
    let env_rows = (open $"($tables_root)/env_vars_parsed.csv")
    let nu_rows = (open $"($tables_root)/nushell_env_config_parsed.csv")
    (duplicate-env-conflicts $env_rows) ++ (path-merge-conflicts $nu_rows) ++ (curated-conflicts)
}

def main [
    table: string = "precedence"
    --json
] {
    let rows = if $table == "precedence" {
        precedence-rows
    } else if $table == "conflicts" {
        conflict-rows
    } else {
        error make {msg: $"unsupported table '($table)'; expected precedence or conflicts"}
    }

    if $json {
        $rows | to json
    } else {
        $rows
    }
}
