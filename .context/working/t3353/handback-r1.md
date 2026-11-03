# T-3353 step 4 round 1 hand-back (architect)

1. ARC4 is now v0.2 (sha256 96659c76…08cc). It fixes only the three findings of V-20261103-717b2df9. 04a is unchanged.
2. Finding 1: 11.4 makes D-1's built-coloured subset the baseline view. 11.5 is a table giving, for the baseline, each of T1..T6 and the target, the TB-n introduced (+) or changed (~) and the [i] that shows the transition took effect. Two D-1 labels were corrected: the operator zone is TB-10 (was TB-12), and the lease is TB-12 [i].
3. Finding 2: section 15 has new rows for K-12, K-17, K-24 and K-25. 15.1 covers the built components and K-26..K-31, whose measurement is deferred to their sub-designs and must be added before each build starts.
4. Finding 3: section 18 has Status and Decision columns: all proposed, none decided, each tied to its O-n or to the step approval (18.1). 18.2 gives a disposition for all 18 FP 14.1.b items: BV composition, heartbeats, claim/transcript-complete and lease durations are built; R-74 sizing, the confirmation-provider interface and GP-11 are open.
5. Inputs were re-hashed and are unchanged. The review evidence was added as input REV1 (1.5 and the hand-back JSON). role-handback-step-04.json was rewritten and validates against role-handback/1.
6. All five drawings were re-rendered with mmdc: rc 0 for each, no parse errors (24.3).
7. No review or approval criteria ticked. T-3353 not closed. No new DRs, XRs or build items.
