## 1 Completion conditions 6.1–6.5

1a **6.1 met.** Section 14 answers all ten questions (six STRIDE plus CD/AD/RP/BG) for each of TB-1..TB-13 (14.2–14.14); cells with no threat say why (e.g. TB-2 CD "none while SI-1 holds").

1b **6.2 met.** Every threat in section 15 (TH-1..TH-57, including the out-of-order TH-56/TH-57) names a countermeasure (SI/PR/R) or a residual risk, or both.

1c **6.3 met.** Section 17 covers the routes the profile names — hub secrets and pins (BP-6), the OneDev/GitHub/release pipeline (BP-17), the cron canaries (BP-18) — and 17.2 states in one sentence per route what remains open.

1d **6.4 met.** SI-1..SI-28 each carry a concrete probe, framed as kill-checked negative controls per R-69 (18.1), which is exactly what step 3 needs.

1e **6.5 met.** D-1..D-4 have ids, captions and text equivalents (5.1.1, 16.1.1, 16.3.1, 17.3); the render check is recorded with method and results (section 24).

1f Cosmetic defects that break no condition: the PR index is numbered "19.2" but sits between section 21 and 22; TH-56 and TH-57 appear out of numeric order inside 15.2/15.3; PR-7, PR-12 and PR-19 are referenced nowhere (holes in the PR series).

## 2 Coverage

2a **Fleet admission (CAND-17): adequately analysed.** Section 10 covers roster admission, signed cards with sequence and TTL, home-hub binding, first-contact TOFU, registration caps; the enrolled-but-compromised hub is bounded (10.4) and residualled (RR-4), and the decision is correctly routed to OQ-8/CR-8.

2b **J4 same-host new conversation: analysed, but conditionally.** The option table (7.6) and the honest availability cost are good, and recommendation A is grounded in R-7.e (2). However the TB-6 STRIDE matrix (14.7) is answered *under recommendation A's assumption*; if the operator picks B or C, the S, E and CD rows of TB-6 need re-analysis. The document should say the matrix is conditional on OQ-4 A.

2c **Circuit credential lifetime: well analysed** (7.2–7.4), including the outage-survival versus revocation-window trade-off. One under-analysed case: credential minting is by the *receiver's home hub* (7.2.a step 3), so when the sender's hub is reachable but the minting hub is not, renewal and revocation differ from the symmetric "hub unreachable" treatment of 7.3 and 11.2. One sentence would close it.

2d **Missing or under-analysed items.** The card's example adversary "someone holding the approval device" is not explicitly modelled: ADV-5 is a hurried operator, BP-13/RR-13 cover the terminal, but a compromised operator device — or the future operator key C-7 stolen — has no dedicated scenario, and SI-13's digest binding does not catch it (largely gated on GP-11, but it should be named). Sender-side delivery of replies (roles swap, R-27) relies on TB-1's symmetry without saying so. Wholesale chain rewrite — a same-user attacker holding a hub token rewriting local log and hub record consistently (the TH-15 position) — is only detectable by the canary and reply deadlines; it is folded into RR-2 but deserves explicit mention at 13.3.

## 3 Fidelity

3a **Overall faithful.** Ruled requirements are cited, not edited; every mitigation that would change one is a change request (CR-1 for the R-45/R-7.e conflict, CR-2 for the missing signature requirement, CR-5 upgrading R-23's [P] clauses, CR-10 for the R-63 allow-list), and 1.3.c records the discipline.

3b **Two minor issues.** PR-9 (9.3.a) introduces a trust class "allowed peer" beside peer and operator-class; no ruling defines a second peer tier — R-63 rules own project plus operator only — and this extension rides inside CR-6 without its own sentence. And 15.2.15 (TH-5) states "until GP-11 is ruled, the rule is that nothing is operator-class… the allow-list… is empty in practice" as if settled, though it is CR-10/OQ-6 and undecided; the hedging used elsewhere ("these are proposals", 7.3) should be applied there.

## 4 Residual risks RR-1..RR-14

4a **Each is honest and correctly scoped.** All fourteen state the risk and what accepting it means in plain language; none is cheaply mitigable inside the model's constraints (RR-2 needs OS separation, OQ-2; RR-6 needs encryption or scopes, OQ-7; RR-1/RR-3 are inherent to LLM agents; RR-13 is open by design as the founding verbs).

4b **One residual is missing.** BP-19 (TOFU repair paths: today an agent with a shell can run `fleet reauth`/re-pin a rogue hub) is "open now, gated in the target", but no RR accepts the interim openness and 17.2's closing sentence does not cover it — BP-19's target is *gated*, not monitored or closed, and its present state has no detection. Either an interim control or a fifteenth residual is needed.

4c **Conditionals are handled adequately**: RR-5 names PN-1 and OQ-3; RR-11 names OQ-4; the operator is told what acceptance implies.

## 5 Open questions OQ-1..OQ-9 and the proposed numbers

5a **Options are fairly presented.** Each OQ carries options with costs; recommendations cite evidence rather than taste — 7.3.a ties lifetime B to the operator's own R-7 rationale, 7.6.c ties option A to R-7.e (2) and the single-authority argument, OQ-6 A prevents any message from claiming the top of the authority model, and OQ-2's C is honest about B being unmeasured.

5b **The numbers are properly tagged.** PN-1..PN-11 each state what they buy and what they cost, all [P], with the decision owner named (OQ-3 for PN-1/PN-2, step 4 for the rest, OQ-9 for the bundle).

5c **Nothing operator-only is decided.** Residual acceptance is explicitly withheld (20.1); nothing is "accepted" by this step. The one phrasing that reads as a decision is 15.2.15 (see 3b); D-4's use of target-state classes is explained in its caption and is acceptable.

## 6 Fitness as input for step 3 (security floor and phasing)

6a **Fit overall.** The invariants are probe-backed and phase-scoped ("hold in every phase in which their subject exists"), the L/I ratings give step 3 a ranking basis, and the coverage table (23.1) maps every handed topic to a section — I verified the step-2 row of REQ 13 against it.

6b **Before handing on, fix three presentational items**: the misplaced "19.2" PR index, the unexplained PR holes (PR-7, PR-12, PR-19), and the unstated sender-side symmetry assumption. None blocks step 3, but the PR index is what step 3 will cite, and unexplained holes invite mis-reference. Substantively, add the BP-19 interim residual (4b), the operator-device adversary (2d), and the conditional-labelling of TB-6 (2b); the rest can proceed as written.
