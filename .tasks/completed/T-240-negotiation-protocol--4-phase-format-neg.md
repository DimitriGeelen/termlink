---
id: T-240
name: "Negotiation protocol — 4-phase format negotiation over agent events"
description: >
  Implement negotiate.offer/attempt/correction/accept protocol. Orchestrator brokers introduction then steps back. Agent and specialist talk directly. Max 5 rounds. JSON Schema as wire format. See T-233 research: Q2b-negotiation-protocol.md

status: work-completed
workflow_type: inception
owner: agent
horizon: now
tags: [T-233, orchestration, protocol]
components: []
related_tasks: [T-233, T-237]
created: 2026-03-23T13:27:41Z
last_update: 2026-03-23T23:05:15Z
date_finished: 2026-03-23T23:05:15Z
---

# T-240: Negotiation protocol — 4-phase format negotiation over agent events

## Problem Statement

When agents collaborate with specialists (format output, structure configs, produce artifacts), the interaction isn't a single request-response — it's a dialogue. The T-233 research (Q2b-negotiation-protocol.md) proposed a 4-phase protocol: offer → attempt → correction → accept, built on existing `agent.request`/`agent.response` primitives.

**The question:** Is a formal negotiation protocol needed, or is the simpler dispatch-collect convention (T-257) sufficient for real-world agent collaboration?

**For whom:** Agents orchestrating specialist work (format compliance, schema validation, multi-step artifact production).
**Why now:** T-257 shipped the dispatch-collect foundation. Negotiation would layer on top of it.

## Assumptions

- A1: Real-world agent-specialist interactions frequently require iterative correction (not just one-shot)
- A2: JSON Schema is the right wire format for structural validation between agents
- A3: The existing `agent ask`/`agent listen` primitives can support multi-round negotiation without new RPCs
- A4: 5-round max is sufficient for convergence in realistic scenarios
- A5: Agents benefit from caching negotiated schemas for repeated interactions

## Exploration Plan

1. **Validate A1:** Search episodic memory and task history for instances where agent output was corrected/reformatted by another agent or human. How often? What kinds of corrections? (1 hour)
2. **Validate A3:** Spike a 2-round negotiation using existing `agent ask`/`agent listen` — does the current protocol support correlated multi-round exchanges? (1 hour)
3. **Assess alternatives:** Compare 4-phase negotiation vs simpler approaches: (a) publish schema upfront in dispatch prompt, (b) validate on return and reject/retry, (c) no negotiation, just strict schemas. (30 min)

## Technical Constraints

- Must layer on existing event primitives (no new RPCs unless justified)
- Must work over the dispatch-collect pattern (T-257)
- Protocol messages must be backward-compatible with `agent.request`/`agent.response` schemas

## Scope Fence

**IN scope:** Negotiation protocol design, validation of need, spike on existing primitives
**OUT of scope:** Implementation of JSON Schema validation library, template caching (T-241), trust/supervision (T-242)

## Acceptance Criteria

- [x] Problem statement validated with evidence from task history
- [x] Assumptions A1-A5 tested
- [x] Go/No-Go decision made

## Go/No-Go Criteria

**GO if:**
- Evidence of 3+ real cases where iterative format correction occurred between agents
- Existing `agent ask`/`agent listen` supports multi-round exchanges without modification
- The protocol complexity (4 phases, 5-round cap) is justified by the correction frequency

**NO-GO if:**
- Format issues are rare (<3 instances in project history) — just validate on return
- Upfront schema in dispatch prompt eliminates most corrections
- The `agent ask` protocol can't support correlated multi-round without new RPCs (too much protocol work)

## Verification

## Decisions

**Decision**: NO-GO

**Rationale**: Zero instances of iterative format correction in 233 tasks. Schema-in-prompt (Layer 1) delivers 85-90% coverage at 5% complexity. 4-phase negotiation is premature — save for when evidence shows Layers 1+2 are insufficient (>10% failure rate signal).

**Date**: 2026-03-23T23:05:14Z

## Updates

### 2026-03-23T13:27:41Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /Users/dimidev32/001-projects/010-termlink/.tasks/active/T-240-negotiation-protocol--4-phase-format-neg.md
- **Context:** Initial task creation

### 2026-03-23T23:05:14Z — inception-decision [inception-workflow]
- **Action:** Recorded inception decision
- **Decision:** NO-GO
- **Rationale:** Zero instances of iterative format correction in 233 tasks. Schema-in-prompt (Layer 1) delivers 85-90% coverage at 5% complexity. 4-phase negotiation is premature — save for when evidence shows Layers 1+2 are insufficient (>10% failure rate signal).

### 2026-03-23T23:05:15Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
- **Reason:** Inception decision: NO-GO
