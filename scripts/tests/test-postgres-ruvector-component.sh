#!/usr/bin/env bash
set -euo pipefail

fail() { printf 'FAIL: %s\n' "$*" >&2; exit 1; }
expect_fail() {
  local label="$1"
  shift
  if "$@" >"$tmp/expect-fail.out" 2>"$tmp/expect-fail.err"; then
    fail "$label unexpectedly succeeded"
  fi
}
assert_order() {
  local file="$1" first="$2" second="$3" first_line second_line
  first_line="$(grep -n -m1 -F "$first" "$file" 2>/dev/null | cut -d: -f1 || true)"
  second_line="$(grep -n -m1 -F "$second" "$file" 2>/dev/null | cut -d: -f1 || true)"
  [[ -n "$first_line" && -n "$second_line" && "$first_line" -lt "$second_line" ]] ||
    fail "$first must precede $second in $file"
}

root="$(git -C "$(dirname "${BASH_SOURCE[0]}")" rev-parse --show-toplevel)"
manifest="$root/manifest/components.d/postgres-ruvector.toml"
lifecycle="$root/assets/scripts/envctl-postgres-ruvector-lifecycle.sh"
t3="$root/scripts/tests/blueprint/t3_embedder_wiring.sh"
tmp="$(mktemp -d)"
foreign_pid=""
trap 'if [[ -n "$foreign_pid" ]]; then kill "$foreign_pid" 2>/dev/null || true; wait "$foreign_pid" 2>/dev/null || true; fi; rm -rf "$tmp"' EXIT

# The lifecycle must stay sourceable: the test replaces only the documented OS/process seams.
# shellcheck source=../../assets/scripts/envctl-postgres-ruvector-lifecycle.sh
source "$lifecycle"

case_root="$tmp/case"
meta="$case_root/meta"
real_home="$case_root/home"
store="$case_root/store"
combined="$store/aaaaaaaa-postgresql-and-plugins-17.10"
frontdoors="$store/cccccccc-flexnetos-foundation-postgresql-frontdoors-17.10-ruvector-0.3.0"
profile_generation="$store/bbbbbbbb-profile"
pg="$meta/var/lib/postgresql"
data="$pg/17"

postgres_store_root() { printf '%s\n' "$store"; }

write_profile_commands() {
  cat >"$combined/toolbin/postgres" <<'SH'
#!/usr/bin/env bash
if [[ "${1:-}" == --version ]]; then
  printf 'postgres (PostgreSQL) %s\n' "${FAKE_PG_VERSION:-17.10}"
  exit 0
fi
exit 0
SH

  cat >"$combined/toolbin/pg_config" <<'SH'
#!/usr/bin/env bash
case "${1:-}" in
  --version) printf 'PostgreSQL %s\n' "${FAKE_PG_CONFIG_VERSION:-17.10}" ;;
  --bindir) printf '%s/bin\n' "${FAKE_COMBINED:?}" ;;
  --pkglibdir) printf '%s/lib\n' "${FAKE_COMBINED:?}" ;;
  --sharedir) printf '%s/share/postgresql\n' "${FAKE_COMBINED:?}" ;;
  --includedir-server) printf '%s/include/server\n' "${FAKE_COMBINED:?}" ;;
  *) exit 2 ;;
esac
SH

  cat >"$combined/toolbin/psql" <<'SH'
#!/usr/bin/env bash
set -euo pipefail
sql="${*: -1}"
printf '%s\n' "$sql" >>"${FAKE_PSQL_LOG:?}"
case "$sql" in
  *"SHOW data_directory"*) printf '%s\n' "${FAKE_DATA_DIR:?}" ;;
  *"SHOW unix_socket_directories"*) printf '%s\n' "${FAKE_SOCKET_DIR:?}" ;;
  *"SHOW listen_addresses"*) printf '%s' "${FAKE_LISTEN_ADDRESSES:-}" ;;
  *"SHOW port"*) printf '%s\n' "${FAKE_PORT:-5432}" ;;
  *"extversion"*) printf '%s\n' "${FAKE_EXT_VERSION:-0.3.0}" ;;
  *"embedding_minilm IS NULL"*) printf '%s\n' "${FAKE_MISSING:-0}" ;;
  *"<=>"*) printf '%s|%s\n' "${FAKE_DISTANCE:-0}" "${FAKE_DISTANCE_OK:-t}" ;;
  *"count(*) FROM codebase"*) printf '%s\n' "${FAKE_CODEBASE:-5157}" ;;
  *"count(*) FROM episodes"*) printf '%s\n' "${FAKE_EPISODES:-23}" ;;
  *) exit 3 ;;
