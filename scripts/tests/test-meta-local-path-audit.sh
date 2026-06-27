#!/usr/bin/env bash
set -euo pipefail
root="$(git -C "$(dirname "${BASH_SOURCE[0]}")/../.." rev-parse --show-toplevel)"
tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT

meta="$tmp/meta"
home="$tmp/home"
outside="$tmp/outside"
mkdir -p "$meta/.local/bin" "$meta/usr/bin" "$meta/envctl/home" "$home" "$outside" "$home/.ssh" "$home/.aws" "$home/.cache" "$home/.cargo"
printf '# managed gitconfig\n' >"$meta/envctl/home/.gitconfig"
printf '# managed shell config\n' >"$meta/envctl/home/.zshrc"
printf '# unmanaged shell config\n' >"$home/.zshrc"
printf '#!/usr/bin/env bash\nexit 0\n' >"$meta/usr/bin/hf"
chmod +x "$meta/usr/bin/hf"
printf '#!/usr/bin/env bash\nexit 0\n' >"$outside/hf"
chmod +x "$outside/hf"
ln -s "$outside/hf" "$meta/.local/bin/hf"
ln -s "$meta/envctl/home/.gitconfig" "$home/.gitconfig"
ln -s "$meta/usr/bin/hf" "$home/.inside-link"
ln -s "$outside/hf" "$home/.outside-link"

if "$root/scripts/audit-meta-local-paths.sh" --meta-root "$meta" --real-home "$home" --envctl-home-source "$meta/envctl/home" >"$tmp/pre.out" 2>"$tmp/pre.err"; then
  echo "expected audit to fail before safe migration" >&2
  exit 1
fi
grep -q 'FAIL: .*\.local missing' "$tmp/pre.err"
grep -q 'FAIL: .*\.local/bin/hf resolves outside META_ROOT' "$tmp/pre.err"
grep -q 'FAIL: .*\.gitconfig resolves to' "$tmp/pre.err"

"$root/scripts/audit-meta-local-paths.sh" --apply --meta-root "$meta" --real-home "$home" --envctl-home-source "$meta/envctl/home" >"$tmp/apply.out" 2>"$tmp/apply.err"
"$root/scripts/audit-meta-local-paths.sh" --inventory "$tmp/inventory.tsv" --inventory-summary "$tmp/inventory-summary.tsv" --meta-root "$meta" --real-home "$home" --envctl-home-source "$meta/envctl/home" >"$tmp/post.out" 2>"$tmp/post.err"

test "$(readlink -f "$home/.local")" = "$meta/.local"
test "$(readlink "$home/.gitconfig")" = "$meta/.gitconfig"
test "$(readlink -f "$home/.gitconfig")" = "$meta/envctl/home/.gitconfig"
test "$(readlink -f "$meta/.local/bin/hf")" = "$meta/usr/bin/hf"
grep -q 'WARN: .*\.ssh is real-home state outside META_ROOT; skipped automatic move' "$tmp/post.err"
grep -q 'WARN: .*\.aws is real-home state outside META_ROOT; skipped automatic move' "$tmp/post.err"
grep -q 'WARN: .*\.zshrc is real-home state outside META_ROOT with managed source' "$tmp/post.err"
grep -q 'OK: .*\.inside-link resolves inside META_ROOT' "$tmp/post.out"
grep -q 'WARN: .*\.outside-link symlink resolves outside META_ROOT' "$tmp/post.err"
grep -q 'meta-local audit: PASS' "$tmp/post.out"
grep -q 'dot_entries=9' "$tmp/post.out"

head -n 1 "$tmp/inventory.tsv" | grep -qx $'dot_entry\ttype\tstate\ttarget_class\tcanonical_target\taction\tapply_safe'
awk -F '\t' 'NF != 7 { print "bad inventory row: " $0 >"/dev/stderr"; bad=1 } END { exit bad }' "$tmp/inventory.tsv"
grep -qx $'.local\tsymlink\tmeta-bridge\tbridge\t'"$meta"$'/.local\tensure-symlink\tyes' "$tmp/inventory.tsv"
grep -qx $'.gitconfig\tsymlink\tmanaged-bridge\tmanaged-dotfile\t'"$meta"$'/.gitconfig\tbridge-canonical\tyes' "$tmp/inventory.tsv"
grep -qx $'.zshrc\tfile\treal-home-state\tmanaged-dotfile\t'"$meta"$'/envctl/home/.zshrc\towner-supervised-bridge\tno' "$tmp/inventory.tsv"
grep -qx $'.aws\tdirectory\treal-home-state\tsensitive\t\towner-supervised-vault-or-bridge\tno' "$tmp/inventory.tsv"
grep -qx $'.ssh\tdirectory\treal-home-state\tsensitive\t\towner-supervised-vault-or-bridge\tno' "$tmp/inventory.tsv"
grep -qx $'.cache\tdirectory\treal-home-state\tcache\t'"$meta"$'/.local/cache\tcomponent-managed-cache-migration\tno' "$tmp/inventory.tsv"
grep -qx $'.cargo\tdirectory\treal-home-state\ttoolchain-state\t'"$meta"$'/.toolchains/cargo\tcomponent-managed-toolchain-migration\tno' "$tmp/inventory.tsv"
grep -qx $'.inside-link\tsymlink\talready-meta\talready-meta\t'"$meta"$'/usr/bin/hf\tnone\tn/a' "$tmp/inventory.tsv"
grep -qx $'.outside-link\tsymlink\texternal-symlink\texternal-symlink\t'"$outside"$'/hf\towner-supervised-relink\tno' "$tmp/inventory.tsv"

head -n 1 "$tmp/inventory-summary.tsv" | grep -qx $'target_class\ttotal\tapply_safe_yes\tapply_safe_no\tapply_safe_na\tactions'
grep -qx $'bridge\t1\t1\t0\t0\tensure-symlink' "$tmp/inventory-summary.tsv"
grep -qx $'managed-dotfile\t2\t1\t1\t0\tbridge-canonical,owner-supervised-bridge' "$tmp/inventory-summary.tsv"
grep -qx $'sensitive\t2\t0\t2\t0\towner-supervised-vault-or-bridge' "$tmp/inventory-summary.tsv"
grep -qx $'toolchain-state\t1\t0\t1\t0\tcomponent-managed-toolchain-migration' "$tmp/inventory-summary.tsv"
grep -qx $'already-meta\t1\t0\t0\t1\tnone' "$tmp/inventory-summary.tsv"
grep -qx $'external-symlink\t1\t0\t1\t0\towner-supervised-relink' "$tmp/inventory-summary.tsv"

"$root/scripts/audit-meta-local-paths.sh" --inventory-summary "$tmp/summary-only.tsv" --meta-root "$meta" --real-home "$home" --envctl-home-source "$meta/envctl/home" >"$tmp/summary-only.out" 2>"$tmp/summary-only.err"
grep -qx $'managed-dotfile\t2\t1\t1\t0\tbridge-canonical,owner-supervised-bridge' "$tmp/summary-only.tsv"

# Shell dotfiles need an explicit owner-supervised apply mode: safe cases move/canonicalize
# into META_ROOT, while conflicting canonical files stay untouched for a human merge.
printf '# portable profile\n' >"$home/.profile"
printf '# portable zshenv\n' >"$home/.zshenv"
printf '# real bashrc\n' >"$home/.bashrc"
printf '# canonical bashrc\n' >"$meta/.bashrc"
printf '# duplicate logout\n' >"$home/.bash_logout"
printf '# duplicate logout\n' >"$meta/.bash_logout"

"$root/scripts/audit-meta-local-paths.sh" --inventory "$tmp/shell-pre.tsv" --shell-dotfile-conflict-report "$tmp/shell-conflicts.tsv" --meta-root "$meta" --real-home "$home" --envctl-home-source "$meta/envctl/home" >"$tmp/shell-pre.out" 2>"$tmp/shell-pre.err"
grep -qx $'.profile\tfile\treal-home-state\tshell-dotfile\t'"$meta"$'/.profile\tmove-to-canonical-and-bridge\tyes' "$tmp/shell-pre.tsv"
grep -qx $'.zshenv\tfile\treal-home-state\tshell-dotfile\t'"$meta"$'/.zshenv\tmove-to-canonical-and-bridge\tyes' "$tmp/shell-pre.tsv"
grep -qx $'.bashrc\tfile\treal-home-state\tshell-dotfile\t'"$meta"$'/.bashrc\towner-supervised-merge-and-bridge\tno' "$tmp/shell-pre.tsv"
grep -qx $'.bash_logout\tfile\treal-home-state\tshell-dotfile\t'"$meta"$'/.bash_logout\tbridge-canonical\tyes' "$tmp/shell-pre.tsv"
head -n 1 "$tmp/shell-conflicts.tsv" | grep -qx $'dot_entry\treal_path\tcanonical_target\taction\tapply_safe\treal_sha256\tcanonical_sha256\treal_lines\tcanonical_lines\trecommendation'
awk -F '\t' 'NF != 10 { print "bad shell conflict row: " $0 >"/dev/stderr"; bad=1 } END { exit bad }' "$tmp/shell-conflicts.tsv"
test "$(wc -l <"$tmp/shell-conflicts.tsv" | tr -d '[:space:]')" = 2
awk -F '\t' -v home="$home" -v meta="$meta" '
  $1 == ".bashrc" {
    if ($2 != home "/.bashrc") bad=1
    if ($3 != meta "/.bashrc") bad=1
    if ($4 != "owner-supervised-merge-and-bridge") bad=1
    if ($5 != "no") bad=1
    if ($6 !~ /^[0-9a-f]{64}$/ || $7 !~ /^[0-9a-f]{64}$/) bad=1
    if ($8 != "1" || $9 != "1") bad=1
    if ($10 != "merge-canonical-then-bridge") bad=1
    found=1
  }
  END { exit !(found && !bad) }
' "$tmp/shell-conflicts.tsv"

"$root/scripts/audit-meta-local-paths.sh" --apply --apply-shell-dotfiles --meta-root "$meta" --real-home "$home" --envctl-home-source "$meta/envctl/home" >"$tmp/shell-apply.out" 2>"$tmp/shell-apply.err"
test "$(readlink "$home/.profile")" = "$meta/.profile"
test "$(readlink -f "$home/.profile")" = "$meta/.profile"
grep -qx '# portable profile' "$meta/.profile"
test "$(readlink "$home/.zshenv")" = "$meta/.zshenv"
grep -qx '# portable zshenv' "$meta/.zshenv"
test "$(readlink "$home/.bash_logout")" = "$meta/.bash_logout"
grep -qx '# duplicate logout' "$meta/.bash_logout"
test ! -L "$home/.bashrc"
grep -qx '# real bashrc' "$home/.bashrc"
grep -qx '# canonical bashrc' "$meta/.bashrc"
grep -q 'WARN: .*\.bashrc differs from canonical .* owner-supervised merge required' "$tmp/shell-apply.err"

"$root/scripts/audit-meta-local-paths.sh" --inventory "$tmp/shell-post.tsv" --meta-root "$meta" --real-home "$home" --envctl-home-source "$meta/envctl/home" >"$tmp/shell-post.out" 2>"$tmp/shell-post.err"
grep -qx $'.profile\tsymlink\talready-meta\talready-meta\t'"$meta"$'/.profile\tnone\tn/a' "$tmp/shell-post.tsv"
grep -qx $'.zshenv\tsymlink\talready-meta\talready-meta\t'"$meta"$'/.zshenv\tnone\tn/a' "$tmp/shell-post.tsv"
grep -qx $'.bash_logout\tsymlink\talready-meta\talready-meta\t'"$meta"$'/.bash_logout\tnone\tn/a' "$tmp/shell-post.tsv"
grep -qx $'.bashrc\tfile\treal-home-state\tshell-dotfile\t'"$meta"$'/.bashrc\towner-supervised-merge-and-bridge\tno' "$tmp/shell-post.tsv"

