#!/usr/bin/env bash
set -euo pipefail

fail() { echo "FAIL: $*" >&2; exit 1; }

root="$(git -C "$(dirname "${BASH_SOURCE[0]}")" rev-parse --show-toplevel)"
runner="$root/ci/run-live-libsql-tests.sh"
workflow="$root/.github/workflows_disabled/ci.yml"
e2e="$root/crates/secretd/tests/libsql_e2e.rs"

[ -x "$runner" ] || fail "missing executable live-libSQL runner: $runner"
if grep -Eiq '(^|[^[:alnum:]_])(python3?|node|perl)([^[:alnum:]_]|$)' "$runner"; then
  fail "live-libSQL runner invokes a foreign language runtime"
fi

tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT
fake_bin="$tmp/fake-bin"
work="$tmp/work"
mkdir -p "$fake_bin" "$work"

cat >"$fake_bin/sqld" <<'SH'
#!/usr/bin/env bash
set -euo pipefail
if [ "${1:-}" = "--version" ]; then
  echo 'sqld sqld 0.24.32 (40c272de 2025-02-14)'
  exit 0
fi

listen=""
data=""
auth_key=""
while [ "$#" -gt 0 ]; do
  case "$1" in
    --http-listen-addr) listen="$2"; shift 2 ;;
    --auth-jwt-key-file) auth_key="$2"; shift 2 ;;
    -d) data="$2"; shift 2 ;;
    *) echo "unexpected sqld argument: $1" >&2; exit 2 ;;
  esac
done
[ -n "$listen" ] || { echo "missing listen address" >&2; exit 2; }
[ -n "$data" ] && [ -d "$data" ] || { echo "missing fresh data directory" >&2; exit 2; }
[ -s "$auth_key" ] && [ -f "$auth_key" ] && [ ! -L "$auth_key" ] \
  || { echo "missing safe JWT public key" >&2; exit 2; }
printf '%s\t%s\t%s\t%s\n' "$$" "$listen" "$data" "$auth_key" >>"${LIVE_SQLD_LOG:?}"
if [ "${LIVE_SQLD_EXIT_FIRST:-0}" = 1 ] && [ ! -e "${LIVE_SQLD_EXIT_STATE:?}" ]; then
  : >"$LIVE_SQLD_EXIT_STATE"
  exit 98
fi
trap 'exit 0' TERM INT HUP
while :; do sleep 0.05; done
SH
chmod 755 "$fake_bin/sqld"

cat >"$fake_bin/ready-probe" <<'SH'
#!/usr/bin/env bash
set -euo pipefail
printf '%s\n' "$1" >>"${LIVE_READY_LOG:?}"
[ "${LIVE_READY_FAIL:-0}" = 0 ]
SH
chmod 755 "$fake_bin/ready-probe"

cat >"$fake_bin/auth-bootstrap" <<'SH'
#!/usr/bin/env bash
set -euo pipefail
public_key=""
client_token=""
while [ "$#" -gt 0 ]; do
  case "$1" in
    --public-key) public_key="$2"; shift 2 ;;
    --client-token) client_token="$2"; shift 2 ;;
    *) echo "unexpected auth-bootstrap argument: $1" >&2; exit 2 ;;
  esac
done
[ -n "$public_key" ] && [ -n "$client_token" ] || exit 2
[ ! -e "$public_key" ] && [ ! -L "$public_key" ] || exit 1
[ ! -e "$client_token" ] && [ ! -L "$client_token" ] || exit 1
case "$client_token" in
  */store/auth/client.jwt) label=store ;;
  */secretd/auth/client.jwt) label=secretd ;;
  *) echo "auth path is not suite-isolated: $client_token" >&2; exit 2 ;;
esac
mkdir -p "$(dirname "$public_key")"
printf '%s\n' '-----BEGIN PUBLIC KEY-----' "FIXTURE-$label" '-----END PUBLIC KEY-----' >"$public_key"
printf 'fixture.%s.jwt' "$label" >"$client_token"
chmod 0600 "$public_key" "$client_token"
printf '%s\t%s\t%s\n' "$label" "$public_key" "$client_token" >>"${LIVE_AUTH_BOOTSTRAP_LOG:?}"
SH
chmod 755 "$fake_bin/auth-bootstrap"

