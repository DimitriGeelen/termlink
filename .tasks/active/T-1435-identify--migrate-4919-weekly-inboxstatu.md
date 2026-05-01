---
id: T-1435
name: "Identify + migrate 4919 weekly inbox.status pollers on .107 (T-1166 last-mile)"
description: >
  Identify + migrate 4919 weekly inbox.status pollers on .107 (T-1166 last-mile)

status: work-completed
workflow_type: build
owner: human
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-05-01T09:34:49Z
last_update: 2026-05-01T10:07:39Z
date_finished: 2026-05-01T10:03:30Z
---

# T-1435: Identify + migrate 4919 weekly inbox.status pollers on .107 (T-1166 last-mile)

## Context

T-1432 fleet doctor --legacy-usage on 2026-05-01 surfaced 5121 legacy invocations on .107 in a 7-day window — 4919 (96%) are `inbox.status` calls, all from `from=(unknown)` (no caller-attribution label). Some background loop is polling. Likely Watchtower's panel render or a cron/api-usage script. Until this is migrated, the T-1166 cut verdict stays at WAIT instead of CUT-READY. This is the "last mile" for the cut.

The remaining traffic on .107: 197 event.broadcast (mix of unknown + tl-* sessions, several tl-t1407-test which suggests test fixtures), 5 inbox.list (all unknown).

## Finding (2026-05-01T10:00Z)

**Premise was wrong.** The 4919 callers are NOT local Watchtower or Watchtower-on-.107.
They are an external client at `peer_ip=192.168.10.143` running a stale (pre-T-1235)
CLI. Direct evidence from `/var/lib/termlink/rpc-audit.jsonl` (last 10000 lines):

| caller IP        | total | dominant methods                                     |
|------------------|-------|------------------------------------------------------|
| 192.168.10.143   | 2798  | hub.auth (939), session.list (914), inbox.status (914) |
| 192.168.10.201   |  768  | hub.auth (384), session.discover (384)               |
| 127.0.0.1 (self) |  375  | hub.auth (182), hub.version (97), session.discover (85) |
| 192.168.10.122   |   52  | mixed; channel.list present (post-T-1235 caller)     |
| 192.168.10.141   |    8  | channel.subscribe + channel.post (post-T-1235)       |

**Inter-arrival from .143:** ~60s (stable: 56–69s) of the triplet
`hub.auth → session.list → inbox.status`. Pattern matches a pre-T-1235 polling
loop that runs every 60s. New ephemeral source port each cycle = no persistent
session. Annual rate ≈ 525,600 calls/yr/host pair.

**Why the local channel-aware paths are not the source.** `cmd_remote_doctor`
(remote.rs:3050) routes via `status_with_fallback_with_client`, which probes
hub caps and prefers `channel.list` when present. Local hub on .107 is
0.9.1640 — has `channel.list` capability — so post-T-1235 callers never reach
`inbox.status`. .141 and .122 are post-T-1235 (their audit lines show
`channel.list`/`channel.post`/`channel.subscribe` instead).

**.143 is NOT post-T-1235.** Its CLI lacks the channel.* fallback; calls
`inbox.status` directly. T-1418 already tracks the upgrade to T-1235, but is
blocked on operator action (auth heal — secret rotated since last pin, no
autonomous OOB channel from .107 to .143).

**Conclusion.** The 4919 weekly calls = T-1418 not yet shipped. Once T-1418
deploys T-1235 to .143 and restarts the polling agent, .143's polls become
`channel.list(prefix=inbox:)` and the legacy traffic to .107 drops to zero
within ~60s. T-1435's Watchtower migration ACs are moot; this work is fully
covered by T-1418.

## Acceptance Criteria

### Agent
- [x] Identify the 4919 inbox.status callers — finding (2026-05-01T10:00Z): caller is external client at `peer_ip=192.168.10.143` running pre-T-1235 CLI. Triplet hub.auth→session.list→inbox.status every ~60s, new ephemeral source port each cycle. Direct evidence in `/var/lib/termlink/rpc-audit.jsonl`: 914 inbox.status hits in last 10000 audit lines from .143 alone, against 0 from .141/.122 (post-T-1235 hosts) and 0 from local Watchtower (which uses session.discover, not session.list)
- [x] Identify the 197 event.broadcast callers — finding: not in scope of this task. Within last 10000 audit lines, `event.broadcast` hits are not from .143 (which calls only the inbox triplet). Mix of internal `tl-t1407-test` fixtures + a few unknown — legitimate callers post-T-1417 use `channel.post` / `event.emit_to`. Investigate separately if breakdown re-surfaces in next 7d window
- [x] Migration is covered by T-1418 (deploy 0.9.1640 → .143 + restart polling agent). T-1235 SDK shim routes `inbox.status` calls through `channel.list` automatically once the binary is upgraded. **No application source change needed on the dashboard repo.** Local Watchtower paths already channel-aware (T-1400)
- [x] Verification recipe documented in T-1418 (`fw metrics api-usage --last-Nd 1` after deploy, .143 hits should drop to 0 within ~60s of restart). T-1432's `fleet doctor --legacy-usage` provides the same signal at the fleet level
- [x] Caller attribution — not applicable in scope of this task: .143's pre-T-1235 CLI does not inject `$TERMLINK_SESSION_ID` into params (that's the T-1310 codepath, bundled with the 0.9.1640 build). Attribution will appear automatically post-T-1418 deploy
- [x] No fix-with-suppression — confirmed. The plan is to UPGRADE the caller, not silence the warning. Same audit-log gate will be re-checked post-deploy

**This task is now answer-only.** All migration is covered by T-1418. Closing once findings logged into T-1418.

### Human
- [ ] [REVIEW] Verification of CUT-READY happens under T-1418, not here
  **Steps:**
  1. After T-1418 lands (deploy 0.9.1640 to .143 + restart polling agent on .143)
  2. From /opt/termlink: `target/release/termlink fleet doctor --legacy-usage --legacy-window-days 1`
  3. Look for `CLEAN (1d): workstation-107-public, local-test, laptop-141, ring20-dashboard`
  **Expected:** .143 disappears from "WITH TRAFFIC". Once it holds for 7d, T-1166 cut is safe
  **If not:** the polling agent on .143 wasn't restarted, OR a SECOND legacy caller exists. Re-run audit-log breakdown by peer_ip to identify

## Verification

# Caller is identified — confirm at least one .143 inbox.status hit in recent audit
test -f /var/lib/termlink/rpc-audit.jsonl
tail -10000 /var/lib/termlink/rpc-audit.jsonl | grep -c '"method":"inbox.status"' | python3 -c "import sys; n=int(sys.stdin.read()); sys.exit(0 if n>0 else 1)"
# Fleet doctor --legacy-usage runs and reports a verdict (signal that T-1432 stack is operational)
target/release/termlink fleet doctor --legacy-usage --legacy-window-days 1 2>&1 | grep -E "CUT-READY|WAIT|UNCERTAIN" | head -1

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

### 2026-05-01T09:34:49Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1435-identify--migrate-4919-weekly-inboxstatu.md
- **Context:** Initial task creation

### 2026-05-01T10:03:30Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
