{
  "hooks": {
    "SessionStart": [
      {
        "matcher": "startup|resume|clear|compact",
        "hooks": [
          {
            "type": "command",
            "command": "sh -lc 'exec /bin/bash \"/home/flexnetos/FlexNetOS/.codex/hooks/flexnetos-codex-hook.sh\" session-start'",
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
            "command": "sh -lc 'exec /bin/bash \"/home/flexnetos/FlexNetOS/.codex/hooks/flexnetos-codex-hook.sh\" pre-tool-use'",
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
            "command": "sh -lc 'exec /bin/bash \"/home/flexnetos/FlexNetOS/.codex/hooks/flexnetos-codex-hook.sh\" permission-request'",
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
            "command": "sh -lc 'exec /bin/bash \"/home/flexnetos/FlexNetOS/.codex/hooks/flexnetos-codex-hook.sh\" post-tool-use'",
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
            "command": "sh -lc 'exec /bin/bash \"/home/flexnetos/FlexNetOS/.codex/hooks/flexnetos-codex-hook.sh\" pre-compact'",
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
            "command": "sh -lc 'exec /bin/bash \"/home/flexnetos/FlexNetOS/.codex/hooks/flexnetos-codex-hook.sh\" stop'",
            "timeout": 10,
            "statusMessage": "FlexNetOS runtime gate: stop"
          }
        ]
      }
    ]
  }
}
