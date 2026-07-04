# T-2349 — A1/A2 spikes: reviewer rail vs inception task files

**Task:** T-2349 (test/spike; follow-through of the T-2348 GO, recorded c62de76b)
**Date:** 2026-07-04
**Rail under test:** fw independent-review v0.1 (`fw reviewer`, T-1885), vendored at
`.agentic-framework/lib/reviewer/static_scan.py`, catalogue `v1.3-seed`.

## A1 verdict — YES, natively (no input adapter needed)

**Question:** can the T-1885 rail consume an inception TASK FILE as its review subject?

**Method:** `fw reviewer <id> --no-write --json` against three real inceptions:
T-2338 (active, decision-ready), T-2276 (active, decision-ready), T-2348 (completed).

**Result:** all three parsed and scanned without refusal. The rail is already
inception-aware: catalogue pattern `disposition-incomplete` ("Inception IW-N
answered-without-citation", T-2191) gates on `workflow_type: inception` frontmatter,
extracts the `## Open Questions` section, slices per-`IW-N` entries, and evaluates
`disposition:` / `rationale:` fields per entry. Sample verdict envelope (T-2338):

```json
{"task_id": "T-2338", "overall": "CONCERN", "catalogue_version": "v1.3-seed",
 "findings": [{"pattern_id": "disposition-incomplete",
   "location": "## Open Questions: IW-1",
   "evidence": "IW-1 disposition='answered' but rationale has no evidence citation (…)"}]}
```

**Implication for pickup 073:** the proposal's "validator profile extending v0.1"
is architecturally confirmed — the rail's input layer already handles inception task
files. The AEF-side build needs NO new input adapter; it extends the pattern
catalogue plus verdict rendering.

## A2 verdict — YES, claims are machine-extractable; extraction has 2 verified defects

**Question:** are inception evidence claims machine-extractable with enough structure
to verify?

**Claim shapes found** (Recommendation/Evidence + IW rationales of T-2338, T-2276,
T-2348): task refs (`T-2314`), file:line cites (`channel.rs:8568-8611`,
`channel.rs:8597-8601`), report paths (`docs/reports/T-XXX-*.md`), gap/learning ids
(`G-063`, `PL-236`), commit hashes (`c62de76b`), tool-consult refs
("claude-code-guide 2026-06-24"), runnable evidence commands (`termlink list`,
`grep -a -c dm.queued /proc/<pid>/exe`). The rail already encodes almost exactly this
taxonomy as `_CITATION_PATTERNS` (static_scan.py:1518) — T-NNNN, docs/reports/,
G-/L-/D-id, file:line, file#Lnnn, dialogue-log, commit-hash. So the extraction
question is settled in principle: the shapes exist and the regexes exist.

**But extraction quality has two verified defects** (reproduced in isolation by
importing the vendored `static_scan` and running its own slicing on the real files):

1. **Template-comment IW entries parsed as real entries.** The task template's HTML
   comment inside `## Open Questions` contains a literal example entry
   (`- **IW-1: <question text>**` … `disposition: answered | deferred | dissolved` …
   `rationale: <one-line evidence — …>`). The detector does not strip HTML comments,
   so the example matches the IW-bullet regex, `(\S+)` captures `answered` from the
   pipe-separated example, and the placeholder rationale has no citation → a
   **phantom finding** on every inception that retains the template comment.
   Observed: T-2338's flagged "IW-1" is the comment's example, not the real IW-1;
   T-2276's IW-1 is flagged TWICE (comment phantom + real entry).

2. **Multi-line rationales truncated to the first line.** Check D extracts the
   rationale via `^\s*rationale:\s*(.+?)$` (first line only). T-2338's real IW-2
   rationale carries its citation (`channel.rs:8597-8601`) on the continuation
   line → **false "answered-without-citation"**. Repro:

   ```
   rationale-line: rationale: Premise false — … (T-2314), `channel.rs:8568-8611`. The | has_citation: True
   rationale-line: rationale: Yes, one narrow one — after 6 consecutive …            | has_citation: False
                   (continuation line, unseen: "…(`channel.rs:8597-8601`, never re-probes WS)")
   ```

**Extraction ratio observed:** of the 4 real `answered` IW entries examined across
T-2338 + T-2276, 3 carry regex-matchable citations in their full rationale text, but
the current first-line-only extraction detects just 1 → 2 false positives + the
phantom/duplicate class on top. Fixing the two defects lifts precision without any
new claim-shape work.

## Implication for the AEF-side build (pickup 073)

- **No input adapter** — the rail consumes inception task files today (A1).
- The verdict-rail work decomposes into: (a) fix the two extraction defects above
  (strip HTML comments before entry slicing; accumulate rationale continuation lines
  until the next `key:` field or entry boundary); (b) extend from *shape* checking
  ("does a citation exist?") to *verification* ("does the citation hold?" — re-read
  the cited file:line, re-run the cited command) producing the per-claim
  CONFIRMED / UNVERIFIED / CONTRADICTED taxonomy proposed in pickup 073.
- The two defects are AEF-upstream code (`lib/reviewer/static_scan.py`) — filed to
  AEF as pickup `074-reviewer-disposition-detector-two-defects.md` (see T-2349
  Updates for the delivery record).

## Spike method note

All scans ran with `--no-write` — no task file was mutated by the rail during the
spikes. Detector repro used the vendored module directly
(`sys.path.insert(.agentic-framework/lib/reviewer)`) against the real task files.
