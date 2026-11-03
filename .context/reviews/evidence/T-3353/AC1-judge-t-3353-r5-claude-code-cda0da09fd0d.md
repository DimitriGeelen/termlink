# T-3353 AC1 — independent review, step 4 architecture v0.3

Reviewer: claude-code seat (rung-5-panel:claude-code), dispatch judge-t-3353-r5-claude-code-cda0da09fd0d
Revision reviewed: 153b8b4cdf0c48543518af50f276eb86300c534e (HEAD at review time, same)
Subject: docs/design/interactive-agent-communication-04-architecture.md (v0.3, sha256 d1b63c4b…003a9c),
04a-traceability (sha256 c516a290…d4667), hand-back role-handback-step-04.json.
Yardstick: docs/design/roles/cards/architect.md section 6 (completion conditions 6.1–6.5), plus fitness as input
to step 5 (cards/evidence-specialist.md, sections 2–3).
Note: all earlier evidence in this directory reviewed v0.1/v0.2. This is the first review of v0.3 (rulings O-1..O-7).

## Integrity checks
- Hand-back output sha256 = file sha256 (d1b63c4b…). All 8 hand-back input hashes recomputed: all match.
- 04a hash matches the one the hand-back summary cites (c516a290…).
- Commits cited as ruling sources exist: c04ffb160, bd9380d10, 0c57b7b51, 6078601dd.

## Card 6.1 — every R-n, TH-n, SI-n addressed; no unexplained element
- Scripted: ids collected from 01/02/03 vs ids in 04+04a. R-1..R-82 (82), TH-1..TH-65 (65), FX-1..FX-8, BP-1..BP-20:
  none missing. SI: only SI-32 absent, and SI-32 exists only as a [P] suggestion inside FP CX-1 (a change request to
  step 2), not an established invariant; its subject (BP-20, hook wiring/hash check) is covered (14, 18.2.3, 12.6).
- New v0.3 elements K-32, K-33, K-34, DR-15, DR-16 appear in 04a. B-25..B-29 are in the build table (section 22).
- FINDING F-1 (see below) is an element that contradicts a ruling, so 6.1 is not fully met.

## Card 6.2 — baseline, transitions, target with boundaries and instrumentation
- Section 4 baseline; 5 target (D-1, D-2); 11 transitions (D-5, 11.2.x, 11.5 TB-n and [i] per transition); 12 later phases. Met.

## Card 6.3 — every bypass route removed or monitored
- Section 14 lists BP-1..BP-21, each with a design element and "removed"/"monitored"/"open by design (RR-n)". Met.

## Card 6.4 — open decisions as decision records with options + recommendation
- Section 18: DR-1..DR-16, each with context, options, recommendation, consequences, status, decision (card 3.8).
  Accepted ones cite the ruling and commit; DR-6 marked superseded by DR-15; 18.1 lists those still proposed;
  18.2 disposes of each FP 14.1.b item. Met. (Cosmetic: the section 18 heading still says "all proposed".)

## Card 6.5 — D-1..D-5 present with id, caption, text equivalent, render without error
- Captions at 5.2, 5.4, 7.6, 10.1, 11.1 with ids. Text equivalents present (e.g. section 4 for the D-1 baseline subset).
- Independent re-render: extracted the 5 mermaid blocks; mmdc with /usr/bin/chromium --no-sandbox:
  all exit 0; SVG sizes 50690, 26237, 43920, 81624, 24856 bytes — byte-identical to the doc's 24.4 claim; no
  "syntax error"/"parse error" in logs or SVGs. Rendered D-4 to PNG and viewed it: three state machines, every
  transition labelled, PROVISIONAL state and HANDOVER_UNCONFIRMED label present as 24.4 says. Met.

## Rulings applied (v0.3 purpose)
- O-1: 6.4.6, DR-5, BP-21. O-2 D+: 6.9.0–6.9.7 (READY + 30 s no typing + no conversation; 300 s watchdog K-33 with its own
  audit row 6.9.7.c = CR-O2b), 6.14 activity S1–S3 and conversation signal separate from READY (CR-O2a, CR-O2c).
- O-3: 6.13.1 prompt origin per harness (CR-O3 a), 6.13.3 fast takeover with bound (b), 6.13.5 waiting-mail signal
  and where it lands (c). O-4: 6.10.2, B-18, B-26. O-5: section 8 package. O-7: 8.4, DR-13.
- O-6 OP-12: 15.2 reliability thresholds, H-21 dropped (XR-7). OP-22: section 16 corrected to OD-7/OD-9 (CR-O6c), section 17 contract.
- O-6 OP-5 (CR-O6a): applied in 3.4.a, K-27 row, 12.2, 18.2.17, XR-8 — but NOT everywhere (F-1).

## FINDING F-1 (amber): residual "approve verb" contradicts the O-6 OP-5 ruling
The operator ruled (T-3353 Decisions, O-6 OP-5): approvals are "NOT a new hub-admin verb", they go through Watchtower
and the OOB channels; CR-O6a asks step 4 to replace the hub-verb placement. The doc's own 12.2 says "no new hub verb is
added", and XR-8 asks step 3 to remove the verb "the same for the PH-3 scope item ... wherever it names the verb". Yet
the doc itself still names the verb in four places:
- line 1012, drawing D-5 node T4: "Tier 0 registry and approve verb"
- line 1038, 11.2.5 (T4, PH-3): "the Tier 0 registry with the approve verb"
- line 1070, 11.5 table row T4: "~TB-10: Tier 0 registry and approve verb"
- line 1081, 12.1: "Enrolment: C-1 human route through the approve verb with the typed-back fingerprint"
Why it matters for step 5: the evidence specialist writes adversarial specs per transition and per TB change; D-5 and
11.5 would lead it to specify evidence for a hub approve verb the operator ruled out (and the TB-10 change of T4 is
stated in terms of it). Fix is small: reword the four places to "Tier 0 registry; decisions via Watchtower/OOB written
back through the existing admin path (12.2)", re-render D-5, update 24 and the hand-back hash.

## Minor (not blocking)
- Section 18 heading "(all proposed; the operator decides)" is stale; 18.1 states the actual status correctly.

## Verdict
amber — conditions 6.2–6.5 met with evidence; 6.1 and fitness for step 5 are met except F-1, a ruling not applied
consistently in four places (one inside a required drawing).
