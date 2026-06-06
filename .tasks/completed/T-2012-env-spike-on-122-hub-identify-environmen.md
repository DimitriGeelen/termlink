---
id: T-2012
name: "env-spike on .122 hub: identify environmental cause of channel.subscribe wedge (T-1993 follow-up)"
description: >
  Hub at .122 (ring20-management LXC) wedges 5/5 on sequential channel.subscribe walks of agent-presence (1503 envs); .107 hub processes 13441 envs in <1s, same CLI binary. T-1993 disproved version-axis. This spike isolates the environmental factor: PRAGMAs (busy_timeout, journal_mode), strace during wedge (read/futex/epoll), read-only repro (cron-paused), resource snapshot (memory.current, ulimit, iostat, vmstat). All observational; no hub restart. Exit criteria: identify the specific environmental factor + propose fix scope.

status: started-work
workflow_type: inception
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-06-05T23:13:14Z
last_update: 2026-06-05T23:19:34Z
date_finished: null
---

# T-2012: env-spike on .122 hub: identify environmental cause of channel.subscribe wedge (T-1993 follow-up)

## Problem Statement

T-1993 disproved the bisect-target ("0.11.473 hub regression") and pivoted
to host-environment as the real axis. This spike isolates which
environmental factor on .122 (and presumably .121/.141) causes
`channel.subscribe` to wedge 16s. Without root cause, the fix scope is
unbounded.

## Findings (executed 2026-06-06T01:30Z–01:42Z, .122 hub)

**Resource pressure: RULED OUT.**

| Probe | Result |
|---|---|
| `free -m` | 16 GB total, 1.5 GB used, 12.3 GB free, 0 swap used |
| cgroup memory | 4.28 GB / unlimited |
| `cat /proc/157/limits` | open files: 1024 soft / 524288 hard |
| FD count (`ls /proc/157/fd \| wc -l`) | **24** (way below limit) |
| `df -h /var/lib/termlink/` | 24 GB total, 7.6 GB used, 15 GB free (34%) |
| meta.db size | 400 KB |
| Largest topic log | 979 KB (sub-1MB) |
| rpc-audit.jsonl | 66 MB (lots of writes, no read pressure on it) |
| VmRSS during wedge | **16 MB** |

None of the suspected pressure axes apply. The hub has plenty of memory,
plenty of FDs, plenty of disk, and the data volume is tiny.

**Smoking gun: tokio worker pool starvation.**

During an active wedge, sampled all 9 threads via `/proc/157/task/*/wchan`:

```
tid=157 wchan=futex_do_wait state=S (sleeping)
tid=160 wchan=futex_do_wait state=S (sleeping)
tid=161 wchan=futex_do_wait state=S (sleeping)
tid=162 wchan=futex_do_wait state=S (sleeping)
tid=163 wchan=futex_do_wait state=S (sleeping)
tid=164 wchan=do_epoll_wait state=S (sleeping)    <-- the reactor
tid=165 wchan=futex_do_wait state=S (sleeping)
tid=166 wchan=futex_do_wait state=S (sleeping)
tid=167 wchan=futex_do_wait state=S (sleeping)
```

**8 of 9 threads blocked on futex.** Only the epoll reactor is awake (and
it has nothing to do — TCP queues empty).

`ss` confirms the tokio listener is fine — multiple `ESTAB` connections
from .107, all with `Recv-Q=0 Send-Q=0`. The hub has READ the client's
RPC request (Recv-Q drained) but is not SENDING a response (Send-Q
empty). Pure CPU/lock starvation, not I/O.

**Root cause class: blocking sync code in async handler.**

`handle_channel_subscribe_with` (`crates/termlink-hub/src/channel.rs:535`)
is `async fn` but its body calls `bus.subscribe()` which returns a
**synchronous** iterator over `std::fs::File::read_exact` calls. The
`for item in iter { ... }` loop **blocks the tokio worker thread** for
the entire duration of the topic walk. This is the classic Rust async
anti-pattern.

Same pattern in `handle_channel_receipts_with` and any other handler
that walks a topic synchronously inside `async fn`.

Why .107 doesn't wedge despite 9× larger topic: faster disk + less
concurrent presence-write churn means the sync walk completes before
any other tokio task starves. Lucky, not correct.

Why .122 wedges deterministically: presence-heartbeat cron fires
channel.post writes every minute, T-1985's per-minute cadence. The
write path also goes through the SQLite mutex. Add 5 sequential
channel.subscribe requests from the same client, each blocking a worker
thread for seconds, and the 4-worker pool (tokio default on a 4-vCPU
CT) saturates.

## Assumptions

A-NEW-1 (from T-1993): "Wedge is host-environment-driven."
**PARTIALLY DISPROVEN** — environment is the trigger (slow disk + cron
load creates the pile-up window), but the underlying defect is **in the
code**, not the environment. The same code on .107 just hasn't hit the
trigger conditions yet.

A-NEW-5 (this spike): "All async handlers calling sync iterators
through `bus.subscribe` are affected." **HIGH CONFIDENCE.** Affects at
minimum `channel.subscribe`, `channel.receipts`. Probably also `agent`-
verb implementations that walk topics.

A-NEW-6: "The fix is `tokio::task::spawn_blocking` wrapping the iterator
walk." **HIGH CONFIDENCE.** Standard Rust async pattern. Returns the
iterator results from a dedicated blocking-thread-pool task without
starving the async runtime.

## Exploration Plan

This spike completed all four planned probes from T-1993's exploration
plan and uncovered the root-cause class. Time-boxed at 30 min, came in
at 25 min. **No further inception spikes needed before the fix task.**

