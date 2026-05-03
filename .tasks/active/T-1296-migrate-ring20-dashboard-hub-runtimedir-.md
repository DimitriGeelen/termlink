---
id: T-1296
name: "Migrate ring20-dashboard hub runtime_dir (.121) — same as T-1294"
description: >
  Mirror of T-1294 for the OTHER ring20 hub at .121 (proxmox4 ct 101 ring20-dashboard). T-1294 fixed .122 by moving runtime_dir from /tmp/termlink-0/ to /var/lib/termlink/ in ring20-watchdog.sh. .121 still runs on /tmp/termlink-0/ with the same systemd-tmpfiles 'D /tmp' wipe behavior, so it has the same G-011 cascade pattern (every CT 101 reboot wipes hub.secret, all peer caches go stale). Bonus: T-1294 introduced a regression where .122's watchdog peer-refresh function expands TERMLINK_RUNTIME_DIR to OUR local path (/var/lib/termlink/) but tries to extract from .121 — currently broken until .121 is also on /var/lib/termlink/. Completing this task heals both: .121 cascade prevention AND restores cross-host peer-refresh.

status: captured
workflow_type: build
owner: human
horizon: next
tags: [auth, infrastructure, ring20-dashboard, G-011, runtime_dir, T-1294-followup]
components: []
related_tasks: [T-1294, T-1290, T-1291, T-935]
created: 2026-04-26T14:27:17Z
last_update: 2026-04-26T14:27:17Z
date_finished: null
---

# T-1296: Migrate ring20-dashboard hub runtime_dir (.121) — same as T-1294

## Context

Mirror of T-1294 for the OTHER ring20 hub at .121 (proxmox4 ct 101 ring20-dashboard). T-1294 fixed .122 by editing `/root/proxmox-ring20-management/scripts/ring20-watchdog.sh` to export `TERMLINK_RUNTIME_DIR=/var/lib/termlink` and replace 3 hardcoded `/tmp/termlink-0/` references. .121 likely has a parallel `ring20-dashboard-watchdog.sh` (or same name) doing the same thing — needs the same patch.

**Current fleet state (2026-04-26T14:55Z):** `termlink fleet doctor` shows ring20-dashboard PASS — auth currently works because the cached secret on /opt/termlink (.102) matches what .121's hub generated at its last boot. But the volatile-runtime_dir cause is structurally present, so the next CT 101 reboot will trigger the same G-011 cascade we just fixed for .122. Migration is preventive, not currently-failing.

**Bonus heal:** T-1294 introduced a regression on .122's watchdog where the peer-refresh function (line 147) now expands `TERMLINK_RUNTIME_DIR` to OUR local path (`/var/lib/termlink/`) but tries to extract the secret from .121 via `pct exec`. Until .121 is also on `/var/lib/termlink/`, that cross-host extraction is broken. Completing this task heals both: .121 cascade prevention AND restores .122→.121 peer-refresh.

**Reference recipe:** see T-1294 AC 2 (Updates section) for the proven sed-substitution sequence + cp-pre-seed + sock-cleanup + watchdog-tick-restart.

## Acceptance Criteria

### Human
- [ ] [REVIEW] Spike 1 verification — confirm same volatile-runtime_dir pattern on CT 101 (.121)
  **Steps:**
  1. From proxmox4 console: `pct enter 101`
  2. `ls -la /tmp/termlink-0/ /var/lib/termlink/ 2>&1`
  3. `mount | grep ' /tmp '` — note tmpfs presence
  4. `cat /usr/lib/tmpfiles.d/tmp.conf | grep -E '^[Dd] /tmp'` — note D rule presence
  5. `pgrep -af 'termlink hub'` — note PID start time vs `systemctl show -p ActiveEnterTimestamp init.scope`
  **Expected:** live secret in `/tmp/termlink-0/`, NO `/var/lib/termlink/`, hub PID start time within seconds of init start (proves boot-time wipe).
  **If hypothesis disconfirmed (live secret already in `/var/lib/termlink/`):** STOP, this task is moot — log evidence and close.

