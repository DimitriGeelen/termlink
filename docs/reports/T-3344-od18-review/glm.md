# GLM review — OD-18 (2026-10-06)

## Findings

**On Q1 (what the lease proves)**

1. [CHANGE] A sidecar-renewed lease proves the wrong thing: it proves a process is alive, exactly the signal CAND-6 ruled out ("last real hand-over time, never a heartbeat"). The 32,205-refusal sidecar would have held "main" indefinitely. Renewal must be gated on harness evidence — adapter READY plus recent real HANDED_OVER progress — reusing the existing back channel (CAND-6 6c), not sidecar aliveness.
2. [CHANGE] Renewal on traffic alone is also wrong the other way: a healthy idle main with no mail never hands anything over and would lose the role. Renew on adapter readiness; the open no-surfacing threshold (CAND-6 6d) becomes the idle tolerance and must be set. OD-6's PostToolUse hooks fire during long turns, so busy-but-working agents keep renewing without interruption.
3. [AGREE] Renewal by code, never an AI turn — round-3 consensus, keep.

**On Q2 (failover, flapping, split brain)**

4. [CHANGE] Busy must not trigger failover. Unresponsive-but-renewing is OD-3 STUCK plus OD-14 escalation, never role theft; failover fires only on lease lapse under the corrected renewal rule. Without this line, every long task flaps "main".
5. [CHANGE] Add a quiescence window before takeover (minutes, tolerates a hub restart — GLM round 1) with renewal jitter; takeover evaluated only at the home hub, one winner.
6. [CHANGE] "Checked wherever accepted" is too vague. Enumerate the fencing points: every home-hub state change made *as the role* (obligation accept/complete writes, escalation consumption, claim ops); expiry judged on the home hub's clock alone (CAND-13). Do not fence the old holder's OD-10-bound conversations — those stay with the instance (finding 8 depends on this).
7. [CHANGE] The old holder must be *told* it lost the role and stop answering role mail; fencing refuses actions but notification closes the window. Add voluntary release on clean shutdown.

**On Q3 (moving obligations)**

8. [CHANGE] Offered/unaccepted role-addressed requests move cleanly. Accepted-but-incomplete ones must not auto-move: the old holder may still finish them through its bound conversations (OD-10), and auto-move risks duplicate or contradictory execution. The new holder sees the obligation states (CAND-14) and owed list at session start; a conflict on an accepted obligation goes to the operator, not to guesswork.

**On Q4 (selection)**

9. [MISSING] Rule: operator pin first — appointment ≠ eligibility (round-3 item 48; self-declared eligibility is squattable, 055's M2), and eligibility must be attested by the authenticated home-hub card, not self-claimed. Otherwise deterministic order at the home hub: best reachability by CAND-6 fields, tie-break by stable start time. Yes, the operator must be able to pin.

**On Q5 (missing)**

10. [MISSING] The operator as a party: OD-14's last rung lands in *main's* session-start list — circular exactly when main is the failure. Vacancy or repeated takeover must escalate straight to the operator/cockpit; the failover loop needs a terminal human rung.
11. [MISSING] Zero eligible agents: resolve must return the distinct "role unassigned" state (round-3 item 47) and may *propose* an OD-15 Tier-2/3 start; never auto-start without a grant. Also cold start: who takes main first — appointment at project inception, not a race.
12. [MISSING] A second live instance of the same pid (OD-11 2d): two copies, each with its own "main". Role addressing needs the project-instance scope or role mail crosses copies; the same-id-in-two-places flag detects this but the recommendation doesn't resolve it.
13. [MISSING] Cross-hub resolve is silent: senders on other hubs reach the home hub via card exchange plus relay (OD-11/OD-1); specify that leg. Add the human-readable "who holds main" surface (055 item 40, GLM round 3).
14. [MISSING] Verify the claim/lease primitive before reuse — durable grants, ownership checks, restart safety (Codex round 3); the generation counter must be minted once and survive hub restarts (OD-12 pattern), not derive from anything rotatable.

**On Q6 (remove)**

15. [CHANGE] Don't fence per-hop or per-message: CAND-18 sequences already order conversations; fencing belongs at the home hub's role-state-changing endpoints only. Ship "main" alone first — a governed multi-role taxonomy is speculative weight. Pools later: agreed.

16. [MISSING] Tests: make the two motivating failures standing negative controls (CAND-3): lapse the holder mid-obligation (the 8 unanswered) and a wedged-harness holder that renews nothing (the 32,205 refusals) must both end in takeover-plus-operator-visibility, not silence.

**Verdict:** The skeleton is right and consistent with the rulings, but the lease as written proves liveness instead of can-take-a-turn, busy holders will flap or be stolen from, and the operator, vacancy, multi-instance and cross-hub legs — where the real failures lived — are all missing.
