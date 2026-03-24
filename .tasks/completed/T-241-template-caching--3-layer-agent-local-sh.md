---
id: T-241
name: "Template caching — 3-layer agent-local, shared, canonical"
description: >
  3-layer template cache: Layer 1 agent-local (.context/specialists/<id>/templates/), Layer 2 shared registry (promoted at 5 uses/0 corrections), Layer 3 specialist canonical (source of truth). Lazy invalidation via schema hash. Pull-on-miss. See T-233 research: Q2b-template-caching.md

status: work-completed
workflow_type: inception
owner: agent
horizon: next
tags: [T-233, orchestration, cache]
components: []
related_tasks: [T-233, T-240]
created: 2026-03-23T13:27:50Z
last_update: 2026-03-24T06:24:04Z
date_finished: 2026-03-24T06:24:04Z
---

# T-241: Template caching — 3-layer agent-local, shared, canonical

## Context

Evaluate whether a 3-layer template caching system is needed now, given: (1) T-240 NO-GO on negotiation protocol, (2) no persistent specialist agents exist yet, (3) the T-233 Q2b design assumes specialist interactions that haven't materialized. Research: `docs/reports/T-233-Q2b-template-caching.md`.

## Problem Statement

Is a formal template caching mechanism needed for agent-specialist collaboration, or are current dispatch patterns (schema-in-prompt, T-257 convention) sufficient?

## Assumptions to Validate

- A1: Agents interact with specialists repeatedly, making caching valuable
- A2: Template formats change frequently enough to need version invalidation
- A3: Per-agent template variants (usage-specific) are meaningfully different from shared templates
- A4: The pull-on-miss + lazy invalidation model is the right caching strategy
- A5: 5-use/0-correction promotion threshold is appropriate

## Research Questions

### Q1: Evidence of Repeated Specialist Interactions
How many times has the same agent type interacted with the same specialist type? Is there actual repetition that caching would optimize?

### Q2: Current Template/Schema Handling
How do current dispatch patterns handle format expectations? Does schema-in-prompt (T-257) already solve the problem template caching aims to address?

### Q3: Prerequisites — Do Specialists Exist?
What specialist infrastructure exists? Can template caching be built without persistent specialist agents?

## Acceptance Criteria

### Agent
- [x] Research artifact created at `docs/reports/T-241-template-caching-inception.md`
- [x] Each research question answered with evidence
- [x] All assumptions validated/invalidated with evidence
- [x] GO/NO-GO decision recorded with rationale

## Verification

test -f docs/reports/T-241-template-caching-inception.md
grep -q "GO\|NO-GO" docs/reports/T-241-template-caching-inception.md

## Decisions

**Decision**: NO-GO

**Rationale**: Zero repeated specialist interactions, schema-in-prompt already works, specialist ecosystem doesn't exist yet

**Date**: 2026-03-24T06:24:04Z

## Updates

### 2026-03-23T13:27:50Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /Users/dimidev32/001-projects/010-termlink/.tasks/active/T-241-template-caching--3-layer-agent-local-sh.md
- **Context:** Initial task creation

### 2026-03-24T06:19:18Z — status-update [task-update-agent]
- **Change:** status: captured → started-work

### 2026-03-24T06:24:04Z — inception-decision [inception-workflow]
- **Action:** Recorded inception decision
- **Decision:** NO-GO
- **Rationale:** Zero repeated specialist interactions, schema-in-prompt already works, specialist ecosystem doesn't exist yet

### 2026-03-24T06:24:04Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
- **Reason:** Inception decision: NO-GO
