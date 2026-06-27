#!/usr/bin/env bash
set -euo pipefail
root="$(git -C "$(dirname "${BASH_SOURCE[0]}")/../.." rev-parse --show-toplevel)"
tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT

fake_bin="$tmp/bin"
mkdir -p "$fake_bin"
cat >"$fake_bin/lsof" <<'EOF'
#!/usr/bin/env bash
source_arg="${1:-}"
if [ "${1:-}" = "+D" ]; then
  source_arg="${2:-}"
fi
if [ -n "${ENVCTL_TEST_LSOF_OPEN_SOURCE:-}" ] && [ "$source_arg" = "$ENVCTL_TEST_LSOF_OPEN_SOURCE" ]; then
  printf 'COMMAND PID USER FD TYPE DEVICE SIZE/OFF NODE NAME\n'
  printf 'chrome 123 drdave 118u REG 0,0 0 1 %s/nssdb/key4.db\n' "$ENVCTL_TEST_LSOF_OPEN_SOURCE"
  exit 0
fi
exit 1
EOF
chmod +x "$fake_bin/lsof"
export PATH="$fake_bin:$PATH"

meta="$tmp/meta"
home="$tmp/home"
outside="$tmp/outside"
mkdir -p "$meta/.local/bin" "$meta/usr/bin" "$meta/envctl/home" "$home" "$outside" "$home/.ssh" "$home/.aws" "$home/.cache" "$home/.cargo"
printf '# managed gitconfig\n' >"$meta/envctl/home/.gitconfig"
printf '# managed shell config\n' >"$meta/envctl/home/.zshrc"
printf '# unmanaged shell config\n' >"$home/.zshrc"
printf 'fixture private key\n' >"$home/.ssh/id_ed25519"
printf 'fixture token\n' >"$home/.aws/session-token"
chmod 600 "$home/.ssh/id_ed25519" "$home/.aws/session-token"
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
"$root/scripts/audit-meta-local-paths.sh" --inventory "$tmp/inventory.tsv" --inventory-summary "$tmp/inventory-summary.tsv" --sensitive-state-report "$tmp/sensitive-state.tsv" --owner-supervised-sensitive-review-plan "$tmp/sensitive-review-plan.tsv" --meta-root "$meta" --real-home "$home" --envctl-home-source "$meta/envctl/home" >"$tmp/post.out" 2>"$tmp/post.err"

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

head -n 1 "$tmp/sensitive-state.tsv" | grep -qx $'dot_entry\treal_path\ttype\tdigest\tentries\tdirect_files\tdirect_dirs\tsymlinks\tsensitive_hints\taction\tapply_safe\trecommendation'
awk -F '\t' 'NF != 12 { print "bad sensitive-state row: " $0 >"/dev/stderr"; bad=1 } END { exit bad }' "$tmp/sensitive-state.tsv"
test "$(wc -l <"$tmp/sensitive-state.tsv" | tr -d '[:space:]')" = 3
awk -F '\t' -v home="$home" '
  $1 == ".aws" {
    if ($2 != home "/.aws") bad=1
    if ($3 != "directory") bad=1
    if ($4 !~ /^[0-9a-f]{64}$/) bad=1
    if ($5 != "1") bad=1
    if ($6 != "1") bad=1
    if ($7 != "0") bad=1
    if ($8 != "0") bad=1
    if ($9 != "1") bad=1
    if ($10 != "owner-supervised-vault-or-bridge") bad=1
    if ($11 != "no") bad=1
    if ($12 != "owner-supervised-vault-or-bridge-before-migration") bad=1
    found_aws=1
  }
  $1 == ".ssh" {
    if ($2 != home "/.ssh") bad=1
    if ($3 != "directory") bad=1
    if ($4 !~ /^[0-9a-f]{64}$/) bad=1
    if ($5 != "1") bad=1
    if ($6 != "1") bad=1
    if ($7 != "0") bad=1
    if ($8 != "0") bad=1
    if ($9 != "1") bad=1
    if ($10 != "owner-supervised-vault-or-bridge") bad=1
    if ($11 != "no") bad=1
    if ($12 != "owner-supervised-vault-or-bridge-before-migration") bad=1
    found_ssh=1
  }
  END { exit !(found_aws && found_ssh && !bad) }
' "$tmp/sensitive-state.tsv"

head -n 1 "$tmp/sensitive-review-plan.tsv" | grep -qx $'dot_entry	real_path	type	target_class	digest	entries	direct_files	direct_dirs	symlinks	sensitive_hints	supervision	next_action	sensitive_scope	review_hint	apply_command'
awk -F '	' 'NF != 15 { print "bad sensitive review plan row: " $0 >"/dev/stderr"; bad=1 } END { exit bad }' "$tmp/sensitive-review-plan.tsv"
test "$(wc -l <"$tmp/sensitive-review-plan.tsv" | tr -d '[:space:]')" = 3
awk -F '	' -v home="$home" '
  $1 == ".aws" {
    if ($2 != home "/.aws") bad=1
    if ($3 != "directory") bad=1
    if ($4 != "sensitive") bad=1
    if ($5 !~ /^[0-9a-f]{64}$/) bad=1
    if ($6 != "1") bad=1
    if ($7 != "1") bad=1
    if ($8 != "0") bad=1
    if ($9 != "0") bad=1
    if ($10 != "1") bad=1
    if ($11 != "owner-reviewed") bad=1
    if ($12 != "owner-supervised-vault-or-bridge") bad=1
    if ($13 != "credential-or-private-state") bad=1
    if ($14 != "inspect-sensitive-state-before-owner-approved-vault-or-bridge") bad=1
    if ($15 != "") bad=1
    found_aws=1
  }
  $1 == ".ssh" {
    if ($2 != home "/.ssh") bad=1
    if ($3 != "directory") bad=1
    if ($4 != "sensitive") bad=1
    if ($5 !~ /^[0-9a-f]{64}$/) bad=1
    if ($6 != "1") bad=1
    if ($7 != "1") bad=1
    if ($8 != "0") bad=1
    if ($9 != "0") bad=1
    if ($10 != "1") bad=1
    if ($11 != "owner-reviewed") bad=1
    if ($12 != "owner-supervised-vault-or-bridge") bad=1
    if ($13 != "credential-or-private-state") bad=1
    if ($14 != "inspect-sensitive-state-before-owner-approved-vault-or-bridge") bad=1
    if ($15 != "") bad=1
    found_ssh=1
  }
  $1 == ".cache" { bad=1 }
  $1 == ".config" { bad=1 }
  $1 == ".gitconfig" { bad=1 }
  END { exit !(found_aws && found_ssh && !bad) }
' "$tmp/sensitive-review-plan.tsv"
"$root/scripts/audit-meta-local-paths.sh" --owner-supervised-sensitive-review-plan "$tmp/sensitive-review-plan-only.tsv" --meta-root "$meta" --real-home "$home" --envctl-home-source "$meta/envctl/home" >"$tmp/sensitive-review-plan-only.out" 2>"$tmp/sensitive-review-plan-only.err"
cmp "$tmp/sensitive-review-plan.tsv" "$tmp/sensitive-review-plan-only.tsv"

supervised_meta="$tmp/supervised-meta"
supervised_home="$tmp/supervised-home"
mkdir -p "$supervised_meta/.local/cache/meta-cache" "$supervised_meta/envctl/home/.config/managed-app/type-conflict" "$supervised_home/.cache/tool" "$supervised_home/.config/app" "$supervised_home/.config/managed-app" "$supervised_home/.ssh"
printf '# managed gitconfig\n' >"$supervised_meta/envctl/home/.gitconfig"
ln -s "$supervised_meta/envctl/home/.gitconfig" "$supervised_meta/.gitconfig"
ln -s "$supervised_meta/.local" "$supervised_home/.local"
ln -s "$supervised_meta/.gitconfig" "$supervised_home/.gitconfig"
printf 'cache-index\n' >"$supervised_home/.cache/tool/index"
printf 'meta-cache-index\n' >"$supervised_meta/.local/cache/meta-cache/index"
ln -s "$supervised_meta/.local/cache/meta-cache" "$supervised_home/.cache/meta-cache"
printf 'settings\n' >"$supervised_home/.config/app/settings.json"
printf 'nested-secret\n' >"$supervised_home/.config/app/token"
printf 'managed-canonical-settings\n' >"$supervised_meta/envctl/home/.config/managed-app/settings.json"
printf 'managed-only\n' >"$supervised_meta/envctl/home/.config/managed-app/managed-only.json"
printf 'managed-settings\n' >"$supervised_home/.config/managed-app/settings.json"
printf 'real-only\n' >"$supervised_home/.config/managed-app/real-only.json"
printf 'real-type-conflict\n' >"$supervised_home/.config/managed-app/type-conflict"
ln -s "$outside/hf" "$supervised_home/.config/external-app"
printf 'key\n' >"$supervised_home/.ssh/id_ed25519"
"$root/scripts/audit-meta-local-paths.sh" --owner-supervised-state-report "$tmp/owner-supervised-state.tsv" --owner-supervised-child-report "$tmp/owner-supervised-child.tsv" --owner-supervised-child-candidates-report "$tmp/owner-supervised-child-candidates.tsv" --owner-supervised-child-candidate-actions "$tmp/owner-supervised-child-candidate-actions.tsv" --owner-supervised-cache-child-component-plan "$tmp/owner-supervised-cache-child-component-plan.tsv" --owner-supervised-managed-config-child-review-plan "$tmp/owner-supervised-managed-config-child-review-plan.tsv" --owner-supervised-managed-config-child-conflict-plan "$tmp/owner-supervised-managed-config-child-conflict-plan.tsv" --owner-supervised-managed-config-child-conflict-summary "$tmp/owner-supervised-managed-config-child-conflict-summary.tsv" --owner-supervised-config-child-classification-plan "$tmp/owner-supervised-config-child-classification-plan.tsv" --owner-supervised-child-candidate-action-summary "$tmp/owner-supervised-child-candidate-action-summary.tsv" --owner-supervised-child-candidates-summary "$tmp/owner-supervised-child-candidates-summary.tsv" --meta-root "$supervised_meta" --real-home "$supervised_home" --envctl-home-source "$supervised_meta/envctl/home" >"$tmp/owner-supervised-state.out" 2>"$tmp/owner-supervised-state.err"
head -n 1 "$tmp/owner-supervised-state.tsv" | grep -qx $'dot_entry\treal_path\ttype\ttarget_class\tshallow_digest\tdirect_entries\tdirect_files\tdirect_dirs\tdirect_symlinks\taction\tapply_safe\trecommendation'
awk -F '\t' 'NF != 12 { print "bad owner-supervised-state row: " $0 >"/dev/stderr"; bad=1 } END { exit bad }' "$tmp/owner-supervised-state.tsv"
test "$(wc -l <"$tmp/owner-supervised-state.tsv" | tr -d '[:space:]')" = 3
awk -F '\t' -v home="$supervised_home" '
  $1 == ".cache" {
    if ($2 != home "/.cache") bad=1
    if ($3 != "directory") bad=1
    if ($4 != "cache") bad=1
    if ($5 !~ /^[0-9a-f]{64}$/) bad=1
    if ($6 != "2") bad=1
    if ($7 != "0") bad=1
    if ($8 != "1") bad=1
    if ($9 != "1") bad=1
    if ($10 != "component-managed-cache-migration") bad=1
    if ($11 != "no") bad=1
    if ($12 != "use-component-managed-cache-migration") bad=1
    found_cache=1
  }
  $1 == ".config" {
    if ($2 != home "/.config") bad=1
    if ($3 != "directory") bad=1
    if ($4 != "managed-dotfile") bad=1
    if ($5 !~ /^[0-9a-f]{64}$/) bad=1
    if ($6 != "3") bad=1
    if ($7 != "0") bad=1
    if ($8 != "2") bad=1
    if ($9 != "1") bad=1
    if ($10 != "owner-supervised-bridge") bad=1
    if ($11 != "no") bad=1
    if ($12 != "owner-review-before-bridge") bad=1
    found_config=1
  }
  END { exit !(found_cache && found_config && !bad) }
' "$tmp/owner-supervised-state.tsv"
if awk -F '\t' '$1 == ".ssh" { found=1 } END { exit !found }' "$tmp/owner-supervised-state.tsv"; then
  echo "unexpected owner-supervised-state report row for sensitive .ssh" >&2
  exit 1
fi

"$root/scripts/audit-meta-local-paths.sh" --owner-supervised-child-plan "$tmp/owner-supervised-child-plan.tsv" --meta-root "$supervised_meta" --real-home "$supervised_home" --envctl-home-source "$supervised_meta/envctl/home" >"$tmp/owner-supervised-child-plan.out" 2>"$tmp/owner-supervised-child-plan.err"
head -n 1 "$tmp/owner-supervised-child-plan.tsv" | grep -qx $'dot_entry\tchild_name\tchild_path\ttype\ttarget_class\tsupervision\tnext_action\tmigration_scope\trecommendation'
awk -F '\t' 'NF != 9 { print "bad owner-supervised-child-plan row: " $0 >"/dev/stderr"; bad=1 } END { exit bad }' "$tmp/owner-supervised-child-plan.tsv"
test "$(wc -l <"$tmp/owner-supervised-child-plan.tsv" | tr -d '[:space:]')" = 6
awk -F '\t' -v home="$supervised_home" '
  $1 == ".cache" && $2 == "tool" {
    if ($3 != home "/.cache/tool") bad=1
    if ($4 != "directory") bad=1
    if ($5 != "cache") bad=1
    if ($6 != "component-managed") bad=1
    if ($7 != "component-manifest-or-tool-cache-route") bad=1
    if ($8 != "cache-child") bad=1
    if ($9 != "classify-cache-child-component-before-migration") bad=1
    found_cache=1
  }
  $1 == ".config" && $2 == "app" {
    if ($3 != home "/.config/app") bad=1
    if ($4 != "directory") bad=1
    if ($5 != "managed-dotfile") bad=1
    if ($6 != "owner-reviewed") bad=1
    if ($7 != "owner-review-config-child-before-bridge-or-migration") bad=1
    if ($8 != "config-child") bad=1
    if ($9 != "classify-config-child-before-bridge-or-migration") bad=1
    found_config=1
  }
  $2 == "settings.json" || $2 == "token" { nested=1 }
  $1 == ".ssh" { sensitive=1 }
  END { exit !(found_cache && found_config && !bad && !nested && !sensitive) }
