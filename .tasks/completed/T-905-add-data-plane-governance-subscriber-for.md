---
id: T-905
name: "Add data plane governance subscriber for post-hoc pattern detection"
description: >
  Add data plane governance subscriber for post-hoc pattern detection

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-04-08T06:56:20Z
last_update: 2026-04-11T14:33:32Z
date_finished: 2026-04-11T14:33:32Z
---

# T-905: Add data plane governance subscriber for post-hoc pattern detection

## Context

Add a governance subscriber to the data plane that watches Output frames for configurable regex patterns and emits Governance frames for audit/metrics. Non-blocking, opt-in, post-hoc detection.

## Acceptance Criteria

### Agent
- [x] Governance frame type (0x8) added to FrameType enum in data.rs
- [x] GovernanceEvent payload struct defined in termlink-protocol with pattern_name, match_text, timestamp
- [x] GovernanceSubscriber struct in termlink-session that receives Output frames via broadcast channel
- [x] Subscriber strips ANSI before matching configurable regex patterns
- [x] Subscriber emits Governance frames when patterns match
- [x] Subscriber is non-blocking (async processing, bounded channel)
- [x] Tests pass: pattern matching, governance frame emission, ANSI stripping
- [x] cargo test passes for termlink-protocol and termlink-session crates

## Verification

cd /opt/termlink && cargo test -p termlink-protocol 2>&1 | tail -3
cd /opt/termlink && cargo test -p termlink-session --lib -- governance 2>&1 | tail -3

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

### 2026-04-08T06:56:20Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-905-add-data-plane-governance-subscriber-for.md
- **Context:** Initial task creation

### 2026-04-11T14:33:32Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
