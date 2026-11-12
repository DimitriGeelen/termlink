# T-3357 AC1 (r3): independent review of the step 8 plan v0.3 against the planner card's completion conditions

Reviewer: reviewer-judge-t-3357-r1-7d6b2dfb08fc (rung-1-same-vendor-independent)
Revision reviewed: 1938e19898bc7ea072c5200ac2f7459811fffdb2 (files read with `git show 1938e19:<path>`)
Plan: v0.3, 1067 lines, sha256 714b0ea5...cb10b. This equals the hand-back output.sha256.

## Method (my own scripts in /tmp/rv3357; I did not run or trust the producer's plan.py)
1. Hand-back integrity: I recomputed the sha256 of the output and all 23 inputs at the revision. 0 of 24 mismatch.
2. The v0.2 -> v0.3 diff (git diff 907da00 1938e19, word-diff). Only these change: the status line, a version-history
   row, and the Acceptance/Verification cells of P-25, P-35, P-37, P-40, P-52, P-54, P-63 and P-71. No edge, order row or
   drawing changes.
3. Graph (card 6.1):
   - Unit tables: 71 units. Kahn's algorithm removed all 83 nodes the unit tables reference, so there is no cycle.
   - Order table 6.1: 87 nodes and 150 edges (the claim), with no unknown node. 0 edges point to a node placed at or
     after its dependant, so the order is a valid topological order and the graph is acyclic.
4. Coverage, against REQ (R-1..R-82) and EV:
   - Every R-n is served except R-38, R-40 and R-43. The plan explains each in 11.1 (deferred, merged, dropped), as the
     prior reviews confirmed against REQ.
   - EV has 173 distinct E-n. All 173 are served, and no unit names an unknown E-n.
5. Every unit has acceptance and a verification check (card 6.2):
   - All 71 units have non-empty Acceptance and Verification cells.
   - No unit without an E-n uses the generic "PASS line per listed E-n" check.
   - A scan for report/review checks that name no failing condition flags only P-9 and P-44. See the observations below.
6. Drawings (card 6.4): I extracted the three Mermaid blocks (D-1.1, D-1.2, D-2). They are byte-identical to v0.2. I
   rendered each with mmdc and chromium: all three exit 0, giving SVGs of 169923, 383897 and 134578 bytes, with no
   syntax or parse error. The id, caption and text equivalent are unchanged from v0.2 (verified by r2) and are untouched
   by the diff.
7. Rulings: the diff changes no edge, order or decision text. Ruling A (P-4 -> P-6 -> P-7 serial), T-3689 9a (P-5),
   ruling D / F-20 (P-7 interim veto; P-43 behind G-8), NID decommission units behind gates, and "no work item created"
   (card 3.2) all stand as verified in r1/r2.

## The r2 findings, re-checked against the sources
- F-5 FIXED. FP 6.2.3.d (03 line 442) has items (1)..(8).
  - P-40 Acceptance now names all eight, each matching FP: probes; the live test plus the 14-day canary (H-14); GP-12
    plus unsurfaced mail (H-18, H-19); kill switch; install/upgrade/rollback/macOS/platform-lock; strength flags; H-3,
    H-7, H-11, H-13, H-20 and H-30; inventories. It adds H-26 (FP 6.1.b (v)).
  - P-40 Verification: "exit report ... lists FP 6.2.3.d (1)..(8) and H-26, each with PASS ... and a link to its
    evidence; arc-live-probe exits 0 on .107; an item missing, without evidence or marked FAIL, or an H-n without a
    measured value, fails the unit". This states a failing condition.
- F-6 FIXED. FP 6.2.7.a (6) / 6.2.7.b (03 lines 465-466) require:
  - every R-69 dimension run;
  - every tripwire reviewed;
  - the bypass inventory re-run;
  - arc-live-probe on every hub (G-069);
  - H-29 met;
  - the register saying what operates (A-6).
  P-71 lists these as (a)..(f), with the H-29 bounds copied from the FP H-29 row (03 line 663). Its check lists
  (a)..(f), each with PASS and evidence, and fails on any item that is missing, has no evidence or is marked FAIL, or on
  any hub whose probe exits non-zero.
- Sweep consistency. FP 9.3 says "H-26 at every exit". The exit units are P-40 (PH-2), P-54 (PH-3), P-61 (PH-4), P-63
  (PH-5) and P-71 (PH-6). P-40, P-54, P-63 and P-71 name H-26 explicitly. P-61 carries it through FP 6.2.5.c (7), which
  is itself "bypass inventory re-run ... (H-26)", and P-61 lists (1)..(9). So every exit covers H-26.
- P-25, P-35, P-37 and P-52 now name what fails each report, and each failing condition matches its acceptance.

## Observations (not findings; they do not affect card 6.2)
- P-44 (AEF request). Its check, "parity fixture run reported by AEF", states no failing condition, unlike P-45 and P-47.
  It is not a rubber stamp: the acceptance requires the listed E-n and the B-29 specs to pass against fixtures that
  TermLink publishes, and a fixture run is pass/fail by construction. Both r1 and r2 cited P-44 as acceptable. The
  planner may align its wording with P-47 when convenient.
- P-9. Its check, "H-11 report with the 7-day numbers", is bounded by the acceptance "H-11 (a)..(f) within their [P]
  bounds for 7 days", so the failing condition is stated in the acceptance.

## Card 6.3
"After approval: mapped to work items" is correctly not yet met. This is pre-approval, and section 12 gives the rule.

## Verdict
green. Card 6.1 (coverage complete, no cycle), 6.2 (every unit has acceptance and a real verification check, no
rubber-stamp criteria), 6.4 (drawings render) and the rulings are each independently verified at revision 1938e19.
Both open r2 findings (F-5, F-6) are fixed against the FP text.
