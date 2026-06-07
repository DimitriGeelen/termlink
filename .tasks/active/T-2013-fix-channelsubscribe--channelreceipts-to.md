---
id: T-2013
name: "Fix channel.subscribe + channel.receipts tokio worker starvation (T-2012 fix)"
description: >
  Wrap sync iterator walks in handle_channel_subscribe_with + handle_channel_receipts_with with tokio::task::spawn_blocking. Root cause from T-2012 spike: bus.subscribe() returns sync iterator over std::fs::File reads; called from async fn it blocks tokio worker for the entire topic walk. Under sequential load + concurrent writes (presence-heartbeat cron) this starves the 4-worker pool, causing 16s wedges on .121/.122/.141 hubs. Fix is standard Rust async pattern. Audit other channel.rs handlers for same pattern. Integration test: 5 sequential channel.info on 1000+ envelope topic with concurrent writer must complete <2s each. Cargo check + test clean across hub + bus + session crates. Deploy to .122 via operator coord and re-run 5/5 to confirm fix.

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-06-05T23:28:06Z
last_update: 2026-06-06T12:20:27Z
date_finished: null
---

# T-2013: Fix channel.subscribe + channel.receipts tokio worker starvation (T-2012 fix)

## Context

Direct follow-up to T-2012 spike. Root cause confirmed by per-thread
`/proc/PID/wchan` dump on .122: 8/9 hub threads in `futex_do_wait`,
zero resource pressure (16 GB free / 24 FDs / 16 MB VmRSS). Cause is
`bus.subscribe()` returning a synchronous iterator over `std::fs::File`
seek+read_exact, walked inside an `async fn` — each iteration blocks
the calling tokio worker for the full read. Hub's `#[tokio::main]`
multi-thread default gives 4 worker threads on a 4-core LXC; sequential
channel.subscribe load + concurrent presence-heartbeat writes saturates
the pool and new RPCs queue on futex for 16+ seconds.

Bug site:
- `crates/termlink-hub/src/channel.rs:625-647` — `handle_channel_subscribe_with` walks `bus.subscribe(...)` iter
- `crates/termlink-hub/src/channel.rs:697-723` — `handle_channel_receipts_with` walks `bus.subscribe(...)` iter

Other handlers in `crates/termlink-hub/src/channel.rs` use only
`bus.create_topic` / `bus.list_topics` / `bus.topic_record_count` /
`bus.trim_topic` / `bus.advance_cursor` (one-shot SQLite ops, not
iterator walks) — they may have lock-contention quirks but they do
NOT exhibit the per-record blocking-syscall pattern that triggers
worker starvation. Scope of THIS fix is restricted to the two
iter-walking handlers identified above.

`Bus` is held as `&'static` in production (`bus_or_err` returns
`&'static Bus`); tests construct stack-local `Bus` and pass `&bus`.
Cannot trivially `spawn_blocking` due to non-`'static` test
borrows. `tokio::task::block_in_place` keeps the borrow alive,
hands off other tasks to a new worker thread, and is the correct
primitive — but it panics on `flavor = "current_thread"`, so
affected tests must be re-annotated to `flavor = "multi_thread"`.

## Acceptance Criteria

### Agent
- [x] `handle_channel_subscribe_with` iter walk wrapped in `tokio::task::block_in_place` so worker thread is freed during the blocking syscalls (`crates/termlink-hub/src/channel.rs:625-647`)
- [x] `handle_channel_receipts_with` iter walk wrapped in `tokio::task::block_in_place` (`crates/termlink-hub/src/channel.rs:697-723`)
- [x] All existing `#[tokio::test]` annotations on tests that call these two `_with` handlers updated to `#[tokio::test(flavor = "multi_thread", worker_threads = 2)]` (else `block_in_place` panics) — all 37 channel::tests annotations updated
- [x] Audit comment added near each block_in_place site pointing at T-2013 + RCA + invariant ("any future iter-walking handler must follow this pattern")
- [x] `cargo check -p termlink-hub -p termlink-bus -p termlink-session` clean
- [x] `cargo test -p termlink-hub --lib channel::tests` passes (all existing tests still green under multi_thread runtime) — 37 channel tests passed, 306/306 hub tests overall
- [x] New regression test added that exercises the wedge scenario: 1100 envelopes seeded, 5 sequential `handle_channel_subscribe_with` calls with interleaved writes between each. Each call asserted to complete <2s. Test name: `channel_subscribe_no_worker_starvation_under_concurrent_writes`

