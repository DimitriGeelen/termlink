---
id: T-249
name: "Bypass command validation — denylist and caller identity"
description: >
  No restrictions on what command strings can be promoted to bypass. Any session can self-promote arbitrary strings including destructive commands. Add command denylist (pattern-based), optional caller diversity requirement, and metadata clarifying bypass != execution authorization. See docs/reports/T-247-scenarios-adversarial.md Scenario 2.

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: [T-247, T-238, orchestration, bypass, security]
components: []
related_tasks: [T-247, T-238, T-233]
created: 2026-03-23T16:54:20Z
last_update: 2026-03-23T20:46:51Z
date_finished: 2026-03-23T20:46:51Z
---

# T-249: Bypass command validation — denylist and caller identity

## Context

Security gap found by adversarial scenario agent in T-247. Any session can promote arbitrary command strings to bypass tier, including destructive commands. See `docs/reports/T-247-scenarios-adversarial.md` Scenario 2. Modified files: `crates/termlink-hub/src/bypass.rs`.

## Acceptance Criteria

### Agent
- [x] Command denylist in BypassRegistry (15 case-insensitive substring patterns: rm, drop, delete, force-push, reset --hard, kill, etc.)
- [x] `record_orchestrated_run` rejects denylisted commands (returns false, logs warning)
- [x] Bypass response includes `note: "routing shortcut, not execution authorization"` metadata
- [x] Test: denylisted command not promotable after 10+ successful runs
- [x] Test: denylist patterns match 7 dangerous commands, reject 5 safe commands
- [x] All 64 hub tests pass

## Verification

/Users/dimidev32/.cargo/bin/cargo test --package termlink-hub

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

### 2026-03-23T16:54:20Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /Users/dimidev32/001-projects/010-termlink/.tasks/active/T-249-bypass-command-validation--denylist-and-.md
- **Context:** Initial task creation

### 2026-03-23T20:15:04Z — status-update [task-update-agent]
- **Change:** status: captured → started-work

### 2026-03-23T20:46:51Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