# Explicit dot migration is opt-in, allow-listed, dry-run by default, and preserves any existing
# canonical META_ROOT target by archiving the real-home state inside META_ROOT instead of clobbering.
mig_meta="$tmp/mig-meta"
mig_home="$tmp/mig-home"
mkdir -p \
  "$mig_meta/.local" \
  "$mig_meta/envctl/home" \
  "$mig_home/.cargo" \
  "$mig_home/.npm" \
  "$mig_home/.dotnet" \
  "$mig_home/.unknown-ai-sensitive/cache" \
  "$mig_home/.unknown-ai-sensitive/tokens" \
  "$mig_home/.gemini" \
  "$mig_home/.ai/mcp" \
  "$mig_home/.jetbrains" \
  "$mig_home/.meta/plugins" \
  "$mig_home/.java/.userPrefs/jetbrains/auth-tokens" \
  "$mig_home/.java/fonts/25.0.3" \
  "$mig_home/.nv/ComputeCache/0/7" \
  "$mig_home/.archon" \
  "$mig_home/.hermes" \
  "$mig_home/.n8n-mcp" \
  "$mig_home/.gphoto" \
  "$mig_home/.junie" \
  "$mig_home/.vscode-shared/sharedStorage" \
  "$mig_home/.repomix/outputs" \
  "$mig_home/.junie/sessions" \
  "$mig_home/.junie/mcp" \
  "$mig_home/.junie/versions/1892.22/skills" \
  "$mig_home/.kimi-code" \
  "$mig_home/.ollama" \
  "$mig_meta/.local/share/junie/current" \
  "$mig_meta/.local/share/junie/updates" \
  "$mig_meta/.local/share/junie/versions/1892.22"
printf '# managed gitconfig\n' >"$mig_meta/envctl/home/.gitconfig"
ln -s "$mig_meta/envctl/home/.gitconfig" "$mig_meta/.gitconfig"
ln -s "$mig_meta/.gitconfig" "$mig_home/.gitconfig"
ln -s "$mig_meta/.local" "$mig_home/.local"
printf 'real-home cargo state\n' >"$mig_home/.cargo/config"
printf 'ai-profile\n' >"$mig_home/.unknown-ai-sensitive/settings.toml"
printf 'secret\n' >"$mig_home/.unknown-ai-sensitive/tokens/api-token"
ln -s ../settings.toml "$mig_home/.unknown-ai-sensitive/cache/settings-link"
printf 'real-home npm state\n' >"$mig_home/.npm/npmrc"
printf 'real-home dotnet state\n' >"$mig_home/.dotnet/state"
printf 'set ideajoin\n' >"$mig_home/.ideavimrc"
printf '{ "theme": "dark" }\n' >"$mig_home/.gemini/settings.json"
printf '' >"$mig_home/.ai/mcp/mcp.json"
printf '{"default_mcp_settings":{},"agent_servers":{"goose":{"command":"/usr/bin/goose","args":["acp"]},"kimi":{"command":"/home/drdave/.local/bin/kimi","args":["--acp"]}}}\n' >"$mig_home/.jetbrains/acp.json"
printf '{"context":{},"timestamp":"2026-06-27T00:00:00Z","workspace_root":"/home/drdave/Desktop/meta"}\n' >"$mig_home/.meta/context_cache.json"
printf '{"worktrees":[]}\n' >"$mig_home/.meta/worktree.json"
touch "$mig_home/.java/.userPrefs/.user.lock.drdave"
touch "$mig_home/.java/.userPrefs/.userRootModFile.drdave"
printf '<map MAP_XML_VERSION="1.0"><entry key="sample" value="present"/></map>\n' >"$mig_home/.java/.userPrefs/jetbrains/auth-tokens/prefs.xml"
printf 'font-cache\n' >"$mig_home/.java/fonts/25.0.3/fcinfo.properties"
chmod 600 "$mig_home/.java/.userPrefs/.user.lock.drdave" "$mig_home/.java/.userPrefs/.userRootModFile.drdave" "$mig_home/.java/fonts/25.0.3/fcinfo.properties"
printf 'cache-index\n' >"$mig_home/.nv/ComputeCache/index"
printf 'compiled-kernel\n' >"$mig_home/.nv/ComputeCache/0/7/kernel.bin"
chmod 700 "$mig_home/.nv" "$mig_home/.nv/ComputeCache" "$mig_home/.nv/ComputeCache/0" "$mig_home/.nv/ComputeCache/0/7"
chmod 600 "$mig_home/.nv/ComputeCache/index" "$mig_home/.nv/ComputeCache/0/7/kernel.bin"
printf '{ "lastChecked": "2026-06-27T00:00:00Z" }\n' >"$mig_home/.archon/update-check.json"
printf 'provider: ollama\nbase_url: http://localhost:11434/v1\n' >"$mig_home/.hermes/config.yaml"
printf '{ "telemetry": false }\n' >"$mig_home/.n8n-mcp/telemetry.json"
printf 'camera-port=usb\n' >"$mig_home/.gphoto/settings"
printf 'sqlite-state\n' >"$mig_home/.vscode-shared/sharedStorage/state.vscdb"
printf 'repomix-output\n' >"$mig_home/.repomix/outputs/latest.txt"
printf '{ "theme": "light" }\n' >"$mig_home/.junie/settings.json"
printf '{ "secrets": {} }\n' >"$mig_home/.junie/secure_credentials.json"
chmod 600 "$mig_home/.junie/secure_credentials.json"
printf '{ "mcpServers": {} }\n' >"$mig_home/.junie/mcp/mcp.json"
printf 'session event\n' >"$mig_home/.junie/sessions/events.jsonl"
printf 'skill note\n' >"$mig_home/.junie/versions/1892.22/skills/local.md"
printf '#!/usr/bin/env bash\nexit 0\n' >"$mig_meta/.local/share/junie/current/junie"
chmod +x "$mig_meta/.local/share/junie/current/junie"
printf '{ "pending": true }\n' >"$mig_meta/.local/share/junie/updates/pending-update.json"
printf 'bundled app asset\n' >"$mig_meta/.local/share/junie/versions/1892.22/app.txt"
printf 'real-home kimi-code credentials\n' >"$mig_home/.kimi-code/credentials.json"
printf 'real-home ollama history\n' >"$mig_home/.ollama/history"
printf '{ "mcpServers": {} }\n' >"$mig_home/.claude.json"

"$root/scripts/audit-meta-local-paths.sh" --inventory "$tmp/app-config-inventory.tsv" --meta-root "$mig_meta" --real-home "$mig_home" --envctl-home-source "$mig_meta/envctl/home" >"$tmp/app-config-inventory.out" 2>"$tmp/app-config-inventory.err"
grep -qx $'.gemini\tdirectory\treal-home-state\tapp-config-state\t'"$mig_meta"$'/.local/share/gemini\towner-supervised-config-migration\tno' "$tmp/app-config-inventory.tsv"
grep -qx $'.ai\tdirectory\treal-home-state\tapp-config-state\t'"$mig_meta"$'/.local/share/ai\tmigrate-dir-to-meta-share-and-bridge\tyes' "$tmp/app-config-inventory.tsv"
grep -qx $'.jetbrains\tdirectory\treal-home-state\tapp-config-state\t'"$mig_meta"$'/.local/share/jetbrains\tmigrate-dir-to-meta-share-and-bridge\tyes' "$tmp/app-config-inventory.tsv"
grep -qx $'.meta\tdirectory\treal-home-state\tapp-config-state\t'"$mig_meta"$'/.local/share/meta\tmigrate-dir-to-meta-share-and-bridge\tyes' "$tmp/app-config-inventory.tsv"
grep -qx $'.java\tdirectory\treal-home-state\tapp-config-state\t'"$mig_meta"$'/.local/share/java\tmigrate-dir-to-meta-share-and-bridge\tyes' "$tmp/app-config-inventory.tsv"
grep -qx $'.nv\tdirectory\treal-home-state\tcache\t'"$mig_meta"$'/.local/cache/nvidia\tmigrate-dir-to-meta-cache-and-bridge\tyes' "$tmp/app-config-inventory.tsv"
grep -qx $'.gphoto\tdirectory\treal-home-state\tapp-config-state\t'"$mig_meta"$'/.config/gphoto\tmigrate-dir-to-meta-config-and-bridge\tyes' "$tmp/app-config-inventory.tsv"
grep -qx $'.archon\tdirectory\treal-home-state\tapp-config-state\t'"$mig_meta"$'/.local/share/archon\tmigrate-dir-to-meta-share-and-bridge\tyes' "$tmp/app-config-inventory.tsv"
grep -qx $'.hermes\tdirectory\treal-home-state\tapp-config-state\t'"$mig_meta"$'/.local/share/hermes\tmigrate-dir-to-meta-share-and-bridge\tyes' "$tmp/app-config-inventory.tsv"
grep -qx $'.n8n-mcp\tdirectory\treal-home-state\tapp-config-state\t'"$mig_meta"$'/.local/share/n8n-mcp\tmigrate-dir-to-meta-share-and-bridge\tyes' "$tmp/app-config-inventory.tsv"
grep -qx $'.vscode-shared\tdirectory\treal-home-state\tapp-config-state\t'"$mig_meta"$'/.local/share/vscode-shared\tmigrate-dir-to-meta-share-and-bridge\tyes' "$tmp/app-config-inventory.tsv"
grep -qx $'.repomix\tdirectory\treal-home-state\tapp-config-state\t'"$mig_meta"$'/.local/share/repomix\tmigrate-dir-to-meta-share-and-bridge\tyes' "$tmp/app-config-inventory.tsv"
grep -qx $'.junie\tdirectory\treal-home-state\tapp-config-state\t'"$mig_meta"$'/.local/share/junie\tmerge-dir-to-existing-meta-share-and-bridge\tyes' "$tmp/app-config-inventory.tsv"
grep -qx $'.kimi-code\tdirectory\treal-home-state\tapp-config-state\t'"$mig_meta"$'/.local/share/kimi-code\towner-supervised-config-migration\tno' "$tmp/app-config-inventory.tsv"
grep -qx $'.ollama\tdirectory\treal-home-state\tapp-config-state\t'"$mig_meta"$'/var/lib/ollama\towner-supervised-config-migration\tno' "$tmp/app-config-inventory.tsv"
grep -qx $'.claude.json\tfile\treal-home-state\tapp-config-state\t'"$mig_meta"$'/.local/share/claude/claude.json\towner-supervised-config-migration\tno' "$tmp/app-config-inventory.tsv"
grep -qx $'.ideavimrc\tfile\treal-home-state\tapp-config-state\t'"$mig_meta"$'/.ideavimrc\tmigrate-file-to-meta-root-and-bridge\tyes' "$tmp/app-config-inventory.tsv"

"$root/scripts/audit-meta-local-paths.sh" --app-config-conflict-report "$tmp/app-config-conflicts.tsv" --meta-root "$mig_meta" --real-home "$mig_home" --envctl-home-source "$mig_meta/envctl/home" >"$tmp/app-config-conflicts.out" 2>"$tmp/app-config-conflicts.err"
head -n 1 "$tmp/app-config-conflicts.tsv" | grep -qx $'dot_entry\treal_path\tcanonical_target\taction\tapply_safe\treal_type\tcanonical_type\treal_digest\tcanonical_digest\treal_entries\tcanonical_entries\trecommendation'
awk -F '\t' 'NF != 12 { print "bad app-config conflict row: " $0 >"/dev/stderr"; bad=1 } END { exit bad }' "$tmp/app-config-conflicts.tsv"
test "$(wc -l <"$tmp/app-config-conflicts.tsv" | tr -d '[:space:]')" = 1
if awk -F '\t' '$1 == ".junie" { found=1 } END { exit !found }' "$tmp/app-config-conflicts.tsv"; then
  echo "unexpected app-config conflict report row for merge-safe .junie target" >&2
  exit 1
