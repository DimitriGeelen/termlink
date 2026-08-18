---
id: T-2507
name: "wait_for_peer_ack reads count-anchored slice, returning false 'delivery unconfirmed'
  on swept dm topics"
description: "The synchronous ack-wait behind `agent contact --ack-required` reads
  its tail via fetch_topic_msgs, whose tail_slice_cursor(count,slice)=count-slice
  treats channel.list count as the max offset. After a retention sweep front-trims
  a Messages(1000) dm topic, count decouples from the tail offset, the hub returns
  the OLDEST live page, and the peer's just-posted ack at the tail is missed → false
  Ok(None) 'delivery unconfirmed'. Fix: poll via offset-cursor pagination (walk_topic_from),
  correct under any sweep. Sibling of T-2390/T-2391 (which fixed the same count-vs-offset
  decoupling on the presence read)."
status: work-completed
workflow_type: build
horizon:
owner: agent
created: 2026-08-03
last_update: '2026-08-18T18:59:12Z'
tags: [reliability, correctness, delivery-confirmation, ack, retention-sweep, 
      count-vs-offset]
components: [crates/termlink-cli/src/commands/channel.rs]
bvp_scores_proposed:
  - ts: '2026-08-18T18:56:49Z'
    estimator: bvp-estimator-v1-heuristic
    scores:
      D1: 1
      D2: 0
      D3: 0
      D4: 0
      F-RECALL: 0
      F-ORCH: 0
    rationale: D1=1 (body:fix-without-learning); D2=0 (no-signal); D3=0 
      (no-signal); D4=0 (no-signal); F-RECALL=0 (no-signal); F-ORCH=0 
      (no-signal)
    rubric_sha: missing
cost_estimate_proposed:
  - ts: '2026-08-18T18:59:12Z'
    estimator: bvp-estimator-v1-heuristic
    cost_estimate:
      blast_radius: 1
      tier: 2
      effort: 6
    rationale: blast_radius=1 (no-signal); tier=2 (no-signal); effort=6 
      (no-signal)
    rubric_sha: missing
---

## Context

Correctness class (found by the 20th adversarial sweep). Charter path: exchange
durable messages WITH delivery confirmation — `agent contact --ack-required`
(agent.rs:1663) calls `wait_for_peer_ack` (T-1485) to synchronously confirm the peer
received the message. This is the exact "why is there still no response?" surface the
substrate reliability arc targets.

## RCA

- **Symptom:** `agent contact --ack-required` reports the peer did NOT ack (exit 10),
  even though the peer posted an ack, whenever the dm topic has been swept.
- **Root cause:** `wait_for_peer_ack` polls with `fetch_topic_msgs(topic, hub, 200)`,
  which computes its cursor via `tail_slice_cursor(count, slice) = count - slice`,
  treating the `channel.list` record COUNT as the max OFFSET. These are equal only
  while the topic is never front-trimmed.
- **Trigger:** every `dm:*` topic is created `Retention::Messages(1000)`
  (`is_high_rate_pattern` matches `dm:`). Sweep is explicit-only (T-1155, no background
  thread), but the T-2252 topic-growth canary + retention-reset runbook encourage
  running `channel sweep` on high-rate topics. Once a busy dm thread (>1000 posts) is
  swept, `count` caps at 1000 while `max_offset` keeps rising — the decoupling is
  permanent and worsens with each new post. `tail_slice_cursor(1000, 200) = 800`;
  the hub advances the below-window cursor 800 to the oldest live offset (e.g. 4000)
  and returns offsets 4000..4199 — the OLDEST 200, not the newest. The peer's ack
  (offset ~4999, ts=now) is not in that slice, so `detect_ack_in_msgs` (which requires
  `ts > send_ts_ms`) returns None → false `Ok(None)`.
