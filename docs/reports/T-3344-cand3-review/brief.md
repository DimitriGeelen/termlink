# External review: how arc-011 requirements are verified (T-3344, OD-17 CAND-3)

You are an external reviewer. Be critical: your job is to find where this proposal under-tests,
over-tests, or is wrong. Read-only access to /opt/termlink. Answer in under 900 words.

## Context

arc-011 makes agent-to-agent messages actually reach the agent: a message goes from one AI coding
agent (e.g. a Claude Code session) through a TermLink hub to another agent, and must be stored,
handed over into the receiving agent's session, and answered. Stages: SENT → RECEIVED → STORED →
HANDED_OVER → REPLIED | ACKNOWLEDGED_NO_ACTION. Requirements are in
`docs/design/interactive-agent-communication-01-requirements.md`; the operator's rulings so far are in
`docs/design/interactive-agent-communication/interview-step-01.md` (OD-1..OD-17).

Three recent production failures, all of which looked healthy on every local check:
1. 2026-10-03: mail stored on the hub, never surfaced to the receiving agent.
2. A stray second hub (different runtime dir) silently split sessions from their inboxes.
3. A receiver sidecar with a fresh heartbeat refused every delivery confirmation 32,205 times
   (its signing key was stored under another name).

The operator's standing rule (profile P1.2.e): "Nothing is called working without a live test with
two real agents and a negative control." (A negative control is a run where the thing is deliberately
broken and the test must then fail.)

Candidate requirement CAND-3 turns that into the verification rule for arc-011. The operator found
"two real agents for every requirement" too heavy, then pointed out a missing one-agent tier.

## The proposal under review

Each requirement is tested at the LOWEST tier that can see its failure, always with a negative
control at that tier:

1. Fixture (hermetic, no hub, no agent): pure logic — per-stage idempotence per message id
   (duplicates answered with the stage reached, never a second hand-over), the five message states
   (WAITING, WAITING FOR RECIPIENT, STUCK, UNKNOWN, DEAD), retention, consent allow-list rules,
   resend-on-loss. Negative control: a mutant that must turn the test red. Every requirement; runs
   on every push.
2. Live hub, no agent: hub behaviour on a real hub (hub id reported by an authenticated RPC,
   retention sweep, telemetry journey copied to the hub), poked by a scripted client.
3. One real agent with a scripted peer: the receive side of the harness boundary — urgent content
   delivered mid-turn via the harness hook and never typed into a busy prompt; HANDED_OVER proven by
   the receiver's own transcript; readiness; a downgraded urgent shown as downgraded; a duplicate
   never causes a second hand-over. The scripted sender gives precise timing and edge cases.
4. Two real agents: only claims about the round trip between agents — a reply actually returns,
   a conversation continues, a dead agent is resumed mid-conversation from its transcript and
   answers, consent between two real identities across projects, and the daily end-to-end canary
   (operator ruling OD-14) which IS this tier running every day.
5. Closing rule: arc-011 closes only when tiers 1-3 are green and the tier-4 suite passes live on the
   real fleet ("shipped == live").
6. The evidence step marks every requirement with its tier; the review step checks no requirement is
   tested below the tier where its failure shows.

## Questions

Q1. Is "lowest tier that can see its failure" a sound selection rule? Where does it fail?
Q2. Are any of the listed claims placed in too low a tier (would pass while broken in production)?
    Name them, citing the three failures above if relevant.
Q3. Is a tier missing? Specifically consider: two hubs / cross-host (the design routes over
    "circuits" between hubs), other harnesses than Claude Code (an adapter contract exists for
    opencode etc.), restart/reboot of hub or host, and the deployed install (cron/systemd/hooks
    actually installed) versus the source tree.
Q4. What does a meaningful negative control look like at tiers 2, 3 and 4? Give one concrete
    example each.
Q5. Is the closing rule sufficient to prevent "shipped but dark" (passes once, breaks later)?
Q6. Anything you would remove as unnecessary weight?

Format: numbered findings, each tagged [AGREE], [CHANGE] or [MISSING], with one-line reasoning;
then a one-line overall verdict.