fi
if awk -F '\t' '$1 == ".gemini" { found=1 } END { exit !found }' "$tmp/app-config-conflicts.tsv"; then
  echo "unexpected app-config conflict report row for missing canonical .gemini target" >&2
  exit 1
fi

mkdir -p "$mig_home/.ssh"
printf 'key\n' >"$mig_home/.ssh/id_ed25519"
"$root/scripts/audit-meta-local-paths.sh" --unknown-app-config-report "$tmp/unknown-app-config.tsv" --meta-root "$mig_meta" --real-home "$mig_home" --envctl-home-source "$mig_meta/envctl/home" >"$tmp/unknown-app-config.out" 2>"$tmp/unknown-app-config.err"
head -n 1 "$tmp/unknown-app-config.tsv" | grep -qx $'dot_entry	real_path	type	digest	entries	direct_files	direct_dirs	symlinks	sensitive_hints	recommendation'
awk -F '\t' 'NF != 10 { print "bad unknown app-config row: " $0 >"/dev/stderr"; bad=1 } END { exit bad }' "$tmp/unknown-app-config.tsv"
test "$(wc -l <"$tmp/unknown-app-config.tsv" | tr -d '[:space:]')" = 2
awk -F '\t' -v home="$mig_home" '
  $1 == ".unknown-ai-sensitive" {
    if ($2 != home "/.unknown-ai-sensitive") bad=1
    if ($3 != "directory") bad=1
    if ($4 !~ /^[0-9a-f]{64}$/) bad=1
    if ($5 != "5") bad=1
    if ($6 != "1") bad=1
    if ($7 != "2") bad=1
    if ($8 != "1") bad=1
    if ($9 != "2") bad=1
    if ($10 != "classify-canonical-target-before-migration") bad=1
    found=1
  }
  END { exit !(found && !bad) }
' "$tmp/unknown-app-config.tsv"
if awk -F '\t' '$1 == ".gemini" { found=1 } END { exit !found }' "$tmp/unknown-app-config.tsv"; then
  echo "unexpected unknown app-config report row for known canonical .gemini target" >&2
  exit 1
fi
if awk -F '\t' '$1 == ".ai" { found=1 } END { exit !found }' "$tmp/unknown-app-config.tsv"; then
  echo "unexpected unknown app-config report row for allow-listed .ai target" >&2
  exit 1
fi
if awk -F '\t' '$1 == ".jetbrains" { found=1 } END { exit !found }' "$tmp/unknown-app-config.tsv"; then
  echo "unexpected unknown app-config report row for allow-listed .jetbrains target" >&2
  exit 1
fi
if awk -F '\t' '$1 == ".meta" { found=1 } END { exit !found }' "$tmp/unknown-app-config.tsv"; then
  echo "unexpected unknown app-config report row for allow-listed .meta target" >&2
  exit 1
fi
if awk -F '\t' '$1 == ".java" { found=1 } END { exit !found }' "$tmp/unknown-app-config.tsv"; then
  echo "unexpected unknown app-config report row for allow-listed .java target" >&2
  exit 1
fi
if awk -F '\t' '$1 == ".nv" { found=1 } END { exit !found }' "$tmp/unknown-app-config.tsv"; then
  echo "unexpected unknown app-config report row for allow-listed .nv cache target" >&2
  exit 1
fi
if awk -F '\t' '$1 == ".archon" { found=1 } END { exit !found }' "$tmp/unknown-app-config.tsv"; then
  echo "unexpected unknown app-config report row for allow-listed .archon target" >&2
  exit 1
fi
if awk -F '\t' '$1 == ".hermes" { found=1 } END { exit !found }' "$tmp/unknown-app-config.tsv"; then
  echo "unexpected unknown app-config report row for allow-listed .hermes target" >&2
  exit 1
fi
if awk -F '\t' '$1 == ".n8n-mcp" { found=1 } END { exit !found }' "$tmp/unknown-app-config.tsv"; then
  echo "unexpected unknown app-config report row for allow-listed .n8n-mcp target" >&2
  exit 1
fi
if awk -F '\t' '$1 == ".ssh" { found=1 } END { exit !found }' "$tmp/unknown-app-config.tsv"; then
  echo "unexpected unknown app-config report row for sensitive .ssh" >&2
  exit 1
fi

"$root/scripts/audit-meta-local-paths.sh" --migrate-dot .cargo --meta-root "$mig_meta" --real-home "$mig_home" --envctl-home-source "$mig_meta/envctl/home" >"$tmp/migrate-dry.out" 2>"$tmp/migrate-dry.err"
grep -q 'DRY-RUN: would move .*\.cargo to .*\.toolchains/cargo' "$tmp/migrate-dry.out"
test -d "$mig_home/.cargo"
test ! -e "$mig_meta/.toolchains/cargo"

"$root/scripts/audit-meta-local-paths.sh" --migrate-dot .gemini --meta-root "$mig_meta" --real-home "$mig_home" --envctl-home-source "$mig_meta/envctl/home" >"$tmp/migrate-gemini-dry.out" 2>"$tmp/migrate-gemini-dry.err"
grep -q 'DRY-RUN: would move .*\.gemini to .*\.local/share/gemini' "$tmp/migrate-gemini-dry.out"
test -d "$mig_home/.gemini"
test ! -e "$mig_meta/.local/share/gemini"

"$root/scripts/audit-meta-local-paths.sh" --migrate-dot .ideavimrc --meta-root "$mig_meta" --real-home "$mig_home" --envctl-home-source "$mig_meta/envctl/home" >"$tmp/migrate-ideavim-dry.out" 2>"$tmp/migrate-ideavim-dry.err"
grep -q 'DRY-RUN: would move .*\.ideavimrc to .*\.ideavimrc' "$tmp/migrate-ideavim-dry.out"
test -f "$mig_home/.ideavimrc"
test ! -e "$mig_meta/.ideavimrc"

"$root/scripts/audit-meta-local-paths.sh" --apply --migrate-dot .ideavimrc --meta-root "$mig_meta" --real-home "$mig_home" --envctl-home-source "$mig_meta/envctl/home" >"$tmp/migrate-ideavim.out" 2>"$tmp/migrate-ideavim.err"
test "$(readlink -f "$mig_home/.ideavimrc")" = "$mig_meta/.ideavimrc"
grep -qx 'set ideajoin' "$mig_meta/.ideavimrc"

"$root/scripts/audit-meta-local-paths.sh" --inventory "$tmp/migrate-file-post.tsv" --meta-root "$mig_meta" --real-home "$mig_home" --envctl-home-source "$mig_meta/envctl/home" >"$tmp/migrate-file-post.out" 2>"$tmp/migrate-file-post.err"
grep -qx $'.ideavimrc\tsymlink\talready-meta\talready-meta\t'"$mig_meta"$'/.ideavimrc\tnone\tn/a' "$tmp/migrate-file-post.tsv"

"$root/scripts/audit-meta-local-paths.sh" --apply --migrate-dot .npm --meta-root "$mig_meta" --real-home "$mig_home" --envctl-home-source "$mig_meta/envctl/home" >"$tmp/migrate-npm.out" 2>"$tmp/migrate-npm.err"
test "$(readlink -f "$mig_home/.npm")" = "$mig_meta/.toolchains/npm"
test -f "$mig_meta/.toolchains/npm/npmrc"

"$root/scripts/audit-meta-local-paths.sh" --apply --migrate-dot .dotnet --meta-root "$mig_meta" --real-home "$mig_home" --envctl-home-source "$mig_meta/envctl/home" >"$tmp/migrate-dotnet.out" 2>"$tmp/migrate-dotnet.err"
test "$(readlink -f "$mig_home/.dotnet")" = "$mig_meta/.toolchains/dotnet"
test -f "$mig_meta/.toolchains/dotnet/state"

"$root/scripts/audit-meta-local-paths.sh" --apply --migrate-dot .kimi-code --meta-root "$mig_meta" --real-home "$mig_home" --envctl-home-source "$mig_meta/envctl/home" >"$tmp/migrate-kimi-code.out" 2>"$tmp/migrate-kimi-code.err"
test "$(readlink -f "$mig_home/.kimi-code")" = "$mig_meta/.local/share/kimi-code"
test -f "$mig_meta/.local/share/kimi-code/credentials.json"

"$root/scripts/audit-meta-local-paths.sh" --apply --migrate-dot .ollama --meta-root "$mig_meta" --real-home "$mig_home" --envctl-home-source "$mig_meta/envctl/home" >"$tmp/migrate-ollama.out" 2>"$tmp/migrate-ollama.err"
test "$(readlink -f "$mig_home/.ollama")" = "$mig_meta/var/lib/ollama"
test -f "$mig_meta/var/lib/ollama/history"

"$root/scripts/audit-meta-local-paths.sh" --apply --migrate-dot .claude.json --meta-root "$mig_meta" --real-home "$mig_home" --envctl-home-source "$mig_meta/envctl/home" >"$tmp/migrate-claude-json.out" 2>"$tmp/migrate-claude-json.err"
test "$(readlink -f "$mig_home/.claude.json")" = "$mig_meta/.local/share/claude/claude.json"
test -f "$mig_meta/.local/share/claude/claude.json"

"$root/scripts/audit-meta-local-paths.sh" --migrate-dot .ai --meta-root "$mig_meta" --real-home "$mig_home" --envctl-home-source "$mig_meta/envctl/home" >"$tmp/migrate-ai-dry.out" 2>"$tmp/migrate-ai-dry.err"
grep -q 'DRY-RUN: would move .*\.ai to .*\.local/share/ai' "$tmp/migrate-ai-dry.out"
test -d "$mig_home/.ai"
test ! -e "$mig_meta/.local/share/ai"

"$root/scripts/audit-meta-local-paths.sh" --apply --migrate-dot .ai --meta-root "$mig_meta" --real-home "$mig_home" --envctl-home-source "$mig_meta/envctl/home" >"$tmp/migrate-ai.out" 2>"$tmp/migrate-ai.err"
test "$(readlink -f "$mig_home/.ai")" = "$mig_meta/.local/share/ai"
test -f "$mig_meta/.local/share/ai/mcp/mcp.json"
test ! -s "$mig_meta/.local/share/ai/mcp/mcp.json"

"$root/scripts/audit-meta-local-paths.sh" --inventory "$tmp/ai-post.tsv" --inventory-summary "$tmp/ai-post-summary.tsv" --meta-root "$mig_meta" --real-home "$mig_home" --envctl-home-source "$mig_meta/envctl/home" >"$tmp/ai-post.out" 2>"$tmp/ai-post.err"
grep -qx $'.ai	symlink	already-meta	already-meta	'"$mig_meta"$'/.local/share/ai	none	n/a' "$tmp/ai-post.tsv"

"$root/scripts/audit-meta-local-paths.sh" --migrate-dot .jetbrains --meta-root "$mig_meta" --real-home "$mig_home" --envctl-home-source "$mig_meta/envctl/home" >"$tmp/migrate-jetbrains-dry.out" 2>"$tmp/migrate-jetbrains-dry.err"
grep -q 'DRY-RUN: would move .*\.jetbrains to .*\.local/share/jetbrains' "$tmp/migrate-jetbrains-dry.out"
test -d "$mig_home/.jetbrains"
test ! -e "$mig_meta/.local/share/jetbrains"

"$root/scripts/audit-meta-local-paths.sh" --apply --migrate-dot .jetbrains --meta-root "$mig_meta" --real-home "$mig_home" --envctl-home-source "$mig_meta/envctl/home" >"$tmp/migrate-jetbrains.out" 2>"$tmp/migrate-jetbrains.err"
test "$(readlink -f "$mig_home/.jetbrains")" = "$mig_meta/.local/share/jetbrains"
grep -Fqx '{"default_mcp_settings":{},"agent_servers":{"goose":{"command":"/usr/bin/goose","args":["acp"]},"kimi":{"command":"/home/drdave/.local/bin/kimi","args":["--acp"]}}}' "$mig_meta/.local/share/jetbrains/acp.json"

