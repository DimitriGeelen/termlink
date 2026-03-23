---
id: T-221
name: "Commit Cargo.lock for reproducible builds"
description: >
  Commit Cargo.lock for reproducible builds

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-03-22T21:24:55Z
last_update: 2026-03-23T07:48:00Z
date_finished: 2026-03-23T07:48:00Z
---

# T-221: Commit Cargo.lock for reproducible builds

## Context

Per Rust convention, binary crates should commit Cargo.lock for reproducible builds. TermLink is a binary — CI and release builds should use exact same dependency versions. Currently gitignored.

## Acceptance Criteria

### Agent
- [x] Cargo.lock removed from .gitignore
- [x] Cargo.lock tracked in git

## Verification

! grep -q '^Cargo.lock' .gitignore
test -f Cargo.lock
git ls-files --error-unmatch Cargo.lock

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

### 2026-03-22T21:24:55Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-221-commit-cargolock-for-reproducible-builds.md
- **Context:** Initial task creation

### 2026-03-23T07:48:00Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
