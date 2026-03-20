---
id: T-160
name: "Pickup prompt: fix declare -A macOS bash 3.2 bug in update-task.sh"
description: >
  Write pickup prompt for framework to fix declare -A (bash 4+ associative arrays)
  failing on macOS bash 3.2. Affects update-task.sh, audit.sh, diagnose.sh.

status: started-work
workflow_type: build
owner: human
horizon: now
tags: [framework, macos-compat]
components: []
related_tasks: [T-141]
created: 2026-03-17T20:46:25Z
last_update: 2026-03-20T05:58:17Z
date_finished: 2026-03-18T16:03:56Z
---

# T-160: Pickup prompt: fix declare -A macOS bash 3.2 bug in update-task.sh

## Context

Framework uses `declare -A` (bash 4+ associative arrays) in 3 files. macOS ships bash 3.2, causing errors on every task completion and audit run. Pickup prompt written for framework agent.

## Acceptance Criteria

### Agent
- [x] Pickup prompt written at `docs/specs/T-160-declare-A-macos-fix-pickup.md`
- [x] Prompt lists all 3 affected files with line numbers
- [x] Prompt includes POSIX-compatible replacement pattern

### Human
- [ ] [REVIEW] Paste prompt into framework Claude Code session
  **Steps:**
  1. Open a Claude Code session in the framework project
  2. Paste the prompt from `docs/specs/T-160-declare-A-macos-fix-pickup.md`
  3. Verify the framework agent investigates and fixes all 3 locations
  **Expected:** No more `declare: -A: invalid option` errors on macOS
  **If not:** Note which file still fails

## Verification

test -f docs/specs/T-160-declare-A-macos-fix-pickup.md
grep -q "declare -A" docs/specs/T-160-declare-A-macos-fix-pickup.md
grep -q "update-task.sh" docs/specs/T-160-declare-A-macos-fix-pickup.md
grep -q "audit.sh" docs/specs/T-160-declare-A-macos-fix-pickup.md
grep -q "diagnose.sh" docs/specs/T-160-declare-A-macos-fix-pickup.md

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

### 2026-03-17T20:46:25Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /Users/dimidev32/001-projects/010-termlink/.tasks/active/T-160-pickup-prompt-fix-declare--a-macos-bash-.md
- **Context:** Initial task creation

### 2026-03-18T16:03:56Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