"$root/scripts/audit-meta-local-paths.sh" --inventory "$tmp/jetbrains-post.tsv" --inventory-summary "$tmp/jetbrains-post-summary.tsv" --meta-root "$mig_meta" --real-home "$mig_home" --envctl-home-source "$mig_meta/envctl/home" >"$tmp/jetbrains-post.out" 2>"$tmp/jetbrains-post.err"
grep -qx $'.jetbrains	symlink	already-meta	already-meta	'"$mig_meta"$'/.local/share/jetbrains	none	n/a' "$tmp/jetbrains-post.tsv"

"$root/scripts/audit-meta-local-paths.sh" --migrate-dot .meta --meta-root "$mig_meta" --real-home "$mig_home" --envctl-home-source "$mig_meta/envctl/home" >"$tmp/migrate-meta-dry.out" 2>"$tmp/migrate-meta-dry.err"
grep -q 'DRY-RUN: would move .*\.meta to .*\.local/share/meta' "$tmp/migrate-meta-dry.out"
test -d "$mig_home/.meta"
test ! -e "$mig_meta/.local/share/meta"

"$root/scripts/audit-meta-local-paths.sh" --apply --migrate-dot .meta --meta-root "$mig_meta" --real-home "$mig_home" --envctl-home-source "$mig_meta/envctl/home" >"$tmp/migrate-meta.out" 2>"$tmp/migrate-meta.err"
test "$(readlink -f "$mig_home/.meta")" = "$mig_meta/.local/share/meta"
grep -Fqx '{"worktrees":[]}' "$mig_meta/.local/share/meta/worktree.json"
grep -Fqx '{"context":{},"timestamp":"2026-06-27T00:00:00Z","workspace_root":"/home/drdave/Desktop/meta"}' "$mig_meta/.local/share/meta/context_cache.json"
test -d "$mig_meta/.local/share/meta/plugins"

"$root/scripts/audit-meta-local-paths.sh" --inventory "$tmp/meta-post.tsv" --inventory-summary "$tmp/meta-post-summary.tsv" --meta-root "$mig_meta" --real-home "$mig_home" --envctl-home-source "$mig_meta/envctl/home" >"$tmp/meta-post.out" 2>"$tmp/meta-post.err"
grep -qx $'.meta	symlink	already-meta	already-meta	'"$mig_meta"$'/.local/share/meta	none	n/a' "$tmp/meta-post.tsv"

"$root/scripts/audit-meta-local-paths.sh" --migrate-dot .java --meta-root "$mig_meta" --real-home "$mig_home" --envctl-home-source "$mig_meta/envctl/home" >"$tmp/migrate-java-dry.out" 2>"$tmp/migrate-java-dry.err"
grep -q 'DRY-RUN: would move .*\.java to .*\.local/share/java' "$tmp/migrate-java-dry.out"
test -d "$mig_home/.java"
test ! -e "$mig_meta/.local/share/java"

"$root/scripts/audit-meta-local-paths.sh" --apply --migrate-dot .java --meta-root "$mig_meta" --real-home "$mig_home" --envctl-home-source "$mig_meta/envctl/home" >"$tmp/migrate-java.out" 2>"$tmp/migrate-java.err"
test "$(readlink -f "$mig_home/.java")" = "$mig_meta/.local/share/java"
grep -Fqx '<map MAP_XML_VERSION="1.0"><entry key="sample" value="present"/></map>' "$mig_meta/.local/share/java/.userPrefs/jetbrains/auth-tokens/prefs.xml"
grep -Fqx 'font-cache' "$mig_meta/.local/share/java/fonts/25.0.3/fcinfo.properties"
test "$(stat -c %a "$mig_meta/.local/share/java/.userPrefs/.user.lock.drdave")" = "600"
test "$(stat -c %a "$mig_meta/.local/share/java/.userPrefs/.userRootModFile.drdave")" = "600"
test "$(stat -c %a "$mig_meta/.local/share/java/fonts/25.0.3/fcinfo.properties")" = "600"

"$root/scripts/audit-meta-local-paths.sh" --inventory "$tmp/java-post.tsv" --inventory-summary "$tmp/java-post-summary.tsv" --meta-root "$mig_meta" --real-home "$mig_home" --envctl-home-source "$mig_meta/envctl/home" >"$tmp/java-post.out" 2>"$tmp/java-post.err"
grep -qx $'.java	symlink	already-meta	already-meta	'"$mig_meta"$'/.local/share/java	none	n/a' "$tmp/java-post.tsv"

"$root/scripts/audit-meta-local-paths.sh" --migrate-dot .nv --meta-root "$mig_meta" --real-home "$mig_home" --envctl-home-source "$mig_meta/envctl/home" >"$tmp/migrate-nv-dry.out" 2>"$tmp/migrate-nv-dry.err"
grep -q 'DRY-RUN: would move .*\.nv to .*\.local/cache/nvidia' "$tmp/migrate-nv-dry.out"
test -d "$mig_home/.nv"
test ! -e "$mig_meta/.local/cache/nvidia"

"$root/scripts/audit-meta-local-paths.sh" --apply --migrate-dot .nv --meta-root "$mig_meta" --real-home "$mig_home" --envctl-home-source "$mig_meta/envctl/home" >"$tmp/migrate-nv.out" 2>"$tmp/migrate-nv.err"
test "$(readlink -f "$mig_home/.nv")" = "$mig_meta/.local/cache/nvidia"
grep -Fqx 'cache-index' "$mig_meta/.local/cache/nvidia/ComputeCache/index"
grep -Fqx 'compiled-kernel' "$mig_meta/.local/cache/nvidia/ComputeCache/0/7/kernel.bin"
test "$(stat -c %a "$mig_meta/.local/cache/nvidia")" = "700"
test "$(stat -c %a "$mig_meta/.local/cache/nvidia/ComputeCache/index")" = "600"

"$root/scripts/audit-meta-local-paths.sh" --inventory "$tmp/nv-post.tsv" --inventory-summary "$tmp/nv-post-summary.tsv" --meta-root "$mig_meta" --real-home "$mig_home" --envctl-home-source "$mig_meta/envctl/home" >"$tmp/nv-post.out" 2>"$tmp/nv-post.err"
grep -qx $'.nv	symlink	already-meta	already-meta	'"$mig_meta"$'/.local/cache/nvidia	none	n/a' "$tmp/nv-post.tsv"

"$root/scripts/audit-meta-local-paths.sh" --migrate-dot .archon --meta-root "$mig_meta" --real-home "$mig_home" --envctl-home-source "$mig_meta/envctl/home" >"$tmp/migrate-archon-dry.out" 2>"$tmp/migrate-archon-dry.err"
grep -q 'DRY-RUN: would move .*\.archon to .*\.local/share/archon' "$tmp/migrate-archon-dry.out"
test -d "$mig_home/.archon"
test ! -e "$mig_meta/.local/share/archon"

"$root/scripts/audit-meta-local-paths.sh" --apply --migrate-dot .archon --meta-root "$mig_meta" --real-home "$mig_home" --envctl-home-source "$mig_meta/envctl/home" >"$tmp/migrate-archon.out" 2>"$tmp/migrate-archon.err"
test "$(readlink -f "$mig_home/.archon")" = "$mig_meta/.local/share/archon"
grep -qx '{ "lastChecked": "2026-06-27T00:00:00Z" }' "$mig_meta/.local/share/archon/update-check.json"

"$root/scripts/audit-meta-local-paths.sh" --inventory "$tmp/archon-post.tsv" --inventory-summary "$tmp/archon-post-summary.tsv" --meta-root "$mig_meta" --real-home "$mig_home" --envctl-home-source "$mig_meta/envctl/home" >"$tmp/archon-post.out" 2>"$tmp/archon-post.err"
grep -qx $'.archon\tsymlink\talready-meta\talready-meta\t'"$mig_meta"$'/.local/share/archon\tnone\tn/a' "$tmp/archon-post.tsv"

"$root/scripts/audit-meta-local-paths.sh" --migrate-dot .hermes --meta-root "$mig_meta" --real-home "$mig_home" --envctl-home-source "$mig_meta/envctl/home" >"$tmp/migrate-hermes-dry.out" 2>"$tmp/migrate-hermes-dry.err"
grep -q 'DRY-RUN: would move .*\.hermes to .*\.local/share/hermes' "$tmp/migrate-hermes-dry.out"
test -d "$mig_home/.hermes"
test ! -e "$mig_meta/.local/share/hermes"

"$root/scripts/audit-meta-local-paths.sh" --apply --migrate-dot .hermes --meta-root "$mig_meta" --real-home "$mig_home" --envctl-home-source "$mig_meta/envctl/home" >"$tmp/migrate-hermes.out" 2>"$tmp/migrate-hermes.err"
test "$(readlink -f "$mig_home/.hermes")" = "$mig_meta/.local/share/hermes"
grep -qx 'provider: ollama' "$mig_meta/.local/share/hermes/config.yaml"
grep -qx 'base_url: http://localhost:11434/v1' "$mig_meta/.local/share/hermes/config.yaml"

"$root/scripts/audit-meta-local-paths.sh" --inventory "$tmp/hermes-post.tsv" --inventory-summary "$tmp/hermes-post-summary.tsv" --meta-root "$mig_meta" --real-home "$mig_home" --envctl-home-source "$mig_meta/envctl/home" >"$tmp/hermes-post.out" 2>"$tmp/hermes-post.err"
grep -qx $'.hermes\tsymlink\talready-meta\talready-meta\t'"$mig_meta"$'/.local/share/hermes\tnone\tn/a' "$tmp/hermes-post.tsv"

"$root/scripts/audit-meta-local-paths.sh" --migrate-dot .n8n-mcp --meta-root "$mig_meta" --real-home "$mig_home" --envctl-home-source "$mig_meta/envctl/home" >"$tmp/migrate-n8n-mcp-dry.out" 2>"$tmp/migrate-n8n-mcp-dry.err"
grep -q 'DRY-RUN: would move .*\.n8n-mcp to .*\.local/share/n8n-mcp' "$tmp/migrate-n8n-mcp-dry.out"
test -d "$mig_home/.n8n-mcp"
test ! -e "$mig_meta/.local/share/n8n-mcp"

"$root/scripts/audit-meta-local-paths.sh" --apply --migrate-dot .n8n-mcp --meta-root "$mig_meta" --real-home "$mig_home" --envctl-home-source "$mig_meta/envctl/home" >"$tmp/migrate-n8n-mcp.out" 2>"$tmp/migrate-n8n-mcp.err"
test "$(readlink -f "$mig_home/.n8n-mcp")" = "$mig_meta/.local/share/n8n-mcp"
grep -qx '{ "telemetry": false }' "$mig_meta/.local/share/n8n-mcp/telemetry.json"

"$root/scripts/audit-meta-local-paths.sh" --inventory "$tmp/n8n-mcp-post.tsv" --inventory-summary "$tmp/n8n-mcp-post-summary.tsv" --meta-root "$mig_meta" --real-home "$mig_home" --envctl-home-source "$mig_meta/envctl/home" >"$tmp/n8n-mcp-post.out" 2>"$tmp/n8n-mcp-post.err"
grep -qx $'.n8n-mcp\tsymlink\talready-meta\talready-meta\t'"$mig_meta"$'/.local/share/n8n-mcp\tnone\tn/a' "$tmp/n8n-mcp-post.tsv"

"$root/scripts/audit-meta-local-paths.sh" --migrate-dot .gphoto --meta-root "$mig_meta" --real-home "$mig_home" --envctl-home-source "$mig_meta/envctl/home" >"$tmp/migrate-gphoto-dry.out" 2>"$tmp/migrate-gphoto-dry.err"
grep -q 'DRY-RUN: would move .*\.gphoto to .*\.config/gphoto' "$tmp/migrate-gphoto-dry.out"
test -d "$mig_home/.gphoto"
test ! -e "$mig_meta/.config/gphoto"

