#!/usr/bin/env nu

use ./meta-paths.nu *

# Commit every remaining /home/flexnetos/FlexNetOS bootstrap row onto its real
# owner. Dry-run is the default; --apply archives the exact prior table,
# publishes one candidate atomically, verifies every owner row, and emits a
# hash receipt.
#
# Sibling of profile-env-cutover.nu and codex-env-cutover.nu, and the sole
# authoritative committer for these rows.
#
# Why an EXPLICIT mapping instead of a prefix swap: /home/flexnetos/FlexNetOS is
# an empty husk, but a blind FlexNetOS -> meta rewrite is wrong for three rows.
#   * META_ROOT would become <meta>/src/meta, which does not exist. The meta
#     workspace root is the directory holding the .meta.yaml marker.
#   * FLEXNETOS_RUNNER_ROOT would become <meta>/src/flexnetos_runner. That path
#     does exist, but it is a duplicate checkout; .meta.yaml declares
#     flexnetos_runner with no path override, so the canonical peer is
#     <meta>/flexnetos_runner.
#   * KACHE_CACHE_DIR would become <meta>/var/cache/kache, which yazelix's
#     nushell/system/volatile_runtime.nu lists in LEGACY_KACHE_ROOTS and
#     actively refuses to let exist. Kache's real root is ~/.cache/kache.
# Every other row is a straight root swap onto a path that already exists.

const RETIRED_ROOT = "/home/flexnetos/FlexNetOS"

# Paths an active owner elsewhere forbids. Committing one of these would put the
# table in direct conflict with a live gate. Derived from the resolved root so
# this file pins no user path.
def forbidden-targets [meta: string] {
    [
        $"($meta)/.cache/kache"
        $"($meta)/var/cache/kache"
    ]
}

def owned-values [meta: string] {
    {
        FLEXNETOS_WORKSPACE: $meta
        FLEXNETOS_SRC: $"($meta)/src"
        ENVCTL_ROOT: $"($meta)/src/envctl"
        FLEXNETOS_RUNNER_ROOT: $"($meta)/flexnetos_runner"
        META_ROOT: $meta
        FLEXNETOS_ETC: $"($meta)/etc"
        FLEXNETOS_RELEASE_ROOT: $"($meta)/release"
        FLEXNETOS_TEST_PREFIX_ROOT: $"($meta)/test-prefix"
        FLEXNETOS_VAR: $"($meta)/var"
        ENVCTL_TABLE_ROOT: $"($meta)/var/lib/envctl/tables"
        FXRUN_STATE_DIR: $"($meta)/var/lib/runner"
        GITKB_HOME: $"($meta)/var/lib/gitkb"
        META_STATE_DIR: $"($meta)/var/lib/meta"
        RUSTUP_HOME: $"($meta)/var/lib/rustup"
        ENVCTL_LOG_DIR: $"($meta)/var/log/envctl"
        RAW_FAILURE_LOG_DIR: $"($meta)/var/log/raw"
        RUNNER_LOG_DIR: $"($meta)/var/log/runner"
        RTK_LOG_DIR: $"($meta)/var/log/rtk"
        RTK_CACHE_DIR: $"($meta)/var/cache/rtk"
        YAZELIX_CACHE_DIR: $"($meta)/var/cache/yazelix"
        # Kache is rooted outside the payload by yazelix's own enforced owner.
        KACHE_CACHE_DIR: ($env.HOME | path join ".cache" "kache")
    }
}

def fail [message: string] {
    print --stderr $"retired root cutover: ($message)"
    exit 1
}

def main [
    --meta-root: string = ""
    --timestamp: string = ""
    --apply
] {
    let root = (meta-root $meta_root)
    let tables_root = ($root | path join "var" "lib" "envctl" "tables")
    let table = ($tables_root | path join "bootstrap_env_vars.csv")
    if not ($table | path exists) {
        fail $"canonical table is missing: ($table)"
    }

    let owned = (owned-values $root)
    let forbidden = (forbidden-targets $root)
    let names = ($owned | columns)

    # No committed value may re-enter the retired husk or land on a path another
    # owner forbids.
    for name in $names {
        let value = ($owned | get $name)
        if ($value | str starts-with $RETIRED_ROOT) {
            fail $"($name) still targets the retired root: ($value)"
        }
        if $value in $forbidden {
            fail $"($name) targets a path an active owner forbids: ($value)"
        }
    }

    let rows = (open $table)
    for name in $names {
        let matches = ($rows | where {|row| $row.name == $name })
        if (($matches | length) != 1) {
            fail $"expected exactly one ($name) row, found ($matches | length)"
        }
    }

    let prior = ($names | reduce --fold {} {|name, acc|
        $acc | upsert $name ($rows | where {|row| $row.name == $name } | first | get value)
    })

    let updated = ($rows | each {|row|
        if $row.name in $names {
            $row | upsert value ($owned | get $row.name)
        } else {
            $row
        }
    })

    # Fail closed if any row anywhere would still name the retired root.
    let residual = ($updated | where {|row| $row.value | str starts-with $RETIRED_ROOT } | get name)
    if not ($residual | is-empty) {
        fail $"rows would still name the retired root: ($residual | str join ',')"
    }

    let before_hash = (open --raw $table | hash sha256)
    let rendered = ($updated | to csv)
    let after_hash = ($rendered | hash sha256)
    let observed_at = if ($timestamp | is-empty) {
        date now | format date "%Y%m%dT%H%M%S%3fZ"
    } else {
        $timestamp
    }
    let archive = ($root | path join "var" "lib" "envctl" "archives" "retired-root-cutover" $observed_at)
    mut receipt = {
        schema: "envctl.retired-root-cutover.v1"
        observed_at: $observed_at
        applied: $apply
        table: ($table | into string)
        archive: ($archive | into string)
        retired_root: $RETIRED_ROOT
        rows_committed: ($names | length)
        prior_values: $prior
        committed_values: $owned
        before_sha256: $before_hash
        after_sha256: $after_hash
        verified: false
    }

    if $apply {
        mkdir $archive
        cp $table ($archive | path join "bootstrap_env_vars.csv.before")
        let candidate = ($tables_root | path join "bootstrap_env_vars.csv.envctl-candidate")
        $rendered | save --raw --force $candidate
        mv --force $candidate $table
        let committed = (open $table)
        let rows_ok = ($names | all {|name|
            let expected = ($owned | get $name)
            (($committed | where {|row| $row.name == $name and $row.value == $expected }) | length) == 1
        })
        let none_retired = (($committed | where {|row| $row.value | str starts-with $RETIRED_ROOT }) | is-empty)
        if not ($rows_ok and $none_retired) {
            cp ($archive | path join "bootstrap_env_vars.csv.before") $table
            fail "post-commit verification failed; prior table restored"
        }
        $receipt = ($receipt | upsert verified true)
        $receipt | to json --indent 2 | save --force ($archive | path join "retired-root-cutover.receipt.json")
    }

    $receipt | to json --indent 2
}
