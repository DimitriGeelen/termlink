# arc-011 step 1 — operator interview log (T-3344)

The open questions of `docs/design/interactive-agent-communication-01-requirements.md` section 9, put to the
operator one at a time by the orchestrator (standing instruction 2026-10-01). Each entry: the question, the
operator's ruling as given, how it was read back, and what it authorises. The role session folds all rulings into
the output in one pass after the last question.

| Version | Date | Change | Task |
|---|---|---|---|
| 0.1 | 2026-10-04 | Log started | T-3344 |

## OD-1 Cross-host send path (section 9.4) — ruled C

1. Asked 2026-10-04 with options A (hub carries cross-host), B (sidecar-to-sidecar for everything, charter change),
   C (hubs set up a circuit; established conversations run sidecar-to-sidecar; hub path as fallback),
   D (conversation bound on the hub path; socket circuit only after a measurement, AEF).
2. Orchestrator recommended C; the output's own recommendation (A, first build) was shown and why it was not
   followed (it leaned on round-1 positions three of four reviewers revised in round 2).
3. **Operator ruling: "c".** Read back as C, built in the order every reviewer agrees on: identity hygiene and the
   hub directory with liveness first (circuit set-up needs it); conversations bound to an instance on the hub path
   (D's binding); then the direct circuit, under the reviewers' conditions: set-up and authorisation through the
   hubs, per-circuit short-lived credentials minted by the hubs, one delivery contract and one log per
   conversation on both paths. AEF's measurement (circuit-break rate, hub-path share of turn time) belongs in the
   circuit's first build.
4. Authorises: OD-1 = C in step 1 (changes R-7, R-12, R-13); the conditions become requirements; the circuit's
   trust model goes to step 2. Leaves open: the hub-to-hub directory/relay principle and the charter rewording,
   OD-18, transport details (step 4).

## OD-2 Urgent into a busy prompt (section 9.5) — ruled B

1. Asked 2026-10-04 with options A (bypass, type only the fixed doorbell, content stored first; 055),
   B (never type into a busy prompt; urgent content through the harness's hook channel; GLM),
   C (authenticated harness interrupt, else report and escalate; Codex), D (keep SQ-4: urgent only shortens the wait).
2. Orchestrator recommended B and showed the output's own recommendation (A plus B's hook route). Fact added,
   verified in the orchestrator's own session: Claude Code delivers PostToolUse hook output into a running turn
   after each tool call ("PostToolUse:Write hook additional context" appeared mid-turn); the Stop hook can hand
   content over at turn end and continue the agent.
3. **Operator ruling: "B".** Read back as: urgent content reaches a busy agent through the harness's hook channel
   (next tool call, or end of turn); nothing is ever typed into a busy prompt; an idle agent gets the normal
   one-line inject. SQ-4 is recorded as reconciled, not superseded: urgent bypasses the wait, never the
   prompt-free check.
4. Authorises: OD-2 = B in step 1 (changes R-19, R-23, R-24); a delivery-adapter requirement for harnesses without
   hooks (055's adapter point); an acceptance test that an urgent message reaches a busy agent mid-turn with
   transcript evidence. Leaves open: interrupt consent (OD-15); other harnesses' routes (OD-7).
