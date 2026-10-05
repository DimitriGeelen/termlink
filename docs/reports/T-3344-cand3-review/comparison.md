# CAND-3 review — comparison (Codex, GLM), 2026-10-05

Brief: `brief.md`. Answers: `codex.md` (16 findings), `glm.md` (13 findings). GLM's first run read nothing (sandbox
refused /opt/termlink) and was rerun with the two files copied into its working directory.

## Both reviewers
1. Keep the tiers including the one-agent tier (Codex 1; GLM 1).
2. Select per FAILURE MODE, not per requirement; justify each test by the failure mode it observes (Codex 2; GLM 2).
3. Tier 1 overclaims: stage memory/dedupe (hub, survives restart), consent with real keys, sweeping, resend need
   real components (Codex 3; GLM 3, 4).
4. Hub id is not binding: a stray hub reports its own id truthfully; assert session, inbox, sidecar and sender all
   resolve to the intended hub, with a second-runtime-dir control (Codex 4; GLM 6, CAND-8).
5. Missing dimensions: two hubs/circuits (Codex 7; GLM 7), harness parity incl. opencode (Codex 8; GLM 8),
   crash/restart/reboot at the worst moments (Codex 9; GLM 9), the deployed install not the checkout (Codex 10; GLM 10).
6. Concrete negative controls at tiers 2-4 (Codex 11-13; GLM 11).
7. One live pass is a release gate, not continuing proof (Codex 14; GLM 12).
8. Drop blanket fixture+mutant for every requirement on every push (Codex 16; GLM 13).

## Only one reviewer
9. Codex: heartbeat must not imply health — R-29.e accepts "fresh heartbeat and no flag" as CLEAR (6); a
   three-agent many-to-many scenario is already required by R-1.e, plus ACKNOWLEDGED_NO_ACTION and denied-resume
   cases (15); simulated time for long schedules (16).
10. GLM: each control shown green before the break and red after ("kill-check") (11); reproduce the three real
    failures as the controls (11); tier 3 must run through the deployed scheduler and hooks (5); continuous
    per-agent reachability (CAND-6), canary rotated across hosts/pairs, re-run after cert rotation (12); keep
    mutant-per-push for invariants only (13); merge "conversation continues" and "resumed mid-conversation" (13).

## Disagreements
11. None of substance. Tone differs: GLM's verdict is that the draft "would have passed all three cited failures";
    Codex's is "keep the four tiers, but replace exclusive tier assignment with failure-mode coverage".
