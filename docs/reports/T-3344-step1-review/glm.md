# T-3344 step-1 review — GLM-5.3



# Review of step 1 (requirements collector), arc-011-design

## 1 Completion conditions 6.2–6.5

1a Condition 6.2 is met in form: every entry R-1..R-43 carries statement, type, priority, source, verification and at least one acceptance criterion, with two qualifications. R-1.e(3) explicitly has no acceptance criterion for the operator leg (GP-11), and R-43.d/e are pointers to R-9 rather than criteria of their own.

1b Condition 6.3 is partially met. Adversaries are named stakeholders in 2.3 and D-1, but as an unconfirmed proposal (GP-0, 11.4). Three impossibility statements carry no "holds against" field: R-16 ("MUST NOT be raised before the store"), R-23 ("MUST NOT carry peer content") and R-20's "MUST be injected only if the prompt is free". R-2, R-15, R-22, R-24, R-28, R-29 and R-34 do name adversaries.

1c Condition 6.4 is met: Glossary (section 5), Conflicts (section 7) and Earlier requirements (section 10) are all present.

1d Condition 6.5 is met on presence: D-1, D-2, D-3 each have id, caption and textual equivalent (3.1–3.3). "Renders without error" is attested only by the output itself (11.6); the Mermaid is syntactically plausible, but this reviewer could not execute it, so rendering is unverified rather than confirmed.

1e Condition 6.1 is unmet as expected, and the output says so itself (1.4.c, 11.1).

## 2 Fidelity

2a Nothing is dropped: all of R-1.1..R-14.4 map in section 10. But several re-casts change meaning without being marked "changed" with a reason, which card 4.5 requires.

2b R-3.2 → R-7 is weakened in the statement itself: R-7.a adds "from a sender on the same host", a qualifier absent from the confirmed requirement. The disposition row says "narrowed acceptance only", but the narrowing sits in the MUST statement, not only the acceptance criterion — mislabelled.

2c R-7.1 → R-21 adds "A missing or unreadable flag MUST read as not ready", which comes from AEF's build (adapter.py), not from the confirmed text.

2d R-8.1 → R-23 adds "The line MUST NOT carry peer content" plus the claim-before-typing rule in R-23.e; both derive from the design record, not the operator's sentence.

2e R-13.1 → R-39 adds "MUST start from the deployment and not from a source checkout", an inference from RQ section 13, not the operator's words.

2f Smaller unmarked strengthenings: R-1.2 → R-2 adds a new impossibility clause ("MUST NOT be in transit with its stage invisible"); R-5.1 → R-14 adds "with a timestamp"; R-6.2 → R-18 adds the arrival-time tie-break. All are traceable to the record, but every section 10 row still reads "kept".

## 3 Neutrality

3a The OD sections quote all three weighted reviewers and 055/AEF where they spoke, keep the operator's contrary positions as options (OD-1.c.B, OD-2.c.D, OD-3.c.A, OD-7.c.D), and say plainly when reviewers did not address a question (OD-11, OD-12, OD-16). Recommendations otherwise track their citations.

3b Exceptions: OD-18.d claims "all four consulted parties agree on this shape" citing RV2 item 71, but item 71 concerns directory and liveness; the lease shape rests on items 43–45 — a slight overreach. OD-3.c.D and OD-4.c.D are synthesized options with no quoted source. OD-1.d recommends against the operator's confirmed read-back; it is transparently labelled ("only the operator can choose it"), so it is fair, but it is the one recommendation asking the operator to reverse himself.

3c Deciding the undecided: R-7.a's same-host scope partially decides OD-1/C-4 inside a "confirmed" requirement — the one place the output moves ahead of the operator. Everything else provisional is visibly labelled (2.3 proposal, NO_ACTION marked in D-3, R-43 merge proposed, glossary contested marks). Priority assignments P1–P3 are agent judgement, which the card permits.

## 4 Weakest acceptance criteria

4a R-1.e(3): no criterion exists for the operator leg (acknowledged as GP-11).

4b R-3.e and R-14.e: "within a stated bound (proposed …, not confirmed)" — no number, therefore untestable as written.

4c R-6.d/e: verification is "a review of size against a measure that the operator has not given"; the two-agents/two-sidecars criterion is true today and tests adjacency, not simplicity (GP-12).

4d R-37.e: the escalation threshold "is to be decided", so the pile-up path cannot yet fail a test.

4e R-38.d/e: "review" plus a property only a "later consumer" can exercise — not runnable in this build. R-36.e's "a missing digest is itself visible" names no surface where visibility is checked.

## 5 Fitness as input for the threat modeler

5a Usable overall: inputs are hash-pinned (1.2), trust gaps are explicitly routed to step 2 (GP-1, GP-2, GP-6), the adversary table cites documented evidence, and the security invariants that matter most (R-2, R-15, R-19, R-24, R-29, R-34) name their adversaries and carry negative-control tests.

5b Risks for step 2: the adversary list is a proposal (GP-0), so the stakeholder model it inherits is provisional; R-16, R-20 and R-23 lack "holds against" bindings; and with OD-1 open plus the R-7 narrowing, two architectures (hub-mediated vs cross-host sidecar API) with different trust boundaries remain live — the threat modeler must model both and should not read section 6 as having pre-selected the same-host reading. D-1 also omits the sender sidecar, the operator terminal and credential custody (A-5) as adversary touchpoints, understating the surface the next role must assess.

5c Recommended fixes before hand-off: restore R-7.a to the operator's unqualified wording and move the same-host scope into the acceptance criterion only; add "holds against" to R-16, R-20 and R-23; mark the additions in R-21, R-23, R-39 (and the smaller ones in 2f) as changes with reasons; give R-3.e and R-14.e confirmed numeric bounds or mark the bounds open.
