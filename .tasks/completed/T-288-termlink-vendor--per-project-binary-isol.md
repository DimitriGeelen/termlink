---
id: T-288
name: "termlink vendor — per-project binary isolation (same pattern as framework .agentic-framework/)"
description: >
  termlink vendor — per-project binary isolation (same pattern as framework .agentic-framework/)

status: work-completed
workflow_type: build
owner: human
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-03-25T22:13:57Z
last_update: 2026-04-15T15:27:19Z
date_finished: 2026-03-25T22:30:08Z
---

# T-288: termlink vendor — per-project binary isolation (same pattern as framework .agentic-framework/)

## Context

T-287 inception found TermLink has no path isolation — one global binary shared by all projects. Framework uses `.agentic-framework/` vendoring. TermLink needs the same: `.termlink/bin/termlink` per project.

## Acceptance Criteria

### Agent
- [x] `termlink vendor --help` shows usage
- [x] `termlink vendor` copies current binary to `.termlink/bin/termlink` in project dir
- [x] `termlink vendor --dry-run` shows what would happen without copying
- [x] `termlink vendor --source /path/to/binary` copies a specific binary
- [x] `termlink vendor --target /path/to/project` vendors into a specific project
- [x] `termlink vendor status` shows vendor state (version, path, drift from global)
- [x] Vendored binary is executable
- [x] `.termlink/VERSION` records the vendored version
- [x] Warns if `.gitignore` doesn't exclude `.termlink/bin/`
- [x] All existing tests pass (`cargo test --workspace`)
- [x] 0 compiler warnings

### Human
- [x] [REVIEW] Vendor into the upgrade-test clone on .107, verify vendored binary works
  **Steps:**
  1. `termlink remote exec mint fw-master "termlink vendor --target /tmp/termlink-upgrade-test"`
  2. `termlink remote exec mint fw-master "/tmp/termlink-upgrade-test/.termlink/bin/termlink --version"`
  **Expected:** Binary copied, version matches, runs correctly
  **If not:** Check `.termlink/bin/` exists and binary has +x permission

## Verification

/Users/dimidev32/.cargo/bin/cargo test --workspace

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

### 2026-03-25T22:13:57Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /Users/dimidev32/001-projects/010-termlink/.tasks/active/T-288-termlink-vendor--per-project-binary-isol.md
- **Context:** Initial task creation

### 2026-03-25T22:30:08Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
