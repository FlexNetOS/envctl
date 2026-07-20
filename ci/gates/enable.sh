#!/usr/bin/env bash
# ci/gates/enable.sh — the Phase-6 / OI-1 enable gate (post-Phase-6 inverted form).
#
# Materialized from docs/ops/02-envctl-component.md §4.3. PRE-Phase-6 this gate asserted the manifest
# kept `enable = false` while `crates/secretd/src/main.rs` was the `todo!("secretd server bring-up")`
# scaffold (so an auto-enabled unit could not panic-loop into a false "vault is up"). Phase 6 has
# landed, so the gate now asserts the INVERSE invariant, fail-closed:
#   (1) main.rs is no longer the scaffold — the daemon actually serves, AND
#   (2) IF the unit ships enabled, its `verify` hook's `secretd --self-check` surface must exist in
#       both the manifest (the hook calls it) and the source (the subcommand is defined). An enabled
#       unit whose verify references a missing subcommand would wire a verify that can never pass —
#       and a bare `secretd` with no `--self-check` would hang the hook by serving forever.
# Run from the repo root: `bash ci/gates/enable.sh`.
set -euo pipefail
export PATH=/usr/bin:/bin
fail() { echo "ENABLE GATE FAIL: $*" >&2; exit 1; }

ROOT="${ENVCTL_GATE_ROOT:-$(git rev-parse --show-toplevel)}"
MAIN="$ROOT/crates/secretd/src/main.rs"
MANIFEST="$ROOT/manifest/env-ctl.toml"
SQLD_MANIFEST="$ROOT/manifest/sqld.toml"
STORE="$ROOT/crates/secrets-store-libsql/src/store.rs"
SECRETCTL_CLI="$ROOT/crates/secretctl/src/cli.rs"
SECRETCTL_AUTH="$ROOT/crates/secretctl/src/sqld_auth.rs"
SQLD_CARGO_HELPER="$ROOT/assets/scripts/envctl-sqld-hermetic-cargo.sh"
LLVM_MANIFEST="$ROOT/manifest/components.d/epic-h-toolchains.toml"
LLVM_LIFECYCLE="$ROOT/assets/scripts/envctl-llvm-clang-lifecycle.sh"
LLVM_FIXTURE="$ROOT/scripts/tests/test-llvm-clang-component.sh"

[ -f "$MAIN" ]     || fail "missing $MAIN"
[ -f "$MANIFEST" ] || fail "missing $MANIFEST"
[ -f "$SQLD_MANIFEST" ] || fail "missing $SQLD_MANIFEST"
[ -f "$STORE" ]    || fail "missing $STORE"
[ -f "$SECRETCTL_CLI" ] || fail "missing $SECRETCTL_CLI"
[ -f "$SECRETCTL_AUTH" ] || fail "missing $SECRETCTL_AUTH"
[ -f "$SQLD_CARGO_HELPER" ] || fail "missing $SQLD_CARGO_HELPER"
[ -f "$LLVM_MANIFEST" ] || fail "missing $LLVM_MANIFEST"
[ -f "$LLVM_LIFECYCLE" ] || fail "missing $LLVM_LIFECYCLE"
[ -f "$LLVM_FIXTURE" ] || fail "missing $LLVM_FIXTURE"

# (1) Phase 6 must really be done: main.rs is no longer the `todo!()` scaffold.
if grep -q 'todo!("secretd server bring-up' "$MAIN"; then
  fail "secretd main.rs is still the Phase-6 todo!() scaffold — the manifest must keep enable=false"
fi

# (2) If the systemd unit ships enabled, the verify hook's `secretd --self-check` surface MUST exist.
if grep -Eq '^[[:space:]]*enable[[:space:]]*=[[:space:]]*true' "$MANIFEST"; then
  grep -q -- 'secretd --self-check' "$MANIFEST" \
    || fail "manifest enables the unit but its verify hook does not invoke 'secretd --self-check'"
  grep -q -- 'self-check' "$MAIN" \
    || fail "manifest enables the unit but secretd defines no --self-check subcommand"
