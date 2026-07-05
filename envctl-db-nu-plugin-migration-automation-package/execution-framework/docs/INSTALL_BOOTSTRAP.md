# Install Bootstrap

This page is the verified operator entrypoint for installing the package into local `envctl` and `nu_plugin` checkouts and starting Codex/background helpers from generated execution packets.

## Environment

Set these variables before running the templates:

```bash
export PROMPT_PACKAGE_DIR=/path/to/envctl-db-nu-plugin-migration-automation-package
export ENVCTL_REPO=/path/to/envctl
export NU_PLUGIN_REPO=/path/to/nu_plugin
export FLEXNETOS_PACKAGE=$PROMPT_PACKAGE_DIR/source/codex-flexnetos-migration-prompt-package
```

## Repo Install

```bash
cd "$PROMPT_PACKAGE_DIR"
bash INSTALL_IN_REPOS.sh --envctl-repo "$ENVCTL_REPO" --nu-plugin-repo "$NU_PLUGIN_REPO"
```

The installer writes only namespaced additive files into the target repositories. The expected target paths are recorded in `generated/install_bootstrap_manifest.json`.

## Codex Bootstrap

```bash
cd "$PROMPT_PACKAGE_DIR"
bash RUN_WITH_CODEX_ENVCTL.sh \
  --envctl-repo "$ENVCTL_REPO" \
  --nu-plugin-repo "$NU_PLUGIN_REPO" \
  --flexnetos-package "$FLEXNETOS_PACKAGE"
```

## Bounded Packet Execution

```bash
cd "$PROMPT_PACKAGE_DIR/execution-framework"
codex exec < generated/execution_packets/REQ-044_INSTALL_BOOTSTRAP.json
```

## Background Helper Template

```bash
cd "$PROMPT_PACKAGE_DIR/execution-framework"
mkdir -p logs state proof_records
nohup codex exec < generated/execution_packets/REQ-044_INSTALL_BOOTSTRAP.json \
  > logs/REQ-044_INSTALL_BOOTSTRAP.codex-exec.stdout.log 2>&1 &
echo $!
```

Every helper must produce the task proof named by its packet before dependent work advances.

## Verification

```bash
cd "$PROMPT_PACKAGE_DIR/execution-framework"
python3 scripts/verify_install_bootstrap.py
test -s proof_records/REQ-044_INSTALL_BOOTSTRAP.proof.json
```

The verifier checks the package install scripts, regenerates the command template manifest, updates the heartbeat, writes the log, and appends the proof ledger entry.

## Command Template Index

| command id | purpose |
|---|---|
| `install-into-repos` | Copy namespaced Codex prompts, schemas, SQL, and helper configs into both local repositories. |
| `run-codex-master-prompt` | Start Codex from the envctl repo with the combined migration automation package prompt. |
| `run-single-execution-packet` | Execute a single generated packet and require its proof record before advancing dependent tasks. |
| `run-background-helper` | Launch a bounded packet in a background shell while preserving stdout/stderr evidence under logs/. |
