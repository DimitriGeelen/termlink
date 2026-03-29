---
id: T-758
name: "Optimize release binary — add LTO, strip symbols, single codegen unit"
description: >
  Optimize release binary — add LTO, strip symbols, single codegen unit

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-03-29T19:54:54Z
last_update: 2026-03-29T19:54:54Z
date_finished: null
---

# T-758: Optimize release binary — add LTO, strip symbols, single codegen unit

## Context

Release binary is 18MB with default profile. Adding LTO, stripping, and single codegen unit should reduce size significantly.

## Acceptance Criteria

### Agent
- [x] `[profile.release]` section added to workspace Cargo.toml with lto, strip, codegen-units
- [x] Release binary builds successfully
- [x] Release binary is 12MB (down from 18MB — 33% reduction)
- [x] All 528 tests pass

## Verification

grep -q "profile.release" Cargo.toml
test -f target/release/termlink

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

### 2026-03-29T19:54:54Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-758-optimize-release-binary--add-lto-strip-s.md
- **Context:** Initial task creation
