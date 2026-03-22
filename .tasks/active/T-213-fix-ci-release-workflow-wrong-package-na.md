---
id: T-213
name: "Fix CI release workflow: wrong package name termlink-cli → termlink"
description: >
  Fix CI release workflow: wrong package name termlink-cli → termlink

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-03-22T17:08:08Z
last_update: 2026-03-22T17:08:08Z
date_finished: null
---

# T-213: Fix CI release workflow: wrong package name termlink-cli → termlink

## Context

release.yml uses `-p termlink-cli` but the CLI package is named `termlink` in Cargo.toml. This will cause all CI release builds to fail.

## Acceptance Criteria

### Agent
- [x] release.yml uses `-p termlink` (not `-p termlink-cli`)
- [x] `cargo build --release -p termlink` succeeds locally

## Verification

grep -q "\-p termlink " .github/workflows/release.yml
! grep -q "termlink-cli" .github/workflows/release.yml

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

### 2026-03-22T17:08:08Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-213-fix-ci-release-workflow-wrong-package-na.md
- **Context:** Initial task creation