' "$tmp/owner-supervised-child-plan.tsv"

head -n 1 "$tmp/owner-supervised-child.tsv" | grep -qx $'dot_entry\tchild_name\tchild_path\ttype\ttarget_class\tshallow_digest\tdirect_entries\tdirect_files\tdirect_dirs\tdirect_symlinks\trecommendation'
awk -F '\t' 'NF != 11 { print "bad owner-supervised-child row: " $0 >"/dev/stderr"; bad=1 } END { exit bad }' "$tmp/owner-supervised-child.tsv"
test "$(wc -l <"$tmp/owner-supervised-child.tsv" | tr -d '[:space:]')" = 6
awk -F '\t' -v home="$supervised_home" '
  $1 == ".cache" && $2 == "tool" {
    if ($3 != home "/.cache/tool") bad=1
    if ($4 != "directory") bad=1
    if ($5 != "cache") bad=1
    if ($6 !~ /^[0-9a-f]{64}$/) bad=1
    if ($7 != "1") bad=1
    if ($8 != "1") bad=1
    if ($9 != "0") bad=1
    if ($10 != "0") bad=1
    if ($11 != "classify-cache-child-component-before-migration") bad=1
    found_cache=1
  }
  $1 == ".config" && $2 == "app" {
    if ($3 != home "/.config/app") bad=1
    if ($4 != "directory") bad=1
    if ($5 != "managed-dotfile") bad=1
    if ($6 !~ /^[0-9a-f]{64}$/) bad=1
    if ($7 != "2") bad=1
    if ($8 != "2") bad=1
    if ($9 != "0") bad=1
    if ($10 != "0") bad=1
    if ($11 != "classify-config-child-before-bridge-or-migration") bad=1
    found_config=1
  }
  $2 == "settings.json" || $2 == "token" { nested=1 }
  $1 == ".ssh" { sensitive=1 }
  END { exit !(found_cache && found_config && !bad && !nested && !sensitive) }
' "$tmp/owner-supervised-child.tsv"

head -n 1 "$tmp/owner-supervised-child-candidates.tsv" | grep -qx $'dot_entry\tchild_name\tchild_path\ttype\tchild_state\tchild_target_class\tcanonical_target\tshallow_digest\tdirect_entries\tdirect_files\tdirect_dirs\tdirect_symlinks\tcandidate_action\tapply_safe\trecommendation'
awk -F '\t' 'NF != 15 { print "bad owner-supervised-child-candidates row: " $0 >"/dev/stderr"; bad=1 } END { exit bad }' "$tmp/owner-supervised-child-candidates.tsv"
test "$(wc -l <"$tmp/owner-supervised-child-candidates.tsv" | tr -d '[:space:]')" = 6
awk -F '\t' -v home="$supervised_home" -v meta="$supervised_meta" -v outside="$outside" '
  $1 == ".cache" && $2 == "tool" {
    if ($3 != home "/.cache/tool") bad=1
    if ($4 != "directory") bad=1
    if ($5 != "real-home-state") bad=1
    if ($6 != "cache-child") bad=1
    if ($7 != meta "/.local/cache/tool") bad=1
    if ($8 !~ /^[0-9a-f]{64}$/) bad=1
    if ($9 != "1") bad=1
    if ($10 != "1") bad=1
    if ($11 != "0") bad=1
    if ($12 != "0") bad=1
    if ($13 != "component-managed-cache-child-migration") bad=1
    if ($14 != "no") bad=1
    if ($15 != "add-component-cache-rule-or-owner-approved-child-migration") bad=1
    found_cache=1
  }
  $1 == ".cache" && $2 == "meta-cache" {
    if ($3 != home "/.cache/meta-cache") bad=1
    if ($4 != "symlink") bad=1
    if ($5 != "already-meta") bad=1
    if ($6 != "already-meta") bad=1
    if ($7 != meta "/.local/cache/meta-cache") bad=1
    if ($13 != "none") bad=1
    if ($14 != "n/a") bad=1
    if ($15 != "none") bad=1
    found_meta_cache=1
  }
  $1 == ".config" && $2 == "managed-app" {
    if ($3 != home "/.config/managed-app") bad=1
    if ($4 != "directory") bad=1
    if ($5 != "real-home-state") bad=1
    if ($6 != "managed-config-child") bad=1
    if ($7 != meta "/envctl/home/.config/managed-app") bad=1
    if ($9 != "3") bad=1
    if ($10 != "3") bad=1
    if ($11 != "0") bad=1
    if ($12 != "0") bad=1
    if ($13 != "owner-supervised-config-child-bridge") bad=1
    if ($14 != "no") bad=1
    if ($15 != "owner-review-managed-config-child-before-bridge") bad=1
    found_managed=1
  }
  $1 == ".config" && $2 == "app" {
    if ($3 != home "/.config/app") bad=1
    if ($4 != "directory") bad=1
    if ($5 != "real-home-state") bad=1
    if ($6 != "config-child") bad=1
    if ($7 != meta "/.config/app") bad=1
    if ($9 != "2") bad=1
    if ($10 != "2") bad=1
    if ($11 != "0") bad=1
    if ($12 != "0") bad=1
    if ($13 != "classify-config-child-before-bridge-or-migration") bad=1
    if ($14 != "no") bad=1
    if ($15 != "classify-config-child-before-bridge-or-migration") bad=1
    found_config=1
  }
  $1 == ".config" && $2 == "external-app" {
    if ($3 != home "/.config/external-app") bad=1
    if ($4 != "symlink") bad=1
    if ($5 != "external-symlink") bad=1
    if ($6 != "external-symlink") bad=1
    if ($7 != outside "/hf") bad=1
    if ($13 != "owner-supervised-relink") bad=1
    if ($14 != "no") bad=1
    if ($15 != "owner-review-before-relink") bad=1
    found_external=1
  }
  $2 == "settings.json" || $2 == "token" { nested=1 }
  $1 == ".ssh" { sensitive=1 }
  END { exit !(found_cache && found_meta_cache && found_managed && found_config && found_external && !bad && !nested && !sensitive) }
' "$tmp/owner-supervised-child-candidates.tsv"

head -n 1 "$tmp/owner-supervised-child-candidate-actions.tsv" | grep -qx $'dot_entry\tchild_name\tchild_target_class\tcandidate_action\tapply_safe\tsupervision\tnext_action\tcanonical_target\tenvctl_home_source\tapply_command'
awk -F '\t' 'NF != 10 { print "bad owner-supervised-child-candidate-actions row: " $0 >"/dev/stderr"; bad=1 } END { exit bad }' "$tmp/owner-supervised-child-candidate-actions.tsv"
test "$(wc -l <"$tmp/owner-supervised-child-candidate-actions.tsv" | tr -d '[:space:]')" = 6
awk -F '\t' -v meta="$supervised_meta" '
  $1 == ".cache" && $2 == "tool" {
    if ($3 != "cache-child") bad=1
    if ($4 != "component-managed-cache-child-migration") bad=1
    if ($5 != "no") bad=1
    if ($6 != "component-managed") bad=1
    if ($7 != "add-component-cache-rule-or-owner-approved-child-migration") bad=1
    if ($8 != meta "/.local/cache/tool") bad=1
    if ($9 != "") bad=1
    if ($10 != "") bad=1
    found_cache=1
  }
  $1 == ".cache" && $2 == "meta-cache" {
    if ($3 != "already-meta") bad=1
    if ($4 != "none") bad=1
    if ($5 != "n/a") bad=1
    if ($6 != "none") bad=1
    if ($7 != "none") bad=1
    if ($8 != meta "/.local/cache/meta-cache") bad=1
    if ($9 != "") bad=1
    if ($10 != "") bad=1
    found_meta_cache=1
  }
  $1 == ".config" && $2 == "managed-app" {
    if ($3 != "managed-config-child") bad=1
    if ($4 != "owner-supervised-config-child-bridge") bad=1
    if ($5 != "no") bad=1
    if ($6 != "owner-reviewed") bad=1
    if ($7 != "review-envctl-home-config-child-before-bridge") bad=1
    if ($8 != meta "/envctl/home/.config/managed-app") bad=1
    if ($9 != meta "/envctl/home/.config/managed-app") bad=1
    if ($10 != "") bad=1
    found_managed=1
  }
  $1 == ".config" && $2 == "app" {
    if ($3 != "config-child") bad=1
    if ($4 != "classify-config-child-before-bridge-or-migration") bad=1
    if ($5 != "no") bad=1
    if ($6 != "owner-reviewed") bad=1
    if ($7 != "classify-config-child-before-bridge-or-migration") bad=1
    if ($8 != meta "/.config/app") bad=1
    if ($9 != "") bad=1
    if ($10 != "") bad=1
    found_config=1
  }
  $1 == ".config" && $2 == "external-app" {
    if ($3 != "external-symlink") bad=1
    if ($4 != "owner-supervised-relink") bad=1
    if ($5 != "no") bad=1
    if ($6 != "owner-reviewed") bad=1
    if ($7 != "review-external-symlink-before-bridge") bad=1
    if ($8 == "") bad=1
    if ($9 != "") bad=1
    if ($10 != "") bad=1
    found_external=1
  }
  $2 == "settings.json" || $2 == "token" { nested=1 }
  $1 == ".ssh" { sensitive=1 }
  END { exit !(found_cache && found_meta_cache && found_managed && found_config && found_external && !bad && !nested && !sensitive) }
' "$tmp/owner-supervised-child-candidate-actions.tsv"

head -n 1 "$tmp/owner-supervised-child-candidates-summary.tsv" | grep -qx $'dot_entry\tchild_target_class\tcandidate_action\tapply_safe\trecommendation\ttotal\tdirect_entries\tdirect_files\tdirect_dirs\tdirect_symlinks'
awk -F '\t' 'NF != 10 { print "bad owner-supervised-child-candidates summary row: " $0 >"/dev/stderr"; bad=1 } END { exit bad }' "$tmp/owner-supervised-child-candidates-summary.tsv"
awk -F '\t' '
  $1 == ".cache" && $2 == "cache-child" {
    if ($3 != "component-managed-cache-child-migration") bad=1
    if ($4 != "no") bad=1
    if ($5 != "add-component-cache-rule-or-owner-approved-child-migration") bad=1
    if ($6 != "1") bad=1
    if ($7 != "1") bad=1
    if ($8 != "1") bad=1
    if ($9 != "0") bad=1
    if ($10 != "0") bad=1
    found_cache=1
  }
  $1 == ".cache" && $2 == "already-meta" {
    if ($3 != "none") bad=1
    if ($4 != "n/a") bad=1
    if ($5 != "none") bad=1
    if ($6 != "1") bad=1
    if ($7 != "1") bad=1
    found_meta_cache=1
  }
  $1 == ".config" && $2 == "managed-config-child" {
    if ($3 != "owner-supervised-config-child-bridge") bad=1
    if ($4 != "no") bad=1
    if ($5 != "owner-review-managed-config-child-before-bridge") bad=1
    if ($6 != "1") bad=1
    if ($7 != "3") bad=1
    if ($8 != "3") bad=1
    found_managed=1
  }
  $1 == ".config" && $2 == "config-child" {
    if ($3 != "classify-config-child-before-bridge-or-migration") bad=1
    if ($4 != "no") bad=1
    if ($5 != "classify-config-child-before-bridge-or-migration") bad=1
    if ($6 != "1") bad=1
    if ($7 != "2") bad=1
    if ($8 != "2") bad=1
    found_config=1
  }
  $1 == ".config" && $2 == "external-symlink" {
    if ($3 != "owner-supervised-relink") bad=1
    if ($4 != "no") bad=1
    if ($5 != "owner-review-before-relink") bad=1
    if ($6 != "1") bad=1
    found_external=1
  }
  END { exit !(found_cache && found_meta_cache && found_managed && found_config && found_external && !bad) }
' "$tmp/owner-supervised-child-candidates-summary.tsv"

"$root/scripts/audit-meta-local-paths.sh" --owner-supervised-child-candidates-summary "$tmp/owner-supervised-child-candidates-summary-only.tsv" --meta-root "$supervised_meta" --real-home "$supervised_home" --envctl-home-source "$supervised_meta/envctl/home" >"$tmp/owner-supervised-child-candidates-summary-only.out" 2>"$tmp/owner-supervised-child-candidates-summary-only.err"
cmp "$tmp/owner-supervised-child-candidates-summary.tsv" "$tmp/owner-supervised-child-candidates-summary-only.tsv"
"$root/scripts/audit-meta-local-paths.sh" --owner-supervised-child-candidate-actions "$tmp/owner-supervised-child-candidate-actions-only.tsv" --meta-root "$supervised_meta" --real-home "$supervised_home" --envctl-home-source "$supervised_meta/envctl/home" >"$tmp/owner-supervised-child-candidate-actions-only.out" 2>"$tmp/owner-supervised-child-candidate-actions-only.err"
cmp "$tmp/owner-supervised-child-candidate-actions.tsv" "$tmp/owner-supervised-child-candidate-actions-only.tsv"

head -n 1 "$tmp/owner-supervised-cache-child-component-plan.tsv" | grep -qx $'dot_entry	child_name	child_path	type	canonical_target	component_key	cache_scope	supervision	next_action	manifest_hint	apply_command'
awk -F '	' 'NF != 11 { print "bad owner-supervised-cache-child-component-plan row: " $0 >"/dev/stderr"; bad=1 } END { exit bad }' "$tmp/owner-supervised-cache-child-component-plan.tsv"
test "$(wc -l <"$tmp/owner-supervised-cache-child-component-plan.tsv" | tr -d '[:space:]')" = 2
awk -F '	' -v home="$supervised_home" -v meta="$supervised_meta" '
  $1 == ".cache" && $2 == "tool" {
    if ($3 != home "/.cache/tool") bad=1
    if ($4 != "directory") bad=1
    if ($5 != meta "/.local/cache/tool") bad=1
    if ($6 != "tool") bad=1
    if ($7 != "cache-child") bad=1
    if ($8 != "component-managed") bad=1
    if ($9 != "add-component-cache-rule-or-owner-approved-child-migration") bad=1
    if ($10 != "manifest/components.d/cache-tool.toml") bad=1
    if ($11 != "") bad=1
    found_cache=1
  }
  $1 == ".cache" && $2 == "meta-cache" { already_meta=1 }
  $1 == ".config" { config_child=1 }
  $2 == "settings.json" || $2 == "token" { nested=1 }
  $1 == ".ssh" { sensitive=1 }
  END { exit !(found_cache && !bad && !already_meta && !config_child && !nested && !sensitive) }
