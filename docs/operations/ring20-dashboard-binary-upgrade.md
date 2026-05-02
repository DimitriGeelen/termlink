# ring20-dashboard (.121) — Binary upgrade + runtime_dir migration runbook

**Tasks:** T-1418 (binary upgrade to T-1235+), T-1296 (runtime_dir migration off /tmp).
**Status as of 2026-05-02 08:18Z:** auth healed (commit `36869732`), but hub crashed during chat-arc test post (PL-122). Hub is currently DOWN on .121.
**Current state:** binary 0.9.844, runtime_dir `/tmp/termlink-0/` (volatile), TLS fp `1389a831…`, secret prefix `08e61c486d28…`.
**Goal:** restart with T-1235+ binary AND durable `/var/lib/termlink` runtime_dir, in a single restart cycle (don't restart twice).

> **Why both at once:** the hub is already down, so the cost of a restart is paid. A second restart later (just for T-1296) would mean another G-049 fire window. Bundle into one operator action.

---

## Pre-flight (run on .107)

```
# Confirm .121 hub is still down
nc -zv -w 3 192.168.10.121 9100
# expect: Connection refused

# Confirm container responds at the IP layer
ping -c 2 192.168.10.121

# Confirm cached secret on .107 (post-heal)
ls -la ~/.termlink/secrets/ring20-dashboard.hex
# expect: 64 hex chars + newline, chmod 600
sha256sum ~/.termlink/secrets/ring20-dashboard.hex
# capture this; will compare post-restart

# Confirm pinned TLS fp on .107
grep -A2 192.168.10.121 ~/.termlink/known_hubs.json 2>/dev/null || \
  grep -rA2 1389a831 ~/.termlink/ 2>/dev/null | head -10
# expect: sha256:1389a831016c4bf150587879af620b227df9bdb27dfabc66d2673827cecd7c5b
```

If `nc` reports the port is OPEN, the hub came back up on its own (watchdog respawn). Skip Phase 3's "start hub" and validate state at Phase 4.

---

## Phase 1 — Runtime_dir migration on .121 (T-1296)

**Operator on .121 console** (pct exec 101 -- bash, or SSH if reachable):

```
# 1. Confirm current state
ls -la /tmp/termlink-0/
# expect: hub.secret, hub.cert.pem, hub.key.pem (mtime ~heal time)
sha256sum /tmp/termlink-0/hub.secret /tmp/termlink-0/hub.cert.pem
# capture both — they must survive the migration intact

# 2. Make destination
mkdir -p /var/lib/termlink
chmod 700 /var/lib/termlink

# 3. Pre-seed the new runtime_dir with the freshly-minted secret + cert
#    (this is what makes persist-if-present preserve them post-restart)
cp -a /tmp/termlink-0/hub.secret     /var/lib/termlink/hub.secret
cp -a /tmp/termlink-0/hub.cert.pem   /var/lib/termlink/hub.cert.pem
cp -a /tmp/termlink-0/hub.key.pem    /var/lib/termlink/hub.key.pem

# 4. Verify SHAs match post-copy
sha256sum /var/lib/termlink/hub.secret /tmp/termlink-0/hub.secret
sha256sum /var/lib/termlink/hub.cert.pem /tmp/termlink-0/hub.cert.pem
# expect: identical pairs

# 5. Make sure stale lock files don't block bind on the new path
rm -f /var/lib/termlink/hub.sock /var/lib/termlink/hub.pid
```

**Update the launcher** so the next start uses the new runtime_dir.

Find which mechanism launches the hub:

```
# Check systemd
systemctl status termlink-hub 2>&1 | head -3
ls /etc/systemd/system/termlink-hub.service /lib/systemd/system/termlink-hub.service 2>/dev/null

# Check watchdog scripts
ls /root/ring20-watchdog.sh /usr/local/bin/ring20-watchdog.sh /root/termlink/scripts/*watchdog* 2>/dev/null

# Check the original launch command from history (if hub was running interactively)
ps -eo pid,etime,cmd | grep termlink | grep -v grep
```

### If launched by systemd
Edit the unit file (most likely `/etc/systemd/system/termlink-hub.service`):

```
[Service]
Environment=TERMLINK_RUNTIME_DIR=/var/lib/termlink
ExecStart=/usr/local/bin/termlink hub start --tcp 0.0.0.0:9100
```

Then:
```
systemctl daemon-reload
# DO NOT systemctl start yet — Phase 2 swaps the binary first.
```

### If launched by a watchdog script (e.g. ring20-watchdog.sh)
Edit the script: add near the top, before any `termlink hub start` invocation:
```
export TERMLINK_RUNTIME_DIR="/var/lib/termlink"
```
Replace any hardcoded `/tmp/termlink-0/...` references (esp. `hub.sock`, `hub.secret`) with the new path.

If the watchdog is currently respawning the (broken) hub in a loop, stop the loop:
```
systemctl stop ring20-watchdog 2>/dev/null \
  || pkill -f ring20-watchdog.sh \
  || echo "find and kill the watchdog manually"
```

### If launched manually
Just remember to run with `TERMLINK_RUNTIME_DIR=/var/lib/termlink` set in the env at start time.

---

## Phase 2 — Stage T-1235+ binary

**Operator on .107:**

```
cd /opt/termlink

# Pick a binary that contains T-1235 (channel.list dual-read shim).
# Easiest: latest release build that's already passed musl probes for the fleet.
ls -la /tmp/termlink-staged-* 2>/dev/null
# Use 0.9.1702 if available (covers T-1235, T-1427, T-1429-Phase2, T-1441, T-1443)

# Or build fresh:
# cargo build --release --target x86_64-unknown-linux-musl --bin termlink

# Probe-ship to .121 (push, do not swap yet)
.agentic-framework/bin/fw fleet deploy-binary --hub ring20-dashboard \
  --binary /tmp/termlink-staged-0.9.1702 --probe
# expect: 453 chunks delivered, sha verified, stage path /tmp/termlink-staged-0.9.1702 on .121
```

If fleet-deploy-binary fails because the hub is down (it will), use the manual fallback — copy via SSH/PVE/SCP:

```
# From .107, push to /121 via PVE (operator has PVE creds):
scp /tmp/termlink-staged-0.9.1702 root@<pve-host>:/tmp/
ssh root@<pve-host> "pct push 101 /tmp/termlink-staged-0.9.1702 /tmp/termlink-staged-0.9.1702"

# Verify the SHA on both ends:
sha256sum /tmp/termlink-staged-0.9.1702
ssh root@<pve-host> "pct exec 101 -- sha256sum /tmp/termlink-staged-0.9.1702"
# expect: identical hashes
```

---

## Phase 3 — Binary swap + first start under new runtime_dir

**Operator on .121:**

```
# 1. If hub is somehow up again (watchdog respawned 0.9.844), stop it
systemctl stop termlink-hub 2>/dev/null
pkill -f 'termlink hub start' || true
sleep 2
nc -z 127.0.0.1 9100 && echo "FAILED: hub still up" || echo "hub down — ok to swap"

# 2. Swap the binary (rm-then-cp avoids 'text file busy' if a stray ref exists)
BACKUP=/usr/local/bin/termlink.0.9.844.bak
cp /usr/local/bin/termlink "$BACKUP"
rm /usr/local/bin/termlink
cp /tmp/termlink-staged-0.9.1702 /usr/local/bin/termlink
chmod 755 /usr/local/bin/termlink
/usr/local/bin/termlink --version
# expect: termlink 0.9.1702 (or whatever you staged)

# 3. Start the hub under the NEW runtime_dir
# Systemd path:
systemctl start termlink-hub
# Watchdog path:
systemctl start ring20-watchdog 2>/dev/null || /path/to/ring20-watchdog.sh &
# Manual path:
TERMLINK_RUNTIME_DIR=/var/lib/termlink \
  nohup /usr/local/bin/termlink hub start --tcp 0.0.0.0:9100 \
  > /var/log/termlink-hub.log 2>&1 &

# 4. Verify hub came up under new runtime_dir
sleep 3
ss -tlnp | grep :9100
# expect: termlink listening
ls -la /var/lib/termlink/hub.sock /var/lib/termlink/hub.secret /var/lib/termlink/hub.cert.pem
# expect: hub.sock now exists (newly created); secret + cert PRESERVED from Phase 1 pre-seed

# 5. CRITICAL — confirm persist-if-present held
sha256sum /var/lib/termlink/hub.secret /var/lib/termlink/hub.cert.pem
# Must equal the SHAs you captured at the END of Phase 1 step 4.
# If they DIFFER, the binary regenerated → upgrade is broken; rollback by:
#   systemctl stop ...; cp $BACKUP /usr/local/bin/termlink; restart with /tmp/termlink-0
```

---

## Phase 4 — Verification (run on .107)

```
cd /opt/termlink
termlink fleet doctor 2>&1 | grep -A3 ring20-dashboard
# expect: PASS, version 0.9.1702 (NOT "unknown" anymore — modern hub returns version)

# Confirm cached secret + pinned TLS fp on .107 still match
sha256sum ~/.termlink/secrets/ring20-dashboard.hex
# (compare to pre-flight capture — should be unchanged)

# Compare TLS fingerprint
termlink fleet doctor --json 2>&1 | python3 -c "
import json, sys
d = json.load(sys.stdin)
for h in d['hubs']:
  if h['name'] == 'ring20-dashboard':
    print(h)
" 2>/dev/null | head -5
# expect: tofu_pin_match=true, fingerprint=sha256:1389a831016c4bf150587879af620b227df9bdb27dfabc66d2673827cecd7c5b

# Modern envelope smoke — should now succeed (was the trigger that crashed 0.9.844)
termlink channel post --hub ring20-dashboard \
  --msg-type milestone --metadata '_thread=T-1418' \
  --payload '[T-1418] hub upgraded to 0.9.1702 + runtime_dir migrated to /var/lib/termlink — chat-arc capable' \
  agent-chat-arc
# expect: Posted to agent-chat-arc — offset=0, ts=...
```

---

## Phase 5 — Chat-arc skill propagation (run on .107, after Phase 4 PASS)

```
cd /opt/termlink

# Get a session id on .121
termlink remote list ring20-dashboard

# Push agent-handoff.md + check-arc.md to BOTH user-home AND project-local
B64_HANDOFF=$(base64 -w0 /opt/termlink/.claude/commands/agent-handoff.md)
B64_CHECK=$(base64 -w0 /opt/termlink/.claude/commands/check-arc.md)
SESSION=$(termlink remote list ring20-dashboard | awk 'NR>2 && $3=="ready" {print $1; exit}')

termlink remote exec ring20-dashboard "$SESSION" "
mkdir -p /root/.claude/commands /root/ring20-dashboard/.claude/commands
echo '$B64_HANDOFF' | base64 -d > /root/.claude/commands/agent-handoff.md
echo '$B64_HANDOFF' | base64 -d > /root/ring20-dashboard/.claude/commands/agent-handoff.md
echo '$B64_CHECK'   | base64 -d > /root/.claude/commands/check-arc.md
echo '$B64_CHECK'   | base64 -d > /root/ring20-dashboard/.claude/commands/check-arc.md
md5sum /root/.claude/commands/agent-handoff.md /root/ring20-dashboard/.claude/commands/agent-handoff.md \
       /root/.claude/commands/check-arc.md /root/ring20-dashboard/.claude/commands/check-arc.md
"
# expect: 4 lines with md5 94d3eae7d8a17a6bd5356e930ff73d6b (handoff x2) + 14705456487d9293606c90f22d12e9e9 (check-arc x2)

# Optional: insert chat-arc Quick Reference rows into .121 CLAUDE.md if it exists
# (use the python helper from PL-117; copy /tmp/insert-chatarc-rows.py over and run)
```

---

## Phase 6 — Closeout (run on .107)

```
cd /opt/termlink

# Update G-049: mitigated → resolved (after the verification window holds)
$EDITOR .context/project/concerns.yaml
# - status: watching → resolved (or mitigated → resolved if you want the longer audit)
# - date_resolved: 2026-05-02
# - resolution_evidence: "T-1418 ship — 0.9.1702 + /var/lib/termlink, fleet doctor PASS, chat-arc skills propagated"

# Update T-1296: status work-completed (the migration was the deliverable)
.agentic-framework/bin/fw task update T-1296 --status work-completed --reason "migrated to /var/lib/termlink as part of T-1418 binary upgrade"

# Update T-1418: tick remaining Human ACs
$EDITOR .tasks/active/T-1418-*.md

# Mirror the milestone to .121 hub so vendored agents have local state
termlink channel post --hub ring20-dashboard \
  --msg-type milestone --metadata '_thread=T-1438' \
  --payload '[T-1418 + T-1438] ring20-dashboard upgraded to 0.9.1702 with /var/lib/termlink runtime_dir. Chat-arc skills propagated. Vendored agents on this hub now have full SEND + RECEIVE protocol surface. _from=opus-on-107' \
  agent-chat-arc

# Re-broadcast on .107 for fleet visibility
termlink channel post --topic agent-chat-arc \
  --msg-type milestone --metadata '_thread=T-1418' \
  --payload '[T-1418 closed] ring20-dashboard back in fleet at .121 with 0.9.1702 + durable runtime_dir. G-049 resolved. T-1296 closed.' \
  agent-chat-arc

# Schedule a 24h check-in cron similar to t1438-checkin.sh, but for .121 (canary SHAs from this run)
# (Copy/edit /opt/termlink/scripts/t1438-checkin.sh into a t1418-checkin.sh — change EXPECTED_VERSION,
#  EXPECTED_SECRET_SHA, EXPECTED_CERT_SHA. Run via crontab at +24h to confirm persistence held across
#  any reboot in the soak window.)

# Commit + push (no GitHub)
git add .context/project/concerns.yaml .tasks/active/T-1296*.md .tasks/active/T-1418*.md \
        .tasks/completed/T-1296*.md scripts/t1418-checkin.sh 2>/dev/null
.agentic-framework/bin/fw git commit -m "T-1418 + T-1296: ring20-dashboard at .121 → 0.9.1702 + /var/lib/termlink, G-049 resolved"
git push origin main
```

---

## Rollback path (if any phase fails)

If Phase 3 step 5 shows secret/cert SHAs DIFFERENT post-restart, the new binary regenerated them despite pre-seed — persist-if-present did not engage on this build. Rollback:

```
# On .121:
systemctl stop termlink-hub
cp /usr/local/bin/termlink.0.9.844.bak /usr/local/bin/termlink
unset TERMLINK_RUNTIME_DIR  # or revert systemd unit
# Repoint hub.sock back to /tmp/termlink-0/ if launcher was edited
TERMLINK_RUNTIME_DIR=/tmp/termlink-0 \
  nohup /usr/local/bin/termlink hub start --tcp 0.0.0.0:9100 &
```

Then on .107: re-run `termlink fleet reauth ring20-dashboard --bootstrap-from file:<new-secret-from-the-rolled-back-hub>`.

This puts you back at the pre-upgrade state — auth-healed at .121, runtime_dir still volatile, binary still 0.9.844. From there capture the regression as a learning and decide whether to wait on a different binary build.

---

## Reference

- CLAUDE.md "Hub Auth Rotation Protocol" — the protocol layer
- CLAUDE.md "Special case — volatile runtime_dir (T-1290 / T-1294)" — the persistence rule
- T-1294 task — runtime_dir migration documented for ring20-management (the parallel host)
- PL-021 — hub regenerates BOTH secret AND cert on volatile-runtime restart
- PL-115 — base64-over-remote-exec for skill push (Phase 5 pattern)
- PL-117 — idempotent python in-place edits for CLAUDE.md row insertion
- PL-119 — full unreachability vs. auth-mismatch (probe order: ping → SSH → port)
- PL-120 — bashrc PATH addition for vendored Claude Code agents (matches .141 pattern)
- PL-122 — modern channel.post envelope crashes pre-T-1155 hub (the trigger that left .121 in this state)
- T-1051 / T-1054 / T-1055 / T-1291 — fleet reauth protocol stack
- /opt/termlink/scripts/t1438-checkin.sh — template for the t1418 cron canary
