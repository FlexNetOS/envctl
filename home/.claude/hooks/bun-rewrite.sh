#!/usr/bin/env bash
# bun-rewrite.sh — PreToolUse[Bash]. Toolchain contract enforcement.
#
# Owner directive (corrected 10+ times on 2026-07-09: "it is bunx run",
# "do it right bunx. not bun run"): the JS/node/wasm layer is bun-only. This
# hook makes npm/npx/pnpm/yarn impossible to run as-is at the TOOL layer instead
# of relying on the model remembering the rule — same idea as block-cherry-pick.sh.
#
# It detects a package manager ONLY in command position (splits on shell
# separators, strips quoted spans + wrappers + env assignments) so `echo "npm i"`
# or `grep npm f` never trip it, then maps to the bun/bunx equivalent:
#   npm install / npm ci / pnpm install / yarn        -> bun install
#   npm install X / npm i X / npm add X / pnpm add X / yarn add X -> bun add X
#   npm run S / pnpm run S / pnpm S / yarn S           -> bun run S   (NOT bunx: proven
#                                                        `bunx S` fails on scripts)
#   npx X / npm exec X / pnpm dlx X / pnpm exec X / yarn dlx X -> bunx X  (package binary)
# Any OTHER npm/pnpm/yarn subcommand (ls, view, publish, why, ...) has no clean bun
# rewrite and is DENIED with guidance (pure bun-only; no runtime escape hatch —
# disable this hook for a genuine npm-only op).
#
# MODE below is set ONCE at install time from the updatedInput smoke test:
#   rewrite -> transparently swap the command (hookSpecificOutput.updatedInput)
#   deny    -> hard-deny with the exact bun command in the reason (always works)
set -u
. "$(dirname "$0")/lib.sh"

# @@BUN_REWRITE_MODE@@  transparent updatedInput verified on Claude Code >=2.1.142
# (docs: hookSpecificOutput.updatedInput full tool_input replace; this host is 2.1.202)
MODE="rewrite"

INPUT=$(cat)

