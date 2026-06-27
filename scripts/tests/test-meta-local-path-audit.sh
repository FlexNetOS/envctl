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
mkdir -p "$mig_meta/.local" "$mig_meta/envctl/home" "$mig_home/.cargo" "$mig_home/.npm" "$mig_home/.dotnet"
printf '# managed gitconfig\n' >"$mig_meta/envctl/home/.gitconfig"
ln -s "$mig_meta/envctl/home/.gitconfig" "$mig_meta/.gitconfig"
ln -s "$mig_meta/.gitconfig" "$mig_home/.gitconfig"
ln -s "$mig_meta/.local" "$mig_home/.local"
printf 'real-home cargo state\n' >"$mig_home/.cargo/config"
printf 'real-home npm state\n' >"$mig_home/.npm/npmrc"
printf 'real-home dotnet state\n' >"$mig_home/.dotnet/state"

"$root/scripts/audit-meta-local-paths.sh" --migrate-dot .cargo --meta-root "$mig_meta" --real-home "$mig_home" --envctl-home-source "$mig_meta/envctl/home" >"$tmp/migrate-dry.out" 2>"$tmp/migrate-dry.err"
grep -q 'DRY-RUN: would move .*\.cargo to .*\.toolchains/cargo' "$tmp/migrate-dry.out"
test -d "$mig_home/.cargo"
test ! -e "$mig_meta/.toolchains/cargo"

"$root/scripts/audit-meta-local-paths.sh" --apply --migrate-dot .npm --meta-root "$mig_meta" --real-home "$mig_home" --envctl-home-source "$mig_meta/envctl/home" >"$tmp/migrate-npm.out" 2>"$tmp/migrate-npm.err"
test "$(readlink -f "$mig_home/.npm")" = "$mig_meta/.toolchains/npm"
test -f "$mig_meta/.toolchains/npm/npmrc"

"$root/scripts/audit-meta-local-paths.sh" --apply --migrate-dot .dotnet --meta-root "$mig_meta" --real-home "$mig_home" --envctl-home-source "$mig_meta/envctl/home" >"$tmp/migrate-dotnet.out" 2>"$tmp/migrate-dotnet.err"
test "$(readlink -f "$mig_home/.dotnet")" = "$mig_meta/.toolchains/dotnet"
test -f "$mig_meta/.toolchains/dotnet/state"

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
