# T-3351 step 2, review round 2: disposition of every finding

**Task:** T-3351 · **Role:** threat modeler (final revision round) · **Date:** 2026-10-07
**Document revised:** `docs/design/interactive-agent-communication-02-threat-model.md`, version 0.2 to 0.3
**Reviews answered:** `codex-r2.md` (Codex, not fit yet) and `glm-r2.md` (GLM-5.3, fit after three text fixes), stored unedited, with `comparison.md` section 6.
**Input unchanged:** `docs/design/interactive-agent-communication-01-requirements.md` v0.4.1 was not edited (`git diff` on it is empty). Where a mitigation needs a ruled requirement changed, it is a change request that names the requirement (section 21 of the threat model, with 21.2 for the fidelity decisions).

## 1 How the findings were handled

1.1 Every finding in both reviews was checked against the v0.2 text and against the requirement text it cites (R-2, R-14, R-15, R-19 to R-24, R-35, R-36, R-44 to R-47, R-51, R-58, R-62 to R-68). The reviewers disagree; I sided with neither by default. On the points where they disagree (is the disposition true, is the document faithful to the rulings), the text bears Codex out: GLM's "35 of 36 dispositions true" and "no silent re-scoping" are not upheld (section 4).
1.2 Verdicts: **valid** (fixed), **partly** (the valid part fixed, the rest answered with a citation), **not valid** (rebutted). **n/a** marks a sentence that is a confirmation or a summary and claims no defect.
1.3 Result, counted per finding row in sections 2 and 3: **25 findings: 23 valid, 2 partly, 0 not valid**, plus 4 rows that claim no defect (n/a). Two sub-claims are answered rather than followed (Codex 3a "refusal while unavailable" for ordinary delivery; Codex 4c "profile completeness"). No finding was rebutted as a whole. GLM 1b and GLM 3a repeat Codex 1b and are counted once each in their own table and fixed by one edit.
1.4 Section numbers below are those of threat model v0.3.

## 2 The three fidelity cases (the card's core rule: a threat model never weakens a ruled requirement by itself)

| Case | Ruled guarantee | Option chosen | What it is | Where |
|---|---|---|---|---|
| F1 Crash after hand-over and before its record | R-51.a "never handed over twice"; R-46.e "nothing delivered twice" | **(a) a mechanism** | One atomic, flushed claim per message before content leaves the store; close by transcript evidence; redeliver only after proof of absence at an evidence-complete point, otherwise wait ("hand-over uncertain"). Cost: waiting, not repeats. R-51 and R-46 are not changed. CR-17 adds the evidence rule and the adapter's evidence-complete point (an addition, not an exception). Version 0.2's "at least once" is withdrawn | 13.4, PR-31, SI-30, RR-17, CR-17, 21.2.1 |
| F2 Readiness-to-injection race | R-19.r, R-19.e (2), R-20.a, R-20.e, R-23.o "nothing ever typed into a busy prompt" | **(b) an explicit change request** | No mechanism today makes the check and the keystrokes one act, so the "never" cannot hold. CR-5 names R-19, R-20, R-23 and states the new guarantee; OQ-16 asks the operator; RR-19 says what remains; until ruled the rulings stand and the document says it does not meet them. PR-32 no longer reads the screen (R-22). SI-1 gains "inert in a shell" for the case where the harness exits in the window | 9.2.d, PR-32, SI-1, SI-2, CR-5, OQ-16, RR-19, 21.2.2 |
| F3 Per-message digest retention for one year | R-35.o (2), R-36.o rule the daily telemetry digests only | **Claim dropped; optional proposal** | The baseline no longer says a content digest is kept. PR-34 is an explicit optional proposal with storage, identity mapping, ceiling semantics and cost; CR-16 names R-35.o, R-36.o, R-51.a; OQ-15 asks the operator. RR-10 now states that without PR-34 nothing recognises a conflicting re-send after 14 days | 8.2, 8.2.a, SI-4, RR-10, TH-19, PR-34, CR-16, OQ-15, PN-16, 21.2.3 |

## 3 Codex findings (`codex-r2.md`)

