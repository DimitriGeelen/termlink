---
id: T-1016
name: "termlink deploy — atomic binary deployment to remote hosts via termlink (no SSH)"
description: >
  Inception: termlink deploy — atomic binary deployment to remote hosts via termlink (no SSH)

status: started-work
workflow_type: inception
owner: human
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-04-13T11:23:11Z
last_update: 2026-04-13T13:42:37Z
date_finished: null
---

# T-1016: termlink deploy — atomic binary deployment to remote hosts via termlink (no SSH)

## Problem Statement

Deploying termlink to remote hosts (.109, .121) currently requires 6+ manual steps with multiple failure modes: binary transfer, atomic swap (Text file busy), hub restart coordination, TOFU re-trust, and auth re-validation. The hub restart is the critical step -- killing the hub severs the deployment channel. Observed in T-1023 (.121 connectivity lost permanently) and T-1027 (both hosts unreachable).

**For whom:** Any operator deploying termlink updates across the fleet.
**Why now:** T-1028 (cert persistence) and T-933 (secret persistence) solved auth/TOFU breakage. The remaining challenge is bounded.

## Assumptions

- A1: Target hosts have systemd-managed hubs (T-931) -- validated on .107, .109, .121
- A2: T-1028 + T-933 make hub restarts safe (cert+secret persist) -- validated on .107
- A3: `remote exec` can run `systemctl restart termlink-hub` -- validated (requires execute scope)
- A4: Binary can be swapped via rename pattern -- validated on .107
- A5: Hub comes back within 5s of systemctl restart -- validated on .107 (sub-1s)

## Exploration Plan

1. [DONE] Current deployment flow analysis -- 5 spikes in docs/reports/T-1016-termlink-deploy.md
2. [DONE] Command design -- connect, transfer, swap, restart, verify
3. [DONE] Prerequisites audit -- T-1028, T-933, systemd unit required
4. [DONE] Edge cases -- bootstrap (first deploy), old binary, hub not running

## Technical Constraints

- Target must have a running hub with valid auth (no bootstrap without SSH)
- Binary must be musl static (cross-distro compatibility)
- systemd unit must exist for automatic restart
- Restart kills all active sessions on target hub

## Scope Fence

**IN:** `termlink deploy <hub-profile>` command -- transfer, swap, restart, verify
**OUT:** First-time bootstrap (requires SSH), rolling fleet deployment, binary building

## Acceptance Criteria

### Agent
- [x] Problem statement validated (T-1023 + T-1027 incidents)
- [x] Assumptions tested (all 5 validated)
- [x] Recommendation written with rationale (GO)

### Human
- [ ] [REVIEW] Review exploration findings and approve go/no-go decision
  **Steps:**
  1. `cd /opt/termlink && bin/fw task review T-1016`
  2. Review recommendation and evidence from T-1023/T-1027
  3. `cd /opt/termlink && bin/fw inception decide T-1016 go --rationale "..."`
  **Expected:** Decision recorded, task completed
  **If not:** Ask agent for clarification

## Go/No-Go Criteria

**GO if:**
- Prerequisites met (T-1028, T-933, systemd units on targets)
- Command is bounded (~100 lines) and follows existing patterns

**NO-GO if:**
- systemd units not deployed on target hosts
- T-1028/T-933 not reliable enough

## Verification

test -f docs/reports/T-1016-termlink-deploy.md

## Recommendation

**Recommendation:** GO

**Rationale:** All prerequisites are met. T-1028 (cert persistence) and T-933 (secret persistence) solve the auth/TOFU breakage. The remaining work is ~100 lines orchestrating: connect, transfer, swap, restart, verify.

**Evidence:**
- T-1023: Manual deployment required 6+ steps with 3 failure modes
- T-1027: Deployment blocked -- both hubs unreachable due to secret/cert rotation
- T-1028 validated: TLS cert persists across restart on .107 (no TOFU re-trust)
- T-933 validated: Hub secret persists (auth tokens stay valid)
- A3 validated: `remote exec` can run `systemctl restart termlink-hub`
- Command design: 5 steps, ~100 lines, follows existing CLI patterns

**Build scope (if GO):**
1. Add `Command::Deploy` variant to cli.rs
2. Implement `cmd_deploy` in remote.rs: connect, send-file, swap, restart, verify
3. Add `termlink_deploy` MCP tool (T-922 codification)
4. Test against local-test hub profile

## Decisions

### 2026-04-13 -- restart mechanism
- **Chose:** systemd restart via `remote exec "systemctl restart termlink-hub"`
- **Why:** Atomic lifecycle management, handles cleanup, auto-restart on failure
- **Rejected:** fork-exec self-restart (complex), manual restart (slow), two-phase (too slow)

## Decision

<!-- Filled at completion via: fw inception decide T-XXX go|no-go --rationale "..." -->

## Updates

<!-- Auto-populated by git mining at task completion.
     Manual entries optional during execution. -->

### 2026-04-13T13:42:37Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