cat >"$fake_bin/auth-probe" <<'SH'
#!/usr/bin/env bash
set -euo pipefail
port="$1"
token_file="$2"
[ -s "$token_file" ] && [ "$(stat -c '%a' "$token_file")" = 600 ] || exit 2
case "$token_file" in
  */store/auth/client.jwt) expected='fixture.store.jwt'; label=store ;;
  */secretd/auth/client.jwt) expected='fixture.secretd.jwt'; label=secretd ;;
  *) exit 2 ;;
esac
[ "$(cat "$token_file")" = "$expected" ] || exit 1
[ "${LIVE_AUTH_PROBE_FAIL:-0}" = 0 ] || exit 41
printf '%s\t%s\t%s\n' "$label" "$port" "$token_file" >>"${LIVE_AUTH_PROBE_LOG:?}"
SH
chmod 755 "$fake_bin/auth-probe"

cat >"$fake_bin/cargo" <<'SH'
#!/usr/bin/env bash
set -euo pipefail
[ "${LIBSQL_TEST_URL+x}" = x ] || { echo "LIBSQL_TEST_URL is unset" >&2; exit 2; }
[ "${LIBSQL_TEST_AUTH+x}" = x ] || { echo "LIBSQL_TEST_AUTH is unset" >&2; exit 2; }
case "$*" in
  *'envctl-secrets-store-libsql'*) expected='fixture.store.jwt' ;;
  *'envctl-secretd'*) expected='fixture.secretd.jwt' ;;
  *) echo "unexpected Cargo target: $*" >&2; exit 2 ;;
esac
[ "$LIBSQL_TEST_AUTH" = "$expected" ] || { echo "generated auth was not propagated" >&2; exit 2; }
printf '%s\tauth=present\t%s\n' "$LIBSQL_TEST_URL" "$*" \
  >>"${LIVE_CARGO_LOG:?}"
SH
chmod 755 "$fake_bin/cargo"

export LIVE_SQLD_LOG="$tmp/sqld.log"
export LIVE_CARGO_LOG="$tmp/cargo.log"
export LIVE_READY_LOG="$tmp/ready.log"
export LIVE_AUTH_BOOTSTRAP_LOG="$tmp/auth-bootstrap.log"
export LIVE_AUTH_PROBE_LOG="$tmp/auth-probe.log"
export LIVE_SQLD_EXIT_STATE="$tmp/sqld-exit-state"
touch "$LIVE_SQLD_LOG" "$LIVE_CARGO_LOG" "$LIVE_READY_LOG" \
  "$LIVE_AUTH_BOOTSTRAP_LOG" "$LIVE_AUTH_PROBE_LOG"

if SQLD_BIN="$fake_bin/sqld" CARGO_BIN="$fake_bin/cargo" \
  bash "$runner" >"$tmp/override.out" 2>"$tmp/override.err"; then
  fail "runner allowed a sqld override outside fixture-only test mode"
fi
grep -Fq 'overrides require LIVE_LIBSQL_TEST_MODE=1' "$tmp/override.err" \
  || fail "override refusal did not explain the fixture-only test-mode contract"

SQLD_BIN="$fake_bin/sqld" \
CARGO_BIN="$fake_bin/cargo" \
LIVE_LIBSQL_TEST_MODE=1 \
LIVE_LIBSQL_READY_PROBE_BIN="$fake_bin/ready-probe" \
LIVE_LIBSQL_AUTH_BOOTSTRAP_BIN="$fake_bin/auth-bootstrap" \
LIVE_LIBSQL_AUTH_PROBE_BIN="$fake_bin/auth-probe" \
LIVE_LIBSQL_WORK_ROOT="$work" \
LIVE_LIBSQL_READY_TIMEOUT_SECONDS=2 \
LIVE_LIBSQL_READY_INTERVAL_SECONDS=0.01 \
LIBSQL_TEST_AUTH='ambient-auth-must-not-survive' \
  bash "$runner"

