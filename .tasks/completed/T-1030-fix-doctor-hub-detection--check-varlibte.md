---
id: T-1030
name: "Fix doctor hub detection — check /var/lib/termlink when default runtime dir has no pidfile"
description: >
  termlink doctor reports hub as not running when hub runs from /var/lib/termlink/ (systemd) but doctor checks /tmp/termlink-0/ (default runtime_dir). Add fallback check for alternate runtime dir.

status: work-completed
workflow_type: build
owner: human
horizon: now
tags: []
components: [crates/termlink-cli/src/commands/infrastructure.rs]
related_tasks: []
created: 2026-04-13T13:46:49Z
last_update: 2026-04-23T19:17:25Z
date_finished: 2026-04-23T19:17:25Z
---

# T-1030: Fix doctor hub detection — check /var/lib/termlink when default runtime dir has no pidfile

## Context

<!-- One sentence for small tasks. Link to design docs for substantial ones. -->

## Acceptance Criteria

### Agent
- [x] Doctor checks alternate runtime dir (/var/lib/termlink) when default has no pidfile
- [x] Displays hub status correctly when hub runs from alternate dir (shows "via /var/lib/termlink")
- [x] Builds and passes clippy
- [x] Existing doctor tests pass

### Human
- [x] [REVIEW] Verify `termlink doctor` detects systemd hub — ticked by user direction 2026-04-23. Evidence: Live: `termlink doctor` shows '✓ hub: running (PID 1718329), responding' — detects the locally-running hub including systemd-launched ones via /var/lib/termlink pidfile fallback. User direction 2026-04-23.
  **Steps:**
  1. `cd /opt/termlink && cargo run -- doctor`
  2. Verify hub line shows "running (PID ...)" not "not running"
  **Expected:** Hub detected from /var/lib/termlink runtime dir
  **If not:** Check if hub.pid exists in /var/lib/termlink/


**Agent evidence (auto-batch 2026-04-19, G-008 remediation, hub-detection, doctor-detects-systemd-hub):** Live: `termlink doctor` on this host reports `hub: running (PID 2861), responding`. The running hub is the systemd-managed one at `/var/lib/termlink` — doctor correctly detects it via the split-brain-aware resolver (not the default /tmp path). REVIEW-approvable.

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

### 2026-04-13T13:46:49Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1030-fix-doctor-hub-detection--check-varlibte.md
- **Context:** Initial task creation

### 2026-04-23T19:17:25Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
- **Reason:** Completed via Watchtower UI (human action)
