# weave — code graph diff

**No prior snapshot — baseline established this run.**

This is the first committed code-graph snapshot for the **weave** target (cycle 4, parallel
instance). The delta this cycle is the full baseline. Future cycles diff against
`graph/weave.symbols.json@4fe2419`.

## Baseline counts (snapshot @4fe2419, branch `plan/weave-red-tests`)

| metric | value |
|---|---|
| crates (workspace members) | 4 (`weave-core`, `weave-inject`, `weave-mcp`, `weave`) |
| index symbol_count | 2722 (rust 2685, python 37) |
| resolved call edges (index) | 9571 |
| unresolved calls (index) | 25821 (no_match 14492 · skip_list 10534 · ambiguous 774 · stdlib 21) |
| deep-indexed source files | 36 |
| captured symbols (de-duped src) | 2119 |
| production call edges (derived) | 4204 over 1717 nodes |
| inferred entrypoints | 809 (binary `main` @ weave/src/main.rs:4489) |
| traced flows | 464 |
| CLI verbs (top-level `Cmd`) | 71 |
| MCP tools (`weave_*`) | 78 |
| mux backends (`Mux` enum) | 7 (tmux·zellij·kitty·wezterm·screen·iterm2·none) |
| multi-node SCCs (cycles) | 3 (all same-file, ≤5, likely resolver artifacts) |
| genuine dead production code | 0 |
| compile-time layering violations | 0 |
| repo-escaping path deps | 0 |

## What a future cycle should watch

- **`weave-core/src/model.rs`** — blast radius 1238; any `Intent`/`Message` schema change ripples
  fleet-wide. Track field additions here as the highest-risk delta.
- **`SqliteStore` / `LibsqlStore` symmetry** — two backends must stay behavior-equivalent; watch for
  edge churn that lands in one but not the other.
- **MCP tool count (78) vs CLI verb count (71)** — these surfaces should evolve together; a diverging
  count is a parity-drift signal.
- **SCC set** — if a new multi-node or cross-crate SCC appears, that is a real regression (the
  current 3 are name-ambiguity noise, not architecture).
