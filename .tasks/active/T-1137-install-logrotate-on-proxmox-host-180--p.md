---
id: T-1137
name: "Install logrotate on proxmox host .180 — prevent /var/log full cascade (G-009)"
description: >
  Proxmox host 192.168.10.180 has /var/log on a 224M zram0 filesystem at 100%; pveproxy
  access.log is 145M. Full /var/log cascades into LXC container reboot loops (CT 200
  /
  ring20-management / .122 rebooted 5× in 5h on 2026-04-19, producing 4 distinct TLS
  cert
  rotations observed from termlink clients). Structural fix: logrotate config on the
  pve
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
last_update: '2026-08-27T21:13:19Z'
date_finished:
bvp_scores_proposed:
  - ts: '2026-08-20T15:20:35Z'
    estimator: bvp-estimator-v1-heuristic
    scores:
      D1: 2
      D2: 0
      D3: 0
      D4: 4
      F-RECALL: 2
      F-ORCH: 0
    rationale: D1=2 (body:concern-ref); D2=0 (no-signal); D3=0 (no-signal); D4=4
      (body:cross-machine); F-RECALL=2 (body:lightly-promoted); F-ORCH=0 
      (no-signal)
    rubric_sha: e4a00f38e801
cost_estimate_proposed:
  - ts: '2026-08-20T15:21:20Z'
    estimator: bvp-estimator-v1-heuristic
    cost_estimate:
      blast_radius: 0
      tier: 2
      effort: 5
    rationale: blast_radius=0 (no-signal); tier=2 (no-signal); effort=5 
      (no-signal)
    rubric_sha: e4a00f38e801
  - ts: '2026-08-27T21:13:19Z'
    estimator: bvp-estimator-v1-heuristic
    cost_estimate:
      blast_radius:
      tier: 2
      effort: 6
    rationale: blast_radius=? (no-components-UNMEASURED-not-zero); tier=2 
      (workflow:build); effort=6 (lines=174,acs=3)
    rubric_sha: e4a00f38e801
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

## Recommendation

**Recommendation:** CLOSE — the cascade this task exists to prevent was removed by
a different change, and the two open ACs measure a host that no longer exists in
the shape they describe.

**Rationale:** T-1137's whole premise is that `/var/log` on .180 lives on a 224 MiB
zram0 volume, so a 145 MiB `access.log` fills it and PVE starts killing containers.
That premise is false today. `log2ram` — the mechanism that put `/var/log` in RAM —
is installed but **disabled and inactive**, there is no zram mount, and `/var/log`
is a plain directory on the 68 GB `pve-root` filesystem holding 76 MB. There is no
longer a small volume to fill, so pveproxy log growth cannot cascade. AC 2 asks for
"`/var/log` below 50 %"; the number `df` now returns for that path (56 %) is the
whole root filesystem and is not a measurement of anything this task is about. AC 3
asks whether **CT 200** stopped rebooting; CT 200 is not on .180 at all — `pct list`
shows nine containers and none is 200.

**Evidence:** Measured on .180 over SSH, 2026-08-27. `mount | grep -i zram` → no
matches. `df -h /var/log` → `/dev/mapper/pve-root 68G 36G 29G 56% /`. `du -sh
/var/log` → 76M. `systemctl is-enabled log2ram` → `disabled`; `is-active` →
`inactive`. `uptime` → up 52 days. `pct list` → VMIDs 108/109/162/170/201/310/400/
401/450; no 200. Rotation of `/var/log/pveproxy/access.log` **is** happening —
`access.log.1.gz` (606 KB, Aug 27 00:11) and `access.log.2.gz` (635 KB, Aug 26
21:17) are present — but it is stock `/etc/logrotate.d/pve` doing it (`rotate 2`,
`daily`, `maxsize 10M`, `compress`), not this task's file.

**The one finding that argues the other way.** `/etc/logrotate.d/pveproxy-access` —
the file AC 1 was ticked for on 2026-04-26 — **no longer exists on .180**. The
artifact this task delivered is gone, presumably lost in whatever host work removed
log2ram. Its function is covered by the stock `pve` config above, which is stricter
on size (`maxsize 10M`) and looser on retention (`rotate 2` vs 3), so nothing is
unrotated; but if you close this on the strength of AC 1, close it knowing the file
it certifies is not there.

