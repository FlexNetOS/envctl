# shellcheck shell=bash
# envctl-gh-fetch.sh — shared GitHub fetch-token resolver for Epic-H toolchain installs (TASK-0077).
#
# A SOURCEABLE library (functions only): no top-level `set -e`, no top-level execution, so
# sourcing into a `set -euo pipefail` install hook is safe. ALL diagnostics go to STDERR —
# stdout carries return values that callers capture with `$(...)`, so a stray stdout line
# would corrupt a `TAG=$(...)` capture.
#
# Three-tier token resolution (`envctl_gh_token`), each falling through SILENTLY on ANY failure:
#   1. isolated (preferred): secretctl mint-github  — vault-sealed GitHub App token.
#   2. gh (fallback):        gh auth token          — the developer's authenticated gh.
#   3. unauth:               no token               — caller fetches anonymously (60/hr limit).
#
# The gh and curl network tiers are BYTE-FOR-BYTE behaviour-identical to the inline logic they
# replace; the resolver only OPTIONALLY prepends an `Authorization: Bearer` header when a token
# is available. A token never changes WHAT is fetched, only the rate-limit bucket it counts against.
#
# Usage from a component install hook:
#   ROOT="${META_ROOT:-$HOME/Desktop/meta}/envctl"
#   source "$ROOT/assets/scripts/envctl-gh-fetch.sh"
#   TAG="$(envctl_gh_api 'repos/cli/cli/releases/latest' --jq .tag_name)"

# Resolve a GitHub bearer token to STDOUT and return 0 if one is available; return 1 (echo
# nothing) otherwise. Diagnostics on tier fallback go to stderr.
envctl_gh_token() {
  local out tok

  # Tier 1 — isolated (vault-sealed GitHub App). `secretctl mint-github` REQUIRES
  # --installation-id, --ttl-secs and --output json (frozen contract, TASK-0020). The
  # installation-id is NOT something this resolver can invent, so the mint tier only fires when
  # the operator supplies one via ENVCTL_GH_INSTALLATION_ID; otherwise it falls through cleanly
  # to gh (fail-open). ANY failure (no binary / vault locked / daemon down / malformed JSON /
  # no installation-id) silently drops to the next tier.
  if [ -n "${ENVCTL_GH_INSTALLATION_ID:-}" ] && command -v secretctl >/dev/null 2>&1; then
    if out=$(secretctl mint-github \
               --installation-id "$ENVCTL_GH_INSTALLATION_ID" \
               --ttl-secs "${ENVCTL_GH_TTL_SECS:-3600}" \
               --output json 2>/dev/null); then
      if command -v jq >/dev/null 2>&1; then
        tok=$(printf '%s' "$out" | jq -r '.token // empty' 2>/dev/null)
      else
        # Fallback extractor over the frozen compact shape `{"token":"...","expires_at_unix":N}`.
        tok=$(printf '%s' "$out" | grep -oE '"token":"[^"]*"' | head -1 | sed 's/.*:"//;s/"$//')
      fi
      if [ -n "$tok" ]; then
        printf '%s\n' "$tok"
        return 0
      fi
    fi
    >&2 echo "envctl-gh-fetch: mint unavailable, using gh"
  fi

  # Tier 2 — authenticated gh.
  if command -v gh >/dev/null 2>&1 && gh auth status >/dev/null 2>&1; then
    if tok=$(gh auth token 2>/dev/null) && [ -n "$tok" ]; then
      printf '%s\n' "$tok"
      return 0
    fi
  fi

  # Tier 3 — unauthenticated.
  return 1
}

# Is gh present AND authenticated? Used to choose the gh path over the curl path so behaviour
# stays byte-identical to the components' existing `gh ... || curl ...` branches.
_envctl_gh_authed() {
  command -v gh >/dev/null 2>&1 && gh auth status >/dev/null 2>&1
}

# `curl -fsSL <url> [extra curl args…]`, injecting `-H "Authorization: Bearer $tok"` only when a
# token resolves. Identical plain `curl -fsSL "$url" "$@"` when no token is available.
envctl_gh_curl() {
  local url="$1"; shift
  local tok
  if tok=$(envctl_gh_token); then
    curl -fsSL -H "Authorization: Bearer $tok" "$url" "$@"
  else
    curl -fsSL "$url" "$@"
  fi
}

# `gh api <path> [args…]` when gh is authed (preserves --jq and all gh api flags), else
# `curl -fsSL` against https://api.github.com/<path> with a bearer header when a token resolves.
# NOTE: the curl branch cannot honour gh-specific flags like --jq; the only api.github.com JSON
# caller (llvm-clang) already keeps its own non-gh fallback that does the filtering with grep, so
# this wrapper is used on the gh side and the component preserves its curl-side parsing.
envctl_gh_api() {
  local path="$1"; shift
  if _envctl_gh_authed; then
    gh api "$path" "$@"
  else
    >&2 echo "envctl-gh-fetch: gh unavailable, using api.github.com"
    envctl_gh_curl "https://api.github.com/${path}" "$@"
  fi
}

# Download a release asset. `gh release download --repo R --pattern P --output O --clobber` when
# gh is authed; otherwise resolve the latest tag via the /releases/latest redirect and curl the
# asset URL. The non-gh path treats --pattern as the literal asset filename (the components that
# use this wrapper pass a concrete filename, not a glob).
envctl_gh_release_download() {
  local repo="" pattern="" output=""
  while [ $# -gt 0 ]; do
    case "$1" in
      --repo)    repo="$2";    shift 2 ;;
      --pattern) pattern="$2"; shift 2 ;;
      --output)  output="$2";  shift 2 ;;
      *) shift ;;
    esac
  done
  if _envctl_gh_authed; then
    gh release download --repo "$repo" --pattern "$pattern" --output "$output" --clobber
  else
    >&2 echo "envctl-gh-fetch: gh unavailable, resolving latest release via redirect"
    local tag
    tag="$(curl -fsSLI -o /dev/null -w '%{url_effective}' "https://github.com/${repo}/releases/latest" | sed 's#.*/tag/##')"
    [ -n "$tag" ] || { >&2 echo "envctl-gh-fetch: could not resolve latest tag for ${repo}"; return 1; }
    envctl_gh_curl "https://github.com/${repo}/releases/download/${tag}/${pattern}" -o "$output"
  fi
}