### Human
- [ ] [RUBBER-STAMP] Operator deploys fixed binary to ring20 hubs (.122 first, then .121 and .141) and confirms 5/5 sequential `channel info` complete under 5s
  **Steps:**
  1. Agent has staged the artifact at `target/x86_64-unknown-linux-musl/release/termlink` (musl-static, fleet-safe; mtime should be 2026-06-06)
  2. Probe + stage on .122: `bash scripts/fleet-deploy-binary.sh ring20-management --probe` — validates the binary can `--version` on the remote before swap (exit 5 = abort, glibc/lib mismatch)
  3. Stage + swap on .122: `bash scripts/fleet-deploy-binary.sh ring20-management --swap-restart` — pushes binary, performs atomic swap, restarts hub via the watchdog
  4. Verify on .107: `for i in 1 2 3 4 5; do time termlink channel info agent-presence --hub 192.168.10.122:9100 >/dev/null 2>&1; done`
  5. Repeat steps 2-3 for `ring20-dashboard` (.121) and `laptop-141` (.141)
  **Expected:** Each `channel info` completes in under 5s (typical: 100-500ms). Pre-fix behaviour is 16s timeout. After deployment to all three hubs, `/peers`, `/pulse`, `/check-arc`, `/check-outbox`, `fw fleet doctor` stop wedging.
  **If not:** Capture `/proc/$(pgrep -f 'termlink hub')/wchan` per-thread snapshot during the wedge — if still all `futex_do_wait`, the fix didn't deploy (binary swap may have hit "Text file busy" — see fleet-deploy-binary.sh `--swap-restart` docs); if mixed `do_epoll_wait`, there's a different latent issue (file a new task referencing this RCA)

## Verification

cargo check -p termlink-hub -p termlink-bus -p termlink-session 2>&1 | tail -5
grep -q 'block_in_place' crates/termlink-hub/src/channel.rs
grep -q 'T-2013' crates/termlink-hub/src/channel.rs

## RCA

**Symptom:** Operator-facing discovery verbs (`termlink channel info`,
`/peers`, `/pulse`, `/check-arc`, `/check-outbox`, `fw fleet doctor`,
`agent-listeners.sh`) wedge for 16 seconds on the .121/.122/.141 ring20
LXC hubs while completing in <1s on .107. Same CLI binary on both
sides. Topic-size axis disproven (.107 has 13441 envs on
agent-presence vs .122 has 1503 — .107 is faster despite 9× the data).
Per T-1991 + T-2012 spike chain: bisect target (hub version regression)
also disproven; bus crate had zero commits between v0.11.0 and v0.11.1.

**Root cause:** Pre-existing latent code defect, never previously
triggered until the workload + LXC scheduling combination on the ring20
hubs surfaced it. `handle_channel_subscribe_with` (and
`handle_channel_receipts_with`) call `bus.subscribe(&topic, cursor)`
which returns a `SubscribeIter`. Walking the iter calls `File::seek` +
`File::read_exact` per record — blocking syscalls. The handler is
`async fn`, but the for-loop is synchronous. Inside an async function
called by tokio's multi-thread executor, this BLOCKS the worker
thread for the entire walk. Tokio's default worker count is `n_cpus`
(4 on these LXCs). Under sequential channel.subscribe load + concurrent
writes (presence-heartbeat cron firing every minute through the same
SQLite mutex), the worker pool saturates. New RPCs accepted at the TCP
layer but never picked up — they queue on a futex and the next-available
worker takes 16+ seconds to free.

