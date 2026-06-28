#!/usr/bin/env bash
# add-repo peer-vs-component policy gate.
#
# add-repo must register owned/FlexNetOS repos as first-class meta PEERS (.meta.yaml
# + .gitignore + sibling clone), not as private child components — that drift is what
# this gate forbids. It is a fast, hermetic, grep-only invariant check (no network,
# no build): the behavioral proof lives in `cargo test -p envctl-engine peer`.
set -euo pipefail

ROOT="$(git rev-parse --show-toplevel)"
cd "$ROOT"

PEER=crates/engine/src/peer.rs
EXEC=crates/engine/src/executor.rs
MODEL=crates/engine/src/model.rs
DOC=docs/ADD-REPO.md

fail() {
  echo "add-repo-policy: $1" >&2
  exit 1
}

# 1. The peer path exists with the owned-org router.
[ -f "$PEER" ] || fail "missing $PEER (the meta-native peer registration path)"
grep -q 'OWNED_ORGS' "$PEER" || fail "$PEER must define OWNED_ORGS (the auto-route allowlist)"
grep -q '"FlexNetOS"' "$PEER" || fail "$PEER OWNED_ORGS must include FlexNetOS"
grep -q 'pub fn is_owned_remote' "$PEER" || fail "$PEER must expose is_owned_remote()"

# 2. Registration is meta-native: edits .meta.yaml + .gitignore, NOT a drop-in.
grep -q '\.meta\.yaml' "$PEER" || fail "$PEER must register into .meta.yaml"
grep -q '\.gitignore' "$PEER" || fail "$PEER must register into .gitignore"
grep -q 'synth_dropin\|components\.d/' <(grep -v '^//' "$PEER") \
  && fail "$PEER must NOT write a components.d drop-in (peer != child component)"

# 3. Edits are grep-guarded — never a blind append (ADR-0001 §6).
grep -q 'pub fn meta_has_project' "$PEER" || fail "$PEER must grep-guard the .meta.yaml edit (meta_has_project)"
grep -q 'pub fn gitignore_has' "$PEER" || fail "$PEER must grep-guard the .gitignore edit (gitignore_has)"
grep -q 'refusing to blind-append' "$PEER" || fail "$PEER insert must fail-closed when no projects: block (no blind append)"

# 4. The executor routes Auto/owned to the peer path before the component pipeline.
grep -q 'AddRepoMode' "$MODEL" || fail "$MODEL must define AddRepoMode { Auto, Peer, Component }"
grep -q 'fn resolve_peer' "$EXEC" || fail "$EXEC must define resolve_peer (the mode router)"
grep -q 'crate::peer::register_peer' "$EXEC" || fail "$EXEC add_repo must dispatch to peer::register_peer"

# 5. The front-ends expose the mode (CLI + GUI), so the app can't silently regress.
CLI=crates/cli/src/main.rs
GUI=crates/gui/src/main.rs
grep -q '"peer" => AddRepoMode::Peer' "$CLI" || fail "$CLI must parse --mode peer"
grep -q 'Register as' "$GUI" || fail "$GUI must offer the 'Register as' (peer/component) selector"
# Stale child-repo wording: a host-home '.local/bin' install path. The '[~]' class
# matches a literal tilde without this gate file itself tripping meta-local-policy.
grep -Eq '[~]/\.local/bin|home/\.local/bin' "$GUI" \
  && fail "$GUI must not advertise a host-home .local/bin install path (stale child-repo wording)"

# 6. The doctrine is documented.
grep -qi 'peer' "$DOC" || fail "$DOC must document peer mode"

echo "add-repo-policy: OK (owned remotes route to .meta.yaml peers; edits grep-guarded; CLI+GUI expose the mode; component path intact)"
