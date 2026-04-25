---
id: T-243
name: "Multi-turn agent conversation primitive over TermLink (re-scoped from script-error-yielding)"
description: >
  Inception: Two or more agents need to hold a reliable multi-turn conversation over TermLink. Existing primitives (`agent.ask`, `request`, `channel.post`) are single-shot. Design a session-scoped dialog protocol with explicit yield/resume semantics. Re-scoped 2026-04-26 — original "script error yielding" framing was one downstream symptom; root pain is missing multi-turn agent dialogue. Auth foundation (G-011) deferred to T-1284.

status: started-work
workflow_type: inception
owner: human
horizon: now
tags: [T-233, orchestration, error-yielding]
components: []
related_tasks: [T-233]
created: 2026-03-23T13:28:06Z
last_update: 2026-04-25T22:16:23Z
date_finished: null
---

# T-243: Script error yielding — checkpoint-based execution via TermLink sessions

## Problem Statement

Two or more agents need to hold a reliable multi-turn conversation over TermLink. Today's primitives (`agent.ask`, `request`, `channel.post`) are single-shot — they have no notion of dialog state, no clean way for one side to pause mid-turn ("hold on, I need to consult my LLM"), and no resumption protocol. The original framing (script-error-yielding) was one downstream symptom; the real missing primitive is **session-scoped multi-turn dialog with yield/resume semantics.**

Re-scoped 2026-04-26 after operator dialogue. Original framing (script error yielding via checkpoints / error-streams / PTY-pause) preserved in `docs/reports/T-243-multi-turn-agent-conversation-inception.md` as one of the use cases this primitive enables.

Research artifact: `docs/reports/T-243-multi-turn-agent-conversation-inception.md` (C-001).

## Assumptions

- A-1: Auth stability (G-011, hub.secret rotation, TOFU drift) will be solved separately by T-1284 plus the existing T-1051..T-1058 line. T-243 assumes a stable auth foundation.
- A-2: Existing single-shot primitives (`agent.ask`, `request`, `channel.post`) cannot be incrementally extended into a multi-turn protocol without breaking their stateless contracts — a new primitive (or a clearly bounded extension) is needed.
- A-3: A multi-turn dialog is most usefully modelled as a typed bidirectional channel scoped to a `conversation_id`, with explicit yield/resume signaling, not as a series of independent RPCs.
- A-4: The yield primitive ("I need to consult my LLM, expect reply within Δt") is more important than fancy turn-ordering — without it, B blocks the channel or A times out.

## Exploration Plan

1. **Dialogue 1 (operator)** — Concrete failing scenarios: which agent pairs are trying to talk, what protocol they're using today, where it breaks. Distinguish auth-flake symptoms (defer to T-1284) from genuine missing-primitive symptoms.
2. **Dialogue 2 (operator)** — Surface area review: ask/request/channel — which is closest to a multi-turn dialog, and what minimally extends it. Decide build-on-existing vs. new primitive.
3. **Spike A — `dialog.open` / `dialog.send` / `dialog.yield` / `dialog.close`** — typed RPCs for an ID-scoped multi-turn channel. One-page protocol sketch + minimal hub stub. No client integration.
4. **Spike B — yield-resume semantics** — Signaling (event? RPC ack with "pending"? channel marker?) and timeout/keepalive policy. Sketch the state machine.
5. **Assessment** — Is the new primitive worth the surface area, or does extending existing ones (e.g., `channel.post` with `conversation_id` metadata + yield event types) cover 80%?

## Technical Constraints

- TermLink's bus has retention policies per topic — multi-turn state survives only as long as retention. Conversations longer than retention need explicit checkpointing.
- Yield-resume must work across hub restarts (auth heal mid-conversation should not lose dialog state).
- Cross-host conversations route through hub forwarding; latency budget for "type response, route, reach peer" is bounded by the hub's request timeout (today 30s default).
- Conversation state is partially in agents (their LLM context) and partially in hub (session-pair binding). Drift between the two on either side breaks the dialog.
- Depends on T-1284 / G-011 closure for auth stability — building on flaky auth produces unreliable conversations and wrong root-cause attribution.

