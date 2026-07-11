#!/usr/bin/env python3
"""PreToolUse(Bash) hook: route Bash-tool commands through nushell supervision.

Contract (agent-env prompt, BASH-TOOL ROUTING CONTRACT): nu is the supervising
outer process; bash stays the inner POSIX executor (nu does not parse bash
syntax). The command is written to a scratch file for byte-perfect fidelity (no
quoting layer) and run via `nu -c "^bash <file>"` — the `-c` body is only the
fixed `^bash <file>` dispatcher, never the user's program, so there is no inline
shell-program seam. NO `-l` login: no per-call login-profile re-source (that
coupled every command to nu-login-startup health); PATH is inherited from the
launched session. External exit code and stdout/stderr pass through faithfully.

Composition: rtk's rewrite is applied INTERNALLY (this hook invokes
`rtk hook claude` on the same input and wraps its updated command). Whatever
the harness's hook-chaining semantics are — last-wins, first-wins, or merge —
the outcome is either full routing (rtk+nu) or the pre-hook status quo.
Never a downgrade of rtk coverage.

Escape hatches: a command starting with `nu ` or `\\` passes through
unmodified; BASH_NU_ROUTE=0 disables routing. Fail-open: ANY internal error
=> plain allow with the command untouched.
"""
import json
import os
import subprocess
import sys
import tempfile
import time


def allow(updated_command=None):
    out = {"hookSpecificOutput": {"hookEventName": "PreToolUse", "permissionDecision": "allow"}}
    if updated_command is not None:
        out["hookSpecificOutput"]["updatedInput"] = {"command": updated_command}
    print(json.dumps(out))
    sys.exit(0)


def main():
    data = json.load(sys.stdin)
    cmd = (data.get("tool_input") or {}).get("command", "")
    if not isinstance(cmd, str) or not cmd.strip():
        allow()
    stripped = cmd.lstrip()
    # escape hatches + idempotency (never double-wrap)
    if stripped.startswith("nu ") or stripped.startswith("\\") or os.environ.get("BASH_NU_ROUTE") == "0":
        allow()

    # compose the rtk rewrite first (same stdin contract; pure rewrite, safe to re-run)
    new_cmd = cmd
    try:
        r = subprocess.run(
            ["rtk", "hook", "claude"],
            input=json.dumps(data), capture_output=True, text=True, timeout=5,
        )
        if r.returncode == 0 and r.stdout.strip():
            hso = json.loads(r.stdout).get("hookSpecificOutput") or {}
            updated = (hso.get("updatedInput") or {}).get("command")
            if isinstance(updated, str) and updated.strip():
                new_cmd = updated
    except Exception:
        pass  # rtk unavailable or unexpected output => wrap the original command

    scratch = os.path.join(
        os.environ.get("HARNESS_VAR", "/home/flexnetos/meta/var"),
        "lib", "claude-harness", "nu-route",
    )
    os.makedirs(scratch, exist_ok=True)
    # opportunistic cleanup: scratch files older than a day
    try:
        cutoff = time.time() - 86400
        for name in os.listdir(scratch):
            p = os.path.join(scratch, name)
            if os.path.isfile(p) and os.path.getmtime(p) < cutoff:
                os.unlink(p)
    except Exception:
        pass

    f = tempfile.NamedTemporaryFile("w", dir=scratch, suffix=".sh", delete=False)
    f.write(new_cmd)
    f.close()
    allow(f'nu -c "^bash {f.name}"')


if __name__ == "__main__":
    try:
        main()
    except Exception:
        # fail-open: never break the Bash tool
        print(json.dumps({"hookSpecificOutput": {"hookEventName": "PreToolUse", "permissionDecision": "allow"}}))
        sys.exit(0)
