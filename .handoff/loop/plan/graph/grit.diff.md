# Graph diff — `grit`

**Baseline: no prior snapshot.** This is the first code-graph snapshot for target `grit` (cycle 5).
There is no previous committed `graph/grit.symbols.json` to diff against.

Recorded baseline for future delta computation:

| metric | baseline value |
|---|---|
| snapshot | `graph/grit.symbols.json@57b60842d71145c271b994bb7a8c33c3bca42dfe` (branch `master`) |
| symbols (src) | 305 |
| intra-src call edges | 548 |
| `pub` symbols | 74 |
| files | 11 |
| true architectural cycles | 0 (one SCC found = resolver artifact) |
| dead (no internal caller) | 167 (mostly dyn/clap dispatch + pub API + serde helper) |
| layering violations | 0 |
| top hotspot | `SymbolIndex.new` (parser/mod.rs:94) |

Next cycle: re-run `git-kb code` for grit, recompute, and report new/removed symbols, edge churn, and metric movement against this baseline.