"$root/scripts/audit-meta-local-paths.sh" --apply --migrate-dot .gphoto --meta-root "$mig_meta" --real-home "$mig_home" --envctl-home-source "$mig_meta/envctl/home" >"$tmp/migrate-gphoto.out" 2>"$tmp/migrate-gphoto.err"
test "$(readlink -f "$mig_home/.gphoto")" = "$mig_meta/.config/gphoto"
grep -qx 'camera-port=usb' "$mig_meta/.config/gphoto/settings"

"$root/scripts/audit-meta-local-paths.sh" --inventory "$tmp/gphoto-post.tsv" --inventory-summary "$tmp/gphoto-post-summary.tsv" --meta-root "$mig_meta" --real-home "$mig_home" --envctl-home-source "$mig_meta/envctl/home" >"$tmp/gphoto-post.out" 2>"$tmp/gphoto-post.err"
grep -qx $'.gphoto\tsymlink\talready-meta\talready-meta\t'"$mig_meta"$'/.config/gphoto\tnone\tn/a' "$tmp/gphoto-post.tsv"

"$root/scripts/audit-meta-local-paths.sh" --migrate-dot .vscode-shared --meta-root "$mig_meta" --real-home "$mig_home" --envctl-home-source "$mig_meta/envctl/home" >"$tmp/migrate-vscode-shared-dry.out" 2>"$tmp/migrate-vscode-shared-dry.err"
grep -q 'DRY-RUN: would move .*\.vscode-shared to .*\.local/share/vscode-shared' "$tmp/migrate-vscode-shared-dry.out"
test -d "$mig_home/.vscode-shared"
test ! -e "$mig_meta/.local/share/vscode-shared"

"$root/scripts/audit-meta-local-paths.sh" --apply --migrate-dot .vscode-shared --meta-root "$mig_meta" --real-home "$mig_home" --envctl-home-source "$mig_meta/envctl/home" >"$tmp/migrate-vscode-shared.out" 2>"$tmp/migrate-vscode-shared.err"
test "$(readlink -f "$mig_home/.vscode-shared")" = "$mig_meta/.local/share/vscode-shared"
grep -qx 'sqlite-state' "$mig_meta/.local/share/vscode-shared/sharedStorage/state.vscdb"

"$root/scripts/audit-meta-local-paths.sh" --inventory "$tmp/vscode-shared-post.tsv" --inventory-summary "$tmp/vscode-shared-post-summary.tsv" --meta-root "$mig_meta" --real-home "$mig_home" --envctl-home-source "$mig_meta/envctl/home" >"$tmp/vscode-shared-post.out" 2>"$tmp/vscode-shared-post.err"
grep -qx $'.vscode-shared\tsymlink\talready-meta\talready-meta\t'"$mig_meta"$'/.local/share/vscode-shared\tnone\tn/a' "$tmp/vscode-shared-post.tsv"

"$root/scripts/audit-meta-local-paths.sh" --migrate-dot .repomix --meta-root "$mig_meta" --real-home "$mig_home" --envctl-home-source "$mig_meta/envctl/home" >"$tmp/migrate-repomix-dry.out" 2>"$tmp/migrate-repomix-dry.err"
grep -q 'DRY-RUN: would move .*\.repomix to .*\.local/share/repomix' "$tmp/migrate-repomix-dry.out"
test -d "$mig_home/.repomix"
test ! -e "$mig_meta/.local/share/repomix"

"$root/scripts/audit-meta-local-paths.sh" --apply --migrate-dot .repomix --meta-root "$mig_meta" --real-home "$mig_home" --envctl-home-source "$mig_meta/envctl/home" >"$tmp/migrate-repomix.out" 2>"$tmp/migrate-repomix.err"
test "$(readlink -f "$mig_home/.repomix")" = "$mig_meta/.local/share/repomix"
grep -qx 'repomix-output' "$mig_meta/.local/share/repomix/outputs/latest.txt"

"$root/scripts/audit-meta-local-paths.sh" --inventory "$tmp/repomix-post.tsv" --inventory-summary "$tmp/repomix-post-summary.tsv" --meta-root "$mig_meta" --real-home "$mig_home" --envctl-home-source "$mig_meta/envctl/home" >"$tmp/repomix-post.out" 2>"$tmp/repomix-post.err"
grep -qx $'.repomix\tsymlink\talready-meta\talready-meta\t'"$mig_meta"$'/.local/share/repomix\tnone\tn/a' "$tmp/repomix-post.tsv"

"$root/scripts/audit-meta-local-paths.sh" --migrate-dot .junie --meta-root "$mig_meta" --real-home "$mig_home" --envctl-home-source "$mig_meta/envctl/home" >"$tmp/migrate-junie-dry.out" 2>"$tmp/migrate-junie-dry.err"
grep -q 'DRY-RUN: would merge source-only entries from .*\.junie into existing .*\.local/share/junie' "$tmp/migrate-junie-dry.out"
test -d "$mig_home/.junie"
test -f "$mig_meta/.local/share/junie/current/junie"
test -f "$mig_meta/.local/share/junie/updates/pending-update.json"
test ! -f "$mig_meta/.local/share/junie/settings.json"

"$root/scripts/audit-meta-local-paths.sh" --apply --migrate-dot .junie --meta-root "$mig_meta" --real-home "$mig_home" --envctl-home-source "$mig_meta/envctl/home" >"$tmp/migrate-junie.out" 2>"$tmp/migrate-junie.err"
test "$(readlink -f "$mig_home/.junie")" = "$mig_meta/.local/share/junie"
grep -qx '{ "theme": "light" }' "$mig_meta/.local/share/junie/settings.json"
grep -qx '{ "secrets": {} }' "$mig_meta/.local/share/junie/secure_credentials.json"
test "$(stat -c %a "$mig_meta/.local/share/junie/secure_credentials.json")" = "600"
grep -qx '{ "mcpServers": {} }' "$mig_meta/.local/share/junie/mcp/mcp.json"
grep -qx 'session event' "$mig_meta/.local/share/junie/sessions/events.jsonl"
grep -qx 'skill note' "$mig_meta/.local/share/junie/versions/1892.22/skills/local.md"
grep -qx 'bundled app asset' "$mig_meta/.local/share/junie/versions/1892.22/app.txt"
test -x "$mig_meta/.local/share/junie/current/junie"
grep -qx '{ "pending": true }' "$mig_meta/.local/share/junie/updates/pending-update.json"
archive_junie="$(find "$mig_meta/var/lib/envctl/real-home-dotfile-migration" -mindepth 2 -maxdepth 2 -type d -name .junie -print -quit)"
test -n "$archive_junie"
grep -qx '{ "theme": "light" }' "$archive_junie/settings.json"
test "$(stat -c %a "$archive_junie/secure_credentials.json")" = "600"

"$root/scripts/audit-meta-local-paths.sh" --inventory "$tmp/junie-post.tsv" --inventory-summary "$tmp/junie-post-summary.tsv" --meta-root "$mig_meta" --real-home "$mig_home" --envctl-home-source "$mig_meta/envctl/home" >"$tmp/junie-post.out" 2>"$tmp/junie-post.err"
grep -qx $'.junie\tsymlink\talready-meta\talready-meta\t'"$mig_meta"$'/.local/share/junie\tnone\tn/a' "$tmp/junie-post.tsv"

gphoto_bad_meta="$tmp/gphoto-bad-meta"
gphoto_bad_home="$tmp/gphoto-bad-home"
mkdir -p "$gphoto_bad_meta/.local" "$gphoto_bad_meta/envctl/home" "$gphoto_bad_home"
printf '# managed gitconfig\n' >"$gphoto_bad_meta/envctl/home/.gitconfig"
ln -s "$gphoto_bad_meta/envctl/home/.gitconfig" "$gphoto_bad_meta/.gitconfig"
ln -s "$gphoto_bad_meta/.gitconfig" "$gphoto_bad_home/.gitconfig"
ln -s "$gphoto_bad_meta/.local" "$gphoto_bad_home/.local"
printf 'not a directory\n' >"$gphoto_bad_home/.gphoto"
if "$root/scripts/audit-meta-local-paths.sh" --apply --migrate-dot .gphoto --meta-root "$gphoto_bad_meta" --real-home "$gphoto_bad_home" --envctl-home-source "$gphoto_bad_meta/envctl/home" >"$tmp/migrate-gphoto-file.out" 2>"$tmp/migrate-gphoto-file.err"; then
  echo "expected --migrate-dot .gphoto to fail closed for non-directory source" >&2
  exit 1
fi
test -f "$gphoto_bad_home/.gphoto"
test ! -e "$gphoto_bad_meta/.config/gphoto"
grep -q -- '--migrate-dot .gphoto: .* is not a directory; refusing automatic app-config directory migration' "$tmp/migrate-gphoto-file.err"

vscode_shared_bad_meta="$tmp/vscode-shared-bad-meta"
vscode_shared_bad_home="$tmp/vscode-shared-bad-home"
mkdir -p "$vscode_shared_bad_meta/.local" "$vscode_shared_bad_meta/envctl/home" "$vscode_shared_bad_home"
printf '# managed gitconfig\n' >"$vscode_shared_bad_meta/envctl/home/.gitconfig"
ln -s "$vscode_shared_bad_meta/envctl/home/.gitconfig" "$vscode_shared_bad_meta/.gitconfig"
ln -s "$vscode_shared_bad_meta/.gitconfig" "$vscode_shared_bad_home/.gitconfig"
ln -s "$vscode_shared_bad_meta/.local" "$vscode_shared_bad_home/.local"
printf 'not a directory\n' >"$vscode_shared_bad_home/.vscode-shared"
if "$root/scripts/audit-meta-local-paths.sh" --apply --migrate-dot .vscode-shared --meta-root "$vscode_shared_bad_meta" --real-home "$vscode_shared_bad_home" --envctl-home-source "$vscode_shared_bad_meta/envctl/home" >"$tmp/migrate-vscode-shared-file.out" 2>"$tmp/migrate-vscode-shared-file.err"; then
  echo "expected --migrate-dot .vscode-shared to fail closed for non-directory source" >&2
  exit 1
fi
test -f "$vscode_shared_bad_home/.vscode-shared"
test ! -e "$vscode_shared_bad_meta/.local/share/vscode-shared"
grep -q -- '--migrate-dot .vscode-shared: .* is not a directory; refusing automatic app-config directory migration' "$tmp/migrate-vscode-shared-file.err"

ai_bad_meta="$tmp/ai-bad-meta"
ai_bad_home="$tmp/ai-bad-home"
mkdir -p "$ai_bad_meta/.local" "$ai_bad_meta/envctl/home" "$ai_bad_home"
printf '# managed gitconfig
' >"$ai_bad_meta/envctl/home/.gitconfig"
ln -s "$ai_bad_meta/envctl/home/.gitconfig" "$ai_bad_meta/.gitconfig"
ln -s "$ai_bad_meta/.gitconfig" "$ai_bad_home/.gitconfig"
ln -s "$ai_bad_meta/.local" "$ai_bad_home/.local"
printf 'not a directory
' >"$ai_bad_home/.ai"
if "$root/scripts/audit-meta-local-paths.sh" --apply --migrate-dot .ai --meta-root "$ai_bad_meta" --real-home "$ai_bad_home" --envctl-home-source "$ai_bad_meta/envctl/home" >"$tmp/migrate-ai-file.out" 2>"$tmp/migrate-ai-file.err"; then
  echo "expected --migrate-dot .ai to fail closed for non-directory source" >&2
  exit 1
fi
test -f "$ai_bad_home/.ai"
test ! -e "$ai_bad_meta/.local/share/ai"
grep -q -- '--migrate-dot .ai: .* is not a directory; refusing automatic app-config directory migration' "$tmp/migrate-ai-file.err"