RESULT=$(python3 - "$INPUT" <<'PY'
import json, re, shlex, sys

raw = sys.argv[1] if len(sys.argv) > 1 else ""
try:
    cmd = json.loads(raw).get("tool_input", {}).get("command", "") or ""
except Exception:
    cmd = ""

if not cmd.strip():
    print("PASS")
    sys.exit(0)

PMS = {"npm", "npx", "pnpm", "yarn"}
WRAPPERS = {"sudo", "env", "exec", "command", "nohup", "setsid", "nice",
            "time", "timeout", "xargs", "then", "do", "else"}

# Split into alternating [seg, sep, seg, sep, ...] on top-level shell separators,
# respecting single/double quotes so a quoted separator is never a break point.
def split_segments(s):
    parts, buf, i, n = [], [], 0, len(s)
    q = None  # current quote char or None
    while i < n:
        c = s[i]
        if q:
            buf.append(c)
            if c == q:
                q = None
            i += 1
            continue
        if c in ("'", '"'):
            q = c; buf.append(c); i += 1; continue
        two = s[i:i+2]
        if two in ("&&", "||", ";;"):
            parts.append("".join(buf)); parts.append(two); buf = []; i += 2; continue
        if c in (";", "\n", "|", "&"):
            parts.append("".join(buf)); parts.append(c); buf = []; i += 1; continue
        buf.append(c); i += 1
    parts.append("".join(buf))
    return parts  # even indices = segments, odd = separators

# Strip leading env-assignments and wrappers; return (prefix_str, tokens_after).
ENVA = re.compile(r'^[A-Za-z_][A-Za-z0-9_]*=')
def strip_prefix(tokens):
    pre = []
    i = 0
    while i < len(tokens):
        t = tokens[i]
        if ENVA.match(t) or t in WRAPPERS:
            pre.append(t); i += 1
            # `timeout 5` / `nice -n 5` numeric or flag arg
            while i < len(tokens) and (tokens[i].lstrip("-").isdigit() or tokens[i].startswith("-")):
                pre.append(tokens[i]); i += 1
            continue
        break
    return pre, tokens[i:]

# Map already-stripped [pm, sub, *rest] -> (kind, new_tokens or reason)
#   kind: "same" (no change) | "rewrite" (new_tokens) | "deny" (reason)
def map_pm(toks):
    pm = toks[0]
    rest = toks[1:]
    def is_flag(t): return t.startswith("-")
    def nonflags(ts): return [t for t in ts if not is_flag(t)]
    # normalize flags npm->bun: --save-dev/-D -> -d ; keep -g/--global
    def mapflags(ts):
        out = []
        for t in ts:
            if t in ("--save-dev", "-D"): out.append("-d")
            elif t == "--save": pass  # bun saves by default
            else: out.append(t)
        return out

    if pm == "npx":
        if not rest: return ("deny", "bare `npx` with no package")
        return ("rewrite", ["bunx"] + rest)

    sub = rest[0] if rest else ""
    args = rest[1:]

    # package-binary execution (npx-style) -> bunx  (bunx runs an installed/fetched
    # binary; this is the ONE place bunx is correct — NOT for package.json scripts).
    if pm in ("pnpm",) and sub in ("dlx", "exec"):
        return ("rewrite", ["bunx"] + args) if args else ("deny", f"pnpm {sub} with no package")
    if pm == "yarn" and sub == "dlx":
        return ("rewrite", ["bunx"] + args) if args else ("deny", "yarn dlx with no package")
    if pm == "npm" and sub == "exec":
        return ("rewrite", ["bunx"] + args) if args else ("deny", "npm exec with no package")

    # bare `pnpm` / `yarn` (no sub) -> install
    if pm in ("pnpm", "yarn") and not rest:
        return ("rewrite", ["bun", "install"])

    if sub in ("install", "i", "add", "ci"):
        pkgs = nonflags(args)
        if sub == "ci" or (sub in ("install", "i") and not pkgs):
            return ("rewrite", ["bun", "install"] + mapflags(args))
        return ("rewrite", ["bun", "add"] + mapflags(args))

    # package.json SCRIPT execution -> `bun run <script>`. `bunx <script>` is WRONG
    # here (proven: it tries to fetch a package named <script> and fails), so scripts
    # are the one case that must use `bun run`, not bunx.
    if sub == "run":
        return ("rewrite", ["bun", "run"] + args) if args else ("deny", f"{pm} run with no script")

    # yarn/pnpm builtin subcommands are NOT scripts and have no clean bun rewrite.
    BUILTINS = {"why", "info", "list", "ls", "outdated", "audit", "publish", "pack",
                "init", "create", "link", "unlink", "dedupe", "licenses", "node",
                "patch", "config", "cache", "bin", "global", "workspace", "workspaces",
                "import", "version", "tag", "owner", "login", "logout", "whoami",
                "store", "env", "deploy", "fetch", "rebuild", "prune", "setup", "root",
                "unplug", "plugin", "policies", "set", "get", "constraints", "remove",
                "rm", "up", "upgrade", "update", "upgrade-interactive", "exec", "dlx"}

    # yarn <script> / pnpm <script>  (implicit run) -> `bun run <script>`, but only
    # when the first token is NOT a known builtin.
    if pm in ("pnpm", "yarn") and sub and not is_flag(sub) and sub not in BUILTINS:
        return ("rewrite", ["bun", "run", sub] + args)

    return ("deny", f"`{pm} {sub}`".strip())

parts = split_segments(cmd)
changed = False
denies = []
for idx in range(0, len(parts), 2):
    seg = parts[idx]
    if not seg.strip():
        continue
    try:
        toks = shlex.split(seg)
    except ValueError:
        continue
    if not toks:
        continue
    pre, body = strip_prefix(toks)
    if not body or body[0] not in PMS:
        continue
    kind, payload = map_pm(body)
    if kind == "same":
        continue
    if kind == "deny":
        denies.append(payload)
        changed = True
        continue
    # rewrite: reconstruct segment preserving leading + trailing whitespace so
    # separators stay spaced (e.g. `bun add x && curl`, not `bun add x&& curl`).
    lead = seg[:len(seg) - len(seg.lstrip())]
    trail = seg[len(seg.rstrip()):]
    new_tokens = pre + payload
    parts[idx] = lead + " ".join(shlex.quote(t) if re.search(r'\s', t) else t for t in new_tokens) + trail
    changed = True

if not changed:
    print("PASS")
    sys.exit(0)

if denies:
    # any unmappable PM subcommand -> whole command denied (bun-only, no bypass)
    print("DENY\t" + "; ".join(denies))
    sys.exit(0)

new_cmd = "".join(parts)
# Count non-empty command segments. A rewrite may only AUTO-ALLOW (bypass the
# normal permission prompt) when the WHOLE command is a single pm invocation —
# otherwise `allow` would auto-approve risky sibling segments (e.g.
# `npm i && curl x | sh`). Compound commands are handed back as a deny+replacement
# so the re-issued bun command flows through normal permissions + the guards.
nonempty_segs = sum(1 for i in range(0, len(parts), 2) if parts[i].strip())
if nonempty_segs <= 1:
    print("ALLOW\t" + new_cmd)
else:
    print("MULTI\t" + new_cmd)
PY
)

KIND=$(printf '%s' "$RESULT" | head -1 | cut -f1)
PAYLOAD=$(printf '%s' "$RESULT" | head -1 | cut -f2-)

bun_reason="bun-only toolchain (owner directive, corrected 10+ times): run \`$PAYLOAD\` instead. npm/npx/pnpm/yarn are not used on this machine — bun install/add, bunx for package binaries, bun run for scripts."

case "$KIND" in
  PASS|"")
    exit 0 ;;
  DENY)
    ledger "bun.deny" "\"sub\":\"$(json_escape "$PAYLOAD")\",\"cmd\":\"$(json_escape "$INPUT")\""
    deny "bun-only toolchain (owner directive): $PAYLOAD has no transparent bun rewrite. Use the bun/bunx equivalent, or if this is a genuine npm-only operation (native node-gyp build) disable this hook for the one command." ;;
  ALLOW)
    # Single-segment pm command: safe to transparently rewrite + auto-allow.
    if [ "$MODE" = "rewrite" ]; then
      ledger "bun.rewrite" "\"new\":\"$(json_escape "$PAYLOAD")\""
      esc=$(json_escape "$PAYLOAD")
      printf '{"hookSpecificOutput":{"hookEventName":"PreToolUse","permissionDecision":"allow","updatedInput":{"command":"%s"}}}\n' "$esc"
      exit 0
    else
      ledger "bun.deny" "\"replacement\":\"$(json_escape "$PAYLOAD")\""
      deny "$bun_reason"
    fi ;;
  MULTI)
    # Compound/piped command: never auto-allow (would approve sibling segments).
    # Hand back the fully-rewritten command; the re-issued form flows through
    # normal permissions + guards.
    ledger "bun.deny" "\"replacement\":\"$(json_escape "$PAYLOAD")\",\"reason\":\"compound-no-autoallow\""
    deny "$bun_reason" ;;
esac
exit 0