## Scope Fence

**IN:** Multi-turn agent-to-agent dialog primitive design (RPCs, events, state model). One spike to validate the protocol shape. Assessment of build-new vs. extend-existing.

**OUT:** Auth hardening (T-1284 / G-011). Driving interactive CLI tools via PTY (existing `termlink interact` and pty primitives already cover this). Specialist orchestration / routing (T-237 done). Script error yielding via checkpoints (subsumed — once the dialog primitive exists, scripts use it).

## Acceptance Criteria

### Agent
- [ ] Research artifact `docs/reports/T-243-multi-turn-agent-conversation-inception.md` created and updated through both dialogues
- [ ] Concrete failing scenarios captured from operator (Dialogue 1)
- [ ] Existing-primitive surface review documented (Dialogue 2)
- [ ] One protocol spike sketched (RPC names, params, state model, yield/resume semantics)
- [ ] Build-new vs. extend-existing recommendation made with rationale
- [ ] Go/No-Go decision recorded via `fw inception decide T-243`

### Human
- [ ] Operator confirms the recommendation reflects the actual pain and the chosen direction is buildable
  - **Steps:** Read the Recommendation block in this task file and the matching section in the research artifact. Compare against the stated pain (recurring auth issues + multi-turn agent dialogue not working).
  - **Expected:** Recommendation matches the stated need; if GO, child build tasks are reasonable scope.
  - **If not:** Re-open dialogue, redirect, or NO-GO the inception.

## Go/No-Go Criteria

**GO if:**
- A clear protocol shape emerges (named RPCs, state machine, yield semantics) that two operators could plausibly implement against
- T-1284 (auth foundation) is in flight — without it, the primitive can't be tested reliably
- Operator confirms this addresses the actual multi-turn-agent pain, not just the original script-yielding framing

**NO-GO if:**
- Existing primitives (`channel.post` with `conversation_id` metadata + a yield event type) cover the use cases at <30% of the cost — in which case the inception's output is the extension proposal, not a new primitive
- The pain turns out to be entirely auth-driven and dissolves once T-1284 lands

## Verification

<!-- Shell commands that MUST pass before work-completed. One per line.
     Lines starting with # are comments. Empty lines ignored.
     The completion gate runs each command — if any exits non-zero, completion is blocked.
     For inception tasks, verification is often not needed (decisions, not code).
-->

## Decisions

<!-- Record decisions ONLY when choosing between alternatives.
     Skip for tasks with no meaningful choices.
     Format:
     ### [date] — [topic]
     - **Chose:** [what was decided]
     - **Why:** [rationale]
     - **Rejected:** [alternatives and why not]
-->

## Recommendation

_Backfilled 2026-04-19 under T-1139/T-1112 scope — inception decide ran before `## Recommendation` became a required section. Content mirrors the `## Decision` block below for audit compliance (CTL-027)._

<!-- Filled at completion via: fw inception decide T-XXX go|no-go --rationale "..." -->

## Decision

<!-- Filled at completion via: fw inception decide T-XXX go|no-go --rationale "..." -->

## Updates

<!-- Auto-populated by git mining at task completion.
     Manual entries optional during execution. -->

### 2026-04-19T12:30Z — housekeeping [agent]
- **Action:** T-1139 audit remediation touch. Task remains captured/horizon=later pending operator prioritization; no scope change.
- **Status:** Still backlog — inception not yet entered. Will move when another exploration slot opens.

### 2026-04-22T04:52:50Z — status-update [task-update-agent]
- **Change:** horizon: later → next

### 2026-04-25T22:16:23Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
- **Change:** horizon: next → now (auto-sync)
