# T-1993 — Spike: `channel.subscribe` wedge is host-environment, not hub-version

**Date:** 2026-06-06
**Owner:** claude (agent), human (review)
**Status:** Inception — recommendation: GO on env-spike follow-up; close T-1993 with disproven bisect-target.
**Source incident:** T-1991 (`docs/reports/T-1991-channel-info-hub-concurrency-regression.md`)

## TL;DR

T-1991 framed the `channel info agent-presence` wedge as a "0.11.473 hub-
side concurrency regression vs 0.11.472." This spike disproves that
framing on three independent axes (code, binary version, host repro)
and proposes the next investigation: host-environment factors on the
flaky .121/.122/.141 LXC containers.

## Re-repro (current state, 2026-06-06T01:21Z)

Same CLI binary (`termlink 0.11.472`, on .107). Same command.
Only the `--hub` address differs.

| Hub | Topic | Envelopes | Trials | Result | Avg time |
|---|---|---|---|---|---|
| 192.168.10.107:9100 | agent-presence | 13441 | 5 | **5/5 PASS** | 0.80s |
| 192.168.10.122:9100 | agent-presence | 1503 | 5 | **5/5 TIMEOUT** | 16.005s |

The clean hub holds 9× more envelopes than the flaky hub. **Topic-size
is NOT the dominant axis.** The hub host is.

## Hypothesis matrix

| Hypothesis | T-1991 frame | Verified? | Evidence |
|---|---|---|---|
| Version regression (472→473) | YES | **DISPROVEN** | Bus crate: 0 commits since v0.11.1; hub crate: only T-1415 deletions (`f7b8d057`, `85117320`) |
| Topic-size amplification | partial | confirmed *within* a host (T-1991 inline) — NOT *across* hosts (this spike) | .107/13441 fast; .122/1503 wedged |
| Concurrent-writer lock contention | implied | unlikely as root cause | `bus.subscribe` drops the SQLite mutex BEFORE iterator walks (bus/lib.rs:199-215) |
| Host environment | not considered | **highly likely** (NEW) | .107 fast / .122 wedged with identical client+command — no other axis remains |

## Why the version-string axis collapsed

`crates/termlink-cli/build.rs:75`:

```rust
let major_minor = base.rsplitn(2, '.').last()?;
Some(format!("{major_minor}.{commits}"))
```

For `v0.11.0-N-gXXX` → "0.11.N". For `v0.11.1-N-gXXX` → also "0.11.N".

The two tags collapse into the same reported string. So `0.11.472`
could mean "472 commits past v0.11.0" OR "472 commits past v0.11.1"
depending on whether v0.11.1 was reachable when the binary was built.
The version number does not uniquely identify the source commit.

This is independently a framework bug worth a follow-up
(file: `T-1995-build.rs-version-string-disambiguate-v0.11.0-vs-v0.11.1.md`
if no instance already filed).

## What the spike does NOT prove

- WHICH environment factor on .122 causes the wedge (FD, memory, disk
  I/O, kernel scheduling, etc.).
- WHETHER .121 and .141 share the same root cause as .122 (they're
  also LXC; could be different per host).
- WHETHER the wedge is permanent or load-dependent (a `strace` during
  a wedge would reveal which syscall is stalled).

The next task (env-spike on .122 — TBD T-1994) takes these on.

## Next-task design (env-spike, deferred to T-1993 decide-time follow-up)

**Spike 2 — Hub PRAGMA + filesystem.** `pgrep -af 'termlink hub'` on
.122, then read SQLite PRAGMAs from the live `meta.db`:

```
sqlite3 /var/lib/termlink/meta.db \
  ".pragma journal_mode" ".pragma busy_timeout" ".pragma cache_size"
```

Compare to .107's same PRAGMAs. Look for `journal_mode=DELETE`
(blocking writes-during-reads vs WAL's concurrency-friendly mode).

**Spike 3 — `strace` during wedge.** While a `channel info` is in
flight on .122, `strace -p $(pgrep -f 'termlink hub') -e trace=read,futex,epoll_wait -tt -T -o /tmp/wedge.strace`.
The stalled syscall identifies the root cause: `read()` = disk; `futex()` =
lock; `epoll_wait()` = event-loop stall.

**Spike 4 — Read-only repro.** Stop the .122 presence-heartbeat cron
(or test against a topic without an active writer). Re-run 5 sequential.
If still 5/5 wedge: not write-vs-read contention; disk/scheduling. If
clean: write-vs-read interaction (consistent with `bus.subscribe`
holding the SQLite mutex during write-path operations elsewhere, even
though it doesn't during read).

**Spike 5 — Hub resource snapshot.** On .122: `cat
/sys/fs/cgroup/memory.current /sys/fs/cgroup/memory.max`,
`ulimit -n -a` of the hub PID, `iostat -x 1 5`, `vmstat 1 5`. Compare
the running env to .107. Look for memory pressure, FD exhaustion, I/O
wait.

Time-box: 1–2 hours; observational only; no hub restart needed.

## Operator action

Once T-1993 is decided GO with this disproven-version + env-spike
direction, file the follow-up build/decision task (working title
**T-1994 — env-spike on .122 channel.subscribe wedge**) and queue it.
Do NOT continue the bisect path; it has no target.
