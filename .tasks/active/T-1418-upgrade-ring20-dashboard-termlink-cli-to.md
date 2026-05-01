---
id: T-1418
name: "Upgrade ring20-dashboard termlink-cli to T-1235 build (clear T-1166 cut blocker)"
description: >
  ring20-dashboard (TLS fingerprint sha256:53de15ec…, currently observed at
  192.168.10.143, previously .121) polls inbox.status ~620 times / 7d against
  this host's hub. After T-1166 cut flips, those calls return -32601 and the
  agent goes silently functional-dead. T-1235 dual-read shim translates
  inbox.status → channel.list at the SDK layer when the hub advertises
  channel.* capabilities. Upgrading the dashboard's termlink binary to a
  build that contains T-1235 makes its polling cut-survivable without any
  source change to the dashboard's own code. This is the last known holdout
  in the fleet's audit log.

status: started-work
workflow_type: build
owner: human
horizon: now
tags: [T-1166, T-1235, ring20-dashboard, cut-blocker, operator-runbook]
components: [target/release/termlink]
related_tasks: [T-1166, T-1235, T-1296, T-1417, T-1290]
created: 2026-04-30T08:11:23Z
last_update: 2026-04-30T08:11:23Z
date_finished: null
---

# T-1418: Upgrade ring20-dashboard termlink-cli to T-1235 build (clear T-1166 cut blocker)

## Context

This task is the **client-side counterpart** of T-1296 (which fixes the
hub-side runtime_dir persistence on the same host). They are independent:
T-1296 prevents secret rotation on reboot; T-1418 makes the polling client
survive the T-1166 cut. Either can ship first.

**Why this is the cut blocker.** Live audit (`fw metrics api-usage --last-Nd 7
--json` at 2026-04-30T08:10Z):

```
total legacy attributable: 630 in 7d
  └─ 620 of those: inbox.status from peer_ip 192.168.10.143
```

**Re-confirmed 2026-05-01T10:00Z** with `fleet doctor --legacy-usage` (T-1432)
+ raw `rpc-audit.jsonl` breakdown by peer_ip on the live .107 hub. Inter-arrival
from .143: ~60s (range 56–69s) of the triplet `hub.auth → session.list →
inbox.status`. Post-T-1235 hosts (.141, .122) show only `channel.*` traffic in
the same window — confirming T-1235's SDK shim routes correctly when the
client binary contains it. **.143 is the only outstanding host without T-1235.**
See T-1435 for the diagnostic write-up that re-validated this on 2026-05-01.

Every other source of legacy traffic in this fleet either ages out
(pre-T-1409 unattributable backlog) or is local stale sessions covered by
T-1417's bake. .143 is the one external client that needs an SDK upgrade
before the cut.

**Why T-1235 alone is sufficient.** The dashboard's polling code calls
`inbox.status` directly, but the call goes through
`crates/termlink-session/src/inbox_channel.rs::status_with_fallback`. After
T-1235, that function asks the hub for capabilities first, then routes to
`channel.list` if the hub supports it (this hub does). So the dashboard
doesn't need any application-level migration — just a binary upgrade.

**Local staged binary** (ready to ship):

- Path: `/opt/termlink/target/release/termlink`
- Version: `termlink 0.9.1591`
- SHA256: `484fef8801479163f80926cafe59577b5c65bf7ac849dea54ce6138d1a30be77`
- Built: 2026-04-30T00:15Z (this session's source tree, contains T-1235 +
  T-1417 + all pre-bake ships)

**Identity continuity.** ring20-dashboard re-numbers across container
restarts. The TLS fingerprint `sha256:53de15ec…` in `~/.termlink/known_hubs`
identifies it regardless of IP. As of 2026-04-30T08:12Z the pin is at
`192.168.10.121:9100` (last_seen 2026-04-29). If the operator finds it at a
different IP, the fingerprint match confirms identity.

## Operator Runbook

