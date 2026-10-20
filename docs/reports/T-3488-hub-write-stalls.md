# T-3488 — Hub channel.create / channel.post stalls (read-only investigation)

Status: investigation only. No source edited, nothing restarted. Sources: `/var/lib/termlink/rpc-audit.jsonl{,.1}`,
`journalctl -u termlink-hub`, sysstat (`sar -d -q -f /var/log/sysstat/sa20`), `/proc`, code in `crates/termlink-{hub,bus}`.
Window analysed: 2026-10-19 22:10Z to 2026-10-20 ~11:10Z.

## 1. Symptom, with numbers

- Over the window: 2608 `channel.post`, 2586 `channel.create`. Slow (handle_ms >= 1 s): 91 posts (3.5%), 79 creates (3.1%). Worst: `channel.create` 68 897 ms from 192.168.10.132 (finished 10:22:35Z on the 20th).
- The slow ones are not evenly spread. Slow create+post per UTC hour: 19th 22Z=61, 23Z=43; 20th 00Z=1, 01Z=13, 02Z=3, 03Z=2, 05Z=1, 08Z=16, 09Z=10, 10Z=18, 11Z=2.
- 179 slow non-subscribe lines in total. By peer: .132 = 105, .122 = 8, .121 = 8, local 127.0.0.1 = 2, rest are unix-socket callers with no peer_addr. .132 is simply the heaviest remote poster; normal rows carry no duration, so I cannot compute a per-peer slow rate.
- Slow requests overlap each other: 179 events fall into 104 overlap clusters; the biggest (7 events, 10:20:32 to 10:21:05Z) mixes .132, .121, .122 and local callers, and includes a `channel.set_retention` (9 s) and a `channel.sweep` (5 s) — both meta *writes*.
- `write_ms` is 0 on these lines, so the time is inside `process_request_message`, not in the TLS response write.
- Time of day: slow starts peak in minutes :20–:24 (35 events vs ~11 baseline per 5-min bucket) and :40–:44 (23). A weaker recurrence near second :45 (head-start claim 2) is not visible in the data: slow ends are spread over all seconds (no :45 spike).

## 2. Write path (what a channel.post / channel.create waits on)

`channel.post` (`crates/termlink-hub/src/channel.rs:829` `handle_channel_post_with_peer`):
1. Ed25519 verify, identity check, dedupe LRU (`crate::dedupe`, in-memory) — microseconds, no I/O (channel.rs:~880–1030).
2. `bus.post` (`crates/termlink-bus/src/lib.rs:391`):
   a. `meta.topic_exists` — read-only connection `rconn` (meta.rs:204, `with_read` meta.rs:51). Does not queue behind commits (T-3473).
   b. `appender_for(topic)`; `appender.append` (`log.rs:~60`): takes a **per-topic tokio Mutex held across the write + `file.sync_data()`** (log.rs:~62–76). **fsync #1.**
   c. `meta.record_append` (meta.rs:218): takes the **single global write-connection `std::sync::Mutex`** (`with_conn_mut`, meta.rs:97), one transaction (SELECT, UPDATE offsets, INSERT records). `synchronous=FULL`, WAL, so the commit fsyncs the WAL. **fsync #2.**
   d. `meta.note_write` (meta.rs:996): a **second, separate commit** on the same mutex (autocommit UPSERT into `topic_activity`). **fsync #3.** Best effort, but it is on the request path and not batched with (c).
   e. `maybe_apply_ceiling` every Nth post (limits, lib.rs) — more write transactions occasionally.
3. After the post: `cv_index::record`, `presence`, aggregator inject, webhook fan-out — all in-memory / non-blocking.

`channel.create` (`channel.rs:506` `handle_channel_create_as`): `topic_retention` (read), then `create_topic_owned`; `create_topic` (meta.rs:110) does a SELECT then **two autocommit INSERTs (topics, offsets) = two fsyncs** on the write mutex, then usually a `topic_metadata` post (which is the full post path above, 3 more fsyncs). So a *new-topic create* is roughly 5+ serialized fsyncs; an idempotent re-create still takes the write mutex (`with_conn`) for its SELECT.

Every durable step runs in `run_blocking` = `tokio::task::block_in_place` (lib.rs:~80), i.e. a tokio worker is handed off while the lock+fsync happen. Observed: 85 tokio-rt-worker threads (24 cores), nearly all parked on futexes right now.

Other writers sharing the same write connection/mutex (anything that commits waits behind a slow fsync): `note_fetch` on `channel.subscribe` (channel.rs:1378; empty polls throttled to once per 60 s per (topic, reader) via `fetch_throttle`, data fetches always persisted), sweeps/retention, claims, set_retention, receipts/acks.