fi

# The unit is declarative across workstations: systemd does not expand shell
# variables in ExecStart, so wiring.rs must render the explicit token. A
# historical `%h/Desktop/meta` literal silently targeted a retired checkout.
if grep -Fq '%h/Desktop/meta' "$MANIFEST"; then
  fail "secretd user unit contains the retired %h/Desktop/meta path"
fi
grep -Fq 'ExecStart="${META_ROOT}/usr/libexec/envctl/secrets/bin/secretd"' "$MANIFEST" \
  || fail "secretd user unit must use the engine-rendered META_ROOT token"

# The enabled production daemon is inseparable from the envctl-owned sqld. `Requires` starts it;
# `BindsTo` + `After` also stop secretd when sqld becomes inactive. A weak Wants-only relationship
# previously allowed an open-auth or absent sqld to coexist with an apparently active daemon.
for dependency in \
  'Requires=sqld.service' \
  'BindsTo=sqld.service' \
  'After=sqld.service'
do
  [ "$(grep -Fxc "$dependency" "$MANIFEST")" -eq 1 ] \
    || fail "secretd unit must contain exactly one '$dependency'"
done
if grep -Fqx 'Wants=sqld.service' "$MANIFEST"; then
  fail "secretd unit must not weaken sqld ownership to Wants="
fi

# READY=1 is the readiness barrier. The unit forces the durable backend + owned loopback URL, and
# secretd cannot notify systemd until build_engine returns. The libSQL builder authenticates while
# opening the remote connection and provisions the schema before returning a ready Store.
for contract in \
  'Type=notify' \
  'Environment="SECRETD_STORE_BACKEND=libsql"' \
  'Environment="SECRETD_LIBSQL_URL=http://127.0.0.1:8080"' \
  'Environment="SECRETD_LIBSQL_AUTH_TOKEN_FILE=${META_ROOT}/.config/sqld/client.jwt"' \
  'ConditionPathExists="${META_ROOT}/.config/sqld/auth-jwt-key.pem"' \
  'ConditionPathExists="${META_ROOT}/.config/sqld/client.jwt"'
do
  grep -Fqx "$contract" "$MANIFEST" \
    || fail "secretd readiness contract is missing '$contract'"
done

build_line="$(grep -Fn 'let (engine, profile_b_seams) = build_engine' "$MAIN" | head -n 1 | cut -d: -f1)"
ready_line="$(grep -Fn 'sd_notify::notify(false, &[sd_notify::NotifyState::Ready])' "$MAIN" | head -n 1 | cut -d: -f1)"
[ -n "$build_line" ] || fail "secretd no longer builds its configured engine before serving"
[ -n "$ready_line" ] || fail "Type=notify unit has no READY=1 notification"
[ "$build_line" -lt "$ready_line" ] \
  || fail "secretd can notify READY before the configured store has opened"
grep -Fq 'LibSqlStoreBuilder::new(' "$MAIN" \
  || fail "libSQL backend no longer opens the authenticated remote store"
grep -Fq 'c.execute_batch(schema::DDL)' "$STORE" \
  || fail "libSQL builder no longer provisions the schema before becoming ready"

# `After=sqld.service` is only a real readiness barrier when sqld's start job includes the bounded,
# non-ignored ExecStartPost proof. systemd accounts ExecStartPost completion in ordering. The
# pure-Rust helper must self-hash before credential access, bind the proof to $MAINPID + the envctl
# payload/listener, reject unauthenticated SQL with 401, and read the bearer only by safe file path.
sqld_barrier='ExecStartPost="${META_ROOT}/usr/libexec/envctl/sqld/bin/current/secretctl" internal-sqld-readiness-probe --pid "${MAINPID}" --expected-executable "${META_ROOT}/.toolchains/sqld/bin/sqld" --port 8080 --client-token "${META_ROOT}/.config/sqld/client.jwt" --helper-digest "${META_ROOT}/usr/libexec/envctl/sqld/bin/current/secretctl.sha256" --timeout-seconds 20'
grep -Fqx 'Type=exec' "$SQLD_MANIFEST" \
  || fail "sqld service must use Type=exec before its readiness barrier"
