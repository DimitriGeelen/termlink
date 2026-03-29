---
id: T-760
name: "Update Homebrew formula — add Linux aarch64 variant to match release workflow"
description: >
  Update Homebrew formula — add Linux aarch64 variant to match release workflow

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-03-29T20:06:21Z
last_update: 2026-03-29T20:06:21Z
date_finished: null
---

# T-760: Update Homebrew formula — add Linux aarch64 variant to match release workflow

## Context

Homebrew formula only has Linux x86_64. Needs Linux aarch64 variant to match T-759 release workflow update.

## Acceptance Criteria

### Agent
- [x] Linux `on_linux` block split into arm? and x86_64 variants
- [x] Linux aarch64 URL points to termlink-linux-aarch64 release asset
- [x] Formula structure validated: 4 platform variants (darwin-aarch64, darwin-x86_64, linux-x86_64, linux-aarch64)

## Verification

grep -q "termlink-linux-aarch64" homebrew/Formula/termlink.rb

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

### 2026-03-29T20:06:21Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-760-update-homebrew-formula--add-linux-aarch.md
- **Context:** Initial task creation