' "$tmp/owner-supervised-cache-child-component-plan.tsv"
"$root/scripts/audit-meta-local-paths.sh" --owner-supervised-cache-child-component-plan "$tmp/owner-supervised-cache-child-component-plan-only.tsv" --meta-root "$supervised_meta" --real-home "$supervised_home" --envctl-home-source "$supervised_meta/envctl/home" >"$tmp/owner-supervised-cache-child-component-plan-only.out" 2>"$tmp/owner-supervised-cache-child-component-plan-only.err"
cmp "$tmp/owner-supervised-cache-child-component-plan.tsv" "$tmp/owner-supervised-cache-child-component-plan-only.tsv"

head -n 1 "$tmp/owner-supervised-managed-config-child-review-plan.tsv" | grep -qx $'dot_entry\tchild_name\tchild_path\ttype\tcanonical_target\tenvctl_home_source\tconfig_scope\tsupervision\tnext_action\treview_hint\tapply_command'
awk -F '\t' 'NF != 11 { print "bad owner-supervised-managed-config-child-review-plan row: " $0 >"/dev/stderr"; bad=1 } END { exit bad }' "$tmp/owner-supervised-managed-config-child-review-plan.tsv"
test "$(wc -l <"$tmp/owner-supervised-managed-config-child-review-plan.tsv" | tr -d '[:space:]')" = 2
awk -F '\t' -v home="$supervised_home" -v meta="$supervised_meta" '
  $1 == ".config" && $2 == "managed-app" {
    if ($3 != home "/.config/managed-app") bad=1
    if ($4 != "directory") bad=1
    if ($5 != meta "/envctl/home/.config/managed-app") bad=1
    if ($6 != meta "/envctl/home/.config/managed-app") bad=1
    if ($7 != "managed-config-child") bad=1
    if ($8 != "owner-reviewed") bad=1
    if ($9 != "review-envctl-home-config-child-before-bridge") bad=1
    if ($10 != "review-envctl-home-source-before-owner-approved-bridge") bad=1
    if ($11 != "") bad=1
    found_managed=1
  }
  $1 == ".cache" { cache_child=1 }
  $1 == ".config" && $2 == "app" { unclassified_config=1 }
  $1 == ".config" && $2 == "external-app" { external_config=1 }
  $2 == "settings.json" || $2 == "token" { nested=1 }
  $1 == ".ssh" { sensitive=1 }
  END { exit !(found_managed && !bad && !cache_child && !unclassified_config && !external_config && !nested && !sensitive) }
' "$tmp/owner-supervised-managed-config-child-review-plan.tsv"
"$root/scripts/audit-meta-local-paths.sh" --owner-supervised-managed-config-child-review-plan "$tmp/owner-supervised-managed-config-child-review-plan-only.tsv" --meta-root "$supervised_meta" --real-home "$supervised_home" --envctl-home-source "$supervised_meta/envctl/home" >"$tmp/owner-supervised-managed-config-child-review-plan-only.out" 2>"$tmp/owner-supervised-managed-config-child-review-plan-only.err"
cmp "$tmp/owner-supervised-managed-config-child-review-plan.tsv" "$tmp/owner-supervised-managed-config-child-review-plan-only.tsv"

head -n 1 "$tmp/owner-supervised-managed-config-child-conflict-plan.tsv" | grep -qx $'dot_entry\tchild_name\treal_path\tmanaged_source\treal_type\tmanaged_type\treal_digest\tmanaged_digest\treal_direct_entries\tmanaged_direct_entries\tsupervision\tnext_action\treview_hint\tapply_command'
awk -F '\t' 'NF != 14 { print "bad owner-supervised-managed-config-child-conflict-plan row: " $0 >"/dev/stderr"; bad=1 } END { exit bad }' "$tmp/owner-supervised-managed-config-child-conflict-plan.tsv"
test "$(wc -l <"$tmp/owner-supervised-managed-config-child-conflict-plan.tsv" | tr -d '[:space:]')" = 2
awk -F '\t' -v home="$supervised_home" -v meta="$supervised_meta" '
  $1 == ".config" && $2 == "managed-app" {
    if ($3 != home "/.config/managed-app") bad=1
    if ($4 != meta "/envctl/home/.config/managed-app") bad=1
    if ($5 != "directory") bad=1
    if ($6 != "directory") bad=1
    if ($7 !~ /^[0-9a-f]{64}$/) bad=1
    if ($8 !~ /^[0-9a-f]{64}$/) bad=1
    if ($9 != "3") bad=1
    if ($10 != "3") bad=1
    if ($11 != "owner-reviewed") bad=1
    if ($12 != "owner-review-real-home-config-child-merge-or-remove-before-bridge") bad=1
    if ($13 != "compare-real-home-and-managed-config-child-before-bridge") bad=1
    if ($14 != "") bad=1
    found_managed=1
  }
  $1 == ".cache" { cache_child=1 }
  $1 == ".config" && $2 == "app" { unclassified_config=1 }
  $1 == ".config" && $2 == "external-app" { external_config=1 }
  $2 == "settings.json" || $2 == "token" { nested=1 }
  $1 == ".ssh" { sensitive=1 }
  END { exit !(found_managed && !bad && !cache_child && !unclassified_config && !external_config && !nested && !sensitive) }
' "$tmp/owner-supervised-managed-config-child-conflict-plan.tsv"
"$root/scripts/audit-meta-local-paths.sh" --owner-supervised-managed-config-child-conflict-plan "$tmp/owner-supervised-managed-config-child-conflict-plan-only.tsv" --meta-root "$supervised_meta" --real-home "$supervised_home" --envctl-home-source "$supervised_meta/envctl/home" >"$tmp/owner-supervised-managed-config-child-conflict-plan-only.out" 2>"$tmp/owner-supervised-managed-config-child-conflict-plan-only.err"
cmp "$tmp/owner-supervised-managed-config-child-conflict-plan.tsv" "$tmp/owner-supervised-managed-config-child-conflict-plan-only.tsv"

head -n 1 "$tmp/owner-supervised-managed-config-child-conflict-summary.tsv" | grep -qx $'dot_entry\tchild_name\treal_type\tmanaged_type\treal_direct_entries\tmanaged_direct_entries\tshared_direct_entries\treal_only_direct_entries\tmanaged_only_direct_entries\ttype_conflict_direct_entries\tdigest_match\tsupervision\tnext_action\tapply_command'
awk -F '\t' 'NF != 14 { print "bad owner-supervised-managed-config-child-conflict-summary row: " $0 >"/dev/stderr"; bad=1 } END { exit bad }' "$tmp/owner-supervised-managed-config-child-conflict-summary.tsv"
test "$(wc -l <"$tmp/owner-supervised-managed-config-child-conflict-summary.tsv" | tr -d '[:space:]')" = 2
awk -F '\t' '
  $1 == ".config" && $2 == "managed-app" {
    if ($3 != "directory") bad=1
    if ($4 != "directory") bad=1
    if ($5 != "3") bad=1
    if ($6 != "3") bad=1
    if ($7 != "2") bad=1
    if ($8 != "1") bad=1
    if ($9 != "1") bad=1
    if ($10 != "1") bad=1
    if ($11 != "no") bad=1
    if ($12 != "owner-reviewed") bad=1
    if ($13 != "owner-review-real-home-config-child-merge-or-remove-before-bridge") bad=1
    if ($14 != "") bad=1
    found_managed=1
  }
  $1 == ".cache" { cache_child=1 }
  $1 == ".config" && $2 == "app" { unclassified_config=1 }
  $1 == ".config" && $2 == "external-app" { external_config=1 }
  $2 == "settings.json" || $2 == "token" || $2 == "real-only.json" || $2 == "managed-only.json" || $2 == "type-conflict" { nested=1 }
  $1 == ".ssh" { sensitive=1 }
  END { exit !(found_managed && !bad && !cache_child && !unclassified_config && !external_config && !nested && !sensitive) }
' "$tmp/owner-supervised-managed-config-child-conflict-summary.tsv"
"$root/scripts/audit-meta-local-paths.sh" --owner-supervised-managed-config-child-conflict-summary "$tmp/owner-supervised-managed-config-child-conflict-summary-only.tsv" --meta-root "$supervised_meta" --real-home "$supervised_home" --envctl-home-source "$supervised_meta/envctl/home" >"$tmp/owner-supervised-managed-config-child-conflict-summary-only.out" 2>"$tmp/owner-supervised-managed-config-child-conflict-summary-only.err"
cmp "$tmp/owner-supervised-managed-config-child-conflict-summary.tsv" "$tmp/owner-supervised-managed-config-child-conflict-summary-only.tsv"

head -n 1 "$tmp/owner-supervised-child-candidate-action-summary.tsv" | grep -qx $'dot_entry\tchild_target_class\tcandidate_action\tapply_safe\tsupervision\tnext_action\ttotal\tenvctl_home_sources'
awk -F '\t' 'NF != 8 { print "bad owner-supervised-child-candidate-action-summary row: " $0 >"/dev/stderr"; bad=1 } END { exit bad }' "$tmp/owner-supervised-child-candidate-action-summary.tsv"
awk -F '\t' '
  $1 == ".cache" && $2 == "cache-child" {
    if ($3 != "component-managed-cache-child-migration") bad=1
    if ($4 != "no") bad=1
    if ($5 != "component-managed") bad=1
    if ($6 != "add-component-cache-rule-or-owner-approved-child-migration") bad=1
    if ($7 != "1") bad=1
    if ($8 != "0") bad=1
    found_cache=1
  }
  $1 == ".cache" && $2 == "already-meta" {
    if ($3 != "none") bad=1
    if ($4 != "n/a") bad=1
    if ($5 != "none") bad=1
    if ($6 != "none") bad=1
    if ($7 != "1") bad=1
    if ($8 != "0") bad=1
    found_meta_cache=1
  }
  $1 == ".config" && $2 == "managed-config-child" {
    if ($3 != "owner-supervised-config-child-bridge") bad=1
    if ($4 != "no") bad=1
    if ($5 != "owner-reviewed") bad=1
    if ($6 != "review-envctl-home-config-child-before-bridge") bad=1
    if ($7 != "1") bad=1
    if ($8 != "1") bad=1
    found_managed=1
  }
  $1 == ".config" && $2 == "config-child" {
    if ($3 != "classify-config-child-before-bridge-or-migration") bad=1
    if ($4 != "no") bad=1
    if ($5 != "owner-reviewed") bad=1
    if ($6 != "classify-config-child-before-bridge-or-migration") bad=1
    if ($7 != "1") bad=1
    if ($8 != "0") bad=1
    found_config=1
  }
  $1 == ".config" && $2 == "external-symlink" {
    if ($3 != "owner-supervised-relink") bad=1
    if ($4 != "no") bad=1
    if ($5 != "owner-reviewed") bad=1
    if ($6 != "review-external-symlink-before-bridge") bad=1
    if ($7 != "1") bad=1
    if ($8 != "0") bad=1
    found_external=1
  }
  END { exit !(found_cache && found_meta_cache && found_managed && found_config && found_external && !bad) }
' "$tmp/owner-supervised-child-candidate-action-summary.tsv"

head -n 1 "$tmp/owner-supervised-config-child-classification-plan.tsv" | grep -qx $'dot_entry\tchild_name\tchild_path\ttype\tcanonical_target\tchild_target_class\tsupervision\tnext_action\tclassification_scope\treview_hint\tapply_command'
awk -F '\t' 'NF != 11 { print "bad owner-supervised-config-child-classification-plan row: " $0 >"/dev/stderr"; bad=1 } END { exit bad }' "$tmp/owner-supervised-config-child-classification-plan.tsv"
test "$(wc -l <"$tmp/owner-supervised-config-child-classification-plan.tsv" | tr -d '[:space:]')" = 2
awk -F '\t' -v home="$supervised_home" -v meta="$supervised_meta" '
  $1 == ".config" && $2 == "app" {
    if ($3 != home "/.config/app") bad=1
    if ($4 != "directory") bad=1
    if ($5 != meta "/.config/app") bad=1
    if ($6 != "config-child") bad=1
    if ($7 != "owner-reviewed") bad=1
    if ($8 != "classify-config-child-before-bridge-or-migration") bad=1
    if ($9 != "unclassified-config-child") bad=1
    if ($10 != "inspect-config-child-before-owner-approved-bridge-or-migration") bad=1
    if ($11 != "") bad=1
    found_config=1
  }
  $1 == ".cache" { cache_child=1 }
  $2 == "managed-app" { managed_config=1 }
  $2 == "external-app" { external_config=1 }
  $2 == "settings.json" || $2 == "token" { nested=1 }
  $1 == ".ssh" { sensitive=1 }
  END { exit !(found_config && !bad && !cache_child && !managed_config && !external_config && !nested && !sensitive) }
' "$tmp/owner-supervised-config-child-classification-plan.tsv"
"$root/scripts/audit-meta-local-paths.sh" --owner-supervised-config-child-classification-plan "$tmp/owner-supervised-config-child-classification-plan-only.tsv" --meta-root "$supervised_meta" --real-home "$supervised_home" --envctl-home-source "$supervised_meta/envctl/home" >"$tmp/owner-supervised-config-child-classification-plan-only.out" 2>"$tmp/owner-supervised-config-child-classification-plan-only.err"
cmp "$tmp/owner-supervised-config-child-classification-plan.tsv" "$tmp/owner-supervised-config-child-classification-plan-only.tsv"

"$root/scripts/audit-meta-local-paths.sh" --owner-supervised-child-candidate-action-summary "$tmp/owner-supervised-child-candidate-action-summary-only.tsv" --meta-root "$supervised_meta" --real-home "$supervised_home" --envctl-home-source "$supervised_meta/envctl/home" >"$tmp/owner-supervised-child-candidate-action-summary-only.out" 2>"$tmp/owner-supervised-child-candidate-action-summary-only.err"
cmp "$tmp/owner-supervised-child-candidate-action-summary.tsv" "$tmp/owner-supervised-child-candidate-action-summary-only.tsv"