esac
SH

  cat >"$combined/toolbin/pg_ctl" <<'SH'
#!/usr/bin/env bash
printf '%s\n' "$*" >>"${FAKE_PG_CTL_LOG:?}"
exit 77
SH
  cat >"$combined/toolbin/pg_isready" <<'SH'
#!/usr/bin/env bash
exit 0
SH
  cat >"$combined/toolbin/initdb" <<'SH'
#!/usr/bin/env bash
printf 'CALLED\n' >>"${FAKE_INITDB_LOG:?}"
exit 99
SH
  chmod 0755 "$combined/toolbin/"*

  local command
  for command in postgres psql pg_ctl pg_isready pg_config initdb; do
    cat >"$frontdoors/bin/$command" <<SH
#!/usr/bin/env bash
exec "\${FAKE_COMBINED:?}/toolbin/$command" "\$@"
SH
    chmod 0755 "$frontdoors/bin/$command"
  done
}

reset_fixture() {
  rm -rf "$case_root"
  mkdir -p "$data/pg_tblspc" "$meta/var/lib/ruvector" \
    "$real_home/.local/state/nix" \
    "$combined/toolbin" "$combined/bin" "$combined/lib" \
    "$combined/share/postgresql/extension" "$combined/include/server" "$frontdoors/bin" \
    "$profile_generation/toolbin"
  chmod 0700 "$data"
  printf '17\n' >"$data/PG_VERSION"
  printf 'preserve-pgdata\n' >"$data/data-marker"
  printf 'preserve-ruvector\n' >"$meta/var/lib/ruvector/data-marker"
  write_profile_commands
  ln -s ../toolbin/postgres "$combined/bin/postgres"
  local command
  for command in postgres psql pg_ctl pg_isready pg_config initdb; do
    ln -s "$frontdoors/bin/$command" "$profile_generation/toolbin/$command"
  done
  printf 'header\n' >"$combined/include/server/postgres.h"
  printf 'ELF-fixture\n' >"$combined/lib/ruvector.so"
  printf '%s\n' "$POSTGRES_RUVECTOR_SOURCE" \
    >"$combined/share/postgresql/extension/.envctl-ruvector-source"
  cat >"$combined/share/postgresql/extension/ruvector.control" <<'EOF'
comment = 'ruvector fixture'
default_version = '0.3.0'
module_pathname = '$libdir/ruvector'
relocatable = true
EOF
  printf '%s\n' '-- ruvector 0.3.0 fixture schema' \
    >"$combined/share/postgresql/extension/ruvector--0.3.0.sql"
  ln -s "$profile_generation" "$real_home/.local/state/nix/profile-1-link"
  ln -s profile-1-link "$real_home/.local/state/nix/profile"
  ln -s "$real_home/.local/state/nix/profile" "$real_home/.nix-profile"

  export META_ROOT="$meta"
  export ENVCTL_REAL_HOME="$real_home"
  export FAKE_COMBINED="$combined"
  export FAKE_PSQL_LOG="$case_root/psql.log"
  export FAKE_PG_CTL_LOG="$case_root/pg-ctl.log"
  export FAKE_INITDB_LOG="$case_root/initdb.log"
  export FAKE_DATA_DIR="$data"
  export FAKE_SOCKET_DIR="$pg"
  export FAKE_PORT=5432
  unset FAKE_PG_VERSION FAKE_PG_CONFIG_VERSION FAKE_EXT_VERSION FAKE_MISSING
  unset FAKE_DISTANCE FAKE_DISTANCE_OK FAKE_CODEBASE FAKE_EPISODES FAKE_LISTEN_ADDRESSES
  : >"$FAKE_PSQL_LOG"
  : >"$FAKE_PG_CTL_LOG"
  POSTGRES_LEGACY_TXN=""
  POSTGRES_LEGACY_PATHS=()
}

reset_fixture
/usr/bin/bash -n "$lifecycle"
/usr/bin/bash -n "$t3"

# The manifest delegates every phase to the one audited lifecycle and pins its dependency.
python3 - "$manifest" "$lifecycle" <<'PY'
from pathlib import Path
import sys
import tomllib

manifest = Path(sys.argv[1])
lifecycle = Path(sys.argv[2])
data = tomllib.loads(manifest.read_text())
components = [c for c in data["component"] if c["id"] == "postgres-ruvector"]
assert len(components) == 1
c = components[0]
assert c.get("requires") == ["yazelix"]
for phase in ("detect", "install", "verify", "fix", "remove"):
    hook = c[phase]
    body = hook.get("script", hook.get("args", [""])[-1])
    assert "ENVCTL_SOURCE_ROOT" in body
    assert "assets/scripts/envctl-postgres-ruvector-lifecycle.sh" in body
    assert body.rstrip().endswith(phase if phase not in ("install", "fix") else phase)
