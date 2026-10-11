# Review: `interactive-agent-communication-03-floor-and-phasing.md` (DRAFT v0.1, T-3352)

## 1 Verification performed (basis of the findings)

1a Input hashes in 1.1 and 1.3 recomputed: all match, including REQ, TM, the two task decision records, `_common.md`, card, adapter, profile, chain yaml, schema, interview log, and `CLAUDE.md` lines 1–2820. The GP-12 folder (1.3, "GP12") exists with both files.

1b The host readings of 1.4.d reproduce now: `testdev` uid 1001 exists; `/opt/termlink`, `.git`, `.tasks`, `.context` are `root:root` 775; `core.sharedRepository` unset; `useradd`/`sudo`/`setfacl` installed; claude-mentioning process lines split ~80 root / 16 other at review time — consistent within the "one moment" caveat of 15.5.3.

1c Counts recomputed by hand: 33 security-typed requirements (3.5) matches REQ line "b" typings; 4.3's 35/34/6/1/3/2/1 = 82 matches table 4.2 row by row; floor = 39 (31 SI + 8 FX); 13 state values + 7 markers; 17 FC rows; 28 hypotheses (11 TM + 17 new); 24 rows / 21 dimensions (section 8); 20 operator questions. Enters/Complete columns of 3.3–3.4 agree with 6.2.3.c–6.2.6.b and with drawing D-2's edges and 3.8. D-1 and D-2 carry id, caption, numbered text equivalent (3.8, 6.8) and a render check (section 16).

1d The dependency statuses of 6.6 match the task files (T-3383 captured/later; T-3384, T-3386, T-3387, T-3347 captured/next; T-3385 started-work, owner human; T-3370 started-work).

## 2 Role-card fidelity (brief item 1)

2a Met. Card 6.1: every SI-n is a floor row (3.3); all 33 security-typed requirements are floor or listed with a reason (3.5, 3.6); the six P-standing items are named for confirmation (OP-10); deferrals are risk-acceptance requests RA-1..RA-8. Card 6.2: every row of 4.2 has a class, a phase and a reason. Card 6.3: all seven phases have scope, exit criteria and rollback; H-14 and H-21 are marked REFUTES THE APPROACH (9.1.b). Card 6.4: both drawings present and checked.

2b One defect, a probe whose prerequisite its own PH-0 does not produce — see 4b.

## 3 Ruling fidelity (brief item 2)

3a The eight named rulings are handled faithfully: OQ-2 (10.1.1 restates the target, helper, Tier-0-per-creation and permanent root residual; placement is OP-1); OQ-3 (PN values and the H-1 re-rule path, 6.2.6.c); OQ-8 D″ (exception classes C-2b/C-4/C-7/C-8 always human; first ruled policy class C-10 kept "ruled"; re-pin record T-3384 in PH-0); OQ-12 (the five ruled profile settings verbatim in 10.3.3; the sixth, `eligible_for_main`, is a loud [P] with CX-3 and OP-4); OQ-14 D‴ (SI-31; IC-2 double approval, 10-minute expiry, no silence); OQ-16 (SI-2 veto; W2 gated on T-3387; busy bit not shown to peers); RR-8/R-67 precedence (6.2.5.a (3) follows CR-20 and ruling D, no contradiction); RR-13/CR-21 (FX-1; the "disable" half is deferred to PH-4 not silently but through RA-5/OP-19, which is the card-sanctioned route for deferring floor work).

3b **Finding (C-15 batch).** 10.2.2 C-15 proposes "one event per migration wave, digest over the named set", while the ruling says "every account creation is a Tier 0 act: human-approved and logged" (R-82.a (3), OQ-2 in T-3351 Decisions). A batch digest is a relaxation of "every", it carries no [P] tag, and it is in neither OP-2 (policy-route list) nor OP-3 (high-impact list — C-15 is HI "no"), so no operator question rules on it. **Needs an operator decision.**