Nothing on the path is O(topics) or O(records): `topic_exists` and `record_append` are PK lookups; no topic lint scan, no retention scan on post (retention sweeps are explicit and hourly, under 1 s per head-start claim 4 — consistent with the audit, which shows only one `channel.sweep` slow line, 5 s, in the window).

Load placed on that mutex (estimate from the audit log): about 119 persisted `note_fetch` commits/min from empty polls alone (simulated throttle, ~18 k distinct (topic, reader) pairs per hour because one-shot CLI callers each look new), plus data-fetch commits, plus ~3.4 posts/min x 3 fsyncs and ~3.3 creates/min x 2–5 fsyncs. That is roughly 3–4 commits/s sustained: trivial at 5 ms per fsync, a queue as soon as one fsync takes seconds.

## 3. Evidence about the host (this is the decisive part)

- **Disk saturation, from sysstat** (`nvme0n1`, btrfs root with zstd, `discard=async`, hub dir on the same fs): 22:00Z to 23:50Z (00:00–01:50 CEST) every 10-min sample shows 58–92 MB/s writes, average queue depth 59–106, await 17–23 ms, %util 61–84. From 00:00Z (02:00 CEST) the same disk is at 4 MB/s, queue < 4, util 2–6%. The hourly counts of slow create/post follow this: 61 and 43 in the two saturated hours, 1 in the quiet hour after.
- **Load / blocked tasks**: `sar -q` at 22:40Z and 22:50Z shows ldavg-1 56 and 55, `blocked` 22 and 33 (tasks in D state). Live now: loadavg ~10–20, 24 cores, PSI io some avg60 3.4%, full avg60 2.1%; cpu PSI some avg300 8%.
- **Journal lag, measured**: over 31 097 hub tracing lines since 22:00Z on the 19th, 320 reached journald > 2 s after their embedded UTC timestamp, max 64.6 s (20:40:05Z on the 19th), also 42.6 s at 22:40:10Z. The hub's stdout is a pipe to journald; both journald (which fsyncs to the same btrfs) and the hub tracing writer wait on the same disk. This confirms head-start claim 3 for the 22:40 event: the hub is not the only process stalled — the whole box was waiting on IO. It is a *symptom*, not the cause: only 19 of the 170 slow create/post requests overlap a >3 s journal-lag interval (those intervals cover about 1% of the time, so there is an ~11x enrichment, but 89% of stalls happen with no visible journal lag).
- **What writes**: not attributable after the fact (per-process `/proc/*/io` are lifetime counters, the top ones are init/cron/tmux/claude/docker). Candidates present on this host: several agentic projects each running `fw index reindex` (or a project audit) at **minute :20** every hour (5 `index reindex` lines at minute 20 in `/etc/cron.d`, plus a project-audit line for 832-Workflow-designer at :20), a `:40` hourly job in a Yellowtwig project, `timeshift --check` at :00 (btrfs), dockerd, many concurrent claude sessions writing transcripts. The :20–:24 and :40–:44 peaks in slow-request start times line up with the :20 and :40 jobs.
- **Disproved / not the cause**: the T-3383 outage collector (every minute, shows no minute-45 signature); rpc-audit writing (moved to a writer thread in T-3475; the 87 MB live file is not on the request path); retention sweep (1 slow line); signature verify / dedupe / cv_index (all CPU or memory, microseconds).

## 4. Ranked hypotheses

**H1 (most likely): host-wide btrfs/NVMe write saturation makes each fsync slow, and the hub amplifies it because every post/create is 3–5 *serialized* fsyncs behind one global SQLite write mutex plus one per-topic file mutex.**
- For: the sar series above, the correlation with hourly counts, the 40–60 s journald lag during the same minutes (journald is a plain fsync-ing writer on the same fs), blocked-task counts of 22–33, slow set_retention and sweep (pure meta writes) in the same clusters, mixed peers in clusters (local callers stall too), `write_ms=0`, nothing else on the path can take tens of seconds. A single fsync that takes 10 s blocks everything queued behind the mutex, so the worst waits (68 s) are queue sums of several slow fsyncs, each of which `note_fetch`, `note_write` and `create_topic`'s second INSERT lengthens.
- Against: not every stall is during a saturated sample; 11:2Z and 08Z/10Z hours show 16–18 slow lines while the sar rows I read for 12:00–13:10 CEST show moderate util (20–55%, queue 11–68). Burst-y fsync outliers can hide inside a 10-min average, so this is weak counter-evidence, but I did not have 1-second disk data.
- Also against (head-start claim 1): a *fast* local `channel.create` (10:22:04Z) and `channel.post` (10:22:34Z) completed inside the 68 s window of the .132 create (10:21:26–10:22:35Z). If the write mutex had been held the whole time those would have waited. Mutex hold times are therefore bursty (many seconds, then free), which fits an fsync with a multi-second tail, not a continuously held lock. Both runs are consistent with H1; the sample is two requests.

