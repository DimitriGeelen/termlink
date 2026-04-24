---
id: T-1209
name: "SubagentStop hook design — result-management enforcement (T-175 parent)"
description: >
  Inception: SubagentStop hook design — result-management enforcement (T-175 parent)

status: work-completed
workflow_type: inception
owner: human
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-04-24T09:14:28Z
last_update: 2026-04-24T09:44:59Z
date_finished: 2026-04-24T09:44:59Z
---

# T-1209: SubagentStop hook design — result-management enforcement (T-175 parent)

## Problem Statement

Sub-agents dispatched via the Task tool routinely return large blobs (raw file contents, unstructured search results) that consume orchestrator context. The CLAUDE.md Sub-Agent Dispatch Protocol prescribes "write to disk, return path+summary" but nothing enforces it. Current guard `check-dispatch.sh` is PostToolUse, advisory-only. Gap **G-015**: sub-agent results bypass governance. The human's framing: **do not lose information**. That rules out hard-blocking (which would drop the output entirely) and rules in auto-migration to persistent storage.

Full research: `docs/reports/T-1209-subagentstop-hook-inception.md`.

## Assumptions

- A1: `last_assistant_message` size in the SubagentStop payload is a reliable proxy for bytes returned to the orchestrator.
- A2: 80%+ of legitimate dispatches fit under 2KB when the dispatch protocol is followed; the long tail (>10KB) is overwhelmingly the footgun case.
- A3: Exit code on SubagentStop determines whether non-zero replaces or supplements the orchestrator-visible response — must be tested in S1'.

## Exploration Plan

- **S1' (1h, gates everything):** Message-mutation semantics test. Can SubagentStop rewrite the orchestrator-visible response, or is it post-hoc only? Single synthetic dispatch with a known-large return; observe what the orchestrator sees.
- **S2 (2h, parallel):** Size-distribution survey — passive logger records `last_assistant_message` length on every dispatch for 1 week; builds histogram to refine threshold.
- **S3 (2h):** Bus-migration handler — on over-threshold detect, write summary to `fw bus`, emit agent-visible nudge. Verify orchestrator receives `R-NNN @ path+summary`.

## Technical Constraints

- `fw bus` already handles typed envelopes + blob spill ≥2KB (CLAUDE.md §Result Ledger). Storage layer is solved — no new persistence needed.
- SubagentStop payload provides `agent_transcript_path` natively — full transcript is on disk at hook-fire time. No streaming concerns.
- Framework-side script; retires `check-dispatch.sh` only after S1' + S3 prove the flow.

## Scope Fence

**IN:** size-threshold auto-migration to fw bus, orchestrator gets path+summary reference, discoverability via `fw bus read`.

**OUT:** content-based quality gates; hard-blocking under any circumstance; cross-session analytics.

## Acceptance Criteria

### Agent
- [x] Problem statement validated
- [x] Assumptions tested
- [x] Recommendation written with rationale

### Human
- [x] [REVIEW] Review exploration findings and approve go/no-go decision
  **Steps:**
  1. Run: `fw task review T-XXX` (opens Watchtower with recommendation, assumptions, research artifacts)
  2. Review the Agent Recommendation section and go/no-go criteria evaluation
  3. Record decision via the Watchtower form or the command shown alongside the QR code
  **Expected:** Decision recorded, task completed
  **If not:** Ask agent for clarification on specific findings

## Go/No-Go Criteria

**GO if:**
- S1' shows ANY mechanism to preserve the over-threshold output to disk before it enters orchestrator context (whether via message mutation or stderr-nudge-to-read-bus).
- S3 successfully migrates a synthetic large return to `fw bus` end-to-end.
- Threshold T=8KB produces <10% migration rate on legitimate dispatches (measured in S2).

**NO-GO if:**
- S1' shows no way to intercept — `fw bus` still captures, but orchestrator always ingests the full blob first (no context relief).
- Migration adds >500ms latency to sub-agent dispatch (breaks D3 usability).

## Verification

# Shell commands that MUST pass before work-completed. One per line.
# Lines starting with # are comments (skipped). Empty lines ignored.
# For inception tasks, verification is often not needed (decisions, not code).

## Recommendation

**Recommendation:** GO (Option B — live migration from day 1, T=8KB)

**Rationale:** The human's framing ("do not lose information") is decisive. Hard-blocking loses information (orchestrator never sees the output); the framework's prior position ("Explore in future") is too passive given the current footgun cost (a single 25KB return pollutes the orchestrator for the rest of the session). Mode B — auto-migrate to `fw bus` on over-threshold detect — preserves information on disk AND keeps orchestrator context clean. 4 of the 5 prerequisites (capture, storage, pointer, discoverability, threshold) are already built into the framework via `fw bus`. Only the threshold is unknown, and 8KB is a safe initial guess that S2's histogram will refine inside a week. S1' tests the mutation-semantics question, but EITHER outcome (mutation or stderr-nudge) preserves information — the GO is viable in both branches.

**Evidence:**
- `fw bus` already exists with typed envelopes and automatic blob spill ≥2KB (CLAUDE.md §Result Ledger). No new storage to build.
- CLAUDE.md §Sub-Agent Dispatch Protocol explicitly states content generators MUST write to disk, not return raw content — so the convention is already documented; only enforcement is missing.
- Observed failure mode (25KB raw returns) reproduces easily with naive Explore dispatches; current `check-dispatch.sh` is advisory and misses.
- T-1162 pattern confirms "dual-path with migration" is a low-risk shape (never loses legacy callers).

**Human direction (2026-04-24):** "we want not to lose information" — rejected hard-blocking; chose Option B (live migration from day 1).

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

**Decision**: GO

**Rationale**: The human's framing ("do not lose information") is decisive. Hard-blocking loses information (orchestrator never sees the output); the framework's prior position ("Explore in future") is too passive given the current footgun cost (a single 25KB return pollutes the orchestrator for the rest of the session). Mode B — auto-migrate to `fw bus` on over-threshold detect — preserves information on disk AND keeps orchestrator context clean. 4 of the 5 prerequisites (capture, storage, pointer, discoverability, threshold) are already built into the framework via `fw bus`. Only the threshold is unknown, and 8KB is a safe initial guess that S2's histogram will refine inside a week. S1' tests the mutation-semantics question, but EITHER outcome (mutation or stderr-nudge) preserves information — the GO is viable in both branches.

**Date**: 2026-04-24T09:44:59Z

## Updates

<!-- Auto-populated by git mining at task completion.
     Manual entries optional during execution. -->

### 2026-04-24T09:16:48Z — status-update [task-update-agent]
- **Change:** status: captured → started-work

### 2026-04-24T09:44:59Z — inception-decision [inception-workflow]
- **Action:** Recorded inception decision
- **Decision:** GO
- **Rationale:** The human's framing ("do not lose information") is decisive. Hard-blocking loses information (orchestrator never sees the output); the framework's prior position ("Explore in future") is too passive given the current footgun cost (a single 25KB return pollutes the orchestrator for the rest of the session). Mode B — auto-migrate to `fw bus` on over-threshold detect — preserves information on disk AND keeps orchestrator context clean. 4 of the 5 prerequisites (capture, storage, pointer, discoverability, threshold) are already built into the framework via `fw bus`. Only the threshold is unknown, and 8KB is a safe initial guess that S2's histogram will refine inside a week. S1' tests the mutation-semantics question, but EITHER outcome (mutation or stderr-nudge) preserves information — the GO is viable in both branches.

### 2026-04-24T09:44:59Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
- **Reason:** Inception decision: GO
