# grit — best-practices + latest trends (cycle 5)

- **Target:** `grit` (v0.4.0) — Rust symbol-level merge/lock substrate: distributed locks over
  symbols, pluggable stores (sqlite / S3-family / Azure), a "room" coordination concept, git
  integration. Planned as the merge engine for union step 2 (dedup the ~95%-shared
  handoff↔rusty-idd crates).
- **Researched:** 2026-06-27. **Recency window:** rolling 90 days (≈ since **2026-03-29**).
- **Method:** fan-out web search → fetch → adversarial verification → cited synthesis. Every
  material finding is **cited + dated**; in-window sources preferred, older ones flagged.
- **Legend:** `[BP]` established best-practice (safe to adopt) · `[TREND]` emerging (watch/pilot) ·
  `[IN-WINDOW]` source dated within 90 days · `[OLDER]` outside the window, with a currency note.
- Claim IDs (`C#`) cross-reference `research/sources-grit.jsonl`.

---

## Headline

The field has moved decisively toward **entity-/symbol-level structural merge** as the answer to the
exact problem grit exists to solve (parallel agents editing a mostly-shared tree). That validates
grit's core design. The one place grit is **behind the field** is its cloud-store layer: it depends
on the **deprecated, legacy community Azure crates** while the official Azure SDK for Rust went **GA
(2026-05-14)** — and several other deps are a few minor versions stale. No CVEs in grit's own
dependency tree were found; the only advisories in scope are Cargo/toolchain CVEs fixed in Rust
1.96.0.

---

## Best-practices & trends — symbol/structural merge

- **C1 [TREND][IN-WINDOW] Entity-level semantic merge now beats line-based by a wide, measured
  margin.** `weave` (an entity-level git merge driver: parses base/ours/theirs into semantic
  entities — functions, classes, JSON keys — via tree-sitter `sem-core`) resolves **31/31** benchmark
  scenarios cleanly vs git's **15/31**, with "zero regressions" across git/git, Flask, CPython, Go,
  TypeScript; on the TS repo it auto-resolved 65 conflicts git flagged. Latest release v0.3.x dated
  2026-06-05. This is direct external validation of grit's **symbol-level** thesis for a 95%-shared
  union — the same input class (independent edits to shared structure) is exactly where line-based
  git invents false conflicts. *Adversarial note: single-vendor self-reported benchmark — treat
  31/31 as a strong signal, not an audited fact; grit should reproduce on its own corpus.*
  (github.com/ataraxy-labs/weave)
- **C2 [BP][OLDER, still current] AST 3-way merge via tree-sitter + GumTree matching is the
  established structural-merge technique.** Mergiraf parses all three revisions with tree-sitter,
  builds matchings between the three syntax trees with the **GumTree classic** algorithm, and merges
  at AST-node granularity (finer than entity-level for expression changes). Architecture stable since
  2024–2025; still the reference design. grit's tree-sitter-based approach is on the mainstream path.
  *Flagged older — Mergiraf architecture docs predate the window but remain the canonical description.*
  (mergiraf.org/architecture.html; techplanet.today)
