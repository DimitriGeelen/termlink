---
id: T-1027
name: "Deploy v0.9.815+ to .109 and .121 — includes T-1026 hub.tcp server-side fix"
description: >
  Build musl static binary, deploy to .109 (ring20-management) and .121 (ring20-dashboard) via termlink send-file. Includes T-1026 hub.tcp server-side write fix for reliable hub restart.

status: started-work
workflow_type: build
owner: human
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-04-13T13:21:55Z
last_update: 2026-04-13T19:46:49Z
date_finished: null
---

# T-1027: Deploy v0.9.815+ to .109 and .121 — includes T-1026 hub.tcp server-side fix

## Context

Deploy latest termlink to .109 (ring20-management) and .121 (ring20-dashboard). Includes: T-1026 (hub.tcp server-side write), T-1028 (TLS cert persistence), T-1029 (TOFU fallback for local TCP). Blocked: .109 hub down, .121 hub secret mismatch.

## Acceptance Criteria

### Agent
- [x] Musl static binary built with T-1026 + T-1028 + T-1029 + T-1030 + T-1031
- [x] Binary deployed to .107 (local) — hub running, doctor + fleet doctor pass
- [x] Binary deployed to .109 — v0.9.844 via termlink remote exec + curl, hub restarted, PONG verified 227ms
- [x] Binary deployed to .121 via termlink remote exec + curl — v0.9.844 running, hub restarted, PONG verified 161ms
- [x] Both remote hosts verified — .109 and .121 both respond to ping with correct secrets

### Human
- [ ] [REVIEW] Verify hub restart preserves TLS cert on both hosts
  **Steps:**
  1. `cd /opt/termlink && termlink fleet doctor`
  2. On .109: restart hub and verify `termlink remote ping ring20-management` works without TOFU violation
  3. On .121: restart hub and verify `termlink remote ping ring20-dashboard` works without TOFU violation
  **Expected:** All hubs pass fleet doctor, restarts don't break TOFU
  **If not:** Check hub.cert.pem files persist in runtime dir after restart

## Verification

termlink fleet doctor 2>&1 | grep -q "0 fail"
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

### 2026-04-13T13:21:55Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1027-deploy-v09815-to-109-and-121--includes-t.md
- **Context:** Initial task creation

### 2026-04-13T13:26:17Z — status-update [task-update-agent]
- **Change:** status: started-work → issues
- **Reason:** Both remote hubs unreachable: .109 hub down (port closed), .121 auth mismatch after cert regen. No SSH access. Musl binary built and ready at target/x86_64-unknown-linux-musl/release/termlink (v0.9.809). Will deploy when hubs are back.

### 2026-04-13T19:38:38Z — status-update [task-update-agent]
- **Change:** status: issues → started-work
