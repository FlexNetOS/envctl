#!/usr/bin/env bash
# Owner ruling 2026-07-07: no META_ROOT/LIFEOS_ROOT wiring (the removed resolver
# defaulted to the old box's /home/drdave/Desktop/meta). The verify script is a
# sibling of this launcher — self-locate, then PATH as fallback.
D="$(cd "$(dirname "$(readlink -f "$0")")" && pwd)"
V="$D/yazelix-gpu-verify.sh"
[ -x "$V" ] || V="$(command -v yazelix-gpu-verify.sh 2>/dev/null || true)"
[ -n "$V" ] && [ -x "$V" ] || exit 0
if   command -v ghostty        >/dev/null; then exec ghostty -e bash -lc "$V"
elif command -v kgx            >/dev/null; then exec kgx -- bash -lc "$V"
elif command -v gnome-terminal >/dev/null; then exec gnome-terminal -- bash -lc "$V"
elif command -v xterm          >/dev/null; then exec xterm -e bash -lc "$V"
else bash -lc "$V"; fi
