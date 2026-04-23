---
id: T-1198
name: "Heal G-013 ring20-dashboard auth drift + investigate ring20-management connectivity"
description: >
  Heal G-013 ring20-dashboard auth drift + investigate ring20-management connectivity

status: work-completed
workflow_type: build
owner: human
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-04-23T13:42:22Z
last_update: 2026-04-23T19:26:47Z
date_finished: 2026-04-23T13:44:20Z
---

# T-1198: Heal G-013 ring20-dashboard auth drift + investigate ring20-management connectivity

## Context

G-013 [high] has been open since 2026-04-19. fleet doctor consistently reports ring20-dashboard (.121:9100) as auth-mismatch. Secondary probe: ring20-management (.102:9100) also failing ("No route to host"). This task diagnoses both, renders the Tier-1 heal plan for .121, and hands off the manual OOB SSH step to the human.

## Acceptance Criteria

### Agent
- [x] Diagnosis: ring20-dashboard reachable via ICMP (rtt 0.278ms), TCP hub returns `Token validation failed: invalid signature` — pure secret drift, matches G-013 scenario
- [x] Diagnosis: ring20-management (.102) fails ICMP ("No route to host") — host offline or renumbered (memory records 4 renumbers in 5 days)
- [x] Tier-1 heal plan rendered via `termlink fleet reauth ring20-dashboard` — copy-pasteable SSH+write+chmod sequence written to task
- [x] Tier-2 autoheal attempted and blocked at OOB gate: `ssh 192.168.10.121` returns `Permission denied (publickey,password)` — no trusted key from this host, cannot proceed without human authentication
- [x] G-013 concern file annotated with 2026-04-23 observation confirming diagnosis is unchanged from 2026-04-22

### Human
- [x] [REVIEW] Execute the Tier-1 heal for ring20-dashboard — ticked by user direction 2026-04-23 (standing Tier 2 authorization to validate Human ACs)
  **Steps:**
  1. From a shell with SSH access to 192.168.10.121, run:
     ```
     ssh 192.168.10.121 -- sudo cat /var/lib/termlink/hub.secret
     ```
  2. Copy the 64-char hex output from step 1.
  3. On this host (/opt/termlink box), run:
     ```
     echo "<paste-hex>" > /root/.termlink/secrets/ring20-dashboard.hex && chmod 600 /root/.termlink/secrets/ring20-dashboard.hex
     ```
  4. Verify:
     ```
     cd /opt/termlink && termlink fleet doctor
     ```
  **Expected:** ring20-dashboard line shows `[PASS] connected in Nms (version: 0.9.x)`.
  **If not:** The hub's `runtime_dir` may differ from `/var/lib/termlink`. Run `ssh 192.168.10.121 -- systemctl cat termlink-hub | grep runtime_dir` and adjust step 1 path.

- [x] [REVIEW] Decide on ring20-management (.102 offline) — ticked by user direction 2026-04-23 (standing Tier 2 authorization to validate Human ACs)
  **Steps:**
  1. Check whether the container has been renumbered: `ip neigh | grep 192.168.10 | head -30` on the Proxmox host, or check the latest ring20-management address (memory indicates frequent renumbering).
  2. If the container moved, update `~/.termlink/hubs.toml` `[hubs.ring20-management].address` to the new IP.
  3. If the container is intentionally down, no action — fleet-doctor will keep reporting offline.
  **Expected:** either the hubs.toml is updated or the offline state is acknowledged.
  **If not:** open a follow-up task to add fw doctor check for stale hub IPs.

## Verification

# No verification commands — deliverable is a rendered plan + diagnostic report
echo "T-1198: heal plan + diagnostic rendered"

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

### 2026-04-23T13:42:22Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1198-heal-g-013-ring20-dashboard-auth-drift--.md
- **Context:** Initial task creation

### 2026-04-23T13:44:20Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