**Pre-step (required, 2026-04-30):** the dashboard hub at .143 has a
rotated secret since this task was drafted. Heal auth first:

```bash
cd /opt/termlink
termlink fleet reauth ring20-dashboard
# follow the printed steps to fetch the new secret OOB and write it
# to /root/.termlink/secrets/ring20-dashboard.hex
termlink fleet doctor   # expect [PASS] for ring20-dashboard
```

Once auth heals, **Method 0** below is the single-command path. The older
Methods A/B/C are kept as fallbacks for environments where Method 0 doesn't
apply.

### Method 0 — fleet-deploy-binary.sh (T-1421, recommended)

After auth heals (above), deploy in one command:

```bash
cd /opt/termlink
./scripts/fleet-deploy-binary.sh ring20-dashboard --swap-restart
```

This:
- Auto-discovers the remote session
- Streams the local `target/release/termlink` (sha pinned via the script's
  pre-check) in 45KB chunks via `remote exec` base64 pipes
- Assembles + sha-verifies on the remote
- Generates and runs a self-detached swap+restart script that handles the
  NTFS DrvFs file-lock case (rm-then-cp + 5s settle + setsid relaunch)
- Verifies the hub came back

Idempotent: if the running hub already matches the local sha, exits early.

### Method A — termlink send-file (preferred if .143 has working hub)

```bash
# From /opt/termlink (this host):
termlink fleet doctor                                # confirm ring20-dashboard PASS
termlink file send ring20-dashboard \
  --src target/release/termlink \
  --dst /tmp/termlink.new

# Then on the dashboard host (operator opens shell there):
sha256sum /tmp/termlink.new
# Expect: 484fef8801479163f80926cafe59577b5c65bf7ac849dea54ce6138d1a30be77

install -m 0755 /tmp/termlink.new /usr/local/bin/termlink     # atomic replace
# (or whatever path `which termlink` returns on the dashboard)
```

If `fleet doctor` shows ring20-dashboard FAIL, fall back to Method B or C.

### Method B — scp via SSH

```bash
# From /opt/termlink (this host):
scp target/release/termlink root@192.168.10.143:/tmp/termlink.new
# (or whatever current IP — check known_hubs last_seen, or arp scan)

# Then on the dashboard host:
ssh root@192.168.10.143
sha256sum /tmp/termlink.new
# Expect: 484fef8801479163f80926cafe59577b5c65bf7ac849dea54ce6138d1a30be77
install -m 0755 /tmp/termlink.new /usr/local/bin/termlink
```

### Method C — build on target

```bash
# On the dashboard host (needs rust toolchain + ~5min):
cd /tmp && git clone https://github.com/<org>/termlink.git  # or: cp -a from a mounted share
cd termlink && cargo build --release
install -m 0755 target/release/termlink /usr/local/bin/termlink
```

### Common — restart polling agents

**Critical step.** Replacing the binary on disk does NOT reload the SDK in
already-running processes. The polling agents must be restarted to pick up
T-1235.

```bash
# On the dashboard host:
pgrep -af termlink                                   # list current termlink processes
# Restart whatever supervises the dashboard's polling agent:
systemctl restart ring20-dashboard.service           # if systemd-managed
# OR
pkill -f 'ring20-dashboard.*poll'                    # if watchdog-managed (auto-respawns)
# OR
# whatever the actual agent invocation is — check `pgrep -af` output above

pgrep -af termlink                                   # confirm new PIDs
```

### Verification (run on /opt/termlink, this host)

**Preferred — freshness check (T-1419, available since 2026-04-30):**
the rolling-window count includes pre-restart calls aging out, but
`last_seen_iso` answers "did .143 call AFTER the restart?" directly.

