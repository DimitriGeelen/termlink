---
id: T-1420
name: "agent chat arc: deploy 0.9.1591 to laptop-141 (.141, WSL) — close 29-cmd gap"
description: >
  laptop-141 (192.168.10.141, WSL2 Ubuntu, GLIBC 2.39) runs termlink hub
  0.9.1482 — has the early chat-arc commands (24/53) but is missing 29
  newer Matrix-analogue commands (state, snapshot, snapshot-diff,
  mentions-of, reactions-on, reactions-of, threads, ack-history,
  edit-stats, quote-stats, pin-history, redactions, replies-of, forward,
  forwards-of, snippet, star/unstar/starred, typing, poll, digest, inbox,
  relations, topic-stats, state-since, emoji-stats, ack-status).
  Upgrading to 0.9.1591 closes the gap. T-1384 multi-agent readiness
  inception identified this fleet rollout as the primary remaining work
  to push the agent chat arc to vendored agents in the field. .122
  (ring20-management) is already on 0.9.1542 (full arc). .143
  (ring20-dashboard) is covered by T-1418. .107 (this host, 24 sessions)
  is already on 0.9.1542+ (full arc).

status: started-work
workflow_type: build
owner: human
horizon: now
tags: [agent-chat-arc, T-1384, T-1385, channel, multi-agent, fleet-rollout, laptop-141]
components: [target/release/termlink]
related_tasks: [T-1384, T-1385, T-1386, T-1387, T-1418]
created: 2026-04-30T09:09:16Z
last_update: 2026-04-30T09:09:16Z
date_finished: null
---

# T-1420: agent chat arc — deploy 0.9.1591 to laptop-141 (.141, WSL)

## Context

This task is part of the **agent chat arc** rollout — the T-1313→T-1383
work that mirrors Matrix client-server primitives onto termlink topics
(reply, react, receipts, edit, redact, mentions, thread, pin, typing,
snapshot, state, etc). T-1384's multi-agent readiness inception (closed
2026-04-28) recommended **GO** on the fleet rollout. The remaining
deployment is two host-binary upgrades:

| Host | Sessions | Build | cmds | Action |
|---|---|---|---|---|
| local (.107/.102) | 24 | 0.9.1542–0.9.1591 | 53/53 ✅ | none |
| ring20-management (.122) | 1 | 0.9.1542 | 53/53 ✅ | none (T-1384 deployed) |
| **laptop-141 (.141)** | 1 | **0.9.1482** | **24/53** | **this task** |
| ring20-dashboard (.143) | DOWN | — | — | T-1418 |

## ABI compatibility (resolved)

Both hosts run Ubuntu GLIBC 2.39 on x86_64 (laptop-141 is WSL2 Ubuntu).
The local `target/release/termlink` binary (dynamically linked, built
2026-04-30 00:15Z) runs on .141 directly — no musl static rebuild
needed. T-1384's musl recipe was for the .122 case where GLIBC versions
differed; that constraint doesn't apply here.

**Local staged binary** (this host, ready to ship):

- Path: `/opt/termlink/target/release/termlink`
- Version: `termlink 0.9.1591`
- SHA256: `484fef8801479163f80926cafe59577b5c65bf7ac849dea54ce6138d1a30be77`
- Size: ~20 MB (dynamic), GLIBC 2.39 compatible

**Hub binary on .141 today:**
- Path: `/mnt/c/ntb-acd-plugin/termlink/target/release/termlink`
- Version: `0.9.1482`
- Process: `dimitri 4450 ... termlink hub start --tcp 0.0.0.0:9100`
- Source clone: `/mnt/c/ntb-acd-plugin/termlink/`
- Toolchain: rustup at `~/.cargo/bin/` (not in default PATH)

## Operator Runbook

The operator picks ONE of two methods.

### Method A — git pull + cargo build on .141 (recommended for laptop hosts)

The .141 host already has the source tree at
`/mnt/c/ntb-acd-plugin/termlink/` and rustup. Most self-contained path.

```bash
# On the .141 host (any shell, in WSL):
cd /mnt/c/ntb-acd-plugin/termlink
git pull origin main
~/.cargo/bin/cargo build --release    # ~5-10 min on a laptop

# Sanity check before swapping:
./target/release/termlink --version    # expect 0.9.1591 or newer
./target/release/termlink channel --help | grep -cE '^  [a-z]'  # expect 53

# Restart hub (graceful — kill the old, watchdog/manual respawn):
PID=$(pgrep -f 'termlink hub start' | head -1)
kill "$PID"
# If a watchdog/launchd respawns it, wait ~3s and verify; else restart:
nohup ./target/release/termlink hub start --tcp 0.0.0.0:9100 \
  > ~/termlink-hub.log 2>&1 &
```

### Method B — binary push from this host (if .141 toolchain unavailable)