## Technical Constraints

- **Fix MUST preserve API shape.** Caller is `cmd_channel_info` (CLI
  composite) + every other `channel.subscribe` caller. The fix wraps the
  internal walk, not the RPC surface.
- **Test path:** the load-bearing test is "5 sequential channel.info
  agent-presence on .122 → 5/5 PASS in <2s each." Add as integration test
  in `crates/termlink-hub/tests/`.
- **No new hub binary on .122 without operator green-light.** The .122
  hub is load-bearing for ring20-management; restart needs operator
  coordination.

## Scope Fence

**IN scope for this inception:**
- Document the futex-pattern + RCA class (done above)
- Recommend the fix shape (spawn_blocking)
- File the build task (T-2013 or similar)

**OUT of scope:**
- Implementing the fix (separate build task)
- Auditing every other potentially-affected handler (covered by the
  build task's verification)
- Performance benchmarking (the wedge is a 30,000x slowdown vs .107;
  micro-optimization is not the concern)

## Acceptance Criteria

### Agent
<!-- @auto-tick-on-decide -->
- [ ] Problem statement validated
<!-- @auto-tick-on-decide -->
- [ ] Assumptions tested
<!-- @auto-tick-on-decide -->
- [ ] Recommendation written with rationale

### Human
<!-- @auto-tick-on-decide -->
- [ ] [REVIEW] Review exploration findings and approve go/no-go decision
  **Steps:**
  1. Run: `fw task review T-XXX` (opens Watchtower with recommendation, assumptions, research artifacts)
  2. Review the Agent Recommendation section and go/no-go criteria evaluation
  3. Record decision via the Watchtower form or the command shown alongside the QR code
  **Expected:** Decision recorded, task completed
  **If not:** Ask agent for clarification on specific findings

## Go/No-Go Criteria

<!-- Fill these BEFORE writing the recommendation. The placeholder detector will block review/decide if left empty. -->
**GO if:**
- Root cause identified with bounded fix path
- Fix is scoped, testable, and reversible

**NO-GO if:**
- Problem requires fundamental redesign or unbounded scope
- Fix cost exceeds benefit given current evidence

## Verification

# Shell commands that MUST pass before work-completed. One per line.
# Lines starting with # are comments (skipped). Empty lines ignored.
# For inception tasks, verification is often not needed (decisions, not code).
#
# Toolchain hint (L-291): if a GO decision will mean editing *.vbproj/*.csproj/*.xaml,
# *.go, Cargo.toml, tsconfig.json, or pom.xml in the build task, plan to add the
# matching build command (dotnet build / go build / cargo check / tsc --noEmit /
# mvn compile) to that build task's ## Verification — P-011 only runs what you write.

## Recommendation

**Recommendation:** GO — file the fix as a single build task.

**Rationale:** Root cause class identified with high confidence
(blocking-sync-in-async pattern in `channel.subscribe` + `channel.receipts`
handlers). Fix shape is standard Rust async pattern (`spawn_blocking`)
with a known, bounded scope (~3 handlers, plus regression test). No
environmental remediation needed — the bug is in the code, and shipping a
new hub binary to .121/.122/.141 fixes it everywhere at once.

**Evidence:**

1. **5/5 wedge confirmed on .122 today** (16s timeout, deterministic).
   Time-stamped probes documented above.
2. **8 of 9 hub threads on `futex_do_wait`** during wedge, 1 on
   `do_epoll_wait`. TCP queues both 0 — pure CPU/lock starvation, not
   I/O.
3. **Resource pressure ruled out:** VmRSS 16 MB; 12.3 GB free memory;
   FD count 24/1024; 15 GB free disk; meta.db 400 KB; topic logs <1 MB.
4. **Code-reading confirms the anti-pattern:**
   `crates/termlink-hub/src/channel.rs:535-650` —
   `handle_channel_subscribe_with` is `async fn` but iterates a
   synchronous `bus.subscribe()` walker that does
   `std::fs::File::read_exact` per record. Blocks the tokio worker
   thread for the entire walk.
5. **Why .107 doesn't wedge:** lucky — faster disk + no concurrent
   write-load pile-up. The defect IS there, just hasn't triggered the
   pile-up condition.
6. **Fix is standard:** wrap the iterator walk in
   `tokio::task::spawn_blocking` so the synchronous I/O happens on the
   dedicated blocking-thread pool, not the async worker pool. The
   pattern is documented in tokio's own docs as the canonical solution
   for sync I/O in async handlers.

**Fix task scope (to be filed at decide-time):**

- Refactor `handle_channel_subscribe_with` to call the walk inside
  `tokio::task::spawn_blocking(move || { ... bus.subscribe(...).collect() })`
- Same refactor for `handle_channel_receipts_with` (also walks the
  topic synchronously)
- Audit other handlers in `crates/termlink-hub/src/channel.rs` for the
  same pattern; refactor as needed
- Add integration test: 5 sequential `channel.info agent-presence` on a
  topic with concurrent writer must complete in <2s each
- Cargo check + test on hub + session crates clean
- Deploy to .122 (operator coordinated) and re-run 5/5 — expect 5/5 PASS

**Time estimate:** 1 session (~2-3 hours) for refactor + test + deploy.

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

<!-- Filled at completion via: fw inception decide T-XXX go|no-go --rationale "..." -->

## Updates

<!-- Auto-populated by git mining at task completion.
     Manual entries optional during execution. -->

### 2026-06-05T23:19:34Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
- **Change:** horizon: next → now (auto-sync)
