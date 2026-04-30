---
id: T-1422
name: "Recover ring20-management hub at .122 — 0.9.1591 swap left hub down"
description: >
  On 2026-04-30T19:55Z deployed 0.9.1591 to .122 via fleet-deploy-binary.sh; staging succeeded, sha verified. Then swapped /usr/local/bin/termlink (backup at .bak) and killed PID 215. Cron watchdog runs every minute and should respawn — but hub at 9100 stayed down 6+ min. Suspect glibc/libc mismatch (binary built on .107). No SSH/console fallback. Operator needs to console into .122 and either rollback (cp /usr/local/bin/termlink.0.9.1542.bak /usr/local/bin/termlink) or run /usr/local/bin/termlink hub start --tcp 0.0.0.0:9100 to read the actual error. See PL-100.

status: issues
workflow_type: build
owner: human
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-04-30T20:03:47Z
last_update: 2026-04-30T20:04:17Z
date_finished: null
---

# T-1422: Recover ring20-management hub at .122 — 0.9.1591 swap left hub down

## Context

<!-- One sentence for small tasks. Link to design docs for substantial ones. -->

## Acceptance Criteria

### Agent
- [ ] Hub at 192.168.10.122:9100 responds again (port open + `termlink fleet doctor` PASS for ring20-management)
- [ ] If recovered via rollback to 0.9.1542: rollback path documented for next attempt
- [ ] If recovered with 0.9.1591 still in place: actual root cause captured (glibc / lib / config) in this task or PL-100
- [ ] PL-100 referenced in T-1421 follow-up (extend fleet-deploy-binary.sh with `--probe` pre-swap dry-run)

### Human
- [ ] [REVIEW] Console into ring20-management LXC and run diagnostic
  **Steps:**
  1. From PVE host: `pct enter <ctid>` (the ring20-management container)
  2. `ls -la /usr/local/bin/termlink* /tmp/termlink-0.9.1591.new /tmp/swap-122.log`
  3. `cat /tmp/swap-122.log` — see how far the swap got
  4. `/usr/local/bin/termlink hub start --tcp 0.0.0.0:9100` — read the actual error message (will fail-fast if binary can't run)
  5. Decide: rollback or fix-forward
     - Rollback: `cp /usr/local/bin/termlink.0.9.1542.bak /usr/local/bin/termlink && /root/proxmox-ring20-management/scripts/ring20-watchdog.sh`
     - Fix-forward: address whatever error step 4 surfaced (e.g. `ldd /usr/local/bin/termlink` for missing libs)
  **Expected:** `termlink fleet doctor` from .107 shows ring20-management PASS again
  **If not:** Capture the error from step 4 in this task's Updates so a follow-up can address it

## Verification

# Verify hub is back up
timeout 5 bash -c "echo > /dev/tcp/192.168.10.122/9100" 2>&1 && echo PORT_OPEN
.agentic-framework/bin/fw 2>/dev/null; timeout 30 termlink fleet doctor 2>&1 | grep -A1 ring20-management | grep -q PASS

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

### 2026-04-30T20:03:47Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1422-recover-ring20-management-hub-at-122--09.md
- **Context:** Initial task creation

### 2026-04-30T20:04:17Z — status-update [task-update-agent]
- **Change:** status: captured → started-work

### 2026-04-30T20:04:17Z — status-update [task-update-agent]
- **Change:** status: started-work → issues
- **Reason:** swap left hub down at 192.168.10.122:9100 — operator console required to diagnose and recover
