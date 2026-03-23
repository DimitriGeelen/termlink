---
id: T-233
name: "Specialist agent orchestration — delegate to domain experts via TermLink"
description: >
  Inception: Specialist agent orchestration — delegate to domain experts via TermLink

status: started-work
workflow_type: inception
owner: human
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-03-23T08:59:00Z
last_update: 2026-03-23T09:20:47Z
date_finished: null
---

# T-233: Specialist agent orchestration — delegate to domain experts via TermLink

## Problem Statement

An orchestrator agent needs to delegate specialized work (research, infrastructure, design, coding) to specialist agents that have pre-loaded domain context. Today, single agents do everything with no specialization. TermLink can be the coordination layer — spawning specialists, routing questions, collecting results — but the orchestration pattern, specialist context loading, and delegation protocol need to be designed.

## Assumptions

- A-1: Specialist agents provide better results than a generalist when given focused domain context
- A-2: TermLink's existing primitives (spawn, agent ask/listen, events) are sufficient for orchestration
- A-3: The overhead of spawning + delegating is worth it vs. doing everything in one context
- A-4: Specialist context can be pre-loaded via system prompts, CLAUDE.md, or injected files

## Exploration Plan

1. **Dialogue 1**: Map use cases — what kinds of delegation? (research, infra, design, code, test)
2. **Dialogue 2**: Architecture options — TermLink-native vs. Claude Agent SDK vs. hybrid
3. **Spike**: Prototype one delegation pattern (orchestrator → research specialist → result)
4. **Assessment**: Is this worth building into TermLink, or is it a framework-level concern?

## Technical Constraints

- Claude Code agents are separate processes (not threads) — each has its own context window
- TermLink `spawn` creates new terminal sessions but doesn't control what runs inside them
- `agent ask/listen` provides typed request-response but is event-based (polling)
- Context loading for specialists requires either custom CLAUDE.md files or prompt injection

## Scope Fence

**IN:** Mapping delegation scenarios, architectural options, protocol design
**IN:** One prototype spike to validate feasibility
**OUT:** Full implementation of orchestration framework
**OUT:** AI model selection/routing (this is about agent coordination, not model choice)

## Acceptance Criteria

- [ ] Problem statement validated with human
- [ ] Use cases mapped (which specialisms, when to delegate)
- [ ] Architecture options compared
- [ ] Go/No-Go decision made

## Go/No-Go Criteria

**GO if:**
- Clear use cases where specialist delegation beats generalist approach
- TermLink primitives can support the pattern without major new protocol
- Prototype demonstrates end-to-end delegation

**NO-GO if:**
- Claude Code's built-in Task tool already covers the use cases adequately
- TermLink overhead (spawn, coordinate, collect) exceeds the benefit of specialization
- Specialist context loading is infeasible or unreliable

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

## Decision

<!-- Filled at completion via: fw inception decide T-XXX go|no-go --rationale "..." -->

## Updates

<!-- Auto-populated by git mining at task completion.
     Manual entries optional during execution. -->
