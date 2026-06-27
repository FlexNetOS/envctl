# Feature Forge architect plan — TASK-0072 Ollama model store into meta

Date: 2026-06-27
Branch/worktree: `task-0072-ollama-models-meta`

## Verified claim

Backlog TASK-0072 requires keeping the meta-owned Ollama runner/client primary while moving model blobs out of root/real-home daemon stores into a meta-owned location. Ollama removal is explicitly deferred until shimmy+ruvllm prove parity.

Existing state before this cycle:
- `ollama` component installed runner bytes under `$META_ROOT/.toolchains/ollama` and exposed `$META_ROOT/usr/bin/ollama` as a symlink to the toolchain binary.
- `envctl env --toolchains` exported `OLLAMA_LIBRARY_PATH` only.
- No canonical layout helper/export existed for `OLLAMA_MODELS`.

## Design

1. Treat Ollama model layers as persistent state, not toolchain binaries:
   - canonical path: `$META_ROOT/var/lib/ollama/models`.
2. Add `MetaLayout::ollama_models()` and export it from `envctl env --toolchains` in shell and JSON modes as `OLLAMA_MODELS`.
3. Change the manifest component from a symlink to a meta-owned wrapper at `$META_ROOT/usr/bin/ollama`:
   - wrapper forces `META_ROOT`, `OLLAMA_MODELS`, and `OLLAMA_LIBRARY_PATH`;
   - wrapper execs `$META_ROOT/.toolchains/ollama/bin/ollama`.
4. During install, create `$META_ROOT/var/lib/ollama/models` and non-destructively copy legacy stores into it only when the meta store is empty. Never delete root/real-home legacy model stores behind envctl's back.
5. Preserve the runner binary under `.toolchains/ollama`; do not implement shimmy/ruvllm removal in this task.

## Runtime surface

- `envctl env --toolchains` must print `OLLAMA_MODELS=$META_ROOT/var/lib/ollama/models`.
- `envctl env --toolchains --json` must carry the same path.
- `envctl lock --check` must accept the updated manifest lock.
