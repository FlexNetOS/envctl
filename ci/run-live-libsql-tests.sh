#!/usr/bin/env bash
# Run envctl's ignored real-libSQL tests against the exact sqld release shipped
# by manifest/sqld.toml. Each target receives its own sqld process, dynamic
# loopback port, and fresh database directory.
set -euo pipefail

readonly SQLD_VERSION="0.24.32"
readonly SQLD_X86_64_ARCHIVE_SHA256="71720fc8648c19efef416efebd47145ef59b62e198770533530a858e1336879f"
readonly SQLD_X86_64_PAYLOAD_SHA256="0863c3fbe68ac9714bca2cec1330def7a0ba5e4a29f199bf60ef46fa0c95b895"
readonly SQLD_AARCH64_ARCHIVE_SHA256="37f9eee45b388a30192907ecf4565b93df945c079331657073b5b3caf8bb1cd0"
readonly SQLD_AARCH64_PAYLOAD_SHA256="54039931c1088483706790e6cf73444ad88b843a9bb0ca8285b82fc309ad4810"

CARGO_BIN="${CARGO_BIN:-cargo}"
CURL_BIN="${CURL_BIN:-curl}"
SQLD_BIN="${SQLD_BIN:-}"
READY_PROBE_BIN="${LIVE_LIBSQL_READY_PROBE_BIN:-}"
AUTH_BOOTSTRAP_BIN="${LIVE_LIBSQL_AUTH_BOOTSTRAP_BIN:-}"
AUTH_PROBE_BIN="${LIVE_LIBSQL_AUTH_PROBE_BIN:-}"
READY_TIMEOUT_SECONDS="${LIVE_LIBSQL_READY_TIMEOUT_SECONDS:-20}"
READY_INTERVAL_SECONDS="${LIVE_LIBSQL_READY_INTERVAL_SECONDS:-0.1}"
WORK_PARENT="${LIVE_LIBSQL_WORK_ROOT:-${RUNNER_TEMP:-${TMPDIR:-/tmp}}}"
TEST_MODE="${LIVE_LIBSQL_TEST_MODE:-0}"

die() {
  echo "LIVE LIBSQL FAIL: $*" >&2
  exit 1
}

if [ "${1:-}" = "--print-pin-contract" ]; then
  # The release label is informational routing. Archive + extracted-payload SHA-256 values below
  # are the executable identity; the lifecycle never executes untrusted `sqld --version` output.
  printf 'version\t%s\n' "$SQLD_VERSION"
  printf 'x86_64-unknown-linux-gnu\t%s\t%s\n' \
    "$SQLD_X86_64_ARCHIVE_SHA256" "$SQLD_X86_64_PAYLOAD_SHA256"
  printf 'aarch64-unknown-linux-gnu\t%s\t%s\n' \
    "$SQLD_AARCH64_ARCHIVE_SHA256" "$SQLD_AARCH64_PAYLOAD_SHA256"
  exit 0
fi
[ "$#" -eq 0 ] || die "unexpected argument: $1"

case "$READY_TIMEOUT_SECONDS" in
  ''|*[!0-9]*|0) die "LIVE_LIBSQL_READY_TIMEOUT_SECONDS must be a positive integer" ;;
esac
if [ "$TEST_MODE" != 1 ] \
  && { [ -n "$SQLD_BIN" ] || [ -n "$READY_PROBE_BIN" ] \
    || [ -n "$AUTH_BOOTSTRAP_BIN" ] || [ -n "$AUTH_PROBE_BIN" ]; }; then
  die "sqld/readiness/auth overrides require LIVE_LIBSQL_TEST_MODE=1"
fi

command -v "$CARGO_BIN" >/dev/null 2>&1 || die "Cargo executable not found: $CARGO_BIN"
if [ -z "$READY_PROBE_BIN" ] || [ -z "$AUTH_PROBE_BIN" ]; then
  command -v "$CURL_BIN" >/dev/null 2>&1 || die "curl executable not found: $CURL_BIN"
fi
if [ -n "$AUTH_BOOTSTRAP_BIN" ]; then
  command -v "$AUTH_BOOTSTRAP_BIN" >/dev/null 2>&1 \
    || die "auth bootstrap executable not found: $AUTH_BOOTSTRAP_BIN"
fi
if [ -n "$AUTH_PROBE_BIN" ]; then
  command -v "$AUTH_PROBE_BIN" >/dev/null 2>&1 \
    || die "auth probe executable not found: $AUTH_PROBE_BIN"
