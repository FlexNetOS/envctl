#!/usr/bin/env bash
# postgres-ruvector lifecycle — profile-owned PostgreSQL 17.10, existing-data only.
#
# This file is intentionally sourceable.  Tests may replace the small operating-system seams
# below, but the executable component path accepts no ambient path or command overrides.

POSTGRES_REQUIRED_VERSION="17.10"
POSTGRES_REQUIRED_MAJOR="17"
POSTGRES_EXTENSION_VERSION="0.3.0"
POSTGRES_RUVECTOR_SOURCE="crates.io:ruvector-postgres:2.0.5:sha256:052dadb088cb26e640833072416ad59a2b2437dbb534f7effe197e30261fe1d7"
POSTGRES_VECTOR_EPSILON="1e-9"
POSTGRES_DATABASE="ruvector"
POSTGRES_PORT="5432"

POSTGRES_LEGACY_TXN=""
POSTGRES_LEGACY_PATHS=()

postgres_error() {
  printf 'postgres-ruvector: %s\n' "$*" >&2
}

# Sourceable test seams.  Production callers use these exact definitions.
postgres_store_root() { printf '/nix/store\n'; }
postgres_readlink() { /usr/bin/readlink -- "$1"; }
postgres_readlink_f() { /usr/bin/readlink -f -- "$1"; }
postgres_pid_alive() { kill -0 "$1" 2>/dev/null; }
postgres_pid_executable() { /usr/bin/readlink -f -- "/proc/$1/exe"; }
postgres_pid_argv0() {
  /usr/bin/tr '\0' '\n' <"/proc/$1/cmdline" | /usr/bin/sed -n '1p'
}
postgres_socket_is_socket() { [[ -S "$1" ]]; }
postgres_scratch_port() { printf '%s\n' "$((55000 + ($$ % 9000)))"; }
postgres_scratch_root_create() {
  /usr/bin/mkdir -p -- "$META_ROOT/var/tmp"
  /usr/bin/chmod 0700 -- "$META_ROOT/var/tmp"
  /usr/bin/mktemp -d "$META_ROOT/var/tmp/postgres-ruvector-parity.XXXXXX"
}
postgres_copy_data_tree() {
  /usr/bin/cp -a --reflink=auto -- "$1/." "$2/"
}
postgres_remove_scratch_tree() { /usr/bin/rm -rf -- "$1"; }
postgres_archive_destination() {
  printf '%s/var/lib/envctl/legacy-archives/postgres-ruvector/%s.%s\n' \
    "$META_ROOT" "$(/usr/bin/date -u +%Y%m%dT%H%M%SZ)" "$$"
}

postgres_is_under() {
  local root="$1" path="$2"
  [[ "$path" == "$root" || "$path" == "$root/"* ]]
}

