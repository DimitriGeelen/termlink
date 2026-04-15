---
id: T-1032
name: "Fix hub status — use resolve_hub_paths for split-brain runtime dir"
description: >
  hub status uses default hub_pidfile_path(), missing hubs at /var/lib/termlink. Apply same resolve_hub_paths() pattern from T-1031.

status: started-work
workflow_type: build
owner: human
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-04-13T14:07:16Z
last_update: 2026-04-13T14:20:44Z
date_finished: null
---

# T-1032: Fix hub status — use resolve_hub_paths for split-brain runtime dir

## Context

<!-- One sentence for small tasks. Link to design docs for substantial ones. -->

## Acceptance Criteria

### Agent
- [x] `cmd_hub_status` uses `resolve_hub_paths()` instead of default paths
- [x] Shows correct PID and runtime dir for systemd-managed hubs
- [x] Builds and passes clippy

### Human
- [ ] [RUBBER-STAMP] Verify `termlink hub status` shows systemd hub
  **Steps:** `cd /opt/termlink && cargo run -- hub status`
  **Expected:** Shows running PID matching systemctl, runtime dir /var/lib/termlink
  **If not:** Check resolve_hub_paths() fallback

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

### 2026-04-13T14:07:16Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1032-fix-hub-status--use-resolvehubpaths-for-.md
- **Context:** Initial task creation
