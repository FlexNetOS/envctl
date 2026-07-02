#!/usr/bin/env bash
# Install project-local Codex hooks for the active FlexNetOS work roots.
#
# The envctl runtime gate stays with envctl. The project-local wrapper in the
# FlexNetOS workspace composes it with workspace-start policy.
set -euo pipefail

source_gate="/home/flexnetos/FlexNetOS/src/envctl/.codex/hooks/flexnetos-runtime-gate.sh"
workspace_hook="/home/flexnetos/FlexNetOS/.codex/hooks/flexnetos-codex-hook.sh"
[ -x "$source_gate" ] || chmod +x "$source_gate"
[ -x "$workspace_hook" ] || chmod +x "$workspace_hook"

install_one() {
  local project_root="$1"
  local codex_dir="$project_root/.codex"
  local hooks_json="$codex_dir/hooks.json"
  mkdir -p "$codex_dir"
  if [ -f "$hooks_json" ] && ! grep -Fq "$workspace_hook" "$hooks_json"; then
    cp "$hooks_json" "$hooks_json.bak.$(date -u +%Y%m%dT%H%M%SZ)"
  fi
  cat >"$hooks_json" <<JSON
{
  "hooks": {
    "SessionStart": [
      {
        "matcher": "startup|resume|clear|compact",
        "hooks": [
          {
            "type": "command",
            "command": "sh -lc 'exec /bin/bash \"$workspace_hook\" session-start'",
            "timeout": 10,
            "statusMessage": "FlexNetOS runtime gate: session start"
          }
        ]
      }
    ],
    "PreToolUse": [
      {
        "matcher": "Bash|apply_patch|Edit|Write",
        "hooks": [
          {
            "type": "command",
            "command": "sh -lc 'exec /bin/bash \"$workspace_hook\" pre-tool-use'",
            "timeout": 10,
            "statusMessage": "FlexNetOS runtime gate: pre tool"
          }
        ]
      }
    ],
    "PermissionRequest": [
      {
        "matcher": "Bash|apply_patch|Edit|Write",
        "hooks": [
          {
            "type": "command",
            "command": "sh -lc 'exec /bin/bash \"$workspace_hook\" permission-request'",
            "timeout": 10,
            "statusMessage": "FlexNetOS runtime gate: permission"
          }
        ]
      }
    ],
    "PostToolUse": [
      {
        "matcher": "Bash|apply_patch|Edit|Write",
        "hooks": [
          {
            "type": "command",
            "command": "sh -lc 'exec /bin/bash \"$workspace_hook\" post-tool-use'",
            "timeout": 10,
            "statusMessage": "FlexNetOS runtime gate: post tool"
          }
        ]
      }
    ],
    "PreCompact": [
      {
        "hooks": [
          {
            "type": "command",
            "command": "sh -lc 'exec /bin/bash \"$workspace_hook\" pre-compact'",
            "timeout": 10,
            "statusMessage": "FlexNetOS runtime gate: pre compact"
          }
        ]
      }
    ],
    "Stop": [
      {
        "hooks": [
          {
            "type": "command",
            "command": "sh -lc 'exec /bin/bash \"$workspace_hook\" stop'",
            "timeout": 10,
            "statusMessage": "FlexNetOS runtime gate: stop"
          }
        ]
      }
    ]
  }
}
JSON
  cat >"$codex_dir/FLEXNETOS_RUNTIME_GATES.md" <<EOF
# FlexNetOS Runtime Gates

This project-local Codex hook file points at the FlexNetOS workspace hook
wrapper:

\`$workspace_hook\`

The wrapper composes the envctl runtime gate:

\`$source_gate\`

with the workspace-start policy:

\`/home/flexnetos/FlexNetOS/.codex/hooks/lib/flexnetos-workspace-policy.sh\`

Session start and stop gates fail closed unless the workspace-owned Beads,
GitKB, and meta surfaces are healthy, and unless the session is rooted at the
FlexNetOS workspace root rather than the old staging directory or a Meta
worktree set.

\`\`\`text
br ready                  # from src/yazelix
git-kb verify/status      # from src/yazelix and src/meta
meta project check        # from src/meta
meta exec -- git-kb verify/status # from src/meta, across the project set
\`\`\`

Reinstall with:

\`\`\`bash
bash /home/flexnetos/FlexNetOS/src/envctl/.codex/hooks/install-flexnetos-runtime-hooks.sh
\`\`\`

Validate with:

\`\`\`bash
bash /home/flexnetos/FlexNetOS/.codex/hooks/flexnetos-codex-doctor.sh
\`\`\`
EOF
  printf 'installed %s\n' "$hooks_json"
}

install_one /home/flexnetos/workspace
install_one /home/flexnetos/FlexNetOS