fi

mkdir -p "$WORK_PARENT"
WORK_PARENT="$(cd "$WORK_PARENT" && pwd -P)"
RUN_ROOT="$(mktemp -d "$WORK_PARENT/envctl-live-libsql.XXXXXX")"
SQLD_PID=""
SQLD_LOG=""
SQLD_URL=""
SQLD_AUTH_KEY=""
SQLD_AUTH_TOKEN_FILE=""
SQLD_AUTH_TOKEN=""
SQLD_HELPER_BIN=""
SQLD_HELPER_DIGEST=""
PREVIOUS_PORT=""

stop_sqld() {
  local attempt
  if [ -z "$SQLD_PID" ]; then
    return 0
  fi
  if kill -0 "$SQLD_PID" 2>/dev/null; then
    kill "$SQLD_PID" 2>/dev/null || true
    for attempt in {1..50}; do
      if ! kill -0 "$SQLD_PID" 2>/dev/null; then
        break
      fi
      sleep 0.1
    done
    if kill -0 "$SQLD_PID" 2>/dev/null; then
      kill -KILL "$SQLD_PID" 2>/dev/null || true
    fi
  fi
  wait "$SQLD_PID" 2>/dev/null || true
  SQLD_PID=""
}

cleanup() {
  local status=$?
  trap - EXIT INT TERM HUP
  stop_sqld
  if [ "${LIVE_LIBSQL_KEEP_WORK:-0}" = 1 ]; then
    echo "live-libSQL work preserved at $RUN_ROOT" >&2
  else
    rm -rf "$RUN_ROOT"
  fi
  exit "$status"
}
trap cleanup EXIT
trap 'exit 130' INT
trap 'exit 143' TERM
trap 'exit 129' HUP

download_pinned_sqld() {
  local arch target expected_sha256 expected_payload_sha256 asset url archive extracted
  command -v "$CURL_BIN" >/dev/null 2>&1 || die "curl executable not found: $CURL_BIN"
  command -v tar >/dev/null 2>&1 || die "tar is required to extract sqld"

  arch="$(uname -m)"
  case "$arch" in
    x86_64)
      target="x86_64-unknown-linux-gnu"
      expected_sha256="$SQLD_X86_64_ARCHIVE_SHA256"
      expected_payload_sha256="$SQLD_X86_64_PAYLOAD_SHA256"
      ;;
    aarch64)
      target="aarch64-unknown-linux-gnu"
      expected_sha256="$SQLD_AARCH64_ARCHIVE_SHA256"
      expected_payload_sha256="$SQLD_AARCH64_PAYLOAD_SHA256"
      ;;
    *) die "unsupported sqld CI architecture: $arch" ;;
  esac

  asset="libsql-server-${target}.tar.xz"
  url="https://github.com/tursodatabase/libsql/releases/download/libsql-server-v${SQLD_VERSION}/${asset}"
  archive="$RUN_ROOT/$asset"
  extracted="$RUN_ROOT/sqld-artifact"
  mkdir -p "$extracted"

  echo "downloading pinned sqld v${SQLD_VERSION} for ${target}"
  "$CURL_BIN" --proto '=https' --tlsv1.2 --location --fail --silent --show-error \
    "$url" -o "$archive"
  chmod 0600 "$archive"
  "$SQLD_HELPER_BIN" internal-sqld-verify-sha256 \
    --file "$archive" --expected-sha256 "$expected_sha256" --expected-mode 0600
  tar -xJf "$archive" -C "$extracted"
  SQLD_BIN="$(find "$extracted" -type f -name sqld -print -quit)"
  [ -n "$SQLD_BIN" ] || die "pinned sqld archive did not contain a sqld binary"
  chmod 755 "$SQLD_BIN"
  "$SQLD_HELPER_BIN" internal-sqld-verify-sha256 \
    --file "$SQLD_BIN" --expected-sha256 "$expected_payload_sha256" --expected-mode 0755
}

allocate_distinct_port() {
  local previous="$1" port attempt
  for attempt in {1..20}; do
    # Do not add another language runtime merely to ask the kernel for a
    # candidate. sqld's bind is authoritative: an occupied candidate exits
    # early and start_sqld retries with a different high unprivileged port.
    port=$((20000 + ((RANDOM + $$ + SECONDS * 7919 + attempt * 104729) % 40000)))
    if [ -n "$port" ] && [ "$port" != "$previous" ]; then
      printf '%s\n' "$port"
      return 0
    fi
  done
  return 1
}