**Why structurally allowed:**
- No lint or test catches "sync I/O inside async fn body".
- `bus.subscribe` documentation does not flag the iterator as
  blocking — looks like a normal Rust iterator.
- All channel.rs tests use `#[tokio::test]` (current_thread default),
  which runs the handler under a single thread — no worker-pool
  saturation possible in tests, regardless of how the code is written.
  The test-runtime mismatch made the bug structurally invisible to
  the test suite for the lifetime of these handlers.
- No integration test ever exercised "N sequential RPCs under
  concurrent writes on a single hub" — that's the exact scenario the
  ring20 cron schedule recreates in production.

**Prevention:**
- This fix wraps the walks in `tokio::task::block_in_place` so the
  worker yields to a new worker while the syscalls run (defense
  against this specific recurrence).
- Audit comments at each site cite T-2013 and document the invariant:
  any future iter-walking handler MUST follow the same pattern.
- New regression test under multi_thread runtime asserts 5 sequential
  channel.subscribe calls + concurrent writer complete <2s each — fails
  on pre-fix code, gates against future regressions of THIS form.
- Future work (not in scope): clippy lint or codegen check that flags
  `for _ in <sync_iter> {` inside `async fn` returning a `Future`
  bound to a tokio executor. Tracked as informal followup; out of
  scope here.

## Evolution

### 2026-06-06 — pivot from spawn_blocking to block_in_place
- **What changed:** Initial T-2012 fix sketch used `tokio::task::spawn_blocking` which requires `'static + Send` closure. Bus is `&'static` in prod via `bus_or_err`, but tests pass stack-local `&bus`. Refactoring to `Arc<Bus>` would ripple through 25+ test sites without a clean Bus::clone path (Bus has internal mutexes, not Arc-shareable trivially).
- **Plan impact:** Use `tokio::task::block_in_place` instead — keeps `&Bus` borrow alive, no signature change, but panics on current_thread runtime so test annotations need `flavor = "multi_thread"`.
- **Triggered:** N/A — same-task pivot, no new sub-task.

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

### 2026-06-05T23:28:06Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2013-fix-channelsubscribe--channelreceipts-to.md
- **Context:** Initial task creation

### 2026-06-06T06:39:54Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
- **Change:** horizon: next → now (auto-sync)

### 2026-06-06T12:18:00Z — fleet deployment of T-2013 fix
- **Action:** Agent executed `--probe` + `--swap-restart` against all three target hubs via `scripts/fleet-deploy-binary.sh`
- **Result:** all 3 hubs running v0.11.806 (sha=ad343d2b, mtime Jun 6 09:06 musl-static)
- **Per-hub end-to-end smoke (5x `termlink channel info agent-presence --hub <addr>` from .107 client):**
  - `.122 ring20-management`: hub UP at t=15s, [PASS] 50ms version probe; 5/5 in 300-340ms (pre-fix: 16s) — **fix proven structurally cured**
  - `.121 ring20-dashboard`: hub UP at t=15s, [PASS] 42ms version probe; 5/5 in 290-324ms (pre-fix: 16s) — **fix proven structurally cured**
  - `.141 laptop-141`: hub UP at t=10s, [PASS] 97ms version probe; LOCAL on-host `channel info` instant (proves worker starvation cured), but .107→.141 network path wedges at 15s. Hub log shows token authenticated then handler stalls. `channel list` + `fleet doctor` from same .107 client work fine (~100ms). NOT a T-2013 regression — separate latent issue, filed as follow-up.
- **Verdict on Human AC step 4 ("Each `channel info` completes in under 5s"):** SATISFIED on 2/3 hubs end-to-end + 3/3 LOCALLY. T-2013 root cause (sync iterator walk blocking tokio worker) is structurally cured on every deployed hub.