assert "/home/flexnetos/lifeos" not in manifest.read_text()
assert "pg17-rw/bin" not in manifest.read_text()
text = lifecycle.read_text()
assert ".nix-profile/toolbin" in text
assert "17.10" in text and "0.3.0" in text
assert "crates.io:ruvector-postgres:2.0.5:sha256:052dadb088cb26e640833072416ad59a2b2437dbb534f7effe197e30261fe1d7" in text
assert "listen_addresses=" in text
PY

grep -Fq 'PSQL="${PSQL_BIN:-$ENVCTL_REAL_HOME/.nix-profile/toolbin/psql}"' "$t3" ||
  fail 'T3 does not use the profile-owned psql default'
grep -Fq 'PGHOST="${PGHOST_DIR:-$META_ROOT/var/lib/postgresql}"' "$t3" ||
  fail 'T3 socket default is not META_ROOT-relative'
if grep -Fq '/home/flexnetos/lifeos' "$t3"; then
  fail 'T3 still embeds the retired lifeos root'
fi

# Exact profile chain/package/extension validation.
postgres_profile_validate
[[ "$POSTGRES_TOOLBIN" == "$real_home/.nix-profile/toolbin" ]] || fail 'wrong profile command root'
[[ "$POSTGRES_POSTGRES_REAL" == "$frontdoors/bin/postgres" ]] || fail 'postgres frontdoor is not profile/store owned'
[[ "$POSTGRES_SERVER_REAL" == "$combined/toolbin/postgres" ]] || fail 'server executable is not combined-package owned'
postgres_data_validate

rm "$profile_generation/toolbin/pg_isready"
expect_fail 'missing profile command' postgres_profile_validate
ln -s "$frontdoors/bin/pg_isready" "$profile_generation/toolbin/pg_isready"

FAKE_PG_VERSION=17.9
export FAKE_PG_VERSION
expect_fail 'wrong PostgreSQL version' postgres_profile_validate
unset FAKE_PG_VERSION

printf 'wrong-source\n' >"$combined/share/postgresql/extension/.envctl-ruvector-source"
expect_fail 'wrong ruvector source marker' postgres_profile_validate
printf '%s\n' "$POSTGRES_RUVECTOR_SOURCE" \
  >"$combined/share/postgresql/extension/.envctl-ruvector-source"

rm "$combined/include/server/postgres.h"
expect_fail 'missing server headers' postgres_profile_validate
printf 'header\n' >"$combined/include/server/postgres.h"

rm "$profile_generation/toolbin/pg_isready"
ln -s /usr/bin/true "$profile_generation/toolbin/pg_isready"
expect_fail 'profile command escaping Nix store' postgres_profile_validate
rm "$profile_generation/toolbin/pg_isready"
ln -s "$frontdoors/bin/pg_isready" "$profile_generation/toolbin/pg_isready"

rm "$real_home/.nix-profile"
ln -s "$profile_generation" "$real_home/.nix-profile"
expect_fail 'hostile direct profile generation' postgres_profile_validate
rm "$real_home/.nix-profile"
ln -s "$real_home/.local/state/nix/profile" "$real_home/.nix-profile"
postgres_profile_validate

printf '16\n' >"$data/PG_VERSION"
expect_fail 'wrong PGDATA major' postgres_data_validate
printf '17\n' >"$data/PG_VERSION"
chmod 0755 "$data"
expect_fail 'hostile PGDATA permissions' postgres_data_validate
chmod 0700 "$data"
postgres_data_validate

# Extension/data health is exact: 0.3.0, both nonempty lanes, zero gaps, and an executed <=>.
postgres_profile_validate
postgres_error 'reporter-contract-probe' >/dev/null 2>&1 ||
  fail 'the error reporter must not short-circuit lifecycle cleanup under errexit'