"$root/scripts/audit-meta-local-paths.sh" --migration-blockers-plan "$tmp/owner-supervised-plan.tsv" --meta-root "$supervised_meta" --real-home "$supervised_home" --envctl-home-source "$supervised_meta/envctl/home" >"$tmp/owner-supervised-plan.out" 2>"$tmp/owner-supervised-plan.err"
head -n 1 "$tmp/owner-supervised-plan.tsv" | grep -qx $'dot_entry\treal_path\tblocker\tblocker_detail\tapply_safe\topen_handles\trecommendation\tsupervision\tnext_action\tapply_command'
awk -F '\t' 'NF != 10 { print "bad owner-supervised plan row: " $0 >"/dev/stderr"; bad=1 } END { exit bad }' "$tmp/owner-supervised-plan.tsv"
awk -F '\t' -v home="$supervised_home" '
  $1 == ".cache" {
    if ($2 != home "/.cache") bad=1
    if ($3 != "owner-supervised-cache") bad=1
    if ($4 != "component-managed-cache-migration") bad=1
    if ($5 != "no") bad=1
    if ($6 != "n/a") bad=1
    if ($7 != "use-component-managed-cache-migration") bad=1
    if ($8 != "component-managed") bad=1
    if ($9 != "design-component-managed-cache-migration") bad=1
    if ($10 != "") bad=1
    found_cache=1
  }
  $1 == ".config" {
    if ($2 != home "/.config") bad=1
    if ($3 != "owner-supervised-managed-dotfile") bad=1
    if ($4 != "owner-supervised-bridge") bad=1
    if ($5 != "no") bad=1
    if ($6 != "n/a") bad=1
    if ($7 != "owner-review-before-bridge") bad=1
    if ($8 != "owner-reviewed") bad=1
    if ($9 != "owner-review-managed-config-before-bridge") bad=1
    if ($10 != "") bad=1
    found_config=1
  }
  $1 == ".ssh" {
    if ($2 != home "/.ssh") bad=1
    if ($3 != "owner-supervised-sensitive") bad=1
    if ($4 != "credential-or-private-state") bad=1
    if ($5 != "no") bad=1
    if ($6 != "n/a") bad=1
    if ($7 != "owner-supervised-vault-or-bridge") bad=1
    if ($8 != "owner-supervised") bad=1
    if ($9 != "owner-decide-vault-or-bridge-no-automation") bad=1
    if ($10 != "") bad=1
    found_ssh=1
  }
  END { exit !(found_cache && found_config && found_ssh && !bad) }
' "$tmp/owner-supervised-plan.tsv"

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
  "$mig_home/.pi/agent/sessions/--home-drdave-Desktop-meta-Archon--" \
  "$mig_home/.n8n/nodes" \
  "$mig_home/.n8n/storage" \
  "$mig_home/.n8n-claude-bridge/sandbox/.claude/sessions" \
  "$mig_home/.n8n-claude-bridge/sandbox/.cache/claude-cli-nodejs" \
  "$mig_home/.pki/nssdb" \
  "$mig_home/.mcp-auth/mcp-remote-0.1.37" \
  "$mig_home/.lane/ca" \
  "$mig_home/.lane/certs" \
  "$mig_home/.lane/relay" \
  "$mig_home/.fxapp-gh-profile/Default/ClientCertificates" \
  "$mig_home/.fxapp-gh-profile/Default/Sessions" \
  "$mig_home/.fxapp-gh-profile/Default/Local Storage" \
  "$mig_home/.forge/cache/mcp_cache" \
  "$mig_home/.ruvector/models/all-MiniLM-L6-v2" \
  "$mig_home/.repowire" \
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
printf 'nss private key db fixture\n' >"$mig_home/.unknown-ai-sensitive/key4.db"
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
printf '{}\n' >"$mig_home/.pi/agent/auth.json"
printf 'session event\n' >"$mig_home/.pi/agent/sessions/--home-drdave-Desktop-meta-Archon--/events.jsonl"
chmod 700 "$mig_home/.pi" "$mig_home/.pi/agent" "$mig_home/.pi/agent/sessions/--home-drdave-Desktop-meta-Archon--"
chmod 775 "$mig_home/.pi/agent/sessions"
chmod 600 "$mig_home/.pi/agent/auth.json"
printf 'n8n config placeholder\n' >"$mig_home/.n8n/config"
printf 'sqlite-db\n' >"$mig_home/.n8n/database.sqlite"
printf 'sqlite-shm\n' >"$mig_home/.n8n/database.sqlite-shm"
printf 'sqlite-wal\n' >"$mig_home/.n8n/database.sqlite-wal"
printf 'event-log\n' >"$mig_home/.n8n/n8nEventLog-3.log"
printf '{"dependencies":{}}\n' >"$mig_home/.n8n/nodes/package.json"
chmod 775 "$mig_home/.n8n" "$mig_home/.n8n/nodes" "$mig_home/.n8n/storage"
chmod 600 "$mig_home/.n8n/config"
chmod 664 "$mig_home/.n8n/n8nEventLog-3.log" "$mig_home/.n8n/nodes/package.json"
printf '{ "mcpServers": {} }\n' >"$mig_home/.n8n-claude-bridge/sandbox/.claude.json"
printf '{ "token": "redacted-fixture" }\n' >"$mig_home/.n8n-claude-bridge/sandbox/.claude/.credentials.json"
printf 'session event\n' >"$mig_home/.n8n-claude-bridge/sandbox/.claude/sessions/events.jsonl"
printf 'cache-index\n' >"$mig_home/.n8n-claude-bridge/sandbox/.cache/claude-cli-nodejs/index"
chmod 700 "$mig_home/.n8n-claude-bridge" "$mig_home/.n8n-claude-bridge/sandbox" "$mig_home/.n8n-claude-bridge/sandbox/.claude"
chmod 775 "$mig_home/.n8n-claude-bridge/sandbox/.claude/sessions"
chmod 600 "$mig_home/.n8n-claude-bridge/sandbox/.claude.json" "$mig_home/.n8n-claude-bridge/sandbox/.claude/.credentials.json"
printf 'cert db fixture\n' >"$mig_home/.pki/nssdb/cert9.db"
printf 'key db fixture\n' >"$mig_home/.pki/nssdb/key4.db"
printf 'library=\nname=NSS Internal PKCS #11 Module\n' >"$mig_home/.pki/nssdb/pkcs11.txt"
chmod 700 "$mig_home/.pki" "$mig_home/.pki/nssdb"
chmod 600 "$mig_home/.pki/nssdb/cert9.db" "$mig_home/.pki/nssdb/key4.db" "$mig_home/.pki/nssdb/pkcs11.txt"
printf '{"access_token":"redacted-fixture"}\n' >"$mig_home/.mcp-auth/mcp-remote-0.1.37/oauth_tokens.json"
chmod 700 "$mig_home/.mcp-auth" "$mig_home/.mcp-auth/mcp-remote-0.1.37"
chmod 600 "$mig_home/.mcp-auth/mcp-remote-0.1.37/oauth_tokens.json"
printf 'root ca key fixture\n' >"$mig_home/.lane/ca/rootCA-key.pem"
printf 'app key fixture\n' >"$mig_home/.lane/certs/myapp.test-key.pem"
printf 'relay key fixture\n' >"$mig_home/.lane/relay/node.key"
printf 'profile\n' >"$mig_home/.lane/config.yaml"
chmod 700 "$mig_home/.lane"
chmod 600 "$mig_home/.lane/ca/rootCA-key.pem" "$mig_home/.lane/certs/myapp.test-key.pem" "$mig_home/.lane/relay/node.key"
printf 'chrome local state fixture\n' >"$mig_home/.fxapp-gh-profile/Local State"
printf 'chrome preferences fixture\n' >"$mig_home/.fxapp-gh-profile/Default/Preferences"
printf 'sqlite login fixture\n' >"$mig_home/.fxapp-gh-profile/Default/Login Data"
printf 'sqlite cookie fixture\n' >"$mig_home/.fxapp-gh-profile/Default/Cookies"
printf 'session fixture\n' >"$mig_home/.fxapp-gh-profile/Default/Sessions/Session_1"
printf 'client cert fixture\n' >"$mig_home/.fxapp-gh-profile/Default/ClientCertificates/cert.db"
chmod 700 "$mig_home/.fxapp-gh-profile" "$mig_home/.fxapp-gh-profile/Default" "$mig_home/.fxapp-gh-profile/Default/ClientCertificates" "$mig_home/.fxapp-gh-profile/Default/Sessions" "$mig_home/.fxapp-gh-profile/Default/Local Storage"
chmod 600 "$mig_home/.fxapp-gh-profile/Local State" "$mig_home/.fxapp-gh-profile/Default/Preferences" "$mig_home/.fxapp-gh-profile/Default/Login Data" "$mig_home/.fxapp-gh-profile/Default/Cookies" "$mig_home/.fxapp-gh-profile/Default/Sessions/Session_1" "$mig_home/.fxapp-gh-profile/Default/ClientCertificates/cert.db"
printf 'forge history fixture\n' >"$mig_home/.forge/.forge_history"
printf '{ "token": "redacted-fixture" }\n' >"$mig_home/.forge/.credentials.json"
printf 'sqlite-state\n' >"$mig_home/.forge/.forge.db"
printf 'mcp-cache\n' >"$mig_home/.forge/cache/mcp_cache/index"
chmod 775 "$mig_home/.forge" "$mig_home/.forge/cache" "$mig_home/.forge/cache/mcp_cache"
chmod 600 "$mig_home/.forge/.forge_history" "$mig_home/.forge/.credentials.json"
chmod 644 "$mig_home/.forge/.forge.db" "$mig_home/.forge/cache/mcp_cache/index"
printf '{"embedding":"state"}\n' >"$mig_home/.ruvector/intelligence.json"
printf 'tokenizer\n' >"$mig_home/.ruvector/models/all-MiniLM-L6-v2/tokenizer.json"
printf 'onnx-model\n' >"$mig_home/.ruvector/models/all-MiniLM-L6-v2/model.onnx"
chmod 775 "$mig_home/.ruvector" "$mig_home/.ruvector/models" "$mig_home/.ruvector/models/all-MiniLM-L6-v2"
chmod 664 "$mig_home/.ruvector/intelligence.json" "$mig_home/.ruvector/models/all-MiniLM-L6-v2/tokenizer.json" "$mig_home/.ruvector/models/all-MiniLM-L6-v2/model.onnx"
printf 'dsn=local\n' >"$mig_home/.repowire/config.yaml"
printf 'sqlite-state\n' >"$mig_home/.repowire/state.db"
printf '{}\n' >"$mig_home/.repowire/spawn_ownership.json"
printf 'daemon log\n' >"$mig_home/.repowire/daemon.log"
chmod 700 "$mig_home/.repowire"
chmod 600 "$mig_home/.repowire/config.yaml"
chmod 644 "$mig_home/.repowire/state.db"
chmod 664 "$mig_home/.repowire/spawn_ownership.json" "$mig_home/.repowire/daemon.log"
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
grep -qx $'.pi\tdirectory\treal-home-state\tapp-config-state\t'"$mig_meta"$'/.local/share/pi\tmigrate-dir-to-meta-share-and-bridge\tyes' "$tmp/app-config-inventory.tsv"
grep -qx $'.n8n\tdirectory\treal-home-state\tapp-config-state\t'"$mig_meta"$'/.local/share/n8n\tmigrate-dir-to-meta-share-and-bridge\tyes' "$tmp/app-config-inventory.tsv"
grep -qx $'.n8n-claude-bridge\tdirectory\treal-home-state\tapp-config-state\t'"$mig_meta"$'/.local/share/n8n-claude-bridge\tmigrate-dir-to-meta-share-and-bridge\tyes' "$tmp/app-config-inventory.tsv"
grep -qx $'.pki\tdirectory\treal-home-state\tapp-config-state\t'"$mig_meta"$'/.local/share/pki\tmigrate-dir-to-meta-share-and-bridge\tyes' "$tmp/app-config-inventory.tsv"
grep -qx $'.mcp-auth\tdirectory\treal-home-state\tsensitive\t\towner-supervised-vault-or-bridge\tno' "$tmp/app-config-inventory.tsv"
grep -qx $'.lane\tdirectory\treal-home-state\tsensitive\t\towner-supervised-vault-or-bridge\tno' "$tmp/app-config-inventory.tsv"
grep -qx $'.fxapp-gh-profile\tdirectory\treal-home-state\tsensitive\t\towner-supervised-vault-or-bridge\tno' "$tmp/app-config-inventory.tsv"
grep -qx $'.forge\tdirectory\treal-home-state\tapp-config-state\t'"$mig_meta"$'/.local/share/forge\tmigrate-dir-to-meta-share-and-bridge\tyes' "$tmp/app-config-inventory.tsv"
grep -qx $'.ruvector\tdirectory\treal-home-state\tapp-config-state\t'"$mig_meta"$'/.local/share/ruvector\tmigrate-dir-to-meta-share-and-bridge\tyes' "$tmp/app-config-inventory.tsv"
grep -qx $'.repowire\tdirectory\treal-home-state\tapp-config-state\t'"$mig_meta"$'/.local/state/repowire\tmigrate-dir-to-meta-state-and-bridge\tyes' "$tmp/app-config-inventory.tsv"
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
    if ($5 != "6") bad=1
    if ($6 != "2") bad=1
    if ($7 != "2") bad=1
    if ($8 != "1") bad=1
    if ($9 != "3") bad=1
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
if awk -F '\t' '$1 == ".pi" { found=1 } END { exit !found }' "$tmp/unknown-app-config.tsv"; then
  echo "unexpected unknown app-config report row for allow-listed .pi target" >&2
  exit 1
fi
if awk -F '\t' '$1 == ".n8n" { found=1 } END { exit !found }' "$tmp/unknown-app-config.tsv"; then
  echo "unexpected unknown app-config report row for allow-listed .n8n target" >&2
  exit 1
fi
if awk -F '\t' '$1 == ".pki" { found=1 } END { exit !found }' "$tmp/unknown-app-config.tsv"; then
  echo "unexpected unknown app-config report row for allow-listed .pki target" >&2
  exit 1
fi
if awk -F '\t' '$1 == ".lane" { found=1 } END { exit !found }' "$tmp/unknown-app-config.tsv"; then
  echo "unexpected unknown app-config report row for sensitive .lane" >&2
  exit 1
fi
if awk -F '\t' '$1 == ".fxapp-gh-profile" { found=1 } END { exit !found }' "$tmp/unknown-app-config.tsv"; then
  echo "unexpected unknown app-config report row for sensitive .fxapp-gh-profile" >&2
  exit 1
