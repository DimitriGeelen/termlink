---
id: T-1137
name: "Install logrotate on proxmox host .180 — prevent /var/log full cascade (G-009)"
description: >
  Proxmox host 192.168.10.180 has /var/log on a 224M zram0 filesystem at 100%; pveproxy
  access.log is 145M. Full /var/log cascades into LXC container reboot loops (CT 200 /
  ring20-management / .122 rebooted 5× in 5h on 2026-04-19, producing 4 distinct TLS cert
  rotations observed from termlink clients). Structural fix: logrotate config on the pve
  host for /var/log/pveproxy/access.log — rotate daily, keep 3, compressed. Short-term
  mitigation (truncate) is a separate operator action.

status: started-work
workflow_type: build
owner: human
horizon: now
tags: [infrastructure, proxmox, operations]
components: []
related_tasks: [T-1064, T-1028, T-1053]
created: 2026-04-19T08:43:09Z
last_update: 2026-04-26T15:18:24Z
date_finished: null
---

# T-1137: Install logrotate on proxmox host .180 — prevent /var/log full cascade (G-009)

## Context

Proxmox host .180 uses a 224 MiB zram0 filesystem for /var/log. `/var/log/pveproxy/access.log`
filled to 145 MiB, pushing the volume to 100 %. When /var/log fills, PVE host services
degrade; LXC containers get killed/restarted. CT 200 (ring20-management / .122) rebooted
5× in 5h on 2026-04-19, regenerating its TLS cert each time (cbc4 → b855 → 5198d1fb →
b90adf2598), triggering TOFU violations on every termlink client.

Root cause chain: **proxmox /var/log fills → PVE host degrades → CTs reboot → hub
regenerates cert → clients fail to reach .122.**

Termlink-side work (T-1028 persist-certs-on-restart, T-1064 investigation) addresses the
symptom *inside* the container. The structural fix for the cascade is host-side: logrotate
on the proxmox .180 host.

See `.context/project/concerns.yaml` entry G-009 for full diagnosis.

## Acceptance Criteria

### Human
- [x] [REVIEW] logrotate config installed on proxmox .180 for /var/log/pveproxy/access.log
  **Steps:**
  1. `ssh root@192.168.10.180`
  2. Create `/etc/logrotate.d/pveproxy-access` with:
     ```
     /var/log/pveproxy/access.log {
         daily
         rotate 3
         compress
         missingok
         notifempty
         copytruncate
     }
     ```
  3. `logrotate -d /etc/logrotate.d/pveproxy-access` (dry-run; check no errors)
  4. `logrotate -f /etc/logrotate.d/pveproxy-access` (force once to verify)
  **Expected:** access.log is rotated to access.log.1.gz; new access.log is small/empty
  **If not:** Check `/var/log/pveproxy/` permissions and logrotate version

- [ ] [REVIEW] /var/log on proxmox .180 is below 50 % after rotation + daily cron active
  **Steps:**
  1. `ssh root@192.168.10.180 df -h /var/log`
  2. Wait 24h, re-check: `ssh root@192.168.10.180 ls -la /var/log/pveproxy/` — expect access.log.1.gz present
  **Expected:** /var/log < 50 %, one rotated compressed file visible
  **If not:** Check `/etc/cron.daily/logrotate` is enabled, or add a specific cron

- [ ] [REVIEW] CT 200 (.122) stops rebooting
  **Steps:**
  1. After 24h of stable pve host: `ssh root@192.168.10.180 pct status 200`
  2. `ssh root@192.168.10.180 journalctl --list-boots -n 10` (via CT or host — however reachable)
  3. `cd /opt/termlink && termlink fleet doctor`
  **Expected:** CT uptime > 24h, no new boots, ring20-management [PASS]
  **If not:** Other resource pressure still present — investigate memory, CPU, or disk on pve host

## Verification

# No agent-runnable verification — this is entirely host-side operator work.

## Decisions

## Updates

### 2026-04-19T08:43:09Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1137-install-logrotate-on-proxmox-host-180--p.md
- **Context:** Follow-up from G-009 (proxmox .180 /var/log full → CT 200 reboot loop → .122 cert rotations). Parked as horizon=later pending operator action on .180.

### 2026-04-22T04:52:49Z — status-update [task-update-agent]
- **Change:** horizon: later → next

### 2026-04-24T09:50:55Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
- **Change:** horizon: next → now (auto-sync)

### 2026-04-26T11:25Z — installed via console [human + agent]
- **Action:** Operator pasted one-liner on .180 console (cross-machine SSH/termlink path was blocked: SSH no key from this container, termlink auth broken via cascading TOFU+secret rotation on .122).
- **Evidence:**
  * `/etc/logrotate.d/pveproxy-access` written with daily/rotate=3/compress/copytruncate
  * `logrotate -d` dry-run: no errors (other than already-rotated note from earlier run)
  * `logrotate -f` force: rotated successfully — access.log → access.log.1.gz (1.8M)
  * `df -h /var/log`: 117M / 224M = **57%** (down from 100%)
  * `ls -la /var/log/pveproxy/`: access.log = 133 bytes (truncated), access.log.1.gz present, access.log.3.gz from Apr 25 confirms a daily cron was already running
- **AC 1 ticked.** AC 2 (<50% after 24h + daily cron active) still pending — currently 57%, will improve as old rotations age out and pre-existing 23M `.backup` rolls off. AC 3 (CT 200 stops rebooting) needs 24h observation; the active TOFU violation we just cleared on .122 suggests it has rebooted recently, so the clock starts now.

### 2026-04-24T09:53Z — cross-agent dispatch [agent]
- **Action:** Injected T-1137 prompt (2044 bytes) to ring20-management agent session `tl-schnqg3a` at 192.168.10.122:9100 via `termlink remote inject --enter`.
- **Prompt file:** /tmp/T-1137-dispatch-prompt.md (transient).
- **Scope requested:** SSH from CT to .180, write `/etc/logrotate.d/pveproxy-access`, logrotate -d + -f, verify, report back.
- **Declined scope (documented in prompt):** no pveproxy restart, no other logs, no reboot.
- **Expected reply:** short report ack or refusal via `termlink emit` with subject `T-1137-report`.
- **Authority:** T-1063 cross-repo work approval (standing user directive 2026-04-24).
- **Next step:** await reply; on success, tick Human ACs with evidence.