**What you are actually deciding.** Not whether logrotate works — it does, via stock
config. You are deciding what to do with a task whose remaining two criteria have
been overtaken by host changes nobody recorded here.

| Option | Effect | Cost |
|---|---|---|
| CLOSE (recommended) | task closes on "premise dissolved", AC 2/AC 3 unmet as written | the record shows two unticked ACs at close; the reason is in this section, not in a tick |
| Re-scope the ACs to today's host | AC 2 becomes "/var/log stays bounded on pve-root", AC 3 drops (CT 200 gone) | rewrites acceptance criteria after the fact to fit the evidence — the shape that makes a gate stop meaning anything |
| KEEP-OPEN | waits for AC 2/AC 3 as written | they can never be satisfied: there is no 224M volume to get below 50 % of, and no CT 200 to stop rebooting |

**What closing does NOT do.** G-009 stays `status: watching`. Two of its four
`what_remains` items are untouched by anything here: a **framework cross-host
disk-pressure check** was never built, and log2ram is disabled rather than removed,
so a future re-enable puts `/var/log` back on a RAM disk and nothing in this repo
would notice. Closing the task should not be read as closing the gap.

**Not measured.** I did not verify that container reboots have stopped. CT 200 is
absent so its boot history is unreadable from here, and `~/.termlink/rotation.log`
holds only three May smoke-test rows — no watch loop has ever captured this fleet,
so it is not evidence either way. Note also that a stable TLS fingerprint no longer
proves stability: T-1294 moved the hub's runtime_dir off `/tmp`, so certs now
survive reboots by design. `.122` has held `sha256:22c19fedafd…` from 2026-07-04 to
2026-08-26, which proves the cert persists — **not** that the container stayed up.

**Why I should not decide this.** The evidence settles the mechanism but not the
bookkeeping. Whether a task closes with unmet criteria, or its criteria get
rewritten to match a changed world, is a judgement about what your task record is
for — and the change that actually fixed this (disabling log2ram) happened outside
this project with no entry here, which is itself worth your eye.

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

### 2026-04-28T08:35Z — AC 2/AC 3 verification with operator creds (re-check; both still UNCHECKED)
- **AC 2 evidence (from `ssh root@192.168.10.180`):**
  * `df -h /var/log` → **98% used** (202M / 224M zram0). Threshold is <50% — NOT met.
  * `du -sh /var/log/*` sorted: **pveproxy 106M, journal 86M**, pve 4.2M, postgresql 3.8M, others <1M
  * `/etc/cron.daily/logrotate` exists and executable; rotated files span 3 days (`access.log.2.gz` 27 00:12, `access.log.3.gz` 26 13:25) — daily cron IS active ✓
  * Today's `access.log.1` is 38M *uncompressed* (rotated 28 00:31, not yet compressed by next-day cron) + 23M legacy `access.log.1-2026042613.backup` left from initial install
  * **journald.conf:** `SystemMaxUse=140M` — too generous for a 224M zram0 volume. Journal alone can eat 62% of /var/log.
- **AC 3 evidence:** CT 200 boot history (last 4 boots):
  * 27 10:42 (up till 16:55) — 6h
  * 27 16:57 (up till 18:22) — 1.5h
  * 27 18:24 (current) — 14h09m
  * **3 reboots between Apr 27 10:42 and Apr 27 18:24** — instability persists post-T-1137-AC1 + post-T-1294
- **Diagnosis:** pveproxy logrotate (T-1137 AC1) is doing its job (3-day retention, daily cron working). The cascade is NOT yet broken because (a) journald can eat 140M unbounded, (b) the legacy 23M backup is still loitering, (c) today's rotated-but-uncompressed 38M file is in flight. AC 2 (<50%) requires either lowering `SystemMaxUse` to ~30M OR expanding /var/log, neither of which is in T-1137 scope. AC 3 requires identifying the actual reboot cause from inside CT 200.
- **Recommendation:** (1) spawn follow-up task for journald sizing + backup cleanup; (2) dispatch ring20-management agent on CT 200 (.122) to investigate CT-internal reboot cause (OOM killer? cron-triggered? watchdog flap?). T-1137 stays open until AC 2 + AC 3 mechanically satisfied. Side gap: G-009 cascade is not yet structurally closed.
