---
id: T-216
name: "Strengthen CI — add tests and clippy to workflow"
description: >
  Strengthen CI — add tests and clippy to workflow

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-03-22T21:12:37Z
last_update: 2026-03-22T21:12:37Z
date_finished: null
---

# T-216: Strengthen CI — add tests and clippy to workflow

## Context

CI workflow (.github/workflows/ci.yml) currently only runs `cargo check`. Adding `cargo clippy` and `cargo test` catches regressions earlier. The codebase already has 277 passing tests and 0 clippy warnings.

## Acceptance Criteria

### Agent
- [x] CI workflow runs `cargo clippy --workspace -- -D warnings`
- [x] CI workflow runs `cargo test --workspace`
- [x] Clippy component added to Rust toolchain install step

## Verification

grep -q 'clippy' .github/workflows/ci.yml
grep -q 'cargo test' .github/workflows/ci.yml

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

### 2026-03-22T21:12:37Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-216-strengthen-ci--add-tests-and-clippy-to-w.md
- **Context:** Initial task creation