jetbrains_bad_meta="$tmp/jetbrains-bad-meta"
jetbrains_bad_home="$tmp/jetbrains-bad-home"
mkdir -p "$jetbrains_bad_meta/.local" "$jetbrains_bad_meta/envctl/home" "$jetbrains_bad_home"
printf '# managed gitconfig\n' >"$jetbrains_bad_meta/envctl/home/.gitconfig"
ln -s "$jetbrains_bad_meta/envctl/home/.gitconfig" "$jetbrains_bad_meta/.gitconfig"
ln -s "$jetbrains_bad_meta/.gitconfig" "$jetbrains_bad_home/.gitconfig"
ln -s "$jetbrains_bad_meta/.local" "$jetbrains_bad_home/.local"
printf 'not a directory\n' >"$jetbrains_bad_home/.jetbrains"
if "$root/scripts/audit-meta-local-paths.sh" --apply --migrate-dot .jetbrains --meta-root "$jetbrains_bad_meta" --real-home "$jetbrains_bad_home" --envctl-home-source "$jetbrains_bad_meta/envctl/home" >"$tmp/migrate-jetbrains-file.out" 2>"$tmp/migrate-jetbrains-file.err"; then
  echo "expected --migrate-dot .jetbrains to fail closed for non-directory source" >&2
  exit 1
fi
test -f "$jetbrains_bad_home/.jetbrains"
test ! -e "$jetbrains_bad_meta/.local/share/jetbrains"
grep -q -- '--migrate-dot .jetbrains: .* is not a directory; refusing automatic app-config directory migration' "$tmp/migrate-jetbrains-file.err"

meta_bad_meta="$tmp/meta-bad-meta"
meta_bad_home="$tmp/meta-bad-home"
mkdir -p "$meta_bad_meta/.local" "$meta_bad_meta/envctl/home" "$meta_bad_home"
printf '# managed gitconfig\n' >"$meta_bad_meta/envctl/home/.gitconfig"
ln -s "$meta_bad_meta/envctl/home/.gitconfig" "$meta_bad_meta/.gitconfig"
ln -s "$meta_bad_meta/.gitconfig" "$meta_bad_home/.gitconfig"
ln -s "$meta_bad_meta/.local" "$meta_bad_home/.local"
printf 'not a directory\n' >"$meta_bad_home/.meta"
if "$root/scripts/audit-meta-local-paths.sh" --apply --migrate-dot .meta --meta-root "$meta_bad_meta" --real-home "$meta_bad_home" --envctl-home-source "$meta_bad_meta/envctl/home" >"$tmp/migrate-meta-file.out" 2>"$tmp/migrate-meta-file.err"; then
  echo "expected --migrate-dot .meta to fail closed for non-directory source" >&2
  exit 1
fi
test -f "$meta_bad_home/.meta"
test ! -e "$meta_bad_meta/.local/share/meta"
grep -q -- '--migrate-dot .meta: .* is not a directory; refusing automatic app-config directory migration' "$tmp/migrate-meta-file.err"

java_bad_meta="$tmp/java-bad-meta"
java_bad_home="$tmp/java-bad-home"
mkdir -p "$java_bad_meta/.local" "$java_bad_meta/envctl/home" "$java_bad_home"
printf '# managed gitconfig\n' >"$java_bad_meta/envctl/home/.gitconfig"
ln -s "$java_bad_meta/envctl/home/.gitconfig" "$java_bad_meta/.gitconfig"
ln -s "$java_bad_meta/.gitconfig" "$java_bad_home/.gitconfig"
ln -s "$java_bad_meta/.local" "$java_bad_home/.local"
printf 'not a directory\n' >"$java_bad_home/.java"
if "$root/scripts/audit-meta-local-paths.sh" --apply --migrate-dot .java --meta-root "$java_bad_meta" --real-home "$java_bad_home" --envctl-home-source "$java_bad_meta/envctl/home" >"$tmp/migrate-java-file.out" 2>"$tmp/migrate-java-file.err"; then
  echo "expected --migrate-dot .java to fail closed for non-directory source" >&2
  exit 1
fi
test -f "$java_bad_home/.java"
test ! -e "$java_bad_meta/.local/share/java"
grep -q -- '--migrate-dot .java: .* is not a directory; refusing automatic app-config directory migration' "$tmp/migrate-java-file.err"

nv_bad_meta="$tmp/nv-bad-meta"
nv_bad_home="$tmp/nv-bad-home"
mkdir -p "$nv_bad_meta/.local" "$nv_bad_meta/envctl/home" "$nv_bad_home"
printf '# managed gitconfig\n' >"$nv_bad_meta/envctl/home/.gitconfig"
ln -s "$nv_bad_meta/envctl/home/.gitconfig" "$nv_bad_meta/.gitconfig"
ln -s "$nv_bad_meta/.gitconfig" "$nv_bad_home/.gitconfig"
ln -s "$nv_bad_meta/.local" "$nv_bad_home/.local"
printf 'not a directory\n' >"$nv_bad_home/.nv"
if "$root/scripts/audit-meta-local-paths.sh" --apply --migrate-dot .nv --meta-root "$nv_bad_meta" --real-home "$nv_bad_home" --envctl-home-source "$nv_bad_meta/envctl/home" >"$tmp/migrate-nv-file.out" 2>"$tmp/migrate-nv-file.err"; then
  echo "expected --migrate-dot .nv to fail closed for non-directory source" >&2
  exit 1
fi
test -f "$nv_bad_home/.nv"
test ! -e "$nv_bad_meta/.local/cache/nvidia"
grep -q -- '--migrate-dot .nv: .* is not a directory; refusing automatic cache directory migration' "$tmp/migrate-nv-file.err"

archon_bad_meta="$tmp/archon-bad-meta"
archon_bad_home="$tmp/archon-bad-home"
mkdir -p "$archon_bad_meta/.local" "$archon_bad_meta/envctl/home" "$archon_bad_home"
printf '# managed gitconfig\n' >"$archon_bad_meta/envctl/home/.gitconfig"
ln -s "$archon_bad_meta/envctl/home/.gitconfig" "$archon_bad_meta/.gitconfig"
ln -s "$archon_bad_meta/.gitconfig" "$archon_bad_home/.gitconfig"
ln -s "$archon_bad_meta/.local" "$archon_bad_home/.local"
printf 'not a directory\n' >"$archon_bad_home/.archon"
if "$root/scripts/audit-meta-local-paths.sh" --apply --migrate-dot .archon --meta-root "$archon_bad_meta" --real-home "$archon_bad_home" --envctl-home-source "$archon_bad_meta/envctl/home" >"$tmp/migrate-archon-file.out" 2>"$tmp/migrate-archon-file.err"; then
  echo "expected --migrate-dot .archon to fail closed for non-directory source" >&2
  exit 1
fi
test -f "$archon_bad_home/.archon"
test ! -e "$archon_bad_meta/.local/share/archon"
grep -q -- '--migrate-dot .archon: .* is not a directory; refusing automatic app-config directory migration' "$tmp/migrate-archon-file.err"

hermes_bad_meta="$tmp/hermes-bad-meta"
hermes_bad_home="$tmp/hermes-bad-home"
mkdir -p "$hermes_bad_meta/.local" "$hermes_bad_meta/envctl/home" "$hermes_bad_home"
printf '# managed gitconfig\n' >"$hermes_bad_meta/envctl/home/.gitconfig"
ln -s "$hermes_bad_meta/envctl/home/.gitconfig" "$hermes_bad_meta/.gitconfig"
ln -s "$hermes_bad_meta/.gitconfig" "$hermes_bad_home/.gitconfig"
ln -s "$hermes_bad_meta/.local" "$hermes_bad_home/.local"
printf 'not a directory\n' >"$hermes_bad_home/.hermes"
if "$root/scripts/audit-meta-local-paths.sh" --apply --migrate-dot .hermes --meta-root "$hermes_bad_meta" --real-home "$hermes_bad_home" --envctl-home-source "$hermes_bad_meta/envctl/home" >"$tmp/migrate-hermes-file.out" 2>"$tmp/migrate-hermes-file.err"; then
  echo "expected --migrate-dot .hermes to fail closed for non-directory source" >&2
  exit 1
fi
test -f "$hermes_bad_home/.hermes"
test ! -e "$hermes_bad_meta/.local/share/hermes"
grep -q -- '--migrate-dot .hermes: .* is not a directory; refusing automatic app-config directory migration' "$tmp/migrate-hermes-file.err"

n8n_mcp_bad_meta="$tmp/n8n-mcp-bad-meta"
n8n_mcp_bad_home="$tmp/n8n-mcp-bad-home"
mkdir -p "$n8n_mcp_bad_meta/.local" "$n8n_mcp_bad_meta/envctl/home" "$n8n_mcp_bad_home"
printf '# managed gitconfig\n' >"$n8n_mcp_bad_meta/envctl/home/.gitconfig"
ln -s "$n8n_mcp_bad_meta/envctl/home/.gitconfig" "$n8n_mcp_bad_meta/.gitconfig"
ln -s "$n8n_mcp_bad_meta/.gitconfig" "$n8n_mcp_bad_home/.gitconfig"
ln -s "$n8n_mcp_bad_meta/.local" "$n8n_mcp_bad_home/.local"
printf 'not a directory\n' >"$n8n_mcp_bad_home/.n8n-mcp"
if "$root/scripts/audit-meta-local-paths.sh" --apply --migrate-dot .n8n-mcp --meta-root "$n8n_mcp_bad_meta" --real-home "$n8n_mcp_bad_home" --envctl-home-source "$n8n_mcp_bad_meta/envctl/home" >"$tmp/migrate-n8n-mcp-file.out" 2>"$tmp/migrate-n8n-mcp-file.err"; then
  echo "expected --migrate-dot .n8n-mcp to fail closed for non-directory source" >&2
  exit 1
fi
test -f "$n8n_mcp_bad_home/.n8n-mcp"
test ! -e "$n8n_mcp_bad_meta/.local/share/n8n-mcp"
grep -q -- '--migrate-dot .n8n-mcp: .* is not a directory; refusing automatic app-config directory migration' "$tmp/migrate-n8n-mcp-file.err"

repomix_bad_meta="$tmp/repomix-bad-meta"
repomix_bad_home="$tmp/repomix-bad-home"
mkdir -p "$repomix_bad_meta/.local" "$repomix_bad_meta/envctl/home" "$repomix_bad_home"
printf '# managed gitconfig\n' >"$repomix_bad_meta/envctl/home/.gitconfig"
ln -s "$repomix_bad_meta/envctl/home/.gitconfig" "$repomix_bad_meta/.gitconfig"
ln -s "$repomix_bad_meta/.gitconfig" "$repomix_bad_home/.gitconfig"
ln -s "$repomix_bad_meta/.local" "$repomix_bad_home/.local"
printf 'not a directory\n' >"$repomix_bad_home/.repomix"
if "$root/scripts/audit-meta-local-paths.sh" --apply --migrate-dot .repomix --meta-root "$repomix_bad_meta" --real-home "$repomix_bad_home" --envctl-home-source "$repomix_bad_meta/envctl/home" >"$tmp/migrate-repomix-file.out" 2>"$tmp/migrate-repomix-file.err"; then
  echo "expected --migrate-dot .repomix to fail closed for non-directory source" >&2
  exit 1
fi
test -f "$repomix_bad_home/.repomix"
test ! -e "$repomix_bad_meta/.local/share/repomix"
grep -q -- '--migrate-dot .repomix: .* is not a directory; refusing automatic app-config directory migration' "$tmp/migrate-repomix-file.err"

junie_bad_meta="$tmp/junie-bad-meta"
junie_bad_home="$tmp/junie-bad-home"
mkdir -p "$junie_bad_meta/.local" "$junie_bad_meta/envctl/home" "$junie_bad_home"
printf '# managed gitconfig\n' >"$junie_bad_meta/envctl/home/.gitconfig"
ln -s "$junie_bad_meta/envctl/home/.gitconfig" "$junie_bad_meta/.gitconfig"
ln -s "$junie_bad_meta/.gitconfig" "$junie_bad_home/.gitconfig"
ln -s "$junie_bad_meta/.local" "$junie_bad_home/.local"
printf 'not a directory\n' >"$junie_bad_home/.junie"
if "$root/scripts/audit-meta-local-paths.sh" --apply --migrate-dot .junie --meta-root "$junie_bad_meta" --real-home "$junie_bad_home" --envctl-home-source "$junie_bad_meta/envctl/home" >"$tmp/migrate-junie-file.out" 2>"$tmp/migrate-junie-file.err"; then
  echo "expected --migrate-dot .junie to fail closed for non-directory source" >&2
  exit 1
