#!/usr/bin/env bash
set -Eeuo pipefail

ROOT_INPUT="${1:-$(pwd)}"
COMPARE_INPUT="${2:-}"

expand_path() {
  local p="$1"
  if [[ -z "$p" ]]; then printf ''; return 0; fi
  if [[ "$p" == ~/* ]]; then printf '%s/%s' "$HOME" "${p#~/}"; else printf '%s' "$p"; fi
}

ROOT="$(expand_path "$ROOT_INPUT")"
COMPARE="$(expand_path "$COMPARE_INPUT")"

if [[ ! -d "$ROOT" ]]; then
  echo "ERROR: ROOT does not exist: $ROOT" >&2
  exit 2
fi

cd "$ROOT"
RUN_ID="$(date -u +%Y%m%dT%H%M%SZ)"
OUT="${MIGRATION_SCAN_OUT:-$ROOT/migration-artifacts/_raw/$RUN_ID}"
mkdir -p "$OUT" "$ROOT/migration-artifacts/_meta"

EXCLUDES='(.git|node_modules|vendor|dist|build|target|.venv|venv|__pycache__|.cache|coverage|.next|.nuxt|.terraform)'

have() { command -v "$1" >/dev/null 2>&1; }
log() { printf '[%s] %s\n' "$(date -u +%H:%M:%S)" "$*" >&2; }

run_bg() {
  local name="$1"; shift
  (
    set +e
    log "start $name"
    "$@" >"$OUT/$name.out" 2>"$OUT/$name.err"
    code=$?
    printf '%s\t%s\t%s\n' "$name" "$code" "$(date -u +%Y-%m-%dT%H:%M:%SZ)" >>"$OUT/jobs.tsv"
    log "done $name code=$code"
    exit 0
  ) &
}

scan_context() {
  echo "run_id=$RUN_ID"
  echo "root_input=$ROOT_INPUT"
  echo "root=$ROOT"
  echo "compare_input=$COMPARE_INPUT"
  echo "compare=$COMPARE"
  echo "pwd=$(pwd)"
  echo "user=$(id -un 2>/dev/null || true)"
  echo "uid_gid=$(id 2>/dev/null || true)"
  echo "date_utc=$(date -u +%Y-%m-%dT%H:%M:%SZ)"
  echo "uname=$(uname -a 2>/dev/null || true)"
  if [[ -f /etc/os-release ]]; then cat /etc/os-release; fi
  echo "--- command availability ---"
  for c in git rg tree find awk sed jq python3 realpath stat readlink du df systemctl crontab docker podman kubectl helm terraform make cargo npm pnpm yarn node python go java mvn gradle; do
    if have "$c"; then echo "$c=$(command -v "$c")"; else echo "$c=missing"; fi
  done
  echo "--- redacted env keys ---"
  env | sed -E 's/=.*$/=<redacted>/' | sort
}

path_resolution() {
  for p in "$ROOT" "$COMPARE" "$HOME/home/flexnetos/FlexNetOS" "$HOME/home/flexnetos/lifeos" "/home/flexnetos/FlexNetOS" "/home/flexnetos/lifeos"; do
    [[ -z "$p" ]] && continue
    echo "## $p"
    if [[ -e "$p" ]]; then
      stat "$p" || true
      echo "realpath=$(realpath -m "$p" 2>/dev/null || true)"
      echo "readlink=$(readlink "$p" 2>/dev/null || true)"
      echo "mountpoint=$(df -P "$p" 2>/dev/null | tail -n 1 || true)"
      if [[ -d "$p/.git" ]] || git -C "$p" rev-parse --show-toplevel >/dev/null 2>&1; then
        echo "git_root=$(git -C "$p" rev-parse --show-toplevel 2>/dev/null || true)"
      fi
    else
      echo "MISSING"
    fi
    echo
  done
}

filesystem_tree() {
  if have tree; then
    tree -a -L 6 -I '.git|node_modules|vendor|dist|build|target|.venv|venv|__pycache__|.cache|coverage|.next|.nuxt|.terraform' "$ROOT"
  else
    find "$ROOT" -maxdepth 6 -print | sed "s#^$ROOT#.#" | grep -Ev "$EXCLUDES" | sort
  fi
}

file_inventory() {
  printf 'path\tsize_bytes\tmtime_utc\text\n'
  find "$ROOT" -type f 2>/dev/null \
    | grep -Ev "$EXCLUDES" \
    | while IFS= read -r f; do
        rel="${f#$ROOT/}"
        size=$(stat -c '%s' "$f" 2>/dev/null || echo UNKNOWN)
        mt=$(date -u -d "@$(stat -c '%Y' "$f" 2>/dev/null || echo 0)" +%Y-%m-%dT%H:%M:%SZ 2>/dev/null || echo UNKNOWN)
        ext="${rel##*.}"; [[ "$ext" == "$rel" ]] && ext=""
        printf '%s\t%s\t%s\t%s\n' "$rel" "$size" "$mt" "$ext"
      done | sort
}

git_inventory() {
  if ! git rev-parse --is-inside-work-tree >/dev/null 2>&1; then
    echo "not_a_git_repo"
    return 0
  fi
  echo "--- git root ---"; git rev-parse --show-toplevel || true
  echo "--- status short ---"; git status --short --branch || true
  echo "--- remotes ---"; git remote -v | sed -E 's#(https?://)[^/@]+@#\1<redacted>@#g' || true
  echo "--- branches ---"; git branch -a --no-color || true
  echo "--- tags recent ---"; git tag --sort=-creatordate | head -n 50 || true
  echo "--- recent commits ---"; git log --date=iso-strict --pretty=format:'%h%x09%ad%x09%an%x09%s' -n 200 || true
  echo "--- name status recent ---"; git log --name-status --date=iso-strict --pretty=format:'COMMIT %h %ad %an %s' -n 80 || true
  echo "--- tracked files ---"; git ls-files || true
  echo "--- grep FlexNetOS/lifeos in git history ---"
  git log --all --grep='FlexNetOS\|flexnetos\|FLEXNETOS\|lifeos\|LifeOS\|LIFEOS\|FlexNet\|Life OS' --regexp-ignore-case --date=iso-strict --pretty=format:'%h%x09%ad%x09%an%x09%s' || true
}

manifest_inventory() {
  find "$ROOT" -type f 2>/dev/null \
    | grep -Ev "$EXCLUDES" \
    | grep -E '/(package.json|package-lock.json|pnpm-lock.yaml|yarn.lock|Cargo.toml|Cargo.lock|pyproject.toml|requirements[^/]*\.txt|Pipfile|poetry.lock|go.mod|go.sum|pom.xml|build.gradle|settings.gradle|Dockerfile|docker-compose[^/]*\.ya?ml|compose\.ya?ml|Makefile|justfile|Taskfile\.ya?ml|\.tool-versions|\.nvmrc|\.python-version|rust-toolchain[^/]*|Gemfile|Gemfile.lock|composer.json|composer.lock|mix.exs|rebar.config|deno.json|bun.lockb|helmfile\.ya?ml|Chart\.yaml|kustomization\.ya?ml|terraform\.tf|.*\.tf|\.github/workflows/.*\.ya?ml|\.gitlab-ci\.yml)$' \
    | sed "s#^$ROOT/##" | sort
}

ripgrep_or_grep() {
  local pattern="$1"
  if have rg; then
    rg -n --hidden --glob '!.git' --glob '!node_modules' --glob '!vendor' --glob '!dist' --glob '!build' --glob '!target' --glob '!.venv' --glob '!venv' --glob '!coverage' -S "$pattern" "$ROOT" || true
  else
    grep -RInE "$pattern" "$ROOT" 2>/dev/null | grep -Ev "$EXCLUDES" || true
  fi
}

imports_routes() {
  ripgrep_or_grep '(^import |^from .* import |require\(|use [a-zA-Z0-9_:]+|mod [a-zA-Z0-9_]+|package [a-zA-Z0-9_.]+|func main\(|fn main\(|class .*|def .*\(|router\.|app\.(get|post|put|delete|patch)|@(Get|Post|Put|Delete|Patch|Controller)|Route\(|routes?[:=])'
}

api_events() {
  ripgrep_or_grep '(openapi|swagger|asyncapi|webhook|callback|topic|queue|exchange|kafka|rabbit|sqs|sns|pubsub|event|producer|consumer|endpoint|graphql|grpc|protobuf|proto3|REST|OAuth|OIDC|SAML|api[_-]?key|bearer)'
}

data_schemas() {
  ripgrep_or_grep '(CREATE TABLE|ALTER TABLE|CREATE INDEX|sequelize|typeorm|prisma|migrate|migration|schema|table_name|db_table|SQLAlchemy|Django model|ActiveRecord|knex|liquibase|flyway|dbt|warehouse|lake|parquet|cdc|replication)'
}

config_env_redacted() {
  echo "--- config-like files ---"
  find "$ROOT" -type f 2>/dev/null | grep -Ev "$EXCLUDES" | grep -E '(\.env(\..*)?$|config|settings|\.ya?ml$|\.toml$|\.ini$|\.conf$|\.properties$|secrets?|cert|key|pem|crt)' | sed "s#^$ROOT/##" | sort
  echo "--- env var references / secret key names redacted ---"
  ripgrep_or_grep '([A-Z][A-Z0-9_]{2,}|process\.env|os\.environ|ENV\[|getenv|dotenv|secret|password|passwd|token|api[_-]?key|private[_-]?key|certificate|BEGIN [A-Z ]*PRIVATE KEY)' | sed -E "s/(=|:)[[:space:]]*[^[:space:]]+/<redacted>/g"
}

infra_iac() {
  ripgrep_or_grep '(terraform|provider "|resource "|module "|kubernetes|helm|ingress|serviceAccount|Deployment|StatefulSet|DaemonSet|ConfigMap|Secret|Dockerfile|docker-compose|FROM |EXPOSE |VOLUME |HEALTHCHECK|systemd|\.service|cron|crontab|nginx|apache|load balancer|vpc|subnet|security_group|firewall|dns|certificate|tls)'
}

observability_tests() {
  ripgrep_or_grep '(logger|logging|log\.|trace|tracing|span|metric|prometheus|opentelemetry|otel|datadog|newrelic|sentry|grafana|alert|SLO|SLA|runbook|health(check)?|readiness|liveness|test\(|describe\(|it\(|pytest|unittest|junit|coverage|benchmark|perf)'
}

flexnetos_lifeos() {
  echo "--- path resolution ---"
  path_resolution
  echo "--- content references ---"
  ripgrep_or_grep '(FlexNetOS|flexnetos|FLEXNETOS|lifeos|LifeOS|LIFEOS|FlexNet|Life OS)'
  echo "--- compare tree if compare path exists ---"
  if [[ -n "$COMPARE" && -d "$COMPARE" ]]; then
    echo "compare=$COMPARE"
    echo "du root:"; du -sh "$ROOT" 2>/dev/null || true
    echo "du compare:"; du -sh "$COMPARE" 2>/dev/null || true
    echo "top-level root:"; find "$ROOT" -maxdepth 2 -mindepth 1 -print 2>/dev/null | sed "s#^$ROOT#ROOT#" | sort | head -n 300
    echo "top-level compare:"; find "$COMPARE" -maxdepth 2 -mindepth 1 -print 2>/dev/null | sed "s#^$COMPARE#COMPARE#" | sort | head -n 300
    echo "file list overlap sample:"
    tmp1="$OUT/root-files.txt"; tmp2="$OUT/compare-files.txt"
    find "$ROOT" -type f 2>/dev/null | grep -Ev "$EXCLUDES" | sed "s#^$ROOT/##" | sort > "$tmp1"
    find "$COMPARE" -type f 2>/dev/null | grep -Ev "$EXCLUDES" | sed "s#^$COMPARE/##" | sort > "$tmp2"
    comm -12 "$tmp1" "$tmp2" | head -n 500 || true
  else
    echo "compare_missing_or_not_directory=$COMPARE"
  fi
}

printf 'job\texit_code\tfinished_at_utc\n' >"$OUT/jobs.tsv"

run_bg scan_context scan_context
run_bg path_resolution path_resolution
run_bg filesystem_tree filesystem_tree
run_bg file_inventory file_inventory
run_bg git_inventory git_inventory
run_bg manifest_inventory manifest_inventory
run_bg imports_routes imports_routes
run_bg api_events api_events
run_bg data_schemas data_schemas
run_bg config_env_redacted config_env_redacted
run_bg infra_iac infra_iac
run_bg observability_tests observability_tests
run_bg flexnetos_lifeos flexnetos_lifeos

wait

(
  cd "$OUT"
  sha256sum *.out *.err 2>/dev/null | sort > checksums.sha256 || true
)

cat > "$ROOT/migration-artifacts/_meta/latest-raw-scan.txt" <<EOF
$OUT
EOF

cat >> "$ROOT/migration-artifacts/_meta/scan-runs.jsonl" <<EOF
{"run_id":"$RUN_ID","root":"$ROOT","compare":"$COMPARE","raw_dir":"$OUT","finished_at_utc":"$(date -u +%Y-%m-%dT%H:%M:%SZ)"}
EOF

echo "$OUT"