[ "$(grep -Fxc "$sqld_barrier" "$SQLD_MANIFEST")" -eq 1 ] \
  || fail "sqld service must contain exactly one non-ignored MainPID/auth ExecStartPost barrier"
if grep -Eq '^Exec(Start|StartPost)=.*sha256sum' "$SQLD_MANIFEST"; then
  fail "sqld runtime unit must not depend on a system-depth checksum executable"
fi
grep -Fqx 'TimeoutStartSec=30' "$SQLD_MANIFEST" \
  || fail "sqld readiness barrier must have a bounded systemd start timeout"
[ "$(grep -Fxc 'LimitCORE=0' "$SQLD_MANIFEST")" -eq 1 ] \
  || fail "sqld service must suppress core dumps that could retain bearer/runtime memory"
grep -Fq 'InternalSqldReadinessProbe' "$SECRETCTL_CLI" \
  || fail "secretctl no longer exposes the private sqld readiness helper"
grep -Fq 'helper_digest: std::path::PathBuf' "$SECRETCTL_CLI" \
  || fail "sqld readiness CLI no longer requires the component-owned helper digest"
self_digest_line="$(grep -Fn 'verify_self_digest(helper_digest)?;' "$SECRETCTL_AUTH" | head -n 1 | cut -d: -f1)"
token_read_line="$(grep -Fn 'let token = read_safe_token(client_token)?;' "$SECRETCTL_AUTH" | head -n 1 | cut -d: -f1)"
[ -n "$self_digest_line" ] && [ -n "$token_read_line" ] \
  || fail "sqld readiness helper lost its self-digest/token ordering proof"
[ "$self_digest_line" -lt "$token_read_line" ] \
  || fail "sqld readiness helper can access the token before proving its own open executable"
payload_identity_line="$(grep -Fn 'let executable_identity = loop {' "$SECRETCTL_AUTH" | head -n 1 | cut -d: -f1)"
payload_digest_line="$(grep -Fn 'match verify_process_executable(pid, expected_executable, expected_sha256, expected_mode)? {' "$SECRETCTL_AUTH" | head -n 1 | cut -d: -f1)"
[ -n "$payload_identity_line" ] && [ -n "$payload_digest_line" ] \
  && [ "$payload_identity_line" -lt "$payload_digest_line" ] \
  && [ "$payload_digest_line" -lt "$token_read_line" ] \
  || fail "sqld readiness helper can access the token before hashing the running MainPID bytes"
for proof in \
  'pid_owns_loopback_listener(pid, port)' \
  'if unauth.status != 401' \
  'read_safe_token(client_token)' \
  'open_running_executable()' \
  'open_safe_0600(record_path, "sqld helper-digest")' \
  'running sqld payload bytes differ from every pinned SHA-256' \
  'installed sqld readiness helper differs from the fresh current-source build'
do
  grep -Fq "$proof" "$SECRETCTL_AUTH" \
    || fail "sqld readiness helper lost required proof '$proof'"
done
for ownership in \
  'helper_owned_or_absent' \
  'internal-sqld-self-digest' \
  'internal-sqld-verify-current-helper' \
  'sqld-verify.XXXXXX' \
  'sqld-install.XXXXXX' \
  'reject_external_build_config' \
  'CARGO_TARGET_*_LINKER' \
  'load_hermetic_cargo_helpers' \
  'stage_hermetic_cargo_home' \
  'run_hermetic_cargo_build' \
  'collect_used_crate_archives' \
  '--toolchain-root "$clang_resource_root"' \
  '--toolchain-root "$rust_toolchain_lib_root"' \
  '--toolchain-root "$llvm_generation_root"' \
  '--toolchain-root "$private_cargo_home/registry/index/$registry_id"' \
  '"${crate_archive_args[@]}"' \
  'secretctl.source.sha256' \
  'refresh_on_verify_failure = true' \
  'revalidate_active_service' \
  'rollback_runtime_activation' \
  'restore_service_after_rollback' \
  'sqld_remove_mktemp' \
  'restore_service_state'