fi
if awk -F '\t' '$1 == ".forge" { found=1 } END { exit !found }' "$tmp/unknown-app-config.tsv"; then
  echo "unexpected unknown app-config report row for allow-listed .forge target" >&2
  exit 1
fi
if awk -F '\t' '$1 == ".ruvector" { found=1 } END { exit !found }' "$tmp/unknown-app-config.tsv"; then
  echo "unexpected unknown app-config report row for allow-listed .ruvector target" >&2
  exit 1
fi
if awk -F '\t' '$1 == ".repowire" { found=1 } END { exit !found }' "$tmp/unknown-app-config.tsv"; then
  echo "unexpected unknown app-config report row for allow-listed .repowire target" >&2
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
if awk -F '\t' '$1 == ".mcp-auth" { found=1 } END { exit !found }' "$tmp/unknown-app-config.tsv"; then
  echo "unexpected unknown app-config report row for sensitive .mcp-auth" >&2
  exit 1
fi

"$root/scripts/audit-meta-local-paths.sh" --sensitive-state-report "$tmp/mig-sensitive-state.tsv" --owner-supervised-sensitive-review-plan "$tmp/mig-sensitive-review-plan.tsv" --meta-root "$mig_meta" --real-home "$mig_home" --envctl-home-source "$mig_meta/envctl/home" >"$tmp/mig-sensitive-state.out" 2>"$tmp/mig-sensitive-state.err"
head -n 1 "$tmp/mig-sensitive-state.tsv" | grep -qx $'dot_entry\treal_path\ttype\tdigest\tentries\tdirect_files\tdirect_dirs\tsymlinks\tsensitive_hints\taction\tapply_safe\trecommendation'
awk -F '\t' 'NF != 12 { print "bad migration sensitive-state row: " $0 >"/dev/stderr"; bad=1 } END { exit bad }' "$tmp/mig-sensitive-state.tsv"
awk -F '\t' -v home="$mig_home" '
  $1 == ".mcp-auth" {
    if ($2 != home "/.mcp-auth") bad=1
    if ($3 != "directory") bad=1
    if ($4 !~ /^[0-9a-f]{64}$/) bad=1
    if ($5 != "2") bad=1
    if ($6 != "0") bad=1
    if ($7 != "1") bad=1
    if ($8 != "0") bad=1
    if ($9 != "1") bad=1
    if ($10 != "owner-supervised-vault-or-bridge") bad=1
    if ($11 != "no") bad=1
    if ($12 != "owner-supervised-vault-or-bridge-before-migration") bad=1
    found=1
  }
  END { exit !(found && !bad) }
' "$tmp/mig-sensitive-state.tsv"

head -n 1 "$tmp/mig-sensitive-review-plan.tsv" | grep -qx $'dot_entry	real_path	type	target_class	digest	entries	direct_files	direct_dirs	symlinks	sensitive_hints	supervision	next_action	sensitive_scope	review_hint	apply_command'
awk -F '	' 'NF != 15 { print "bad migration sensitive review plan row: " $0 >"/dev/stderr"; bad=1 } END { exit bad }' "$tmp/mig-sensitive-review-plan.tsv"
awk -F '	' -v home="$mig_home" '
  $1 == ".mcp-auth" {
    if ($2 != home "/.mcp-auth") bad=1
    if ($3 != "directory") bad=1
    if ($4 != "sensitive") bad=1
    if ($5 !~ /^[0-9a-f]{64}$/) bad=1
    if ($6 != "2") bad=1
    if ($7 != "0") bad=1
    if ($8 != "1") bad=1
    if ($9 != "0") bad=1
    if ($10 != "1") bad=1
    if ($11 != "owner-reviewed") bad=1
    if ($12 != "owner-supervised-vault-or-bridge") bad=1
    if ($13 != "credential-or-private-state") bad=1
    if ($14 != "inspect-sensitive-state-before-owner-approved-vault-or-bridge") bad=1
    if ($15 != "") bad=1
    found=1
  }
  $1 == ".pki" { bad=1 }
  $1 == ".config" { bad=1 }
  $1 == ".cache" { bad=1 }
  END { exit !(found && !bad) }
' "$tmp/mig-sensitive-review-plan.tsv"

ENVCTL_TEST_LSOF_OPEN_SOURCE="$mig_home/.pki" "$root/scripts/audit-meta-local-paths.sh" --migration-blockers-report "$tmp/migration-blockers.tsv" --migration-blockers-summary "$tmp/migration-blockers-summary.tsv" --migration-blockers-plan "$tmp/migration-blockers-plan.tsv" --open-handle-process-window-plan "$tmp/open-handle-process-window-plan.tsv" --meta-root "$mig_meta" --real-home "$mig_home" --envctl-home-source "$mig_meta/envctl/home" >"$tmp/migration-blockers.out" 2>"$tmp/migration-blockers.err"
head -n 1 "$tmp/migration-blockers.tsv" | grep -qx $'dot_entry	real_path	type	target_class	action	apply_safe	canonical_target	sensitive_hints	blocker	blocker_detail	open_handles	open_handle_sample	recommendation'
awk -F '	' 'NF != 13 { print "bad migration blocker row: " $0 >"/dev/stderr"; bad=1 } END { exit bad }' "$tmp/migration-blockers.tsv"
head -n 1 "$tmp/migration-blockers-summary.tsv" | grep -qx $'blocker\ttotal\tapply_safe_yes\tapply_safe_no\topen_handles\trecommendations'
awk -F '\t' 'NF != 6 { print "bad migration blocker summary row: " $0 >"/dev/stderr"; bad=1 } END { exit bad }' "$tmp/migration-blockers-summary.tsv"
head -n 1 "$tmp/migration-blockers-plan.tsv" | grep -qx $'dot_entry\treal_path\tblocker\tblocker_detail\tapply_safe\topen_handles\trecommendation\tsupervision\tnext_action\tapply_command'
awk -F '\t' 'NF != 10 { print "bad migration blocker plan row: " $0 >"/dev/stderr"; bad=1 } END { exit bad }' "$tmp/migration-blockers-plan.tsv"
head -n 1 "$tmp/open-handle-process-window-plan.tsv" | grep -qx $'dot_entry\treal_path\ttype\ttarget_class\tblocker_detail\topen_handles\topen_handle_sample\tsupervision\tnext_action\tretry_command\tapply_command'
awk -F '\t' 'NF != 11 { print "bad open-handle process window plan row: " $0 >"/dev/stderr"; bad=1 } END { exit bad }' "$tmp/open-handle-process-window-plan.tsv"
awk -F '\t' -v home="$mig_home" -v meta="$mig_meta" '
  $1 == ".pki" {
    if ($2 != home "/.pki") bad=1
    if ($3 != "directory") bad=1
    if ($4 != "app-config-state") bad=1
    if ($5 != "migrate-dir-to-meta-share-and-bridge") bad=1
    if ($6 != "yes") bad=1
    if ($7 != meta "/.local/share/pki") bad=1
    if ($8 != "3") bad=1
    if ($9 != "open-handles") bad=1
    if ($10 != "open-handles-present") bad=1
    if ($11 != "1") bad=1
    if ($12 != "chrome/123") bad=1
    if ($13 != "close-processes-then-run-apply-migrate-dot") bad=1
    found=1
  }
  END { exit !(found && !bad) }
' "$tmp/migration-blockers.tsv"
awk -F '\t' -v home="$mig_home" '
  $1 == ".pki" {
    if ($2 != home "/.pki") bad=1
    if ($3 != "open-handles") bad=1
    if ($4 != "open-handles-present") bad=1
    if ($5 != "yes") bad=1
    if ($6 != "1") bad=1
    if ($7 != "close-processes-then-run-apply-migrate-dot") bad=1
    if ($8 != "process-window-required") bad=1
    if ($9 != "close-open-handles-then-rerun-apply-migrate-dot") bad=1
    if ($10 != "scripts/audit-meta-local-paths.sh --apply --migrate-dot .pki") bad=1
    found=1
  }
  END { exit !(found && !bad) }
' "$tmp/migration-blockers-plan.tsv"
awk -F '\t' -v home="$mig_home" '
  $1 == ".pki" {
    if ($2 != home "/.pki") bad=1
    if ($3 != "directory") bad=1
    if ($4 != "app-config-state") bad=1
    if ($5 != "open-handles-present") bad=1
    if ($6 != "1") bad=1
    if ($7 != "chrome/123") bad=1
    if ($8 != "process-window-required") bad=1
    if ($9 != "close-open-handles-then-rerun-apply-migrate-dot") bad=1
    if ($10 != "scripts/audit-meta-local-paths.sh --apply --migrate-dot .pki") bad=1
    if ($11 != "") bad=1
    found=1
  }
  $1 == ".mcp-auth" { bad=1 }
  $1 == ".cache" { bad=1 }
  $1 == ".config" { bad=1 }
  END { exit !(found && !bad) }
' "$tmp/open-handle-process-window-plan.tsv"
awk -F '\t' -v home="$mig_home" '
  $1 == ".mcp-auth" {
    if ($2 != home "/.mcp-auth") bad=1
    if ($3 != "owner-supervised-sensitive") bad=1
    if ($4 != "credential-or-private-state") bad=1
    if ($5 != "no") bad=1
    if ($6 != "n/a") bad=1
    if ($7 != "owner-supervised-vault-or-bridge") bad=1
    if ($8 != "owner-supervised") bad=1
    if ($9 != "owner-decide-vault-or-bridge-no-automation") bad=1
    if ($10 != "") bad=1
    found=1
  }
  END { exit !(found && !bad) }
' "$tmp/migration-blockers-plan.tsv"
awk -F '\t' '
  $1 == "open-handles" {
    if ($2 != "1") bad=1
    if ($3 != "1") bad=1
    if ($4 != "0") bad=1
    if ($5 != "1") bad=1
    if ($6 != "close-processes-then-run-apply-migrate-dot") bad=1
    found=1
  }
  END { exit !(found && !bad) }
' "$tmp/migration-blockers-summary.tsv"
awk -F '\t' '
  $1 == "owner-supervised-sensitive" {
    if ($2 < 3) bad=1
    if ($3 != "0") bad=1
    if ($4 < 3) bad=1
    if ($5 != "0") bad=1
    if ($6 != "owner-supervised-vault-or-bridge") bad=1
    found=1
  }
  END { exit !(found && !bad) }
' "$tmp/migration-blockers-summary.tsv"
ENVCTL_TEST_LSOF_OPEN_SOURCE="$mig_home/.pki" "$root/scripts/audit-meta-local-paths.sh" --migration-blockers-summary "$tmp/migration-blockers-summary-only.tsv" --meta-root "$mig_meta" --real-home "$mig_home" --envctl-home-source "$mig_meta/envctl/home" >"$tmp/migration-blockers-summary-only.out" 2>"$tmp/migration-blockers-summary-only.err"
awk -F '\t' '$1 == "open-handles" && $2 == "1" && $3 == "1" && $4 == "0" && $5 == "1" && $6 == "close-processes-then-run-apply-migrate-dot" { found=1 } END { exit !found }' "$tmp/migration-blockers-summary-only.tsv"
ENVCTL_TEST_LSOF_OPEN_SOURCE="$mig_home/.pki" "$root/scripts/audit-meta-local-paths.sh" --open-handle-process-window-plan "$tmp/open-handle-process-window-plan-only.tsv" --meta-root "$mig_meta" --real-home "$mig_home" --envctl-home-source "$mig_meta/envctl/home" >"$tmp/open-handle-process-window-plan-only.out" 2>"$tmp/open-handle-process-window-plan-only.err"
grep -qx $'.pki\t'"$mig_home"$'/.pki\tdirectory\tapp-config-state\topen-handles-present\t1\tchrome/123\tprocess-window-required\tclose-open-handles-then-rerun-apply-migrate-dot\tscripts/audit-meta-local-paths.sh --apply --migrate-dot .pki\t' "$tmp/open-handle-process-window-plan-only.tsv"
if ENVCTL_TEST_LSOF_OPEN_SOURCE="$mig_home/.pki" "$root/scripts/audit-meta-local-paths.sh" --fail-migration-blockers --meta-root "$mig_meta" --real-home "$mig_home" --envctl-home-source "$mig_meta/envctl/home" >"$tmp/migration-blockers-fail.out" 2>"$tmp/migration-blockers-fail.err"; then
  echo "expected --fail-migration-blockers to fail when residual blockers remain" >&2
  exit 1
fi
grep -q 'FAIL: migration blockers remain' "$tmp/migration-blockers-fail.err"
grep -q 'open-handles=1' "$tmp/migration-blockers-fail.err"
grep -q 'owner-supervised-sensitive=' "$tmp/migration-blockers-fail.err"

clean_meta="$tmp/clean-meta"
clean_home="$tmp/clean-home"
mkdir -p "$clean_meta/.local/bin" "$clean_meta/envctl/home" "$clean_home"
printf '%s\n' '# managed gitconfig' >"$clean_meta/envctl/home/.gitconfig"
ln -s "$clean_meta/envctl/home/.gitconfig" "$clean_meta/.gitconfig"
ln -s "$clean_meta/.local" "$clean_home/.local"
ln -s "$clean_meta/.gitconfig" "$clean_home/.gitconfig"
"$root/scripts/audit-meta-local-paths.sh" --fail-migration-blockers --meta-root "$clean_meta" --real-home "$clean_home" --envctl-home-source "$clean_meta/envctl/home" >"$tmp/migration-blockers-clean.out" 2>"$tmp/migration-blockers-clean.err"
grep -q 'meta-local audit: PASS' "$tmp/migration-blockers-clean.out"
grep -qx $'.mcp-auth\t'"$mig_home"$'/.mcp-auth\tdirectory\tsensitive\towner-supervised-vault-or-bridge\tno\t\t1\towner-supervised-sensitive\tcredential-or-private-state\tn/a\t\towner-supervised-vault-or-bridge' "$tmp/migration-blockers.tsv"
grep -qx $'.lane\t'"$mig_home"$'/.lane\tdirectory\tsensitive\towner-supervised-vault-or-bridge\tno\t\t3\towner-supervised-sensitive\tcredential-or-private-state\tn/a\t\towner-supervised-vault-or-bridge' "$tmp/migration-blockers.tsv"
grep -qx $'.fxapp-gh-profile\t'"$mig_home"$'/.fxapp-gh-profile\tdirectory\tsensitive\towner-supervised-vault-or-bridge\tno\t\t0\towner-supervised-sensitive\tcredential-or-private-state\tn/a\t\towner-supervised-vault-or-bridge' "$tmp/migration-blockers.tsv"
if awk -F '\t' '$1 == ".gitconfig" { found=1 } END { exit !found }' "$tmp/migration-blockers.tsv"; then
  echo "unexpected migration blocker row for already-bridged .gitconfig" >&2
  exit 1
