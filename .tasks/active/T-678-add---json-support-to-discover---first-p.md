---
id: T-678
name: "Add --json support to discover --first (parity with list --first --json)"
description: >
  Add --json support to discover --first (parity with list --first --json)

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-03-28T22:48:47Z
last_update: 2026-03-28T22:48:47Z
date_finished: null
---

# T-678: Add --json support to discover --first (parity with list --first --json)

## Context

`discover --first --json` doesn't output JSON — the --first check runs before --json and outputs plain text. Fix to output single JSON object like `list --first --json`.

## Acceptance Criteria

### Agent
- [x] `discover --first --json` outputs a single JSON session object
- [x] `discover --first` (no --json) still outputs display name
- [x] `discover --first --id` still outputs session ID
- [x] No-match case outputs JSON error when --json is set
- [x] Project compiles cleanly

## Verification

grep -q "if json" /opt/termlink/crates/termlink-cli/src/commands/metadata.rs

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

### 2026-03-28T22:48:47Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-678-add---json-support-to-discover---first-p.md
- **Context:** Initial task creation