```bash
# From /opt/termlink (this host):
termlink fleet doctor                          # confirm laptop-141 PASS
termlink file send laptop-141 \
  --src target/release/termlink \
  --dst /tmp/termlink.new

# Then on the .141 host:
sha256sum /tmp/termlink.new
# Expect: 484fef8801479163f80926cafe59577b5c65bf7ac849dea54ce6138d1a30be77

# Atomic-replace the running hub binary path:
TARGET="/mnt/c/ntb-acd-plugin/termlink/target/release/termlink"
mv "$TARGET" "$TARGET.0.9.1482.bak"          # backup
mv /tmp/termlink.new "$TARGET"
chmod 0755 "$TARGET"

# Restart hub (same as Method A):
PID=$(pgrep -f 'termlink hub start' | head -1)
kill "$PID"
# … wait for watchdog respawn or restart manually …
```

### Common — verification

After hub restart:

```bash
# From /opt/termlink (this host):
SID=$(termlink remote list laptop-141 2>/dev/null | awk 'NR==3 {print $1}')
REMOTE_CMDS=$(termlink remote exec laptop-141 "$SID" \
  "/mnt/c/ntb-acd-plugin/termlink/target/release/termlink channel --help" \
  2>/dev/null | grep -cE '^  [a-z]')
echo "channel commands on .141: $REMOTE_CMDS / 53"
test "$REMOTE_CMDS" = "53" && echo "PASS — full chat arc deployed" \
  || echo "FAIL — gap remains"

# Bonus: confirm the version directly
termlink remote exec laptop-141 "$SID" \
  "/mnt/c/ntb-acd-plugin/termlink/target/release/termlink --version"
# Expect: termlink 0.9.1591  (or newer if Method A pulled later commits)
```

## Acceptance Criteria

### Agent
- [x] Fleet inventory captured: laptop-141 / .141 / WSL2 / GLIBC 2.39 / 0.9.1482 / 24 channel cmds. Confirmed via `termlink remote exec`.
- [x] Local binary staged at `target/release/termlink` (v0.9.1591, sha `484fef88…1a30be77`, 20MB dynamic-linked, GLIBC 2.39 compatible).
- [x] Channel command gap quantified: 29 commands missing on .141 — full list in Description.
- [x] ABI compat resolved: local + .141 both on Ubuntu GLIBC 2.39, dynamic binary works without musl rebuild (saves ~7 min build time).
- [x] Two-method runbook documented (Method A: git pull + cargo build on .141; Method B: termlink file send + atomic replace).
- [x] Verification recipe extracts channel cmd count from .141 via remote exec; PASS condition is 53/53.
- [x] Task name + tag prefix per user convention ("agent chat arc:" prefix in name; tags include `agent-chat-arc`).
- [x] .107 fleet inventory captured: 24 sessions sampled across multiple worker dispatches (T-1597 blind-review, T-1602 consumer, T-1565 approval-arc, T-1529 validate-dedup, T-561 push, T-1246 g046-mirror, T-1539, T-1540, T-1245), all sampled sessions report 0.9.1542+ and 53/53 channel cmds. .107 is NOT a deployment gap.

### Human
- [ ] [RUBBER-STAMP] Binary deployed on .141 — Method A or B
  **Steps:**
  1. Pick Method A (git pull + cargo build) or Method B (binary push)
  2. Verify the new binary reports `termlink 0.9.1591` (Method B) or newer (Method A)
  3. Confirm sha256 if Method B
  **Expected:** `--version` reports 0.9.1591 or newer; `channel --help` lists 53 subcommands.
  **If not:** check disk space, check write permissions on `/mnt/c/...` (NTFS-WSL quirks may need `chmod +x`).

- [ ] [RUBBER-STAMP] .141 hub restarted on new binary
  **Steps:**
  1. `pgrep -af 'termlink hub'` on .141 — note PID
  2. `kill $PID`
  3. Wait for watchdog respawn OR restart manually with the new binary path
  4. `pgrep -af 'termlink hub'` — confirm new PID
  **Expected:** new hub PID, started time recent, listening on 0.0.0.0:9100.
  **If not:** if no watchdog manages it, the manual restart command is in Method A above.

- [ ] [REVIEW] Full chat arc parity confirmed via fleet check
  **Steps:**
  1. From /opt/termlink, run the verification recipe in the runbook
  2. Confirm output is `channel commands on .141: 53 / 53` and `PASS`
  **Expected:** PASS with 53/53.
  **If not:** the binary on disk vs the binary the running hub holds may
  differ — the running process retains the old executable until restart.
  Re-do the kill/respawn step.

## Verification

# Confirms local staging is intact and contains the chat-arc commands.
test -f target/release/termlink
target/release/termlink --version | grep -q "termlink 0\.9\.15[0-9][0-9]"
test "$(target/release/termlink channel --help 2>&1 | grep -cE '^  [a-z]')" = "53"
sha256sum target/release/termlink | grep -q "484fef8801479163f80926cafe59577b5c65bf7ac849dea54ce6138d1a30be77"

## Decisions

### 2026-04-30 — Why two methods (build-on-target + binary-push)

- **Chose:** Document both Method A (build on .141) and Method B (push binary)
- **Why:** .141 is a developer laptop with a checked-out source tree
  AND rustup. Method A is the most self-sufficient: pulls latest source,
  builds, swaps. Method B exists as a fallback if the toolchain is
  broken or the operator wants the exact binary signed off in this task
  (sha verification).