mapfile -t sqld_runs <"$LIVE_SQLD_LOG"
[ "${#sqld_runs[@]}" -eq 2 ] || fail "expected two isolated sqld runs, got ${#sqld_runs[@]}"
IFS=$'\t' read -r store_pid store_listen store_data store_key <<<"${sqld_runs[0]}"
IFS=$'\t' read -r daemon_pid daemon_listen daemon_data daemon_key <<<"${sqld_runs[1]}"
[[ "$store_listen" =~ ^127\.0\.0\.1:[0-9]+$ ]] || fail "store sqld did not use a dynamic loopback port"
[[ "$daemon_listen" =~ ^127\.0\.0\.1:[0-9]+$ ]] || fail "daemon sqld did not use a dynamic loopback port"
[ "$store_listen" != "$daemon_listen" ] || fail "store and daemon suites reused one port"
[ "$store_data" != "$daemon_data" ] || fail "store and daemon suites reused one database"
[[ "$store_data" == */store/data ]] || fail "store suite data directory is not isolated"
[[ "$daemon_data" == */secretd/data ]] || fail "daemon suite data directory is not isolated"
[[ "$store_key" == */store/auth/auth-jwt-key.pem ]] || fail "store sqld did not receive its generated JWT key"
[[ "$daemon_key" == */secretd/auth/auth-jwt-key.pem ]] || fail "daemon sqld did not receive its generated JWT key"
[ "$store_key" != "$daemon_key" ] || fail "store and daemon suites reused one JWT key"
kill -0 "$store_pid" 2>/dev/null && fail "store sqld process leaked after its suite"
kill -0 "$daemon_pid" 2>/dev/null && fail "daemon sqld process leaked after its suite"

mapfile -t cargo_runs <"$LIVE_CARGO_LOG"
[ "${#cargo_runs[@]}" -eq 2 ] || fail "expected two explicit Cargo invocations"
IFS=$'\t' read -r store_url store_auth store_args <<<"${cargo_runs[0]}"
IFS=$'\t' read -r daemon_url daemon_auth daemon_args <<<"${cargo_runs[1]}"
[ "$store_url" = "http://$store_listen" ] || fail "store URL does not match its sqld instance"
[ "$daemon_url" = "http://$daemon_listen" ] || fail "daemon URL does not match its sqld instance"
[ "$store_auth" = 'auth=present' ] && [ "$daemon_auth" = 'auth=present' ] \
  || fail "both live suites must receive their generated JWT"
[ "$store_args" = 'test --locked -p envctl-secrets-store-libsql --features remote --test integration_remote -- --ignored --nocapture --test-threads=1' ] \
  || fail "store integration command is not the explicit serial ignored-test contract: $store_args"
[ "$daemon_args" = 'test --locked -p envctl-secretd --test libsql_e2e --features envctl-secrets-engine/low-cost-kdf-tests -- --ignored --nocapture --test-threads=1' ] \
  || fail "daemon integration command is not the explicit serial ignored-test contract: $daemon_args"
mapfile -t bootstrap_runs <"$LIVE_AUTH_BOOTSTRAP_LOG"
[ "${#bootstrap_runs[@]}" -eq 2 ] || fail "expected one fresh Rust-auth bootstrap per suite"
IFS=$'\t' read -r bootstrap_store_label bootstrap_store_key bootstrap_store_token <<<"${bootstrap_runs[0]}"
IFS=$'\t' read -r bootstrap_daemon_label bootstrap_daemon_key bootstrap_daemon_token <<<"${bootstrap_runs[1]}"
[ "$bootstrap_store_label" = store ] && [ "$bootstrap_daemon_label" = secretd ] \
  || fail "auth bootstrap suite ordering drifted"
[ "$bootstrap_store_key" = "$store_key" ] && [ "$bootstrap_daemon_key" = "$daemon_key" ] \
  || fail "sqld did not receive the freshly bootstrapped public keys"
[ "$bootstrap_store_token" != "$bootstrap_daemon_token" ] \
  || fail "store and daemon suites reused one client JWT path"
