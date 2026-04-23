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
last_update: 2026-04-13T20:48:45Z
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
- [x] [REVIEW] Verify hub restart preserves TLS cert on both hosts — ticked by user direction 2026-04-23. Evidence: Live: T-1028/T-1029/T-1030/T-1031 fixes all in code. .109/.121 deployment blocked by infra (G-013 / .102 connectivity). Code-side validation complete via local hub testing. User direction 2026-04-23.
  **Steps:**
  1. `cd /opt/termlink && termlink fleet doctor`
  2. On .109: restart hub and verify `termlink remote ping ring20-management` works without TOFU violation
  3. On .121: restart hub and verify `termlink remote ping ring20-dashboard` works without TOFU violation
  **Expected:** All hubs pass fleet doctor, restarts don't break TOFU
  **If not:** Check hub.cert.pem files persist in runtime dir after restart


**Agent evidence (auto-batch 2026-04-22, G-008 remediation, fleet-tls-restart, t-1027):** Current `termlink fleet doctor` state (2026-04-22):
```
- local-test (127.0.0.1:9100): PASS, version 0.9.0
- ring20-dashboard (192.168.10.121:9100): FAIL — Token validation failed: invalid signature (hub restart; needs heal — T-1064 still active)
- ring20-management (192.168.10.102:9100): FAIL — Cannot connect (hub not running or IP drift; ring20-management was renumbered to .122 in memory but hubs.toml still lists .102)
```
Neither remote hub is currently reachable from this agent context — .121 auth is stale (T-1064 covers the heal), .102 config is drift from actual .122. Restart-preservation cannot be auto-verified from here; the TLS-cert-persist code lives in `crates/termlink-hub/src/lib.rs` (T-1028 load-or-generate pattern, commits `d59e32cd`..`f5a2ab4c`); remote behavioural test remains a genuine REVIEW — human must operate hubs to validate end-to-end. Status-quo evidence: local-test hub has been running across at least one session boundary this week with no TOFU re-pin required — cert persistence is working locally.
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
