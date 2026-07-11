# Yazelix-managed Zsh hook
# Add Zsh-only commands for Yazelix sessions here
# (envctl-owned: real file in meta/src/envctl/home/.config/yazelix/, symlinked live)

# === rtk (Rust Token Killer) auto-routing — mirrors shell_bash.sh =========
# Escape hatch: prefix with `\` or `command`, e.g. `\git log` for raw git.
# Skipped on purpose: ls/find/grep/tree/wc (coreutils raw output expected).
if command -v rtk >/dev/null 2>&1; then
  for _rtk_cmd in git gh glab gt cargo go pnpm npm npx tsc prettier jest \
    vitest playwright prisma pip pytest ruff mypy rake rubocop rspec dotnet \
    gradlew golangci-lint docker kubectl aws psql curl wget meta; do
    alias "$_rtk_cmd"="rtk $_rtk_cmd"
  done
  unset _rtk_cmd
fi
# === rtk auto-routing — END ================================================
