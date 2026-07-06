# envctl Environment Variable Inventory

This report lists the environment-variable rows envctl currently stores in the catalog, along with producer, scope, sensitivity, and resolved value shape.

## Summary

- total env vars: `106`
- repo root: `/home/flexnetos/FlexNetOS/src/envctl`
- manifest dir: `/home/flexnetos/FlexNetOS/src/envctl/manifest`

## Facets

### Scopes

- `layout`: `45`
- `schema`: `61`

### Producers

- `layout`: `45`
- `secrets_env_schema`: `61`

### Sensitivity

- `sensitive`: `28`
- `non_sensitive`: `78`

## Rows

| var | producer | scope | source | sensitive | value source | current value |
| --- | --- | --- | --- | --- | --- | --- |
| `ANTHROPIC_API_KEY` | `secrets_env_schema` | `schema` | `crates/secretd/src/conv.rs` | `yes` | `missing` | `unset` |
| `ANTHROPIC_API_KEY` | `secrets_env_schema` | `schema` | `crates/secrets-engine/src/inject.rs` | `yes` | `missing` | `unset` |
| `ANTHROPIC_BASE_URL` | `secrets_env_schema` | `schema` | `crates/secretd/src/conv.rs` | `no` | `missing` | `unset` |
| `ANTHROPIC_BASE_URL` | `secrets_env_schema` | `schema` | `crates/secrets-engine/src/inject.rs` | `no` | `missing` | `unset` |
| `ENVCTL_BIN_DIR` | `layout` | `layout` | `crates/engine/src/layout.rs` | `no` | `effective_value` | `/home/flexnetos/FlexNetOS/usr/bin` |
| `ENVCTL_CACHE_DIR` | `layout` | `layout` | `crates/engine/src/layout.rs` | `no` | `effective_value` | `/home/flexnetos/FlexNetOS/var/cache/envctl` |
| `ENVCTL_ENVCTL_LIB_DIR` | `layout` | `layout` | `crates/engine/src/layout.rs` | `no` | `effective_value` | `/home/flexnetos/FlexNetOS/usr/lib/envctl` |
| `ENVCTL_ENVCTL_SHARE_DIR` | `layout` | `layout` | `crates/engine/src/layout.rs` | `no` | `effective_value` | `/home/flexnetos/FlexNetOS/usr/share/envctl` |
| `ENVCTL_ETC` | `layout` | `layout` | `crates/engine/src/layout.rs` | `no` | `effective_value` | `/home/flexnetos/FlexNetOS/etc` |
| `ENVCTL_ETC_DIR` | `layout` | `layout` | `crates/engine/src/layout.rs` | `no` | `effective_value` | `/home/flexnetos/FlexNetOS/etc/envctl` |
| `ENVCTL_GITHUB_API_BASE` | `secrets_env_schema` | `schema` | `crates/secretd/src/grpc.rs` | `no` | `missing` | `unset` |
| `ENVCTL_GITHUB_API_BASE` | `secrets_env_schema` | `schema` | `crates/secrets-engine/src/mint_github.rs` | `no` | `missing` | `unset` |
| `ENVCTL_GITHUB_APP_SECRET` | `secrets_env_schema` | `schema` | `crates/secretd/src/grpc.rs` | `yes` | `missing` | `unset` |
| `ENVCTL_LEGACY_TOOLCHAINS` | `layout` | `layout` | `crates/engine/src/layout.rs` | `no` | `effective_value` | `/home/flexnetos/FlexNetOS/.toolchains` |
| `ENVCTL_LIB_DIR` | `layout` | `layout` | `crates/engine/src/layout.rs` | `no` | `effective_value` | `/home/flexnetos/FlexNetOS/usr/lib` |
| `ENVCTL_LOCAL` | `layout` | `layout` | `crates/engine/src/layout.rs` | `no` | `effective_value` | `/home/flexnetos/FlexNetOS/.local` |
| `ENVCTL_LOCAL_BIN` | `layout` | `layout` | `crates/engine/src/layout.rs` | `no` | `effective_value` | `/home/flexnetos/FlexNetOS/.local/bin` |
| `ENVCTL_META_ROOT` | `layout` | `layout` | `crates/engine/src/layout.rs` | `no` | `effective_value` | `/home/flexnetos/FlexNetOS` |
| `ENVCTL_OPT_DIR` | `layout` | `layout` | `crates/engine/src/layout.rs` | `no` | `effective_value` | `/home/flexnetos/FlexNetOS/opt` |
| `ENVCTL_REPO_STORE` | `layout` | `layout` | `crates/engine/src/layout.rs` | `no` | `effective_value` | `/home/flexnetos/FlexNetOS/var/lib/envctl/repos` |
| `ENVCTL_RUN_DIR` | `layout` | `layout` | `crates/engine/src/layout.rs` | `no` | `effective_value` | `/home/flexnetos/FlexNetOS/run` |
| `ENVCTL_SECRETS_BIN_DIR` | `layout` | `layout` | `crates/engine/src/layout.rs` | `no` | `effective_value` | `/home/flexnetos/FlexNetOS/usr/libexec/envctl/secrets/bin` |
| `ENVCTL_SEED_API` | `secrets_env_schema` | `schema` | `crates/secrets-engine/src/seam.rs` | `no` | `missing` | `unset` |
| `ENVCTL_SEED_CA` | `layout` | `layout` | `crates/engine/src/layout.rs` | `no` | `effective_value` | `/home/flexnetos/FlexNetOS/usr/share/envctl/secrets/ca/cognitum-ca.crt` |
| `ENVCTL_SEED_CA` | `secrets_env_schema` | `schema` | `crates/secrets-engine/src/seam.rs` | `no` | `missing` | `unset` |
| `ENVCTL_SEED_KEK_CONTEXT` | `secrets_env_schema` | `schema` | `crates/secrets-engine/src/seam.rs` | `no` | `missing` | `unset` |
| `ENVCTL_SEED_PUBKEY` | `secrets_env_schema` | `schema` | `crates/secrets-engine/src/broker/gate.rs` | `no` | `missing` | `unset` |
| `ENVCTL_SEED_PUBKEY` | `secrets_env_schema` | `schema` | `crates/secrets-engine/src/lib.rs` | `no` | `missing` | `unset` |
| `ENVCTL_SEED_PUBKEY` | `secrets_env_schema` | `schema` | `crates/secrets-engine/src/seam.rs` | `no` | `missing` | `unset` |
| `ENVCTL_SEED_TOKEN` | `secrets_env_schema` | `schema` | `crates/secrets-engine/src/seam.rs` | `yes` | `missing` | `unset` |
| `ENVCTL_SEED_TOKEN_FILE` | `secrets_env_schema` | `schema` | `crates/secrets-engine/src/seam.rs` | `yes` | `missing` | `unset` |
| `ENVCTL_SHARE_DIR` | `layout` | `layout` | `crates/engine/src/layout.rs` | `no` | `effective_value` | `/home/flexnetos/FlexNetOS/usr/share` |
| `ENVCTL_STATE_DIR` | `layout` | `layout` | `crates/engine/src/layout.rs` | `no` | `effective_value` | `/home/flexnetos/FlexNetOS/var/lib/envctl` |
| `ENVCTL_TMP_DIR` | `layout` | `layout` | `crates/engine/src/layout.rs` | `no` | `effective_value` | `/home/flexnetos/FlexNetOS/var/tmp` |
| `ENVCTL_USR` | `layout` | `layout` | `crates/engine/src/layout.rs` | `no` | `effective_value` | `/home/flexnetos/FlexNetOS/usr` |
| `ENVCTL_USR_BIN` | `layout` | `layout` | `crates/engine/src/layout.rs` | `no` | `effective_value` | `/home/flexnetos/FlexNetOS/usr/bin` |
| `ENVCTL_USR_GAMES` | `layout` | `layout` | `crates/engine/src/layout.rs` | `no` | `effective_value` | `/home/flexnetos/FlexNetOS/usr/games` |
| `ENVCTL_USR_INCLUDE` | `layout` | `layout` | `crates/engine/src/layout.rs` | `no` | `effective_value` | `/home/flexnetos/FlexNetOS/usr/include` |
| `ENVCTL_USR_LIB` | `layout` | `layout` | `crates/engine/src/layout.rs` | `no` | `effective_value` | `/home/flexnetos/FlexNetOS/usr/lib` |
| `ENVCTL_USR_LIB64` | `layout` | `layout` | `crates/engine/src/layout.rs` | `no` | `effective_value` | `/home/flexnetos/FlexNetOS/usr/lib64` |
| `ENVCTL_USR_LIBEXEC` | `layout` | `layout` | `crates/engine/src/layout.rs` | `no` | `effective_value` | `/home/flexnetos/FlexNetOS/usr/libexec` |
| `ENVCTL_USR_LOCAL` | `layout` | `layout` | `crates/engine/src/layout.rs` | `no` | `effective_value` | `/home/flexnetos/FlexNetOS/usr/local` |
| `ENVCTL_USR_LOCAL_BIN` | `layout` | `layout` | `crates/engine/src/layout.rs` | `no` | `effective_value` | `/home/flexnetos/FlexNetOS/usr/local/bin` |
| `ENVCTL_USR_LOCAL_INCLUDE` | `layout` | `layout` | `crates/engine/src/layout.rs` | `no` | `effective_value` | `/home/flexnetos/FlexNetOS/usr/local/include` |
| `ENVCTL_USR_LOCAL_LIB` | `layout` | `layout` | `crates/engine/src/layout.rs` | `no` | `effective_value` | `/home/flexnetos/FlexNetOS/usr/local/lib` |
| `ENVCTL_USR_LOCAL_LIB64` | `layout` | `layout` | `crates/engine/src/layout.rs` | `no` | `effective_value` | `/home/flexnetos/FlexNetOS/usr/local/lib64` |
| `ENVCTL_USR_LOCAL_SBIN` | `layout` | `layout` | `crates/engine/src/layout.rs` | `no` | `effective_value` | `/home/flexnetos/FlexNetOS/usr/local/sbin` |
| `ENVCTL_USR_LOCAL_SHARE` | `layout` | `layout` | `crates/engine/src/layout.rs` | `no` | `effective_value` | `/home/flexnetos/FlexNetOS/usr/local/share` |
| `ENVCTL_USR_SBIN` | `layout` | `layout` | `crates/engine/src/layout.rs` | `no` | `effective_value` | `/home/flexnetos/FlexNetOS/usr/sbin` |
| `ENVCTL_USR_SHARE` | `layout` | `layout` | `crates/engine/src/layout.rs` | `no` | `effective_value` | `/home/flexnetos/FlexNetOS/usr/share` |
| `ENVCTL_USR_SHARE_MAN` | `layout` | `layout` | `crates/engine/src/layout.rs` | `no` | `effective_value` | `/home/flexnetos/FlexNetOS/usr/share/man` |
| `ENVCTL_USR_SRC` | `layout` | `layout` | `crates/engine/src/layout.rs` | `no` | `effective_value` | `/home/flexnetos/FlexNetOS/usr/src` |
| `ENVCTL_VAR` | `layout` | `layout` | `crates/engine/src/layout.rs` | `no` | `effective_value` | `/home/flexnetos/FlexNetOS/var` |
| `ENVCTL_VAR_CACHE` | `layout` | `layout` | `crates/engine/src/layout.rs` | `no` | `effective_value` | `/home/flexnetos/FlexNetOS/var/cache` |
| `ENVCTL_VAR_LIB` | `layout` | `layout` | `crates/engine/src/layout.rs` | `no` | `effective_value` | `/home/flexnetos/FlexNetOS/var/lib` |
| `ENVCTL_VAR_LOG` | `layout` | `layout` | `crates/engine/src/layout.rs` | `no` | `effective_value` | `/home/flexnetos/FlexNetOS/var/log` |
| `ENVCTL_XDG_CACHE_HOME` | `layout` | `layout` | `crates/engine/src/layout.rs` | `no` | `effective_value` | `/home/flexnetos/FlexNetOS/.cache` |
| `ENVCTL_XDG_CONFIG_HOME` | `layout` | `layout` | `crates/engine/src/layout.rs` | `no` | `effective_value` | `/home/flexnetos/FlexNetOS/.config` |
| `ENVCTL_XDG_DATA_HOME` | `layout` | `layout` | `crates/engine/src/layout.rs` | `no` | `effective_value` | `/home/flexnetos/FlexNetOS/.local/share` |
| `ENVCTL_XDG_STATE_HOME` | `layout` | `layout` | `crates/engine/src/layout.rs` | `no` | `effective_value` | `/home/flexnetos/FlexNetOS/.local/state` |
| `ENV_BACKEND` | `secrets_env_schema` | `schema` | `crates/secretd/src/config.rs` | `no` | `missing` | `unset` |
| `ENV_CONFIG` | `secrets_env_schema` | `schema` | `crates/secretd/src/config.rs` | `no` | `missing` | `unset` |
| `ENV_TOKEN` | `secrets_env_schema` | `schema` | `crates/secretd/src/config.rs` | `yes` | `missing` | `unset` |
| `ENV_TOKEN_FILE` | `secrets_env_schema` | `schema` | `crates/secretd/src/config.rs` | `yes` | `missing` | `unset` |
| `ENV_URL` | `secrets_env_schema` | `schema` | `crates/secretd/src/config.rs` | `no` | `missing` | `unset` |
| `GH_TOKEN` | `secrets_env_schema` | `schema` | `crates/secrets-engine/src/inject.rs` | `yes` | `missing` | `unset` |
| `GH_TOKEN` | `secrets_env_schema` | `schema` | `crates/secrets-engine/src/lib.rs` | `yes` | `missing` | `unset` |
| `GITHUB_API_URL` | `secrets_env_schema` | `schema` | `crates/secrets-engine/src/inject.rs` | `no` | `missing` | `unset` |
| `GITHUB_TOKEN` | `secrets_env_schema` | `schema` | `crates/secrets-engine/src/inject.rs` | `yes` | `missing` | `unset` |
| `GITHUB_TOKEN` | `secrets_env_schema` | `schema` | `crates/secrets-engine/src/lib.rs` | `yes` | `missing` | `unset` |
| `HOME` | `secrets_env_schema` | `schema` | `crates/secrets-engine/src/paths.rs` | `no` | `missing` | `unset` |
| `HOME` | `secrets_env_schema` | `schema` | `crates/secrets-engine/src/seam.rs` | `no` | `missing` | `unset` |
| `HTTPS_PROXY` | `secrets_env_schema` | `schema` | `crates/secretd/src/proxy.rs` | `no` | `missing` | `unset` |
| `HTTPS_PROXY` | `secrets_env_schema` | `schema` | `crates/secrets-engine/src/inject.rs` | `no` | `missing` | `unset` |
| `HTTPS_PROXY` | `secrets_env_schema` | `schema` | `crates/secrets-engine/src/lib.rs` | `no` | `missing` | `unset` |
| `LLM_API_KEY` | `secrets_env_schema` | `schema` | `crates/secrets-engine/src/inject.rs` | `yes` | `missing` | `unset` |
| `META_ROOT` | `secrets_env_schema` | `schema` | `crates/secretd/src/config.rs` | `no` | `missing` | `unset` |
| `META_ROOT` | `secrets_env_schema` | `schema` | `crates/secretd/src/edge/tls.rs` | `no` | `missing` | `unset` |
| `META_ROOT` | `secrets_env_schema` | `schema` | `crates/secrets-engine/src/paths.rs` | `no` | `missing` | `unset` |
| `META_ROOT` | `secrets_env_schema` | `schema` | `crates/secrets-engine/src/seam.rs` | `no` | `missing` | `unset` |
| `NODE_EXTRA_CA_CERTS` | `secrets_env_schema` | `schema` | `crates/secrets-engine/src/inject.rs` | `no` | `missing` | `unset` |
| `OPENAI_API_KEY` | `secrets_env_schema` | `schema` | `crates/secrets-engine/src/inject.rs` | `yes` | `missing` | `unset` |
| `OPENAI_BASE_URL` | `secrets_env_schema` | `schema` | `crates/secrets-engine/src/inject.rs` | `no` | `missing` | `unset` |
| `PATH` | `secrets_env_schema` | `schema` | `crates/secrets-engine/src/lib.rs` | `no` | `missing` | `unset` |
| `REQUESTS_CA_BUNDLE` | `secrets_env_schema` | `schema` | `crates/secrets-engine/src/inject.rs` | `no` | `missing` | `unset` |
| `REVOKE_TOKEN` | `secrets_env_schema` | `schema` | `crates/secrets-engine/src/mint_github.rs` | `yes` | `missing` | `unset` |
| `SECRETD_CONFIG` | `secrets_env_schema` | `schema` | `crates/secretd/src/config.rs` | `yes` | `missing` | `unset` |
| `SECRETD_EDGE_BIND_ADDR` | `secrets_env_schema` | `schema` | `crates/secretd/src/config.rs` | `yes` | `missing` | `unset` |
| `SECRETD_EDGE_CLIENT_CA_PATH` | `secrets_env_schema` | `schema` | `crates/secretd/src/config.rs` | `yes` | `missing` | `unset` |
| `SECRETD_EDGE_CLIENT_REVOCATIONS_PATH` | `secrets_env_schema` | `schema` | `crates/secretd/src/config.rs` | `yes` | `missing` | `unset` |
| `SECRETD_EDGE_ENABLED` | `secrets_env_schema` | `schema` | `crates/secretd/src/config.rs` | `yes` | `missing` | `unset` |
| `SECRETD_EDGE_REQUIRE_CLIENT_CERT` | `secrets_env_schema` | `schema` | `crates/secretd/src/config.rs` | `yes` | `missing` | `unset` |
| `SECRETD_LIBSQL_AUTH_TOKEN` | `secrets_env_schema` | `schema` | `crates/secretd/src/config.rs` | `yes` | `missing` | `unset` |
| `SECRETD_LIBSQL_AUTH_TOKEN_FILE` | `secrets_env_schema` | `schema` | `crates/secretd/src/config.rs` | `yes` | `missing` | `unset` |
| `SECRETD_LIBSQL_URL` | `secrets_env_schema` | `schema` | `crates/secretd/src/config.rs` | `yes` | `missing` | `unset` |
| `SECRETD_OPERATOR_AUTHORIZER_URL` | `secrets_env_schema` | `schema` | `crates/secretd/src/config.rs` | `yes` | `missing` | `unset` |
| `SECRETD_REQUIRE_MLOCK` | `secrets_env_schema` | `schema` | `crates/secretd/src/config.rs` | `yes` | `missing` | `unset` |
| `SECRETD_REQUIRE_MLOCK` | `secrets_env_schema` | `schema` | `crates/secretd/src/main.rs` | `yes` | `missing` | `unset` |
| `SECRETD_STORE_BACKEND` | `secrets_env_schema` | `schema` | `crates/secretd/src/config.rs` | `yes` | `missing` | `unset` |
| `SECRETD_TOPOLOGY` | `secrets_env_schema` | `schema` | `crates/secretd/src/config.rs` | `yes` | `missing` | `unset` |
| `SSL_CERT_FILE` | `secrets_env_schema` | `schema` | `crates/secrets-engine/src/inject.rs` | `no` | `missing` | `unset` |
| `XDG_CONFIG_HOME` | `secrets_env_schema` | `schema` | `crates/secrets-engine/src/paths.rs` | `no` | `missing` | `unset` |
| `XDG_DATA_HOME` | `secrets_env_schema` | `schema` | `crates/secrets-engine/src/paths.rs` | `no` | `missing` | `unset` |
| `XDG_DATA_HOME` | `secrets_env_schema` | `schema` | `crates/secrets-engine/src/seam.rs` | `no` | `missing` | `unset` |
| `XDG_RUNTIME_DIR` | `secrets_env_schema` | `schema` | `crates/secrets-engine/src/paths.rs` | `no` | `missing` | `unset` |
| `XDG_STATE_HOME` | `secrets_env_schema` | `schema` | `crates/secrets-engine/src/paths.rs` | `no` | `missing` | `unset` |
