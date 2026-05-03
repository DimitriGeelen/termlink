---
id: T-1455
name: "Install agent-chat-arc heartbeat cron on .121 (ring20-dashboard)"
description: >
  Final field-rollout step: install hourly heartbeat cron on .121 so it actively contributes to agent-chat-arc like .107/.122/.141 do today. Blocked from agent-driven path because no termlink session is registered on .121 (no remote-exec channel). Once operator runs the install steps below + registers a session, future agents can drive .121 directly.

status: captured
workflow_type: build
owner: human
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-05-03T12:53:37Z
last_update: 2026-05-03T12:53:37Z
date_finished: null
---

# T-1455: Install agent-chat-arc heartbeat cron on .121 (ring20-dashboard)

## Context

Closes the field-rollout matrix for T-1438 / T-1166. Three of four vendored hubs already have hourly heartbeats firing into agent-chat-arc (.107 + .122 + .141 — the latter two driven from .107 today via `termlink remote exec`). .121 has no registered termlink session, so the agent has no remote-exec channel into it; this last step is operator-bound. Once a session is registered there, future agents can drive .121 the way they drive .122 and .141 today.

The heartbeat artifacts are already shipped: `.context/cron/heartbeat.crontab` (source-of-truth) and `scripts/install-heartbeat-cron.sh` (idempotent installer that copies to `/etc/cron.d/termlink-heartbeat`).

## Acceptance Criteria

### Agent
- [ ] Heartbeat artifacts exist in repo (`.context/cron/heartbeat.crontab` + `scripts/install-heartbeat-cron.sh`) — already true at task-create (commit 71adf454)

### Human
- [ ] [REVIEW] Heartbeat cron installed + firing on .121
  **Steps:**
  1. Open SSH session to ring20-dashboard (192.168.10.121) — operator-side, since the agent has no termlink session there
  2. Ensure the project tree exists at `/opt/termlink` (or a clone of it). If not, the simplest path is:
     ```
     scp -r workstation-107:/opt/termlink/scripts/vendored-arc-heartbeat.sh \
            workstation-107:/opt/termlink/scripts/install-heartbeat-cron.sh \
            workstation-107:/opt/termlink/.context/cron/heartbeat.crontab \
            /tmp/heartbeat-bundle/
     ```
     Adjust paths to match the layout on .121.
  3. Run the installer (it shells out to sudo internally):
     ```
     sudo bash /tmp/heartbeat-bundle/install-heartbeat-cron.sh
     ```
     If the project tree IS present at `/opt/termlink`:
     ```
     sudo /opt/termlink/scripts/install-heartbeat-cron.sh
     ```
  4. Wait for the next :17 past the hour (or `bash /opt/termlink/scripts/vendored-arc-heartbeat.sh` to smoke-test now)
  5. Verify the post landed:
     ```
     termlink channel info agent-chat-arc | head -10
     # Should show Senders count >= 2 (was 1: just d1993c2c from .107) and a new fingerprint = the .121 session's identity
     ```
  6. Tail the log:
     ```
     tail -5 /var/log/vendored-arc-heartbeat.log
     # Expected: lines like "Posted to agent-chat-arc — offset=N, ts=..."
     ```
  **Expected:** new sender appears on .121 chat-arc; log shows successful posts; cron fires hourly.
  **If not:** check `journalctl -u cron` for cron errors; verify `/etc/cron.d/termlink-heartbeat` was created with mode 644; verify the heartbeat script's binary discovery picks up a working termlink (it probes /usr/local/bin, /opt/termlink/target/release/, /root/termlink/target/release/, and /mnt/c/...). On WSL hosts beware of PL-145 (the /mnt/c segfault).

- [ ] [RUBBER-STAMP] Register a persistent termlink session on .121 once
  **Steps:**
  1. From the SSH session on .121:
     ```
     termlink register --name ring20-dashboard --tags "host=121,project=ring20-dashboard"
     ```
     (Or invoke `termlink spawn` — match the pattern used on .122 / .141.)
  2. Verify from .107:
     ```
     termlink remote list ring20-dashboard
     # Should show one ready session
     ```
  **Expected:** future agents can drive .121 via `termlink remote exec ring20-dashboard <session> '<cmd>'`, eliminating the need for SSH for next iteration.
  **If not:** check that the hub at .121:9100 is reachable; check secret_file path matches.

## Verification

# Operator-side verification — checks that succeed once the heartbeat is firing.
# Each command must pass. Empty / comment lines ignored.
target/release/termlink channel info --hub ring20-dashboard agent-chat-arc 2>&1 | grep -qE "Senders: [2-9]"

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

### 2026-05-03T12:53:37Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1455-install-agent-chat-arc-heartbeat-cron-on.md
- **Context:** Initial task creation