- **Why the framework was blind:** T-2390/T-2391 fixed this exact count-vs-offset
  decoupling but ONLY routed the agent-presence read through the cv_index
  (`fetch_topic_current_values`). The dm ack path is an independent caller of the
  count-anchored `fetch_topic_msgs` and was never swept. The ack-detection logic
  (`detect_ack_in_msgs`) is pure + correct + tested; the bug is entirely in WHICH
  records the poll feeds it.
- **Fix class:** Level C — replace the count-anchored read with offset-cursor
  pagination (`walk_topic_from`), which is correct under any retention sweep. Poll
  incrementally (carry next_cursor across polls) so only the first poll reads the full
  live topic and subsequent polls read only new records — no per-poll cost regression.
- **Learning tie-in:** direct application of PL-292 (a fix in one authoritative read
  does not propagate to sibling callers of the same count-anchored helper).

## Scope note (one-bug-one-task)

The count-vs-offset root cause has other, LOWER-severity callers
(`check_peer_online_via_chat_arc` via `fetch_recent_chat_arc_msgs`), but presence is
documented best-effort with a tolerated false-negative window (channel.rs comment
~1855) and has the authoritative cv_index path. The ack path has NO such tolerance —
it claims to CONFIRM delivery — so it is the genuine charter-critical wrong-answer and
the single deliverable here. Presence siblings are a separate follow-up if
false-negatives are observed.

## Acceptance Criteria

### Agent
- [x] `wait_for_peer_ack` no longer calls the count-anchored `fetch_topic_msgs`
- [x] A `walk_topic_from(sock, topic, start_cursor) -> (Vec<Value>, u64)` offset-cursor helper is added and used by the ack-wait poll loop
- [x] The poll loop carries next_cursor across iterations (incremental — no full re-read per poll)
- [x] Existing `detect_ack_in_msgs` pure tests still pass (ack-detection logic unchanged)
- [x] `cargo test -p termlink --bin termlink` passes

## Verification

# CLI tests pass
out=$(cargo test -p termlink --bin termlink 2>&1); echo "$out" | tail -3; echo "$out" | grep -q "test result: ok"
# The ack-wait path no longer CALLS the count-anchored fetch_topic_msgs (comment mentions are fine)
! awk '/async fn wait_for_peer_ack/,/^}/' crates/termlink-cli/src/commands/channel.rs | grep -q 'fetch_topic_msgs('
# The ack-wait path uses the offset-cursor walk
awk '/async fn wait_for_peer_ack/,/^}/' crates/termlink-cli/src/commands/channel.rs | grep -q 'walk_topic_from(&sock'

## Decisions

Offset-cursor pagination rather than adding a max_offset field to the channel.list
protocol response — the latter is a wire/protocol change (out of authority). The
client can already reach the true tail via cursor pagination (the hub advances a
below-window cursor to the oldest live offset), so no protocol change is needed. The
fix is an I/O-boundary swap; its revert-guard is mechanical (P-011 greps that the
count-anchored read is gone and the offset walk is present), since the pure
ack-detection logic it feeds was already correct and unit-tested.

## Reviewer Verdict (v1.5)

- **Scan ID:** R-965f5f7c
- **Timestamp:** 2026-08-02T23:06:14Z
- **Catalogue:** v1.3-seed
- **Overall:** CONCERN
- **Needs Human:** no
- **Findings:** 2

**Verification-level findings:**

  1. **l387-sigpipe-risk** (partial, heuristic) @ Verification:line 4
     - evidence: `! awk '/async fn wait_for_peer_ack/,/^}/' crates/termlink-cli/src/commands/channel.rs | grep -q 'fetch_topic_msgs('`
  2. **l387-sigpipe-risk** (partial, heuristic) @ Verification:line 6
     - evidence: `awk '/async fn wait_for_peer_ack/,/^}/' crates/termlink-cli/src/commands/channel.rs | grep -q 'walk_topic_from(&sock'`

### 2026-08-02T23:05:17Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
