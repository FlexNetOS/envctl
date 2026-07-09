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
#                                                        `bunx S` fetches+runs an UNRELATED
#                                                        registry package named S)
#   npm test/t/start/stop/restart (lifecycle)          -> bun run <script>  (NOT `bun test`,
#                                                        which is bun's own jest-like runner)
#   npx X / npm exec X / pnpm dlx X / pnpm exec X / yarn dlx X / yarn exec X -> bunx X
#   npm ci                                             -> bun install --frozen-lockfile
#   uninstall/remove -> bun remove ; update/up/upgrade -> bun update ; outdated -> bun outdated
#   audit -> bun audit ; why/explain -> bun why ; view/info/show -> bun info ; ls -> bun pm ls
#   link/unlink -> bun link/unlink ; create/init -> bun create / bun init
#   yarn global add/remove X -> bun add/remove -g X
# `npm publish` is NEVER auto-rewritten (registry publish must be an explicit, humanly
# approved `bun publish`). Any remaining pm subcommand has no clean bun rewrite and is
# DENIED with guidance (pure bun-only; no runtime escape hatch — disable this hook for
# a genuine npm-only op).
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
        # bunx always auto-installs; npx's -y/--yes has no bunx flag and would error
        rest = [t for t in rest if t not in ("-y", "--yes")]
        return ("rewrite", ["bunx"] + rest) if rest else ("deny", "bare `npx` with no package")

    sub = rest[0] if rest else ""
    args = rest[1:]

    # package-binary execution (npx-style) -> bunx  (bunx runs an installed/fetched
    # binary; this is the ONE place bunx is correct — NOT for package.json scripts).
    if (pm == "pnpm" and sub in ("dlx", "exec")) or \
       (pm == "yarn" and sub in ("dlx", "exec")) or \
       (pm == "npm" and sub == "exec"):
        if args and args[0] == "--":   # `npm exec -- foo` separator has no bunx meaning
            args = args[1:]
        return ("rewrite", ["bunx"] + args) if args else ("deny", f"{pm} {sub} with no package")

    # bare `pnpm` / `yarn` (no sub) -> install
    if pm in ("pnpm", "yarn") and not rest:
        return ("rewrite", ["bun", "install"])

    if sub in ("install", "i", "add", "ci"):
        pkgs = nonflags(args)
        if sub == "ci":
            # npm ci = clean install strictly from the lockfile (bun --help: install
            # --frozen-lockfile "Disallow changes to lockfile"; `bun ci` is its alias)
            return ("rewrite", ["bun", "install", "--frozen-lockfile"] + mapflags(args))
        if sub in ("install", "i") and not pkgs:
            return ("rewrite", ["bun", "install"] + mapflags(args))
        return ("rewrite", ["bun", "add"] + mapflags(args))

    # npm lifecycle shortcuts run package.json SCRIPTS -> `bun run <script>`.
    # NEVER `bun test`: that is bun's own jest-like test runner, not the script.
    if sub in ("test", "t", "tst"):
        return ("rewrite", ["bun", "run", "test"] + args)
    if sub in ("start", "stop", "restart"):
        return ("rewrite", ["bun", "run", sub] + args)

    # dependency management verbs with native bun equivalents (bun --help 1.3.13)
    if sub in ("uninstall", "remove", "rm", "un"):
        return ("rewrite", ["bun", "remove"] + mapflags(args)) if nonflags(args) else ("deny", f"{pm} {sub} with no package")
    if sub in ("update", "up", "upgrade"):
        return ("rewrite", ["bun", "update"] + mapflags(args))
    if sub == "outdated":
        return ("rewrite", ["bun", "outdated"] + args)
    if sub == "audit":
        # bare audit maps cleanly; `npm audit fix` has no bun form (bun audit only reports)
        if nonflags(args):
            return ("deny", f"`{pm} audit {' '.join(args)}` — bun audit only reports; use `bun audit` then `bun update`")
        return ("rewrite", ["bun", "audit"] + args)
    if sub in ("why", "explain"):
        return ("rewrite", ["bun", "why"] + args) if nonflags(args) else ("deny", f"{pm} {sub} needs a package")
    if sub in ("view", "info", "show", "v"):
        return ("rewrite", ["bun", "info"] + args) if nonflags(args) else ("deny", f"{pm} {sub} needs a package")
    if sub in ("ls", "list", "la", "ll"):
        # bare tree listing only; filtered/flagged forms have no clean bun mapping
        if args:
            return ("deny", f"`{pm} {sub} {' '.join(args)}` — use `bun pm ls` (full tree) or `bun why <pkg>` (one package)")
        return ("rewrite", ["bun", "pm", "ls"])
    if sub in ("link", "unlink"):
        return ("rewrite", ["bun", sub] + args)
    if sub == "create":
        return ("rewrite", ["bun", "create"] + args) if args else ("deny", f"{pm} create needs a template")
    if sub == "init":
        # bare `npm init [-y]` scaffolds -> bun init; `npm init <tmpl>` == npm create
        return ("rewrite", ["bun", "create"] + args) if nonflags(args) else ("rewrite", ["bun", "init"] + args)
    if pm == "yarn" and sub == "global" and args:
        gsub, gargs = args[0], args[1:]
        if gsub == "add" and nonflags(gargs):
            return ("rewrite", ["bun", "add", "-g"] + mapflags(gargs))
        if gsub in ("remove", "rm") and nonflags(gargs):
            return ("rewrite", ["bun", "remove", "-g"] + gargs)
        return ("deny", f"`yarn global {gsub}`")
    if sub == "publish":
        # bun publish EXISTS, but a registry publish must never be transparently
        # auto-allowed — require an explicit, humanly approved `bun publish`.
        return ("deny", f"`{pm} publish` — run `bun publish` explicitly (never auto-rewritten)")

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
    deny "bun-only toolchain (owner directive): $PAYLOAD. No transparent rewrite applied — use the bun/bunx equivalent, or if this is a genuine npm-only operation (native node-gyp build) disable this hook for the one command." ;;
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