**H2: write-mutex convoy / amplification from the many small commits (secondary, makes H1 worse).** `note_fetch` (every data fetch + each new (topic,reader) per minute), `note_write` as a separate commit, and the two-INSERT `create_topic` all take the same mutex. They do not cause a 60 s stall alone (3–4 commits/s) but turn a 2 s fsync tail into a queue of dozens of waiters. For: audit-derived commit rate; 7-event cluster including set_retention and sweep. Against: no direct measurement of mutex wait.

**H3: the :20 / :40 hourly job pile-up is the *trigger* of H1 (cron-driven IO bursts).** For: slow starts peak at :20–:24 (35 vs ~11) and :40–:44 (23). Against: the saturated sar hours stay saturated across all ten-minute marks (60–84% util every 10 min, not just :20), so there is also a continuous heavy writer in 22:00–23:50Z that I could not identify; something not tied to cron.

**H4: btrfs-specific behaviour (fsync forcing a transaction commit; `discard=async`; zstd; snapshots from `timeshift --check` at :00).** Plausible multiplier — btrfs `sync_data`/WAL fsync on a busy fs can wait for a whole transaction commit — but no direct evidence (no btrfs sysfs / `/proc/<pid>/stack` samples from a stall; `timeshift --check` only checks, and :00 is not a peak: 10 events).

**H5 (rejected): lock held in hub code / O(topics) work / sweep / signature / dedupe / audit writer / cv_index / webhook.** No such work on the path; the stalls vanish when the disk is idle with the same code and the same traffic.

## 5. Most likely root cause

Each `channel.post` costs three serialized durable commits (payload `sync_data`, `record_append`, `note_write`) and a new-topic `channel.create` costs five or more, all behind one global SQLite write-connection mutex. On this host the NVMe/btrfs root is periodically saturated by other workloads (60–90 MB/s writes, queue depth 60–100, load 56), so individual fsyncs take seconds; the mutex then queues every writer behind the slow one, giving 1–69 s `handle_ms`. T-3461's FULL sync (correct, keep it) removed the safety of "fast because unsynced" and T-3473's read connection fixed readers only; the writers still have no isolation from host IO.

## 6. Proposed fix and verification

Fix, in order of value per risk (do not weaken `synchronous=FULL`; accepted must still mean stored):
1. **One transaction per post.** Fold `note_write` into the `record_append` transaction (same mutex acquisition, one commit): removes fsync #3 from the request path with no durability loss, since activity is telemetry, and a post then costs 2 fsyncs. In `create_topic`, wrap the two INSERTs in one transaction (halves create).
2. **Make `note_fetch` / activity telemetry non-durable.** Keep `topic_activity` / `topic_readers` updates in memory and flush on a timer (say every 30–60 s) in a single batch, or put them in a second SQLite file with `synchronous=OFF`. Subscribe traffic (~120+ commits/min, more with data fetches) stops competing with posts for the write mutex. This is the single largest reduction in contention.
3. **Group commit for posts.** Batch the write-mutex section so concurrent posts share one commit/fsync (sqlite WAL + a small batching task), amortising slow fsyncs across waiters; the per-topic payload `sync_data` can stay as is, or share an fsync across topics only if the log files are co-located.
4. **Bound the stall, don't hide it.** Add a mutex-wait vs fsync-time split to the T-3478 slow line (wait for `conn` lock, time inside commit, time in `sync_data`) so the next occurrence names which fsync was slow instead of one `handle_ms` number. Cheap, and it turns H1 vs H2 from inference into measurement.
5. **Operational (outside the hub code):** stagger the hourly `fw index reindex` / audit crons that all fire at :20 and the :40 job; put the hub's `/var/lib/termlink` on its own device or at least a different IO priority (`IOSchedulingClass=` / `IOWeight=` in the systemd unit) than the agent workloads; find the continuous 60–90 MB/s writer in the 22:00–23:50Z window (`iotop -b` / `pidstat -d 1` during the next hour of saturation) — that is the real offender.