fi
if awk -F '\t' '$1 == ".gitconfig" { found=1 } END { exit !found }' "$tmp/migration-blockers-plan.tsv"; then
  echo "unexpected migration blocker plan row for already-bridged .gitconfig" >&2
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

"$root/scripts/audit-meta-local-paths.sh" --migrate-dot .pi --meta-root "$mig_meta" --real-home "$mig_home" --envctl-home-source "$mig_meta/envctl/home" >"$tmp/migrate-pi-dry.out" 2>"$tmp/migrate-pi-dry.err"
grep -q 'DRY-RUN: would move .*\.pi to .*\.local/share/pi' "$tmp/migrate-pi-dry.out"
test -d "$mig_home/.pi"
test ! -e "$mig_meta/.local/share/pi"

"$root/scripts/audit-meta-local-paths.sh" --apply --migrate-dot .pi --meta-root "$mig_meta" --real-home "$mig_home" --envctl-home-source "$mig_meta/envctl/home" >"$tmp/migrate-pi.out" 2>"$tmp/migrate-pi.err"
test "$(readlink -f "$mig_home/.pi")" = "$mig_meta/.local/share/pi"
grep -Fqx '{}' "$mig_meta/.local/share/pi/agent/auth.json"
grep -Fqx 'session event' "$mig_meta/.local/share/pi/agent/sessions/--home-drdave-Desktop-meta-Archon--/events.jsonl"
test "$(stat -c %a "$mig_meta/.local/share/pi")" = "700"
test "$(stat -c %a "$mig_meta/.local/share/pi/agent/auth.json")" = "600"

"$root/scripts/audit-meta-local-paths.sh" --inventory "$tmp/pi-post.tsv" --inventory-summary "$tmp/pi-post-summary.tsv" --meta-root "$mig_meta" --real-home "$mig_home" --envctl-home-source "$mig_meta/envctl/home" >"$tmp/pi-post.out" 2>"$tmp/pi-post.err"
grep -qx $'.pi	symlink	already-meta	already-meta	'"$mig_meta"$'/.local/share/pi	none	n/a' "$tmp/pi-post.tsv"
"$root/scripts/audit-meta-local-paths.sh" --migrate-dot .repowire --meta-root "$mig_meta" --real-home "$mig_home" --envctl-home-source "$mig_meta/envctl/home" >"$tmp/migrate-repowire-dry.out" 2>"$tmp/migrate-repowire-dry.err"
grep -q 'DRY-RUN: would move .*\.repowire to .*\.local/state/repowire' "$tmp/migrate-repowire-dry.out"
test -d "$mig_home/.repowire"
test ! -e "$mig_meta/.local/state/repowire"

"$root/scripts/audit-meta-local-paths.sh" --apply --migrate-dot .repowire --meta-root "$mig_meta" --real-home "$mig_home" --envctl-home-source "$mig_meta/envctl/home" >"$tmp/migrate-repowire.out" 2>"$tmp/migrate-repowire.err"
test "$(readlink -f "$mig_home/.repowire")" = "$mig_meta/.local/state/repowire"
grep -Fqx 'dsn=local' "$mig_meta/.local/state/repowire/config.yaml"
grep -Fqx 'sqlite-state' "$mig_meta/.local/state/repowire/state.db"
grep -Fqx '{}' "$mig_meta/.local/state/repowire/spawn_ownership.json"
grep -Fqx 'daemon log' "$mig_meta/.local/state/repowire/daemon.log"
test "$(stat -c %a "$mig_meta/.local/state/repowire")" = "700"
test "$(stat -c %a "$mig_meta/.local/state/repowire/config.yaml")" = "600"
test "$(stat -c %a "$mig_meta/.local/state/repowire/state.db")" = "644"
test "$(stat -c %a "$mig_meta/.local/state/repowire/spawn_ownership.json")" = "664"
test "$(stat -c %a "$mig_meta/.local/state/repowire/daemon.log")" = "664"

"$root/scripts/audit-meta-local-paths.sh" --inventory "$tmp/repowire-post.tsv" --inventory-summary "$tmp/repowire-post-summary.tsv" --meta-root "$mig_meta" --real-home "$mig_home" --envctl-home-source "$mig_meta/envctl/home" >"$tmp/repowire-post.out" 2>"$tmp/repowire-post.err"
grep -qx $'.repowire\tsymlink\talready-meta\talready-meta\t'"$mig_meta"$'/.local/state/repowire\tnone\tn/a' "$tmp/repowire-post.tsv"


"$root/scripts/audit-meta-local-paths.sh" --migrate-dot .n8n --meta-root "$mig_meta" --real-home "$mig_home" --envctl-home-source "$mig_meta/envctl/home" >"$tmp/migrate-n8n-dry.out" 2>"$tmp/migrate-n8n-dry.err"
grep -q 'DRY-RUN: would move .*\.n8n to .*\.local/share/n8n' "$tmp/migrate-n8n-dry.out"
test -d "$mig_home/.n8n"
test ! -e "$mig_meta/.local/share/n8n"

"$root/scripts/audit-meta-local-paths.sh" --apply --migrate-dot .n8n --meta-root "$mig_meta" --real-home "$mig_home" --envctl-home-source "$mig_meta/envctl/home" >"$tmp/migrate-n8n.out" 2>"$tmp/migrate-n8n.err"
test "$(readlink -f "$mig_home/.n8n")" = "$mig_meta/.local/share/n8n"
grep -Fqx 'n8n config placeholder' "$mig_meta/.local/share/n8n/config"
grep -Fqx 'sqlite-db' "$mig_meta/.local/share/n8n/database.sqlite"
grep -Fqx 'sqlite-shm' "$mig_meta/.local/share/n8n/database.sqlite-shm"
grep -Fqx 'sqlite-wal' "$mig_meta/.local/share/n8n/database.sqlite-wal"
grep -Fqx 'event-log' "$mig_meta/.local/share/n8n/n8nEventLog-3.log"
grep -Fqx '{"dependencies":{}}' "$mig_meta/.local/share/n8n/nodes/package.json"
test "$(stat -c %a "$mig_meta/.local/share/n8n")" = "775"
test "$(stat -c %a "$mig_meta/.local/share/n8n/config")" = "600"
test "$(stat -c %a "$mig_meta/.local/share/n8n/n8nEventLog-3.log")" = "664"

"$root/scripts/audit-meta-local-paths.sh" --inventory "$tmp/n8n-post.tsv" --inventory-summary "$tmp/n8n-post-summary.tsv" --meta-root "$mig_meta" --real-home "$mig_home" --envctl-home-source "$mig_meta/envctl/home" >"$tmp/n8n-post.out" 2>"$tmp/n8n-post.err"
grep -qx $'.n8n	symlink	already-meta	already-meta	'"$mig_meta"$'/.local/share/n8n	none	n/a' "$tmp/n8n-post.tsv"

"$root/scripts/audit-meta-local-paths.sh" --migrate-dot .n8n-claude-bridge --meta-root "$mig_meta" --real-home "$mig_home" --envctl-home-source "$mig_meta/envctl/home" >"$tmp/migrate-n8n-claude-bridge-dry.out" 2>"$tmp/migrate-n8n-claude-bridge-dry.err"
grep -q 'DRY-RUN: would move .*\.n8n-claude-bridge to .*\.local/share/n8n-claude-bridge' "$tmp/migrate-n8n-claude-bridge-dry.out"
test -d "$mig_home/.n8n-claude-bridge"
test ! -e "$mig_meta/.local/share/n8n-claude-bridge"

"$root/scripts/audit-meta-local-paths.sh" --apply --migrate-dot .n8n-claude-bridge --meta-root "$mig_meta" --real-home "$mig_home" --envctl-home-source "$mig_meta/envctl/home" >"$tmp/migrate-n8n-claude-bridge.out" 2>"$tmp/migrate-n8n-claude-bridge.err"
test "$(readlink -f "$mig_home/.n8n-claude-bridge")" = "$mig_meta/.local/share/n8n-claude-bridge"
grep -Fqx '{ "mcpServers": {} }' "$mig_meta/.local/share/n8n-claude-bridge/sandbox/.claude.json"
grep -Fqx '{ "token": "redacted-fixture" }' "$mig_meta/.local/share/n8n-claude-bridge/sandbox/.claude/.credentials.json"
grep -Fqx 'session event' "$mig_meta/.local/share/n8n-claude-bridge/sandbox/.claude/sessions/events.jsonl"
grep -Fqx 'cache-index' "$mig_meta/.local/share/n8n-claude-bridge/sandbox/.cache/claude-cli-nodejs/index"
test "$(stat -c %a "$mig_meta/.local/share/n8n-claude-bridge")" = "700"
test "$(stat -c %a "$mig_meta/.local/share/n8n-claude-bridge/sandbox/.claude/.credentials.json")" = "600"

"$root/scripts/audit-meta-local-paths.sh" --inventory "$tmp/n8n-claude-bridge-post.tsv" --inventory-summary "$tmp/n8n-claude-bridge-post-summary.tsv" --meta-root "$mig_meta" --real-home "$mig_home" --envctl-home-source "$mig_meta/envctl/home" >"$tmp/n8n-claude-bridge-post.out" 2>"$tmp/n8n-claude-bridge-post.err"
grep -qx $'.n8n-claude-bridge	symlink	already-meta	already-meta	'"$mig_meta"$'/.local/share/n8n-claude-bridge	none	n/a' "$tmp/n8n-claude-bridge-post.tsv"

"$root/scripts/audit-meta-local-paths.sh" --migrate-dot .pki --meta-root "$mig_meta" --real-home "$mig_home" --envctl-home-source "$mig_meta/envctl/home" >"$tmp/migrate-pki-dry.out" 2>"$tmp/migrate-pki-dry.err"
grep -q 'DRY-RUN: would move .*\.pki to .*\.local/share/pki' "$tmp/migrate-pki-dry.out"
test -d "$mig_home/.pki"
test ! -e "$mig_meta/.local/share/pki"

pki_open_meta="$tmp/pki-open-meta"
pki_open_home="$tmp/pki-open-home"
mkdir -p "$pki_open_meta/.local" "$pki_open_meta/envctl/home" "$pki_open_home/.pki/nssdb"
printf '# managed gitconfig\n' >"$pki_open_meta/envctl/home/.gitconfig"
ln -s "$pki_open_meta/envctl/home/.gitconfig" "$pki_open_meta/.gitconfig"
ln -s "$pki_open_meta/.gitconfig" "$pki_open_home/.gitconfig"
ln -s "$pki_open_meta/.local" "$pki_open_home/.local"
printf 'key db fixture\n' >"$pki_open_home/.pki/nssdb/key4.db"
if ENVCTL_TEST_LSOF_OPEN_SOURCE="$pki_open_home/.pki" "$root/scripts/audit-meta-local-paths.sh" --apply --migrate-dot .pki --meta-root "$pki_open_meta" --real-home "$pki_open_home" --envctl-home-source "$pki_open_meta/envctl/home" >"$tmp/migrate-pki-open.out" 2>"$tmp/migrate-pki-open.err"; then
  echo "expected --migrate-dot .pki to fail closed with open file handles" >&2
  exit 1
fi
test -d "$pki_open_home/.pki"
test ! -e "$pki_open_meta/.local/share/pki"
grep -q -- '--migrate-dot .pki: .*open file handle(s).*close owning processes before migration' "$tmp/migrate-pki-open.err"
grep -q -- 'nssdb/key4.db' "$tmp/migrate-pki-open.err"

cache_child_meta="$tmp/cache-child-meta"
cache_child_home="$tmp/cache-child-home"
mkdir -p "$cache_child_meta/.local" "$cache_child_meta/envctl/home" "$cache_child_home/.cache/tool"
printf '# managed gitconfig\n' >"$cache_child_meta/envctl/home/.gitconfig"
ln -s "$cache_child_meta/envctl/home/.gitconfig" "$cache_child_meta/.gitconfig"
ln -s "$cache_child_meta/.local" "$cache_child_home/.local"
ln -s "$cache_child_meta/.gitconfig" "$cache_child_home/.gitconfig"
printf 'cache-index\n' >"$cache_child_home/.cache/tool/index"

"$root/scripts/audit-meta-local-paths.sh" --migrate-cache-child tool --meta-root "$cache_child_meta" --real-home "$cache_child_home" --envctl-home-source "$cache_child_meta/envctl/home" >"$tmp/migrate-cache-child-dry.out" 2>"$tmp/migrate-cache-child-dry.err"
grep -q 'DRY-RUN: would move .*\.cache/tool to .*\.local/cache/tool and link .*\.cache/tool -> .*\.local/cache/tool' "$tmp/migrate-cache-child-dry.out"
test -d "$cache_child_home/.cache/tool"
test ! -e "$cache_child_meta/.local/cache/tool"

if "$root/scripts/audit-meta-local-paths.sh" --migrate-cache-child ../evil --meta-root "$cache_child_meta" --real-home "$cache_child_home" --envctl-home-source "$cache_child_meta/envctl/home" >"$tmp/migrate-cache-child-invalid.out" 2>"$tmp/migrate-cache-child-invalid.err"; then
  echo "expected --migrate-cache-child to reject path-like child names" >&2
  exit 1
fi
grep -q -- '--migrate-cache-child ../evil is not a direct .cache child name' "$tmp/migrate-cache-child-invalid.err"