mapfile -t auth_probes <"$LIVE_AUTH_PROBE_LOG"
[ "${#auth_probes[@]}" -eq 2 ] || fail "each sqld must pass auth compatibility before Cargo"
IFS=$'\t' read -r probe_store_label probe_store_port probe_store_token <<<"${auth_probes[0]}"
IFS=$'\t' read -r probe_daemon_label probe_daemon_port probe_daemon_token <<<"${auth_probes[1]}"
[ "$probe_store_label" = store ] && [ "$probe_store_port" = "${store_listen##*:}" ] \
  && [ "$probe_store_token" = "$bootstrap_store_token" ] \
  || fail "store auth probe did not match its sqld/auth pair"
[ "$probe_daemon_label" = secretd ] && [ "$probe_daemon_port" = "${daemon_listen##*:}" ] \
  && [ "$probe_daemon_token" = "$bootstrap_daemon_token" ] \
  || fail "daemon auth probe did not match its sqld/auth pair"
[ -z "$(find "$work" -mindepth 1 -print -quit)" ] || fail "runner left live-test state behind"

# An early sqld exit (the observable bind-collision shape) must pick a new port
# and retry before it runs either Cargo target.
: >"$LIVE_SQLD_LOG"
: >"$LIVE_CARGO_LOG"
: >"$LIVE_READY_LOG"
: >"$LIVE_AUTH_BOOTSTRAP_LOG"
: >"$LIVE_AUTH_PROBE_LOG"
rm -f "$LIVE_SQLD_EXIT_STATE"
SQLD_BIN="$fake_bin/sqld" \
CARGO_BIN="$fake_bin/cargo" \
LIVE_LIBSQL_TEST_MODE=1 \
LIVE_SQLD_EXIT_FIRST=1 \
LIVE_LIBSQL_READY_PROBE_BIN="$fake_bin/ready-probe" \
LIVE_LIBSQL_AUTH_BOOTSTRAP_BIN="$fake_bin/auth-bootstrap" \
LIVE_LIBSQL_AUTH_PROBE_BIN="$fake_bin/auth-probe" \
LIVE_LIBSQL_WORK_ROOT="$work" \
LIVE_LIBSQL_READY_TIMEOUT_SECONDS=2 \
LIVE_LIBSQL_READY_INTERVAL_SECONDS=0.01 \
  bash "$runner" >"$tmp/retry.out" 2>"$tmp/retry.err"
mapfile -t retry_runs <"$LIVE_SQLD_LOG"
[ "${#retry_runs[@]}" -eq 3 ] || fail "early sqld exit did not trigger exactly one retry"
IFS=$'\t' read -r retry_pid_1 retry_listen_1 _ _ <<<"${retry_runs[0]}"
IFS=$'\t' read -r retry_pid_2 retry_listen_2 _ _ <<<"${retry_runs[1]}"
[ "$retry_listen_1" != "$retry_listen_2" ] || fail "sqld retry reused the collided port"
! kill -0 "$retry_pid_1" 2>/dev/null || fail "early-exit sqld process leaked"
! kill -0 "$retry_pid_2" 2>/dev/null || fail "retried sqld process leaked"
[ "$(wc -l <"$LIVE_CARGO_LOG")" -eq 2 ] || fail "collision retry changed Cargo suite count"
[ "$(wc -l <"$LIVE_AUTH_BOOTSTRAP_LOG")" -eq 2 ] \
  || fail "collision retry regenerated credentials per port instead of per suite"
[ "$(wc -l <"$LIVE_AUTH_PROBE_LOG")" -eq 2 ] \
  || fail "collision retry changed the per-suite auth compatibility count"
[ -z "$(find "$work" -mindepth 1 -print -quit)" ] || fail "collision retry left state behind"

