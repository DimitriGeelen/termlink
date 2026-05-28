---
id: T-1844
name: "agent-listeners.sh subscribe-from-tail — fix cursor=0 reading OLDEST not NEWEST envelopes (livelisten count always 0 on busy topics)"
description: >
  agent-listeners.sh subscribe-from-tail — fix cursor=0 reading OLDEST not NEWEST envelopes (livelisten count always 0 on busy topics)

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: [bug]
components: []
related_tasks: []
created: 2026-05-28T17:39:33Z
last_update: 2026-05-28T17:39:33Z
date_finished: null
---

# T-1844: agent-listeners.sh subscribe-from-tail — fix cursor=0 reading OLDEST not NEWEST envelopes (livelisten count always 0 on busy topics)

## Context

T-1833 `agent-listeners.sh` calls `termlink channel subscribe agent-presence
--limit 200` which reads from `--cursor 0` (the OLDEST 200 envelopes). On busy
hubs where agent-presence has accumulated more than `--limit` envelopes
(observed: 474 on local hub), recent heartbeats are NEVER scanned and
`total_listeners=0` is reported even when live heartbeats are actively
arriving. The same bug propagates to T-1837 fleet variant and any verb that
composes them — including the in-progress T-1843 adoption snapshot.

**Symptom (live, today):** `~/.termlink/be-reachable.state` shows pid 3897788
heartbeating every 30s as `root-claude-dimitrimintdev`. `channel subscribe
agent-presence --limit 200 --json | grep -c root-claude-dimitrimintdev` →
102 hits (envelopes exist). But `agent-listeners.sh --json` →
`total_listeners=0`. Cause: the 200 scanned envelopes are offsets 0..199;
my heartbeats are at offsets ≥ 372 (post count 474).

**Fix:** seek to tail before scanning. Call `channel info --json` to get
post count, then `channel subscribe --cursor max(0, count - SCAN_DEPTH)`.

## Acceptance Criteria

### Agent
- [ ] `agent-listeners.sh` seeks to tail before scanning — uses `channel info --json` to get topic post count, then `channel subscribe --cursor <count - SCAN_DEPTH>` where SCAN_DEPTH defaults to 200 (existing `--limit` flag preserved as scan-depth knob)
- [ ] Backwards-compatible: when topic count <= SCAN_DEPTH, behaves as before (cursor=0)
- [ ] G-060 graceful: `channel info` returning -32013 / unknown topic → exit 0 with empty rollup (same convention as T-1842 for subscribe)
- [ ] Live verification: with `/be-reachable` heartbeating as `root-claude-dimitrimintdev` and agent-presence having ≥200 envelopes on the local hub, `agent-listeners.sh --json | jq '.live'` returns ≥1
- [ ] Existing test suite still passes (`bash scripts/test-agent-listeners.sh` — 10/10 from T-1842)
- [ ] One new test covers the seek-to-tail path: post >SCAN_DEPTH dummy heartbeats from one agent + a fresh heartbeat from a second agent; the second agent surfaces as LIVE

## Verification

test -x scripts/agent-listeners.sh
bash scripts/agent-listeners.sh --help >/dev/null
bash scripts/test-agent-listeners.sh
bash scripts/agent-listeners.sh --json | jq -e '.listeners | map(select(.agent_id == "root-claude-dimitrimintdev")) | length >= 1' >/dev/null

## RCA

**Symptom:** `agent-listeners.sh` reports `total_listeners=0` on hubs where
agent-presence has more than `--limit` envelopes accumulated, even when live
heartbeats are actively arriving. Propagates to T-1837 fleet variant and
T-1843 adoption snapshot.

**Root cause:** `termlink channel subscribe --cursor 0 --limit N` returns
the OLDEST N envelopes (offsets 0..N-1). When `topic.post_count > N`, the
most-recent heartbeats sit at offsets ≥ N and are never scanned. The TTL
classification logic (LIVE/STALE/OFFLINE based on age vs interval) then
sees only ancient envelopes and emits zero LIVE.

**Why structurally allowed:** No test in `test-agent-listeners.sh` exercised
a topic with `post_count > --limit`. T-1833's tests all create fresh
ephemeral topics with ≤2 heartbeats. The bug only manifests after the
topic accumulates traffic — exactly what happens in production once
adoption starts.

**Prevention:** New test posts >SCAN_DEPTH heartbeats from one agent +
a single fresh heartbeat from a second, then asserts the second agent
surfaces. Catches the regression on every CI run.

## Evolution

<!-- REQUIRED for arc-tagged build tasks (tags include arc:*). Captures how
     understanding evolved during build — what was learned that wasn't known at
     filing, what in the original plan no longer fits, what triggered pivots
     or new sub-tasks. Mandatory at slice boundaries (when applicable) and
     before --status work-completed.

     Origin: T-1717 grill Q4 — "the understanding of what we need and want
     evolves with the process of materialisation." Structural counter to §ACD:
     spec-vs-build divergence is logged as soon as it happens, not lost as
     folklore.

     Format (one entry per slice boundary or significant insight):
       ### YYYY-MM-DD — [topic]
       - **What changed:** [what we learned that we didn't know at filing]
       - **Plan impact:** [what in the plan no longer fits]
       - **Triggered:** [new sub-task / pivot / scope cut, with task ID if filed]

     The completion gate (T-1718) blocks --status work-completed when this
     section exists but is empty/template-only. Use --skip-evolution to bypass
     (logged Tier-2). Non-arc tasks may leave this empty.
-->

## Decisions

<!-- Record decisions ONLY when choosing between alternatives.
     Skip for tasks with no meaningful choices.
     Format:
     ### [date] — [topic]
     - **Chose:** [what was decided]
     - **Why:** [rationale]
     - **Rejected:** [alternatives and why not]
-->

## Decision

<!-- Filled at completion of inception tasks via:
     fw inception decide T-XXX go|no-go|defer --rationale "..."

     For non-inception tasks this section is ignored. Kept in template
     so `fw inception decide` (lib/inception.sh) finds the anchor heading
     without auto-creating; T-1832 added auto-create as fallback for
     legacy tasks lacking this section. -->

## Updates

### 2026-05-28T17:39:33Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1844-agent-listenerssh-subscribe-from-tail--f.md
- **Context:** Initial task creation
