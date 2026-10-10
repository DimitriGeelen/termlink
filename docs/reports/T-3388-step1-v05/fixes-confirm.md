# T-3388 fixes after the v0.5 check: requirements v0.5.1

Document: `docs/design/interactive-agent-communication-01-requirements.md`, now DRAFT v0.5.1 (version row added, status line set). Source of findings: `codex-confirm.md`. Not committed. No ruling changed; finding 4a / 5f left alone.

## 1 Finding -> change

1. 1a. R-74.a (1): the join-waits-during-outage restriction is now [P] (not ruled). R-53.a: the four fields and last surface time go to the operator and the agent's own project only; the cockpit is removed (D7 does not name it; the record gives the cockpit only the busy bit, OQ-16). The R-81 busy bit is described as operator, own project and cockpit, not peers.
2. 1b. R-65.e: refuses an unsigned or unsequenced DEAD and a lower sequence; equal sequence is [P], step 3 decides. R-51.e: the crash-after-typed-delivery case is tagged [R~], like R-51.a.
3. 1c. R-79.a (2): "for governed work" removed; "not used and never built on".
4. 1d. R-67.a1 (4) and R-67.e: sender notice "delivered via <forwarding copy>; main is <copy> (generation N); resolve via the home hub".
5. 1e. R-60.a: CR-12 (signed, sequenced DEAD) and CR-18 (admission, re-home, key replacement, re-pin are Tier 0 events, R-72). R-61.a: CR-18 (hub admission and signing-key replacement). Section 14.1 rows updated for CR-1, CR-5, CR-12, CR-15, CR-17, CR-18, plus rows for OQ-1, OQ-2, OQ-5, OQ-15.
6. 2a. CR-1 outage exception aligned in D-1 (arrow 3), D-2 (stage write, STORED, ANSWER READY), D-3 (STORED label), 3.1.1.c, 3.2.1.c, 3.2.1.i, 3.3.1.g, glossary (Hub record, Message states), R-44.e.
7. 2b. CR-17: R-24.e, 3.3.1.d, 3.3.1.e, 3.2.1.h, D-3 (ATTEMPTED -> QUEUED only on proof of absence; new self-transition "hand-over uncertain, waits for the operator"), D-2 note, glossary (ATTEMPTED).
8. 2c. R-74.a (5) and R-74.e: fencing against the receiver's authoritative membership state, whatever epoch the turn carries; outage limitation stays in R-74.g.
9. 2d. CR-5: D-1 arrow 4, D-2 (re-check, veto, inert line), D-3 (QUEUED -> ATTEMPTED), 3.1.1.d, 3.2.1.e, 3.2.1.f, 3.3.1.c, glossary Doorbell now [R] (R-23.a). Also glossary Home hub (signed, sequenced DEAD).
10. 3a. Section 2.3: ADV-11..ADV-14 added as ruled (OQ-1 A, 2026-10-07), threat model 4.3 wording; D-1 adversary node; 11.4.
11. 3b. New R-82 (OQ-2: C now, B committed target: isolated account per agent, narrow privileged helper, each creation a human-approved logged Tier 0 act, host root a permanent residual). Pointers in R-72.g, R-78.g; handoff in 13.5.1 (step 3 phasing and price, group access to shared checkouts) and 13.5.2 (step 4 helper design). 6.17.1 and 14.1 mention R-82.
12. 3c. R-15.o durability floor (F-1, F-2 survived with a flush before STORED; F-3..F-5 detected and shown; F-6, F-7 refused or detected); acceptance text in R-15.e, verification in R-15.d; step 3 pointer in 13.5.
13. 3d. 14.5.6 rewritten (OQ-15 is folded by 14.2 and R-51.a); revisit trigger (re-sends older than 14 days in the first-build measurements) added to R-51.a and 14.2. fold-report section 4 item 2 given a "Corrected in v0.5.1" note (the only edit outside the requirements document; item 10 of the brief asked for it).
14. 4a / 5f. Not touched. 14.4.7 added: "Open for the operator: precedence of R-67.a incumbent protection vs R-67.a1 duplicate resolution (T-3388)".
15. Added 14.5.7 (new [P] items) and 14.6 (list of these fixes).

## 2 Diagrams changed (orchestrator re-renders)

D-1 (adversary node, arrow 3, arrow 4), D-2 (stage write, STORED, typing branch and its else branch, HANDED_OVER note, ANSWER READY), D-3 (STORED label, QUEUED -> ATTEMPTED, ATTEMPTED -> QUEUED, new ATTEMPTED self-transition). I rendered all three with mmdc and chromium: exit 0, no syntax error text (SVGs in /tmp/mm, not kept in the repo).

## 3 Grep check of the document

1. "authority unknown": R-67.a, R-67.e, R-34.o carry it only as quoted v0.4.1 text marked replaced by CR-20; section 10.2 row for R-34 is v0.4 history. OK.
2. "not required": R-34.e (2) quoted v0.4.1 text, R-75.a/c state that it was replaced, R-50.g "not required of TermLink senders" is the v0.4.1 status line. OK.
3. "nothing is ever typed": only R-19.r (quoted v0.4.1 text) and OD-2.R (section 9, ruling record). OK.
4. "busy prompt": 2.2.c (asset description), 4.2.12 (v0.3 answer), R-19.r and R-19.e (quoted v0.4.1 text), R-20.c (rationale), 7.x/9.x (conflict and ruling records, v0.4), 10.2 R-19 row (v0.4 history), 3.2.1.e (quoted). OK; sections 7, 9 and 10 are v0.4 records and were not rewritten.
5. "evidence window": 3.3.1.e (now says it never alone authorises redelivery), 3.3.1.h, R-24.e (rewritten). OK.
6. "hub record first": R-14.o, R-26.o, R-45.a (each with its CR-1 exception), 3.2.1.c (exception added), sections 9 and 10 (history). OK.
7. "eligible again": 3.3.1.d (quoted v0.4.1 text) and R-24.e (quoted v0.4.1 text). OK.

Report path: docs/reports/T-3388-step1-v05/fixes-confirm.md
