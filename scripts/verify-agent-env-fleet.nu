#!/usr/bin/env nu

def command-result [envctl_bin: path, repo_path: path, action: string] {
    let config = ($repo_path | path join "agent-env.yaml")
    let result = if $action == "audit" {
        do {
            ^$envctl_bin agent audit --config $config --scope project --json --color never
        } | complete
    } else {
        do {
            ^$envctl_bin agent sync --config $config --scope project --json --color never
        } | complete
    }
    let parsed = try {
        $result.stdout | from json
    } catch {
        null
    }
    let action_statuses = if $parsed == null {
        []
    } else {
        $parsed.actions? | default [] | each {|item| $item.status? | default "unknown" }
    }
    let issue_kinds = if $parsed == null {
        []
    } else {
        $parsed.issues? | default [] | each {|item| $item.kind? | default "unknown" }
    }
    {
        status: (if $result.exit_code == 0 { "ok" } else { "failed" })
        exit_code: $result.exit_code
        failed_count: (if $parsed == null { null } else { $parsed.summary?.failed? | default null })
        action_statuses: $action_statuses
        issue_kinds: $issue_kinds
    }
}

def not-requested [] {
    {
        status: "not-requested"
        exit_code: null
        failed_count: null
        action_statuses: []
        issue_kinds: []
    }
}

def central-runtime-result [active_codex_root: path, codex_bin: path] {
    let catalog_path = ($active_codex_root | path join "model-catalog.json")
    let catalog_exists = ($catalog_path | path exists)
    let catalog = if $catalog_exists {
        try { open $catalog_path } catch { null }
    } else {
        null
    }
    let models = if $catalog == null {
        []
    } else {
        $catalog.models? | default []
    }
    let hy3 = ($models | where {|model| ($model.slug? | default "") == "tencent/hy3:free" } | get -o 0)
    let hy3_valid = if $hy3 == null {
        false
    } else {
        [
            (($hy3.visibility? | default "") == "list")
            (($hy3.provider? | default "") == "openrouter")
            (($hy3.context_window? | default 0) == 262144)
            (($hy3.free_route_expires_on? | default "") == "2026-07-21")
        ] | all {|condition| $condition }
    }
    let profiles = (
        ["envctl-openrouter.config.toml" "envctl-openrouter-gpt.config.toml"]
        | each {|name|
            let path = ($active_codex_root | path join $name)
            let exists = ($path | path exists)
            let profile = if $exists {
                try { open $path } catch { null }
            } else {
                null
            }
            let provider = if $profile == null {
                null
            } else {
                $profile | get -o model_providers.openrouter
            }
            let valid = ([
                ($profile != null)
                (($profile.model? | default "") == "tencent/hy3:free")
                (($profile.model_provider? | default "") == "openrouter")
                ($provider != null)
                (($provider.base_url? | default "") == "https://openrouter.ai/api/v1")
                (($provider.env_key? | default "") == "OPENROUTER_API_KEY")
                (($provider.wire_api? | default "") == "responses")
            ] | all {|condition| $condition })
            {
                name: $name
                path: ($path | into string)
                exists: $exists
                valid: $valid
            }
        }
    )
    let profiles_valid = ($profiles | all {|profile| $profile.valid })
    let codex_path = ($codex_bin | into string)
    let codex_exists = ($codex_bin | path exists)
    let profile_frontdoor = ($codex_path | str ends-with "/.nix-profile/bin/codex")
    let version_result = if $codex_exists {
        do { ^$codex_bin --version } | complete
    } else {
        {exit_code: 1, stdout: "", stderr: ""}
    }
    let version = ($version_result.stdout | str trim)
    let codex_valid = ([
        $codex_exists
        $profile_frontdoor
        ($version_result.exit_code == 0)
        ($version | str starts-with "codex-cli ")
    ] | all {|condition| $condition })
    let ok = $catalog_exists and $catalog != null and $hy3_valid and $profiles_valid and $codex_valid
    {
        status: (if $ok { "ok" } else { "failed" })
        active_codex_root: ($active_codex_root | into string)
        catalog: {
            path: ($catalog_path | into string)
            exists: $catalog_exists
            parsed: ($catalog != null)
            hy3_valid: $hy3_valid
        }
        profiles: $profiles
        codex: {
            path: $codex_path
            exists: $codex_exists
            profile_frontdoor: $profile_frontdoor
            version: $version
            valid: $codex_valid
        }
    }
}