fi
test -f "$junie_bad_home/.junie"
test ! -e "$junie_bad_meta/.local/share/junie"
grep -q -- '--migrate-dot .junie: .* is not a directory; refusing automatic merge app-config directory migration' "$tmp/migrate-junie-file.err"

junie_collision_meta="$tmp/junie-collision-meta"
junie_collision_home="$tmp/junie-collision-home"
mkdir -p "$junie_collision_meta/.local/share/junie/current" "$junie_collision_meta/envctl/home" "$junie_collision_home/.junie/sessions"
printf '# managed gitconfig\n' >"$junie_collision_meta/envctl/home/.gitconfig"
ln -s "$junie_collision_meta/envctl/home/.gitconfig" "$junie_collision_meta/.gitconfig"
ln -s "$junie_collision_meta/.gitconfig" "$junie_collision_home/.gitconfig"
ln -s "$junie_collision_meta/.local" "$junie_collision_home/.local"
printf 'target settings\n' >"$junie_collision_meta/.local/share/junie/settings.json"
printf 'source settings\n' >"$junie_collision_home/.junie/settings.json"
printf 'source session\n' >"$junie_collision_home/.junie/sessions/events.jsonl"
if "$root/scripts/audit-meta-local-paths.sh" --apply --migrate-dot .junie --meta-root "$junie_collision_meta" --real-home "$junie_collision_home" --envctl-home-source "$junie_collision_meta/envctl/home" >"$tmp/migrate-junie-collision.out" 2>"$tmp/migrate-junie-collision.err"; then
  echo "expected --migrate-dot .junie to fail closed for conflicting target file" >&2
  exit 1
fi
test -d "$junie_collision_home/.junie"
test ! -L "$junie_collision_home/.junie"
grep -qx 'target settings' "$junie_collision_meta/.local/share/junie/settings.json"
test ! -e "$junie_collision_meta/.local/share/junie/sessions/events.jsonl"
grep -q -- '--migrate-dot .junie: existing target .* has conflicting entries or unsafe links; refusing automatic merge' "$tmp/migrate-junie-collision.err"


mkdir -p "$mig_meta/.toolchains/cargo"
printf 'canonical cargo state\n' >"$mig_meta/.toolchains/cargo/config"
"$root/scripts/audit-meta-local-paths.sh" --apply --migrate-dot .cargo --meta-root "$mig_meta" --real-home "$mig_home" --envctl-home-source "$mig_meta/envctl/home" >"$tmp/migrate-cargo.out" 2>"$tmp/migrate-cargo.err"
test "$(readlink -f "$mig_home/.cargo")" = "$mig_meta/.toolchains/cargo"
grep -qx 'canonical cargo state' "$mig_meta/.toolchains/cargo/config"
archive_cargo="$(find "$mig_meta/var/lib/envctl/real-home-dotfile-migration" -mindepth 2 -maxdepth 2 -type d -name .cargo -print -quit)"
test -n "$archive_cargo"
grep -qx 'real-home cargo state' "$archive_cargo/config"

mkdir -p "$mig_home/.ssh" "$mig_home/.config"
if "$root/scripts/audit-meta-local-paths.sh" --apply --migrate-dot .ssh --meta-root "$mig_meta" --real-home "$mig_home" --envctl-home-source "$mig_meta/envctl/home" >"$tmp/migrate-ssh.out" 2>"$tmp/migrate-ssh.err"; then
  echo "expected --migrate-dot .ssh to fail closed" >&2
  exit 1
fi
test -d "$mig_home/.ssh"
grep -q -- '--migrate-dot .ssh is not in the supervised migration allowlist' "$tmp/migrate-ssh.err"
if "$root/scripts/audit-meta-local-paths.sh" --apply --migrate-dot .config --meta-root "$mig_meta" --real-home "$mig_home" --envctl-home-source "$mig_meta/envctl/home" >"$tmp/migrate-config.out" 2>"$tmp/migrate-config.err"; then
  echo "expected --migrate-dot .config to fail closed" >&2
  exit 1
fi
test -d "$mig_home/.config"
grep -q -- '--migrate-dot .config is not in the supervised migration allowlist' "$tmp/migrate-config.err"

portable_file_meta="$tmp/portable-file-meta"
portable_file_home="$tmp/portable-file-home"
mkdir -p "$portable_file_meta/.local" "$portable_file_meta/envctl/home" "$portable_file_home/.ideavimrc"
ln -s "$portable_file_meta/.local" "$portable_file_home/.local"
if "$root/scripts/audit-meta-local-paths.sh" --apply --migrate-dot .ideavimrc --meta-root "$portable_file_meta" --real-home "$portable_file_home" --envctl-home-source "$portable_file_meta/envctl/home" >"$tmp/migrate-ideavim-dir.out" 2>"$tmp/migrate-ideavim-dir.err"; then
  echo "expected --migrate-dot .ideavimrc directory to fail closed" >&2
  exit 1
fi
test -d "$portable_file_home/.ideavimrc"
test ! -e "$portable_file_meta/.ideavimrc"
grep -q -- '--migrate-dot .ideavimrc expects a regular file; refusing automatic move' "$tmp/migrate-ideavim-dir.err"

# History and backup dot entries are a separately opt-in safe archive class: read-only inventory
# points at the exact canonical META_ROOT archive target, default --apply does not move them, and
# --apply-history-archives preserves the real-home path as a symlink bridge.
hist_meta="$tmp/hist-meta"
hist_home="$tmp/hist-home"
mkdir -p "$hist_meta/.local" "$hist_meta/envctl/home" "$hist_home/.n8n.bak.1780701915"
printf '# managed gitconfig\n' >"$hist_meta/envctl/home/.gitconfig"
ln -s "$hist_meta/envctl/home/.gitconfig" "$hist_meta/.gitconfig"
ln -s "$hist_meta/.gitconfig" "$hist_home/.gitconfig"
ln -s "$hist_meta/.local" "$hist_home/.local"
printf 'ls -la\n' >"$hist_home/.bash_history"
printf '# old bashrc\n' >"$hist_home/.bashrc.bak.1780388793"
printf 'backup backup\n' >"$hist_home/.tool.backup"
printf 'state\n' >"$hist_home/.n8n.bak.1780701915/state"

"$root/scripts/audit-meta-local-paths.sh" \
  --inventory "$tmp/history-pre.tsv" \
  --inventory-summary "$tmp/history-pre-summary.tsv" \
  --meta-root "$hist_meta" \
  --real-home "$hist_home" \
  --envctl-home-source "$hist_meta/envctl/home" \
  >"$tmp/history-pre.out" 2>"$tmp/history-pre.err"
grep -qx $'.bash_history\tfile\treal-home-state\thistory-or-backup\t'"$hist_meta"$'/var/lib/envctl/real-home-dotfile-migration/history-or-backup/.bash_history\tarchive-and-bridge\tyes' "$tmp/history-pre.tsv"
grep -qx $'.bashrc.bak.1780388793\tfile\treal-home-state\thistory-or-backup\t'"$hist_meta"$'/var/lib/envctl/real-home-dotfile-migration/history-or-backup/.bashrc.bak.1780388793\tarchive-and-bridge\tyes' "$tmp/history-pre.tsv"
grep -qx $'.tool.backup\tfile\treal-home-state\thistory-or-backup\t'"$hist_meta"$'/var/lib/envctl/real-home-dotfile-migration/history-or-backup/.tool.backup\tarchive-and-bridge\tyes' "$tmp/history-pre.tsv"
grep -qx $'.n8n.bak.1780701915\tdirectory\treal-home-state\thistory-or-backup\t'"$hist_meta"$'/var/lib/envctl/real-home-dotfile-migration/history-or-backup/.n8n.bak.1780701915\tarchive-and-bridge\tyes' "$tmp/history-pre.tsv"
grep -qx $'history-or-backup\t4\t4\t0\t0\tarchive-and-bridge' "$tmp/history-pre-summary.tsv"

"$root/scripts/audit-meta-local-paths.sh" \
  --apply \
  --meta-root "$hist_meta" \
  --real-home "$hist_home" \
  --envctl-home-source "$hist_meta/envctl/home" \
  >"$tmp/history-default-apply.out" 2>"$tmp/history-default-apply.err"
test -f "$hist_home/.bash_history"
test ! -e "$hist_meta/var/lib/envctl/real-home-dotfile-migration/history-or-backup/.bash_history"

"$root/scripts/audit-meta-local-paths.sh" \
  --apply \
  --apply-history-archives \
  --meta-root "$hist_meta" \
  --real-home "$hist_home" \
  --envctl-home-source "$hist_meta/envctl/home" \
  >"$tmp/history-apply.out" 2>"$tmp/history-apply.err"
test "$(readlink -f "$hist_home/.bash_history")" = "$hist_meta/var/lib/envctl/real-home-dotfile-migration/history-or-backup/.bash_history"
grep -qx 'ls -la' "$hist_meta/var/lib/envctl/real-home-dotfile-migration/history-or-backup/.bash_history"
test "$(readlink -f "$hist_home/.bashrc.bak.1780388793")" = "$hist_meta/var/lib/envctl/real-home-dotfile-migration/history-or-backup/.bashrc.bak.1780388793"
grep -qx '# old bashrc' "$hist_meta/var/lib/envctl/real-home-dotfile-migration/history-or-backup/.bashrc.bak.1780388793"
test "$(readlink -f "$hist_home/.tool.backup")" = "$hist_meta/var/lib/envctl/real-home-dotfile-migration/history-or-backup/.tool.backup"
grep -qx 'backup backup' "$hist_meta/var/lib/envctl/real-home-dotfile-migration/history-or-backup/.tool.backup"
test "$(readlink -f "$hist_home/.n8n.bak.1780701915")" = "$hist_meta/var/lib/envctl/real-home-dotfile-migration/history-or-backup/.n8n.bak.1780701915"
grep -qx 'state' "$hist_meta/var/lib/envctl/real-home-dotfile-migration/history-or-backup/.n8n.bak.1780701915/state"

"$root/scripts/audit-meta-local-paths.sh" \
  --inventory "$tmp/history-post.tsv" \
  --inventory-summary "$tmp/history-post-summary.tsv" \
  --meta-root "$hist_meta" \
  --real-home "$hist_home" \
  --envctl-home-source "$hist_meta/envctl/home" \
  >"$tmp/history-post.out" 2>"$tmp/history-post.err"
grep -qx $'.bash_history\tsymlink\talready-meta\talready-meta\t'"$hist_meta"$'/var/lib/envctl/real-home-dotfile-migration/history-or-backup/.bash_history\tnone\tn/a' "$tmp/history-post.tsv"
grep -qx $'.bashrc.bak.1780388793\tsymlink\talready-meta\talready-meta\t'"$hist_meta"$'/var/lib/envctl/real-home-dotfile-migration/history-or-backup/.bashrc.bak.1780388793\tnone\tn/a' "$tmp/history-post.tsv"
grep -qx $'.tool.backup\tsymlink\talready-meta\talready-meta\t'"$hist_meta"$'/var/lib/envctl/real-home-dotfile-migration/history-or-backup/.tool.backup\tnone\tn/a' "$tmp/history-post.tsv"
grep -qx $'.n8n.bak.1780701915\tsymlink\talready-meta\talready-meta\t'"$hist_meta"$'/var/lib/envctl/real-home-dotfile-migration/history-or-backup/.n8n.bak.1780701915\tnone\tn/a' "$tmp/history-post.tsv"
grep -qx $'already-meta\t4\t0\t0\t4\tnone' "$tmp/history-post-summary.tsv"