auth_file_is_safe() {
  [ -s "$1" ] && [ -f "$1" ] && [ ! -L "$1" ] \
    && [ "$(stat -c '%a' "$1")" = 600 ] \
    && [ "$(stat -c '%u' "$1")" = "$(id -u)" ]
}

prepare_rust_helper() {
  if [ -n "$AUTH_BOOTSTRAP_BIN" ] && [ -n "$AUTH_PROBE_BIN" ]; then
    return 0
  fi
  local target_dir="$RUN_ROOT/helper-target"
  echo "building the checked-in Rust-native sqld auth/readiness helper"
  CARGO_TARGET_DIR="$target_dir" "$CARGO_BIN" build --quiet --locked \
    -p envctl-secretctl --bin secretctl
  SQLD_HELPER_BIN="$target_dir/debug/secretctl"
  [ -x "$SQLD_HELPER_BIN" ] || die "Rust-native sqld helper build produced no executable"
}

generate_auth_pair() {
  local label="$1" auth_dir
  auth_dir="$RUN_ROOT/$label/auth"
  mkdir -p "$auth_dir"
  chmod 700 "$auth_dir"
  SQLD_AUTH_KEY="$auth_dir/auth-jwt-key.pem"
  SQLD_AUTH_TOKEN_FILE="$auth_dir/client.jwt"

  if [ -n "$AUTH_BOOTSTRAP_BIN" ]; then
    "$AUTH_BOOTSTRAP_BIN" \
      --public-key "$SQLD_AUTH_KEY" --client-token "$SQLD_AUTH_TOKEN_FILE"
  else
    # Production path: exercise the exact checked-in pure-Rust binary used for the readiness proof.
    "$SQLD_HELPER_BIN" internal-sqld-auth-bootstrap \
      --public-key "$SQLD_AUTH_KEY" --client-token "$SQLD_AUTH_TOKEN_FILE"
  fi

  if [ -z "$AUTH_PROBE_BIN" ]; then
    SQLD_HELPER_DIGEST="$auth_dir/secretctl.sha256"
    "$SQLD_HELPER_BIN" internal-sqld-self-digest --output "$SQLD_HELPER_DIGEST"
    auth_file_is_safe "$SQLD_HELPER_DIGEST" \
      || die "$label sqld helper digest is not a current-user-owned 0600 regular file"
  else
    SQLD_HELPER_DIGEST=""
  fi

  auth_file_is_safe "$SQLD_AUTH_KEY" \
    || die "$label sqld public key is not a current-user-owned 0600 regular file"
  auth_file_is_safe "$SQLD_AUTH_TOKEN_FILE" \
    || die "$label sqld client JWT is not a current-user-owned 0600 regular file"
  SQLD_AUTH_TOKEN="$(cat "$SQLD_AUTH_TOKEN_FILE")"
  case "$SQLD_AUTH_TOKEN" in
    ''|*[!A-Za-z0-9._~-]*) die "$label sqld client JWT has an unsafe shape" ;;
  esac
}

port_is_ready() {
  local port="$1"
  if [ -n "$READY_PROBE_BIN" ]; then
    "$READY_PROBE_BIN" "$port"
    return
  fi
  # Any complete HTTP response proves the listener is accepting requests; the
  # immediately-following real Hrana tests prove protocol/database readiness.
  # Do not use --fail: sqld versions without /health may legitimately answer
  # that path with 404 while still proving the HTTP listener is live.
  "$CURL_BIN" --connect-timeout 1 --max-time 2 --silent --show-error \
    --output /dev/null "http://127.0.0.1:$port/health" 2>/dev/null \
    || "$CURL_BIN" --connect-timeout 1 --max-time 2 --silent --show-error \
      --output /dev/null "http://127.0.0.1:$port/" 2>/dev/null
}