postgres_profile_validate() {
  local real_home store_root selector frontend_target selector_target generation generation_from_selector
  local command command_path command_real version pg_config_version marker control_count module_count
  local artifact

  real_home="${ENVCTL_REAL_HOME:-}"
  [[ "$real_home" == /* ]] || {
    postgres_error 'ENVCTL_REAL_HOME must be an absolute path'
    return 1
  }

  store_root="$(postgres_store_root)" || return 1
  store_root="$(postgres_readlink_f "$store_root")" || {
    postgres_error 'cannot resolve the Nix store root'
    return 1
  }
  [[ -d "$store_root" ]] || {
    postgres_error "Nix store root is missing: $store_root"
    return 1
  }

  POSTGRES_PROFILE="$real_home/.nix-profile"
  selector="$real_home/.local/state/nix/profiles/profile"
  [[ -L "$POSTGRES_PROFILE" ]] || {
    postgres_error "profile front door is not a symlink: $POSTGRES_PROFILE"
    return 1
  }
  frontend_target="$(postgres_readlink "$POSTGRES_PROFILE")" || return 1
  [[ "$frontend_target" == "$selector" ]] || {
    postgres_error "profile front door must target $selector exactly"
    return 1
  }
  [[ -L "$selector" ]] || {
    postgres_error "profile generation selector is not a symlink: $selector"
    return 1
  }
  selector_target="$(postgres_readlink "$selector")" || return 1
  [[ "$selector_target" =~ ^profile-[1-9][0-9]*-link$ ]] || {
    postgres_error "profile selector has an invalid generation target: $selector_target"
    return 1
  }

  generation="$(postgres_readlink_f "$POSTGRES_PROFILE")" || {
    postgres_error 'profile front door does not resolve'
    return 1
  }
  generation_from_selector="$(postgres_readlink_f "$selector")" || return 1
  [[ "$generation" == "$generation_from_selector" ]] || {
    postgres_error 'profile front door and selector resolve to different generations'
    return 1
  }
  postgres_is_under "$store_root" "$generation" && [[ "$generation" == *-profile ]] || {
    postgres_error "profile generation is not a Nix profile output: $generation"
    return 1
  }
  [[ -d "$generation" ]] || {
    postgres_error "profile generation is missing: $generation"
    return 1
  }

  # Commands are consumed through the stable Yazelix profile front door, never a resolved store
  # hash or a META_ROOT copy.
  POSTGRES_TOOLBIN="$real_home/.nix-profile/toolbin"
  for command in postgres psql pg_ctl pg_isready pg_config; do
    command_path="$POSTGRES_TOOLBIN/$command"
    [[ -x "$command_path" ]] || {
      postgres_error "profile command is missing or non-executable: $command_path"
      return 1
    }
    command_real="$(postgres_readlink_f "$command_path")" || {
      postgres_error "profile command does not resolve: $command_path"
      return 1
    }
    postgres_is_under "$store_root" "$command_real" || {
      postgres_error "profile command escapes the Nix store: $command_path -> $command_real"
      return 1
    }
  done

  POSTGRES_POSTGRES="$POSTGRES_TOOLBIN/postgres"
  POSTGRES_PSQL="$POSTGRES_TOOLBIN/psql"
  POSTGRES_PG_CTL="$POSTGRES_TOOLBIN/pg_ctl"
  POSTGRES_PG_ISREADY="$POSTGRES_TOOLBIN/pg_isready"
  POSTGRES_PG_CONFIG="$POSTGRES_TOOLBIN/pg_config"
  POSTGRES_POSTGRES_REAL="$(postgres_readlink_f "$POSTGRES_POSTGRES")" || return 1
  case "$POSTGRES_POSTGRES_REAL" in
    "$store_root"/*-flexnetos-foundation-postgresql-frontdoors-17.10-ruvector-0.3.0/bin/postgres) ;;
    *)
      postgres_error "profile postgres is not the exact combined-package frontdoor: $POSTGRES_POSTGRES_REAL"
      return 1
      ;;
  esac

  if ! version="$($POSTGRES_POSTGRES --version 2>/dev/null)"; then
    postgres_error 'profile postgres --version failed'
    return 1
  fi
  [[ "$version" == "postgres (PostgreSQL) $POSTGRES_REQUIRED_VERSION" ]] || {
    postgres_error "profile PostgreSQL must be exactly $POSTGRES_REQUIRED_VERSION (got: $version)"
    return 1
  }
  if ! pg_config_version="$($POSTGRES_PG_CONFIG --version 2>/dev/null)"; then
    postgres_error 'profile pg_config --version failed'
    return 1
  fi
  [[ "$pg_config_version" == "PostgreSQL $POSTGRES_REQUIRED_VERSION" ]] || {
    postgres_error "profile pg_config must report PostgreSQL $POSTGRES_REQUIRED_VERSION"
    return 1
  }

  if ! POSTGRES_BINDIR="$($POSTGRES_PG_CONFIG --bindir 2>/dev/null)" ||
    ! POSTGRES_PKGLIBDIR="$($POSTGRES_PG_CONFIG --pkglibdir 2>/dev/null)" ||
    ! POSTGRES_SHAREDIR="$($POSTGRES_PG_CONFIG --sharedir 2>/dev/null)" ||
    ! POSTGRES_INCLUDEDIR_SERVER="$($POSTGRES_PG_CONFIG --includedir-server 2>/dev/null)"; then
    postgres_error 'profile pg_config could not report its combined output paths'
    return 1
  fi

  for command_path in "$POSTGRES_BINDIR" "$POSTGRES_PKGLIBDIR" "$POSTGRES_SHAREDIR" "$POSTGRES_INCLUDEDIR_SERVER"; do
    [[ "$command_path" == /* && -d "$command_path" ]] || {
      postgres_error "pg_config returned a missing or non-absolute directory: $command_path"
      return 1
    }
    command_real="$(postgres_readlink_f "$command_path")" || return 1
    postgres_is_under "$store_root" "$command_real" || {
      postgres_error "pg_config path escapes the Nix store: $command_path"
      return 1
    }
  done

  [[ -x "$POSTGRES_BINDIR/postgres" ]] || {
    postgres_error 'pg_config --bindir does not expose postgres'
    return 1
  }
  POSTGRES_SERVER_REAL="$(postgres_readlink_f "$POSTGRES_BINDIR/postgres")" || return 1
  postgres_is_under "$store_root" "$POSTGRES_SERVER_REAL" || {
    postgres_error 'pg_config --bindir/postgres escapes the Nix store'
    return 1
  }

  [[ -s "$POSTGRES_INCLUDEDIR_SERVER/postgres.h" ]] || {
    postgres_error 'combined profile does not expose PostgreSQL server development headers'
    return 1
  }
  [[ -s "$POSTGRES_PKGLIBDIR/ruvector.so" ]] || {
    postgres_error 'combined profile does not expose ruvector.so in pg_config --pkglibdir'
    return 1
  }
  POSTGRES_EXTENSION_DIR="$POSTGRES_SHAREDIR/extension"
  [[ -s "$POSTGRES_EXTENSION_DIR/.envctl-ruvector-source" ]] || {
    postgres_error 'combined profile is missing its .envctl-ruvector-source marker'
    return 1
  }
  marker="$(<"$POSTGRES_EXTENSION_DIR/.envctl-ruvector-source")"
  [[ "$marker" == "$POSTGRES_RUVECTOR_SOURCE" ]] || {
    postgres_error 'combined profile ruvector source marker does not match the pinned registry artifact'
    return 1
  }
  [[ -s "$POSTGRES_EXTENSION_DIR/ruvector.control" ]] || {
    postgres_error 'combined profile is missing ruvector.control'
    return 1
  }
  [[ -s "$POSTGRES_EXTENSION_DIR/ruvector--$POSTGRES_EXTENSION_VERSION.sql" ]] || {
    postgres_error "combined profile is missing ruvector--$POSTGRES_EXTENSION_VERSION.sql"
    return 1
  }
  for artifact in \
    "$POSTGRES_INCLUDEDIR_SERVER/postgres.h" \
    "$POSTGRES_PKGLIBDIR/ruvector.so" \
    "$POSTGRES_EXTENSION_DIR/.envctl-ruvector-source" \
    "$POSTGRES_EXTENSION_DIR/ruvector.control" \
    "$POSTGRES_EXTENSION_DIR/ruvector--$POSTGRES_EXTENSION_VERSION.sql"; do
    command_real="$(postgres_readlink_f "$artifact")" || return 1
    postgres_is_under "$store_root" "$command_real" || {
      postgres_error "combined profile artifact escapes the Nix store: $artifact"
      return 1
    }
  done
  control_count="$(/usr/bin/grep -Ec \
    "^[[:space:]]*default_version[[:space:]]*=[[:space:]]*'$POSTGRES_EXTENSION_VERSION'[[:space:]]*$" \
    "$POSTGRES_EXTENSION_DIR/ruvector.control" || true)"
  module_count="$(/usr/bin/grep -Ec \
    "^[[:space:]]*module_pathname[[:space:]]*=[[:space:]]*'\\\$libdir/ruvector'[[:space:]]*$" \
    "$POSTGRES_EXTENSION_DIR/ruvector.control" || true)"
  [[ "$control_count" == 1 && "$module_count" == 1 ]] || {
    postgres_error "ruvector.control does not pin version 0.3.0 and \$libdir/ruvector exactly once"
    return 1
  }
}

postgres_data_validate() {
  local pg_version data_owner current_owner data_mode
  POSTGRES_META_ROOT="${META_ROOT:-}"
  [[ "$POSTGRES_META_ROOT" == /* && -d "$POSTGRES_META_ROOT" ]] || {
    postgres_error 'META_ROOT must be an existing absolute directory'
    return 1
  }
  POSTGRES_ROOT="$POSTGRES_META_ROOT/var/lib/postgresql"
  POSTGRES_DATA="$POSTGRES_ROOT/$POSTGRES_REQUIRED_MAJOR"
  [[ -d "$POSTGRES_ROOT" && ! -L "$POSTGRES_ROOT" ]] || {
    postgres_error "PostgreSQL state root is missing or is a symlink: $POSTGRES_ROOT"
    return 1
  }
  [[ -d "$POSTGRES_DATA" && ! -L "$POSTGRES_DATA" ]] || {
    postgres_error "existing PostgreSQL data directory is required: $POSTGRES_DATA"
    return 1
  }
  [[ -f "$POSTGRES_DATA/PG_VERSION" && ! -L "$POSTGRES_DATA/PG_VERSION" ]] || {
    postgres_error 'PGDATA must contain a regular PG_VERSION file'
    return 1
  }
  pg_version="$(<"$POSTGRES_DATA/PG_VERSION")"
  [[ "$pg_version" == "$POSTGRES_REQUIRED_MAJOR" ]] || {
    postgres_error "PGDATA must already be PostgreSQL major $POSTGRES_REQUIRED_MAJOR (got: $pg_version)"
    return 1
  }
  data_owner="$(/usr/bin/stat -c '%u' "$POSTGRES_DATA")" || return 1
  current_owner="$(/usr/bin/id -u)" || return 1
  [[ "$data_owner" == "$current_owner" ]] || {
    postgres_error "PGDATA must be owned by the lifecycle user (owner=$data_owner user=$current_owner)"
    return 1
  }
  data_mode="$(/usr/bin/stat -c '%a' "$POSTGRES_DATA")" || return 1
  [[ "$data_mode" == 700 || "$data_mode" == 750 ]] || {
    postgres_error "PGDATA permissions must be 0700 or 0750 (got: $data_mode)"
    return 1
  }
}

postgres_query_at() {
  local socket_dir="$1" port="$2" database="$3" sql="$4"
  /usr/bin/env \
    -u PGHOST -u PGHOSTADDR -u PGPORT -u PGDATABASE -u PGUSER -u PGSERVICE \
    -u PGSERVICEFILE -u PGPASSFILE -u PGOPTIONS \
    "$POSTGRES_PSQL" -X -w -qAt --set=ON_ERROR_STOP=1 \
    -h "$socket_dir" -p "$port" -d "$database" -c "$sql"
}

postgres_verify_queries_at() {
  local socket_dir="$1" port="$2" ext codebase episodes missing distance_record distance distance_ok extra
  if ! ext="$(postgres_query_at "$socket_dir" "$port" "$POSTGRES_DATABASE" \
    "SELECT extversion FROM pg_extension WHERE extname = 'ruvector';" 2>/dev/null)"; then
    postgres_error 'ruvector extension-version query failed'
    return 1
  fi
  [[ "$ext" == "$POSTGRES_EXTENSION_VERSION" ]] || {
    postgres_error "ruvector extension must be exactly $POSTGRES_EXTENSION_VERSION (got: ${ext:-missing})"
    return 1
  }

  codebase="$(postgres_query_at "$socket_dir" "$port" "$POSTGRES_DATABASE" \
    'SELECT count(*) FROM codebase;' 2>/dev/null)" || {
    postgres_error 'codebase lane is missing or unreadable'
    return 1
  }
  episodes="$(postgres_query_at "$socket_dir" "$port" "$POSTGRES_DATABASE" \
    'SELECT count(*) FROM episodes;' 2>/dev/null)" || {
    postgres_error 'episodes lane is missing or unreadable'
    return 1
  }
  [[ "$codebase" =~ ^[1-9][0-9]*$ && "$episodes" =~ ^[1-9][0-9]*$ ]] || {
    postgres_error "ruvector lanes must both be nonempty (codebase=$codebase episodes=$episodes)"
    return 1
  }

  missing="$(postgres_query_at "$socket_dir" "$port" "$POSTGRES_DATABASE" \
    'SELECT count(*) FROM codebase WHERE embedding_minilm IS NULL;' 2>/dev/null)" || {
    postgres_error 'MiniLM embedding-completeness query failed'
    return 1
  }
  [[ "$missing" == 0 ]] || {
    postgres_error "codebase contains $missing rows without MiniLM embeddings"
    return 1
  }

  distance_record="$(postgres_query_at "$socket_dir" "$port" "$POSTGRES_DATABASE" \
    "WITH sample AS ( \
       SELECT embedding_minilm <=> embedding_minilm AS distance \
       FROM codebase WHERE embedding_minilm IS NOT NULL LIMIT 1 \
     ) \
     SELECT distance, distance >= 0.0 AND distance <= $POSTGRES_VECTOR_EPSILON \
     FROM sample;" \
    2>/dev/null)" || {
    postgres_error 'real ruvector <=> query failed'
    return 1
  }
  IFS='|' read -r distance distance_ok extra <<<"$distance_record"
  [[ -n "$distance" && "$distance_ok" == t && -z "$extra" ]] || {
    postgres_error \
      "real ruvector <=> self-distance was non-finite, negative, or above $POSTGRES_VECTOR_EPSILON (got: ${distance:-missing})"
    return 1
  }
}

postgres_validate_pid_owner() {
  local data_dir="$1" pid_file="$1/postmaster.pid" pid pid_data pid_exe pid_argv0 data_real pid_data_real
  [[ -f "$pid_file" && ! -L "$pid_file" ]] || {
    postgres_error "running cluster lacks a regular postmaster.pid: $pid_file"
    return 1
  }
  pid="$(/usr/bin/sed -n '1p' "$pid_file")"
  pid_data="$(/usr/bin/sed -n '2p' "$pid_file")"
  [[ "$pid" =~ ^[1-9][0-9]*$ && "$pid" -gt 1 ]] || {
    postgres_error "postmaster.pid contains an invalid PID: $pid"
    return 1
  }
  [[ "$pid_data" == /* ]] || {
    postgres_error 'postmaster.pid does not contain an absolute PGDATA path'
    return 1
  }
  postgres_pid_alive "$pid" || {
    postgres_error "postmaster.pid is stale; refusing to remove it or start another server (PID $pid)"
    return 1
  }
  pid_exe="$(postgres_pid_executable "$pid")" || {
    postgres_error "cannot resolve executable for PostgreSQL PID $pid"
    return 1
  }
  [[ "$pid_exe" == "$POSTGRES_SERVER_REAL" ]] || {
    postgres_error "PID $pid is not the server executable selected by profile pg_config ($pid_exe)"
    return 1
  }
  pid_argv0="$(postgres_pid_argv0 "$pid")" || {
    postgres_error "cannot read argv[0] for PostgreSQL PID $pid"
    return 1
  }
  [[ "$pid_argv0" == "$POSTGRES_BINDIR/postgres" ]] || {
    postgres_error "PID $pid was not launched through the combined PostgreSQL package ($pid_argv0)"
    return 1
  }
  data_real="$(postgres_readlink_f "$data_dir")" || return 1
  pid_data_real="$(postgres_readlink_f "$pid_data")" || {
    postgres_error "postmaster.pid PGDATA does not resolve: $pid_data"
    return 1
  }
  [[ "$pid_data_real" == "$data_real" ]] || {
    postgres_error "PID $pid owns a different PGDATA: $pid_data"
    return 1
  }
}

postgres_validate_running_at() {
  local data_dir="$1" socket_dir="$2" port="$3" socket data_setting socket_setting listen_setting port_setting
  postgres_validate_pid_owner "$data_dir" || return 1
  socket="$socket_dir/.s.PGSQL.$port"
  postgres_socket_is_socket "$socket" || {
    postgres_error "expected PostgreSQL socket is absent or not a socket: $socket"
    return 1
  }
  "$POSTGRES_PG_ISREADY" -q -h "$socket_dir" -p "$port" -d "$POSTGRES_DATABASE" || {
    postgres_error 'profile pg_isready rejected the expected socket server'
    return 1
  }

  data_setting="$(postgres_query_at "$socket_dir" "$port" "$POSTGRES_DATABASE" \
    'SHOW data_directory;' 2>/dev/null)" || {
    postgres_error 'cannot query PostgreSQL data_directory through the expected socket'
    return 1
  }
  [[ "$(postgres_readlink_f "$data_setting" 2>/dev/null || true)" == \
    "$(postgres_readlink_f "$data_dir" 2>/dev/null || true)" ]] || {
    postgres_error "socket server owns different PGDATA: $data_setting"
    return 1
  }
  socket_setting="$(postgres_query_at "$socket_dir" "$port" "$POSTGRES_DATABASE" \
    'SHOW unix_socket_directories;' 2>/dev/null)" || return 1
  [[ "$socket_setting" == "$socket_dir" ]] || {
    postgres_error "socket server advertises unexpected socket directories: $socket_setting"
    return 1
  }
  listen_setting="$(postgres_query_at "$socket_dir" "$port" "$POSTGRES_DATABASE" \
    'SHOW listen_addresses;' 2>/dev/null)" || return 1
  [[ -z "$listen_setting" ]] || {
    postgres_error "PostgreSQL must be socket-only (listen_addresses=$listen_setting)"
    return 1
  }
  port_setting="$(postgres_query_at "$socket_dir" "$port" "$POSTGRES_DATABASE" \
    'SHOW port;' 2>/dev/null)" || return 1
  [[ "$port_setting" == "$port" ]] || {
    postgres_error "socket server advertises unexpected port: $port_setting"
    return 1
  }
}

# Return 0 for a validated running cluster, 1 for cleanly stopped, and 2 for hostile/ambiguous.
postgres_cluster_state() {
  local pid_file="$POSTGRES_DATA/postmaster.pid"
  local socket="$POSTGRES_ROOT/.s.PGSQL.$POSTGRES_PORT"
  local lock="$socket.lock"
  if [[ ! -e "$pid_file" && ! -L "$pid_file" ]]; then
    if [[ -e "$socket" || -L "$socket" || -e "$lock" || -L "$lock" ]]; then
      postgres_error 'socket namespace is occupied without the expected postmaster.pid'
      return 2
    fi
    return 1
  fi
  if ! postgres_validate_running_at "$POSTGRES_DATA" "$POSTGRES_ROOT" "$POSTGRES_PORT"; then
    return 2
  fi
  return 0
}

postgres_pg_ctl_start() {
  local data_dir="$1" socket_dir="$2" port="$3" log_file="$4"
  "$POSTGRES_PG_CTL" -D "$data_dir" -w -t 30 \
    -o "-p $port -k $socket_dir -c listen_addresses=" \
    -l "$log_file" start
}

postgres_pg_ctl_stop() {
  local data_dir="$1"
  "$POSTGRES_PG_CTL" -D "$data_dir" -w -t 30 -m fast stop
}

postgres_stop_owned_cluster() {
  local data_dir="$1"
  postgres_validate_pid_owner "$data_dir" || return 1
  postgres_pg_ctl_stop "$data_dir" || {
    postgres_error "profile pg_ctl could not stop $data_dir"
    return 1
  }
  [[ ! -e "$data_dir/postmaster.pid" && ! -L "$data_dir/postmaster.pid" ]] || {
    postgres_error "postmaster.pid remains after pg_ctl stop: $data_dir"
    return 1
  }
}

postgres_stop_started_if_present() {
  local data_dir="$1" socket_dir="$2" port="$3"
  if [[ -e "$data_dir/postmaster.pid" || -L "$data_dir/postmaster.pid" ]]; then
    postgres_stop_owned_cluster "$data_dir"
    return
  fi
  if [[ -e "$socket_dir/.s.PGSQL.$port" || -L "$socket_dir/.s.PGSQL.$port" ||
    -e "$socket_dir/.s.PGSQL.$port.lock" || -L "$socket_dir/.s.PGSQL.$port.lock" ]]; then
    postgres_error 'failed start left an unowned socket namespace; refusing an unproven stop'
    return 1
  fi
}

postgres_collect_legacy_paths() {
  local path base target store_root store_real
  POSTGRES_LEGACY_PATHS=()
  store_root="$(postgres_store_root)" || return 1
  store_real="$(postgres_readlink_f "$store_root")" || return 1
  for path in "$POSTGRES_ROOT"/pg17*; do
    [[ -e "$path" || -L "$path" ]] || continue
    base="${path##*/}"
    if [[ "$base" == pg17-rw ]]; then
      [[ -d "$path" && ! -L "$path" ]] || {
        postgres_error "legacy pg17-rw must be the known copied directory, not another node type: $path"
        return 1
      }
    else
      [[ -L "$path" ]] || {
        postgres_error "unknown non-symlink pg17 legacy path: $path"
        return 1
      }
      target="$(postgres_readlink_f "$path")" || {
        postgres_error "legacy pg17 symlink is broken: $path"
        return 1
      }
      postgres_is_under "$store_real" "$target" || {
        postgres_error "legacy pg17 symlink does not target the Nix store: $path -> $target"
        return 1
      }
    fi
    POSTGRES_LEGACY_PATHS+=("$path")
  done
}

postgres_assert_no_legacy_paths() {
  postgres_collect_legacy_paths || return 1
  [[ "${#POSTGRES_LEGACY_PATHS[@]}" -eq 0 ]] || {
    postgres_error "legacy copied/store-bound PostgreSQL namespace remains active: ${POSTGRES_LEGACY_PATHS[*]}"
    return 1
  }
}

postgres_run_scratch_parity() {
  local scratch scratch_data scratch_socket scratch_log port started=0 rc=0
  if [[ -d "$POSTGRES_DATA/pg_tblspc" ]] &&
    [[ -n "$(/usr/bin/find "$POSTGRES_DATA/pg_tblspc" -mindepth 1 -maxdepth 1 -print -quit)" ]]; then
    postgres_error 'scratch parity refuses PGDATA with external tablespaces'
    return 1
  fi
  scratch="$(postgres_scratch_root_create)" || {
    postgres_error 'could not allocate private scratch-parity directory'
    return 1
  }
  scratch_data="$scratch/data"
  scratch_socket="$scratch/socket"
  scratch_log="$scratch/postgres.log"
  port="$(postgres_scratch_port)" || rc=1
  if [[ "$port" =~ ^[0-9]+$ && "$port" -ge 49152 && "$port" -le 65535 ]]; then
    /usr/bin/mkdir -m 0700 -- "$scratch_data" "$scratch_socket" || rc=1
  else
    postgres_error "scratch port is not private/high: $port"
    rc=1
  fi
  if [[ "$rc" -eq 0 ]] && ! postgres_copy_data_tree "$POSTGRES_DATA" "$scratch_data"; then
    postgres_error 'could not make the private PGDATA scratch copy'
    rc=1
  fi
  if [[ "$rc" -eq 0 ]] && ! postgres_pg_ctl_start "$scratch_data" "$scratch_socket" "$port" "$scratch_log"; then
    postgres_error 'profile PostgreSQL could not start the private scratch copy'
    rc=1
  elif [[ "$rc" -eq 0 ]]; then
    started=1
  fi
  if [[ "$rc" -eq 0 ]] &&
    ! postgres_validate_running_at "$scratch_data" "$scratch_socket" "$port"; then
    postgres_error 'scratch server failed profile/PGDATA/socket ownership validation'
    rc=1
  fi
  if [[ "$rc" -eq 0 ]] && ! postgres_verify_queries_at "$scratch_socket" "$port"; then
    postgres_error 'scratch server failed ruvector data parity'
    rc=1
  fi
  if [[ "$started" -eq 1 ]] && ! postgres_stop_owned_cluster "$scratch_data"; then
    postgres_error 'could not safely stop the scratch server; scratch directory retained'
    return 1
  fi
  if [[ "$scratch" == "$META_ROOT/var/tmp/postgres-ruvector-parity."* ]]; then
    postgres_remove_scratch_tree "$scratch" || rc=1
  else
    postgres_error "refusing to remove unexpected scratch path: $scratch"
    return 1
  fi
  return "$rc"
}

postgres_restore_legacy_namespace() {
  local path base rc=0
  [[ -n "$POSTGRES_LEGACY_TXN" && -d "$POSTGRES_LEGACY_TXN" ]] || return 0
  for path in "${POSTGRES_LEGACY_PATHS[@]}"; do
    base="${path##*/}"
    [[ -e "$POSTGRES_LEGACY_TXN/$base" || -L "$POSTGRES_LEGACY_TXN/$base" ]] || continue
    if [[ -e "$path" || -L "$path" ]]; then
      postgres_error "cannot roll back legacy namespace; destination reappeared: $path"
      rc=1
      continue
    fi
    /usr/bin/mv -- "$POSTGRES_LEGACY_TXN/$base" "$path" || rc=1
  done
  if [[ "$rc" -eq 0 ]]; then
    /usr/bin/rm -f -- "$POSTGRES_LEGACY_TXN/.envctl-retirement"
    /usr/bin/rmdir -- "$POSTGRES_LEGACY_TXN" 2>/dev/null || true
    POSTGRES_LEGACY_TXN=""
  fi
  return "$rc"
}

postgres_stage_legacy_namespace() {
  local path base
  [[ "${#POSTGRES_LEGACY_PATHS[@]}" -gt 0 ]] || return 0
  POSTGRES_LEGACY_TXN="$POSTGRES_ROOT/.envctl-postgres-ruvector-retire.$$"
  [[ ! -e "$POSTGRES_LEGACY_TXN" && ! -L "$POSTGRES_LEGACY_TXN" ]] || {
    postgres_error "legacy retirement transaction already exists: $POSTGRES_LEGACY_TXN"
    return 1
  }
  /usr/bin/mkdir -m 0700 -- "$POSTGRES_LEGACY_TXN" || return 1
  for path in "${POSTGRES_LEGACY_PATHS[@]}"; do
    base="${path##*/}"
    if ! /usr/bin/mv -- "$path" "$POSTGRES_LEGACY_TXN/$base"; then
      postgres_error "could not stage legacy namespace path: $path"
      postgres_restore_legacy_namespace || true
      return 1
    fi
  done
}

postgres_finalize_legacy_archive() {
  local destination parent
  [[ -n "$POSTGRES_LEGACY_TXN" ]] || return 0
  destination="$(postgres_archive_destination)" || return 1
  parent="${destination%/*}"
  [[ "$destination" == "$META_ROOT/var/lib/envctl/legacy-archives/postgres-ruvector/"* ]] || {
    postgres_error "archive seam returned an unsafe destination: $destination"
    return 1
  }
  [[ ! -e "$destination" && ! -L "$destination" ]] || {
    postgres_error "legacy archive destination already exists: $destination"
    return 1
  }
  printf 'postgres=%s\nruvector_source=%s\n' \
    "$POSTGRES_REQUIRED_VERSION" "$POSTGRES_RUVECTOR_SOURCE" \
    >"$POSTGRES_LEGACY_TXN/.envctl-retirement"
  /usr/bin/mkdir -p -- "$parent" || return 1
  /usr/bin/chmod 0700 -- "$parent" || return 1
  /usr/bin/mv -- "$POSTGRES_LEGACY_TXN" "$destination" || return 1
  POSTGRES_LEGACY_TXN=""
  printf 'postgres-ruvector: archived retired legacy namespace at %s\n' "$destination"
}

postgres_activate() {
  local state stop_rc=0
  postgres_profile_validate || return 1
  postgres_data_validate || return 1

  if postgres_cluster_state; then
    state=0
  else
    state=$?
  fi
  case "$state" in
    0)
      postgres_assert_no_legacy_paths || return 1
      postgres_verify_queries_at "$POSTGRES_ROOT" "$POSTGRES_PORT" || return 1
      printf 'postgres-ruvector: already healthy on %s/.s.PGSQL.%s\n' "$POSTGRES_ROOT" "$POSTGRES_PORT"
      return 0
      ;;
    1) ;;
    *) return 1 ;;
  esac

  postgres_collect_legacy_paths || return 1
  if [[ "${#POSTGRES_LEGACY_PATHS[@]}" -gt 0 ]]; then
    postgres_run_scratch_parity || return 1
    postgres_stage_legacy_namespace || return 1
  fi

  if ! postgres_pg_ctl_start "$POSTGRES_DATA" "$POSTGRES_ROOT" "$POSTGRES_PORT" \
    "$POSTGRES_ROOT/pg_ctl.log"; then
    postgres_stop_started_if_present "$POSTGRES_DATA" "$POSTGRES_ROOT" "$POSTGRES_PORT" || stop_rc=$?
    postgres_restore_legacy_namespace || true
    postgres_error "profile pg_ctl failed to activate the existing cluster (safe-stop status: $stop_rc)"
    return 1
  fi
  if ! postgres_validate_running_at "$POSTGRES_DATA" "$POSTGRES_ROOT" "$POSTGRES_PORT" ||
    ! postgres_verify_queries_at "$POSTGRES_ROOT" "$POSTGRES_PORT"; then
    postgres_stop_owned_cluster "$POSTGRES_DATA" || stop_rc=$?
    postgres_restore_legacy_namespace || true
    if [[ "$stop_rc" -ne 0 ]]; then
      postgres_error 'activation failed and the newly started profile server could not be stopped safely'
    else
      postgres_error 'activation failed; newly started server was stopped and legacy namespace restored'
    fi
    return 1
  fi
  if ! postgres_finalize_legacy_archive; then
    postgres_stop_owned_cluster "$POSTGRES_DATA" || stop_rc=$?
    postgres_restore_legacy_namespace || true
    postgres_error "activation archive finalization failed (stop status: $stop_rc)"
    return 1
  fi
  printf 'postgres-ruvector: healthy profile-owned socket server at %s/.s.PGSQL.%s\n' \
    "$POSTGRES_ROOT" "$POSTGRES_PORT"
}

