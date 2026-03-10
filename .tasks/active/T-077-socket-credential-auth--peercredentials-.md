---
id: T-077
name: "Socket-credential auth — PeerCredentials, UID check on accept"
description: >
  Add auth.rs to termlink-session with PeerCredentials struct. Extract peer UID via SO_PEERCRED (Linux) / LOCAL_PEERCRED (macOS) on socket accept. Reject connections from different UID. Addresses G-002.

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-03-10T20:43:09Z
last_update: 2026-03-10T20:45:45Z
date_finished: null
---

# T-077: Socket-credential auth — PeerCredentials, UID check on accept

## Context

Phase 1 of security model (T-008 inception GO). Adds PeerCredentials extraction on socket accept, UID comparison against session owner. See [docs/reports/T-008-security-model-inception.md].

## Acceptance Criteria

### Agent
- [x] `auth.rs` module in termlink-session with `PeerCredentials` struct (uid, gid, pid)
- [x] Cross-platform credential extraction: SO_PEERCRED (Linux) + LOCAL_PEERCRED/LOCAL_PEERPID (macOS)
- [x] Session server rejects connections from different UID with AUTH_DENIED error
- [x] Hub server rejects connections from different UID with AUTH_DENIED error
- [x] Same-UID connections continue to work (no behavior change for single-user)
- [x] Unit tests for PeerCredentials extraction
- [x] All existing tests pass (cargo test --workspace)

## Verification

/Users/dimidev32/.cargo/bin/cargo build -p termlink-session 2>&1 | tail -1
/Users/dimidev32/.cargo/bin/cargo test -p termlink-session 2>&1 | tail -3
/Users/dimidev32/.cargo/bin/cargo test -p termlink-hub 2>&1 | tail -3
/Users/dimidev32/.cargo/bin/cargo test -p termlink 2>&1 | tail -3

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

### 2026-03-10T20:43:09Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /Users/dimidev32/001-projects/010-termlink/.tasks/active/T-077-socket-credential-auth--peercredentials-.md
- **Context:** Initial task creation

### 2026-03-10T20:45:45Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
