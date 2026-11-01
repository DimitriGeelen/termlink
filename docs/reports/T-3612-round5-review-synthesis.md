# T-3612 final outside review (round 2 of 2) on draft v0.8 — synthesis

Seats: claude, codex, antigravity, opencode (`docs/reports/T-3612-seats/round5-*.md`). Brief:
`.context/working/t3612/round5-brief.md`. All four: **ADOPT WITH THE LISTED CHANGES**.
Codex note: its run exited 1 on the framework integrity check T-3986 (it wrote result.md inside the checkout it
reviewed), so its verdict does not count as a pass; its review was recovered from its transcript and is used as input.
Opencode ran in plan mode and printed its review instead of writing it; the printed text is used.
This is the last round (cap 2). Every finding is dispositioned below; none goes round again.

## A. Blocking findings

1. **The product driver set cannot take effect through arcs' `scoped_drivers` (all four seats).** Global weights in
   `policy/value-drivers.yaml` win any collision (`estimator.py:2727-2736`), `arc-driver-review.sh` refuses D1–D4 as
   scoped drivers, scoped weights are capped at 6 (`arc.sh:1567-1572`; F-TRUST is 7), an arc holds at most 3 scoped
   drivers (`arc.sh:1624`; arc-011 already has 3), and `bvp.sh` ranks only from the live file (`:285-364, 476-500,
   854-889`). `bvp.sh:1503`, which the draft cites, is in `fw bvp driver --explain` only.
   **Disposition: goes to the operator** — it changes the effect of ruling A on the live driver file (v0.6), whose
   recommendation cited `bvp.sh:1503` as the delivery path. The agent's claim was wrong.
2. **Adoption would leave the charter contradicting itself (all four).** `docs/CHARTER.md:21-30` ("a message bus",
   "a strict star: spokes never talk peer-to-peer"), `README.md:68`, `.context/project/objectives.yaml` `out_of_scope`
   (incl. "system of record" vs the bounded authoritative record) and the topology invariant cited by the charter.
   **Disposition: ACCEPT.** §6.1/§6.2 extended to rewrite those passages in the same change (codex's wording for the
   hub record and circuits).
3. **arc-012 is ambiguous as a retire trigger (all four; antigravity blocking).** `arc-012.yaml` has `id: arc-011`
   (and `arc-011.yaml` has `id: arc-010`); `check-arc-claim-drift.sh` keys on `id`. **Disposition: ACCEPT** — the
   retire condition names the arc by file and anchor task (`.context/arcs/arc-012.yaml`, anchor T-3304); the id
   mismatch is filed as its own fix.

## B. Non-blocking, accepted (folded into v0.9)

4. Charter-sentence check: name the new anchor and the end word "stuck"; rewrite the test's fixture sentence, its
   README decoy and drift-case substitutions in the same commit (all four).
5. The purpose sentence states the intended contract; add after it "current coverage and remaining gaps are listed in
   section 2" (codex, claude; opencode finds it acceptable as is).
6. Recall retirement evidence: the prover must be identified and a successful run must cover all three behaviours;
   an allowlisted, not-run or CI-skipped result is not that evidence (codex, claude, opencode, antigravity).
7. F-ORCH's removal from the live file is stated (claude, opencode).
8. objectives.yaml: rewrite `out_of_scope:` to the §4 non-goals keeping T-3371's schema keys; correct "only T-3371
   and T-3477 cite O-n ids" (prose citations in T-3493, T-3515, T-3531, T-3533; none read mechanically) (opencode,
   antigravity, codex).
9. Tripwires: separate hub↔hub (directory cards allowed, topic relay forbidden) from spoke↔spoke (hub-minted circuit
   credential), and change assertions, not only comments (antigravity, codex).
10. §3 heading names the product file (opencode); D3's rationale no longer borrows DP-1's wording (antigravity).
11. P-2/P-3 "not yet" lists name five-level resolution, the fenced main lease (T-3359) and runtime attempt fencing
    (antigravity).
12. Scoring history: label calculations with the driver-set file and revision (codex).
13. DP-3 cap needs enforcement and a committed baseline list (claude, antigravity). Fact checked 2026-11-01: 99
    `check-*.sh` at the cap ruling; 100 now — the agent's T-3631 ratchet added one against the freeze.

## C. Deferred (risk stated)

14. F-SURFACE's overlap with D3/D2 (opencode, re codex round 1): kept as ruled; revisit at the first quarterly
    driver review. Risk: double-counting of "fewer verbs" work.

## D. What the operator decides next

- The blocking finding A1 (how the product set reaches scoring), as its own question.
- Then adoption of v0.9.
