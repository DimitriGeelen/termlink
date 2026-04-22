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
last_update: 2026-04-19T08:45:25Z
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


**Agent evidence (auto-batch 2026-04-22, G-008 remediation, fleet-heal-needed, t-1064):** Fleet heal IS still required — `termlink fleet doctor` on 2026-04-22 shows:
```
- local-test (127.0.0.1:9100): PASS, version 0.9.0
- ring20-dashboard (192.168.10.121:9100): FAIL — Token validation failed: invalid signature (hub restart; needs heal — T-1064 still active)
- ring20-management (192.168.10.102:9100): FAIL — Cannot connect (hub not running or IP drift; ring20-management was renumbered to .122 in memory but hubs.toml still lists .102)
```
ring20-dashboard .121 is the exact symptom this task targets (secret rotation, T-1051 heal path applies). ring20-management at .102 in hubs.toml is stale; per memory `reference_ring20_infrastructure.md` the container renumbered to .122 on 2026-04-15 — the hubs.toml needs updating too (separate cleanup). The two REVIEW actions (heal .121, start hub on .122) remain operator-only steps.
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

### 2026-04-19T00:00Z — observation [agent]
- **Action:** Second `fleet doctor` run (T-1130 session wrap-up).
- **Findings:**
  - ring20-dashboard (.121) still **UP** (58ms, status ok) — healing is stable across days.
  - ring20-management (.122) **DOWN but rotated again** — fingerprint is now `sha256:5198d1fb…` (was `sha256:b855…` on prev observation, and `sha256:cbc4…` originally). Two rotations since this task opened means the hub process is restarting repeatedly (or the container is being rescheduled — see T-1067 renumber history). Port 9100 is accepting TLS again (no "refused" this time), so the hub process is alive now — just un-trusted.
  - local-test healthy.
- **Heal path when operator is ready:** `termlink tofu clear 192.168.10.122:9100 && termlink fleet reauth ring20-management --bootstrap-from ssh:192.168.10.122` (fetches fresh secret + re-pins cert; R2 trust-anchor is SSH which is out-of-band wrt termlink auth). Do NOT auto-clear — rotations this frequent may indicate something operator needs to diagnose (container reschedule loop, hub crash loop, or attacker).
- **Severity signal:** Two cert rotations on one hub in 24h is the trigger condition for auto-registering a G-NNN (per T-1053 rule). The auth-rotation protocol says "≥3 consecutive auth-mismatches spanning >24h" — we're at 2 in ~24h, so not yet at threshold. Monitoring.

### 2026-04-19T08:40Z — root cause [agent + .122-side agent]
- **Action:** Attempted heal via `fleet reauth --bootstrap-from ssh:192.168.10.122`. SSH to .122 failed (no key, no askpass). Cross-host peer on .122 independently diagnosed why the container keeps rotating.
- **Findings (from .122-side agent, cross-verified):**
  - CT 200 (ring20-management / .122) rebooted **5× in 5h** on 2026-04-19 (`journalctl --list-boots`).
  - Proxmox host **.180 has /var/log at 100 %** on a 224 MiB zram0 filesystem.
  - `/var/log/pveproxy/access.log` alone is **145 MiB** — single file dominates the volume.
  - This matches framework PL-054 / T-269 (documented class of pve-host /var/log cascade failure).
  - 4th cert fingerprint observed on .122 this afternoon: `sha256:b90adf2598f5b4a06f73ed69bc5554b3d31d1f9b1578704c1057ddbd51e482af` (chain: cbc4 → b855 → 5198d1fb → b90adf2598). Trips T-1053 threshold (≥3 rotations in >24 h).
- **Root cause chain:** proxmox /var/log full → PVE host degrades → LXCs killed/restarted → hub regenerates TLS cert on each fresh boot → clients hit TOFU violation → clients fail to connect.
- **Registered:** G-009 in `.context/project/concerns.yaml` (severity medium, status watching). Follow-up task **T-1137** created for the structural fix (logrotate on .180), owner=human, horizon=later.
- **Immediate mitigation (operator, one-liner):**
  ```
  ssh root@192.168.10.180 'truncate -s 0 /var/log/pveproxy/access.log && df -h /var/log'
  ```
- **Why termlink-side healing keeps losing:** cert persistence (T-1028) and TOFU re-pin (T-1064/T-1055) are the right fixes in the common case, but they assume the container *stays up long enough for a client to trust the new cert*. When the container reboots every ~60 min, no amount of client-side healing catches up. The structural fix lives on the host, not in termlink.
- **Status transition candidate:** once G-009's mitigation lands (operator truncates .180's access.log) **and** T-1137's structural fix (logrotate) is installed, this task's remaining work (heal .122) becomes trivial — wait for one stable hour, then re-run `termlink tofu clear 192.168.10.122:9100` and re-bootstrap. Not setting work-completed yet; heal hasn't occurred.
