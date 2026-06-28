#!/usr/bin/env bash
# Install envctl-gui as a desktop application (binary + launcher + icon).
# Idempotent: safe to re-run. Meta-scoped (no sudo): artifacts land inside meta.
#
# The binary goes to the canonical frontdoor `$META_ROOT/usr/bin` (matching the
# `desktop-app` manifest component's detect/verify, which key on `$META_ROOT/usr/bin`).
# Override with ENVCTL_BIN_DIR; the data dir (.desktop + icon) stays under
# $META_ROOT/.local/share (XDG data, per the install-locations ADR).
#
#   bash packaging/install-desktop.sh           # build (release) + install for current user
#   bash packaging/install-desktop.sh --no-build # install an already-built binary
#   bash packaging/install-desktop.sh --uninstall
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
BIN_NAME="envctl-gui"
META_ROOT="${META_ROOT:-$HOME/Desktop/meta}"
ENVCTL_LOCAL="${ENVCTL_LOCAL:-$META_ROOT/.local}"
BIN_DIR="${ENVCTL_BIN_DIR:-$META_ROOT/usr/bin}"
SHARE_DIR="${ENVCTL_SHARE_DIR:-$ENVCTL_LOCAL/share}"
APP_DIR="$SHARE_DIR/applications"
ICON_DIR="$SHARE_DIR/icons/hicolor/scalable/apps"
ICON_THEME_DIR="$SHARE_DIR/icons/hicolor"

uninstall() {
  rm -f "$BIN_DIR/$BIN_NAME" \
        "$APP_DIR/$BIN_NAME.desktop" \
        "$ICON_DIR/$BIN_NAME.svg"
  echo "Removed $BIN_NAME desktop integration."
  command -v update-desktop-database >/dev/null 2>&1 && update-desktop-database "$APP_DIR" || true
  exit 0
}

BUILD=1
for arg in "$@"; do
  case "$arg" in
    --uninstall) uninstall ;;
    --no-build)  BUILD=0 ;;
    *) echo "unknown arg: $arg" >&2; exit 2 ;;
  esac
done

if [[ "$BUILD" == 1 ]]; then
  ( cd "$REPO_ROOT" && cargo build -p "$BIN_NAME" --release )
fi

SRC_BIN="$REPO_ROOT/target/release/$BIN_NAME"
[[ -x "$SRC_BIN" ]] || { echo "missing binary: $SRC_BIN (build first, or drop --no-build)" >&2; exit 1; }

mkdir -p "$BIN_DIR" "$APP_DIR" "$ICON_DIR"
install -m 0755 "$SRC_BIN" "$BIN_DIR/$BIN_NAME"
install -m 0644 "$REPO_ROOT/packaging/$BIN_NAME.svg" "$ICON_DIR/$BIN_NAME.svg"
install -m 0644 "$REPO_ROOT/packaging/$BIN_NAME.desktop" "$APP_DIR/$BIN_NAME.desktop"

command -v update-desktop-database >/dev/null 2>&1 && update-desktop-database "$APP_DIR" || true
command -v gtk-update-icon-cache  >/dev/null 2>&1 && \
  gtk-update-icon-cache -f -t "$ICON_THEME_DIR" >/dev/null 2>&1 || true

echo "Installed:"
echo "  binary  -> $BIN_DIR/$BIN_NAME"
echo "  launcher-> $APP_DIR/$BIN_NAME.desktop"
echo "  icon    -> $ICON_DIR/$BIN_NAME.svg"
case ":$PATH:" in *":$BIN_DIR:"*) ;; *) echo "note: $BIN_DIR is not on PATH";; esac
