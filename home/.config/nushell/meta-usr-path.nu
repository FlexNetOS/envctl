# Active-profile and Meta /usr PATH ownership for Nushell (added by envctl)
#
# Always resets Yazelix/Codex runtime commands to the lexical frontdoors in the
# one active Nix profile. This matters after an in-place profile upgrade: a
# long-lived parent process can still export the previous generation's raw
# /nix/store/...-lifeos-foundation-yzx/{toolbin,bin} or codex-path directory.
# Those paths are generated runtime outputs, not independent install owners.
# A child Nu shell must discard them instead of preserving a stale generation.
#
# When META_ROOT is present, also prepends the canonical $META_ROOT/usr tree to
# $env.PATH (and its /usr lib tree to $env.LD_LIBRARY_PATH). This mirrors the
# ordering of `envctl env --toolchains` but is native to Nu.
#
# Portable: derives every path from $env.META_ROOT — no hardcoded $HOME (the
# ADR-0006 wave-2 rule the sibling rtk-wrappers.nu follows). Idempotent: `uniq`
# dedupes if PATH already carries these. Guarded: a missing META_ROOT, missing
# PATH, or a not-yet-converted (string) PATH simply no-ops — never breaks the shell.

const META_USR_BIN_SUBDIRS = ["usr/bin" "usr/sbin" "usr/local/bin" "usr/local/sbin" ".local/bin"]
const META_USR_LIB_SUBDIRS = ["usr/lib" "usr/lib64" "usr/local/lib" "usr/local/lib64"]

if ("HOME" in $env) and ("PATH" in $env) and (($env.PATH | describe) =~ "list") {
    let profile = ([$env.HOME ".nix-profile"] | path join)
    let profile_bins = [
        ([$profile "toolbin"] | path join)
        ([$profile "bin"] | path join)
    ] | where { path exists }
    let profile_nu = ([$profile "toolbin" "nu"] | path join)
    if ($profile_nu | path exists) {
        $env.SHELL = $profile_nu
    }
    let inherited = ($env.PATH | each { into string })
    let current_only = ($inherited | where {|entry|
        not (
            ($entry =~ '^/nix/store/[^/]+-lifeos-foundation-yzx/(toolbin|bin)$') or
            ($entry =~ '^/nix/store/[^/]+-codex-cli-[^/]+/codex-path$')
        )
    })
    $env.PATH = ($profile_bins | append $current_only | uniq)
}

if ("META_ROOT" in $env) and ("PATH" in $env) and (($env.PATH | describe) =~ "list") {
    let m = $env.META_ROOT
    let usr_bins = ($META_USR_BIN_SUBDIRS | each {|s| [$m $s] | path join })
    $env.PATH = ($usr_bins | append $env.PATH | uniq)

    let usr_libs = ($META_USR_LIB_SUBDIRS | each {|s| [$m $s] | path join })
    let joined = ($usr_libs | str join (char esep))
    let prior = ($env.LD_LIBRARY_PATH? | default "")
    $env.LD_LIBRARY_PATH = (if ($prior | is-empty) { $joined } else { $"($joined)(char esep)($prior)" })
}