| Id | Verdict | What changed, or the rebuttal |
|---|---|---|
| 1a (substantial corrections present) | n/a | Confirmation; no change |
| 1b (4d: TH-22 still says "accepted by default today"; split into exposures missing) | valid | TH-22 countermeasure (15.5.3) rewritten: "RR-6 states today's state; nothing has accepted it", and the two exposures (a) token holders, (b) hub operator are named there with OQ-7/PR-24 and OQ-11/PR-33 |
| 1c (1b: reasons missing on TH-25, TH-40, TH-41, TH-42) | valid | Checked: TH-25 impact had no reason; TH-40, TH-41, TH-42 likelihood had none. All four now carry a reason (15.5.11, 15.7.8, 15.7.11, 15.7.14). My own scan of every `L` and `I` line in 15.2 to 15.11 then found two more bare impact ratings that neither reviewer named (TH-38, 15.7.2; TH-49, 15.9.5); both now carry a reason, and a second scan finds none |
| 1d (SI-11, PR-26, SI-22, SI-21 overclaim) | valid | Rollback: 11.2.b, 11.2.f, 10.5.d rewritten (a stored counter is restored with the old list; the hub's live head detects it); SI-11 probe now restores *both* values. Checkpoints: 8.4.c, SI-22, RR-15. SI-21 and the 18.2 limit column: root with or without an operator key. SI-20 likewise |
| 1e (membership and hand-over: removal, epochs, fencing; TH-9 "fixed at creation"; crash CR) | valid | 8.6.b rewritten: join, leave, removal, hand-over, membership epoch on every turn, fencing of the old instance, the circuit limit (RR-5); SI-29, RK-3, CR-15, TH-9 (15.2.27), TH-61, 8.3 last row. The crash requirement change is CR-17 and 21.2.1 |
| 1f (retention: daily digests are not per-message digests) | valid | Codex is right: R-36 is the daily digest. See F3 above. 8.2, SI-4, RR-10, TH-19 corrected |
| 1g (TH-37 "first acceptance"; TH-18 "root-owned"; AC-2 detection claims) | valid | TH-37 (15.6.30) now says establishment, never first use. TH-18 (15.3.27) rewritten: not "storage the agent cannot write" but "a record that verifies", with the operator-key condition and the root limit. AC-2 (16.2.2) separates prevented, detected and not detected, and cites RR-7 and RR-15 |
| 1h (OQ-12 mixes admission and hand-over; first-claim entitlement unproven) | valid | OQ-12 now covers adding and removing participants only; OQ-17 is the hand-over. 10.5.b states plainly that first-claim entitlement is unproven and names the optional per-hub project-id list; RR-14 (b); OQ-8 text; CR-8 |
| 2a (blocking: R-51 and R-46.e weakened without an explicit CR) | valid | Resolved by option (a), not by the requested exception: the mechanism keeps both rulings. See F1. The reviewer asked for an explicit exception; the task allows a mechanism instead, and one is available because R-24.e already rules "no evidence, eligible again" |
| 2b (readiness safety weakened; harness exit puts the line in a shell; PR-32 screen vs R-22/R-47) | valid | Option (b), CR-5 naming R-19, R-20, R-23. Shell case: SI-1 inert-line rule, RR-19 (3). PR-32 reconciled with R-22: unsent input only from an adapter-reported field. See F2 |
| 2c (retention needs its own explicit proposal) | valid | PR-34 with storage, identity mapping, ceiling semantics (8.2.a), CR-16, OQ-15, PN-16 |
| 2d (CR coverage: CR-13 omits R-63; CR-1 mapping; CR-5/CR-6 vs SI-17) | valid | CR-13 names R-63.a. CR-1 now maps the outage exception to R-2.a, R-2.e, R-14.o, R-15.o, R-26.o, R-44.a, R-58.a. SI-17's evidence rule is now requested: CR-17 (R-24.a, R-47.a). CR-7, CR-8, CR-14 text updated for the new mechanisms |
| 3a (rollback is not defeated by restorable counters; signed grants and allow-lists too) | partly | Valid and fixed: 11.2.b (hub's live head; started or restored sidecar is unreconciled), 11.2.f (policy epoch and head for grants, allow-lists, roster entries), 10.5.d (roster sequence in every hub exchange), SI-11 and SI-21 probes restore both values; starts under stale grants are refused (stale rule). **Not followed:** "refusal while unavailable" for everything. Refusing ordinary delivery whenever the hub is unreachable would contradict R-7.a (the sidecar works without the hub). I refuse new trust (circuits, urgent mid-turn delivery, starts under a grant) and keep ordinary delivery, and I state the remaining exposure as RR-20 instead of hiding it |
| 3b (checkpoints protect only the anchored prefix; RR-15 wrong) | valid | 8.4.c states the anchored prefix and the unanchored suffix and adds coverage, timing and comparison rules; SI-22, TH-65, TH-15, 13.3.c, RR-15 (now two cases: the suffix, and a complete attacker) |
| 3c (an operator key does not contain host root) | valid | SI-20, SI-21, SI-13, CR-13, TH-18 and RR-2 now say the limit holds for the whole lifetime, with or without an operator key |
| 3d (approval replay: atomic consumption, executor binding, crashes, backups) | valid | SI-13 rewritten: digest includes executor and target; durable insert-if-absent before the effect; effect idempotent under the approval id; crash recovery completes under the same digest; a restored consumed-set is bounded by the expiry (PN-15) and listed as RR-20. TH-51 and TH-48 updated |
| 3e (RK-3 permits removal, SI-29 requires countersignature by the new one) | valid | RK-3 rewritten (self-removal, operator, OQ-12); SI-29 separates join, leave, removal and hand-over; epoch and fencing in 8.6.b.4 |
| 4a (RR-10, RR-15, RR-17, RR-19 need the corrections; RR-14; RR-12) | valid | RR-10, RR-15, RR-17, RR-19 restated on the corrected mechanisms. RR-14 split into (a) a mistaken approval and (b) first-claim squatting with no operator mistake. RR-12 now bounds receipt traffic (PN-17) and defines reserved capacity as a separate budget. RR-20 added |
| 4b (OQ-10 exposure; OQ-11 substituted keys; OQ-12 split) | valid | OQ-10 states the STORED-during-outage window under both options. OQ-11 states that a hub-served substituted recipient key defeats encryption at first contact. OQ-12 split (OQ-12, OQ-17) |
| 4c (verdict: not fit yet; rendering not independently verified; profile completeness) | partly | 6.4 deficiencies fixed as above. Rendering: all four blocks re-rendered with `mmdc` after the edits, exit 0, sizes equal to v0.2 because no drawing changed (section 24.2.0). Profile completeness: not something text can establish; 17.1.a already says so, no change |
| 4d (minimum repair) | n/a | A summary of the findings above; every item in it is covered by a row above |

## 4 GLM findings (`glm-r2.md`)

| Id | Verdict | What changed, or the rebuttal |
|---|---|---|
| 1a (35 of 36 dispositions true) | n/a | Not upheld as a conclusion: Codex found more than one false disposition (1c, 1d, 1e, 1g, 1h above), all confirmed against the text |
| 1b (TH-22 "accepted by default today" contradicts RR-6) | valid | Same fix as Codex 1b |
| 2a, 2b (round-1 fidelity fixes real; no silent re-scoping) | n/a | The round-1 fixes are real. "No silent re-scoping" is not upheld: three more cases existed (F1, F2, F3), found by Codex and confirmed by my own reading |
| 2c (SI-19 receiver-side cap is new, not "Existing (R-56)") | valid | 18.2 SI-19 row split: existing for the sender's cap (R-56.a binds the sender), proposal for receiver-side enforcement, quota, ceiling, reserved shares and the receipt bound |
| 3a (TH-22 remnant is also an internal contradiction) | valid | Fixed by the same edit as 1b |
| 3b (SI-11 says RK-1..RK-6 of seven scopes) | valid | The exclusion of RK-7 was deliberate; SI-11 now says so (a calendar date checked by R-71 itself) |
| 3c (PR-25 unexplained hole) | valid | 19.2 now lists PR-7, PR-12, PR-19 and PR-25 as unused |
| 3d (1.3.d / 1.3.e order) | valid | Reordered; 1.3.f added for this round |
| 4a, 4b (RR and OQ decidable from their own text) | n/a | Agreed; round-2 changes keep that property (section 5) |
| 5a (verdict: fit after three small fixes; optional SI-19, 1.3 order) | valid | All three fixes and both optional items done |

## 5 Self-check results (run before handing back)

5.1 `grep` for "accepted", "already", "first acceptance", "root-owned", "fixed at creation", "accepted by default": every remaining hit was read. The ones that matter are gone (TH-22, TH-37, TH-18, TH-9). What remains is historical version text (1.3, 7.2.d, version rows), statements about requirements ("R-7.e already says"), an incident fact (the 2026-10-06 duplicate sessions), "accepted" meaning a key or card being accepted, or the sentence "Nothing is accepted by this step and nothing is accepted by default".
5.2 Every R-id named in a change request was checked to exist in the step-1 requirements (55 distinct ids, 0 missing) and its text was read for CR-1, CR-5, CR-13, CR-16, CR-17.
5.3 Every TH, SI, BP, PN, RR, OQ and CR reference in the document resolves to a definition (0 undefined). RR-1 to RR-20, OQ-1 to OQ-17 and CR-1 to CR-17 have no holes.
5.4 Each residual risk and open question was re-read for the property "decidable from its own text": RR-10, RR-12, RR-14, RR-15, RR-17, RR-19 and RR-20 now state what accepting means in each case; OQ-10, OQ-11, OQ-12, OQ-15, OQ-16, OQ-17 state background, options with costs and a recommendation.
5.5 `git diff` of the step-1 requirements file is empty.
5.6 Drawings re-rendered (24.2.0): four blocks, exit 0 each.
