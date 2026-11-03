# T-3353 AC1: independent review of step 4 (architect) output, v0.2

Reviewer: judge-t-3353-r5-claude-code-23bdb3614f77 (rung-5-panel:claude-code). Revision 92d8d1bb1a000079544e94555658a6e966d47031.
Verdict: **green**

**Panel note:** this is a degraded, single-vendor panel. Fewer vendors could be dispatched than the panel needs, so this
run cannot meet a multi-vendor requirement. This verdict is recorded and reported, but the criterion stays open.

## Integrity
- ARC4 sha256 is `96659c76…08cc`. It matches the hand-back `output.sha256`. The 04a sha256 is `597988d7…af9a`. It matches the hand-back summary.
- All 7 hand-back input hashes match the files at this revision. I checked them with a Python sha256 script.
- Card: `docs/design/roles/cards/architect.md` section 6 (6.1 to 6.5).
- Diff 3098980ef..92d8d1bb1 on docs/: only ARC4 (+111/-27) and the hand-back changed. This fits a remediation scoped to the round 1 findings.

## 6.1 Every R-n, TH-n, SI-n addressed; no unexplained element: MET
- I wrote my own script. It pulls ids from REQ, TM and FP and checks each one against ARC4 and 04a.
  - R-1..R-82: all present in 04a table 2. The table has two columns per row.
  - TH-1..TH-65: all present in table 3.
  - SI-1..SI-31 and FX-1..FX-8: all present in table 1.
  - BP-1..BP-19: all present.
- The only miss is FP SI-32. It is the "consider an invariant [P]" of CX-1, a change request to step 2, and it is not in the TM floor. ARC4 covers it as BP-20 [P] and 12.6, and 18.2.3 now disposes of CX-1 explicitly. This is not a gap.
- 04a section 4 maps the elements back to what they serve.

## 6.2 Baseline, transitions, target, with boundaries and instrumentation: MET (round 1 finding 1 closed)
- 11.4 states that D-1's built-coloured subset is the baseline view. It names the boundaries and the built [i] points, and section 4 is its text equivalent.
- 11.5 is a table with one row each for baseline, T1..T6 and target. Each row gives the TB-n introduced (+) or changed (~) and the [i] that shows the transition took effect. A transition counts as done only when every [i] in its row reads as stated.
- Every TB-n cited is TB-1..TB-13, and each exists in TM section 5 (lines 200-215). D-1 now labels the operator zone TB-10 and the lease TB-12 [i], matching TM.
- Target: D-1 and D-2, as in v0.1.
- Non-blocking inconsistency 1: the D-1 caption (5.2) says the drawing shows "TB-1 to TB-13". The Mermaid block labels only TB-1, 2, 5, 6, 7, 8, 10 and 12.
- Non-blocking inconsistency 2: 11.4 places TB-1 to TB-4 and TB-9 on the agent host, but the D-1 subgraph label says "(TB-1, TB-2)".
- Non-blocking inconsistency 3: D-1 labels the hub zone "TB-6", but TM defines TB-6 as sidecar-to-sidecar *without* the hub, and 11.5 says TB-6 stays unused.
- None of these hides a boundary: 11.4 and 11.5 give the full text. They are worth tidying in a later revision.

## 6.3 Every bypass route removed or monitored: MET
- Section 14 has rows BP-1..BP-19, which is every BP in TM, plus BP-20 and BP-21 [P]. Each row has a design element and an outcome.

## 6.4 Every open decision a DR with options and recommendation: MET (round 1 finding 3 closed)
- DR-1..DR-14 each have options (A/B[/C]), a bold recommendation, and consequences. The table now also has Status and Decision columns. Every DR is "proposed", either routed to an O-n or ruled with the approval of this step, and the decision is "none yet".
- 18.1 states the rule for how a DR's status changes.
- 18.2 disposes of 18 FP 14.1.b items as Built, Decided [P], Partly decided or Open, each with a location. This includes the two items round 1 named: BV composition (18.2.1) and claim/transcript-complete (18.2.7).
- O-1..O-7 are in section 21 and match the hand-back's `decisions_needed`.

## 6.5 Drawings D-1..D-5 with id, caption, text equivalent, render without error: MET
- Captions are at 5.2, 5.4, 7.6, 10.1 and 11.1. Text equivalents are at 5.3, 5.5, 7.7, 10.2 and 11.2.
- I extracted the 5 Mermaid blocks myself and rendered each with `mmdc -p {"executablePath":"/usr/bin/chromium","args":["--no-sandbox"]}`.
  - Exit code was 0 for all five.
  - SVG sizes were 47343, 26237, 43435, 73445 and 24588 bytes. These are identical to ARC4 24.3.
  - Zero "syntax error" or "parse error" strings in any SVG or log.

## Fit as input for step 5: MET (round 1 finding 2 closed)
- Section 15 now has rows for K-12, K-17, K-24 and K-25. Each gives the property, how it is checked, and what the check does NOT prove.
- 15.1 covers the built components K-1..K-8, K-18 and K-22. It defers K-26..K-31 to their sub-designs, with a MUST to add their rows before their build starts.
- Section 6 gives a proof and a mutant for each PH-2 row. There are 24 build items (B-n). This matches the evidence-specialist card inputs in section 2.1.

## Other panel runs (input, not trusted)
- r5-claude-code-fb0793567bd1 returned green at d48fa1537. r5-codex returned not-evaluated (Chromium could not launch).
- ARC4 is byte-identical between d48fa1537 and 92d8d1bb1: the hand-back hash is unchanged and only verdict commits came after.
- My render check above independently covers the item codex could not evaluate.
