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
