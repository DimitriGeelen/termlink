---
id: T-167
name: "Agent message protocol — request/response over events"
description: >
  Design and implement agent.request/agent.response/agent.status event schemas for bidirectional agent-to-agent communication over TermLink events.

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: [protocol, agent-comms]
components: []
related_tasks: []
created: 2026-03-18T10:08:36Z
last_update: 2026-03-18T17:16:57Z
date_finished: 2026-03-18T17:16:57Z
---

# T-167: Agent message protocol — request/response over events

## Context

General-purpose agent-to-agent request/response protocol built on TermLink events. Extends the existing task delegation events with `agent.request`, `agent.response`, and `agent.status` schemas for arbitrary bidirectional communication with correlation tracking.

## Acceptance Criteria

### Agent
- [x] `AgentRequest` struct: request_id (ULID), from, to, action, params, timeout_secs
- [x] `AgentResponse` struct: request_id, from, status (ok/error), result, error_message
- [x] `AgentStatus` struct: request_id, from, phase, message, progress percent
- [x] Topic constants: `agent.request`, `agent.response`, `agent.status`
- [x] Serde roundtrip tests for all three types
- [x] Integration test: emit request → emit response → poll matches by request_id
- [x] All workspace tests pass

## Verification

bash -c 'out=$(/Users/dimidev32/.cargo/bin/cargo test --workspace 2>&1); echo "$out" | grep -cq "0 failed"'
grep -q "AgentRequest" crates/termlink-protocol/src/events.rs
grep -q "AgentResponse" crates/termlink-protocol/src/events.rs
grep -q "AgentStatus" crates/termlink-protocol/src/events.rs

## Decisions

<!-- Record decisions ONLY when choosing between alternatives.
     Skip for tasks with no meaningful choices.
     Format:
     ### [date] — [topic]
     - **Chose:** [what was decided]
     - **Why:** [rationale]
     - **Rejected:** [alternatives and why not]
-->

## Updates

### 2026-03-18T10:08:36Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /Users/dimidev32/001-projects/010-termlink/.tasks/active/T-167-agent-message-protocol--requestresponse-.md
- **Context:** Initial task creation

### 2026-03-18T16:43:53Z — status-update [task-update-agent]
- **Change:** status: captured → started-work

### 2026-03-18T17:16:57Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
