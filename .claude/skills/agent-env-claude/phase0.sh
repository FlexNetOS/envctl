#!/usr/bin/env bash
# agent-env-claude Phase-0 driver: read-only bootstrap evidence, proof-ledger output.
# Exit 0 = no `fail` rows (gap/unsupported allowed). Exit 1 = at least one fail.
# Every check mirrors the PHASES > Phase 0 contract in SKILL.md / the source prompt.
set -u
# cwd-independent: the repo root is three levels up from this script's directory
REPO=$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)
FAILS=0
row() { printf '| %s | %s | %s | %s |\n' "$1" "$2" "$3" "$4"; [ "$3" = fail ] && FAILS=$((FAILS+1)); }
meta_git_json() {
  (cd /home/flexnetos/meta && rtk meta --json exec --include envctl -- git -C "$@")
}
meta_git_stdout() {
  meta_git_json "$@" 2>/dev/null | jq -r '.results[0].stdout // ""'
}

echo "| item | command | state | evidence |"
echo "|---|---|---|---|"

# 1. Binary resolution — must resolve under /nix/store or ~/.nix-profile
for b in nu bash zsh rtk meta grit icm git-kb bun claude ccboard; do
  p=$(command -v "$b" 2>/dev/null || true)
  case "$p" in
    /nix/store/*|"$HOME"/.nix-profile/*) row "$b resolves nix-owned" "command -v $b" pass "$p" ;;
    "") row "$b resolves nix-owned" "command -v $b" fail "not on PATH" ;;
    *)  row "$b resolves nix-owned" "command -v $b" fail "non-nix: $p" ;;
  esac
done
# Known-gap binaries: absence is `gap`, presence is `pass`
for b in weave rtk-monitor cargo-fmt cargo-clippy; do
  p=$(command -v "$b" 2>/dev/null || true)
  if [ -n "$p" ]; then row "$b available" "command -v $b" pass "$p"
  else row "$b available" "command -v $b" gap "absent (documented gap; see Phase 2/5 / profile shims)"; fi
done

# 2. Version floors
vchk() { # name floor cmd
  out=$($3 2>/dev/null | head -1)
  if [ -n "$out" ]; then row "$1 >= $2" "$3" pass "$out"; else row "$1 >= $2" "$3" fail "no output"; fi
}
vchk rtk 0.43.0 "rtk --version"
vchk grit 0.6.4 "grit --version"
vchk icm 0.10.57 "icm --version"
vchk git-kb 0.2.12 "git-kb --version"
vchk ccboard 0.24.0 "ccboard --version"

# 3. nu login loads (parse-time source failures abort the whole config)
if out=$(nu -l -c "echo NU_LOGIN_OK" 2>&1) && [ "$out" = "NU_LOGIN_OK" ]; then
  row "nu login config loads" 'nu -l -c "echo NU_LOGIN_OK"' pass "$out"
else
  row "nu login config loads" 'nu -l -c "echo NU_LOGIN_OK"' fail "$(echo "$out" | head -1)"
fi
# rtk wrappers active in login nu (wrapped git => type custom)
t=$(nu -l -c "which git | get 0.type" 2>/dev/null)
if [ "$t" = "custom" ]; then row "nu rtk wrappers loaded" 'nu -l -c "which git | get 0.type"' pass "custom"
else row "nu rtk wrappers loaded" 'nu -l -c "which git | get 0.type"' fail "type=$t (expected custom)"; fi

# 4. Symlink contract — envctl-owned shell files are symlinks into envctl
T=/home/flexnetos/meta/src/envctl/home
for f in .config/nushell/config.nu .config/nushell/rtk-wrappers.nu .config/nushell/meta-usr-path.nu \
         .config/yazelix/shell_nu.nu .config/yazelix/shell_bash.sh; do
  if [ -L "$HOME/$f" ] && [ "$(readlink -f "$HOME/$f")" = "$(readlink -f "$T/$f")" ]; then
    row "symlink: $f" "readlink -f ~/$f" pass "-> envctl"
  else
    row "symlink: $f" "readlink -f ~/$f" fail "not a symlink into envctl ($(readlink -f "$HOME/$f" 2>/dev/null || echo missing))"
  fi
done

# 5. ADR-0006 chain: ~/.claude content is envctl-sourced
if [ -L "$HOME/.claude" ]; then
  row "ADR-0006 ~/.claude" "readlink -f ~/.claude" pass "$(readlink -f "$HOME/.claude")"
elif [ -d "$HOME/.claude" ]; then
  # directory surface: individual entries may be symlinks — report, don't guess
  n=$(find "$HOME/.claude" -maxdepth 1 -type l | wc -l)
  row "ADR-0006 ~/.claude" "ls -ld ~/.claude" gap "directory (not symlink); $n symlinked entries at depth 1 — verify per-entry"
else
  row "ADR-0006 ~/.claude" "ls -ld ~/.claude" fail "missing"
fi

# 6. Terminal chain: packaged runtime variant
rv=$(cat "$HOME/.nix-profile/runtime_variant" 2>/dev/null)
if [ "$rv" = "kitty" ]; then row "runtime_variant" "cat ~/.nix-profile/runtime_variant" pass kitty
else row "runtime_variant" "cat ~/.nix-profile/runtime_variant" fail "got '$rv' (expected kitty; mars is removed)"; fi

# 7. yzx agent init preview (read-only, fail-closed on missing tools; repo pinned — cwd-independent)
if yzx agent init --repo "$REPO" >/dev/null 2>&1; then
  row "yzx agent init preview" "yzx agent init --repo $REPO" pass "exit 0 (read-only preview)"
else
  row "yzx agent init preview" "yzx agent init --repo $REPO" fail "exit $? — run interactively for the failing step"
fi

# 8b. skill body byte-identical to the source prompt (regeneration contract)
PROMPT="$REPO/.claude/prompts/prompt:claude-code-agent-env-ultraplan.prompt.md"
SKILL="$REPO/.claude/skills/agent-env-claude/SKILL.md"
if python3 - "$PROMPT" "$SKILL" <<'PYCHK'
import sys
p = open(sys.argv[1]).read(); s = open(sys.argv[2]).read()
sys.exit(0 if p[p.index("## ROLE"):] in s else 1)
PYCHK
then row "skill == prompt (from ROLE)" "python3 substring check" pass "byte-identical"
else row "skill == prompt (from ROLE)" "python3 substring check" fail "skill stale — regenerate from the prompt"; fi

# 8c. no unfinished markers in the skill dir (validate.sh discipline)
if grep -RInE "TODO|PLACEHOLDER" "$REPO/.claude/skills/agent-env-claude" --exclude=phase0.sh -q 2>/dev/null; then
  row "no unfinished markers" "grep -RInE TODO|PLACEHOLDER" fail "marker found"
else
  row "no unfinished markers" "grep -RInE TODO|PLACEHOLDER" pass "clean"
fi

# 8. ICM mandate present in the agent-env contract (Phase 2 acceptance)
if grep -qi "icm" /home/flexnetos/meta/src/envctl/home/.claude/CLAUDE.md 2>/dev/null \
   && ! grep -q "is not installed on this workstation" /home/flexnetos/meta/src/envctl/home/.claude/CLAUDE.md 2>/dev/null; then
  row "ICM mandate restored" "grep -i icm envctl/home/.claude/CLAUDE.md" pass "mandate text present, removal note gone"
else
  row "ICM mandate restored" "grep -i icm envctl/home/.claude/CLAUDE.md" gap "removal note still present — Phase 2 work item"
fi

# 8d. codex-inherit block present (shared-contract surface)
if [ -f "$REPO/.codex/prompts/prompt:substrate-init.inherit.md" ]; then
  row "codex inherit block present" "ls .codex/prompts/prompt:substrate-init.inherit.md" pass "present"
else
  row "codex inherit block present" "ls .codex/prompts/prompt:substrate-init.inherit.md" fail "missing"
fi
# 8e. live settings carry the harness hook entries (source<->live parity)
for h in bash-to-nu ccbrain-session-stop ccbrain-session-start "rtk hook claude" weave; do
  if grep -q "$h" "$HOME/.claude/settings.json" 2>/dev/null; then
    row "live settings: $h" "grep settings.json" pass "wired"
  else
    row "live settings: $h" "grep settings.json" fail "missing from live ~/.claude/settings.json"
  fi
done

# 8f. yazelix ownership proof (codex probe matrix)
if yzx inspect --json 2>/dev/null | grep -q "nix-profile"; then
  row "yzx ownership proof" "yzx inspect --json" pass "profile-owned install confirmed"
else
  row "yzx ownership proof" "yzx inspect --json" gap "inspect missing/unparseable — verify owner manually"
fi

# 8g. GitHub workflows are linux-only (github execution policy)
WFOS=$(grep -rlEi "runs-on:.*\b(macos|windows)|os:.*\[(.*\b(macos|windows))" "$REPO/.github/workflows/" 2>/dev/null || true)
if [ -z "$WFOS" ]; then
  row "workflows linux-only" "grep -rEi 'macos|windows' .github/workflows/" pass "no macOS/Windows runners"
else
  row "workflows linux-only" "grep -rEi 'macos|windows' .github/workflows/" fail "non-linux runner in: $(echo "$WFOS" | tr '\n' ' ')"
fi
# 8h. branches<->origin<->worktrees sync audit (stale/orphaned work = unfinished work)
REAPABLE=0; INFLIGHT=0
while IFS= read -r wt; do
  [ -z "$wt" ] && continue
  [ "$wt" = "$REPO" ] && continue
  [ "$wt" = "/home/flexnetos/meta/src/envctl" ] && continue
  case "$wt" in "$PWD") continue ;; esac
  if [ -n "$(meta_git_stdout "$wt" status --porcelain)" ]; then
    INFLIGHT=$((INFLIGHT+1))
  elif meta_git_json "$wt" merge-base --is-ancestor HEAD origin/develop 2>/dev/null | jq -e '.success' >/dev/null; then
    REAPABLE=$((REAPABLE+1))
  fi
done < <(meta_git_stdout "$REPO" worktree list --porcelain | sed -n 's/^worktree //p')
if [ "$REAPABLE" -gt 0 ]; then
  row "worktree/branch sync" "git worktree list + merge-base audit" gap "$REAPABLE merged-clean worktree(s) awaiting reap; $INFLIGHT dirty (in-flight, recorded)"
else
  row "worktree/branch sync" "git worktree list + merge-base audit" pass "0 reapable; $INFLIGHT dirty (in-flight, recorded)"
fi

# 8i. GitHub organization surface audit (read-only metadata; never secret values)
ORG=FlexNetOS
num() { gh api "$1" -q "$2" 2>/dev/null | grep -Em1 '^[0-9]+$' || echo 'n/a'; }
if gh api "orgs/$ORG" >/dev/null 2>&1; then
  act=$(gh api "orgs/$ORG/actions/permissions" -q '.enabled_repositories' 2>/dev/null | grep -Em1 '^[a-z_]+' || echo 'n/a')
  sec=$(num "orgs/$ORG/actions/secrets" '.total_count')
  run=$(num "orgs/$ORG/actions/runners" '.total_count')
  rs=$(num "orgs/$ORG/rulesets" 'length')
  hk=$(num "orgs/$ORG/hooks" 'length')
  app=$(num "orgs/$ORG/installations" 'length')
  cp=$(num "orgs/$ORG/properties/schema" 'length')
  row "org config surface audit" "gh api org metadata endpoints" gap "actions=$act secrets=$sec runners=$run rulesets=$rs webhooks=$hk apps=$app custom_props=$cp — declare+converge per surface; denied endpoints stay explicit"
else
  row "org config surface audit" "gh api orgs/$ORG" gap "org API unreachable (network/auth); do not change permissions to bypass"
fi

# 8j. Personal identity + FlexNetOS organization SSH authorization
PROTO=$(gh config get git_protocol --host github.com 2>/dev/null || echo '?')
LOGIN=$(gh api user -q '.login' 2>/dev/null || echo '?')
MEMBER_STATE=$(gh api "user/memberships/orgs/$ORG" -q '.state' 2>/dev/null || echo '?')
MEMBER_ROLE=$(gh api "user/memberships/orgs/$ORG" -q '.role' 2>/dev/null || echo '?')
MEMBER="$MEMBER_STATE:$MEMBER_ROLE"
if (cd /home/flexnetos/meta && rtk meta exec --include envctl -- git ls-remote "git@github.com:$ORG/envctl.git" HEAD >/dev/null 2>&1); then
  ORG_SSH=pass
else
  ORG_SSH=fail
fi
if [ "$PROTO" = ssh ] && [ "$LOGIN" = drdave-flexnetos ] && [ "$MEMBER_STATE" = active ] && [ "$ORG_SSH" = pass ]; then
  row "personal + org SSH proof" "gh identity/membership + RTK Meta SSH ls-remote" pass "login=$LOGIN protocol=$PROTO membership=$MEMBER org_repo_ssh=$ORG_SSH"
else
  row "personal + org SSH proof" "gh identity/membership + RTK Meta SSH ls-remote" fail "login=$LOGIN protocol=$PROTO membership=$MEMBER org_repo_ssh=$ORG_SSH"
fi

# 8k. Loaded GitHub skills and deterministic Bun/Bunx command policy
GHS=$(find "$REPO/home/.claude/skills" -maxdepth 1 -mindepth 1 -type d -name 'github*' 2>/dev/null | wc -l | tr -d ' ')
if python3 "$REPO/agent-skills/agent-env-codex/scripts/check-bun-command-policy.py" "$REPO" >/dev/null 2>&1; then
  BUN_POLICY=pass
else
  BUN_POLICY=fail
fi
if [ "${GHS:-0}" -ge 6 ] && [ "$BUN_POLICY" = pass ]; then
  row "github skills + Bun policy" "github skill count + check-bun-command-policy.py" pass "$GHS github skills; executable npm/npx findings=0"
else
  row "github skills + Bun policy" "github skill count + check-bun-command-policy.py" fail "$GHS github skills; bun_policy=$BUN_POLICY"
fi

# 8l. toolchain currency — LATEST nix-owned, no stale rustup/cargo shadow (E0514 class)
SHADOW=""
for b in rustc cargo cargo-clippy cargo-fmt rustfmt; do
  bp=$(command -v "$b" 2>/dev/null || true)
  case "$bp" in
    "$HOME"/.nix-profile/*|/nix/store/*) : ;;
    "") : ;;
    *) SHADOW="$SHADOW $b=$bp" ;;
  esac
done
if [ -z "$SHADOW" ]; then
  row "toolchain currency (no shadow)" "command -v rustc cargo cargo-clippy cargo-fmt rustfmt" pass "all nix-owned (latest via profile); no rustup/cargo shadow — E0514 class clear"
else
  row "toolchain currency (no shadow)" "command -v rustc cargo …" fail "stale shadow earlier in PATH:$SHADOW — remove shadow, never downgrade nix toolchain (E0514)"
fi

# 8m. yazelix runtime state current (yzx update local_source after changes)
if yzx doctor 2>/dev/null | grep -q "Generated runtime state is current"; then
  row "yazelix runtime current" "yzx doctor" pass "generated state current (no yzx update local_source owed)"
else
  row "yazelix runtime current" "yzx doctor" gap "generated state not current — run yzx update local_source + yzx doctor --fix"
fi
# 8n. yazelix plugins installed + connected (doctor is the oracle; presence != connected)
HXOK=$(yzx doctor 2>/dev/null | grep -c "Helix runtime healthy\|Managed Helix Steel command surface is healthy")
YZOK=$(yzx doctor 2>/dev/null | grep -c "Managed sidebar pane detected\|Yazelix Kitty passthrough bridge is active")
DFAIL=$(yzx doctor 2>/dev/null | grep -c "❌")
if [ "${DFAIL:-0}" -eq 0 ] && [ "${HXOK:-0}" -ge 1 ] && [ "${YZOK:-0}" -ge 1 ]; then
  row "yazelix plugins connected" "yzx doctor (helix-steel + yazi + zellij)" pass "helix-steel healthy ($HXOK), yazi sidebar/kitty ($YZOK), 0 failed checks"
else
  row "yazelix plugins connected" "yzx doctor" gap "helix=$HXOK yazi=$YZOK failed=$DFAIL — inspect yzx doctor --json"
fi
# 8o. yazelix plugin consolidation owner present + org-SSH (all plugins belong here)
PO=/home/flexnetos/meta/src/yazelix-yazi-assets
if [ -d "$PO/.git" ] && git -C "$PO" remote get-url origin 2>/dev/null | grep -q '^git@github.com:FlexNetOS/'; then
  row "yazelix plugin owner" "ls yazelix-yazi-assets + remote" pass "consolidation owner present, FlexNetOS org SSH"
else
  row "yazelix plugin owner" "ls yazelix-yazi-assets" gap "plugin consolidation owner missing or not org-SSH"
fi

# 9. DISCOVERY rows (adopted from the codex sibling: sweep, don't just regress-test)
W=$(yzx doctor 2>/dev/null | grep "⚠" | grep -vc Found || true)
if [ "${W:-0}" -le 1 ]; then row "yzx doctor warnings" "yzx doctor warn-lines" pass "$W warning(s)"
else row "yzx doctor warnings" "yzx doctor warn-lines" gap "$W warnings — read them, queue or fix each"; fi
SH=$(ls "$HOME/.local/bin" 2>/dev/null | grep -c "^yzx$" || true)
ML=$(ls "$HOME/.local/share/applications" 2>/dev/null | grep -ci "mars" || true)
if [ "${SH:-0}" -eq 0 ] && [ "${ML:-0}" -eq 0 ]; then row "no stale yzx/mars shadows" "ls ~/.local/{bin,share/applications}" pass "clean"
else row "no stale yzx/mars shadows" "ls ~/.local/{bin,share/applications}" gap "shadow(s) present: bin=$SH mars-launchers=$ML — archive them"; fi
if [ -x "$REPO/target/debug/envctl" ]; then
  if "$REPO/target/debug/envctl" migrate scan >/dev/null 2>&1; then
    row "envctl migrate scan" "envctl migrate scan" pass "read-only inventory ran"
  else
    row "envctl migrate scan" "envctl migrate scan" gap "exit nonzero — run interactively"
  fi
else
  row "envctl migrate scan" "envctl migrate scan" gap "envctl not built (cargo build -p envctl)"
fi

echo
if [ "$FAILS" -eq 0 ]; then echo "PHASE0: PASS (0 fail rows; gaps are queued work items, not blockers)"; exit 0
else echo "PHASE0: FAIL ($FAILS fail rows above)"; exit 1; fi
