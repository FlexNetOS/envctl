#!/usr/bin/env bash
set -euo pipefail
root="$(git -C "$(dirname "${BASH_SOURCE[0]}")/../.." rev-parse --show-toplevel)"
tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT

meta="$tmp/meta"
home="$tmp/home"
outside="$tmp/outside"
mkdir -p "$meta/.local/bin" "$meta/usr/bin" "$meta/envctl/home" "$home" "$outside" "$home/.ssh" "$home/.aws"
printf '# managed gitconfig\n' >"$meta/envctl/home/.gitconfig"
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
"$root/scripts/audit-meta-local-paths.sh" --meta-root "$meta" --real-home "$home" --envctl-home-source "$meta/envctl/home" >"$tmp/post.out" 2>"$tmp/post.err"

test "$(readlink -f "$home/.local")" = "$meta/.local"
test "$(readlink "$home/.gitconfig")" = "$meta/.gitconfig"
test "$(readlink -f "$home/.gitconfig")" = "$meta/envctl/home/.gitconfig"
test "$(readlink -f "$meta/.local/bin/hf")" = "$meta/usr/bin/hf"
grep -q 'WARN: .*\.ssh is real-home state outside META_ROOT; skipped automatic move' "$tmp/post.err"
grep -q 'WARN: .*\.aws is real-home state outside META_ROOT; skipped automatic move' "$tmp/post.err"
grep -q 'WARN: .*\.zshrc is real-home state outside META_ROOT; skipped automatic move' "$tmp/post.err"
grep -q 'OK: .*\.inside-link resolves inside META_ROOT' "$tmp/post.out"
grep -q 'WARN: .*\.outside-link symlink resolves outside META_ROOT' "$tmp/post.err"
grep -q 'meta-local audit: PASS' "$tmp/post.out"
grep -q 'dot_entries=7' "$tmp/post.out"

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
