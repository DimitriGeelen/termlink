---
id: T-249
name: "Bypass command validation — denylist and caller identity"
description: >
  No restrictions on what command strings can be promoted to bypass. Any session can self-promote arbitrary strings including destructive commands. Add command denylist (pattern-based), optional caller diversity requirement, and metadata clarifying bypass != execution authorization. See docs/reports/T-247-scenarios-adversarial.md Scenario 2.

status: captured
workflow_type: build
owner: agent
horizon: now
tags: [T-247, T-238, orchestration, bypass, security]
components: []
related_tasks: [T-247, T-238, T-233]
created: 2026-03-23T16:54:20Z
last_update: 2026-03-23T16:54:20Z
date_finished: null
---

# T-249: Bypass command validation — denylist and caller identity

## Context

Security gap found by adversarial scenario agent in T-247. Any session can promote arbitrary command strings to bypass tier, including destructive commands. See `docs/reports/T-247-scenarios-adversarial.md` Scenario 2. Modified files: `crates/termlink-hub/src/bypass.rs`.

## Acceptance Criteria

### Agent
- [ ] Command denylist in BypassRegistry (pattern-based, e.g. rm, drop, delete, force-push)
- [ ] `record_orchestrated_run` rejects denylisted commands (returns false, logs warning)
- [ ] Bypass response includes `note: "routing shortcut, not execution authorization"` metadata
- [ ] Test: denylisted command not promotable after 10+ successful runs
- [ ] Test: denylist patterns match substrings and regex
- [ ] All hub tests pass

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
