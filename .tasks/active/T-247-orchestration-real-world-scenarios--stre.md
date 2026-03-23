---
id: T-247
name: "Orchestration real-world scenarios — stress-test orchestrator.route + bypass registry"
description: >
  Inception: Orchestration real-world scenarios — stress-test orchestrator.route + bypass registry

status: started-work
workflow_type: inception
owner: human
horizon: now
tags: [T-233, T-238, T-237, orchestration, bypass, testing]
components: []
related_tasks: [T-233, T-237, T-238, T-248, T-249, T-250, T-251, T-252, T-253, T-254, T-255]
created: 2026-03-23T16:43:45Z
last_update: 2026-03-23T16:43:45Z
date_finished: null
---

# T-247: Orchestration real-world scenarios — stress-test orchestrator.route + bypass registry

## Problem Statement

We built `orchestrator.route` (T-237) and bypass registry (T-238) with unit tests but no end-to-end validation against real-world usage patterns. Need concrete scenarios from multiple perspectives to stress-test, discover gaps, and calibrate parameters.

## Exploration Plan

5 agents explored from different lenses (framework maintenance, code review, infrastructure, research, adversarial). Each produced 3 scenarios = 15 total. Research artifacts in `docs/reports/T-247-scenarios-*.md`.

## Scope Fence

**IN:** Scenario generation, gap discovery, task decomposition for fixes.
**OUT:** Building the fixes (those are T-248..T-255).

## Acceptance Criteria

- [x] 15 scenarios from 5 perspectives documented
- [x] Architectural gaps identified and severity-ranked
- [x] Build tasks created with ACs and artifact links (T-248..T-255)
- [ ] Go/No-Go decision made

## Go/No-Go Criteria

**GO if:**
- Scenarios reveal actionable gaps that improve the orchestration architecture
- Build tasks are well-scoped and have clear ACs

**NO-GO if:**
- Scenarios are theoretical with no practical test approach
- Gaps are already addressed by existing code

## Verification

# Inception — no code verification needed

## Decisions

### 2026-03-23 — Semantic failure gap (research scenario 2)
- **Chose:** Document as caller responsibility, no registry fix
- **Why:** The registry correctly tracks RPC success. Interpreting result quality is the calling agent's job.
- **Rejected:** Adding result-quality scoring to the registry — over-engineering, domain-specific

## Decision

GO — 15 scenarios from 5 lenses revealed 8 real architectural gaps (3 high severity). Decomposed into 8 build tasks (T-248..T-255) with suggested build order. See docs/reports/T-247-orchestration-scenarios.md.

## Updates

### 2026-03-23T16:43:45Z — task-created
- Created inception task for orchestration scenario exploration

### 2026-03-23T16:50:00Z — research complete
- 5 agents returned 15 scenarios, 8 gaps identified
- Research artifacts committed: docs/reports/T-247-scenarios-*.md

### 2026-03-23T16:54:00Z — task decomposition
- Created T-248 through T-255 from gap analysis
- Updated research artifact with gap table and build order
