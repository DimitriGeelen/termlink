---
id: T-679
name: "Make list --count and discover --count respect --json flag"
description: >
  Make list --count and discover --count respect --json flag

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-03-28T22:51:00Z
last_update: 2026-03-28T22:51:00Z
date_finished: null
---

# T-679: Make list --count and discover --count respect --json flag

## Context

`list --count --json` and `discover --count --json` output bare numbers, ignoring the --json flag. Should output `{"count": N}` when --json is set.

## Acceptance Criteria

### Agent
- [x] `list --count --json` outputs `{"count": N}` instead of bare number
- [x] `discover --count --json` outputs `{"count": N}` instead of bare number
- [x] Without --json, both still output bare number
- [x] Project compiles cleanly

## Verification

grep -q '"count"' /opt/termlink/crates/termlink-cli/src/commands/session.rs

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

### 2026-03-28T22:51:00Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-679-make-list---count-and-discover---count-r.md
- **Context:** Initial task creation