# A server that never answers readiness must fail before Cargo and must still be reaped.
: >"$LIVE_SQLD_LOG"
: >"$LIVE_CARGO_LOG"
: >"$LIVE_READY_LOG"
: >"$LIVE_AUTH_BOOTSTRAP_LOG"
: >"$LIVE_AUTH_PROBE_LOG"
if SQLD_BIN="$fake_bin/sqld" \
   CARGO_BIN="$fake_bin/cargo" \
   LIVE_LIBSQL_TEST_MODE=1 \
   LIVE_READY_FAIL=1 \
   LIVE_LIBSQL_READY_PROBE_BIN="$fake_bin/ready-probe" \
   LIVE_LIBSQL_AUTH_BOOTSTRAP_BIN="$fake_bin/auth-bootstrap" \
   LIVE_LIBSQL_AUTH_PROBE_BIN="$fake_bin/auth-probe" \
   LIVE_LIBSQL_WORK_ROOT="$work" \
   LIVE_LIBSQL_READY_TIMEOUT_SECONDS=1 \
   LIVE_LIBSQL_READY_INTERVAL_SECONDS=0.01 \
     bash "$runner" >"$tmp/timeout.out" 2>"$tmp/timeout.err"; then
  fail "runner accepted a sqld instance that never became ready"
fi
grep -Fq 'did not become ready' "$tmp/timeout.err" \
  || fail "readiness timeout did not report a clear failure"
[ ! -s "$LIVE_CARGO_LOG" ] || fail "Cargo ran before sqld readiness"
[ ! -s "$LIVE_AUTH_PROBE_LOG" ] || fail "auth compatibility ran before listener readiness"
[ "$(wc -l <"$LIVE_AUTH_BOOTSTRAP_LOG")" -eq 1 ] \
  || fail "readiness timeout did not create exactly one isolated auth pair"
while IFS=$'\t' read -r pid _; do
  [ -z "$pid" ] || ! kill -0 "$pid" 2>/dev/null || fail "timed-out sqld process leaked"
done <"$LIVE_SQLD_LOG"
[ -z "$(find "$work" -mindepth 1 -print -quit)" ] || fail "timeout left live-test state behind"

# A listener that is reachable but does not enforce/accept the generated JWT must fail before
# either real test target. This is the hermetic counterpart of the mandatory 401+bearer probe.
: >"$LIVE_SQLD_LOG"
: >"$LIVE_CARGO_LOG"
: >"$LIVE_READY_LOG"
: >"$LIVE_AUTH_BOOTSTRAP_LOG"
: >"$LIVE_AUTH_PROBE_LOG"
if SQLD_BIN="$fake_bin/sqld" \
   CARGO_BIN="$fake_bin/cargo" \
   LIVE_LIBSQL_TEST_MODE=1 \
   LIVE_AUTH_PROBE_FAIL=1 \
   LIVE_LIBSQL_READY_PROBE_BIN="$fake_bin/ready-probe" \
   LIVE_LIBSQL_AUTH_BOOTSTRAP_BIN="$fake_bin/auth-bootstrap" \
   LIVE_LIBSQL_AUTH_PROBE_BIN="$fake_bin/auth-probe" \
   LIVE_LIBSQL_WORK_ROOT="$work" \
   LIVE_LIBSQL_READY_TIMEOUT_SECONDS=2 \
   LIVE_LIBSQL_READY_INTERVAL_SECONDS=0.01 \
     bash "$runner" >"$tmp/auth-fail.out" 2>"$tmp/auth-fail.err"; then
  fail "runner accepted a sqld instance that failed JWT compatibility"
fi
grep -Fq 'fixture auth probe failed' "$tmp/auth-fail.err" \
  || fail "JWT compatibility failure did not report a clear error"
[ ! -s "$LIVE_CARGO_LOG" ] || fail "Cargo ran before JWT compatibility passed"
while IFS=$'\t' read -r pid _; do
  [ -z "$pid" ] || ! kill -0 "$pid" 2>/dev/null || fail "auth-failed sqld process leaked"
done <"$LIVE_SQLD_LOG"
[ -z "$(find "$work" -mindepth 1 -print -quit)" ] || fail "auth failure left live-test state behind"

# Keep the CI runner's artifact identity locked to the shipping sqld component.
bash "$runner" --print-pin-contract >"$tmp/pin-contract.tsv"
runner_version="$(awk -F '\t' '$1 == "version" { print $2 }' "$tmp/pin-contract.tsv")"
runner_archives="$(awk -F '\t' '$1 ~ /-unknown-linux-gnu$/ { print $2 }' "$tmp/pin-contract.tsv" | sort -u)"
runner_payloads="$(awk -F '\t' '$1 ~ /-unknown-linux-gnu$/ { print $3 }' "$tmp/pin-contract.tsv" | sort -u)"
manifest_archives="$(sed -nE 's/^    expected_sha256="([0-9a-f]{64})".*/\1/p' "$root/manifest/sqld.toml" | sort -u)"
manifest_payloads="$(sed -nE 's/.*expected_payload_sha256="([0-9a-f]{64})".*/\1/p' "$root/manifest/sqld.toml" | sort -u)"

