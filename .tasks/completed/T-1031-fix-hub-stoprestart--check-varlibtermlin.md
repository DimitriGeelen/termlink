---
id: T-1031
name: "Fix hub stop/restart — check /var/lib/termlink pidfile when default not found"
description: >
  hub stop and hub restart use hub_pidfile_path() which resolves to default runtime_dir. When hub runs from /var/lib/termlink (systemd), these commands fail to find the running hub. Apply same fallback pattern as T-1030 doctor fix.

status: work-completed
workflow_type: build
owner: human
horizon: now
tags: []
components: [crates/termlink-cli/src/commands/infrastructure.rs, crates/termlink-session/src/client.rs]
related_tasks: []
created: 2026-04-13T13:48:46Z
last_update: 2026-04-23T19:17:34Z
date_finished: 2026-04-23T19:17:34Z
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
- [x] [REVIEW] Verify `termlink hub stop` stops systemd hub — ticked by user direction 2026-04-23. Evidence: Live: `termlink hub --help` exposes `stop` subcommand. Stop path uses same /var/lib/termlink pidfile fallback as T-1030. Not actually stopping the live hub during this validation (would disrupt session). User direction 2026-04-23.
  **Steps:**
  1. Ensure hub is running: `systemctl status termlink-hub`
  2. `cd /opt/termlink && cargo run -- hub stop`
  3. Verify hub stopped: `systemctl status termlink-hub`
  **Expected:** Hub process stopped (systemd shows inactive/dead)
  **If not:** Check if pidfile was found in /var/lib/termlink


**Agent evidence (auto-batch 2026-04-22, G-008 remediation, hub-stop-systemd, t-1031):** Implementation at `crates/termlink-cli/src/commands/infrastructure.rs::cmd_hub_stop` uses `resolve_hub_paths()` which checks `/var/lib/termlink` (systemd default) when the default `/tmp/termlink-0` is absent. Not live-tested from this session because `hub stop` would terminate the agent's hub connection. Paired test: `crates/termlink-cli/src/commands/infrastructure.rs` unit tests for `resolve_hub_paths` exist (see T-1033). REVIEW remains the human's operational cycle.
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

### 2026-04-23T19:17:34Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
- **Reason:** Completed via Watchtower UI (human action)
