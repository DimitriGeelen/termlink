---
id: T-1692
name: "MCP channel_post metadata exposure — close envelope.metadata caller-inaccessibility (cohort-agent ask A)"
description: >
  Cohort-agent ask (A). MCP termlink_channel_post does not accept a metadata parameter; envelope.metadata always emits empty even though the protocol carries it. Affects routing for any LLM consumer keying on metadata.thread/conversation_id. Recommend Shape 1 free-form metadata object.

status: work-completed
workflow_type: inception
owner: human
horizon: null
tags: []
components: [crates/termlink-mcp/src/tools.rs]
related_tasks: []
created: 2026-05-18T09:19:40Z
last_update: 2026-05-18T21:02:14Z
date_finished: 2026-05-18T21:02:14Z
---

# T-1692: MCP channel_post metadata exposure — close envelope.metadata caller-inaccessibility (cohort-agent ask A)

## Problem Statement

The TermLink wire protocol carries `envelope.metadata` (proven by chat-arc:350
which has populated `{thread, in_reply_to, task, mentions}`). The MCP tool
`termlink_channel_post` does NOT accept a `metadata` parameter, so any agent
posting via MCP emits `envelope.metadata = {}`. Cohort-agent's n8n event-watch
matcher at `cohort_hub/n8n_event_match.py:42` keys on
`envelope.metadata.thread` + `conversation_id`; Pen's acks land with both
empty → n8n exec 7 is parked waiting for an ack that will never match.

This is a class issue: any MCP tool that wraps a protocol verb can silently
drop protocol-level fields. T-1692 fixes the instance; a sibling audit task
catches the class.

## Assumptions

- A1: `envelope.metadata` is currently a `Map<String, JsonValue>` server-side
  (verify via `crates/termlink-protocol/`).
- A2: Adding `metadata: Option<serde_json::Value>` to `ChannelPostParams`
  is a pure additive MCP schema change — no protocol version bump needed.
- A3: T-1560 (existing) defined `_thread` / `_project` as underscore-prefixed
  metadata keys for `termlink_agent_post`. The new metadata param subsumes
  this convention OR coexists with it.

## Exploration Plan

1. Read protocol envelope definition + current channel_post MCP tool handler
   (~10 min). Confirm A1+A2.
2. Check whether `termlink_agent_post` (T-1560) should also gain the new
   metadata param, or whether one tool subsumes the other (~5 min).
3. Decide Shape 1 (free-form object) vs Shape 2 (reserved keys). Cohort
   recommended Shape 1; verify no reason to deviate.
4. Optional: audit the class — grep `termlink-mcp/src/tools.rs` for
   protocol features hidden by tool surfaces. Output is a punch list
   that becomes a sibling build task.

## Scope Fence

IN: MCP schema change for `termlink_channel_post` to accept metadata pass-through.
IN: Documentation of the metadata convention (per-contract, not enforced).
OUT: Protocol changes (none needed).
OUT: Server-side validation of metadata schema (free-form by design).
OUT: Per-contract metadata schema enforcement (lives at contract layer, not tool layer).
OUT: Audit-the-class punch list (separate task; this one stays focused).

## Acceptance Criteria

### Agent
<!-- @auto-tick-on-decide -->
- [x] Problem statement validated
<!-- @auto-tick-on-decide -->
- [x] Assumptions tested
<!-- @auto-tick-on-decide -->
- [x] Recommendation written with rationale

### Human
<!-- @auto-tick-on-decide -->
- [x] [REVIEW] Review exploration findings and approve go/no-go decision
  **Steps:**
  1. Run: `fw task review T-XXX` (opens Watchtower with recommendation, assumptions, research artifacts)
  2. Review the Agent Recommendation section and go/no-go criteria evaluation
  3. Record decision via the Watchtower form or the command shown alongside the QR code
  **Expected:** Decision recorded, task completed
  **If not:** Ask agent for clarification on specific findings

## Go/No-Go Criteria

<!-- Fill these BEFORE writing the recommendation. The placeholder detector will block review/decide if left empty. -->
**GO if:**
- Root cause identified with bounded fix path
- Fix is scoped, testable, and reversible

**NO-GO if:**
- Problem requires fundamental redesign or unbounded scope
- Fix cost exceeds benefit given current evidence

## Verification

# Shell commands that MUST pass before work-completed. One per line.
# Lines starting with # are comments (skipped). Empty lines ignored.
# For inception tasks, verification is often not needed (decisions, not code).
#
# Toolchain hint (L-291): if a GO decision will mean editing *.vbproj/*.csproj/*.xaml,
# *.go, Cargo.toml, tsconfig.json, or pom.xml in the build task, plan to add the
# matching build command (dotnet build / go build / cargo check / tsc --noEmit /
# mvn compile) to that build task's ## Verification — P-011 only runs what you write.

