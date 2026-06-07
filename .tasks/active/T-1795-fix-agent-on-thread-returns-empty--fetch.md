---
id: T-1795
name: "Fix agent on-thread returns empty — fetch_topic_msgs reads oldest page when slice_size exceeds hub 1000 cap"
description: >
  Fix agent on-thread returns empty — fetch_topic_msgs reads oldest page when slice_size exceeds hub 1000 cap

status: work-completed
workflow_type: build
owner: human
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-05-22T06:52:27Z
last_update: 2026-06-06T16:35:23Z
date_finished: 2026-05-22T07:06:08Z
---

# T-1795: Fix agent on-thread returns empty — fetch_topic_msgs reads oldest page when slice_size exceeds hub 1000 cap

## Context

Found during the T-1166 review-evidence sweep: `agent on-thread <thread>` returns "(no posts found)" even when posts with that thread tag exist, while `agent timeline --thread <thread>` returns them. Both use the same `extract_recent_posts` matcher — the divergence is in the fetch layer.

## Acceptance Criteria

### Agent
- [x] `fetch_topic_msgs` returns the MOST-RECENT page even when `slice_size` exceeds the hub's per-page cap (1000) — i.e. cursor is computed against the effective (capped) slice, so the returned window lands at the tail, not the oldest 1000
- [x] `agent on-thread <thread>` returns the same thread's posts that `agent timeline --thread <thread>` returns (for a thread whose posts are in the most-recent 1000 envelopes) — live-verified: on-thread T-1438 0→50 posts, matches timeline's 50
- [x] Regression test added in `crates/termlink-cli/src/commands/channel.rs` covering: slice_size > page-cap must produce a tail-anchored cursor (not 0 when count > cap) — 3 `fetch_topic_tail_cursor_*` tests, all pass
- [x] `cargo test -p termlink --bins fetch_topic_tail_cursor` passes (3/3) — note: was `--lib` but termlink has no lib target; full-suite run blocked by unrelated T-1798 overflow, so filtered run used
- [x] `cargo build -p termlink` succeeds

### Human
- [ ] [REVIEW] Live confirm the fix on a populated hub
  **Steps:**
  1. Build/install the fixed binary, then run: `termlink agent on-thread T-1438 --window-secs 604800`
  2. Compare with: `termlink agent timeline --thread T-1438 --window-secs 604800`
  **Expected:** Both surface the same T-1438 posts (non-empty); on-thread no longer prints "(no posts found)"
  **If not:** Re-check that the installed binary is the rebuilt one (`termlink --version` / build timestamp), and that the thread actually has posts in the window

## Verification

cargo build -p termlink 2>&1 | tail -3
cargo test -p termlink --bins fetch_topic_tail_cursor 2>&1 | grep -q "test result: ok. 3 passed"

## RCA

**Symptom:** `agent on-thread <thread>` prints "(no posts found on thread=...)"
even when the thread has posts, while `agent timeline --thread <thread>`
returns them. Verified live: on-thread T-1438 = 0 posts; timeline --thread
T-1438 = 50 posts (194 such posts exist in a 7-day window). Both echo
`filter_thread=T-1438` and both call the SAME `extract_recent_posts` matcher.

**Root cause:** the divergence is in the fetch layer, not the matcher.
`cmd_agent_on_thread` fetches via `fetch_recent_chat_arc_msgs(hub, 2000)`
("wider walk since thread logs are denser"); `cmd_agent_timeline` uses 1000.
`fetch_topic_msgs` computes `cursor = count.saturating_sub(slice_size)` and
sends `channel.subscribe {cursor, limit: slice_size}`. The hub caps `limit`
at `.min(1000)` (`crates/termlink-hub/src/channel.rs:553`). With count=1821:
- timeline (slice 1000): cursor = 1821-1000 = 821, limit 1000 → returns
  offsets [821,1821) — the tail, includes recent T-1438 posts. ✓
- on-thread (slice 2000): cursor = max(0, 1821-2000) = 0, limit capped to
  1000 → returns offsets [0,1000) — the OLDEST 1000, excludes the recent
  T-1438 heartbeats (~offset 1714+). ✗
So any caller passing slice_size > 1000 silently reads the oldest page
instead of the most-recent. on-thread is the only verb doing so; its
"wider walk" was never honored by the hub — it was a latent bug.

