# T-3354 step 5 hand-back for the orchestrator (evidence specialist, v0.1)

1. Output: docs/design/interactive-agent-communication-05-evidence.md (1420 lines) + 05a-coverage.md (generated, 0 undeclared gaps); record docs/design/interactive-agent-communication/role-handback-step-05.json (schema OK). Commits d8bec74e1..27d4cfb19 and the hand-back commit.
2. Specs: 152 (E-1..E-152). By category: adversarial 58, core function 50, error handling 15, integration 15, edge case 10, user workflow 4.
3. By level (a spec may name two): L1 property/model 4, L2 hermetic 59, L3 fuzz 5, L4 integration 57, L5 live drill 11, L6 chaos/recovery 9, L7 production check 33 (L7 added: the card's list has no level for checks in operation).
4. PH-2 first: doorbell D+ and 300 s watchdog E-1..E-14, activity meter E-15..E-19, lease tightening + waiting-mail E-20..E-31, forward-once E-32..E-35, hub id/key/revocation E-36..E-43, blobs/keys E-44..E-49, caps E-50..E-54, status/CLEAR E-55..E-61, injectors E-62..E-65, hub changes E-66..E-70, vendor/SendMessage deny E-71..E-74, W3/PH-2 exit E-75..E-83, adapter parity E-84..E-85.
5. Test estate per CAND-3 tier (section 3): tier 1 testkit estate, sim clock, fx-a/fx-b(/fx-c); tier 2 throwaway hub(s); tier 3 testdev-real + testdev-peer (EP-5, needs T-3361); tier 4 W1/W2 pair claude-termlink + claude-termlink-alt (+ main session), canary pair testdev-canary-a/b, W3 population, later 3-agent and cross-host pairs.
6. Coverage gaps: 6a pseudo-code FAILS CLOSED WHEN: open (no step-6 output yet); 6b declared: R-36 deferred PH-6, R-38 deferred, R-40 merged into R-28, R-41 deferred, R-43 dropped; 6c not runnable yet: all tier-3 specs and the canary (testdev pair not built), cross-host (PH-3), VM host crash (EP-12 not provisioned).
7. Architecture fail-closed conditions AFC-1..AFC-34, each with a spec (section 13).
8. Limit probes recorded as LIMIT, never pass: RR-5, RR-7, RR-10, RR-15, RR-19/AR-2, RR-20, AR-1.
9. Open for the OPERATOR (plain language, not blocking this step):
9a. Which Claude subscription the testdev account uses; tier-3 tests and the daily canary cannot run until it has one.
9b. About 28 always-on production checks are proposed; they are planned as one combined check plus status fields so the 99-check guard cap holds, unless you prefer to retire other checks.
10. Change requests: YR-1 (step 3) add H-11 (e),(f) and a bound for unsubmitted typed lines; YR-2 (step 3) split H-24 forward part into PH-2; YR-3 (step 4) say how ARC4 15 checks fit the guard cap; YR-4 (step 4) three ambiguous boundaries (back-off schedule, provisional claim and last_human_at, 15 s lease freshness for forward-once).
11. What step 6 must answer (section 14): 11a FCW-n list with routine and AFC refined; 11b back-off schedule as a function; 11c READY age measured on which clock; 11d "owed answer" for conversation=agent; 11e provisional claim and last_human_at; 11f 15 s lease boundary; 11g which revocation RPC failures are a failed pull; 11h test seams and their production refusal; 11i where the termlink.evidence/1 record is emitted.
12. Not done by me: no review, no approval tick, T-3354 not closed.
