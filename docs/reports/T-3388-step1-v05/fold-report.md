# T-3388 fold report: requirements v0.5

Document: `docs/design/interactive-agent-communication-01-requirements.md` (DRAFT v0.5, not APPROVED). Nothing committed. Source of the 20 change requests: threat model v0.3.7 section 21, register 22.18, and the T-3351 Decisions. The mapping table is section 14.1 of the document.

## 1 What changed, per change request

1. CR-1 (outage exception): R-2.a, R-2.e, R-7.e, R-14.o, R-15.o, R-26.o, R-44.a, R-45.a, R-45.e, R-58.a.
2. CR-2 (signed messages): R-34.e (2) (old "not required" quoted), R-34.f pointer, R-63.g; new R-75.
3. CR-3 (identity triple + digest): R-51.a, R-51.e, R-51.g.
4. CR-4 (as corrected): R-59.a, R-59.e, R-59.g. The sender's own counter is the source; the hub's highest-seen number only raises it after a restore.
5. CR-5 (activity veto): R-19.r, R-19.e, R-20.a, R-20.e, R-23.a, R-23.e, R-23.o (old "never" wording quoted as v0.4.1 text); new R-81 (busy bit visible to operator, own project, cockpit; not peers).
6. CR-6 (frame): R-50.a, R-50.e, R-50.g, R-47.a.
7. CR-7 (circuit trust): R-46.a (new item 4a to 4h and 5), R-46.e, R-46.g; lifetime also in R-7.e. No allow-list check at set-up.
8. CR-8 (admission): R-60.a, R-61.a, R-61.e; new R-77.
9. CR-9 (content on the hub, answered by B′): R-15.o, R-35.o, R-45.a; new R-80.
10. CR-10 (operator-class): R-63.a, R-63.e; R-76.a (7).
11. CR-11 (peers see yes/no): R-53.a, R-53.e, R-63.a.
12. CR-12 (resume check): R-65.a, R-65.e. R-67 and R-68 untouched by this one.
13. CR-13 (operator-signed authority): R-67.a (eligibility), R-64.a, R-63.a; new R-78.
14. CR-14 (as amended by OQ-14 D‴): new R-76; GP-11.d note in 9.22. PN-14 retired.
15. CR-15 (with OQ-12, OQ-13, OQ-17): R-1.e2, R-62.a; new R-74.
16. CR-17 (evidence rule, adapter report): R-24.a, R-47.a, R-51.a.
17. CR-18 (Tier 0 events): new R-72; R-64.a.
18. CR-19 (policy model, "may share history"): new R-73; R-63.a, R-64.a, R-4.o.
19. CR-20 (duplicate mains, forwarding): R-67.a (old "authority unknown" quoted), new R-67.a1, R-67.e, R-67.g, R-62.a, R-32.o, R-34.o.
20. CR-21 (TermLink-only delegation): R-4.o, R-47.a; new R-79.

Also done: status line and version row 0.5 (no APPROVED); 6.1.i convention; B′ and "purpose and intent over form" as 6.17.2 and R-80; section 11.7, 12.2; section 13 header and 13.5 (steps 3, 4, 5, 8: policy rule format and low-risk classes, high-impact list, reconciliation, T-3383, T-3384, T-3385, T-3386, T-3387); section 14 (table of 20 rows, CR-16 note, numbers table, source differences, inference list). New ids R-72..R-81 each have .a to .e and .g (.f where a "holds against" fits).

Numbers: PN-1 1 h, PN-2 30 min / 24 h, PN-13 1 h as [R]; PN-3..PN-12, PN-15, PN-17 as [R~]; PN-14 retired, PN-16 dropped. Table in 14.3.

## 2 Inferences ([P], [A], [R~]) the operator may strike

1. [P] R-75.a: a message that fails verification is refused and recorded.
2. [R~] R-51.a: the write-ahead claim mechanism across a crash (threat model 13.4, PR-31). The ruling accepted CR-17 and RR-17, not the mechanism's details.
3. [A] All new "Given/then" acceptance text restates ruled clauses as tests; some cases go beyond the ruling words (R-46.e, R-65.e, R-67.e, R-72.e, R-81.e). Stated in 6.1.i.
4. [A] Placement of numbers in requirements (PN-3, PN-6, PN-12 in R-46; PN-4, PN-5, PN-17 in R-59; PN-7, PN-8, PN-11 in R-50; PN-9, PN-10 in R-77). The rulings fix values, not places.
5. Priorities of R-72..R-81 are collector proposals ("P1 [P]").
6. R-7.e carries the OQ-3 lifetime and the OQ-4 answer (not a CR, but R-7.e is named by CR-1). R-34.f got a one-line pointer to R-75. Five "Open: ... (step 2)" items were replaced by pointers, each with the old text quoted.
7. R-72.a (5): a Tier 0 event on a human route expires after 10 minutes. The operator did not answer when asked; marked [R~].

## 3 Where sources disagreed

1. OQ-17: threat model 8.6.b.3 body says a dead holder's hand-over is "only the operator"; the ruling says Tier 0 event, policy-approved under B′ conditions. Used the ruling.
2. OQ-12 default: CR-19 (5) is silent on removal outside a profile; the decision record says Tier 0 event. Used the decision record.
3. CR-3 does not name the 14-day window; the decision record does. Used the decision record.
4. CR-14 text lacks "moving a class out of high-impact is itself high-impact" and the provider-change rule; the decision record has both. Used the decision record.
5. CR-7 says head compared "on every contact"; D5 record says "every tick". Both stated.
6. R-67.a (vacancy: operator pin, healthy incumbent, priority...) versus CR-20 (duplicates: operator pin, longest continuously READY, stable id). Different situations, kept apart; whether a healthy incumbent should beat a longer-READY copy in a duplicate is not ruled. Worth the operator's eye.

## 4 Not folded

1. CR-16: dropped (OQ-15 A). It appears only as "dropped" (12.2, 14.2). The clarification that the one-year "digests" are daily telemetry digests sits in R-51.a.
2. Step-2 rulings that are not CRs (OQ-1 adversaries ADV-11..14, OQ-2, OQ-5 store survival, OQ-15): left in the threat model for steps 3 and 4. Section 2.3 was not extended with ADV-11..14; the operator may want that.

## 5 Self-check

1. Each of CR-1..CR-21 appears in the document; CR-16 appears only on the lines 12.2 and 14.2, both saying dropped / not folded.
2. Word-level diff against v0.4.1: no deleted text other than the status-line phrases and the five "Open" items, each preserved in a bracket or quote. No `APPROVED` written for v0.5.
3. Section 14.1 has exactly 20 CR rows.
4. Drawings D-1 to D-3 not touched, so no re-render.

Report path: docs/reports/T-3388-step1-v05/fold-report.md
