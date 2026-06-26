---
name: plan-trend-research
description: >-
  Deep WEB research on a planning target's tech — best-practices + LATEST trends — through a ROLLING
  90-day recency gate, every finding cited AND dated. A thin wrapper over the deep-research method for the
  planning-engineer harness. ALWAYS use for "latest trends in <X>", "current best-practices for <X>",
  "what's new with <library/crate>", "is <tool> still current", "research the field for the plan", AND
  follow-ups — "re-run the research", "update the trends", "redo the research for <target>", "dig
  deeper on <topic>". Recency-gated + adversarially sanity-checked; feeds the tool-currency/advisory
  input to the plan. Used by `plan-trend-researcher`.
---

# plan-trend-research — recency-gated field research (R3a)

Answer "what is the current best practice and the latest movement in this target's technology?" with
**cited, dated** evidence inside a **rolling 90-day window**. This applies the deep-research *method*
(the engine: fan-out web search → deep-read sources → adversarially verify → cited synthesis),
implemented **inline here** — there is no separate `deep-research` skill to load; it adds one thing
that method does not enforce — a **hard recency gate** computed from today's date. Output: `.handoff/loop/plan/research/<T>.trends.md`. Used by
`plan-trend-researcher`.

Why a wrapper and not just `deep-research`: a *plan* must not recommend a stale practice or a tool that
just shipped a breaking release or a CVE. The 90-day gate makes recency a first-class, checkable
property of every finding, and the tool-currency/advisory subsection (below) is the direct input the
architect's tool-evaluation (R7) consumes.

## Method

### 1. Decompose the target's tech into 3–5 sub-questions

From the target's stack (the crates/tools the cartographer surfaced in `metrics.public_api` +
`reports/codemap-<T>.md` external deps), derive 3–5 specific sub-questions, e.g. for a tonic/prost
gRPC crate:
- Current best-practice for tonic service/proto layout and error modeling?
- Latest tonic/prost releases — breaking changes, new APIs, deprecations?
- Known CVEs/advisories for tonic/prost/the TLS stack in the last 90 days?
- Emerging patterns for the target's problem (e.g. gRPC reflection, validation, streaming)?

Keep them answerable and tied to *this* target — not a generic survey.

### 2. Fan-out web search (the `deep-research` engine)

For each sub-question, search 2–3 keyword variations across the available tools:
- **WebSearch / WebFetch** — general + news-focused queries; deep-read 3–5 promising sources in full
  (don't rely on snippets).
- **context7** (`resolve-library-id` → `query-docs`) — for **library/crate/framework/CLI docs**:
  current API, config, version-migration. Prefer this over web search for library documentation — your
  training data may lag the current release.
- **exa** (if configured) — `web_search_exa` / `web_search_advanced_exa` with a `startPublishedDate`
  set to the window start (below) to bias toward in-window results.

Aim for the `deep-research` coverage bar (≈15–30 unique sources for a broad target; fewer is fine for
a narrow one). Mix academic/official/reputable-news over blogs over forums.

### 3. The recency gate — rolling 90-day window (the addition)

- **Compute the window from today's date — do NOT hardcode it.** `window_start = today − 90 days`
  (get today from the system date / the loop's `session_started`; record `recency_window_days: 90` and
  the resolved window in `loop_state.md`).
- **Prefer in-window sources.** Bias every search to the window; when a tool supports a date filter,
  set it to `window_start`.
- **Every finding is cited AND dated** — `(Source, URL, YYYY-MM-DD)`. A finding with no date is
  incomplete; chase the publish date or drop it.
- **Anything older than the window is explicitly flagged** `older — verify still current` and only
  kept when it is a durable best-practice with no in-window contradiction. Never present an out-of-window
  source as "the latest" without the flag.

### 4. Adversarially sanity-check claims (inherit `deep-research`)

Don't just collect — **try to break each claim**: cross-reference (single-source claims are flagged
`unverified`); separate fact from inference/projection; prefer primary sources (release notes,
advisories, official docs) over secondary commentary; flag contradictions between sources. This is the
`deep-research` quality bar (every claim sourced, no hallucination, acknowledge gaps) applied here.

### 5. Output → `research/<T>.trends.md`

```markdown
# <T> — field research (window: <window_start> .. <today>, rolling 90 days)
*Sources: <N> | in-window: <k> | confidence: High/Medium/Low*

## Best-practices (current)
- <practice> — <why it matters here> (Source, URL, YYYY-MM-DD)[ · older — verify still current]

## Latest trends / movement (in-window)
- <trend / new API / pattern> (Source, URL, YYYY-MM-DD)

## Tool-currency & advisories  ← feeds the architect's tool-evaluation (R7)
| tool / crate / CLI / MCP | latest version | released | breaking? | CVE / advisory | recommend |
|--------------------------|----------------|----------|-----------|----------------|-----------|
| tonic | x.y.z | YYYY-MM-DD | yes/no | none / CVE-… | upgrade / hold / pin |

## Gaps / unverified
- <sub-question with thin or no in-window evidence — stated honestly, not invented>

## Sources
1. [Title](url) — YYYY-MM-DD — one-line
```

The **Tool-currency & advisories** subsection is mandatory and is the R7 input — one row per tool/CLI/
MCP/crate the target uses (from the codemap), with its latest version, release date, breaking-change
flag, any advisory, and an upgrade/hold/pin recommendation. The architect cross-references this against
the graph's actual dependency usage; the verifier may sanity-check a currency claim.

## Discipline

- **Recency is a gate, not a vibe** — compute the window from today; date every finding; flag the rest.
- **Cite or it's not a finding** — no unsourced assertions; single-source ⇒ `unverified`; primary over
  secondary.
- **No hallucination, acknowledge gaps** — "insufficient in-window data found" is a valid, honest
  output for a sub-question; the architect will list it under gaps rather than guess.
- **Findings, not facts (yet)** — the researcher proposes; a currency/advisory claim that lands in the
  plan can still be sanity-checked by the verifier. Don't overstate confidence.
- **Stay on-target** — research *this* target's tech, not a general literature review; tie every
  sub-question to a crate/tool/pattern the cartographer actually found.
