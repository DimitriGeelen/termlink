---
id: T-675
name: "Add uid, created_at, heartbeat_at, metadata to discover --json output"
description: >
  Add uid, created_at, heartbeat_at, metadata to discover --json output

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-03-28T22:42:57Z
last_update: 2026-03-28T22:42:57Z
date_finished: null
---

# T-675: Add uid, created_at, heartbeat_at, metadata to discover --json output

## Context

`discover --json` (metadata.rs) is missing `uid`, `created_at`, `heartbeat_at`, `metadata` fields that `list --json` (session.rs) already includes after T-669/T-671/T-672. Parity fix.

## Acceptance Criteria

### Agent
- [x] discover --json output includes uid field
- [x] discover --json output includes created_at field
- [x] discover --json output includes heartbeat_at field
- [x] discover --json output includes metadata field
- [x] Project compiles cleanly

## Verification

grep -q "heartbeat_at" /opt/termlink/crates/termlink-cli/src/commands/metadata.rs

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

### 2026-03-28T22:42:57Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-675-add-uid-createdat-heartbeatat-metadata-t.md
- **Context:** Initial task creation
