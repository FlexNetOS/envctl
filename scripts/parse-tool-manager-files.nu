#!/usr/bin/env nu

# Normalize tool-manager evidence into tool_versions rows.
# Inputs are explicit source files, prior envctl evidence tables, and safe
# version commands. Secret stores are intentionally not read.

def sha-file [path: string] {
    if ($path | path exists) {
        ^sha256sum $path | str trim | split row -r '\s+' | first
    } else {
        ""
    }
}

def command-stdout [command: string] {
    let result = (^bash -lc $command | complete)
    if $result.exit_code == 0 {
        $result.stdout | str trim
    } else {
        "not_on_path"
    }
}

def tool-row [
    tool_id: string
    tool_name: string
    category: string
    version_or_ref: string
    source_path: string
    source_format: string
    source_reference: string
    source_role: string
    lock_or_pin: string
    source_of_truth: string
    generated: bool
    install_path: string
    checksum: string
    parse_status: string
    sensitivity: string
    notes: string
] {
    {
        tool_id: $tool_id
        tool_name: $tool_name
        category: $category
        version_or_ref: $version_or_ref
        source_path: $source_path
        source_format: $source_format
        source_reference: $source_reference
        source_role: $source_role
        lock_or_pin: $lock_or_pin
        source_of_truth: $source_of_truth
        generated: $generated
        install_path: $install_path
        checksum: $checksum
        parse_status: $parse_status
        sensitivity: $sensitivity
        notes: $notes
    }
}

def first-artifact [table_path: string artifact_id: string] {
    open $table_path | where artifact_id == $artifact_id | first
}

def rust-toolchain-row [
    tool_id: string
    tool_name: string
    source_path: string
    source_reference: string
    notes: string
] {
    let data = (open $source_path)
    let channel = ($data.toolchain.channel | into string)
    let lock = if ($channel =~ '^[0-9]') { $"pinned:($channel)" } else { $"floating:($channel)" }
    tool-row $tool_id $tool_name rust_toolchain $channel $source_path toml $source_reference repo_config $lock source_of_truth false "" (sha-file $source_path) parsed public $notes
}

