---
id: T-1428
name: "Foundation soak audit — T-1426 + T-1427 ship status (2-week check)"
description: >
  Scheduled audit fire date: 2026-05-14. Check whether T-1426 (deprecation print on legacy primitives) and T-1427 (termlink whoami + identity binding) have shipped, and gather T-1166 cut-readiness signal from any deprecation telemetry the picks may have produced. This is a foundation-soak sentinel — created at the same time as T-1425 inception RFC + T-1426/T-1427 captures so the system has a structural reminder to re-check that the pre-cut foundation actually got built.

status: captured
workflow_type: build
owner: human
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-04-30T21:21:07Z
last_update: 2026-04-30T21:21:07Z
date_finished: null
---

# T-1428: Foundation soak audit — T-1426 + T-1427 ship status (2-week check)

## Context

Foundation-soak audit fires on 2026-05-14, 14 days after T-1425 RFC post (2026-04-30T21:13Z) and creation of T-1426 / T-1427 / this sentinel. Purpose: verify that the pre-T-1166-cut foundation actually got built (T-1426 deprecation print + T-1427 strict-reject identity binding) and gather the cut-readiness signal from real chat-arc soak telemetry. ACs backfilled by T-1450 (originally created with empty placeholders — would have fired silently on 2026-05-14).

## Acceptance Criteria

### Agent

- [ ] **T-1426 (deprecation print) shipped to fleet** — fleet binaries report version >= 0.9.1638 (commit 81395ce4 introduced the deprecation print). Verify per host via `termlink fleet status` + remote `--version` probe; record version per host.
- [ ] **T-1427 (whoami + identity binding) shipped to fleet** — fleet binaries report version >= 0.9.1688 (commit 0c0b3bfc introduced strict-reject). Verify per host. Spot-check at least one hub by attempting a forged `--sender-id imposter` post and confirming `-32014 CHANNEL_IDENTITY_MISMATCH` rejection.
- [ ] **Chat-arc soak telemetry — post count climbed** — `termlink channel info agent-chat-arc` (per hub) shows posts > 54 (the 2026-05-02 mid-soak baseline). Record post count per hub.
- [ ] **Chat-arc soak telemetry — sender count climbed** — at least 3 distinct sender_ids visible across all hubs combined. Pre-audit baseline was 2 (.107 d1993c2c, .122 9219671e). Adding .141 (6604a2af) was already observed pre-audit; 4 senders means .143 / .121 also active = full fleet participation.
- [ ] **Cut-readiness signal collected** — `fw fleet doctor --legacy-usage` per hub. Record verdict per hub (CUT-READY / WAIT / UNCERTAIN). Aggregate across fleet: cut is safe iff all UP hubs report CUT-READY.
- [ ] **Audit findings written to T-1428 Updates section** — single dated entry with: T-1426 ship status, T-1427 ship status, post count delta, sender count delta, per-hub legacy-usage verdict, aggregate cut-readiness recommendation.

### Human

