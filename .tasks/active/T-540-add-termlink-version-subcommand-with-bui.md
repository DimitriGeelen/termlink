---
id: T-540
name: "Add termlink version subcommand with build info"
description: >
  Add termlink version subcommand with build info

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-03-28T09:10:39Z
last_update: 2026-03-28T09:10:39Z
date_finished: null
---

# T-540: Add termlink version subcommand with build info

## Context

`termlink --version` works but `termlink version` doesn't. Add a `version` subcommand showing version, git commit, and build target.

## Acceptance Criteria

### Agent
- [x] `termlink version` outputs version info
- [x] `termlink version --json` outputs JSON with version, commit, target
- [x] build.rs embeds GIT_COMMIT and TARGET at compile time
- [x] `cargo build` and tests pass

## Verification

cargo build 2>&1
./target/debug/termlink version 2>&1 | grep -q "termlink"

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

### 2026-03-28T09:10:39Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-540-add-termlink-version-subcommand-with-bui.md
- **Context:** Initial task creation
