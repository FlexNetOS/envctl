# Meta /usr mirror on nushell's PATH (added by envctl)
#
# Prepends the canonical $META_ROOT/usr frontdoor tree to $env.PATH (and the
# /usr lib tree to $env.LD_LIBRARY_PATH) for interactive + login nushell —
# yazelix sessions (sourced from ../yazelix/shell_nu.nu) and standalone `nu -l`
# (sourced from ./config.nu). Mirrors the ordering of `envctl env --toolchains`
# but native to nu, so usr/bin lands even when nu is not a child of the bash
# login that runs that eval.
#
# Portable: derives every path from $env.META_ROOT — no hardcoded $HOME (the
# ADR-0006 wave-2 rule the sibling rtk-wrappers.nu follows). Idempotent: `uniq`
# dedupes if PATH already carries these. Guarded: a missing META_ROOT, missing
# PATH, or a not-yet-converted (string) PATH simply no-ops — never breaks the shell.

const META_USR_BIN_SUBDIRS = ["usr/bin" "usr/sbin" "usr/local/bin" "usr/local/sbin" ".local/bin"]
const META_USR_LIB_SUBDIRS = ["usr/lib" "usr/lib64" "usr/local/lib" "usr/local/lib64"]

if ("META_ROOT" in $env) and ("PATH" in $env) and (($env.PATH | describe) =~ "list") {
    let m = $env.META_ROOT
    let usr_bins = ($META_USR_BIN_SUBDIRS | each {|s| [$m $s] | path join })
    $env.PATH = ($usr_bins | append $env.PATH | uniq)

    let usr_libs = ($META_USR_LIB_SUBDIRS | each {|s| [$m $s] | path join })
    let joined = ($usr_libs | str join (char esep))
    let prior = ($env.LD_LIBRARY_PATH? | default "")
    $env.LD_LIBRARY_PATH = (if ($prior | is-empty) { $joined } else { $"($joined)(char esep)($prior)" })
}