```bash
DEPLOY_TS=$(date -u +%Y-%m-%dT%H:%M:%SZ)        # capture BEFORE deploy
# … perform deploy + restart …
sleep 60                                         # let the log catch up
.agentic-framework/bin/fw metrics api-usage --last-Nd 1 --json 2>/dev/null | \
  python3 -c "
import json, sys, os
deploy = os.environ.get('DEPLOY_TS', '')
d = json.load(sys.stdin)
for r in d.get('legacy_callers_by_ip', []):
    if r['peer_ip'] != '192.168.10.143':
        continue
    last = r.get('last_seen_iso', '')
    if last < deploy:
        print(f'PASS — .143 last_seen_iso={last} < deploy={deploy}')
        sys.exit(0)
    print(f'FAIL — .143 last_seen_iso={last} >= deploy={deploy} (still calling legacy)')
    sys.exit(1)
print('PASS — .143 has no legacy entries in window')
"
```

**Fallback — count check** (works without T-1419, but conflates live
calls with rolling-window residue; only definitive after a full 1d
window has passed since deploy):

```bash
.agentic-framework/bin/fw metrics api-usage --last-Nd 1 --json 2>/dev/null | \
  python3 -c '
import json, sys
d = json.load(sys.stdin)
hits = sum(r["count"] for r in d.get("legacy_callers_by_ip", []) if r["peer_ip"] == "192.168.10.143")
print(f".143 legacy hits in 1d window: {hits}")
sys.exit(0 if hits == 0 else 1)
'
```

After a full 7d clean window, the cut-ready gate flips:

```bash
.agentic-framework/bin/fw metrics api-usage --cut-ready --json
# Expect: {"cut_ready": true, "window_days": 7, "legacy_attributable": 0, ...}
```

## Acceptance Criteria

### Agent
- [x] Local binary built and staged at `target/release/termlink` —
  version `0.9.1591`, sha256 `484fef88…1a30be77`. Confirmed contains T-1235
  shim (`grep -l "T-1235" crates/termlink-session/src/inbox_channel.rs`
  returns the file).
- [x] Operator runbook documents three transfer paths
  (termlink/scp/build-on-target), each with sha verification + atomic
  replacement command + restart step.
- [x] Verification recipe extracts `.143` peer_ip count from
  `fw metrics api-usage --json`; PASS condition is zero hits over 1d.
- [x] Cut-ready gate recipe documented for the 7d post-bake check.

### Human
- [ ] [RUBBER-STAMP] Binary deployed on dashboard host
  **Steps:**
  1. Pick Method A, B, or C from the runbook above based on access
  2. Verify sha256 on the dashboard host matches `484fef88…1a30be77`
  3. Atomically replace whatever path `which termlink` reports there
  **Expected:** `termlink --version` on the dashboard reports `0.9.1591`.
  **If not:** check that the install path matches `which termlink` (some
  hosts have multiple copies — ~/.cargo/bin, /usr/local/bin, /usr/bin).

- [ ] [RUBBER-STAMP] Polling agent restarted
  **Steps:**
  1. `pgrep -af termlink` on the dashboard host before restart, note PIDs
  2. Restart the supervising service (systemd, watchdog, or pkill+respawn)
  3. `pgrep -af termlink` after, confirm new PIDs
  **Expected:** All termlink processes have new PIDs and start times.
  **If not:** the agent may be supervised by something not yet restarted —
  check parent process tree.

- [ ] [REVIEW] Migration confirmed via fleet metrics
  **Steps:**
  1. Wait ≥10 minutes after restart
  2. From /opt/termlink: run the 1d verification recipe in the runbook
  3. After ≥7 days: run `fw metrics api-usage --cut-ready --json`
  **Expected:** 1d-window check shows 0 hits from .143; 7d cut-ready gate
  reports `cut_ready: true`.
  **If not:** investigate whether the dashboard has multiple agents holding
  termlink sessions (only one was migrated). Re-check `pgrep -af termlink`.

## Verification

# Confirms local staging is intact and contains the T-1235 shim
test -f target/release/termlink
target/release/termlink --version | grep -q "termlink 0\\."
sha256sum target/release/termlink | grep -q "484fef8801479163f80926cafe59577b5c65bf7ac849dea54ce6138d1a30be77"
grep -q "T-1235" crates/termlink-session/src/inbox_channel.rs

