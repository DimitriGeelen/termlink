# GP-12 review — comparison (Codex, GLM), 2026-10-06

Brief: `brief.md`. Answers: `codex.md` (16 findings), `glm.md` (16 findings). GLM read copies of the requirements and
the interview log.

## Both reviewers
1. The five proxies measure packaging, not what failed: failures 1 (misfiled key), 3 (waker exclusion) and half of 5
   (wrong hub at runtime) pass all five (Codex 1, 2; GLM 1-3).
2. The status call must be truthful, not presentational: separately verified fields (receiver, canonical hub, inbox
   ownership, signing identity, adapter, last real hand-over), each with freshness and explicit unknown (Codex 5;
   GLM 4).
3. Count cooperating components that must be alive (processes, schedulers, hooks, independently failing threads),
   not OS processes (Codex 4, 8; GLM 5).
4. Line counts are not comparable across shell, Python and an in-binary sidecar and are gamed by moving code: a
   growth tripwire within one implementation over everything it owns, never acceptance (Codex 13, 16; GLM 6, 9, 15).
5. API-call counts are gamed by mode-flag calls: inventory operations derived from the ruled contract (Codex 12;
   GLM 10).
6. Missing: a state and persistence model (stores per agent), identities/keys per agent as an invariant, crash
   recovery with zero manual steps, measured time to diagnose (Codex 14, 15; GLM 11-14).
7. Every measure tied to standing fault injection of the real failures (Codex 7; GLM 7).
8. Keep one sidecar per agent (R-6 confirmed); per-host only by operator amendment after comparison (Codex 9, 10;
   GLM 8).

## Only one reviewer
9. Codex: exclusive inbox ownership with fencing (3); a bounded end-to-end probe through the real delivery path
   without an LLM turn (6); install tested in a clean environment with upgrade, rollback, restart (11).
10. GLM: hub binding checked continuously, not only at install (3); a cap on the number of message states (16);
    budget size per function (15).

## Disagreements
11. None of substance. Both: "keep the proxies as review aids/tripwires; make verified behaviour the acceptance."