do
  grep -Fq -- "$ownership" "$SQLD_MANIFEST" \
    || fail "sqld helper deployment lost owned-byte rule '$ownership'"
done
for build_rule in \
  '/usr/bin/env -i' \
  '"$cargo_bin" build --quiet --frozen --locked --offline --release' \
  'PATH="$hermetic_path" CARGO_HOME="$private_cargo_home"' \
  'CFLAGS="$hermetic_cflags" CXXFLAGS="$hermetic_cflags"' \
  'TZ=UTC SOURCE_DATE_EPOCH=1 ZERO_AR_DATE=1' \
  '--remap-path-prefix=$build_workspace=/envctl-build' \
  '-ffile-prefix-map=$build_workspace=/envctl-build' \
  'link-arg=--no-default-config' \
  'link-arg=-Wl,--build-id=sha1' \
  'validate_toolchain_resource_tree' \
  'registry/cache/$registry_id' \
  'registry/index/$registry_id' \
  'install -m444 "$archive" "$staged"' \
  'install -m444 "$index_source" "$index_staged"'
do
  grep -Fq -- "$build_rule" "$SQLD_CARGO_HELPER" \
    || fail "sqld hermetic Cargo helper lost rule '$build_rule'"
done
if grep -Fq -- '--config "$source_root/.cargo/' "$SQLD_MANIFEST"; then
  fail "sqld helper must not execute checked-in Cargo config during its GNU build"
fi
[ "$(grep -Fc 'long = "crate-archive"' "$SECRETCTL_CLI")" -eq 2 ] \
  || fail "source identity must require used crate archives for write and verify"
[ "$(grep -Fc 'long = "toolchain-root"' "$SECRETCTL_CLI")" -eq 2 ] \
  || fail "source identity must require compiler/index data roots for write and verify"
[ "$(grep -Fc 'internal-sqld-verify-current-helper' "$SQLD_MANIFEST")" -eq 3 ] \
  || fail "install, fix, and verify must all use fresh-helper byte equality"
[ "$(grep -Fc 'sqld_mv -T --exchange --no-copy -- "$stage_dir" "$probe_dir"' "$SQLD_MANIFEST")" -eq 2 ] \
  || fail "install and fix must atomically exchange each complete helper generation"
if grep -Fq 'mv -f "$staged_probe" "$probe"' "$SQLD_MANIFEST" \
  || grep -Fq 'mv -f "$staged_digest" "$probe_digest"' "$SQLD_MANIFEST"; then
  fail "sqld helper generation regressed to independent leaf replacement"
fi
grep -Fq 'rustix::fs::RenameFlags::EXCHANGE' "$SECRETCTL_AUTH" \
  || fail "sqld helper generation no longer uses atomic rename exchange"
grep -Fq 'exchange_generation_dirs(&parent.directory, staged_dir, current_dir)' "$SECRETCTL_AUTH" \
  || fail "sqld helper generation exchange is not anchored to the validated parent dirfd"
grep -Fq 'for relative in [".cargo/config.toml", ".cargo/config"]' "$SECRETCTL_AUTH" \
  || fail "accepted checked-in Cargo configs are not part of the helper source identity"
for symlink_rule in \
  'envctl-sqld-toolchain-roots-v2' \
  'validate_owned_contained_toolchain_symlink' \
  'sqld helper toolchain-root symlink escapes its root' \
  'sqld helper toolchain-root symlink target must be relative'
do
  grep -Fq "$symlink_rule" "$SECRETCTL_AUTH" \
    || fail "sqld META compiler-input identity lost symlink rule '$symlink_rule'"
