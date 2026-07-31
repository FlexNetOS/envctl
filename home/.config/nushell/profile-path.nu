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

    # Profile ownership is precedence, not exclusivity. The profile frontdoors are
    # prepended so they win every lookup; only the competing owners named above are
    # discarded. Assigning $profile_bins outright also removed /usr/bin, /bin and
    # /snap/bin -- and because this user's /etc/passwd login shell is nu, that
    # amputated PATH reached the graphical session and left system capabilities
    # (snaps, desktop launchers, D-Bus-activated helpers) unresolvable.
    let local_root = ([$env.HOME ".local"] | path join)
    let inherited = (
        $env.PATH
        | where {|entry| not ($entry | str starts-with "/nix/store/") }
        | where {|entry| not ($entry | str starts-with $local_root) }
        | where {|entry| not ($entry in $profile_bins) }
    )

    # Guaranteed floor: a login that arrives with an already-stripped PATH must not
    # be able to propagate that loss into the session.
    let system_baseline = ([
        "/usr/local/sbin" "/usr/local/bin" "/usr/sbin" "/usr/bin" "/sbin" "/bin"
        "/usr/games" "/usr/local/games" "/snap/bin"
    ] | where { path exists })

    $env.PATH = ($profile_bins | append $inherited | append $system_baseline | uniq)

    let meta_root = "/home/flexnetos/meta"
    let profile_data = ($meta_root | path join "var" "xdg-data")
    let profile_state = ($meta_root | path join "var" "xdg-state")
    let runtime_root = ($meta_root | path join "var" "lib" "yazelix" "runtime")
    let xdg_runtime = ($runtime_root | path join "xdg")
    let profile_cache = ($runtime_root | path join "volatile" "cache")
    let yazelix_state = ($runtime_root | path join "state")
    for directory in [$profile_data $profile_state $xdg_runtime $profile_cache $yazelix_state] {
        mkdir $directory
    }
    $env.XDG_DATA_HOME = $profile_data
    $env.XDG_STATE_HOME = $profile_state
    $env.XDG_RUNTIME_DIR = $xdg_runtime
    $env.XDG_CACHE_HOME = $profile_cache
    $env.YAZELIX_STATE_DIR = $yazelix_state
}

if "LD_LIBRARY_PATH" in $env {
    hide-env LD_LIBRARY_PATH
}
