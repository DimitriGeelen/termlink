---
id: T-1294
name: "Migrate ring20-management hub runtime_dir from /tmp/termlink-0 to /var/lib/termlink (T-1290 GO)"
description: >
  Operator-side migration on CT 200 (.122). Confirm hypothesis (spike 1) then apply T-935 systemd-unit migration so TERMLINK_RUNTIME_DIR=/var/lib/termlink, restart hub, all clients re-pin once. Closes the upstream cause of recurring G-011 cascades — eliminates the loop where every CT reboot wipes hub.secret and triggers TOFU+auth-mismatch storm across the fleet.

status: captured
workflow_type: build
owner: human
horizon: now
tags: [auth, infrastructure, ring20-management, G-011, runtime_dir]
components: []
related_tasks: [T-1290, T-935, T-931, T-1291, T-1137, T-1051]
created: 2026-04-26T12:04:17Z
last_update: 2026-04-26T12:04:17Z
date_finished: null
---

# T-1294: Migrate ring20-management hub runtime_dir from /tmp/termlink-0 to /var/lib/termlink (T-1290 GO)

## Context

T-1290 inception decided GO 2026-04-26T12:03:26Z. Hypothesis: ring20-management hub on CT 200 (.122) runs with default `runtime_dir=/tmp/termlink-0` (tmpfs inside the LXC container) instead of the persistent `/var/lib/termlink` configured via the T-931 systemd unit. On every CT reboot the tmpfs wipes; persist-if-present has nothing to find and regenerates secret + cert. Both peer hubs (.102 self-hub, .121 ring20-dashboard) preserve correctly — bug is .122-specific deploy config.

Fix is exactly the one-line migration documented in `docs/operations/termlink-hub-runtime-migration.md` (T-935). All-in-one task: spike 1 verification (confirm tmpfs is the cause) + the migration itself + post-fix client re-pinning.

Supersedes: nothing — but it eliminates the upstream cause of every G-011 incident on .122 going forward, dramatically lowering the value of T-1291 (declarative heal manifest).

## Acceptance Criteria

### Human
- [ ] [REVIEW] Spike 1 verification — confirm runtime_dir cause on CT 200
  **Steps:**
  1. From .180 console: `pct enter 200`
  2. `ls -la /tmp/termlink-0/ /var/lib/termlink/ 2>&1` — note which directory holds the live `hub.secret`, `hub.cert.pem`, `hub.key.pem` (the live one is the directory that the running hub PID's `/proc/<pid>/cwd` or `lsof -p <pid>` points at; alternatively the one with mtime within seconds of `systemctl status termlink-hub | grep 'Active:'` start-time)
  3. `mount | grep -E ' /tmp | /var/lib '` — note the filesystem type for each path. `tmpfs` on `/tmp` confirms hypothesis.
  4. `systemctl cat termlink-hub 2>&1 | head -40` — look for `Environment=TERMLINK_RUNTIME_DIR=...`. Absent or pointing at `/tmp/...` confirms hypothesis.
  **Expected:** live `hub.secret` lives under `/tmp/termlink-0/`, `/tmp` is `tmpfs`, systemd unit lacks `TERMLINK_RUNTIME_DIR=/var/lib/termlink`.
  **If hypothesis disconfirmed (live secret already in `/var/lib/termlink/` and rotation still happens):** stop, re-open T-1290 with the new evidence — recommendation flips and the fix below is wrong.

- [ ] [RUBBER-STAMP] Apply migration: install systemd unit override + persistent runtime_dir
  **Steps:**
  1. `mkdir -p /var/lib/termlink && chmod 700 /var/lib/termlink`
  2. `mkdir -p /etc/systemd/system/termlink-hub.service.d`
  3. Write `/etc/systemd/system/termlink-hub.service.d/override.conf`:
     ```
     [Service]
     Environment=TERMLINK_RUNTIME_DIR=/var/lib/termlink
     ```
  4. `systemctl daemon-reload`
  5. `systemctl restart termlink-hub`
  6. `systemctl status termlink-hub` — Active (running)
  7. `ls -la /var/lib/termlink/` — `hub.secret`, `hub.cert.pem`, `hub.key.pem`, `hub.sock` all present
  8. (optional cleanup) `rm -rf /tmp/termlink-0` once the new runtime_dir is confirmed live
  **Expected:** Hub running, runtime_dir at `/var/lib/termlink/`, secret + cert present and persistent.
  **If not:** Check service journal `journalctl -u termlink-hub -n 50`.

- [ ] [RUBBER-STAMP] Verify persistence across CT reboot (ground truth, NOT just hub restart)
  **Steps:**
  1. `sha256sum /var/lib/termlink/hub.secret` — note the hash
  2. From .180 host: `pct reboot 200`, wait ~30s for CT to come back
  3. `pct enter 200 && sha256sum /var/lib/termlink/hub.secret`
  4. Hash must match step 1.
  **Expected:** Same hash before and after CT reboot.
  **If not:** `/var/lib/termlink` itself is on volatile storage in CT 200 — escalate, the fix needs a different mount strategy.

- [ ] [RUBBER-STAMP] Re-pin from one client and confirm fleet doctor green
  **Steps:**
  1. From a peer with `ring20-management` profile (e.g. this container .102): `termlink tofu clear 192.168.10.122:9100`
  2. `termlink fleet reauth ring20-management --bootstrap-from ssh:192.168.10.122` (or paste secret manually if SSH unavailable)
  3. `termlink fleet doctor` — line for ring20-management shows [PASS]
  4. Wait until next CT reboot OR force one (`pct reboot 200`); after reboot, `termlink remote ping ring20-management` MUST succeed without re-clearing TOFU and without re-fetching the secret.
  **Expected:** First post-fix reboot does NOT trigger TOFU violation or auth-mismatch on any client.
  **If not:** The migration didn't take — re-run spike 1 with the hub now restarted.

## Verification

# No agent-runnable verification — host-side operator work, gated entirely on Human ACs above.

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

### 2026-04-26T12:04:17Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1294-migrate-ring20-management-hub-runtimedir.md
- **Context:** Initial task creation