How to verify:
- Reproduce with the existing test seam `termlink_bus::set_test_slow_durable_ms(N)` (lib.rs:~97): with N = 2000, N concurrent `post`s should complete in about 2 x N rather than 3 x N after fix 1, and in about N after fix 2 for subscribe-heavy load.
- On the host: run `pidstat -d 1` + `iostat -x 1` during an :20 window, and `cat /proc/<hub>/task/*/wchan` / `stack` during a stall to confirm threads sit in `btrfs_sync_file` / `futex` on the meta mutex.
- Before/after: the count of slow create/post per hour in `rpc-audit.jsonl` under the same traffic (baseline: 61 and 43 in the two bad hours of 19th 22Z–23Z, 0–18 otherwise), and max handle_ms (baseline 68 897 ms). Success: no create/post over 5 s with the disk at < 90% util, and the hourly slow count in a saturated hour at least 5x lower.

## 7. Confidence

Disk saturation as the trigger: high (about 85%) — it is directly measured and temporally matched. The specific split between "one global write mutex amplifying" and "kernel/btrfs fsync tail" is medium (about 60%): the audit proves mixed-peer clusters and the code proves the serialization, but there is no per-lock wait measurement, and two fast requests inside a 68 s stall show the mutex is not held continuously. Fix 4 (instrumentation) resolves that cheaply.

## 8. Corrections to the head-start notes

1. "Local creates were fast in the middle of remote stalls so the mutex was not held continuously": confirmed for one window (2 fast ops inside a 68 s stall), but clusters elsewhere mix local and remote stalls, so it is bursty hold, not proof of no lock involvement.
2. "Hub-wide dip every 60 s near :45": not supported by the audit data (no :45 spike in slow-end seconds); the real periodicity is hourly (:20–:24, :40–:44).
3. "35 s journal lag 22:40:18Z to 22:40:53Z": confirmed and generalised: 320 of 31 097 lines lagged > 2 s, max 64.6 s, evidence the whole box was IO-blocked. PSI now reads low; the historical record is sar.
4. "Retention sweeps ruled out": confirmed (only one slow `channel.sweep`, 5 s, and the hourly sweep at :19:36 is outside the :20–:24 peak by design).

## Daytime stalls (2026-10-20 08–10:30Z)

Read-only follow-up. Times: audit `ts` is UTC ms; sar and `journalctl` (without `--utc`) print local CEST (UTC+2), so sar `10:20` = 08:20Z. Sources: `/var/lib/termlink/rpc-audit.jsonl{.1,}`, `journalctl -u termlink-hub`, `sar -d` on `sa20`, `/etc/cron.d`, `find -newermt` on `/opt/termlink/target` (TZ=UTC).

### Numbers

1. **The slow set is mostly subscribes, not writes.** In 08:00–10:30Z there are 554 `slow:true` lines: 511 `channel.subscribe`, 20 `channel.post`, 19 `channel.create`, 1 `set_retention`, 1 `sweep`, 2 other. None of the subscribe lines carries `requested_wait_ms`, so these are snapshot reads (no `timeout_ms`), not long-polls. Matched back to their normal lines (287 of 554), 54 are `inbox:*` subscribes from the two `termlink sidecar start` processes (pid 3956131 `claude-termlink-alt`, pid 3287399 `claude-termlink`), and the rest are spread over many `inbox:*`, `dm:*` and `sidecar:*` topics. The 267 unmatched lines were not attributed to a topic.
2. **Slow per 10-minute UTC bin** (count): 08:10 73, 08:20 66, 08:30 1, 08:40 15, 08:50 42, 09:00 43, 09:20 62, 09:50 66, 10:20 **122** (the worst bin: 5 creates, 7 posts, 110 subscribes; worst single request 68 897 ms create at 10:22:35, 46 365 ms post at 10:22:42).
3. **Disk, sar (UTC), nvme0n1.** Quiet from 06:10 to 07:50Z (write 2–14 MB/s, util < 5%, queue < 1). Busy from 08:00 to 10:50Z without a break: write 15–58 MB/s, util 16–56%, avg queue 7–58, await 4–24 ms. Examples: 08:20 wkB 51.5 MB/s, aqu 58, util 56; 10:20 wkB 26.8 MB/s, aqu 14; 10:30 wkB 39.3 MB/s, aqu 47. **So the premise "the disk was quiet" does not hold for this window; it was quiet only before 08:00Z**, and the slow reads start only after that.
4. **Cargo write volume** (`find /opt/termlink/target -newermt`, bytes by mtime minute, UTC): 08:10 bin 1.8 GB (1 230 files), 08:20 bin 3.9 GB (1 987), 08:50 bin 1.2 GB (2 979), 09:20 bin 8.0 GB (5 681), 09:40 bin 2.3 GB, 09:50 bin 0.7 GB. Test binaries written 08:18–08:21Z (`t3473_reads_do_not_wait_on_commit`, `parity`, `termlink_mcp`, `termlink_bus`, `cli_integration`). A 93 MB and a 40 MB binary written 09:51–09:53Z (link output of `local-fast/termlink`). **The 10:20 bin has no target writes at all** (none in 10:00–10:50Z); a > 5 MB file scan of `/opt/termlink`, `/var/lib/termlink`, `/var/log`, `/tmp` for 10:10–10:32Z found nothing. A wider scan of `/var/lib /root /opt` timed out, so that bin's writer is not identified.
5. **Hub restarts on 0.12.775.** Stop/start at 07:56:19Z, 08:54:00Z, 09:49:35Z. The 09:49:34Z binary install (`/usr/local/bin/termlink`, mtime 11:49 CEST) is the 0.12.775 deploy. Slow lines in ±3 min of each restart: 0, 2, 0. **No restart-induced cluster.** The one slow burst after the 09:49 deploy (09:53:23–09:58:39Z: two creates at 19 348 and 19 346 ms, posts up to 7 128 ms) lines up with the 09:51–09:53Z link writes in item 4, not with the restart.
6. **Journal lag of hub lines.** 6 070 tracing lines in 08:00–10:30Z; 10 lagged > 2 s, max 21.7 s (08:59:13Z) and 12.4 s (08:59:22Z). Only about 0.2% of lines lagged, so journald was not the bottleneck in the 10:20 bin.
7. **Does an idempotent create take the write path?** In the working tree (uncommitted T-3488 edit to `crates/termlink-bus/src/meta.rs`), `create_topic` runs its SELECT inside `with_conn_mut`, which takes the global write mutex. For a topic that already exists it then returns `Ok(false)` and drops the transaction with no insert. So an existing-topic create waits on the mutex but does not fsync. I did not read `create_topic_owned` (`lib.rs:180`) or the post-create `topic_metadata` path, so I cannot say whether an existing-topic create also re-posts metadata.