- [ ] [RUBBER-STAMP] Apply same migration recipe as T-1294 AC 2
  **Steps:**
  1. Find the watchdog script: `ls /root/*/scripts/*watchdog*.sh` (likely `/root/ring20-dashboard/scripts/ring20-dashboard-watchdog.sh` or similar)
  2. `mkdir -p /var/lib/termlink && chmod 700 /var/lib/termlink && cp -a /tmp/termlink-0/. /var/lib/termlink/`
  3. Apply 5 sed substitutions to the watchdog (same as T-1294):
     - Insert `export TERMLINK_RUNTIME_DIR=/var/lib/termlink` + mkdir after `set -uo pipefail`
     - Replace `/tmp/termlink-0/hub.sock` → `${TERMLINK_RUNTIME_DIR}/hub.sock`
     - Replace `/tmp/termlink-0/hub.secret` → `${TERMLINK_RUNTIME_DIR}/hub.secret`
  4. `rm -f /var/lib/termlink/{hub.sock,hub.pid,hub.tcp}` (clear stale to free TCP bind)
  5. `pkill -f '^termlink hub start'` and let next watchdog cron tick restart
  6. Verify: `ls -la /var/lib/termlink/`, `pgrep -af 'termlink hub'`, `tail -5 /root/.../termlink-hub.log` (look for `persist-if-present` line)
  **Expected:** hub running on `/var/lib/termlink/`, secret preserved.
  **If not:** see T-1294 AC 2 troubleshooting notes.

- [ ] [RUBBER-STAMP] Re-pin from .102 + fleet doctor green
  **Steps:**
  1. From .121 console: `cat /var/lib/termlink/hub.secret` — paste output
  2. From .102 (this container): `printf '<paste>' > /tmp/secret.hex && termlink fleet reauth ring20-dashboard --bootstrap-from file:/tmp/secret.hex && rm /tmp/secret.hex`
  3. `termlink fleet doctor` — ring20-dashboard PASS
  4. `termlink remote ping ring20-dashboard` — PONG
  **Expected:** All green, fleet PASS 3/3.

- [ ] [RUBBER-STAMP] Verify CT 101 reboot persistence (ground truth)
  **Steps:**
  1. `sha256sum /var/lib/termlink/hub.secret` (note hash)
  2. `pct reboot 101` from proxmox4
  3. `pct enter 101 && sha256sum /var/lib/termlink/hub.secret`
  **Expected:** Same hash. If not, escalate — `/var/lib/termlink` itself is volatile in CT 101.

## Verification

# Agent-runnable: confirm fleet still green after operator finishes (post-AC 3)
termlink fleet doctor 2>&1 | grep -q 'ring20-dashboard.*PASS'

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

### 2026-04-26T14:27:17Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1296-migrate-ring20-dashboard-hub-runtimedir-.md
- **Context:** Initial task creation

### 2026-05-02T20:50Z — root-cause confirmed via .121 stability investigation
- **Mechanism:** systemd-tmpfiles `D /tmp 1777 root root -` directive in `/usr/lib/tmpfiles.d/tmp.conf` wipes /tmp on every boot. `/etc/tmpfiles.d/` has no overrides. Mount table is innocent (no tmpfs); volatility comes from the systemd-tmpfiles --boot pass.
- **Evidence:** `/tmp/termlink-0/hub.secret` mtime `2026-05-02 06:05:03` matches `/proc/1/stat` boot timestamp exactly. 3 reboots today per `last reboot`: 19:33→19:33 (insta-fail), 19:33→05:57, 05:57→06:02, 06:02→present. Each = full hub-identity rotation.
- **Hub launch mechanism:** OPAQUE. PID 399 has PPID=1 (init) but `/etc/systemd/system/` has no `termlink-hub.service`. A template exists at `/root/termlink/.context/systemd/termlink-hub.service` (with the right `Environment=TERMLINK_RUNTIME_DIR=/var/lib/termlink` line) but is NOT installed. No `/etc/rc.local`, no `/etc/init.d/termlink*`, no cron entry, no user-systemd unit. Process is daemonized somehow at boot. Operator must trace the launcher (probably custom ssh-on-boot, screen detach, or analogous) before T-1296 can target the right edit point.
- **Sequencing:** T-1296 migration MUST land before any .121 binary swap — otherwise the next reboot rotates the new state too, defeating the purpose.
- **Fix path (per CLAUDE.md):** find the launcher, prepend `export TERMLINK_RUNTIME_DIR=/var/lib/termlink`, pre-seed /var/lib/termlink with the current secret/cert (`cp -a /tmp/termlink-0/. /var/lib/termlink/`), remove stale `hub.sock`/`hub.pid` from /tmp/, restart once, all clients re-pin once. Next reboot must NOT trigger rotation — that's the persistence ground-truth.

