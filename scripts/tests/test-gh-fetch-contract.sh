#!/usr/bin/env bash
# Hermetic behavior contract for the sourceable GitHub fetch-token resolver.
set -euo pipefail

ROOT="$(git -C "$(dirname "${BASH_SOURCE[0]}")" rev-parse --show-toplevel)"
HELPER="$ROOT/assets/scripts/envctl-gh-fetch.sh"

fail() {
  echo "FAIL: $*" >&2
  exit 1
}

[ -r "$HELPER" ] || fail "missing GitHub fetch helper: $HELPER"

tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT
fake_bin="$tmp/bin"
mkdir -p "$fake_bin" "$tmp/home"

cat >"$fake_bin/secretctl" <<'SH'
#!/usr/bin/env bash
set -euo pipefail
: "${GH_FETCH_SECRETCTL_LOG:?}"
printf '%q\n' "$@" >>"$GH_FETCH_SECRETCTL_LOG"
[ "${GH_FETCH_SECRETCTL_FAIL:-0}" = 0 ] || exit 55
printf '%s\n' '{"token":"vault-token","expires_at_unix":4102444800}'
SH

cat >"$fake_bin/gh" <<'SH'
#!/usr/bin/env bash
set -euo pipefail
case "${1-}:${2-}" in
  auth:status) exit 0 ;;
  auth:token) printf '%s\n' 'gh-token' ;;
  *) echo "unexpected gh invocation: $*" >&2; exit 2 ;;
esac
SH
chmod 0755 "$fake_bin/secretctl" "$fake_bin/gh"

secretctl_log="$tmp/secretctl.argv"
stderr_log="$tmp/stderr"
injection_marker="$tmp/injection-marker"

run_token() {
  # The positional parameter in the command text is intentionally expanded by the isolated child
  # shell, after env has replaced the caller's environment and PATH.
  # shellcheck disable=SC2016
  /usr/bin/env -i \
    HOME="$tmp/home" \
    PATH="$fake_bin:/usr/bin:/bin" \
    GH_FETCH_SECRETCTL_LOG="$secretctl_log" \
    GH_FETCH_INJECTION_MARKER="$injection_marker" \
    "$@" \
    /usr/bin/bash --noprofile --norc -c 'source "$1"; envctl_gh_token' _ "$HELPER"
}

assert_secretctl_args() {
  local expected_id="$1" expected_ttl="$2"
  local -a actual expected
  mapfile -t actual <"$secretctl_log"
  expected=(
    mint-github
    --installation-id "$expected_id"
    --ttl-secs "$expected_ttl"
    --output json
  )
  [ "${#actual[@]}" -eq "${#expected[@]}" ] \
    || fail "secretctl received ${#actual[@]} argv entries, expected ${#expected[@]}"
  local i
  for i in "${!expected[@]}"; do
    [ "${actual[$i]}" = "${expected[$i]}" ] \
      || fail "secretctl argv[$i]=${actual[$i]} expected ${expected[$i]}"
  done
}

assert_valid() {
  local installation_id="$1" ttl_secs="$2" expected_ttl="$3" out
  : >"$secretctl_log"
  : >"$stderr_log"
  if [ "$ttl_secs" = __UNSET__ ]; then
    out="$(run_token ENVCTL_GH_INSTALLATION_ID="$installation_id" 2>"$stderr_log")" \
      || fail "valid installation id with default TTL failed"
  else
    out="$(run_token \
      ENVCTL_GH_INSTALLATION_ID="$installation_id" \
      ENVCTL_GH_TTL_SECS="$ttl_secs" 2>"$stderr_log")" \
      || fail "valid GitHub mint inputs failed"
  fi
  [ "$out" = vault-token ] || fail "valid inputs did not return the vault token: $out"
  assert_secretctl_args "$installation_id" "$expected_ttl"
}

