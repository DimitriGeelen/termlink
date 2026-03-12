---
id: T-118
name: "Framework PR — fw fabric context command for LLM context enrichment"
description: >
  Add `fw fabric context` command to the framework that generates a structured
  architecture summary (subsystems, key interfaces, dependency graph) suitable
  for injection into LLM system prompts. Generic — works for any project using
  the framework, not TermLink-specific.

status: captured
workflow_type: inception
owner: agent
horizon: later
tags: [framework-pr, fabric, llm-context]
components: []
related_tasks: [T-109, T-117]
created: 2026-03-12T16:04:36Z
last_update: 2026-03-12T16:05:21Z
date_finished: null
---

# T-118: Framework PR — fw fabric context command for LLM context enrichment

## Problem Statement

LLMs assisting on a project need architectural context to make informed decisions. Currently, an LLM must run multiple `fw fabric` commands (overview, deps, impact) to understand the system. A single `fw fabric context` command should produce a structured document — subsystems, key interfaces, dependency graph, component purposes — optimized for LLM consumption (token-efficient, no noise).

This is a **framework-level** improvement. The framework is bash + Python — no TermLink functionality is involved.

## Assumptions

- A-001: A single structured document (YAML or Markdown) is more token-efficient than multiple CLI queries
- A-002: The existing fabric data (cards, subsystems, edges) contains enough information for useful context
- A-003: The output format should be injectable into CLAUDE.md or system prompts

## Exploration Plan

1. **Spike 1** (~30 min): Prototype `fw fabric context` in the framework's fabric agent
2. **Spike 2** (~30 min): Test injection into a session and measure usefulness

## Technical Constraints

- Framework is bash (64 scripts) + Python (587 files, Flask)
- Must work with `fabric.sh` agent pattern (subcommand routing)
- Output must be deterministic (no LLM-generated content)

## Scope Fence

**IN:** `fw fabric context` command that reads existing fabric data and produces a structured summary
**OUT:** Auto-injection into CLAUDE.md, TermLink-specific features, multi-layer subsystem refactoring

## Acceptance Criteria

- [ ] Problem statement validated
- [ ] Assumptions tested
- [ ] Go/No-Go decision made

## Go/No-Go Criteria

**GO if:**
- Prototype produces useful context in < 2K tokens
- Existing fabric data is sufficient (no new card fields needed)

**NO-GO if:**
- Fabric data too sparse for useful summaries
- Output too large for practical LLM injection

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
