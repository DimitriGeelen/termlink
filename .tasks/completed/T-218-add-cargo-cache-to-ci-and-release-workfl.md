---
id: T-218
name: "Add Cargo cache to CI and release workflows"
description: >
  Add Cargo cache to CI and release workflows

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-03-22T21:15:33Z
last_update: 2026-03-22T21:16:33Z
date_finished: 2026-03-22T21:16:33Z
---

# T-218: Add Cargo cache to CI and release workflows

## Context

CI and release workflows compile from scratch every run. Adding Cargo dependency caching via Swatinem/rust-cache reduces build times significantly on subsequent runs.

## Acceptance Criteria

### Agent
- [x] CI workflow uses Swatinem/rust-cache for dependency caching
- [x] Release workflow uses Swatinem/rust-cache for dependency caching (all 3 jobs)

## Verification

grep -q 'rust-cache' .github/workflows/ci.yml
grep -q 'rust-cache' .github/workflows/release.yml

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

### 2026-03-22T21:15:33Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-218-add-cargo-cache-to-ci-and-release-workfl.md
- **Context:** Initial task creation

### 2026-03-22T21:16:33Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