- **C3 [TREND][IN-WINDOW] The ecosystem is converging on a two-tier "entity-level + AST" split, and
  treating them as complementary.** jj-vcs is actively discussing entity-level merge as a complement
  to Mergiraf's AST merge (Discussion #8831, 2026). Takeaway for grit: symbol/entity granularity
  (grit's "symbol" lock + merge unit) is the right *coordination* grain; AST-node merge is the right
  *content* grain inside a contested symbol. grit can own the former and delegate/borrow the latter.
  (github.com/jj-vcs/jj/discussions/8831)
- **C4 [BP][IN-WINDOW] CRDTs prevent conflicts by construction but do not resolve them post-hoc, and
  remain weak for code/AST structure.** 2025–2026 CRDT field guides reiterate CRDTs target
  offline-first convergence for text/data, "do not resolve conflicts after they occur — they prevent
  conflicts by design," and the literature still centers text (RGA/WOOT/LSEQ), not ASTs. Implication:
  a pure-CRDT reconciliation layer is the *wrong* primitive for grit's job; grit's **lock +
  structural-merge** model (pessimistic claim on a symbol, structural reconcile on release) is better
  matched to code than CRDT-everywhere. Keep CRDT-adjacency as a metadata/room-state idea, not the
  merge core. (iankduncan.com 2025-11-27 [OLDER]; velt.dev 2025-10 [OLDER])

## Best-practices & trends — distributed locks / lease coordination over object stores

- **C5 [BP][IN-WINDOW] Object-store-native conditional writes are now the standard distributed-lock /
  leader-election primitive — no external coordinator needed.** S3 `PutObject` with `If-None-Match`
  succeeds only when the key is absent (else **412 Precondition Failed**); `If-Match` + ETag gives
  compare-and-swap for registry/state updates. AWS extended conditional writes to **copy operations
  in Oct 2025**. This is precisely how grit's S3/R2/GCS store should arbitrate symbol locks and "room"
  state — and it means grit can drop any reliance on out-of-band locking. *Core S3 conditional-write
  GA was 2024 [OLDER]; the copy-op extension is 2025-10 [OLDER but recent] — both still current.*
  (aws.amazon.com/blogs/storage multi-writer; aws.amazon.com/about-aws/whats-new/2025/10)
- **C6 [BP][OLDER, still current] Azure offers both optimistic (ETag/If-Match → 412) and pessimistic
  (Lease Blob) concurrency, and these are the right primitives for grit's Azure store.** Azure Blob
  leases are an exclusive write/delete lock for **15–60s or infinite**, returning a lease ID that
  must accompany subsequent writes; ETag conditional headers give optimistic CAS. grit's Azure path
  should use Lease Blob for held symbol locks and ETag CAS for room-state updates. *MS Learn docs are
  evergreen; a 2026-02-16 lease-management write-up corroborates [OLDER, just outside window].*
  (learn.microsoft.com/.../concurrency-manage; learn.microsoft.com/.../lease-blob)

## Best-practices & trends — merge-lock for multi-agent code editing

- **C7 [TREND][IN-WINDOW] Git worktrees are now the default isolation primitive for parallel AI
  agents — but isolation alone does not solve shared-hotspot conflicts.** Worktree-per-agent (one
  checkout, shared `.git` object store) is natively supported by Claude Code, Codex, and Cursor and
  is the de-facto coordination layer at 4+ concurrent sessions. The unsolved residue: agents editing
  shared hotspot files (routes, configs, registries) still collide. That residue is grit's market:
  symbol-level locks + structural merge layer *on top of* worktree isolation.
  (mindstudio.ai; augmentcode.com; appxlab.io 2026-03-31)
- **C8 [TREND][IN-WINDOW] Empirical 2026 research confirms concurrent-modification / merge conflict
  is a *primary* failure mode of agent fleets, and tools are pre-checking conflicts via
  `git merge-tree`.** The **AgenticFlict** dataset (arXiv, 2026) catalogs merge conflicts in
  AI-agent PRs at scale; work on async SWE agents reports merge conflicts as a top failure when
  concurrency primitives aren't explicitly modeled; tools like "Clash" run 3-way `git merge-tree`
  between worktree pairs to detect conflicts *before* agents finish. grit should expose a
  pre-merge / dry-run conflict probe in the same spirit. (arxiv.org/pdf/2604.03551;
  arxiv.org/pdf/2603.21489)

## Implication for grit's union role (95%-shared fork union)

- The external evidence (C1–C3, C7–C8) says symbol/entity-level merge is *the* technique for the
  union's exact shape (two near-identical crate trees, independent edits). grit's design is aligned;
  the architect should lean on this to justify grit as the union merge engine rather than line-based
  git or a CRDT layer.
- Lock arbitration should be **store-native conditional writes / leases** (C5, C6), keeping grit free
  of an external coordinator — important for a portable substrate.
- Add a **pre-merge dry-run conflict probe** (C8) so the union loop can detect a contested symbol
  before committing an agent's work.

---

## Tool-currency & advisories

Versions pinned read from `/home/drdave/Desktop/meta/grit/Cargo.toml` and resolved in `Cargo.lock`
(2026-06-27). "Latest" from crates.io / docs.rs as of 2026-06-27.

| Crate (grit pin → resolved) | Latest | Gap | Advisory | Action |
|---|---|---|---|---|
| `rusqlite` 0.31 → 0.31.0 | **0.40.0** (≈2026-06-17) | 9 minor releases; bundled SQLite older (0.37 bumped to 3.50.2) | None found (RustSec) | **C9** Upgrade; rusqlite has multiple breaking minors — stage it. |
| `azure_core` 0.21 → 0.21.0 | new GA line **1.x** | major; legacy lineage | Deprecated lineage | **C10** Migrate to GA `azure_core` 1.x. |
| `azure_storage` 0.21 → 0.21.0 | superseded | major | **Legacy/deprecated** | **C10** No GA equivalent of the umbrella crate; fold into blob crate. |
| `azure_storage_blobs` 0.21 → 0.21.0 | **`azure_storage_blob` 1.0.0** (singular, GA 2026-05-14) | crate renamed + majored | **Legacy: "fully deprecated", source moved to `azure-sdk-for-rust/tree/legacy`, "no plans to update"** | **C10 (highest-priority currency item)** Migrate `azure_storage_blobs` → `azure_storage_blob` 1.0. |
| `tree-sitter` 0.25 → 0.25.10 | **0.26.8** (2026-03-31) | 1 minor | None found | **C11** Upgrade to 0.26.x; re-pin grammar crates to match. |
| `colored` 2 → 2.2.0 | **3.0.0** | 1 major | None found for `colored` | **C12** Optional upgrade to 3.0. |
| `aws-sdk-s3` 1 → 1.126.0 | current (1.x) | — | None found | Current; keep tracking 1.x. |
| `aws-config` 1 → 1.8.15 | current (1.x) | — | None found | Current. |
| `tokio` 1 → 1.50.0 | current (1.x) | — | None found | Current. |
| `serde` 1 → 1.0.228 | current | — | None | Current. |
| `serde_json` 1 | current | — | None | Current. |
| `anyhow` 1 → 1.0.102 | current | — | None | Current. |
| `chrono` 0.4 → 0.4.44 | current (0.4.x) | — | None | Current. |
| `clap` 4 → 4.6.0 | current (4.x) | — | None | Current. |
| `glob` 0.3 → 0.3.3 | current | — | None | Current. |
| `futures` 0.3 → 0.3.32 | current | — | None | Current. |
| `tempfile` 3 → 3.27.0 | current | — | None | Current. |
| `urlencoding` 2 → 2.1.3 | current | — | None | Current. |

### Advisories (toolchain — affects the build, not grit's deps)

- **C13 [IN-WINDOW] Cargo CVE-2026-5223 & CVE-2026-5222** — symlink handling in third-party-registry
  crate tarballs lets a malicious crate override another crate's source / escape the cache. **Fixed
  in Rust 1.96.0 (released 2026-05-28).** crates.io users are unaffected (it forbids symlink
  uploads), but vendored/mirror/private-registry flows are exposed. **Action:** pin the union build
  toolchain to **Rust ≥ 1.96.0**. (blog.rust-lang.org/2026/05/25/cve-2026-5223)
- **C14 [IN-WINDOW/OLDER] Cargo CVE-2026-33056** (2026-03-21, Rust blog) — earlier 2026 Cargo
  advisory; same mitigation (use a current toolchain). (blog.rust-lang.org/2026/03/21/cve-2026-33056)
- **C15 [OLDER, thematically relevant] RUSTSEC-2024-0364 (gitoxide-core)** — failure to neutralize
  terminal special characters. grit emits colored/terminal output (`colored`) over untrusted symbol
  names / diff content; **sanitize control characters before printing to a TTY** as a hardening
  best-practice. Not a vuln in grit's deps, carried as a relevant pattern.
  (rustsec.org/advisories/RUSTSEC-2024-0364.html)

### Currency verdict

grit's general-purpose deps are **current**. Two real currency actions, in priority order:
1. **Azure crates (C10)** — on a *deprecated legacy lineage*; migrate to GA `azure_storage_blob` 1.0
   / `azure_core` 1.x. Highest priority: the legacy crates will not receive updates.
2. **rusqlite 0.31 → 0.40 (C9)** — 9 minor versions stale (staged, breaking minors).
   `tree-sitter` and `colored` are minor/major-behind but low-risk.

---

## Gaps / low-confidence

- **weave benchmark (C1)** is vendor self-reported — corroborated for *approach* (tree-sitter,
  entity-level) but the 31/31 figure is a single-source claim; flagged, not treated as audited.
- No public RustSec advisory was found *specifically* for `rusqlite`, `colored`, or the legacy Azure
  crates; absence-of-advisory ≠ proof of safety (the legacy Azure crates being unmaintained is itself
  a supply-chain risk even without a CVE).
- AST-based CRDT comparison (C4) is thin in the literature for the window — finding rests on general
  CRDT field guides, not a code-specific 2026 study.

## Sources

In-window (within 90 days, ≥ 2026-03-29):
1. github.com/ataraxy-labs/weave — entity-level merge driver, 31/31 vs 15/31, tree-sitter (C1) — release dated 2026-06-05.
2. github.com/jj-vcs/jj/discussions/8831 — entity-level merge complementing Mergiraf (C3), 2026.
3. docs.rs/crate/tree-sitter/latest — tree-sitter 0.26.8, 2026-03-31 (C11).
4. crates.io/crates/rusqlite/versions — rusqlite 0.40.0 latest, ~2026-06 (C9).
5. devblogs.microsoft.com/azure-sdk/from-beta-to-stable-announcing-the-azure-sdk-for-rust-ga — Azure SDK for Rust GA 2026-05-14, `azure_storage_blob` 1.0.0 (C10).
6. devops.com/microsoft-brings-the-azure-sdk-for-rust-to-general-availability — GA coverage incl. Storage Blobs/Queues (C10).
7. crates.io/crates/azure_storage_blobs + docs.rs — legacy notice, source moved to azure-sdk-for-rust/tree/legacy, "no plans to update" (C10), accessed 2026-06-27.
8. blog.rust-lang.org/2026/05/25/cve-2026-5223 — Cargo symlink CVE, fixed Rust 1.96.0 2026-05-28 (C13).
9. blog.rust-lang.org/2026/05/25/cve-2026-5222 — companion Cargo CVE (C13).
10. blog.rust-lang.org/2026/03/21/cve-2026-33056 — earlier 2026 Cargo advisory (C14).
11. aws.amazon.com/about-aws/whats-new/2025/10 — S3 conditional writes extended to copy ops, 2025-10 (C5).
12. arxiv.org/pdf/2604.03551 — AgenticFlict merge-conflict dataset for AI-agent PRs, 2026 (C8).
13. arxiv.org/pdf/2603.21489 — async SWE agents; concurrency/merge-conflict failure mode, 2026-03 (C8).
14. blog.appxlab.io/2026/03/31/multi-agent-ai-coding-workflow-git-worktrees — worktree coordination, 2026-03-31 (C7).
15. oneuptime.com/blog/post/2026-02-16-...lease-management... — Azure blob lease concurrency, 2026-02-16 (C6) [edge/older].

Older — relied on with a currency note (cited because nothing in-window supersedes them):
16. mergiraf.org/architecture.html — tree-sitter + GumTree AST 3-way merge (C2) [OLDER, canonical].
17. techplanet.today/post/mergiraf-ast-oriented-tool-for-three-way-merging-in-git — Mergiraf overview (C2) [OLDER].
18. aws.amazon.com/blogs/storage/building-multi-writer-applications-on-amazon-s3-using-native-controls — If-None-Match/If-Match CAS, leader election (C5) [OLDER 2024-25, still current].
19. learn.microsoft.com/en-us/azure/storage/blobs/concurrency-manage — ETag optimistic + lease pessimistic concurrency (C6) [evergreen MS Learn].
20. learn.microsoft.com/en-us/rest/api/storageservices/lease-blob — Lease Blob 15–60s/infinite, lease ID (C6) [evergreen].
21. iankduncan.com/engineering/2025-11-27-crdt-dictionary — CRDT field guide; prevent-not-resolve (C4) [OLDER 2025-11].
22. velt.dev/blog/crdt-implementation-guide-conflict-free-apps — CRDT guide, 2025-10 (C4) [OLDER].
23. rustsec.org/advisories/RUSTSEC-2024-0364.html — terminal escape neutralization hardening pattern (C15) [OLDER, relevant].