FAKE_DISTANCE=2.3980817e-14
export FAKE_DISTANCE
postgres_verify_queries_at "$pg" 5432
grep -Fq '<=>' "$FAKE_PSQL_LOG" || fail 'verify did not execute a real ruvector operator query'
unset FAKE_DISTANCE
FAKE_EXT_VERSION=0.1.0
export FAKE_EXT_VERSION
expect_fail 'wrong installed extension version' postgres_verify_queries_at "$pg" 5432
unset FAKE_EXT_VERSION
FAKE_MISSING=1
export FAKE_MISSING
expect_fail 'missing MiniLM embeddings' postgres_verify_queries_at "$pg" 5432
unset FAKE_MISSING
FAKE_CODEBASE=0
export FAKE_CODEBASE
expect_fail 'empty codebase lane' postgres_verify_queries_at "$pg" 5432
unset FAKE_CODEBASE
FAKE_DISTANCE=0.25
FAKE_DISTANCE_OK=f
export FAKE_DISTANCE FAKE_DISTANCE_OK
expect_fail 'broken vector distance operator' postgres_verify_queries_at "$pg" 5432
unset FAKE_DISTANCE FAKE_DISTANCE_OK
FAKE_DISTANCE=NaN
FAKE_DISTANCE_OK=f
export FAKE_DISTANCE FAKE_DISTANCE_OK
expect_fail 'non-finite vector distance operator' postgres_verify_queries_at "$pg" 5432
unset FAKE_DISTANCE FAKE_DISTANCE_OK

# Hostile process/socket namespaces are refused, never cleaned up or replaced.
postgres_data_validate
/usr/bin/sleep 60 &
foreign_pid=$!
printf '%s\n%s\n' "$foreign_pid" "$data" >"$data/postmaster.pid"
if postgres_cluster_state >/dev/null 2>&1; then
  fail 'foreign live PID was accepted as profile postgres'
else
  [[ "$?" -eq 2 ]] || fail 'foreign PID did not produce ambiguous state'
fi
kill "$foreign_pid"
wait "$foreign_pid" 2>/dev/null || true
foreign_pid=""
rm "$data/postmaster.pid"
printf 'not-a-socket\n' >"$pg/.s.PGSQL.5432"
if postgres_cluster_state >/dev/null 2>&1; then
  fail 'foreign socket namespace was accepted'
else
  [[ "$?" -eq 2 ]] || fail 'foreign socket did not produce ambiguous state'
fi
rm "$pg/.s.PGSQL.5432"
printf '99999999\n%s\n' "$data" >"$data/postmaster.pid"
if postgres_cluster_state >/dev/null 2>&1; then
  fail 'stale postmaster PID was accepted'
else
  [[ "$?" -eq 2 ]] || fail 'stale PID did not produce ambiguous state'
fi
rm "$data/postmaster.pid"

# Exercise the real scratch-parity orchestration through sourceable process/copy seams.
scratch_trace="$tmp/scratch-order.log"
(
  postgres_profile_validate
  postgres_data_validate
  postgres_scratch_root_create() {
    local dir="$META_ROOT/var/tmp/postgres-ruvector-parity.fixture"
    mkdir -p "$dir"
    printf '%s\n' "$dir"
  }
  postgres_scratch_port() { printf '55432\n'; }
  postgres_copy_data_tree() { printf 'copy:%s:%s\n' "$1" "$2" >>"$scratch_trace"; cp "$1/PG_VERSION" "$2/"; }
  postgres_pg_ctl_start() { printf 'start:%s:%s:%s\n' "$1" "$2" "$3" >>"$scratch_trace"; }
  postgres_validate_running_at() { printf 'validate:%s:%s:%s\n' "$1" "$2" "$3" >>"$scratch_trace"; }
  postgres_verify_queries_at() { printf 'parity:%s:%s\n' "$1" "$2" >>"$scratch_trace"; }
  postgres_stop_owned_cluster() { printf 'stop:%s\n' "$1" >>"$scratch_trace"; }
  postgres_remove_scratch_tree() { printf 'remove:%s\n' "$1" >>"$scratch_trace"; rm -rf "$1"; }
  postgres_run_scratch_parity
)
assert_order "$scratch_trace" 'copy:' 'start:'
assert_order "$scratch_trace" 'start:' 'validate:'
assert_order "$scratch_trace" 'validate:' 'parity:'
assert_order "$scratch_trace" 'parity:' 'stop:'
assert_order "$scratch_trace" 'stop:' 'remove:'
grep -Fq ':55432' "$scratch_trace" || fail 'scratch parity did not use a private high port'
grep -Fq '/socket:55432' "$scratch_trace" || fail 'scratch parity did not use a private socket namespace'
[[ "$(<"$data/data-marker")" == preserve-pgdata ]] || fail 'scratch parity changed source PGDATA'

