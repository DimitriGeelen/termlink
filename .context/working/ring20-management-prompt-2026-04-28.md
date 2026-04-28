# Dispatch prompt — ring20-management (CT 200 / .122) — 2026-04-28

**Authority:** standing user directive (T-1063 cross-repo work approval, re-issued 2026-04-28).
**Sender:** /opt/termlink agent (.102) acting on T-1137 + T-1294.

## Background you need

You are the agent running INSIDE Proxmox container CT 200 (hostname `ring20-manager`, IP 192.168.10.122). You have full root inside this CT.

Two outer-context tasks just landed verifications from the .180 console (Proxmox host):

1. **T-1294 AC 3 SATISFIED** — `/var/lib/termlink/hub.secret` (sha256 `3dd9d01a…fa9a71`, mtime `Apr 25 20:34`) survived ≥3 CT reboots since the runtime_dir migration. Migration to `/var/lib/termlink` works. Clients no longer re-pin per reboot. Do NOT redo this — just be aware.

2. **T-1137 AC 2/AC 3 still UNCHECKED.** From the .180 host:
   - `/var/log` is at **98%** (202M / 224M on zram0). pveproxy logrotate works (3-day retention) but `/var/log/journal` alone eats **86M** with `journald.conf SystemMaxUse=140M`. AC 2 wants <50%.
   - **CT 200 rebooted 3× yesterday** (Apr 27 10:42 → 16:55, 16:57 → 18:22, 18:24 → present). The host journal shows no OOM, no kernel kills, no `pve-container@200` shutdown reason. We cannot see the cause from outside the CT.

## Your scope (one investigation, no remediation)

Investigate **why CT 200 rebooted 3× yesterday** from inside the container. **Do not fix anything yet** — gather the evidence that lets us classify the cause and decide the right structural fix.

Specifically, collect (and report concisely):

1. **CT-internal boot history.** `journalctl --list-boots -n 10` from inside the CT. Note times and durations. Match the 3 boots from yesterday.

2. **Last 30 lines of journal BEFORE each crash.** For each boot in the list, run:
   ```
   journalctl -b <boot-id> --no-pager | tail -50
   ```
   Look for: panic, kernel oops, OOM-killer, segfault, watchdog timeout, fs errors, network flaps. Quote the last meaningful line(s) per boot.

3. **OOM history inside CT.** `dmesg --since '3 days ago' | grep -iE 'oom|killed process|cgroup'` and `journalctl --since '3 days ago' | grep -iE 'oom|killed|out of memory'`.

4. **Memory pressure.** `free -h`, `cat /proc/pressure/memory`, `cat /proc/pressure/cpu`, `cat /proc/loadavg`. CT was configured with 8 GB RAM (per `pct config 200`).

5. **Container watchdog / cron self-reboot.** Audit:
   - `crontab -l -u root` and `ls -la /etc/cron.*/` — any reboot-triggering cron?
   - `/root/proxmox-ring20-management/scripts/ring20-watchdog.sh` — does it ever issue a `reboot` or `shutdown -r`? Grep it.
   - `systemctl list-timers --all` — anything restart-y?
   - Any external orchestrator that might be restarting the CT?

6. **Process longevity.** `ps -eo pid,etime,cmd --sort=-etime | head -20` — what's been running since boot vs longer? Anything OOM-restart-spam?

7. **Disk inside CT.** `df -h` — is anything full inside the CT itself? (Different volume than .180's /var/log.)

## Output format (mandatory)

Write a report **to `/root/T-1137-ct-reboot-investigation-2026-04-28.md`** using this skeleton:

```
# T-1137 / T-1294 follow-up — CT 200 reboot cause investigation
Date: 2026-04-28
Investigator: ring20-management agent

## Summary (3 lines)
- Most likely cause: ...
- Evidence strength: high / medium / low
- Recommended next action: ...

## Boot history
[journalctl --list-boots output]

## Last lines before each crash (Apr 27)
### Boot ending 16:55 UTC
[quote]
### Boot ending 18:22 UTC
[quote]

## OOM / kernel events
[evidence]

## Memory + load
[evidence]

## Cron / watchdog audit
[evidence]

## Disk inside CT
[df -h output]

## Conclusion
[one paragraph: cause + confidence + recommended fix scope]
```

Then, when the report is written, send a one-line `termlink emit` to topic `t-1137-ct-investigation` containing exactly:
```
report-ready: /root/T-1137-ct-reboot-investigation-2026-04-28.md
```

## Boundaries (HARD)

- **Do NOT reboot the CT.** Not even to test. We are diagnosing, not patching.
- **Do NOT change `journald.conf`, `crontab`, or `ring20-watchdog.sh`.** Read-only audit.
- **Do NOT touch `/var/lib/termlink/`** or anything related to the hub auth path. T-1294 is closed; do not regress it.
- **Do NOT shutdown or restart any service.** No `systemctl restart`, no `kill`.
- If you find an active in-progress problem (e.g. OOM-killer is running RIGHT NOW), STOP and emit a warning to topic `t-1137-ct-investigation` instead of taking action.

## Time budget
~10 minutes. If something blocks the investigation, emit what you have and request guidance.

## Why this matters
T-1137 (logrotate on .180) addressed one cascade vector (/var/log full → host degrades → CTs reboot). The fact that CT 200 still reboots 3×/day means there's a second cause — possibly inside the CT itself. Without identifying it, every reboot still produces the operational pain the entire G-011 lineage was supposed to eliminate.