postgres_detect() {
  postgres_profile_validate || return 1
  postgres_data_validate || return 1
  postgres_assert_no_legacy_paths || return 1
  printf 'postgres-ruvector: profile PostgreSQL %s and existing PGDATA %s are present\n' \
    "$POSTGRES_REQUIRED_VERSION" "$POSTGRES_DATA"
}

postgres_verify() {
  local state
  postgres_profile_validate || return 1
  postgres_data_validate || return 1
  postgres_assert_no_legacy_paths || return 1
  if postgres_cluster_state; then state=0; else state=$?; fi
  [[ "$state" -eq 0 ]] || {
    postgres_error 'cluster is not a validated profile-owned socket server'
    return 1
  }
  postgres_verify_queries_at "$POSTGRES_ROOT" "$POSTGRES_PORT" || return 1
  printf 'postgres-ruvector: healthy (PostgreSQL %s, extension %s, lanes complete, <=> live)\n' \
    "$POSTGRES_REQUIRED_VERSION" "$POSTGRES_EXTENSION_VERSION"
}

postgres_remove() {
  local state
  postgres_profile_validate || return 1
  postgres_data_validate || return 1
  if postgres_cluster_state; then state=0; else state=$?; fi
  case "$state" in
    0)
      postgres_stop_owned_cluster "$POSTGRES_DATA" || return 1
      printf 'postgres-ruvector: stopped profile-owned server; PGDATA and vector data untouched\n'
      ;;
    1)
      printf 'postgres-ruvector: already stopped; PGDATA and vector data untouched\n'
      ;;
    *)
      postgres_error 'refusing remove because PostgreSQL PID/socket ownership is ambiguous'
      return 1
      ;;
  esac
}

postgres_main() {
  export PATH=/usr/bin:/bin
  umask 077
  [[ "$#" -eq 1 ]] || {
    postgres_error 'usage: envctl-postgres-ruvector-lifecycle.sh detect|install|verify|fix|remove'
    return 64
  }
  case "$1" in
    detect) postgres_detect ;;
    install | fix) postgres_activate ;;
    verify) postgres_verify ;;
    remove) postgres_remove ;;
    *)
      postgres_error "unknown lifecycle phase: $1"
      return 64
      ;;
  esac
}

if [[ "${BASH_SOURCE[0]}" == "$0" ]]; then
  set -euo pipefail
  postgres_main "$@"
fi