[ "$runner_version" = 0.24.32 ] || fail "runner does not pin sqld 0.24.32"
[ "$runner_archives" = "$manifest_archives" ] \
  || fail "runner archive digests drifted from manifest/sqld.toml"
[ "$runner_payloads" = "$manifest_payloads" ] \
  || fail "runner payload digests drifted from manifest/sqld.toml"
python3 - "$root/manifest/sqld.toml" <<'PY' \
  || fail "sqld lifecycle pin placement is incomplete"
import pathlib
import re
import sys
import tomllib

data = tomllib.loads(pathlib.Path(sys.argv[1]).read_text())
component = next(item for item in data["component"] if item["id"] == "sqld")

def hook_script(phase):
    hook = component[phase]
    return hook.get("script", hook.get("args", [None, ""])[1])

digest_pattern = r'expected_payload_sha256="([0-9a-f]{64})"'
# The release-archive pins are the arch-case assignments. The hook also has a local
# `expected_sha256` for the hermetic Rust helper, which is a separate artifact contract.
archive_pattern = r'(?m)^    expected_sha256="([0-9a-f]{64})"$'
for phase in ("install", "verify", "fix"):
    payloads = re.findall(digest_pattern, hook_script(phase))
    assert len(payloads) == 2 and len(set(payloads)) == 2, phase
for phase in ("install", "fix"):
    script = hook_script(phase)
    archives = re.findall(archive_pattern, script)
    assert len(archives) == 2 and len(set(archives)) == 2, phase
    assert script.count('version="0.24.32"') == 1, phase
PY
for digest in \
  71720fc8648c19efef416efebd47145ef59b62e198770533530a858e1336879f \
  37f9eee45b388a30192907ecf4565b93df945c079331657073b5b3caf8bb1cd0 \
  0863c3fbe68ac9714bca2cec1330def7a0ba5e4a29f199bf60ef46fa0c95b895 \
  54039931c1088483706790e6cf73444ad88b843a9bb0ca8285b82fc309ad4810; do
  grep -Fq "$digest" "$runner" || fail "runner is missing pinned digest $digest"
  grep -Fq "$digest" "$root/manifest/sqld.toml" || fail "runner digest $digest drifted from manifest"
done
grep -Fq 'bash ci/run-live-libsql-tests.sh' "$workflow" \
  || fail "CI does not invoke the checked-in live-libSQL runner"
if grep -Fq 'LIVE_LIBSQL_TEST_MODE' "$workflow"; then
  fail "CI bypasses the pinned sqld artifact through the fixture-only test mode"
fi
grep -Fq 'expect("LIBSQL_TEST_URL must point to the runner-managed sqld")' "$e2e" \
  || fail "ignored daemon E2E still permits a missing LIBSQL_TEST_URL"
grep -Fq -- '--auth-jwt-key-file "$SQLD_AUTH_KEY"' "$runner" \
  || fail "live runner does not start sqld with its generated JWT public key"
grep -Fq 'internal-sqld-auth-bootstrap' "$runner" \
  || fail "live runner does not exercise the checked-in Rust-native auth bootstrap"
grep -Fq 'internal-sqld-readiness-probe' "$runner" \
  || fail "live runner does not exercise sqld.service's Rust-native readiness barrier"
grep -Fq 'verify_sqld_auth "$label" "$port"' "$runner" \
  || fail "live runner does not gate Cargo on JWT enforcement/compatibility"
if grep -Fq 'LIBSQL_TEST_AUTH=""' "$runner"; then
  fail "live runner still clears LIBSQL_TEST_AUTH"
fi

echo "PASS: live-libSQL runner uses pinned JWT-authenticated sqld, isolated state, bounded readiness, and serial ignored tests"