done

retirement_prepare_line="$(grep -Fn 'front_retired="$(sqld_remove_mktemp ' "$SQLD_MANIFEST" | tail -n 1 | cut -d: -f1)"
remove_query_line="$(grep -Fn 'load_state="$(systemctl --user show --property=LoadState --value sqld.service)"' "$SQLD_MANIFEST" | tail -n 1 | cut -d: -f1)"
[ -n "$retirement_prepare_line" ] && [ -n "$remove_query_line" ] \
  && [ "$retirement_prepare_line" -lt "$remove_query_line" ] \
  || fail "sqld Remove must prepare retirement rollback paths before service mutation"
grep -Fq 'if ! systemctl --user disable --now sqld.service; then' "$SQLD_MANIFEST" \
  || fail "sqld Remove must explicitly restore a partially failed stop/disable"

# External lifecycle scripts are executable trust-boundary inputs. Their exact bytes are bound by
# a digest carried inside the lock-covered TOML hooks; changing an asset without updating/relocking
# its manifest must fail this gate.
sqld_helper_sha256="$(sha256sum "$SQLD_CARGO_HELPER" | awk '{print $1}')"
[ "$(grep -Fc "$sqld_helper_sha256" "$SQLD_MANIFEST")" -eq 3 ] \
  || fail "sqld hermetic-Cargo helper bytes are not SHA-256-bound in all three build hooks"
llvm_lifecycle_sha256="$(sha256sum "$LLVM_LIFECYCLE" | awk '{print $1}')"
[ "$(grep -Fc "$llvm_lifecycle_sha256" "$LLVM_MANIFEST")" -eq 5 ] \
  || fail "LLVM lifecycle bytes are not SHA-256-bound in all five component hooks"

grep -Fq 'requires = ["rustup", "llvm-clang"]' "$SQLD_MANIFEST" \
  || fail "sqld must require the exact META-owned Rust and LLVM toolchains"
for llvm_rule in \
  'llvmorg-21.1.8' \
  'b3b7f2801d15d50736acea3c73982994d025b01c2f035b91ae3b49d1b575732b' \
  '65ce0b329514e5643407db2d02a5bd34bf33d159055dafa82825c8385bd01993' \
  '8103c17f58639c829047e9166a65f0ba68d94c9cd1f55ae0dc6db526187af142' \
  '48fc701c3989881594c5ec7c8a8c354ccbe36c76fca3fad7868d5edc5aea407a' \
  '89df891585a701ce617b1d91ec1757db9f00337b4a70e7bd4dd28cb68e1dacac' \
  'c8b5bc8d10d092c8d3edb996ffcf3dd4e78992b8a0fc9f88f910a5b3cc0c1383' \
  '/usr/bin/env -i HOME="$download_root" PATH=/usr/bin:/bin' \
  "/usr/bin/curl --disable --proto '=https' --tlsv1.2" \
  '/usr/bin/tar -xJf' \
  'llvm_require_atomic_mv' \
  'llvm_rollback_frontdoors_and_activation' \
  'refusing to remove LLVM while the sqld helper provenance chain is installed'
do
  grep -Fq "$llvm_rule" "$LLVM_LIFECYCLE" \
    || fail "LLVM lifecycle lost rule '$llvm_rule'"
done
llvm_component="$(sed -n '/^id = "llvm-clang"$/,/^id = "libgccjit"$/p' "$LLVM_MANIFEST")"
[ "$(grep -Fc 'export PATH=/usr/bin:/bin' <<<"$llvm_component")" -eq 5 ] \
  || fail "all LLVM hooks, including Detect, must pin the host utility PATH before preflight"
if grep -Fq 'releases/latest' "$LLVM_LIFECYCLE"; then
  fail "LLVM lifecycle must not resolve a floating release"
fi
bash "$LLVM_FIXTURE" >/dev/null \
  || fail "LLVM lifecycle transaction/no-source fixture failed"

echo "ENABLE GATE PASS"
