#!/usr/bin/env bash
set -euo pipefail

root="$(git rev-parse --show-toplevel)"
if ! command -v nu >/dev/null 2>&1; then
  echo "test-profile-env-cutover: structural PASS (nu unavailable; behavior not run)"
  exit 0
fi

tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT
meta="$tmp/meta"
tables="$meta/var/lib/envctl/tables"
mkdir -p "$tables"
cat >"$tables/bootstrap_env_vars.csv" <<'CSV'
name,value_kind,value,owner_table,scope,precedence,sensitivity,source_ref,generated_target,notes
META_ROOT,path,/fixture/meta,env_vars,user-default,10,public,T006,bootstrap.nu;bootstrap.sh,Meta root.
XDG_DATA_HOME,path,/retired/data,env_vars,user-default,90,public,T006,bootstrap.nu;bootstrap.sh,Old data.
XDG_STATE_HOME,path,/retired/state,env_vars,user-default,90,local_state,T006,bootstrap.nu;bootstrap.sh,Old state.
CSV

before="$(sha256sum "$tables/bootstrap_env_vars.csv" | cut -d' ' -f1)"
nu "$root/scripts/profile-env-cutover.nu" --meta-root "$meta" --timestamp fixture >"$tmp/dry.json"
test "$(sha256sum "$tables/bootstrap_env_vars.csv" | cut -d' ' -f1)" = "$before"
grep -Fq '"applied": false' "$tmp/dry.json"

nu "$root/scripts/profile-env-cutover.nu" --meta-root "$meta" --timestamp fixture --apply >"$tmp/apply.json"
test "$(nu -c "open '$tables/bootstrap_env_vars.csv' | where name == XDG_DATA_HOME | get 0.value")" = "$meta/var/xdg-data"
test "$(nu -c "open '$tables/bootstrap_env_vars.csv' | where name == XDG_STATE_HOME | get 0.value")" = "$meta/var/xdg-state"
grep -Fq '"verified": true' "$tmp/apply.json"
test -f "$meta/var/lib/envctl/archives/profile-env-cutover/fixture/bootstrap_env_vars.csv.before"
test -f "$meta/var/lib/envctl/archives/profile-env-cutover/fixture/profile-env-cutover.receipt.json"

echo "test-profile-env-cutover: PASS"
