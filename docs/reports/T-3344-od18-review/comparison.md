# OD-18 review — comparison (Codex, GLM), 2026-10-06

Brief: `brief.md`. Answers: `codex.md` (16 findings), `glm.md` (16 findings). GLM read copies of the requirements,
the interview log and the round-3 comparison (its sandbox cannot read the repository).

## Both reviewers
1. A sidecar-renewed lease proves the wrong thing (process alive); renewal must be gated on readiness / ability to
   deliver, or the 32,205-refusal sidecar would hold "main" forever (Codex 1, 3; GLM 1).
2. Busy is not dead: a working-but-slow holder keeps the role; slowness is STUCK/escalation, not takeover; add
   thresholds, quiescence/cooldown against flapping (Codex 2, 3; GLM 2, 4, 5).
3. Fence at role-protected state changes at the home hub (obligation writes, escalation, claims), not per message;
   losing "main" must not invalidate the old holder's conversation-bound replies (Codex 4, 7; GLM 6, 15).
4. Generation minted durably, never reused after restart or restore (Codex 6; GLM 14).
5. Do not blindly move accepted or half-done work on failover; classify, reconcile, uncertain to the operator
   (Codex 9; GLM 8).
6. The operator can pin a holder (Codex 12; GLM 9); deterministic selection at the home hub (Codex 11; GLM 9).
7. Zero eligible agents: a visible vacancy, escalation to the operator, start only under an OD-15 grant
   (Codex 14; GLM 11).
8. OD-14's last rung lands in main's list, which is circular when main is the failure: a human fallback is needed
   (Codex 15; GLM 10).
9. Cross-hub resolution via verified project and home-hub identity (Codex 14; GLM 13); a visible "who holds main"
   (Codex 15; GLM 13).
10. Ship "main" alone first; general roles and pools later (Codex 16; GLM 15).

## Only one reviewer
11. Codex: some external effects cannot be fenced (5); bind acceptance and obligation ownership atomically (10);
    keep a healthy incumbent, no automatic preemption when a preferred agent recovers (11, 12); strict vs
    preferred pin (12); audited operator override (15).
12. GLM: tell the old holder it lost the role, voluntary release on clean shutdown (7); first holder appointed at
    project setup, not a race (11); two live copies of the same project id each grabbing "main" (12); the two
    motivating failures as standing negative controls (16).

## Disagreements
13. Selection criteria: Codex — operator-configured priority, then stable ids; GLM — operator pin, then best
    reachability (CAND-6), then start time. Compatible as an ordered rule.
14. Obligations: Codex — classify every obligation on failover; GLM — unaccepted ones move, accepted ones stay with
    the old instance and conflicts go to the operator. Compatible: GLM's split, Codex's classification when the old
    instance is dead.
