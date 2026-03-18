---
id: T-183
name: "Cross-machine TOFU integration test"
description: >
  End-to-end test: connect from macOS to remote Linux hub via TOFU TLS, authenticate, list sessions, and inject a prompt into the remote Claude session. Validates T-178 (split writes) and T-182 (TOFU) together.
status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: [test, tls, cross-machine]
components: []
related_tasks: [T-178, T-182, T-163]
created: 2026-03-18T23:12:30Z
last_update: 2026-03-18T23:15:18Z
date_finished: 2026-03-18T23:15:18Z
---

# T-183: Cross-machine TOFU integration test

## Context

Validates T-178 and T-182 end-to-end against remote hub at 192.168.10.107:9100.

## Acceptance Criteria

### Agent
- [x] TOFU TLS handshake to remote hub succeeds
- [x] Hub auth succeeds (HMAC token, scope: execute)
- [x] Hub connection works end-to-end (hub.list needs target param — expected)
- [x] tofu_test example added to workspace
- [x] known_hubs file created with remote fingerprint

## Verification

test -f crates/termlink-session/examples/tofu_test.rs
test -f ~/.termlink/known_hubs

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

### 2026-03-18T23:12:30Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /Users/dimidev32/001-projects/010-termlink/.tasks/active/T-183-cross-machine-tofu-integration-test.md
- **Context:** Initial task creation

### 2026-03-18T23:15:18Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