verify_sqld_auth() {
  local label="$1" port="$2"
  if [ -n "$AUTH_PROBE_BIN" ]; then
    "$AUTH_PROBE_BIN" "$port" "$SQLD_AUTH_TOKEN_FILE" \
      || die "$label sqld fixture auth probe failed"
    return 0
  fi

  # Exercise the exact pure-Rust helper installed as sqld.service's ExecStartPost barrier. It binds
  # the proof to this process identity/listening FD, requires unauthenticated SQL to return exactly
  # 401, and reads the bearer from its 0600 file (never argv, environment, or logs).
  "$SQLD_HELPER_BIN" internal-sqld-readiness-probe \
    --pid "$SQLD_PID" --expected-executable "$SQLD_BIN" --port "$port" \
    --client-token "$SQLD_AUTH_TOKEN_FILE" --helper-digest "$SQLD_HELPER_DIGEST" \
    --timeout-seconds 5 \
    || die "$label sqld Rust-native ownership/auth readiness barrier failed"
}

wait_for_sqld() {
  local port="$1" deadline
  deadline=$((SECONDS + READY_TIMEOUT_SECONDS))
  while [ "$SECONDS" -lt "$deadline" ]; do
    if ! kill -0 "$SQLD_PID" 2>/dev/null; then
      wait "$SQLD_PID" 2>/dev/null || true
      SQLD_PID=""
      return 2
    fi
    if port_is_ready "$port"; then
      sleep "$READY_INTERVAL_SECONDS"
      if kill -0 "$SQLD_PID" 2>/dev/null && port_is_ready "$port"; then
        return 0
      fi
    fi
    sleep "$READY_INTERVAL_SECONDS"
  done
  return 1
}

show_sqld_log() {
  if [ -n "$SQLD_LOG" ] && [ -f "$SQLD_LOG" ]; then
    echo "--- sqld log: $SQLD_LOG ---" >&2
    sed -n '1,240p' "$SQLD_LOG" >&2
  fi
}

start_sqld() {
  local label="$1" data_dir="$2" port attempt readiness
  mkdir -p "$data_dir"
  for attempt in {1..5}; do
    port="$(allocate_distinct_port "$PREVIOUS_PORT")" \
      || die "unable to allocate a dynamic loopback port for $label"
    PREVIOUS_PORT="$port"
    SQLD_LOG="$RUN_ROOT/$label/sqld-attempt-$attempt.log"
    echo "starting $label sqld on 127.0.0.1:$port with fresh data $data_dir"
    "$SQLD_BIN" --http-listen-addr "127.0.0.1:$port" \
      --auth-jwt-key-file "$SQLD_AUTH_KEY" -d "$data_dir" \
      >"$SQLD_LOG" 2>&1 &
    SQLD_PID=$!

    if wait_for_sqld "$port"; then
      verify_sqld_auth "$label" "$port"
      SQLD_URL="http://127.0.0.1:$port"
      return 0
    else
      readiness=$?
    fi

    if [ "$readiness" -eq 2 ]; then
      echo "sqld exited before readiness for $label (attempt $attempt/5)" >&2
      show_sqld_log
      stop_sqld
      continue
    fi

    show_sqld_log
    die "$label sqld did not become ready within ${READY_TIMEOUT_SECONDS}s"
  done
  die "$label sqld exited before readiness on five dynamic ports"
}

run_suite() {
  local label="$1"
  shift
  local data_dir="$RUN_ROOT/$label/data"
  mkdir -p "$RUN_ROOT/$label"
  generate_auth_pair "$label"
  start_sqld "$label" "$data_dir"
  echo "running $label ignored tests against $SQLD_URL"
  if LIBSQL_TEST_URL="$SQLD_URL" LIBSQL_TEST_AUTH="$SQLD_AUTH_TOKEN" "$CARGO_BIN" "$@"; then
    stop_sqld
    SQLD_AUTH_TOKEN=""
    return 0
  fi
  show_sqld_log
  stop_sqld
  SQLD_AUTH_TOKEN=""
  die "$label ignored tests failed"
}

cd "$(git -C "$(dirname "${BASH_SOURCE[0]}")" rev-parse --show-toplevel)"
prepare_rust_helper
if [ -z "$SQLD_BIN" ]; then
  download_pinned_sqld
fi
[ -x "$SQLD_BIN" ] || die "sqld executable is missing: $SQLD_BIN"

run_suite store \
  test --locked -p envctl-secrets-store-libsql --features remote --test integration_remote -- \
  --ignored --nocapture --test-threads=1

run_suite secretd \
  test --locked -p envctl-secretd --test libsql_e2e \
  --features envctl-secrets-engine/low-cost-kdf-tests -- \
  --ignored --nocapture --test-threads=1

echo "PASS: real libSQL Store and secretd durability tests passed on isolated pinned sqld instances"