assert_rejected_before_secretctl() {
  local kind="$1" value="$2" out
  : >"$secretctl_log"
  : >"$stderr_log"
  case "$kind" in
    installation-id)
      out="$(run_token \
        ENVCTL_GH_INSTALLATION_ID="$value" \
        ENVCTL_GH_TTL_SECS=3600 2>"$stderr_log")" \
        || fail "invalid installation id did not fall through to gh"
      grep -Fq 'invalid ENVCTL_GH_INSTALLATION_ID' "$stderr_log" \
        || fail "invalid installation id did not produce the fixed diagnostic"
      ;;
    ttl)
      out="$(run_token \
        ENVCTL_GH_INSTALLATION_ID=140063898 \
        ENVCTL_GH_TTL_SECS="$value" 2>"$stderr_log")" \
        || fail "invalid TTL did not fall through to gh"
      grep -Fq 'invalid ENVCTL_GH_TTL_SECS' "$stderr_log" \
        || fail "invalid TTL did not produce the fixed diagnostic"
      ;;
    *) fail "unknown rejection case: $kind" ;;
  esac
  [ "$out" = gh-token ] || fail "invalid input did not fall through to authenticated gh: $out"
  [ ! -s "$secretctl_log" ] || fail "invalid $kind reached secretctl argv"
}

# Preserve ordinary behavior, the empty/unset TTL default, leading-zero decimals, and the exact
# integer limits accepted by the frozen secretctl CLI contract.
assert_valid 140063898 __UNSET__ 3600
assert_valid 140063898 '' 3600
assert_valid 140063898 600 600
assert_valid 0 0 0
assert_valid 000140063898 000600 000600
assert_valid 18446744073709551615 9223372036854775807 9223372036854775807
assert_valid 00018446744073709551615 0009223372036854775807 0009223372036854775807

# A well-formed request that the vault tier cannot serve still reaches secretctl with the exact
# frozen argv, then preserves the authenticated-gh fallback.
: >"$secretctl_log"
out="$(run_token \
  ENVCTL_GH_INSTALLATION_ID=140063898 \
  ENVCTL_GH_TTL_SECS=3600 \
  GH_FETCH_SECRETCTL_FAIL=1 2>"$stderr_log")" \
  || fail "secretctl failure did not fall through to gh"
[ "$out" = gh-token ] || fail "secretctl failure did not preserve authenticated-gh fallback"
assert_secretctl_args 140063898 3600
grep -Fq 'mint unavailable, using gh' "$stderr_log" \
  || fail "secretctl failure did not produce the fixed fallback diagnostic"

# These command-shaped strings must remain literal data. ShellCheck's expansion warning is exactly
# the behavior this fixture intentionally avoids.
# shellcheck disable=SC2016
invalid_installation_ids=(
  -1 +1 1.0 '1 2' $'1\t2' $'1\n2' 18446744073709551616 018446744073709551616
  999999999999999999999999999999
  '1;touch "$GH_FETCH_INJECTION_MARKER"' '$(touch "$GH_FETCH_INJECTION_MARKER")'
)
for value in "${invalid_installation_ids[@]}"; do
  assert_rejected_before_secretctl installation-id "$value"
done

# shellcheck disable=SC2016
invalid_ttls=(
  -1 +1 1.0 '1 2' $'1\t2' $'1\n2' 9223372036854775808 09223372036854775808
  999999999999999999999999999999
  '1;touch "$GH_FETCH_INJECTION_MARKER"' '$(touch "$GH_FETCH_INJECTION_MARKER")'
)
for value in "${invalid_ttls[@]}"; do
  assert_rejected_before_secretctl ttl "$value"
done

# Without an installation id, minting remains disabled even if a TTL is present.
: >"$secretctl_log"
out="$(run_token ENVCTL_GH_TTL_SECS=600 2>"$stderr_log")" \
  || fail "missing installation id did not fall through to gh"
[ "$out" = gh-token ] || fail "missing installation id did not preserve gh fallback"
[ ! -s "$secretctl_log" ] || fail "secretctl ran without an installation id"

[ ! -e "$injection_marker" ] || fail "hostile input executed as shell code"

echo "PASS: GitHub fetch ambient inputs are range-checked before secretctl argv construction"
