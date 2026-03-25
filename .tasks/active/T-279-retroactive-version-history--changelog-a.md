---
id: T-279
name: "Retroactive version history — CHANGELOG and proper semver"
description: >
  Retroactive version history — CHANGELOG and proper semver

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-03-25T14:03:40Z
last_update: 2026-03-25T14:03:40Z
date_finished: null
---

# T-279: Retroactive version history — CHANGELOG and proper semver

## Context

Version has been 0.1.0 since project start despite 8 major feature milestones. Create CHANGELOG.md with retroactive version history and bump workspace version to 0.8.0.

## Acceptance Criteria

### Agent
- [x] CHANGELOG.md created with all 8 version milestones from git history
- [x] Cargo.toml workspace version bumped to 0.8.0
- [x] `cargo build --workspace` passes with new version
- [x] All tests pass (472 pass, 0 fail)

## Verification

/Users/dimidev32/.cargo/bin/cargo test --workspace
grep -q '0.8.0' Cargo.toml
test -f CHANGELOG.md

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

### 2026-03-25T14:03:40Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /Users/dimidev32/001-projects/010-termlink/.tasks/active/T-279-retroactive-version-history--changelog-a.md
- **Context:** Initial task creation
