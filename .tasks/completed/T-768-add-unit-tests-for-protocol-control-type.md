---
id: T-768
name: "Add unit tests for protocol control types — CommonParams, TerminalInfo, Capabilities serde"
description: >
  Add unit tests for protocol control types — CommonParams, TerminalInfo, Capabilities serde

status: work-completed
workflow_type: test
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-03-29T23:07:10Z
last_update: 2026-03-29T23:08:23Z
date_finished: 2026-03-29T23:08:23Z
---

# T-768: Add unit tests for protocol control types — CommonParams, TerminalInfo, Capabilities serde

## Context

`control.rs` defines `CommonParams`, `TerminalInfo`, `Capabilities`, and `KeyEntry` types. Only `KeyEntry` serialization and `Capabilities` default are tested. The other types have no serde roundtrip tests.

## Acceptance Criteria

### Agent
- [x] Add serde roundtrip test for `CommonParams` (with and without optional fields)
- [x] Add serde roundtrip test for `TerminalInfo` (with and without optional fields)
- [x] Add `Capabilities` serde roundtrip test with non-default values
- [x] Add `KeyEntry` variant-specific deserialization tests
- [x] Add test verifying method name constants match expected strings
- [x] All new tests pass via `cargo test -p termlink-protocol control`

## Verification

cargo test -p termlink-protocol control

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

### 2026-03-29T23:07:10Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-768-add-unit-tests-for-protocol-control-type.md
- **Context:** Initial task creation

### 2026-03-29T23:08:23Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