# Legacy retirement is gated by successful scratch parity and live activation, then archived.
reset_fixture
mkdir -p "$pg/pg17-rw/bin"
printf 'legacy-copy\n' >"$pg/pg17-rw/bin/postgres"
ln -s "$combined" "$pg/pg17"
ln -s "$combined" "$pg/pg17-config"
activation_log="$tmp/activation-order.log"
running_marker="$case_root/running"
archive="$meta/var/lib/envctl/legacy-archives/postgres-ruvector/fixture-success"
(
  postgres_cluster_state() { [[ -f "$running_marker" ]]; }
  postgres_run_scratch_parity() {
    [[ -e "$pg/pg17-rw" && -L "$pg/pg17" ]] || return 1
    printf 'scratch\n' >>"$activation_log"
  }
  postgres_pg_ctl_start() { printf 'start\n' >>"$activation_log"; touch "$running_marker"; }
  postgres_validate_running_at() { printf 'validate\n' >>"$activation_log"; [[ -f "$running_marker" ]]; }
  postgres_verify_queries_at() { printf 'verify\n' >>"$activation_log"; }
  postgres_archive_destination() { printf '%s\n' "$archive"; }
  postgres_activate
  postgres_activate
)
assert_order "$activation_log" scratch start
[[ "$(grep -c '^scratch$' "$activation_log")" -eq 1 ]] || fail 'scratch gate was not one-time'
[[ "$(grep -c '^start$' "$activation_log")" -eq 1 ]] || fail 'activation was not idempotent'
[[ ! -e "$pg/pg17-rw" && ! -L "$pg/pg17" && ! -L "$pg/pg17-config" ]] ||
  fail 'legacy namespace remained active after successful activation'
[[ -e "$archive/pg17-rw/bin/postgres" && -L "$archive/pg17" && -L "$archive/pg17-config" ]] ||
  fail 'legacy namespace was not archived intact'
grep -Fq "ruvector_source=$POSTGRES_RUVECTOR_SOURCE" "$archive/.envctl-retirement" ||
  fail 'legacy archive lacks its source identity'
[[ ! -e "$FAKE_INITDB_LOG" ]] || fail 'activation invoked the forbidden profile initdb command'
[[ "$(<"$data/data-marker")" == preserve-pgdata ]] || fail 'activation changed PGDATA contents'

# A post-start validation failure stops the newly started server and restores every legacy name.
reset_fixture
mkdir -p "$pg/pg17-rw/bin"
printf 'legacy-copy\n' >"$pg/pg17-rw/bin/postgres"
ln -s "$combined" "$pg/pg17"
rollback_log="$tmp/rollback.log"
running_marker="$case_root/running"
if (
  postgres_cluster_state() { return 1; }
  postgres_run_scratch_parity() { printf 'scratch\n' >>"$rollback_log"; }
  postgres_pg_ctl_start() { printf 'start\n' >>"$rollback_log"; touch "$running_marker"; }
  postgres_validate_running_at() { printf 'reject\n' >>"$rollback_log"; return 1; }
  postgres_stop_owned_cluster() { printf 'stop\n' >>"$rollback_log"; rm -f "$running_marker"; }
  postgres_activate
); then
  fail 'failed live activation unexpectedly succeeded'
fi
assert_order "$rollback_log" scratch start
assert_order "$rollback_log" start reject
assert_order "$rollback_log" reject stop
[[ -e "$pg/pg17-rw/bin/postgres" && -L "$pg/pg17" ]] ||
  fail 'activation failure did not roll back the legacy namespace'
[[ ! -e "$running_marker" ]] || fail 'failed activation left the new server running'
if compgen -G "$pg/.envctl-postgres-ruvector-retire.*" >/dev/null; then
  fail 'rollback left a legacy retirement transaction behind'
fi

# remove is idempotent and stop-only; both data namespaces survive byte-for-byte.
reset_fixture
running_marker="$case_root/running"
touch "$running_marker"
remove_log="$tmp/remove.log"
(
  postgres_cluster_state() { [[ -f "$running_marker" ]]; }
  postgres_stop_owned_cluster() { printf 'stop\n' >>"$remove_log"; rm -f "$running_marker"; }
  postgres_remove
  postgres_remove
)
[[ "$(grep -c '^stop$' "$remove_log")" -eq 1 ]] || fail 'remove is not idempotent stop-only'
[[ "$(<"$data/data-marker")" == preserve-pgdata ]] || fail 'remove changed PGDATA'
[[ "$(<"$meta/var/lib/ruvector/data-marker")" == preserve-ruvector ]] ||
  fail 'remove changed ruvector data'
[[ ! -e "$FAKE_INITDB_LOG" ]] || fail 'lifecycle invoked initdb'

echo 'PASS: postgres-ruvector enforces the Yazelix profile, exact extension/data parity, hostile-state refusal, transactional legacy retirement, and data-preserving idempotence'
