# Kache-only / Nushell-only baseline

- Base: `b62669c`
- Worktree: `/tmp/kache_nu_enforce/envctl`
- Runners are stopped; no runner may restart before the all-green barrier.
- `origin/master` contains six `Swatinem/rust-cache` uses in `.github/workflows/ci.yml`.
- Automatic CI still enters shell scripts and therefore does not meet the owner policy.
- No product symbol may be edited without the repository impact-analysis rule.

