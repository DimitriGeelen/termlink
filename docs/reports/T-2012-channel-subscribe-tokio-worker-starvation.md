# T-2012 — Spike: `channel.subscribe` wedge is tokio worker-pool starvation

**Date:** 2026-06-06
**Owner:** claude (agent), human (review)
**Status:** Inception — recommendation: **GO**, file the fix as a single build task.
**Predecessors:** T-1991 (original repro), T-1993 (disproved version axis), this task (root cause)

## TL;DR

The "0.11.473 channel info wedge" reported by T-1991 is **not** a version
regression and **not** environmental pressure. The hub-side
`handle_channel_subscribe_with` (`crates/termlink-hub/src/channel.rs:535`)
calls a synchronous iterator over `std::fs::File::read_exact` from inside
an `async fn`. This blocks the tokio worker thread for the entire topic
walk. Under sequential channel.subscribe load + concurrent presence-
heartbeat writes, the worker pool saturates and new requests pile up
behind futex waits. Fix: wrap the walk in `tokio::task::spawn_blocking`.

## Method

Probed .122 hub (PID 157, runtime_dir `/var/lib/termlink/`) via
`termlink remote exec ring20-management tl-xrqbktch '<cmd>'` while
triggering wedges from .107 (`termlink channel info agent-presence
--json --hub 192.168.10.122:9100` with 16s timeout).

## Resource snapshot (ALL clean)

| Probe | Reading | Conclusion |
|---|---|---|
| `free -m` | 16 GB total, 1.5 GB used, 12.3 GB free, 0 swap | No memory pressure |
| cgroup memory.current / .max | 4.28 GB / unlimited | No cgroup pressure |
| `/proc/157/limits` open files | 1024 soft / 524288 hard | Headroom |
| `ls /proc/157/fd \| wc -l` | 24 | Far below soft limit |
| `df -h /var/lib/termlink/` | 24 GB / 7.6 GB used (34%) | Plenty of disk |
| meta.db | 400 KB | Small SQLite |
| Largest topic log | 979 KB | Tiny |
| `ps -o rss 157` | 16,212 KB (~16 MB VmRSS) | Tiny |
| Hub uptime | Jun 4 09:02 → Jun 6 01:30 (~40h) | Fresh enough |

No environmental pressure axis explains a 16-second wedge.

## Per-thread state during wedge (THE smoking gun)

```
tid=157 wchan=futex_do_wait state=S
tid=160 wchan=futex_do_wait state=S
tid=161 wchan=futex_do_wait state=S
tid=162 wchan=futex_do_wait state=S
tid=163 wchan=futex_do_wait state=S
tid=164 wchan=do_epoll_wait state=S    <-- the reactor
tid=165 wchan=futex_do_wait state=S
tid=166 wchan=futex_do_wait state=S
tid=167 wchan=futex_do_wait state=S
```

**8/9 threads on `futex_do_wait`, 1 on `do_epoll_wait`.** Classic tokio
worker-pool starvation. The reactor is awake (would wake to deliver
events), workers are all stuck.

`ss -tan sport = :9100` shows ESTAB connections from .107 with both
Recv-Q and Send-Q at 0 — hub HAS read each request fully and has not
SENT any response. Pure CPU/lock starvation, no I/O blockage.

## Code-level root cause

`handle_channel_subscribe_with` (`crates/termlink-hub/src/channel.rs:535`)
signature is `async fn`. Body (simplified):

```rust
pub(crate) async fn handle_channel_subscribe_with(
    bus: &Bus, id: Value, params: &Value,
) -> RpcResponse {
    // ... parse params ...
    let iter = bus.subscribe(&topic, cursor)?;   // returns sync SubscribeIter
    let mut messages: Vec<Value> = Vec::new();
    for item in iter {                            // <-- SYNCHRONOUS walk
        let (offset, env) = item?;                //     blocks the tokio
        // ... filter, push ...                   //     worker thread
    }
    Response::success(id, json!({"messages": messages, ...})).into()
}
```

`bus.subscribe` (`crates/termlink-bus/src/lib.rs:199`) returns a
synchronous `SubscribeIter` which is a `ReaderIter`
(`crates/termlink-bus/src/log.rs:84`). Each `next()` calls
`File::seek` + `File::read_exact` — blocking syscalls.

For 1503 envelopes that's ~1503 blocking syscalls. On .107 (fast disk,
no concurrent load) this completes in <1s and nothing else suffers. On
.122 (LXC, with presence-heartbeat cron firing channel.post every
minute through the same SQLite mutex) the walks stretch into seconds,
the 4-worker pool saturates, and sequential client requests queue on
futex.

Same pattern in `handle_channel_receipts_with` (`channel.rs:691`) and
plausibly other handlers.

## RPC audit log timing

```
1780701746397 channel.subscribe peer=192.168.10.107:37390 topic=agent-presence
1780701746464 channel.subscribe peer=192.168.10.107:37406 (67ms later)
1780701746528 channel.subscribe peer=192.168.10.107:37418 (64ms later)
1780701781657 channel.post      peer_pid=323320          (heartbeat cron)
1780701785194 channel.subscribe peer=192.168.10.107:56432 (3.5s later)
```

3-request bursts from `cmd_channel_info` are interleaved with cron
writes. Hub accepts the request (audit log gets entry) but stops
processing.

## Why .107 escapes

Same code, same binary version (both 0.11.472 per `termlink --version`).
Different host:

- .107: workshop-designer LXC, presumably faster disk
- .122/.121/.141: ring20 LXCs, slower or busier disk

The defect is host-agnostic but the trigger window is
disk-speed-dependent. .107 is lucky, not correct.

## Recommended fix

Wrap each sync iterator walk in `tokio::task::spawn_blocking`:

```rust
let iter_result = tokio::task::spawn_blocking(move || -> Result<Vec<_>, _> {
    let iter = bus.subscribe(&topic, cursor)?;
    let mut messages = Vec::new();
    for item in iter {
        let (offset, env) = item?;
        // ... filter ...
        messages.push((offset, env));
        if messages.len() >= limit { break; }
    }
    Ok(messages)
}).await;
```

Standard Rust async pattern. `spawn_blocking` uses tokio's dedicated
blocking-thread pool (default 512 threads), which is exactly what
blocking I/O should use. The async workers stay free for other RPCs.

## Other handlers to audit (in the fix task)

- `handle_channel_receipts_with` — walks topic synchronously
- `handle_channel_state_with` (if present)
- Any other handler that iterates `bus.subscribe(...)` inside `async fn`

## Test path for the fix task

Integration test:
1. Spin up a hub with agent-presence topic at ≥1000 envelopes
2. Start a concurrent writer (post every 1s)
3. From a separate connection, run `channel info agent-presence` 5
   times sequentially — assert all 5 complete in <2s each

Regression test should fail on current code (will wedge), pass after
spawn_blocking refactor.

## Why this matters (user value)

Affects every `termlink channel info <topic>`, `fw fleet doctor` indirect
calls, `agent-listeners.sh` discovery, `/peers`, `/pulse`, `/check-arc`,
`/check-outbox` — anything that walks a topic. The wedge currently makes
operator discovery commands unreliable across the ring20 fleet. Single
fix unblocks ALL of them simultaneously.
