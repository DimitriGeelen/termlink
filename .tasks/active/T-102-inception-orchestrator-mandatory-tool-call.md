---
id: T-102
name: "Inception — Orchestrator mandatory tool call constraint"
description: >
  Explore the architectural idea: restrict the orchestrator agent so every substantive
  response MUST include a tool call. If the orchestrator needs to explore/discuss, it
  spawns a dedicated agent for that. This transforms invisible conversations into
  traceable tool-call sequences. Explore only — understand tradeoffs before deciding.
status: captured
workflow_type: inception
owner: human
horizon: later
tags: [architecture, orchestrator, tool-call, exploration]
components: []
related_tasks: [T-094, T-099, T-100, T-101]
created: 2026-03-11T12:00:00Z
last_update: 2026-03-11T12:00:00Z
date_finished: null
---

# T-102: Inception — Orchestrator Mandatory Tool Call Constraint

## Problem Statement

What if we change the architectural constraint: the orchestrator MUST make a tool call
for every substantive response. Pure conversation is not allowed at the orchestrator level.
If exploration is needed, it must be delegated to a spawned agent.

## The Idea

Three variants to explore:

**Variant A — Mandatory `fw note` per response:**
Every substantive assistant response must include `fw note "..."` as a tool call.
Conversation is logged structurally. All turns become hookable tool events.
Tradeoff: overhead, changes the feel of interaction.

**Variant B — Spawn-for-conversation:**
Orchestrator never explores inline. "We need to think about X" → spawns an inception
agent → returns a structured result. Orchestrator stays clean; all exploration is
delegated and therefore tracked.
Tradeoff: heavyweight, latency, cost.

**Variant C — Scribe agent:**
Lightweight TermLink session acting as a conversation logger. Orchestrator routes all
responses through the scribe, which logs them as TermLink events (persistent, replayable).
Tradeoff: requires Agent Mesh Phase 1 to be built first.

## Relationship to Other Options

This is the most architectural of the four options — it changes HOW the orchestrator
works rather than capturing what it produces. Orthogonal to T-101 (reading existing
transcript) and T-100 (capturing terminal output). Complementary to T-099 (platform fix).

Could be combined: use JSONL reader (T-101) as the immediate fix, and orchestrator
constraint (T-102) as the long-term architectural norm.

## Scope Fence

**IN:** Understand the tradeoffs of each variant, map implications for human-agent UX
**OUT:** Any implementation — this is exploration and dialogue only

## Acceptance Criteria

### Agent
- [ ] Three variants documented with tradeoffs
- [ ] Impact on human-agent interaction analyzed
- [ ] Relationship to Agent Mesh roadmap mapped
- [ ] Go/no-go framed for discussion

### Human
- [ ] Variants discussed, preferred direction identified

## Decisions

## Decision

## Updates
