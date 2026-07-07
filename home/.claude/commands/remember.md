Store the following in ICM memory: $ARGUMENTS

Run:
```bash
command -v icm >/dev/null 2>&1 || { echo "icm is not installed on this workstation — /remember is inert until it ships via the foundation profile (see ~/.claude/CLAUDE.md ICM note). Use the file-based memory at \$HARNESS_VAR/lib/claude-harness/memory instead."; exit 0; }
icm store -t "note" -c "$ARGUMENTS"
```