def main [
    --meta-root: path = "."
    --project-list-json: path
    --envctl-bin: path
    --active-codex-root: path
    --codex-bin: path
    --execute-preview
    --execute-audit
    --json
] {
    let root = ($meta_root | path expand)
    let resolved_envctl = if $envctl_bin == null {
        $root | path join "usr/libexec/envctl/cli/bin/envctl"
    } else {
        $envctl_bin | path expand
    }
    let real_home = ($env.ENVCTL_REAL_HOME? | default $env.HOME | path expand --no-symlink)
    let resolved_active_codex = if $active_codex_root == null {
        $real_home | path join ".codex" | path expand --no-symlink
    } else {
        $active_codex_root | path expand --no-symlink
    }
    let resolved_codex = if $codex_bin == null {
        $real_home | path join ".nix-profile/bin/codex" | path expand --no-symlink
    } else {
        $codex_bin | path expand --no-symlink
    }
    let central_runtime = (central-runtime-result $resolved_active_codex $resolved_codex)
    let inventory = if $project_list_json != null {
        open $project_list_json
    } else {
        let result = (do { ^meta project list --json } | complete)
        if $result.exit_code != 0 {
            error make {msg: "meta project list --json failed"}
        }
        $result.stdout | from json
    }
    let declared = (
        $inventory.projects?
        | default []
        | each {|project|
            {
                name: ($project.name? | default $project.path)
                path: $project.path
                repo: ($project.repo? | default null)
            }
        }
    )
    let definitions = ([{
        name: "meta"
        path: "."
        repo: ($inventory.repo? | default null)
    }] | append $declared)
    let execution_requested = $execute_preview or $execute_audit
    let envctl_available = ($resolved_envctl | path exists)
    if $execution_requested and (not $envctl_available) {
        error make {msg: $"canonical envctl engine is unavailable: ($resolved_envctl)"}
    }
    let repos = (
        $definitions
        | each {|definition|
            let relative = ($definition.path | str trim --right --char "/")
            let normalized = if ($relative | is-empty) { "." } else { $relative }
            let repo_path = if $normalized == "." {
                $root
            } else {
                $root | path join $normalized
            }
            let exists = ($repo_path | path exists)
            let has_config = (($repo_path | path join "agent-env.yaml") | path exists)
            let has_lock = (($repo_path | path join "agent-env.lock") | path exists)
            let independent = $exists and $has_config and $has_lock
            let partial = $exists and ($has_config != $has_lock)
            let state = if not $exists {
                "missing"
            } else if $partial {
                "partial-agent-env"
            } else if $independent {
                "ready"
            } else {
                "central-runtime"
            }
            let preview = if $independent and $execute_preview {
                command-result $resolved_envctl $repo_path "preview"
            } else {
                not-requested
            }
            let audit = if $independent and $execute_audit {
                command-result $resolved_envctl $repo_path "audit"
            } else {
                not-requested
            }
            {
                name: $definition.name
                path: $normalized
                absolute_path: ($repo_path | into string)
                repo: $definition.repo
                exists: $exists
                ownership: (if $independent { "independent" } else { "central-inherited" })
                state: $state
                has_config: $has_config
                has_lock: $has_lock
                central_runtime_status: $central_runtime.status
                preview: $preview
                audit: $audit
            }
        }
    )
    let inventory_ok = ($repos | all {|repo|
        [
            $repo.exists
            ($repo.state != "partial-agent-env")
            ($repo.preview.status != "failed")
            ($repo.audit.status != "failed")
        ] | all {|condition| $condition }
    })
    let independent_repos = ($repos | where ownership == "independent")
    let sync_verified = ([
        ($central_runtime.status == "ok")
        $execute_preview
        $execute_audit
        ($independent_repos | all {|repo|
            $repo.preview.status == "ok" and $repo.audit.status == "ok"
        })
    ] | all {|condition| $condition })
    let report = {
        schema: "envctl-agent-env-fleet/1"
        meta_root: ($root | into string)
        execution_requested: $execution_requested
        verification_mode: (if $execute_preview and $execute_audit { "sync" } else { "inventory" })
        envctl_bin: ($resolved_envctl | into string)
        envctl_available: $envctl_available
        central_runtime: $central_runtime
        preview_requested: $execute_preview
        audit_requested: $execute_audit
        ok: ($inventory_ok and $central_runtime.status == "ok")
        sync_verified: $sync_verified
        independent_count: ($repos | where ownership == "independent" | length)
        central_inherited_count: ($repos | where ownership == "central-inherited" | length)
        repos: $repos
    }
    if $json {
        print ($report | to json --indent 2)
    } else {
        print ($report.repos | select name path ownership state central_runtime_status preview audit)
    }
    if not $report.ok {
        exit 1
    }
}