- [ ] [REVIEW] Approve cut-readiness verdict for T-1166
  **Steps:**
  1. Read the audit-findings entry the agent appended to `## Updates` below
  2. Verify the agent's per-host evidence matches your independent check (open Watchtower /fleet or run `termlink fleet status` from your operator session)
  3. Decide: GO (retire legacy primitives now), DEFER (extend soak by N days, name new sentinel), or BLOCK (a foundation didn't actually ship, fix that first)
  **Expected:** Decision recorded as a `## Decisions` entry on this task or on T-1166 directly
  **If not:** Identify which AC's evidence is insufficient; ask agent to re-run with more detail

## Verification

# Shell commands that MUST pass before work-completed. One per line.
# Lines starting with # are comments (skipped). Empty lines ignored.
# Run these on audit day (2026-05-14) — they are the mechanical floor for AC1, AC3, AC5.
termlink fleet status 2>&1 | grep -qE 'UP[[:space:]]+(workstation-107|local-test|laptop-141|ring20-management|ring20-dashboard)'
termlink channel info agent-chat-arc 2>&1 | grep -qE 'Posts:[[:space:]]+[0-9]+'

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

### 2026-04-30T21:21:07Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1428-foundation-soak-audit--t-1426--t-1427-sh.md
- **Context:** Initial task creation

## Updates

### 2026-05-02T06:46:00Z — Mid-soak data point (12 days before formal audit fire)

**T-1426 (deprecation print on legacy primitives):** Status unchanged from prior sessions — implementation reviewed in commit history but no telemetry yet on production usage frequency. Refer to T-1432 fleet doctor `--legacy-usage` output once cron data accumulates.

**T-1427 (whoami + identity binding):** Live at 0.9.1693+ on .107 + .122. Strict-reject `-32014` on sender_id/pubkey mismatch enforced. Verified during T-1438 chat-arc rollout — both `framework-agent` (.107) and `ring20-management-agent` (.122) carry `identity_fingerprint` in remote_list metadata.

**T-1438 chat-arc soak — early indicator (6 days post-bus-launch):**
- `agent-chat-arc` topic: 54 posts, 2 senders.
  - `d1993c2c3ec44c94` (.107 framework-agent): 46 posts.
  - `9219671e28054458` (.122 ring20-management-agent): 2 posts.
- Topic description carries full T-1429.5/T-1430 invariant block (5 invariants).
- 1 read receipt: .107 acked through offset 37.
- 123 dm:* topics on .107 hub (heavy fleet activity); 26 contain self-fp; 23/26 unread (typical async DM backlog).

**Cut-readiness signal:** All three blockers from T-1438 fields ("Field-readiness matrix") are operator-gated, not protocol gaps:
1. .143 auth heal (T-1418)
2. .141 binary swap to 0.9.1702
3. .141 PATH wiring + identity registration

The T-1166 cut depends on (1) — auth heal must land before legacy event.broadcast can be retired without orphaning .143's chat-arc signal.

**Recommendation for the formal 2026-05-14 audit:** Compare these numbers against the same fields. If sender count is still 2 of 4 hosts (unchanged), .141 + .143 are still cold and the cut should be deferred. If sender count climbs to 3-4, the protocol has soaked successfully and T-1166 cut is safe.

### 2026-05-02T22:39Z — Pre-audit binary-version probe (T+11d before fire)

Direct binary-version probe per host (post-compact, autonomous re-verification of the 21:11Z evidence in T-1438):

| Host | Hub binary | T-1426 (>=0.9.1638) | T-1427 (>=0.9.1688) | Notes |
|---|---|---|---|---|
| .107 (workstation-107) | 0.9.1771 (this build, post-handover) | PASS | PASS | local — newest |
| .122 (ring20-management) | 0.9.1702 | PASS | PASS | confirmed via `remote exec ring20-management tl-vtvvv2tj 'termlink --version'` |
| .141 (laptop-141) | 0.9.1702 STAGED at `/tmp/termlink-staged-0.9.1702` (not on PATH for dimitri user) | STAGED | STAGED | confirmed via `remote exec laptop-141 tl-gibzucwp '/tmp/termlink-staged-0.9.1702 --version'`; user PATH=/usr/local/bin:/usr/bin:/bin lacks the binary; hub process is on a newer build (channel.* works) but agent CLI is stale |
| .121 (ring20-dashboard) | **0.9.844** | FAIL | FAIL | confirmed via `remote exec ring20-dashboard ring20-dashboard 'termlink --version'`; predates T-1155 channel API entirely |

**Chat-arc participation matrix (also re-verified):**

| Hub | Posts | Senders | Notes |
|---|---|---|---|
| .107 | 106→107 | 2 (d1993c2c=93, 9219671e=2) | latest receipt at offset 104 from .107 |
| .122 | 25→25 | 2 (9219671e=12, d1993c2c=12) | mostly cross-host sync |
| .141 | 21→21 | 2 (d1993c2c=12, 6604a2af=8) | own-fp 6604 posting via cron heartbeat |
| .121 | N/A | N/A | -32001 channel.post protocol mismatch |

Multicast post-compact: 3/4 OK + 1/4 SKIPPED-LEGACY (.121) — unchanged from earlier.

**Pre-audit verdict (provisional, T+11d):** Without operator action between now and 2026-05-14, the formal audit will rule **DEFER** — `.121` cannot post (binary swap T-1418 needed) and `.141` user-CLI is on PATH-gap (sudo symlink or `~/.local/bin` install needed). Sender count would remain 3 (d199 .107, 9219 .122, 6604 .141) — meets `>=3` AC threshold but `.121 = 0.9.844` fails AC1 + AC2 hard.

If both operator gates clear before 2026-05-14 → 4 senders, all hubs PASS, cut goes GO.
If neither clears → DEFER + extend soak.
If one clears → split decision; depends which.

### 2026-05-02T22:55Z — T-1427 strict-reject spot-check + RUNNING-vs-DISK binary divergence

Per AC2 spot-check ("forged --sender-id imposter post → expect -32014 CHANNEL_IDENTITY_MISMATCH"):

| Hub | Forged-sender result | T-1427 enforced? |
|---|---|---|
| .122 (ring20-management, on-disk 0.9.1702) | `-32014: sender_id="0000..." does not match identity fingerprint d1993c2c… (T-1427)` | ✓ ENFORCED |
| .141 (laptop-141) | accepted at offset 24 | ✗ NOT enforced — hub binary predates T-1427 (commit 0c0b3bfc on 2026-05-01T21:39Z) |
| .107 (workstation, on-disk `/root/.cargo/bin/termlink` = 0.9.1701) | accepted at offset 110 | ✗ NOT enforced — see "running-vs-disk divergence" below |
| .121 (ring20-dashboard) | -32001 channel.post unsupported | N/A (cut blocker is binary swap, not T-1427) |

**Root cause for .107 NOT enforcing despite on-disk 0.9.1701:**
- Hub PID 103255 started 2026-05-01 13:17:25 (running binary = whatever was on disk at that time)
- T-1427 commit landed 2026-05-01 21:39 (8h after the hub started)
- Disk binary `/root/.cargo/bin/termlink` rebuilt 2026-05-01 23:21:07 (post-T-1427, includes the strict-reject)
- But the kernel keeps the in-memory binary loaded for the running PID — disk replacement does NOT propagate to a running process
- **Result:** .107 hub runs PRE-T-1427 code in-memory until next process restart

**Implication for the 2026-05-14 audit:**
- Hub-binary check must compare RUNNING (`/proc/<pid>/status` start time + binary at that time) vs DISK
- If disk has post-T-1427 build but hub PID predates the commit, AC2 fails at runtime regardless of disk state
- A clean `termlink hub restart` on .107 would activate T-1427 enforcement immediately

**Pollution cleanup:** Forged posts (offset 110 on .107, offset 24 on .141) redacted via `msg_type=redaction` (offsets 111 + 25). `channel info` Senders count still shows `0000000000000000  (1 posts)` because redaction is a signal record, not a hard delete — audit-day Sender filter must exclude redacted-target offsets when counting sender contributions.

**Pre-audit recommendation update:** Restarting .107 hub before 2026-05-14 is no-blast-radius for T-1427 enforcement (T-1294 persist-if-present means clients don't re-pin) but high-blast-radius for sessions (37 active). Operator decision. Without restart, .107 audit verdict is "binary at-disk PASSES, runtime FAILS" — a partial PASS at best.

**.141 hub restart needed too** — same root cause likely (hub PID predates the staged 0.9.1702 binary swap that hasn't been performed). Confirms that .141 binary swap (T-1438 field-readiness item 2) is needed for both T-1426 deprecation print AND T-1427 strict-reject runtime activation.
