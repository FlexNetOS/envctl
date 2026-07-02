#!/usr/bin/env bash
# meta-session-env: wire the meta /usr mirror onto the graphical / desktop login
# session via systemd user `environment.d`, so `.desktop` launchers and GUI
# sessions — which never source ~/.bashrc — resolve $META_ROOT/usr/bin and the
# meta library path. The bash/zsh shells get this from `envctl env` (shell-rc
# eval) and nushell from the overlay module; this closes the third surface.
#
# Values come from `envctl env --toolchains` (the engine is the single source of
# truth — no hand-derived path lists). Idempotent, backs up before overwrite, and
# is meta-resident (writes only under $META_ROOT/.config). No system-depth writes.
#
#   bash assets/scripts/meta-session-env.sh          # apply (default)
#   bash assets/scripts/meta-session-env.sh remove   # unwire
set -euo pipefail

M="${META_ROOT:?META_ROOT required}"
ACTION="${1:-apply}"

# Resolve the canonical layout from envctl itself (source of truth). Prefer a
# PATH-resolved envctl, else the canonical frontdoor.
ENVCTL="$M/usr/bin/envctl"
if command -v envctl >/dev/null 2>&1; then ENVCTL="$(command -v envctl)"; fi
[ -x "$ENVCTL" ] || { echo "meta-session-env: envctl not found at $ENVCTL" >&2; exit 1; }
# shellcheck disable=SC1090
eval "$("$ENVCTL" env --toolchains)"

CFG="${ENVCTL_XDG_CONFIG_HOME:-$M/.config}"
ENVD="$CFG/environment.d"
FILE="$ENVD/10-meta.conf"
MARK="# >>> meta /usr mirror (added by envctl) >>>"

stamp() { date +%s; }

if [ "$ACTION" = "remove" ]; then
  if [ -f "$FILE" ] && grep -qF "$MARK" "$FILE"; then
    cp -p "$FILE" "$FILE.bak.$(stamp)"
    rm -f "$FILE"
    echo "meta-session-env: removed $FILE"
  else
    echo "meta-session-env: nothing to remove"
  fi
  exit 0
fi

# Ensure the FHS /usr mirror tree exists. The engine's ensure_dirs() materializes
# it on a full install/add-repo; this component completes the usr-mirror wiring, so
# it also guarantees the tree (from envctl's own resolved layout — not a hand-kept
# path list) for the single-component install / fix path.
install -d \
  "$ENVCTL_USR_SBIN" "$ENVCTL_USR_LIB64" "$ENVCTL_USR_INCLUDE" "$ENVCTL_USR_SRC" \
  "$ENVCTL_USR_GAMES" "$ENVCTL_USR_SHARE_MAN" "$ENVCTL_USR_LOCAL_BIN" \
  "$ENVCTL_USR_LOCAL_SBIN" "$ENVCTL_USR_LOCAL_LIB" "$ENVCTL_USR_LOCAL_LIB64" \
  "$ENVCTL_USR_LOCAL_INCLUDE" "$ENVCTL_USR_LOCAL_SHARE"

install -d -m 755 "$ENVD"
TMP="$FILE.envctl-tmp.$$"
# environment.d is static KEY=VALUE parsed by systemd (NOT a shell): `${PATH}`
# references the inherited value (always set), so PATH prepends safely. We do NOT
# reference `${LD_LIBRARY_PATH}` — it is usually unset at graphical login and a
# trailing colon would inject the CWD into the loader path (a real hazard).
{
  echo "$MARK"
  echo "# Read by systemd --user / pam_systemd at graphical login; reaches .desktop"
  echo "# launchers + GUI sessions. Generated from \`envctl env\` — do not hand-edit;"
  echo "# re-run \`envctl install meta-session-env\` (or auto-fix) to refresh."
  echo "META_ROOT=$M"
  echo "PATH=$ENVCTL_USR_BIN:$ENVCTL_USR_SBIN:$ENVCTL_USR_LOCAL_BIN:$ENVCTL_USR_LOCAL_SBIN:$ENVCTL_LOCAL_BIN:\${PATH}"
  echo "LD_LIBRARY_PATH=$ENVCTL_USR_LIB:$ENVCTL_USR_LIB64:$ENVCTL_USR_LOCAL_LIB:$ENVCTL_USR_LOCAL_LIB64"
} >"$TMP"

if [ -f "$FILE" ]; then cp -p "$FILE" "$FILE.bak.$(stamp)"; fi
mv -f "$TMP" "$FILE"
echo "meta-session-env: wrote $FILE"