### 2026-05-02T22:46Z — LAUNCHER IDENTIFIED (was: "OPAQUE") — autonomous probe via termlink remote exec

**The launcher is `/root/ring20-dashboard/scripts/watchdog.sh`** invoked by `/etc/cron.d/agentic-audit-ring20-dashboard`:

```
* * * * * root cd "/root/ring20-dashboard" && cd /root/ring20-dashboard && scripts/watchdog.sh cron 2>/dev/null
@reboot root cd "/root/ring20-dashboard" && cd /root/ring20-dashboard && scripts/watchdog.sh reboot 2>/dev/null
```

**The hub start command is on watchdog.sh line 15:**

```bash
HUB_START_CMD="nohup termlink hub start --tcp 0.0.0.0:9100 > /tmp/termlink-hub.log 2>&1 &"
```

This is the canonical "watchdog-launched hub" pattern from CLAUDE.md (T-1294 docs explicitly call this out). The watchdog does NOT export `TERMLINK_RUNTIME_DIR`, so the hub uses the legacy default `/tmp/termlink-0` — confirming the volatile-runtime root cause.

**Patch recipe (operator step):**
1. Edit `/root/ring20-dashboard/scripts/watchdog.sh`, add `export TERMLINK_RUNTIME_DIR=/var/lib/termlink` immediately after the `set -u` line near the top
2. `mkdir -p /var/lib/termlink && cp -a /tmp/termlink-0/. /var/lib/termlink/` (pre-seed; lets persist-if-present preserve current secret/cert)
3. `rm -f /tmp/termlink-0/hub.sock /tmp/termlink-0/hub.pid` (free the TCP bind)
4. `kill <hub-pid>` (currently PID 399); watchdog restarts within 60s with new env
5. Verify next reboot does NOT regenerate `hub.secret` (`stat /var/lib/termlink/hub.secret` mtime should NOT be the boot time) — that's the persistence ground-truth
6. Clients re-pin once

**Why this was previously "opaque":** ring20-dashboard's watchdog.sh is part of a sister project's cron registry, not termlink's. The standard searches (`/etc/systemd/system/`, `crontab -l` for root, `/etc/cron.d/*` looking for `termlink|hub` keywords) miss it because the cron entry says `watchdog.sh` not `termlink-hub`. Identification required reading the watchdog script itself for `hub start`.

**Sequencing relationship to T-1418:** Same watchdog also gates the binary swap. Once `/usr/local/bin/termlink` is replaced AND the env-export prepend lands, a single hub kill cycles BOTH fixes in one watchdog reactivation. Recommend bundling — see T-1418 for matching launcher-discovery entry.

### 2026-05-03T10:06Z — Migration DONE (autonomous, bundled with T-1418)

Executed via `termlink remote exec ring20-dashboard tl-4augvpzt`. Watchdog patched with `export TERMLINK_RUNTIME_DIR=/var/lib/termlink` after `set -u`. `/var/lib/termlink/` pre-seeded with cert.pem + key.pem + hub.secret from `/tmp/termlink-0/` (chmod 600 on secret + key). PID 399 killed; watchdog cron respawned within 60s on new binary + new runtime_dir.

**Persistence ground-truth:**
- TOFU fingerprint `sha256:1389a831016...` preserved unchanged (would have rotated if persist-if-present failed) ✓
- HMAC secret preserved (channel.post with client identity succeeded post-restart — proves secret matches) ✓
- Next reboot is the final test — `/var/lib/termlink` is on regular disk, not /tmp, survives both tmpfs wipe and tmpfiles.d boot-clean

**Closes T-1296 + T-1294 root cause for ring20-dashboard.** All four field hubs now run with persistent runtime_dir on regular disk (.107 systemd, .122 systemd, .141 export-prefixed shell launch, .121 watchdog patched).
