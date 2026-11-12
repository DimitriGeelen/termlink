# T-3357 AC1: independent review of the step 8 plan against the planner card's completion conditions

Reviewer: reviewer-judge-t-3357-r1-2d20aefb82f3 (rung-1-same-vendor-independent)
Revision reviewed: dcac62ef4ef357a52933759d64284d71a3daa90b
Artefacts read at that revision (git show):
- docs/design/interactive-agent-communication-08-plan.md (1056 lines; sha256 b2a658f0...fdcfbd9, which matches handback output.sha256)
- docs/design/interactive-agent-communication/role-handback-step-08.json
- docs/design/roles/cards/planner.md, docs/design/roles/cards/_common.md
- REQ 01, EV 05, T-3426 / T-3356 / T-3689 / T-3357 task files

## Method (my own checks; I did not run or trust the producer's plan.py)
1. Hand-back integrity: recomputed sha256 of the output and all 21 inputs at the revision. All match (0 mismatches).
2. Graph: my own parser read the order table (section 6.1, 87 nodes), the "Depends on" column of every unit table (71 units),
   and compared them. Results: 87 unique nodes, 150 edges (matches the claim), no unknown node, unit-table deps equal
   order-table deps for all 71 units, Kahn's algorithm removed 87 of 87 nodes (no cycle), 0 order violations.
3. Coverage: R-n ids taken from REQ 01 (R-1..R-82 plus R-67.a1). Every R-n is served by a unit except R-38, R-40 and R-43.
   The plan explains each one, and REQ confirms all three: R-38 is "Later" (REQ 732-733), R-40 is merged into R-28 and
   R-43 is dropped (REQ 753, 781). E-1..E-173 (EV has 173 distinct ids): every one appears in at least one unit's
   "E-n served" column (0 missing). Card 3.6 items: section 11.3 covers all ten.
4. Drawings (card 6.4): D-1.1, D-1.2 and D-2 each have an id, a caption and a numbered text equivalent (5.5, 5.7, 6.4).
   I re-rendered all three Mermaid blocks with mmdc. Each exited 0, gave a non-empty SVG (169939, 383913 and 134550 bytes)
   and showed no syntax or parse error.
5. Rulings:
   - Ruling A (T-3426 2026-11-11: B-27 -> B-25 -> B-11 -> B-16, serialised in the main checkout, no worktrees) is kept.
     B-27 is baseline. P-4 -> P-6 -> P-7 are `serial` edges, and 3.4.c says no worktree is used.
   - The plan inserts P-1..P-3 before B-25 and builds B-16 before the full B-14. Both depart from the ruled sequence, but
     they are put to the operator as PD-3 and PD-1, not decided by the plan.
   - T-3689 9a (role-mail-only withhold) -> P-5. Ruling D (F-20) -> P-7 carries the interim ps veto, and P-43 (T-3688)
     replaces it only behind G-8.
   - NID: every removal is a decommission unit behind a decision gate (P-28, P-41, P-42, P-43, P-53, P-70).
   - Card 3.1 / 3.2: no work item created; supersede and re-scope are proposals only (7.3).
   - Card 3.3: explorations X-1..X-7 are operator GO/NO-GO.
   - Card 3.4: security edges are typed and enforced (5.1).
6. Acceptance and verification (card 6.2, 3.5): every unit has non-empty Acceptance and Verification cells. I then read
   each check against its unit.

## Findings (why not green)
F-1 (6.2, verification check vacuous). Six units declare "no E-n of its own in 05" (the E-n served column is "-"). Their
    declared verification check is still the generic template "fixture suite for the unit exits 0 with a PASS line per
    listed E-n, and its mutant list is all killed". With no E-n listed, that check passes trivially. The units are P-43,
    P-51, P-55, P-60, P-67 and P-68.
    - P-43 and P-60 have a second, real check (an M1 field run; a status field).
    - P-51, P-55, P-67 and P-68 have no other check. Their acceptance text is concrete (for example P-51: "stage and reply
      read by id across two hubs; unknown id answered as unknown"), but the check declared for it verifies nothing.
    - Needed: a unit-specific verification check for each, for example "fixture suite with a PASS line per acceptance
      clause, plus a negative control", or naming the FP 6.2.5.c(4) fixtures for P-55. For P-67 and P-68, name the
      check that will run on the specs that X-1 and the PH-6 entry produce.
F-2 (6.2, rubber-stamp risk). P-69 (request to AEF, send-and-wait): Acceptance is "no E-n...; AEF report" and the
    Verification is "AEF report on framework:pickup". Neither says what the report must show or what would fail it, so it
    would be ticked on any report. By contrast, P-44 and P-47 name parity fixtures.
    - Needed: a concrete criterion, for example "send-and-wait returns the reply by message id across two hubs using
      P-51; a timeout yields a stated outcome".
F-3 (minor, accuracy). Twenty-plus units say "see CR-8.3 where specs are missing", but CR-8.3 (14.3) only requests specs
    for R-36 and R-41.
    - Units such as P-23, P-31, P-35, P-42, P-46 and P-54 are not covered by any CR, so the pointer is misleading.
    - Needed: either drop the pointer, or say for each that no spec is needed (operations, reports, requests) and why.
F-4 (minor, accuracy). PL-3 (13.4) says "Four PH-2 units depend on AEF" but names three (P-44, P-45, P-46).

## Not findings
- Card 6.3 (mapping after approval) is correctly "not yet". Section 12 gives an idempotent rule.
- P-38's one Human criterion says what to look at (the W3 rollback runbook) and what fails it (a step the operator cannot
  do).

## Verdict
amber. Coverage is complete, there is no cycle, the drawings render and the rulings are honoured; all four are verified
independently. Condition 6.2 is mostly met: F-1 and F-2 must be fixed so that every unit has a real verification check
and no rubber-stamp criterion. F-3 and F-4 are editorial.
