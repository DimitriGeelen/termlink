---
id: T-104
name: "Inception — Tool call capture store (data layer, all calls + errors, cross-session)"
description: >
  Design and explore a tool call capture store: capture every tool call, its result,
  and whether it errored, across all sessions. Store the raw data without pre-optimizing
  for reporting. When the data exists, insights and reporting (T-105) can be derived
  from it. Principle: capture is permanent, reporting is iterative. Explore only.
status: captured
workflow_type: inception
owner: human
horizon: now
tags: [observability, tool-calls, data-capture, cross-session, jsonl]
components: []
related_tasks: [T-094, T-101, T-103, T-105]
created: 2026-03-11T13:00:00Z
last_update: 2026-03-11T13:00:00Z
date_finished: null
---

# T-104: Inception — Tool Call Capture Store

## Problem Statement

No persistent record of tool call activity exists across sessions. Each session's
tool calls live in the JSONL transcript, but that transcript is session-scoped and
not aggregated. After context compaction or session end, the data is effectively
inaccessible without going back to parse raw files.

**Design principle (from dialogue):** Capture everything first, report later.
Don't design reports before you have data — you'll optimize for the wrong questions.
Once data is stored, any insight can be derived. What you don't capture, you can
never retroactively recover.

## What to Capture (per tool call)

From the JSONL transcript, each tool call event contains:
- Tool name (Bash, Read, Write, Edit, Grep, Agent, etc.)
- Tool input (the arguments — may be large, consider truncation/hashing)
- Result: success or error (`is_error: true/false`)
- Error content (if error — already structured in JSONL)
- Timestamp
- Session ID
- Task context (from focus.yaml at time of call)
- Token count at time of call (from budget state)

## Questions to Explore

1. **Storage format options:**
   - Append-only JSONL at `.context/telemetry/tool-calls.jsonl` — simple, queryable with Python, no dependencies
   - SQLite at `.context/telemetry/tool-calls.db` — queryable with SQL, better for cross-session aggregation, more complex
   - YAML — human-readable but gets large fast, probably wrong for this
   - Existing metrics-history.yaml extension — already exists, already structured

2. **Volume and storage:**
   - This session alone: ~350 lines in JSONL, subset are tool calls
   - Estimate: ~50-200 tool calls per active session
   - At 500 bytes/record: ~100KB per session, ~3MB/month at one session/day
   - Monitor `.context/telemetry/` size as part of `fw metrics` output

3. **Capture timing:**
   - Option A: Extract from JSONL at PreCompact / session end — batch, no overhead during session
   - Option B: PostToolUse hook appends each call in real-time — live data, more overhead
   - Option C: Hybrid — real-time count only (cheap), full extraction at session end

4. **Tool input capture:**
   - Full input could be large (e.g., Write tool with file contents)
   - Options: truncate at N chars, hash for deduplication, store metadata only (tool name + size)
   - Errors: always store full error output (diagnostic value outweighs size cost)

5. **Cross-session aggregation:**
   - How to query: "how many Bash errors in last 10 sessions?"
   - Simple Python script vs. SQLite query vs. fw command
   - Links to T-105 (reporting page)

6. **What does NOT need capturing:**
   - `progress` events (tool execution intermediate states) — noise
   - `file-history-snapshot` events — not tool calls
   - `system` events — metadata only

## Relationship to Other Tasks

- **T-103 (error escalation):** Errors are a subset of tool calls. T-103 should
  consume the T-104 data store rather than building its own parser.
- **T-105 (reporting page):** Entirely depends on T-104. Cannot start until T-104
  has a defined schema and storage format.
- **T-101 (conversation capture):** Same JSONL source, different event types.
  Parsers can share infrastructure.

## Scope Fence

**IN:** Design the capture schema, storage format, and extraction timing
**OUT:** Implementation, reporting UI — capture layer design only

## Acceptance Criteria

### Agent
- [ ] Storage format decided (JSONL / SQLite / other) with rationale
- [ ] Schema defined: what fields are captured per tool call
- [ ] Volume estimate confirmed against actual session data
- [ ] Capture timing decided (PreCompact / PostToolUse / hybrid)
- [ ] GO/NO-GO framed

### Human
- [ ] Approach reviewed and direction decided

## Decisions

## Decision

## Updates