cache_child_open_meta="$tmp/cache-child-open-meta"
cache_child_open_home="$tmp/cache-child-open-home"
mkdir -p "$cache_child_open_meta/.local" "$cache_child_open_meta/envctl/home" "$cache_child_open_home/.cache/tool"
printf '# managed gitconfig\n' >"$cache_child_open_meta/envctl/home/.gitconfig"
ln -s "$cache_child_open_meta/envctl/home/.gitconfig" "$cache_child_open_meta/.gitconfig"
ln -s "$cache_child_open_meta/.local" "$cache_child_open_home/.local"
ln -s "$cache_child_open_meta/.gitconfig" "$cache_child_open_home/.gitconfig"
printf 'cache-index\n' >"$cache_child_open_home/.cache/tool/index"
if ENVCTL_TEST_LSOF_OPEN_SOURCE="$cache_child_open_home/.cache/tool" "$root/scripts/audit-meta-local-paths.sh" --apply --migrate-cache-child tool --meta-root "$cache_child_open_meta" --real-home "$cache_child_open_home" --envctl-home-source "$cache_child_open_meta/envctl/home" >"$tmp/migrate-cache-child-open.out" 2>"$tmp/migrate-cache-child-open.err"; then
  echo "expected --migrate-cache-child to fail closed with open file handles" >&2
  exit 1
fi
test -d "$cache_child_open_home/.cache/tool"
test ! -e "$cache_child_open_meta/.local/cache/tool"
grep -q -- '--migrate-cache-child tool: .*open file handle(s).*close owning processes before migration' "$tmp/migrate-cache-child-open.err"
grep -q -- 'tool/nssdb/key4.db' "$tmp/migrate-cache-child-open.err"

cache_child_collision_meta="$tmp/cache-child-collision-meta"
cache_child_collision_home="$tmp/cache-child-collision-home"
mkdir -p "$cache_child_collision_meta/.local/cache/tool" "$cache_child_collision_meta/envctl/home" "$cache_child_collision_home/.cache/tool"
printf '# managed gitconfig\n' >"$cache_child_collision_meta/envctl/home/.gitconfig"
ln -s "$cache_child_collision_meta/envctl/home/.gitconfig" "$cache_child_collision_meta/.gitconfig"
ln -s "$cache_child_collision_meta/.local" "$cache_child_collision_home/.local"
ln -s "$cache_child_collision_meta/.gitconfig" "$cache_child_collision_home/.gitconfig"
printf 'source-cache\n' >"$cache_child_collision_home/.cache/tool/index"
printf 'target-cache\n' >"$cache_child_collision_meta/.local/cache/tool/index"
if "$root/scripts/audit-meta-local-paths.sh" --apply --migrate-cache-child tool --meta-root "$cache_child_collision_meta" --real-home "$cache_child_collision_home" --envctl-home-source "$cache_child_collision_meta/envctl/home" >"$tmp/migrate-cache-child-collision.out" 2>"$tmp/migrate-cache-child-collision.err"; then
  echo "expected --migrate-cache-child to reject existing targets" >&2
  exit 1
fi
grep -q -- '--migrate-cache-child tool: existing target .* already exists; refusing automatic cache-child migration' "$tmp/migrate-cache-child-collision.err"
grep -Fqx 'source-cache' "$cache_child_collision_home/.cache/tool/index"
grep -Fqx 'target-cache' "$cache_child_collision_meta/.local/cache/tool/index"

"$root/scripts/audit-meta-local-paths.sh" --apply --migrate-cache-child tool --meta-root "$cache_child_meta" --real-home "$cache_child_home" --envctl-home-source "$cache_child_meta/envctl/home" >"$tmp/migrate-cache-child.out" 2>"$tmp/migrate-cache-child.err"
test "$(readlink -f "$cache_child_home/.cache/tool")" = "$cache_child_meta/.local/cache/tool"
grep -Fqx 'cache-index' "$cache_child_meta/.local/cache/tool/index"
test -d "$cache_child_home/.cache"

"$root/scripts/audit-meta-local-paths.sh" --owner-supervised-child-candidates-report "$tmp/cache-child-post.tsv" --meta-root "$cache_child_meta" --real-home "$cache_child_home" --envctl-home-source "$cache_child_meta/envctl/home" >"$tmp/cache-child-post.out" 2>"$tmp/cache-child-post.err"
awk -F '\t' -v home="$cache_child_home" -v meta="$cache_child_meta" '
  $1 == ".cache" && $2 == "tool" {
    if ($3 != home "/.cache/tool") bad=1
    if ($4 != "symlink") bad=1
    if ($5 != "already-meta") bad=1
    if ($6 != "already-meta") bad=1
    if ($7 != meta "/.local/cache/tool") bad=1
    if ($13 != "none") bad=1
    if ($14 != "n/a") bad=1
    if ($15 != "none") bad=1
    found=1
  }
  END { exit !(found && !bad) }
' "$tmp/cache-child-post.tsv"

config_bridge_meta="$tmp/config-bridge-meta"
config_bridge_home="$tmp/config-bridge-home"
mkdir -p "$config_bridge_meta/.local" "$config_bridge_meta/envctl/home/.config/managed-app" "$config_bridge_home"
printf '# managed gitconfig\n' >"$config_bridge_meta/envctl/home/.gitconfig"
printf 'managed config\n' >"$config_bridge_meta/envctl/home/.config/managed-app/config.toml"
ln -s "$config_bridge_meta/envctl/home/.gitconfig" "$config_bridge_meta/.gitconfig"
ln -s "$config_bridge_meta/.local" "$config_bridge_home/.local"
ln -s "$config_bridge_meta/.gitconfig" "$config_bridge_home/.gitconfig"

"$root/scripts/audit-meta-local-paths.sh" --bridge-managed-config-child managed-app --meta-root "$config_bridge_meta" --real-home "$config_bridge_home" --envctl-home-source "$config_bridge_meta/envctl/home" >"$tmp/config-bridge-dry.out" 2>"$tmp/config-bridge-dry.err"
grep -q 'DRY-RUN: would link .*\.config/managed-app -> .*envctl/home/.config/managed-app' "$tmp/config-bridge-dry.out"
test ! -e "$config_bridge_home/.config/managed-app"

if "$root/scripts/audit-meta-local-paths.sh" --bridge-managed-config-child ../evil --meta-root "$config_bridge_meta" --real-home "$config_bridge_home" --envctl-home-source "$config_bridge_meta/envctl/home" >"$tmp/config-bridge-invalid.out" 2>"$tmp/config-bridge-invalid.err"; then
  echo "expected --bridge-managed-config-child to reject path-like child names" >&2
  exit 1
fi
grep -q -- '--bridge-managed-config-child ../evil is not a direct .config child name' "$tmp/config-bridge-invalid.err"

if "$root/scripts/audit-meta-local-paths.sh" --bridge-managed-config-child missing --meta-root "$config_bridge_meta" --real-home "$config_bridge_home" --envctl-home-source "$config_bridge_meta/envctl/home" >"$tmp/config-bridge-missing.out" 2>"$tmp/config-bridge-missing.err"; then
  echo "expected --bridge-managed-config-child to reject missing managed sources" >&2
  exit 1
fi
grep -q -- '--bridge-managed-config-child missing: managed source .* is missing; refusing automatic managed config-child bridge' "$tmp/config-bridge-missing.err"

config_bridge_external_meta="$tmp/config-bridge-external-meta"
config_bridge_external_home="$tmp/config-bridge-external-home"
mkdir -p "$config_bridge_external_meta/.local" "$config_bridge_external_meta/envctl/home/.config/managed-app" "$config_bridge_external_home/.config"
printf '# managed gitconfig\n' >"$config_bridge_external_meta/envctl/home/.gitconfig"
printf 'managed config\n' >"$config_bridge_external_meta/envctl/home/.config/managed-app/config.toml"
ln -s "$config_bridge_external_meta/envctl/home/.gitconfig" "$config_bridge_external_meta/.gitconfig"
ln -s "$config_bridge_external_meta/.local" "$config_bridge_external_home/.local"
ln -s "$config_bridge_external_meta/.gitconfig" "$config_bridge_external_home/.gitconfig"
ln -s "$outside/hf" "$config_bridge_external_home/.config/managed-app"
if "$root/scripts/audit-meta-local-paths.sh" --bridge-managed-config-child managed-app --meta-root "$config_bridge_external_meta" --real-home "$config_bridge_external_home" --envctl-home-source "$config_bridge_external_meta/envctl/home" >"$tmp/config-bridge-external.out" 2>"$tmp/config-bridge-external.err"; then
  echo "expected --bridge-managed-config-child to reject external real-home symlinks" >&2
  exit 1
fi
grep -q -- '--bridge-managed-config-child managed-app: .*\.config/managed-app is an external symlink .*refusing automatic managed config-child bridge' "$tmp/config-bridge-external.err"

config_bridge_existing_meta="$tmp/config-bridge-existing-meta"
config_bridge_existing_home="$tmp/config-bridge-existing-home"
mkdir -p "$config_bridge_existing_meta/.local" "$config_bridge_existing_meta/envctl/home/.config/managed-app" "$config_bridge_existing_home/.config/managed-app"
printf '# managed gitconfig\n' >"$config_bridge_existing_meta/envctl/home/.gitconfig"
printf 'managed config\n' >"$config_bridge_existing_meta/envctl/home/.config/managed-app/config.toml"
printf 'real config\n' >"$config_bridge_existing_home/.config/managed-app/config.toml"
ln -s "$config_bridge_existing_meta/envctl/home/.gitconfig" "$config_bridge_existing_meta/.gitconfig"
ln -s "$config_bridge_existing_meta/.local" "$config_bridge_existing_home/.local"
ln -s "$config_bridge_existing_meta/.gitconfig" "$config_bridge_existing_home/.gitconfig"
"$root/scripts/audit-meta-local-paths.sh" --bridge-managed-config-child managed-app --meta-root "$config_bridge_existing_meta" --real-home "$config_bridge_existing_home" --envctl-home-source "$config_bridge_existing_meta/envctl/home" >"$tmp/config-bridge-existing-dry.out" 2>"$tmp/config-bridge-existing-dry.err"
grep -q 'DRY-RUN: would refuse automatic managed config-child bridge because source .* already exists; owner-reviewed merge/removal required before bridge' "$tmp/config-bridge-existing-dry.out"
if "$root/scripts/audit-meta-local-paths.sh" --apply --bridge-managed-config-child managed-app --meta-root "$config_bridge_existing_meta" --real-home "$config_bridge_existing_home" --envctl-home-source "$config_bridge_existing_meta/envctl/home" >"$tmp/config-bridge-existing.out" 2>"$tmp/config-bridge-existing.err"; then
  echo "expected --bridge-managed-config-child to reject existing real-home state on apply" >&2
  exit 1
fi
grep -q -- '--bridge-managed-config-child managed-app: source .* already exists; owner-reviewed merge/removal required before bridge' "$tmp/config-bridge-existing.err"
grep -Fqx 'real config' "$config_bridge_existing_home/.config/managed-app/config.toml"
grep -Fqx 'managed config' "$config_bridge_existing_meta/envctl/home/.config/managed-app/config.toml"

"$root/scripts/audit-meta-local-paths.sh" --apply --bridge-managed-config-child managed-app --meta-root "$config_bridge_meta" --real-home "$config_bridge_home" --envctl-home-source "$config_bridge_meta/envctl/home" >"$tmp/config-bridge.out" 2>"$tmp/config-bridge.err"
test "$(readlink -f "$config_bridge_home/.config/managed-app")" = "$config_bridge_meta/envctl/home/.config/managed-app"
grep -Fqx 'managed config' "$config_bridge_meta/envctl/home/.config/managed-app/config.toml"

"$root/scripts/audit-meta-local-paths.sh" --owner-supervised-child-candidates-report "$tmp/config-bridge-post.tsv" --meta-root "$config_bridge_meta" --real-home "$config_bridge_home" --envctl-home-source "$config_bridge_meta/envctl/home" >"$tmp/config-bridge-post.out" 2>"$tmp/config-bridge-post.err"
awk -F '\t' -v home="$config_bridge_home" -v meta="$config_bridge_meta" '
  $1 == ".config" && $2 == "managed-app" {
    if ($3 != home "/.config/managed-app") bad=1
    if ($4 != "symlink") bad=1
    if ($5 != "already-meta") bad=1
    if ($6 != "already-meta") bad=1
    if ($7 != meta "/envctl/home/.config/managed-app") bad=1
    if ($13 != "none") bad=1
    if ($14 != "n/a") bad=1
    if ($15 != "none") bad=1
    found=1
  }
  END { exit !(found && !bad) }
' "$tmp/config-bridge-post.tsv"

"$root/scripts/audit-meta-local-paths.sh" --apply --migrate-dot .pki --meta-root "$mig_meta" --real-home "$mig_home" --envctl-home-source "$mig_meta/envctl/home" >"$tmp/migrate-pki.out" 2>"$tmp/migrate-pki.err"
test "$(readlink -f "$mig_home/.pki")" = "$mig_meta/.local/share/pki"
grep -Fqx 'cert db fixture' "$mig_meta/.local/share/pki/nssdb/cert9.db"
grep -Fqx 'key db fixture' "$mig_meta/.local/share/pki/nssdb/key4.db"
grep -Fqx 'library=' "$mig_meta/.local/share/pki/nssdb/pkcs11.txt"
test "$(stat -c %a "$mig_meta/.local/share/pki")" = "700"
test "$(stat -c %a "$mig_meta/.local/share/pki/nssdb")" = "700"
test "$(stat -c %a "$mig_meta/.local/share/pki/nssdb/key4.db")" = "600"

"$root/scripts/audit-meta-local-paths.sh" --inventory "$tmp/pki-post.tsv" --inventory-summary "$tmp/pki-post-summary.tsv" --meta-root "$mig_meta" --real-home "$mig_home" --envctl-home-source "$mig_meta/envctl/home" >"$tmp/pki-post.out" 2>"$tmp/pki-post.err"
grep -qx $'.pki	symlink	already-meta	already-meta	'"$mig_meta"$'/.local/share/pki	none	n/a' "$tmp/pki-post.tsv"

"$root/scripts/audit-meta-local-paths.sh" --migrate-dot .forge --meta-root "$mig_meta" --real-home "$mig_home" --envctl-home-source "$mig_meta/envctl/home" >"$tmp/migrate-forge-dry.out" 2>"$tmp/migrate-forge-dry.err"
grep -q 'DRY-RUN: would move .*\.forge to .*\.local/share/forge' "$tmp/migrate-forge-dry.out"
test -d "$mig_home/.forge"
test ! -e "$mig_meta/.local/share/forge"

