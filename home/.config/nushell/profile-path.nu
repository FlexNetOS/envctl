# Strict single-profile PATH ownership for Nushell.
#
# Every installed development/runtime command is exposed through the current
# Yazelix profile. Inherited raw store paths, other Nix profile selectors,
# workspace tool farms, and user-agent state directories are competing owners
# and are deliberately discarded at every login-shell boundary.

if ("HOME" in $env) and ("PATH" in $env) and (($env.PATH | describe) =~ "list") {
    let profile = ([$env.HOME ".nix-profile"] | path join)
    let profile_bins = [
        ([$profile "toolbin"] | path join)
        ([$profile "bin"] | path join)
    ] | where { path exists }

    let expected = ["toolbin" "bin"] | each {|part| [$profile $part] | path join }
    if $profile_bins != $expected {
        error make {msg: $"strict profile PATH is incomplete: expected ($expected | str join ', ')"}
    }

    let profile_nu = ([$profile "toolbin" "nu"] | path join)
    if not ($profile_nu | path exists) {
        error make {msg: $"strict profile Nushell frontdoor is missing: ($profile_nu)"}
    }

    $env.SHELL = $profile_nu
    $env.PATH = $profile_bins

    let meta_root = ([$env.HOME "meta"] | path join)
    let profile_data = ([$meta_root "var" "lib"] | path join)
    let runtime_root = (($env.XDG_RUNTIME_DIR? | default "/run/user/1001") | path join "yazelix")
    let profile_cache = ($runtime_root | path join "volatile" "cache")
    let yazelix_state = ($runtime_root | path join "profile-runtime" "yazelix")
    for directory in [$profile_data $profile_cache $yazelix_state] {
        mkdir $directory
    }
    $env.XDG_DATA_HOME = $profile_data
    $env.XDG_STATE_HOME = $profile_data
    $env.XDG_CACHE_HOME = $profile_cache
    $env.YAZELIX_STATE_DIR = $yazelix_state
}

if "LD_LIBRARY_PATH" in $env {
    hide-env LD_LIBRARY_PATH
}