hist_collision_meta="$tmp/hist-collision-meta"
hist_collision_home="$tmp/hist-collision-home"
mkdir -p "$hist_collision_meta/.local" "$hist_collision_meta/envctl/home" "$hist_collision_meta/var/lib/envctl/real-home-dotfile-migration/history-or-backup" "$hist_collision_home"
printf '# managed gitconfig\n' >"$hist_collision_meta/envctl/home/.gitconfig"
ln -s "$hist_collision_meta/envctl/home/.gitconfig" "$hist_collision_meta/.gitconfig"
ln -s "$hist_collision_meta/.gitconfig" "$hist_collision_home/.gitconfig"
ln -s "$hist_collision_meta/.local" "$hist_collision_home/.local"
printf 'real\n' >"$hist_collision_home/.zsh_history"
printf 'canonical\n' >"$hist_collision_meta/var/lib/envctl/real-home-dotfile-migration/history-or-backup/.zsh_history"
"$root/scripts/audit-meta-local-paths.sh" \
  --apply \
  --apply-history-archives \
  --meta-root "$hist_collision_meta" \
  --real-home "$hist_collision_home" \
  --envctl-home-source "$hist_collision_meta/envctl/home" \
  >"$tmp/history-collision.out" 2>"$tmp/history-collision.err"
test ! -L "$hist_collision_home/.zsh_history"
grep -qx 'real' "$hist_collision_home/.zsh_history"
grep -qx 'canonical' "$hist_collision_meta/var/lib/envctl/real-home-dotfile-migration/history-or-backup/.zsh_history"
grep -q 'owner-supervised merge required' "$tmp/history-collision.err"


# History/backup dot entries have a single owner-approved mutation path: --apply-history-archives
# archives into stable meta-owned state and preserves the real-home path as a symlink bridge.
backup_meta="$tmp/backup-meta"
backup_home="$tmp/backup-home"
mkdir -p "$backup_meta/.local" "$backup_meta/envctl/home" "$backup_home/.n8n.bak.1780701915"
printf '# managed gitconfig\n' >"$backup_meta/envctl/home/.gitconfig"
ln -s "$backup_meta/envctl/home/.gitconfig" "$backup_meta/.gitconfig"
ln -s "$backup_meta/.gitconfig" "$backup_home/.gitconfig"
ln -s "$backup_meta/.local" "$backup_home/.local"
printf 'active history\n' >"$backup_home/.bash_history"
printf '# backup bashrc\n' >"$backup_home/.bashrc.bak.123"
printf '# backup zshrc\n' >"$backup_home/.zshrc.bak.2026-06-03_05-44-02"
printf 'state\n' >"$backup_home/.n8n.bak.1780701915/state"

"$root/scripts/audit-meta-local-paths.sh" \
  --inventory "$tmp/backup-pre.tsv" \
  --inventory-summary "$tmp/backup-pre-summary.tsv" \
  --meta-root "$backup_meta" \
  --real-home "$backup_home" \
  --envctl-home-source "$backup_meta/envctl/home" \
  >"$tmp/backup-pre.out" 2>"$tmp/backup-pre.err"
grep -qx $'.bash_history\tfile\treal-home-state\thistory-or-backup\t'"$backup_meta"$'/var/lib/envctl/real-home-dotfile-migration/history-or-backup/.bash_history\tarchive-and-bridge\tyes' "$tmp/backup-pre.tsv"
grep -qx $'.bashrc.bak.123\tfile\treal-home-state\thistory-or-backup\t'"$backup_meta"$'/var/lib/envctl/real-home-dotfile-migration/history-or-backup/.bashrc.bak.123\tarchive-and-bridge\tyes' "$tmp/backup-pre.tsv"
grep -qx $'.zshrc.bak.2026-06-03_05-44-02\tfile\treal-home-state\thistory-or-backup\t'"$backup_meta"$'/var/lib/envctl/real-home-dotfile-migration/history-or-backup/.zshrc.bak.2026-06-03_05-44-02\tarchive-and-bridge\tyes' "$tmp/backup-pre.tsv"
grep -qx $'.n8n.bak.1780701915\tdirectory\treal-home-state\thistory-or-backup\t'"$backup_meta"$'/var/lib/envctl/real-home-dotfile-migration/history-or-backup/.n8n.bak.1780701915\tarchive-and-bridge\tyes' "$tmp/backup-pre.tsv"
grep -qx $'history-or-backup\t4\t4\t0\t0\tarchive-and-bridge' "$tmp/backup-pre-summary.tsv"

"$root/scripts/audit-meta-local-paths.sh" \
  --apply \
  --apply-history-archives \
  --meta-root "$backup_meta" \
  --real-home "$backup_home" \
  --envctl-home-source "$backup_meta/envctl/home" \
  >"$tmp/backup-apply.out" 2>"$tmp/backup-apply.err"
test "$(readlink -f "$backup_home/.bash_history")" = "$backup_meta/var/lib/envctl/real-home-dotfile-migration/history-or-backup/.bash_history"
test "$(readlink -f "$backup_home/.bashrc.bak.123")" = "$backup_meta/var/lib/envctl/real-home-dotfile-migration/history-or-backup/.bashrc.bak.123"
test "$(readlink -f "$backup_home/.zshrc.bak.2026-06-03_05-44-02")" = "$backup_meta/var/lib/envctl/real-home-dotfile-migration/history-or-backup/.zshrc.bak.2026-06-03_05-44-02"
test "$(readlink -f "$backup_home/.n8n.bak.1780701915")" = "$backup_meta/var/lib/envctl/real-home-dotfile-migration/history-or-backup/.n8n.bak.1780701915"
grep -qx 'active history' "$backup_meta/var/lib/envctl/real-home-dotfile-migration/history-or-backup/.bash_history"
grep -qx '# backup bashrc' "$backup_meta/var/lib/envctl/real-home-dotfile-migration/history-or-backup/.bashrc.bak.123"
grep -qx '# backup zshrc' "$backup_meta/var/lib/envctl/real-home-dotfile-migration/history-or-backup/.zshrc.bak.2026-06-03_05-44-02"
grep -qx 'state' "$backup_meta/var/lib/envctl/real-home-dotfile-migration/history-or-backup/.n8n.bak.1780701915/state"
grep -Eq 'APPLY: archived .*\.bashrc\.bak\.123 to .*history-or-backup/\.bashrc\.bak\.123' "$tmp/backup-apply.out"

# Recursive deep-link inventory walks the actual META_ROOT .local/.toolchains stores without
# failing by default on embedded system/container links or broken toolchain-internal links, but it
# can be made fail-closed for symlinks that resolve back into the real home outside META_ROOT.
deep_meta="$tmp/deep-meta"
deep_home="$tmp/deep-home"
mkdir -p \
  "$deep_meta/.local/share/app" \
  "$deep_meta/.toolchains/cargo/bin" \
  "$deep_meta/usr/bin" \
  "$deep_meta/envctl/home" \
  "$deep_home/.cache/app"
printf '# managed gitconfig\n' >"$deep_meta/envctl/home/.gitconfig"
ln -s "$deep_meta/envctl/home/.gitconfig" "$deep_meta/.gitconfig"
ln -s "$deep_meta/.gitconfig" "$deep_home/.gitconfig"
ln -s "$deep_meta/.local" "$deep_home/.local"
printf '#!/usr/bin/env bash\nexit 0\n' >"$deep_meta/usr/bin/rustc"
chmod +x "$deep_meta/usr/bin/rustc"
ln -s "$deep_meta/usr/bin/rustc" "$deep_meta/.toolchains/cargo/bin/rustc"
ln -s /usr/bin/env "$deep_meta/.toolchains/cargo/bin/env"
ln -s "$deep_home/.cache/app" "$deep_meta/.local/share/app/cache"
ln -s "$deep_meta/.toolchains/cargo/bin/absent" "$deep_meta/.toolchains/cargo/bin/missing"

"$root/scripts/audit-meta-local-paths.sh" \
  --deep-link-inventory "$tmp/deep-links.tsv" \
  --deep-link-summary "$tmp/deep-links-summary.tsv" \
  --meta-root "$deep_meta" \
  --real-home "$deep_home" \
  --envctl-home-source "$deep_meta/envctl/home" \
  >"$tmp/deep.out" 2>"$tmp/deep.err"
grep -q 'meta-local audit: PASS' "$tmp/deep.out"
grep -q 'resolves into real home outside META_ROOT' "$tmp/deep.err"
head -n 1 "$tmp/deep-links.tsv" | grep -qx $'scan_root\tsymlink\tlink_text\tresolved_target\ttarget_class\taction'
awk -F '\t' 'NF != 6 { print "bad deep-link row: " $0 >"/dev/stderr"; bad=1 } END { exit bad }' "$tmp/deep-links.tsv"
grep -qx "$deep_meta/.toolchains"$'\t'"$deep_meta/.toolchains/cargo/bin/rustc"$'\t'"$deep_meta/usr/bin/rustc"$'\t'"$deep_meta/usr/bin/rustc"$'\tinside-meta\tnone' "$tmp/deep-links.tsv"
awk -F '\t' -v p="$deep_meta/.toolchains/cargo/bin/env" \
  '$2 == p && $5 == "external-system" && $6 == "embedded-toolchain-or-system-reference" { found=1 } END { exit !found }' \
  "$tmp/deep-links.tsv"
grep -qx "$deep_meta/.local"$'\t'"$deep_meta/.local/share/app/cache"$'\t'"$deep_home/.cache/app"$'\t'"$deep_home/.cache/app"$'\treal-home-leak\tmigrate-or-relink-to-meta' "$tmp/deep-links.tsv"
awk -F '\t' -v p="$deep_meta/.toolchains/cargo/bin/missing" \
  '$2 == p && $5 == "missing-target" && $6 == "owner-supervised-repair-or-ignore-embedded-toolchain-link" { found=1 } END { exit !found }' \
  "$tmp/deep-links.tsv"
head -n 1 "$tmp/deep-links-summary.tsv" | grep -qx $'target_class\ttotal\tactions'
grep -qx $'inside-meta\t1\tnone' "$tmp/deep-links-summary.tsv"
grep -qx $'external-system\t1\tembedded-toolchain-or-system-reference' "$tmp/deep-links-summary.tsv"
grep -qx $'real-home-leak\t1\tmigrate-or-relink-to-meta' "$tmp/deep-links-summary.tsv"
grep -qx $'missing-target\t1\towner-supervised-repair-or-ignore-embedded-toolchain-link' "$tmp/deep-links-summary.tsv"

if "$root/scripts/audit-meta-local-paths.sh" \
  --fail-real-home-deep-links \
  --meta-root "$deep_meta" \
  --real-home "$deep_home" \
  --envctl-home-source "$deep_meta/envctl/home" \
  >"$tmp/deep-fail.out" 2>"$tmp/deep-fail.err"; then
  echo "expected recursive real-home symlink leak to fail when requested" >&2
  exit 1
fi
grep -q 'resolves into real home outside META_ROOT' "$tmp/deep-fail.err"


# If no meta-owned replacement exists for an escaping .local/bin symlink, --apply must fail closed
# and leave the unsafe link untouched for owner-supervised remediation.
rm -f "$meta/usr/bin/hf"
ln -sfn "$outside/hf" "$meta/.local/bin/hf"
if "$root/scripts/audit-meta-local-paths.sh" --apply --meta-root "$meta" --real-home "$home" --envctl-home-source "$meta/envctl/home" >"$tmp/no-candidate.out" 2>"$tmp/no-candidate.err"; then
  echo "expected --apply to fail when no meta replacement exists" >&2
  exit 1
fi
test "$(readlink -f "$meta/.local/bin/hf")" = "$outside/hf"
grep -q 'no safe meta replacement was applied' "$tmp/no-candidate.err"

echo "test-meta-local-path-audit: PASS"
