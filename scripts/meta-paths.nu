#!/usr/bin/env nu

# Resolve the meta workspace root without hardcoding a user path.
#
# Every envctl script that needs an absolute location under the workspace should
# derive it from here instead of embedding /home/flexnetos/meta. The root is the
# directory holding the `.meta.yaml` marker -- the same seam `envctl env` exposes
# ("locate meta WITHOUT hardcoding paths"), resolved the way git finds `.git`.
#
# Resolution order:
#   1. an explicit override (a --meta-root flag)
#   2. $META_ROOT from the environment (what `envctl env` exports)
#   3. walk up from the caller's directory looking for .meta.yaml
#   4. walk up from this file's own directory
#
# Deliberately does NOT fall back to a literal path: a wrong-but-plausible default
# is how the retired /home/flexnetos/FlexNetOS root survived in generated tables
# long after the tree itself was gone. Fail loudly instead.

export def find-marker [start: path] {
    mut current = ($start | path expand)
    loop {
        if (($current | path join ".meta.yaml") | path exists) {
            return $current
        }
        let parent = ($current | path dirname)
        if $parent == $current {
            return ""
        }
        $current = $parent
    }
}

export def meta-root [override?: string] {
    if ($override | default "" | is-not-empty) {
        let expanded = ($override | path expand)
        if not (($expanded | path join ".meta.yaml") | path exists) {
            error make {msg: $"--meta-root ($expanded) has no .meta.yaml marker"}
        }
        return $expanded
    }

    let from_env = ($env.META_ROOT? | default "")
    if ($from_env | is-not-empty) and (($from_env | path join ".meta.yaml") | path exists) {
        return ($from_env | path expand)
    }

    let from_cwd = (find-marker $env.PWD)
    if ($from_cwd | is-not-empty) { return $from_cwd }

    let from_file = (find-marker $env.FILE_PWD)
    if ($from_file | is-not-empty) { return $from_file }

    error make {msg: "cannot resolve the meta root: no .meta.yaml marker above PWD or this script, and $META_ROOT is unset or invalid"}
}

export def tables-root [override?: string] {
    (meta-root $override) | path join "var" "lib" "envctl" "tables"
}

# Render an absolute path under the workspace as its portable `$META_ROOT/...`
# form, for values RECORDED INTO TABLES. Scripts should read from the resolved
# absolute path and record the portable form, so a table never pins one user's
# home. `$META_ROOT/...` is the established table convention -- see the
# secretd_toml row in env_precedence.csv.
export def meta-rel [absolute: string, root?: string] {
    let base = (if ($root | default "" | is-not-empty) { $root } else { (meta-root "") })
    if ($absolute | str starts-with $base) {
        $absolute | str replace $base "$META_ROOT"
    } else {
        $absolute
    }
}

# Inverse of meta-rel: expand a recorded `$META_ROOT/...` value back to an
# absolute path for filesystem access.
export def meta-abs [recorded: string, root?: string] {
    let base = (if ($root | default "" | is-not-empty) { $root } else { (meta-root "") })
    $recorded | str replace "$META_ROOT" $base
}

def main [--meta-root: string = ""] {
    let root = (meta-root $meta_root)
    print ({META_ROOT: $root, META_FILE: ($root | path join ".meta.yaml"), TABLES_ROOT: (tables-root $meta_root)} | to json --indent 2)
}