"$root/scripts/audit-meta-local-paths.sh" --apply --migrate-dot .forge --meta-root "$mig_meta" --real-home "$mig_home" --envctl-home-source "$mig_meta/envctl/home" >"$tmp/migrate-forge.out" 2>"$tmp/migrate-forge.err"
test "$(readlink -f "$mig_home/.forge")" = "$mig_meta/.local/share/forge"
grep -Fqx 'forge history fixture' "$mig_meta/.local/share/forge/.forge_history"
grep -Fqx '{ "token": "redacted-fixture" }' "$mig_meta/.local/share/forge/.credentials.json"
grep -Fqx 'sqlite-state' "$mig_meta/.local/share/forge/.forge.db"
grep -Fqx 'mcp-cache' "$mig_meta/.local/share/forge/cache/mcp_cache/index"
test "$(stat -c %a "$mig_meta/.local/share/forge")" = "775"
test "$(stat -c %a "$mig_meta/.local/share/forge/.credentials.json")" = "600"
test "$(stat -c %a "$mig_meta/.local/share/forge/.forge.db")" = "644"

"$root/scripts/audit-meta-local-paths.sh" --inventory "$tmp/forge-post.tsv" --inventory-summary "$tmp/forge-post-summary.tsv" --meta-root "$mig_meta" --real-home "$mig_home" --envctl-home-source "$mig_meta/envctl/home" >"$tmp/forge-post.out" 2>"$tmp/forge-post.err"
grep -qx $'.forge	symlink	already-meta	already-meta	'"$mig_meta"$'/.local/share/forge	none	n/a' "$tmp/forge-post.tsv"

"$root/scripts/audit-meta-local-paths.sh" --migrate-dot .ruvector --meta-root "$mig_meta" --real-home "$mig_home" --envctl-home-source "$mig_meta/envctl/home" >"$tmp/migrate-ruvector-dry.out" 2>"$tmp/migrate-ruvector-dry.err"
grep -q 'DRY-RUN: would move .*\.ruvector to .*\.local/share/ruvector' "$tmp/migrate-ruvector-dry.out"
test -d "$mig_home/.ruvector"
test ! -e "$mig_meta/.local/share/ruvector"

"$root/scripts/audit-meta-local-paths.sh" --apply --migrate-dot .ruvector --meta-root "$mig_meta" --real-home "$mig_home" --envctl-home-source "$mig_meta/envctl/home" >"$tmp/migrate-ruvector.out" 2>"$tmp/migrate-ruvector.err"
test "$(readlink -f "$mig_home/.ruvector")" = "$mig_meta/.local/share/ruvector"
grep -Fqx '{"embedding":"state"}' "$mig_meta/.local/share/ruvector/intelligence.json"
grep -Fqx 'tokenizer' "$mig_meta/.local/share/ruvector/models/all-MiniLM-L6-v2/tokenizer.json"
grep -Fqx 'onnx-model' "$mig_meta/.local/share/ruvector/models/all-MiniLM-L6-v2/model.onnx"
test "$(stat -c %a "$mig_meta/.local/share/ruvector")" = "775"
test "$(stat -c %a "$mig_meta/.local/share/ruvector/intelligence.json")" = "664"
test "$(stat -c %a "$mig_meta/.local/share/ruvector/models/all-MiniLM-L6-v2/model.onnx")" = "664"

"$root/scripts/audit-meta-local-paths.sh" --inventory "$tmp/ruvector-post.tsv" --inventory-summary "$tmp/ruvector-post-summary.tsv" --meta-root "$mig_meta" --real-home "$mig_home" --envctl-home-source "$mig_meta/envctl/home" >"$tmp/ruvector-post.out" 2>"$tmp/ruvector-post.err"
grep -qx $'.ruvector	symlink	already-meta	already-meta	'"$mig_meta"$'/.local/share/ruvector	none	n/a' "$tmp/ruvector-post.tsv"

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
printf '# managed gitconfig\n' >"$ai_bad_meta/envctl/home/.gitconfig"
ln -s "$ai_bad_meta/envctl/home/.gitconfig" "$ai_bad_meta/.gitconfig"
ln -s "$ai_bad_meta/.gitconfig" "$ai_bad_home/.gitconfig"
ln -s "$ai_bad_meta/.local" "$ai_bad_home/.local"
printf 'not a directory\n' >"$ai_bad_home/.ai"
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

pi_bad_meta="$tmp/pi-bad-meta"
pi_bad_home="$tmp/pi-bad-home"
mkdir -p "$pi_bad_meta/.local" "$pi_bad_meta/envctl/home" "$pi_bad_home"
printf '# managed gitconfig\n' >"$pi_bad_meta/envctl/home/.gitconfig"
ln -s "$pi_bad_meta/envctl/home/.gitconfig" "$pi_bad_meta/.gitconfig"
ln -s "$pi_bad_meta/.gitconfig" "$pi_bad_home/.gitconfig"
ln -s "$pi_bad_meta/.local" "$pi_bad_home/.local"
printf 'not a directory\n' >"$pi_bad_home/.pi"
if "$root/scripts/audit-meta-local-paths.sh" --apply --migrate-dot .pi --meta-root "$pi_bad_meta" --real-home "$pi_bad_home" --envctl-home-source "$pi_bad_meta/envctl/home" >"$tmp/migrate-pi-file.out" 2>"$tmp/migrate-pi-file.err"; then
  echo "expected --migrate-dot .pi to fail closed for non-directory source" >&2
  exit 1
fi
test -f "$pi_bad_home/.pi"
test ! -e "$pi_bad_meta/.local/share/pi"
grep -q -- '--migrate-dot .pi: .* is not a directory; refusing automatic app-config directory migration' "$tmp/migrate-pi-file.err"

n8n_bad_meta="$tmp/n8n-bad-meta"
n8n_bad_home="$tmp/n8n-bad-home"
mkdir -p "$n8n_bad_meta/.local" "$n8n_bad_meta/envctl/home" "$n8n_bad_home"
printf '# managed gitconfig\n' >"$n8n_bad_meta/envctl/home/.gitconfig"
ln -s "$n8n_bad_meta/envctl/home/.gitconfig" "$n8n_bad_meta/.gitconfig"
ln -s "$n8n_bad_meta/.gitconfig" "$n8n_bad_home/.gitconfig"
ln -s "$n8n_bad_meta/.local" "$n8n_bad_home/.local"
printf 'not a directory\n' >"$n8n_bad_home/.n8n"
if "$root/scripts/audit-meta-local-paths.sh" --apply --migrate-dot .n8n --meta-root "$n8n_bad_meta" --real-home "$n8n_bad_home" --envctl-home-source "$n8n_bad_meta/envctl/home" >"$tmp/migrate-n8n-file.out" 2>"$tmp/migrate-n8n-file.err"; then
  echo "expected --migrate-dot .n8n to fail closed for non-directory source" >&2
  exit 1
fi
test -f "$n8n_bad_home/.n8n"
test ! -e "$n8n_bad_meta/.local/share/n8n"
grep -q -- '--migrate-dot .n8n: .* is not a directory; refusing automatic app-config directory migration' "$tmp/migrate-n8n-file.err"

n8n_claude_bridge_bad_meta="$tmp/n8n-claude-bridge-bad-meta"
n8n_claude_bridge_bad_home="$tmp/n8n-claude-bridge-bad-home"
mkdir -p "$n8n_claude_bridge_bad_meta/.local" "$n8n_claude_bridge_bad_meta/envctl/home" "$n8n_claude_bridge_bad_home"
printf '# managed gitconfig\n' >"$n8n_claude_bridge_bad_meta/envctl/home/.gitconfig"
ln -s "$n8n_claude_bridge_bad_meta/envctl/home/.gitconfig" "$n8n_claude_bridge_bad_meta/.gitconfig"
ln -s "$n8n_claude_bridge_bad_meta/.gitconfig" "$n8n_claude_bridge_bad_home/.gitconfig"
ln -s "$n8n_claude_bridge_bad_meta/.local" "$n8n_claude_bridge_bad_home/.local"
printf 'not a directory\n' >"$n8n_claude_bridge_bad_home/.n8n-claude-bridge"
if "$root/scripts/audit-meta-local-paths.sh" --apply --migrate-dot .n8n-claude-bridge --meta-root "$n8n_claude_bridge_bad_meta" --real-home "$n8n_claude_bridge_bad_home" --envctl-home-source "$n8n_claude_bridge_bad_meta/envctl/home" >"$tmp/migrate-n8n-claude-bridge-file.out" 2>"$tmp/migrate-n8n-claude-bridge-file.err"; then
  echo "expected --migrate-dot .n8n-claude-bridge to fail closed for non-directory source" >&2
  exit 1
fi
test -f "$n8n_claude_bridge_bad_home/.n8n-claude-bridge"
test ! -e "$n8n_claude_bridge_bad_meta/.local/share/n8n-claude-bridge"
grep -q -- '--migrate-dot .n8n-claude-bridge: .* is not a directory; refusing automatic app-config directory migration' "$tmp/migrate-n8n-claude-bridge-file.err"

pki_bad_meta="$tmp/pki-bad-meta"
pki_bad_home="$tmp/pki-bad-home"
mkdir -p "$pki_bad_meta/.local" "$pki_bad_meta/envctl/home" "$pki_bad_home"
printf '# managed gitconfig\n' >"$pki_bad_meta/envctl/home/.gitconfig"
ln -s "$pki_bad_meta/envctl/home/.gitconfig" "$pki_bad_meta/.gitconfig"
ln -s "$pki_bad_meta/.gitconfig" "$pki_bad_home/.gitconfig"
ln -s "$pki_bad_meta/.local" "$pki_bad_home/.local"
printf 'not a directory\n' >"$pki_bad_home/.pki"
if "$root/scripts/audit-meta-local-paths.sh" --apply --migrate-dot .pki --meta-root "$pki_bad_meta" --real-home "$pki_bad_home" --envctl-home-source "$pki_bad_meta/envctl/home" >"$tmp/migrate-pki-file.out" 2>"$tmp/migrate-pki-file.err"; then
  echo "expected --migrate-dot .pki to fail closed for non-directory source" >&2
  exit 1
fi
test -f "$pki_bad_home/.pki"
test ! -e "$pki_bad_meta/.local/share/pki"
grep -q -- '--migrate-dot .pki: .* is not a directory; refusing automatic app-config directory migration' "$tmp/migrate-pki-file.err"

forge_bad_meta="$tmp/forge-bad-meta"
forge_bad_home="$tmp/forge-bad-home"
mkdir -p "$forge_bad_meta/.local" "$forge_bad_meta/envctl/home" "$forge_bad_home"
printf '# managed gitconfig\n' >"$forge_bad_meta/envctl/home/.gitconfig"
ln -s "$forge_bad_meta/envctl/home/.gitconfig" "$forge_bad_meta/.gitconfig"
ln -s "$forge_bad_meta/.gitconfig" "$forge_bad_home/.gitconfig"
ln -s "$forge_bad_meta/.local" "$forge_bad_home/.local"
printf 'not a directory\n' >"$forge_bad_home/.forge"
if "$root/scripts/audit-meta-local-paths.sh" --apply --migrate-dot .forge --meta-root "$forge_bad_meta" --real-home "$forge_bad_home" --envctl-home-source "$forge_bad_meta/envctl/home" >"$tmp/migrate-forge-file.out" 2>"$tmp/migrate-forge-file.err"; then
  echo "expected --migrate-dot .forge to fail closed for non-directory source" >&2
  exit 1
fi
test -f "$forge_bad_home/.forge"
test ! -e "$forge_bad_meta/.local/share/forge"
grep -q -- '--migrate-dot .forge: .* is not a directory; refusing automatic app-config directory migration' "$tmp/migrate-forge-file.err"

ruvector_bad_meta="$tmp/ruvector-bad-meta"
ruvector_bad_home="$tmp/ruvector-bad-home"
mkdir -p "$ruvector_bad_meta/.local" "$ruvector_bad_meta/envctl/home" "$ruvector_bad_home"
printf '# managed gitconfig\n' >"$ruvector_bad_meta/envctl/home/.gitconfig"
ln -s "$ruvector_bad_meta/envctl/home/.gitconfig" "$ruvector_bad_meta/.gitconfig"
ln -s "$ruvector_bad_meta/.gitconfig" "$ruvector_bad_home/.gitconfig"
ln -s "$ruvector_bad_meta/.local" "$ruvector_bad_home/.local"
printf 'not a directory\n' >"$ruvector_bad_home/.ruvector"
if "$root/scripts/audit-meta-local-paths.sh" --apply --migrate-dot .ruvector --meta-root "$ruvector_bad_meta" --real-home "$ruvector_bad_home" --envctl-home-source "$ruvector_bad_meta/envctl/home" >"$tmp/migrate-ruvector-file.out" 2>"$tmp/migrate-ruvector-file.err"; then
  echo "expected --migrate-dot .ruvector to fail closed for non-directory source" >&2
  exit 1
fi
test -f "$ruvector_bad_home/.ruvector"
test ! -e "$ruvector_bad_meta/.local/share/ruvector"
grep -q -- '--migrate-dot .ruvector: .* is not a directory; refusing automatic app-config directory migration' "$tmp/migrate-ruvector-file.err"

repowire_bad_meta="$tmp/repowire-bad-meta"
repowire_bad_home="$tmp/repowire-bad-home"
mkdir -p "$repowire_bad_meta/.local" "$repowire_bad_meta/envctl/home" "$repowire_bad_home"
printf '# managed gitconfig\n' >"$repowire_bad_meta/envctl/home/.gitconfig"
ln -s "$repowire_bad_meta/envctl/home/.gitconfig" "$repowire_bad_meta/.gitconfig"
ln -s "$repowire_bad_meta/.gitconfig" "$repowire_bad_home/.gitconfig"
ln -s "$repowire_bad_meta/.local" "$repowire_bad_home/.local"
printf 'not a directory\n' >"$repowire_bad_home/.repowire"
if "$root/scripts/audit-meta-local-paths.sh" --apply --migrate-dot .repowire --meta-root "$repowire_bad_meta" --real-home "$repowire_bad_home" --envctl-home-source "$repowire_bad_meta/envctl/home" >"$tmp/migrate-repowire-file.out" 2>"$tmp/migrate-repowire-file.err"; then
  echo "expected --migrate-dot .repowire to fail closed for non-directory source" >&2
  exit 1
fi
test -f "$repowire_bad_home/.repowire"
test ! -e "$repowire_bad_meta/.local/state/repowire"
grep -q -- '--migrate-dot .repowire: .* is not a directory; refusing automatic app-config directory migration' "$tmp/migrate-repowire-file.err"

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
