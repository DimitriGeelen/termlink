---
id: T-1455
name: "Install agent-chat-arc heartbeat cron on .121 (ring20-dashboard)"
description: >
  Final field-rollout step: install hourly heartbeat cron on .121 so it actively contributes to agent-chat-arc like .107/.122/.141 do today. Blocked from agent-driven path because no termlink session is registered on .121 (no remote-exec channel). Once operator runs the install steps below + registers a session, future agents can drive .121 directly.

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: [scripts/check-vendored-arc-rollout.sh]
related_tasks: []
created: 2026-05-03T12:53:37Z
last_update: 2026-05-18T18:34:48Z
date_finished: 2026-05-18T18:34:48Z
---

# T-1455: Install agent-chat-arc heartbeat cron on .121 (ring20-dashboard)

## Context

Closes the field-rollout matrix for T-1438 / T-1166. Three of four vendored hubs already have hourly heartbeats firing into agent-chat-arc (.107 + .122 + .141 — the latter two driven from .107 today via `termlink remote exec`). .121 has no registered termlink session, so the agent has no remote-exec channel into it; this last step is operator-bound. Once a session is registered there, future agents can drive .121 the way they drive .122 and .141 today.

The heartbeat artifacts are already shipped: `.context/cron/heartbeat.crontab` (source-of-truth) and `scripts/install-heartbeat-cron.sh` (idempotent installer that copies to `/etc/cron.d/termlink-heartbeat`).

## Acceptance Criteria

### Agent
- [x] Heartbeat artifacts exist in repo (`.context/cron/heartbeat.crontab` + `scripts/install-heartbeat-cron.sh`) — already true at task-create (commit 71adf454)
- [x] Persistent termlink session registered on .121 (moved from Human AC per PL-169 — session-presence is mechanically verifiable via `termlink remote list ring20-dashboard`).
  **Evidence (2026-05-18T18:30Z):** session `tl-qe2pao72` ready (fp 33df8954b2a9b70d, project=ring20-dashboard) verified via `termlink remote list ring20-dashboard`. Constraint that originally made this Human-owned (no session → no remote-exec channel) is gone.
- [x] Heartbeat cron installed and firing on .121 (moved from Human AC per PL-169 — installation is mechanical: file send + cron-file write + smoke-test + detector verification, all command-line verifiable).
  **Evidence (2026-05-18T18:30Z):**
  - Heartbeat script delivered: `/root/termlink-heartbeat/vendored-arc-heartbeat.sh` (3114 bytes, +x) via base64-stdin through `termlink remote exec ring20-dashboard tl-qe2pao72`.
  - Cron file installed at `/etc/cron.d/termlink-heartbeat` (376 bytes, mode 644). Schedule `17 * * * *` matches .122 fleet cadence.
  - Smoke-test: `bash /root/termlink-heartbeat/vendored-arc-heartbeat.sh` → `Posted to agent-chat-arc — offset=7` + EXIT=0 (drained 22 prior queued posts).
  - Detector confirms: ring20-dashboard LAST_SEEN dropped 15d-STALE → 28s; SENDERS 1 → 2; heartbeat cron INSTALLED (was MISSING).

### Human
<!-- All ACs moved to ### Agent per PL-169 — the original constraint
     ("agent has no session on .121") no longer holds since session
     tl-qe2pao72 became reachable. Installation steps are mechanical
     (file delivery + cron config + smoke-test), all command-line verifiable. -->

## Verification

# Mechanical verification — checks that succeed once heartbeat install is live on .121.
# Detector reports SENDERS >= 2 on ring20-dashboard's chat-arc topic AND cron INSTALLED.
bash scripts/check-vendored-arc-rollout.sh 2>&1 | awk '/ring20-dashboard/ && /[0-9]+ +2 +[0-9]+ +YES/ {found=1} END {exit !found}'
bash scripts/check-vendored-arc-rollout.sh 2>&1 | grep -E "ring20-dashboard +INSTALLED"

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

### 2026-05-03T19:10Z — PL-146 forward-warning for the .121 install
- **Why this matters now:** PL-146 (commit fb95f06d, registered in `.context/project/learnings.yaml`) caught a class of cron-env failure that produces silent queueing rather than a visible error. On .141 the heartbeat appeared to fire (cron logged each invocation) but nothing landed on chat-arc — POSTS / SENDERS held steady, last_seen drifted 5h+ before anyone noticed. The script-level fix in fb95f06d auto-resolves `TERMLINK_RUNTIME_DIR` from `$HOME` when the per-uid `/tmp` socket is absent, so the same failure mode cannot recur on .121.
- **What the operator should verify after install:**
  1. `tail -5 /var/log/vendored-arc-heartbeat.log` (or the script's actual stdout target) — entries should say `Posted to agent-chat-arc — offset=N` not `Queued to agent-chat-arc — queue_id=N (hub unreachable…)`.
  2. From .107: `bash scripts/check-vendored-arc-rollout.sh` — ring20-dashboard's `LAST_SEEN` column should drop below 90 min and lose the `STALE` tag once the first :17 fires.
- **No additional steps required** — the fix is in the script the install copies. As long as you scp from `/opt/termlink/scripts/vendored-arc-heartbeat.sh` (or git-pull a recent commit), .121 inherits PL-146 protection automatically.

### 2026-05-18T18:34:34Z — status-update [task-update-agent]
- **Change:** owner: human → agent
- **Reason:** Original Human-AC constraint (no termlink session on .121) is no longer true — session tl-qe2pao72 is ready. PL-169 migration: install steps are mechanical (remote exec + cron file + smoke-test), all command-line verifiable. User directive: 'proceed as seen fit'.

### 2026-05-18T18:34:39Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
- **Reason:** Resuming after PL-169 ownership migration to complete install + verification

## Reviewer Verdict (v1.4)

- **Scan ID:** R-c7f4c2b9
- **Timestamp:** 2026-05-18T18:36:39Z
- **Catalogue:** v1.3-seed
- **Overall:** CONCERN
- **Needs Human:** no
- **Findings:** 1

**Per-AC findings:**

- **AC#1 (Agent)** — Heartbeat artifacts exist in repo (`.context/cron/heartbeat.crontab` + `scripts/install-heartbeat-cron.sh`) — already true at task-create (commit 71adf454)
  - **AC-verify-mismatch** (narrow, heuristic) — `path=scripts/install-heartbeat-cron.sh in: Heartbeat artifacts exist in repo (`.context/cron/heartbeat.crontab` + `scripts/install-heartbeat-cron.sh`) — already true at task-create (commit 71ad`

### 2026-05-18T18:34:48Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
- **Reason:** Heartbeat install + verification complete on .121; all 3 Agent ACs ticked with evidence; verification commands pass
