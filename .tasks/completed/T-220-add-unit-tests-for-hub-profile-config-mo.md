---
id: T-220
name: "Add unit tests for hub profile config module"
description: >
  Add unit tests for hub profile config module

status: work-completed
workflow_type: test
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-03-21T12:59:56Z
last_update: 2026-03-21T13:01:59Z
date_finished: 2026-03-21T13:01:59Z
---

# T-220: Add unit tests for hub profile config module

## Context

`config.rs` handles hub profile resolution (direct address vs TOML profile lookup) with CLI override precedence. No tests exist yet. Add tests for TOML serialization roundtrip, direct address resolution, profile lookup with overrides, and save/load with temp files.

## Acceptance Criteria

### Agent
- [x] Tests for `resolve_hub_profile()` — direct address, CLI overrides, profile lookup, not-found error
- [x] Tests for TOML serialization roundtrip of `HubsConfig`/`HubEntry` + empty deserialize
- [x] Tests for `save_hubs_config()`/`load_hubs_config()` with temp HOME
- [x] All 8 config tests pass

## Verification

/Users/dimidev32/.cargo/bin/cargo test -p termlink config:: --manifest-path /Users/dimidev32/001-projects/010-termlink/Cargo.toml

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

### 2026-03-21T12:59:56Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /Users/dimidev32/001-projects/010-termlink/.tasks/active/T-220-add-unit-tests-for-hub-profile-config-mo.md
- **Context:** Initial task creation

### 2026-03-21T13:01:59Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