**Why structurally allowed:** `fetch_topic_msgs`'s contract ("fetch the last
slice_size envelopes") silently breaks when slice_size exceeds the hub's
page cap — there was no clamp tying the cursor math to the actual returnable
page size, and no test exercising slice_size > 1000. on-thread looked
correct in code review (same matcher as timeline) so the fetch-depth
asymmetry went unnoticed until a live populated arc grew past 1000 envelopes.

**Prevention:** (1) clamp the effective slice to the hub page cap inside
`fetch_topic_msgs` so cursor always lands at the tail; (2) regression test
asserting cursor is tail-anchored (not 0) when count > cap and slice > cap.
Related learning to record: any single-page fetch against a capped RPC must
clamp its cursor math to the cap, or it reads the wrong window.

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

## Recommendation

**Recommendation:** GO

**Rationale:** Root cause identified and fixed at the shared fetch layer
(`fetch_topic_msgs` clamps the effective slice to the hub's 1000-per-page
cap so the cursor stays tail-anchored). The fix is scoped, reversible, and
corrects four verbs that all shared the bug (on-thread, overview, presence,
presence --by-project). Live-verified on a populated hub; regression tests
added and passing.

**Evidence:**
- Live: `on-thread T-1438` 0 → 50 posts, matches `timeline --thread T-1438`
  (50); `agent presence` empty → 1 active peer (rebuilt `target/debug/termlink`)
- Tests: 3 `fetch_topic_tail_cursor_*` tests pass (`cargo test -p termlink
  --bins fetch_topic_tail_cursor` → 3 passed)
- Build: `cargo build -p termlink` succeeds; completion gate Verification 2/2
- RCA in this task file; root cause = single-page fetch vs hub `limit.min(1000)`
  at `crates/termlink-hub/src/channel.rs:553`
- Follow-up T-1796 (deeper-history pagination) parked; the deeper-walk intent
  behind the original 2000 slices is tracked there

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

### 2026-06-06T15:39Z — Human AC fresh re-smoke for [REVIEW] click [agent autonomous]

Per `[Fresh re-smoke before rubber-stamp]` memory: task is ~15 days old. Ran the AC verbatim against T-1438 (vendored-arc heartbeat thread with many posts):

```
$ termlink agent on-thread T-1438 --window-secs 604800
# agent on-thread T-1438 | window=604800s | n=50
[2d ago] @2628 peer=d1993c2c3ec4 msg_type=chat project=010-termlink
    T-1438 vendored-arc heartbeat from dimitrimintdev ...
[2d ago] @2631 peer=d1993c2c3ec4 ...
[1d ago] @2634 peer=d1993c2c3ec4 ...
...

$ termlink agent timeline --thread T-1438 --window-secs 604800
# agent timeline | window=604800s | n=50 thread=T-1438
[2d ago] [d1993c2c] @2628 msg_type=chat thread=T-1438 ...
[2d ago] [d1993c2c] @2631 ...
[1d ago] [d1993c2c] @2634 ...
...
```

**Both surfaces return the same envelopes** (@2628, @2631, @2634 ... matched across both views). on-thread no longer prints "(no posts found)" — the fetch_topic_msgs bug is verifiably fixed. Box ready to tick.

### 2026-05-22T06:52:27Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1795-fix-agent-on-thread-returns-empty--fetch.md
- **Context:** Initial task creation

## Reviewer Verdict (v1.4)

- **Scan ID:** R-0d9caeb6
- **Timestamp:** 2026-05-22T07:06:34Z
- **Catalogue:** v1.3-seed
- **Overall:** CONCERN
- **Needs Human:** no
- **Findings:** 1

**Per-AC findings:**

- **AC#3 (Agent)** — Regression test added in `crates/termlink-cli/src/commands/channel.rs` covering: slice_size > page-cap must produce a tail-anchored cursor (not 0 when count > cap) — 3 `fetch_topic_tail_cursor_*` test
  - **AC-verify-mismatch** (narrow, heuristic) — `path=crates/termlink-cli/src/commands/channel.rs in: Regression test added in `crates/termlink-cli/src/commands/channel.rs` covering: slice_size > page-cap must produce a tail-anchored cursor (not 0 when`

### 2026-05-22T07:06:08Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