## Decisions

### 2026-04-30 — Why a separate task from T-1296

- **Chose:** New task (T-1418) for the client-side upgrade
- **Why:** T-1296 fixes hub-side runtime_dir persistence (independent of
  T-1166 cut). T-1418 fixes client-side polling method (the cut blocker).
  Different files touched, different ACs, different verification. CLAUDE.md:
  "One bug = one task."
- **Rejected:** Bundling into T-1296 — would conflate two distinct
  operational concerns and dilute the cut-blocker tracking.

### 2026-04-30 — Why no application-level change on the dashboard

- **Chose:** Binary-only upgrade; rely on T-1235 SDK-layer rewrite
- **Why:** T-1235 was designed precisely so callers don't have to change.
  The dashboard's polling code keeps calling `inbox.status` in source; the
  SDK silently routes to `channel.list` when supported. Zero source change
  on the dashboard repo.
- **Rejected:** Application-level migration to channel.list — would require
  source changes in a repo we don't currently have access to, and is
  unnecessary because T-1235 covers exactly this case.

## Updates

### 2026-04-30T19:25Z — runbook simplified to Method 0 [agent autonomous pass]

After T-1421 codified PL-096 as `scripts/fleet-deploy-binary.sh`, updated
the runbook so operators have a single one-liner once auth heals:
`./scripts/fleet-deploy-binary.sh ring20-dashboard --swap-restart`. The
ad-hoc Method B (SSH) and Method A (termlink file send) are still
documented as fallbacks but Method 0 supersedes them for this fleet
topology.

### 2026-04-30T18:26Z — host located but auth-blocked [agent autonomous pass]

**Identity confirmed at 192.168.10.143.** TCP :9100 reachable; TOFU
fingerprint match: `sha256:53de15ec8b33b4…6fe4` — same hub identity as
the prior .121 pin. Updated hubs.toml address from .121→.143 (backup at
`~/.termlink/hubs.toml.bak.<ts>`). ARP confirms .143 is the live ring20
host.

**Hub secret has rotated since last pin.** `fleet doctor` returns
`-32010 Token validation failed: invalid signature`, hint:
`Secret mismatch — hub was likely restarted with a new secret`. Per
T-1054 Tier-1 heal, the new secret must be fetched out-of-band. SSH from
.107 to .143 fails (permission denied: publickey/password); SSH from
.122 to .143 fails the same way (known_hosts mismatch + no pubkey).
**No autonomous OOB channel available** from this host — the heal is
operator-only without setting up an SSH key first.

**Streaming-transport readiness.** PL-096 (b64-stream-via-remote-exec)
just succeeded on .141 (T-1420). The same script template will work for
.143 once auth heals, because the legacy `file send` path is also blocked
by PL-095 there. Expect ~2 min total deploy time from auth-heal.

**Operator next step (single command):**
```
termlink fleet reauth ring20-dashboard
```
…then paste the .143 hub.secret value into the printed echo.

### 2026-04-30T08:11:23Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1418-upgrade-ring20-dashboard-termlink-cli-to.md
- **Context:** Initial task creation

### 2026-04-30T08:15Z — runbook-drafted [agent autonomous]
- **Action:** Filled context, three-method operator runbook (send-file/scp/build-on-target), verification recipes (1d + 7d), and decisions section.
- **Agent ACs:** all four ticked — binary staged at `target/release/termlink` (v0.9.1591, sha 484fef88…), runbook covers all transfer paths + restart, verification extracts .143 peer_ip count and chains to cut-ready gate.
- **Human ACs:** three drafted — deploy/restart/confirm.
- **Owner flipped to human:** agent ACs done; remaining work is operator-side (binary transfer + restart on the dashboard host).
- **Context:** Task created in response to user request "draft a one-shot upgrade procedure for .143". The cut blocker is now staged with a copy-pasteable runbook.
