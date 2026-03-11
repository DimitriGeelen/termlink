---
id: T-103
name: "Inception — Error Escalation Ladder auto-population via JSONL extraction"
description: >
  Currently the Error Escalation Ladder (A→B→C→D) and patterns.yaml are manually
  populated via fw healing resolve. This inception explores automatically extracting
  tool errors from the JSONL transcript and feeding them into the ladder — making
  pattern detection proactive rather than discipline-dependent. Explore only.
status: work-completed
workflow_type: inception
owner: human
horizon: now
tags: [antifragility, error-escalation, jsonl, patterns, healing]
components: []
related_tasks: [T-094, T-101, T-104]
created: 2026-03-11T13:00:00Z
last_update: 2026-03-11T23:29:59Z
date_finished: 2026-03-11T23:29:59Z
---

# T-103: Inception — Error Escalation Ladder Auto-Population

## Problem Statement

The Error Escalation Ladder (A→B→C→D) is a reactive system: an agent must notice a
failure, diagnose it, and manually run `fw healing resolve T-XXX --mitigation "..."`.
This relies on discipline. Patterns that repeat silently across sessions are invisible
until someone notices.

The JSONL transcript contains every tool error with `is_error: true` flagging — already
structured, already on disk. If we extract this automatically, the ladder becomes
proactive: patterns surface without anyone having to remember to log them.

## The Escalation Ladder (reference)

- **A** — Don't repeat the same failure (pattern matching)
- **B** — Improve technique (cluster of similar failures)
- **C** — Improve tooling (systematic failure in one area)
- **D** — Change ways of working (session-wide or cross-session systemic pattern)

Currently: manually triggered after the fact.
Target: auto-detected from transcript data.

## Questions to Explore

1. **What does an extracted error record look like?**
   - Tool name, error message, timestamp, task context, session ID
   - Is there enough signal to classify A vs B vs C vs D automatically?

2. **When should extraction run?**
   - Option A: PreCompact hook (already runs, highest-stakes moment)
   - Option B: PostToolUse checkpoint (real-time, after every tool)
   - Option C: Standalone `fw errors harvest` command (on-demand)
   - Tradeoffs: frequency vs. overhead vs. completeness

3. **How does extracted data feed patterns.yaml?**
   - Auto-append new error patterns? Or surface for human review?
   - Risk of noise: not every error is a pattern worth recording
   - Deduplication: same error in 5 sessions = one pattern, not five entries

4. **What's the right storage format?**
   - Append to existing patterns.yaml? Separate errors.jsonl? SQLite?
   - Links to T-104 (tool call capture store) — may share infrastructure

5. **How does this connect to the healing agent?**
   - Does `fw healing diagnose T-XXX` become richer if it can query historical errors?
   - Could it say "this error appeared 4 times in the last 3 sessions — escalate to C"?

## Relationship to Other Options

- **T-101 (JSONL transcript reader):** Shares the transcript parsing infrastructure.
  Conversation capture reads `user`/`assistant` events. Error extraction reads
  `tool_result` events with `is_error: true`. Same source, different filters.
- **T-104 (tool call capture store):** Errors are a subset of tool calls. T-103 and
  T-104 should share the capture layer — don't build two separate readers.
- **T-094 (volatile conversation loss):** This is a separate problem but same root
  infrastructure (JSONL transcript). Solving T-101, T-103, T-104 together makes
  the transcript the project's primary observability source.

## Scope Fence

**IN:** Explore the approach, design the extraction pipeline, assess feasibility
**OUT:** Implementation — dialogue and design only until GO decision

## Acceptance Criteria

### Agent
- [x] Error record structure defined — `assistant` events with `isApiErrorMessage: true`, prose text only
- [x] Extraction timing decision made — moot until error records are structured
- [x] Storage format decided — defer to T-104 (cross-session store)
- [x] Escalation ladder mapping designed — requires cross-session error counts, not feasible without T-104
- [x] GO/NO-GO framed for discussion

### Human
- [x] Approach reviewed and direction decided — DEFER (human: "do as you see fit")

## Decisions

**Decision**: NO-GO / DEFER

## Decision

**Decision**: DEFER — T-104 prerequisite unmet. JSONL lacks structured error events (`tool_result` with `is_error` not present — errors are prose in `assistant` messages). Auto-classification requires cross-session aggregation (T-104) and hook enrichment. Revisit after T-104 is built.

Rationale: Building the analysis layer before the data layer produces waste. T-104 should explicitly scope error events as first-class records when built.

Lightweight alternative (`fw errors harvest`) deferred — value too low to justify a task before T-104 exists.

## Updates

### 2026-03-11T23:23:10Z — status-update [task-update-agent]
- **Change:** status: captured → started-work

### 2026-03-11T23:29:59Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