### Peers and topics

- Creates and posts come from mixed peers: `.132` (the heaviest remote poster), `.121`, `.122`, `.107`, and unix-socket callers. The worst create (68.9 s) is from `.132`. Nothing singles out one peer for the creates.
- Subscribes are unix-socket (`peer_addr` absent) and concentrated on `inbox:*` topics of the two sidecars. These subscribes persist fetch telemetry through `note_subscribe_fetch` → `bus.note_fetch` (throttled to once per 60 s per (topic, reader), `channel.rs:~1378`). That commit goes through the same global write mutex, so the subscribe waits the same fsync queue as posts.

### Most likely cause

Host-wide write load on the same NVMe/btrfs root (not a restart, not one peer, not one topic) makes each durable commit slow. The hub serialises those commits behind one write mutex (post: payload `sync_data` + `record_append` + `note_write`; subscribe: `note_fetch` once a minute per reader). Waiting on that queue turns a multi-second fsync into 1–69 s request latency for every caller that needs the mutex, including snapshot subscribes that never write data to the caller. The clearest case is 08:10–08:29Z: test/link writes of 1.8–3.9 GB per 10 minutes coincide with 139 slow lines. The 10:20 cluster (122 lines) has no target writes, so its writer is still unidentified; the `:20` hourly `fw index reindex` and project-audit jobs are the candidates in the cron files, but I could not confirm that any of them wrote during that bin.

### Confidence

- Disk-busy window as the environment for all of 08:00–10:50Z: **high (~90%)**. sar and the audit agree; the timezone mapping is checked against the 08:18–08:21Z test-binary writes landing in the sar row labelled 10:20.
- Serialised commits (mutex plus fsync) as the amplifier: **medium (~65%)**, carried over from the first section; the subscribe result supports it, since reads should not wait on disk if the mutex were not in the path.
- Cargo as the main writer in 08:10–09:55Z: **medium (~60%)**; four of six bins match, the 10:20 bin does not.
- The 10:20 cluster's writer: **unknown**.

### What would settle it

- `iostat -x 1` plus `pidstat -d 1` during the next 10:20-style `:20` window, and `/proc/<hub-pid>/task/*/wchan` sampled during a stall (confirms `btrfs_sync_file` vs `futex` on the write mutex).
- Add the wait-vs-fsync split to the T-3478 slow line (already proposed in section 6, fix 4). It turns "which fsync" from inference into a measurement.
- Rule out the sidecar snapshot subscribes as write load: check `note_subscribe_fetch` throttling in the 08–10Z window (count of persisted fetches per minute).