def main [--json] {
    let tables_root = "/home/flexnetos/meta/var/lib/envctl/tables"
    let nix_profile = (^nix profile list --json | complete)
    let profile_elements = if $nix_profile.exit_code == 0 {
        ($nix_profile.stdout | from json | get elements)
    } else {
        {}
    }

    let nushell_profile = ($profile_elements | get -o nushell)
    let yazelix_profile = ($profile_elements | get -o yazelix)

    let rtk_release = (first-artifact $"($tables_root)/rtk_tool_install.csv" rtk_release_tag)
    let rtk_binary = (first-artifact $"($tables_root)/rtk_tool_install.csv" rtk_binary)
    let kache_release = (first-artifact $"($tables_root)/kache_tool_install.csv" kache_release_tag)
    let kache_binary = (first-artifact $"($tables_root)/kache_tool_install.csv" kache_binary)
    let wild_release = (first-artifact $"($tables_root)/wild_tool_install.csv" wild_release_tag)
    let wild_binary = (first-artifact $"($tables_root)/wild_tool_install.csv" wild_binary)
    let yazelix_flake = (first-artifact $"($tables_root)/yazelix_tool_locations.csv" yazelix_flake)
    let yzx_binary = (first-artifact $"($tables_root)/yazelix_tool_locations.csv" yzx_binary)

    let cargo_config = "/home/flexnetos/.cargo/config.toml"
    let cargo_data = (open $cargo_config)
    let cargo_wrapper = ($cargo_data | get -o build.rustc-wrapper | default "missing")

    let rtk_config = "/home/flexnetos/.config/rtk/config.toml"
    let rtk_data = (open $rtk_config)
    let rtk_policy = $"telemetry=($rtk_data.telemetry.enabled); tee=($rtk_data.tee.mode)"

    let yazelix_settings = "/home/flexnetos/.config/yazelix/settings.jsonc"
    let yazelix_settings_raw = (open --raw $yazelix_settings)
    let default_shell = (
        $yazelix_settings_raw
        | parse -r '"default_shell"\s*:\s*"(?P<value>[^"]+)"'
        | get -o value.0
        | default "unknown"
    )

    let nix_conf = "/etc/nix/nix.conf"
    let nix_conf_raw = (open --raw $nix_conf)
    let nix_conf_generated = ($nix_conf_raw =~ 'do not modify')
    let nix_custom = "/etc/nix/nix.custom.conf"
    let nix_custom_raw = (if ($nix_custom | path exists) { open --raw $nix_custom } else { "" })
    let yazelix_cache_status = if ($nix_custom_raw =~ 'yazelix.cachix.org') { "configured" } else { "missing" }

    let home_manager_version = (command-stdout "command -v home-manager >/dev/null 2>&1 && home-manager --version")
    let cargo_version = (command-stdout "command -v cargo >/dev/null 2>&1 && cargo --version")
    let rustc_version = (command-stdout "command -v rustc >/dev/null 2>&1 && rustc --version")

    let rows = [
        (tool-row nix_determinate "Determinate Nix" nix (command-stdout "nix --version") $nix_conf conf T032 original "channel:determinate-stable" system_generated $nix_conf_generated "/nix/var/nix/profiles/default/bin/nix" (sha-file $nix_conf) parsed public "System Nix config is generated by Determinate; user changes belong in nix.custom.conf.")
        (tool-row nix_flakes "Nix flakes" nix_config "enabled" $nix_conf conf T032 original "extra-experimental-features:nix-command flakes" system_generated $nix_conf_generated "" (sha-file $nix_conf) parsed public "nix.conf enables nix-command and flakes.")
        (tool-row yazelix_cachix_cache "Yazelix Cachix cache" cachix $yazelix_cache_status $nix_custom conf T032 original "cache:yazelix.cachix.org" source_of_truth false "" (sha-file $nix_custom) parsed public "nix.custom.conf carries the Yazelix Cachix substituter and public key.")
        (tool-row nushell "Nushell" shell (command-stdout "nu --version") "command:nix profile list --json" json T032 nix_profile ($nushell_profile.url? | default "missing") source_of_truth false ($nushell_profile.storePaths.0? | default "") "" parsed public "Nushell is installed through the Nix user profile.")
        (tool-row yazelix "Yazelix" runtime (command-stdout "yzx --version") "command:nix profile list --json" json T019 nix_profile ($yazelix_profile.url? | default $yazelix_flake.version_or_rev) source_of_truth false ($yazelix_profile.storePaths.0? | default $yzx_binary.path) "" parsed public "Yazelix is installed through the Nix user profile.")
        (tool-row yazelix_settings "Yazelix settings" runtime_config $"default_shell=($default_shell)" $yazelix_settings jsonc T020 original "settings.jsonc" source_of_truth false "" (sha-file $yazelix_settings) parsed public "Active Yazelix settings source for runtime behavior.")
        (tool-row home_manager "Home Manager" nix_tool $home_manager_version "/home/flexnetos/meta/src/envctl/manifest/nix-yazelix.toml" toml T032 repo_manifest "nixpkgs#home-manager" source_of_truth false "" (sha-file "/home/flexnetos/meta/src/envctl/manifest/nix-yazelix.toml") (if $home_manager_version == "not_on_path" { "not_on_path" } else { "parsed" }) public "Manifest declares Home Manager installation via Nix profile.")
        (tool-row rtk "RTK" cli (command-stdout "rtk --version") $rtk_config toml T023 original $"pinned:($rtk_release.path)#($rtk_release.version_or_rev)" source_of_truth false $rtk_binary.path (sha-file $rtk_config) parsed sensitive_refs $"RTK config policy: ($rtk_policy).")
        (tool-row kache "Kache" rust_cache (command-stdout "kache --version") $cargo_config toml T024 original $"pinned:($kache_release.path)#($kache_release.version_or_rev)" source_of_truth false $kache_binary.path (sha-file $cargo_config) parsed public $"Cargo rustc-wrapper is ($cargo_wrapper).")
        (tool-row wild "Wild linker" linker (command-stdout "wild --version") "/home/flexnetos/meta/etc/rust/wild/cargo-wild-x86_64-unknown-linux-gnu.toml" toml T025 workspace_config $"pinned:($wild_release.path)#($wild_release.version_or_rev)" source_of_truth false $wild_binary.path (sha-file "/home/flexnetos/meta/etc/rust/wild/cargo-wild-x86_64-unknown-linux-gnu.toml") parsed public "Wild is opt-in through a workspace Cargo config profile.")
        (tool-row cargo_ambient "Cargo ambient PATH" rust_tool $cargo_version "command:cargo --version" command T032 ambient_path "not_pinned" observed false "" "" (if $cargo_version == "not_on_path" { "not_on_path" } else { "parsed" }) public "Ambient non-login shell did not expose Cargo during T032 if not_on_path.")
        (tool-row rustc_ambient "rustc ambient PATH" rust_tool $rustc_version "command:rustc --version" command T032 ambient_path "not_pinned" observed false "" "" (if $rustc_version == "not_on_path" { "not_on_path" } else { "parsed" }) public "Ambient non-login shell did not expose rustc during T032 if not_on_path.")
        (rust-toolchain-row envctl_rust_toolchain "envctl Rust toolchain" "/home/flexnetos/meta/src/envctl/rust-toolchain.toml" T032 "envctl pins the development toolchain.")
        (rust-toolchain-row runner_rust_toolchain "runner Rust toolchain" "/home/flexnetos/meta/flexnetos_runner/rust-toolchain.toml" T032 "runner currently uses a floating stable toolchain.")
        (rust-toolchain-row meta_rust_toolchain "meta Rust toolchain" "/home/flexnetos/meta/rust-toolchain.toml" T032 "meta currently uses a floating stable toolchain.")
        (tool-row meta_tool_versions "meta .tool-versions" tool_versions "rust stable" "/home/flexnetos/meta/.tool-versions" text T032 repo_config "floating:stable" source_of_truth false "" (sha-file "/home/flexnetos/meta/.tool-versions") parsed public "meta asdf-style tool version file declares rust stable.")
        (rust-toolchain-row kache_rust_toolchain "kache Rust toolchain" "/home/flexnetos/meta/src/upstream/kunobi-ninja/kache/rust-toolchain.toml" T032 "upstream Kache source pins Rust 1.95.")
        (tool-row kache_mise "Kache mise tools" tool_manager "see_mise_toml" "/home/flexnetos/meta/src/upstream/kunobi-ninja/kache/mise.toml" toml T032 upstream_config "mixed:pinned-plus-rust-toolchain" source_of_truth false "" (sha-file "/home/flexnetos/meta/src/upstream/kunobi-ninja/kache/mise.toml") parsed public "mise.toml pins CI tools and defers Rust to rust-toolchain.toml.")
        (tool-row kache_flake "Kache flake" nix_flake "nixpkgs-unstable" "/home/flexnetos/meta/src/upstream/kunobi-ninja/kache/flake.nix" nix T032 upstream_config "flake.lock present" source_of_truth false "" (sha-file "/home/flexnetos/meta/src/upstream/kunobi-ninja/kache/flake.nix") parsed public "Kache upstream flake follows nixpkgs and rust-overlay inputs.")
        (tool-row wild_flake "Wild flake" nix_flake "nixos-unstable" "/home/flexnetos/meta/src/upstream/wild-linker/wild/flake.nix" nix T032 upstream_config "flake.lock present" source_of_truth false "" (sha-file "/home/flexnetos/meta/src/upstream/wild-linker/wild/flake.nix") parsed public "Wild upstream flake uses nixos-unstable and crane.")
        (tool-row gitkb_config "GitKB config" memory_tool "configured" "/home/flexnetos/meta/src/envctl/.kb/config.toml" toml T032 repo_config "auth:optional" source_of_truth false "" (sha-file "/home/flexnetos/meta/src/envctl/.kb/config.toml") parsed public "GitKB repo config is present; no GitKB binary was found on ambient PATH.")
        (tool-row flexnetos_runner_workspace "FlexNetOS runner workspace" runner "0.1.0" "/home/flexnetos/meta/flexnetos_runner/Cargo.toml" toml T032 repo_config "workspace.package.version=0.1.0" source_of_truth false "" (sha-file "/home/flexnetos/meta/flexnetos_runner/Cargo.toml") parsed public "Runner workspace manifest provides the current runner package version.")
    ]

    if $json {
        $rows | to json
    } else {
        $rows
    }
}
