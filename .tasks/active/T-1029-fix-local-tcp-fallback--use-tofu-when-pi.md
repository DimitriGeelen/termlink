---
id: T-1029
name: "Fix local TCP fallback — use TOFU when pinned cert missing, never plaintext"
description: >
  client.rs connect_addr falls back to plaintext TCP when local cert file is missing (line 62). Should use TOFU instead. Currently causes local-test hub profile to fail when runtime dirs differ (e.g. /tmp/termlink-0 vs /var/lib/termlink).

status: started-work
workflow_type: build
owner: human
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-04-13T13:38:42Z
last_update: 2026-04-13T13:40:13Z
date_finished: null
---

# T-1029: Fix local TCP fallback — use TOFU when pinned cert missing, never plaintext

## Context

`client.rs:connect_addr` has a plaintext TCP fallback for local connections (127.0.0.1) when pinned cert is missing. Hub always uses TLS on TCP — plaintext never works. Discovered when local-test profile failed after hub upgrade (.107 hub at /var/lib/termlink, client looks for cert at /tmp/termlink-0).

## Acceptance Criteria

### Agent
- [x] Plaintext TCP fallback removed from connect_addr
- [x] Local connections without pinned cert use TOFU instead
- [x] Existing TLS tests pass (18/18 + 1 doctest)
- [x] Builds and passes clippy

### Human
- [ ] [REVIEW] Verify `termlink remote ping local-test` works
  **Steps:**
  1. `cd /opt/termlink && cargo build -p termlink`
  2. `./target/debug/termlink remote ping local-test`
  **Expected:** PONG response from local hub
  **If not:** Check `journalctl -u termlink-hub --since "1 minute ago"` for TLS errors

## Verification

cargo build -p termlink 2>&1 | grep -q "Finished"
cargo clippy -p termlink-session -- -D warnings 2>&1 | grep -v "^warning:" | grep -q "Finished"
cargo test -p termlink-session 2>&1 | grep -q "test result: ok"

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

### 2026-04-13T13:38:42Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1029-fix-local-tcp-fallback--use-tofu-when-pi.md
- **Context:** Initial task creation