3c **Finding (SG and NK unconfirmed).** The ship gate (2.3.c/3.1.b), no-knob-before-its-lock (2.3.d/3.1.c), rollback-keeps-the-floor (3.1.d), status honesty (3.1.e) and the deferral rule (3.1.f) are new normative mechanisms that bind every phase and drive real sequencing (e.g. SI-20/SI-21's fixed-default form in PH-2). OP-10 asks confirmation of "the 31 invariants and the 8 extra items"; OP-11 of "the seven phases, the BV and the CAND-3 staging". Neither names the five floor rules. **Needs an operator decision** (smallest form: add "and the binding rules 3.1.b–3.1.f" to OP-10 A).

## 4 Internal consistency (brief item 3)

4a **Finding (PH-3 exit uses a PH-4 capability).** 6.2.4.c (2) requires "the policy route for the ruled class, recorded" as a PH-3 exit test. The only ruled policy class is C-10 (10.2.3), whose subject — a dead holder's hand-over with a same-role R-67 lease successor — cannot exist before PH-4 (FX-3, SI-29 and the lease itself are PH-4 scope, 6.2.5). At PH-3 the exercisable policy classes are C-2a and C-6a (the signing key arrives PH-2, 4.2 R-61). As written the exit criterion is untestable in its phase. Editorial fix: point the test at C-2a/C-6a, or say C-10's route is exercised by fixture.

4b **Finding (FX-5 probe depends on a baseline PH-0 does not take).** FX-5's probe (3.4) and RA-3.b's stand-in rest on "never above the PH-0 baseline" of plaintext content fields on the hub, but the PH-0 baseline list (6.2.1.a (2), exit (4)) covers message sizes, agents per project, blob sizes, transcript share and re-send age — not hub plaintext exposure. The probe is un-runnable as written from PH-2. Editorial fix: add that baseline to PH-0.

4c **Finding (minor, "last of the P1 items").** 4.4.c and 11.2 say R-7's independence "arrives last of the P1 items". R-76 (P1) completes only at PH-6 (real provider) and R-1's operator leg (P1) is deferred to GP-11, both later than PH-5. The substantive point (the cost of the ruled OD-1 C order) stands; the sentence overstates. Editorial fix.

4d Otherwise clean: I traced every floor item's Enters/Complete against the phase sections and D-2 (no mismatch), every phase's scope against 4.2's phase column (no capability used before delivery apart from 4a), and the numbers (FP-2's 13+7, the PN-n values, deadlines, H-3/H-4/H-6/H-7/H-12 thresholds against FP-4's headroom rule) against REQ 14.3 and TM 19 — no disagreement.

## 5 Open questions (brief item 4)

5a Genuinely the operator's, one at a time (13.1), each with options A–D and a recommendation; OP-13..OP-20 correctly split the eight risk acceptances into single questions. The four items the hand-over named are OP-1..OP-5.

5b Missing from the list: the two decisions behind 3b and 3c (the C-15 batch digest; explicit confirmation of the floor rules). Nothing else I could find: FP-1..FP-4 are OP-6/7/8/12; the six P-standing SIs are OP-10; E-7 rides OP-11; RR-21 is OP-19 plus CX-1; the T-3383 horizon change is correctly routed to step 8, not the operator.

## 6 Would step 4 be misled? (brief item 5)

6a Nothing material beyond 4a and 4b: the C-10 seam could make step 4 design a PH-3 route test that cannot run, and the missing baseline silently weakens the only FX-5 probe. Positively: 14.1.b is a complete and accurate hand-over list, CX-2 correctly surfaces the real R-48.e vs R-63.a ambiguity rather than resolving it silently, and 14.1.a states the floor's status as constraint, which is the right frame.

## 7 Verdict

**Fit after listed fixes.** The floor is complete and traceable, the phasing is internally consistent except at one seam, the rulings are respected with two exceptions that need the operator's eye, and the coverage machinery (counts, drawings, probes) checks out against both inputs of record.

## 8 Fix list, smallest first

8a Editorial (4c): reword 4.4.c and 11.2 ("last of the P1 delivery items; R-76's provider and R-1's operator leg arrive later").

8b Editorial (4a): in 6.2.4.c (2), name C-2a/C-6a as the PH-3 policy-route test, or mark the C-10 route as fixture-tested.

8c Editorial (4b): add "plaintext content fields readable on the hub" to the PH-0 baselines (6.2.1.a (2), exit (4)).

8d **Operator decision** (3c): extend OP-10 option A to cover the floor's binding rules 3.1.b–3.1.f (SG, NK, rollback-keeps-floor, status honesty, deferral rule).

8e **Operator decision** (3b): either one Tier 0 event per account creation per R-82.a (3), or add the batch-digest form to OP-1's options so the operator rules on it explicitly.
