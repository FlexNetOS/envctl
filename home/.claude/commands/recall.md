Search ICM memory for: $ARGUMENTS

Run:
```bash
command -v icm >/dev/null 2>&1 || { echo "icm is not installed on this workstation — /recall is inert until it ships via the foundation profile (see ~/.claude/CLAUDE.md ICM note). Use the file-based memory at \$HARNESS_VAR/lib/claude-harness/memory instead."; exit 0; }
icm recall "$ARGUMENTS"
```
