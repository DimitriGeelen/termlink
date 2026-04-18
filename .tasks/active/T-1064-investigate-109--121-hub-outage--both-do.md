---
id: T-1064
name: "Investigate .109 + .121 hub outage — both down 2026-04-15 ~15:50Z"
description: >
  ring20-management (.109) and ring20-dashboard (.121) both fail termlink fleet doctor as of 2026-04-15 ~15:50Z. Diagnostics: ping OK on both, but port 9100 connection refused on .121 and timing-out on .109. T-1027 reported both running at session-start two days ago. Operator action: SSH in, check systemd hub service status on both hosts. If restart policy not deployed there, see T-931..T-935. Registered from T-1061 housekeeping session. No code fix needed — this is operational.

status: started-work
workflow_type: build
owner: human
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-04-15T17:09:39Z
last_update: 2026-04-16T21:16:44Z
date_finished: null
---

# T-1064: Investigate .109 + .121 hub outage — both down 2026-04-15 ~15:50Z

## Context

**UPDATE 2026-04-15T17:25Z (user report):** ".109 has become .126" — ring20-management container renumbered. Verified: ping .126 OK (0.15ms), .109 no longer responds, port 9100 still refused on .126 (hub process down). .121 still timing out. **Scope revision:** .109 is not "down" — it's gone. The container migrated. Client profile updated by T-1065. Remaining work here: (1) start the hub process on .126, (2) investigate .121 (may also be renumbered — operator to confirm).

**UPDATE 2026-04-15T17:32Z (agent network probe):** Scanned 192.168.10.120-135 for port 9100 + pings. Findings:
- .121 responds to ping, :9100 refused (host alive, hub process down — matches original diagnosis)
- .131 has :9100 open but RPC times out. Almost certainly a JetDirect printer (port 9100 is IANA for HP printers), NOT a termlink hub.
- .105:9100 still accepting connections (old hub from pre-T-1061 cleanup — still running, stale secret).
- No plausible "new home" candidate for ring20-dashboard found via scan.

**Conclusion:** ring20-dashboard at .121 most likely has a down hub process, not a renumber. Operator action: check systemd `termlink-hub.service` on the .121 container.

**UPDATE 2026-04-15T18:55Z (broader ring20 outage detected):** OneDev (`onedev.docker.ring20.geelenandcompany.com`) returning HTTP 502 — server outage, not routing issue (DNS resolves, TLS handshakes, HTTP response is 502 from reverse proxy). GitHub remote is reachable. Combined picture: .109→.126 renumber + .121 hub down + OneDev 502 + (from G-007) mirror lag = ring20 infrastructure is having a bad afternoon. Probable common cause: Proxmox/PVE maintenance, container rescheduling, or network equipment issue. Operator action: check PVE host health and docker-compose stack on ring20 hypervisor.

**UPDATE 2026-04-15T19:00Z (second renumber — T-1067):** User reports ".109 now is 122" — ring20-management container migrated AGAIN (.109 → .126 → .122 in one afternoon). Verified: .126 gone, .122 alive (elevated 113ms latency = different routing path). Port 9100 still refused on .122 — hub process has not followed the container. Strong signal that container is being actively rescheduled on the hypervisor and the hub service inside is not auto-starting on the new network. Hypothesis: hub systemd unit expects a fixed IP binding (T-945 fix may not be enough; bindaddr might need 0.0.0.0). Operator action still required; add to check list: inspect hub service config for IP-hardcoded ExecStart.

## Acceptance Criteria

### Agent
- [x] Cleared stale TOFU pin for .121 (cert had changed after hub restart)
- [x] Diagnosed .121: hub is running (port 9100 open), auth mismatch (secret rotated)
- [x] Diagnosed .122: hub process not running (port 9100 refused)
- [x] Ran `termlink fleet reauth ring20-dashboard` — printed heal steps

### Human
- [ ] [REVIEW] Heal .121 (ring20-dashboard) auth — fetch new secret via SSH
  **Steps:**
  1. `ssh 192.168.10.121 -- sudo cat /var/lib/termlink/hub.secret`
  2. `echo "<hex>" > /root/.termlink/secrets/ring20-dashboard.hex && chmod 600 /root/.termlink/secrets/ring20-dashboard.hex`
  3. `cd /opt/termlink && termlink fleet doctor`
  **Expected:** ring20-dashboard shows [PASS]
  **If not:** Check if hub uses different runtime_dir (`termlink doctor` on .121)

- [ ] [REVIEW] Start hub on .122 (ring20-management)
  **Steps:**
  1. SSH to .122: `ssh 192.168.10.122`
  2. Check hub service: `systemctl status termlink-hub.service`
  3. Start if not running: `systemctl start termlink-hub.service`
  4. From local: `cd /opt/termlink && termlink fleet doctor`
  **Expected:** ring20-management shows [PASS]
  **If not:** Check if systemd unit exists, install via deploy script if missing

## Verification

# Shell commands that MUST pass before work-completed. One per line.
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

### 2026-04-15T17:09:39Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1064-investigate-109--121-hub-outage--both-do.md
- **Context:** Initial task creation

### 2026-04-16T21:15:18Z — status-update [task-update-agent]
- **Change:** status: captured → started-work

### 2026-04-18T19:35Z — observation [agent]
- **Action:** Ran `termlink fleet status` to verify current state.
- **Findings:**
  - ring20-dashboard (.121) is **UP** — auth healed; status shows `[UP] 1 sessions (56ms)`. Human AC #1 effectively satisfied (someone fetched the new secret already; the fleet is now passing on this hub).
  - ring20-management (.122) still **DOWN** — TOFU violation (fingerprint changed sha256:cbc4… → sha256:b855…) AND port 9100 refused. Hub was restarted with a new cert at some point and the process is no longer accepting connections. Human AC #2 still required.
  - local-test (127.0.0.1:9100) UP — 3 sessions.
- **Recommendation:** Once .122 is healed (Human AC #2), the human can check both ACs and close. No further agent action needed.