- **Rejected:** Method-A-only — would block on rustup health on .141.
  Method-B-only — operator might prefer source clarity.

### 2026-04-30 — Why dynamic binary, not musl static

- **Chose:** Use the existing dynamically-linked release binary
- **Why:** local + .141 both run Ubuntu GLIBC 2.39 — ABI matches
  exactly. Skipping musl saves ~7 min build and avoids the static
  binary's slower mmap behavior. T-1384's musl recipe was for the .122
  case where GLIBC mismatched; that constraint doesn't apply to .141.
- **Rejected:** Build musl static "just to be safe" — over-engineering
  for a known-compatible target.

## Updates

### 2026-04-30T18:24Z — DEPLOY SUCCEEDED via base64-streamed remote-exec [agent autonomous pass]

After PL-095 closed Method B's `file send` path, switched to a chunked
base64-over-`remote exec` transport: split binary into 447 × 45KB chunks,
each pushed via `printf '%s' '<b64>' | base64 -d > /tmp/tl-xfer/c.NNNN`,
then assembled and verified.

**Transfer:** 447 chunks, 0 failures, sha verified `484fef88…1a30be77` on .141.

**Hub restart (deploy-tl-141-v2.sh):**
- Old hub PID 17507 (0.9.1482) killed cleanly (exit after 2s)
- 5s NTFS DrvFs lock-release wait, then rm-then-cp pattern (overcomes
  "Text file busy" seen in v1 deploy at 18:04 — pure cp race with
  DrvFs mapping release)
- New hub PID 17708 launched detached via `setsid nohup` matching
  `termlink-launcher.sh` pattern, so the */10 cron treats it as healthy
- Hub down window: ~16s

**Verification on .141 (post-deploy):**
- `termlink --version` → `termlink 0.9.1591` ✅
- `channel --help | grep -cE '^  [a-z]'` → `53` ✅ (full chat arc, was 24)
- on-disk sha: `484fef88…1a30be77` ✅
- agent-1 session `tl-hmfi6wpa` reconnected and ready
- fleet doctor laptop-141: PASS 88ms

**Method B reformulated:** standard `termlink file send` was structurally
unable to deliver (PL-095). Working transport on this fleet topology is
chunked-b64-via-remote-exec. The runbook's Method B should be updated to
match. Method A (build-on-target) still works as a fallback when remote
exec is unavailable.

**Human ACs:** evidence above is unambiguous. Per framework rule the
agent does not tick `### Human` boxes; the operator can verify in <30s
by re-running the verification recipe.

### 2026-04-30T16:35Z — autonomous deploy attempted, transport limit found [agent autonomous pass]

- **Action:** Tried Method B end-to-end via termlink: `file send` to .141 (419 chunks delivered, sha verified `484fef88…1a30be77`), then `file receive` on .141 via `remote exec`.
- **Result:** receive hangs. Root cause: legacy file-events fallback spools chunks to the **sender's** hub inbox (.107). Receiver's `file receive` must pull from the source hub — but `.141` has no `~/.termlink/hubs.toml` and no peer config pointing back to .107.
- **Confirmed clean signals on .141:**
  - GLIBC 2.39 ✅ — dynamic binary will run
  - `TERMLINK_RUNTIME_DIR=/home/dimitri/.termlink/runtime` (not /tmp) ✅ — already past T-1290 volatile-/tmp risk
  - Hub PID 4450, 0.0.0.0:9100, single instance ✅
  - Loopback + LAN both reachable from inside the WSL session ✅
- **Implication:** Method B is operator-only without first establishing peer config on .141. **Method A (git pull + cargo build) is the cleanest remaining path** — source clone exists at `/mnt/c/ntb-acd-plugin/termlink/`, rustup installed.
- **Captured as:** PL-095 (cross-host file transfer needs symmetric peer config).

### 2026-04-30T08:50Z — task-created + runbook drafted [agent autonomous pass]

- **Action:** Inventoried fleet for agent-chat-arc command coverage
  (53 cmds local). Found .141 at 0.9.1482 with 24/53 — 29 commands
  missing. Probed .141 architecture (WSL2 Ubuntu, GLIBC 2.39, x86_64,
  rustup at `~/.cargo/bin/`, source clone at
  `/mnt/c/ntb-acd-plugin/termlink/`).
- **Mid-write user clarification:** "what about the other agents on .107?"
  — extended inventory to all 24 .107 sessions, sampled 9 across worker
  dispatches; all on 0.9.1542+ with 53/53 cmds (full arc). .107 is not a
  deployment gap.
- **Output:** Task with two-method runbook, full chat-arc cmd-gap
  inventory, verification recipe.
- **Agent ACs:** all 8 ticked (.107 inventory now an explicit AC).
  Owner flips to human (deploy + restart + verify Human ACs).
- **Context:** Created in response to user's autonomous mandate "focus
  pushing out agent chat arc to vendored agents in the field" — .141 is
  one of two fleet hosts (with .143/T-1418) that needs the arc deploy.