## Recommendation

**Recommendation:** GO

**Rationale:** Confirmed gap with named consumer impact (cohort-agent n8n exec 7
parked). Pure additive MCP-schema change, no protocol version bump, no
breaking change to existing callers. Cohort-agent recommends Shape 1 (free-form
object) — matches existing protocol design, lets contracts evolve independently,
no schema lock-in at tool layer. Implementation likely <1 hour. Sibling audit
task ("MCP-vs-protocol surface gap") is the class fix; this one closes the
named instance.

**Evidence:**
- Cohort-agent letter (operator-relayed via chat-arc) — explicit ask, named consumer
- chat-arc:350 — proves envelope.metadata is on-wire
- chat-arc:351, 383 — Pen acks with empty envelope.metadata (the gap)
- cohort_hub/n8n_event_match.py:42 (consumer-side, in different repo) — matcher keys on metadata fields
- T-1560 (existing) — `termlink_agent_post` already accepts underscore-prefixed metadata keys; this task generalizes

## Steel-man Design Shapes (from cohort-agent's letter)

**Shape 1 — Free-form metadata object (RECOMMENDED).** Caller passes
`metadata: {thread, in_reply_to, conversation_id, …}` as arbitrary JSON.
Tool layer pass-through; contracts define their own conventions.
- Antifragility ✓ Reliability ✓ Usability ✓ Portability ✓
- Risk: contracts disagree on key names. Mitigation: per-contract convention docs.

**Shape 2 — Structured with reserved keys.** Tool layer defines a fixed
schema (thread, in_reply_to, mentions) and rejects unknown keys.
- Antifragility ◌ Reliability ✓ Usability ✗ Portability ✗

**Shape 3 — Per-contract typed MCP tools.** `termlink_channel_post_reply(parent_offset, thread, …)`, etc.
- Reliability ✓ Usability ◌ Portability ✗ (proliferation)

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

**Rationale**: Recommendation: GO

Rationale: Confirmed gap with named consumer impact (cohort-agent n8n exec 7
parked). Pure additive MCP-schema change, no protocol version bump, no
breaking change to existing callers. Cohort-agent recommends Shape 1 (free-form
object) — matches existing protocol design, lets contracts evolve independently,
no schema lock-in at tool layer. Implementation likely <1 hour. Sibling audit
task ("MCP-vs-protocol surface gap") is the class fix; this one closes the
named instance.

Evidence:
- Cohort-agent letter (operator-relayed via chat-arc) — explicit ask, named consumer
- chat-arc:350 — proves envelope.metadata is on-wire
- chat-arc:351, 383 — Pen acks with empty envelope.metadata (the gap)
- cohort_hub/n8n_event_match.py:42 (consumer-side, in different repo) — matcher keys on metadata fields
- T-1560 (existing) — `termlink_agent_post` already accepts underscore-prefixed metadata keys; this task generalizes

**Date**: 2026-05-18T21:02:13Z

## Updates

<!-- Auto-populated by git mining at task completion.
     Manual entries optional during execution. -->

### 2026-05-18T09:27:25Z — status-update [task-update-agent]
- **Change:** status: captured → started-work

### 2026-05-18T21:02:13Z — inception-decision [inception-workflow]
- **Action:** Recorded inception decision
- **Decision:** GO
- **Rationale:** Recommendation: GO

Rationale: Confirmed gap with named consumer impact (cohort-agent n8n exec 7
parked). Pure additive MCP-schema change, no protocol version bump, no
breaking change to existing callers. Cohort-agent recommends Shape 1 (free-form
object) — matches existing protocol design, lets contracts evolve independently,
no schema lock-in at tool layer. Implementation likely <1 hour. Sibling audit
task ("MCP-vs-protocol surface gap") is the class fix; this one closes the
named instance.

Evidence:
- Cohort-agent letter (operator-relayed via chat-arc) — explicit ask, named consumer
- chat-arc:350 — proves envelope.metadata is on-wire
- chat-arc:351, 383 — Pen acks with empty envelope.metadata (the gap)
- cohort_hub/n8n_event_match.py:42 (consumer-side, in different repo) — matcher keys on metadata fields
- T-1560 (existing) — `termlink_agent_post` already accepts underscore-prefixed metadata keys; this task generalizes

## Reviewer Verdict (v1.4)

- **Scan ID:** R-cc17c6af
- **Timestamp:** 2026-05-18T21:02:14Z
- **Catalogue:** v1.3-seed
- **Overall:** PASS
- **Needs Human:** no
- **Findings:** none

### 2026-05-18T21:02:14Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
- **Reason:** Inception decision: GO
