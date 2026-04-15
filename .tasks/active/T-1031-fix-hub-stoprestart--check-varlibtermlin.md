---
id: T-1031
name: "Fix hub stop/restart — check /var/lib/termlink pidfile when default not found"
description: >
  hub stop and hub restart use hub_pidfile_path() which resolves to default runtime_dir. When hub runs from /var/lib/termlink (systemd), these commands fail to find the running hub. Apply same fallback pattern as T-1030 doctor fix.

status: started-work
workflow_type: build
owner: human
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-04-13T13:48:46Z
last_update: 2026-04-13T14:00:39Z
date_finished: null
---

# T-1031: Fix hub stop/restart — check /var/lib/termlink pidfile when default not found

## Context

<!-- One sentence for small tasks. Link to design docs for substantial ones. -->

## Acceptance Criteria

### Agent
- [x] `cmd_hub_stop` checks /var/lib/termlink when default pidfile not found
- [x] `cmd_hub_restart` checks /var/lib/termlink when default pidfile not found
- [x] Pidfile resolution extracted to shared `resolve_hub_paths()` helper
- [x] Hub restart passes `TERMLINK_RUNTIME_DIR` to spawned process when using alt dir
- [x] Client cert lookup checks /var/lib/termlink when default cert not found (T-1029 improvement)
- [x] Builds and passes clippy

### Human
- [ ] [REVIEW] Verify `termlink hub stop` stops systemd hub
  **Steps:**
  1. Ensure hub is running: `systemctl status termlink-hub`
  2. `cd /opt/termlink && cargo run -- hub stop`
  3. Verify hub stopped: `systemctl status termlink-hub`
  **Expected:** Hub process stopped (systemd shows inactive/dead)
  **If not:** Check if pidfile was found in /var/lib/termlink

## Verification

cargo build -p termlink 2>&1 | grep -q "Finished"
cargo clippy -p termlink -- -D warnings 2>&1 | grep -v "^warning:" | grep -q "Finished"
# Lines starting with # are comments (skipped). Empty lines ignored.
# The completion gate runs each command — if any exits non-zero, completion is blocked.

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

### 2026-04-13T13:48:46Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1031-fix-hub-stoprestart--check-varlibtermlin.md
- **Context:** Initial task creation
