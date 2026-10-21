# T-3488 — disk writers on .107 (read-only analysis of the sampler)

**Inputs:** `pidstat.txt` (`pidstat -d`, 10 s ticks), `iostat.txt` (`iostat -x`, 10 s, device `nvme0n1`).
**Window:** 780 ticks, 2026-10-20 23:50:00 → 2026-10-21 01:59:50 local (+0200) = 21:50Z → 23:59:50Z.
**Units:** "MB/s" here means MiB/s (pidstat/iostat kB divided by 1024). Averages are over all 780 ticks, so an idle process counts as zero.

**Caveat up front.** The per-process pidstat totals are about **3× the device's own write rate** (see table 3). The per-process peaks (up to 2.4 GB/s for `init`) are physically impossible for this NVMe device, whose maximum observed write rate is 354 MB/s. Treat pidstat's per-process numbers as **ranking evidence, not exact byte counts**.

---

## 1. Top 15 writers by mean pidstat kB_wr/s (per command name)

| # | Command (pidstat comm, first token) | Mean MB/s | Peak MB/s (1 tick) | Total GiB | Active ticks | Distinct PIDs |
|---|---|---:|---:|---:|---:|---:|
| 1 | `claude` (Claude Code sessions) | 11.52 | 1626.3 | 8.78 | 426 | 68 |
| 2 | `init` (PID 1, systemd) | 8.54 | 2366.5 | 6.51 | 699 | 1 |
| 3 | `tmux` (incl. `t537-canary sleep 86400`) | 5.55 | 1946.2 | 4.23 | 139 | 33 |
| 4 | `bash bats-exec-file` (bats test runs) | 4.85 | 165.2 | 3.69 | 184 | 126 |
| 5 | `git` | 4.07 | 117.2 | 3.10 | 191 | 282 |
| 6 | `headless_shell` (headless Chromium) | 4.02 | 55.4 | 3.06 | 162 | 369 |
| 7 | `bash -c` (agent shell-snapshot wrappers) | 3.63 | 1132.6 | 2.76 | 238 | 162 |
| 8 | `bash bats-exec-test` | 3.25 | 57.5 | 2.47 | 262 | 275 |
| 9 | `bash fw` (framework CLI) | 2.92 | 350.7 | 2.23 | 335 | 453 |
| 10 | `cron` (PID 1369; launches `fw index reindex` and others) | 2.60 | 350.7 | 1.98 | 201 | 1 |
| 11 | `java` | 2.57 | 17.8 | 1.96 | 181 | 126 |
| 12 | `python3 -c` (incl. `web.embeddings.reindex_incremental`) | 2.45 | 362.3 | 1.86 | 195 | 53 |
| 13 | `bash update-task.sh` | 2.22 | 1028.7 | 1.69 | 150 | 78 |
| 14 | `python3 volle_stufe.py` | 1.82 | 982.8 | 1.39 | 8 | 1 |
| 15 | `python3 -m` (pytest and others) | 1.64 | 162.3 | 1.25 | 385 | 76 |

Distinct-PID counts are high for the tool rows (`git`, `fw`, `bash bats-*`) because those are short-lived processes spawned repeatedly. I did not compute the share of total bytes that the top 15 cover.

## 2. Device view (iostat `nvme0n1`, all 780 ticks)

| Metric | Mean | Median | p90 | Max |
|---|---:|---:|---:|---:|
| wkB/s → MB/s written | 23.1 | 9.2 | 66.5 | 353.8 (01:20:10) |
| %util | 20.3 | 6.6 | 65.8 | 93.2 |
| w_await (ms) | 14.6 | — | 36.5 | 496.6 (00:09:10) |
| aqu-sz | — | — | — | up to 230 (01:45:30) |

- Ticks with %util > 50: 142 of 780 (18%). Ticks with %util > 80: 33.
- **Mean device write rate is 23 MB/s, not a sustained 60–90 MB/s.** The 60–90 MB/s figure matches the pidstat per-process sum (mean 67–74 MB/s), not the device. p90 of the device is 66 MB/s, so the device reaches 60–90 MB/s only in bursts.

## 3. Attribution sanity check: pidstat total vs device

| Quantity | Value |
|---|---|
| pidstat per-tick total, mean (all 780 ticks) | 73.9 MB/s (median 21.4) |
| iostat device write, mean (same ticks) | 23.1 MB/s |
| Ratio of sums, pidstat / device (732 aligned ticks) | 3.00 |
| Pearson r, per-tick pidstat total vs device wkB/s | 0.29 |

The low correlation (r = 0.29) and the 3× ratio mean pidstat's per-process write accounting does not track the block layer. The likely cause is the filesystem (btrfs, with `btrfs-transaction`, `btrfs-delalloc` and compressed-write kernel threads in the top list), where dirty-page accounting is attributed to whichever process dirtied the page. I did not verify that cause. The result is the same either way: **per-process write attribution here is approximate.**

## 4. Ten-minute blocks, local time (busy vs quiet)

| Block (local) | Ticks | pidstat total mean MB/s | Device mean MB/s | Device p90 MB/s | Top writers (mean MB/s in block) |
|---|---:|---:|---:|---:|---|
| 23:50 | 60 | 141.5 | 41.0 | 70.9 | tmux 36.6, claude 30.0, init 17.2, bats-exec-file 11.5 |
| 00:00 | 60 | 116.4 | 35.9 | 68.6 | bash -c 19.5, update-task.sh 17.2, bats-exec-file 11.8, claude 10.7 |
| 00:10 | 60 | 146.6 | 40.1 | 88.6 | tmux 25.2, claude 21.1, bash -c 17.6, bats-exec-file 16.6 |
| 00:20 | 60 | 56.2 | 15.2 | 35.9 | init 8.7, bash -c 8.1, update-task.sh 8.0, bash fw 7.5 |
| 00:30 | 60 | 37.2 | 14.4 | 18.1 | python3 -c 8.1, claude 7.2, cron 6.4, bash fw 5.8 |
| 00:40 | 60 | 40.4 | 8.0 | 15.2 | claude 27.1, init 4.6, java 2.6 |
| 00:50 | 60 | 8.8 | 5.5 | 9.3 | init 2.7, java 2.3, journald 1.1 |
| 01:00 | 60 | 12.0 | 4.1 | 8.0 | cron 4.0, init 3.0, java 2.9 |
| 01:10 | 60 | 15.3 | 8.2 | 10.9 | cron 4.5, python3 -c 4.4, init 3.0 |
| 01:20 | 60 | 20.3 | 10.4 | 7.6 | cron 6.6, python3 -c 6.5, init 3.0 |
| 01:30 | 60 | 116.8 | 50.3 | 99.0 | git 23.2, headless_shell 18.6, python3 -m 17.6, volle_stufe.py 16.8 |
| 01:40 | 60 | 88.8 | 29.1 | 67.2 | headless_shell 26.4, claude 22.2, bats-exec-file 6.2, cargo 4.0 |
| 01:50 | 60 | 139.5 | 37.7 | 66.2 | init 45.7, claude 27.9, cargo 13.2, volle_stufe.py 6.8 |

## 5. Busiest device intervals and who wrote in them

| Time (local) | Device wkB/s (MB/s) | %util | aqu-sz | Top pidstat writers (MB/s) |
|---|---:|---:|---:|---|
| 01:20:10 | 353.8 | 49.1 | 75.5 | python3 -c 362.3 (single tick); dockerd 3.5 |
| 01:17:10 | 276.2 | 10.8 | 4.5 | python3 -c 265.5 (`reindex_incremental`); btrfs-transaction 2.0 |
| 00:35:10 | 231.4 | 27.4 | 38.9 | python3 -c 244.6 (PID 2230868); dockerd 2.7 |
| 01:37:30 | 217.0 | 41.1 | 28.0 | python3 -m 160.6; bats-exec-file 74.3; headless_shell 33.8 |
| 00:17:30 | 198.8 | 38.5 | 36.9 | python3 -c 112.9; bash fw 45.4; init 17.5 |
| 00:39:10 | 134.2 | 53.9 | 36.1 | python3 -c 134.7; pg_dump 4.9 |
| 01:37:00 | 131.1 | 47.5 | 59.3 | bats-exec-test 55.1; headless_shell 50.0; init 17.6 |
| 01:38:00 | 125.6 | 65.9 | 84.0 | git 80.4; headless_shell 35.5; java 15.1 |
| 23:59:10 | 126.1 | 12.1 | 5.2 | bash fw 51.4; tmux 50.9; java 14.8 |
| 00:12:30 | 122.0 | 52.0 | 53.7 | bash fw 53.2; pre-commit 5.8 |

Two named bursts, both traced to the sampler's own pidstat rows:

- **`web.embeddings.reindex_incremental`** (`python3 -c`, PIDs 1364193, 3908546, 2230868 and others). At 01:17:10 these PIDs together write 272 MB/s, matching the device's 276 MB/s at that tick. At 00:35:10 PID 2230868 writes 245 MB/s against a device value of 231 MB/s. Separately, `fw index reindex` was launched at 00:21:20 by cron (PID 1369) and wrote 220 MB/s at that tick. These reindex jobs are the largest identifiable write source. I did not confirm the cron schedule for them.
- **Test runs and git** (bats, `git`, `fw`, `update-task.sh`, headless Chromium): these dominate the 01:30–01:50 block and the 00:00–00:20 block.

## 6. Steady or bursty

- **Bursty.** The top 5% of ticks hold **43%** of pidstat bytes, and the top 1% hold 21%.
- The load falls into **two active windows**: about 23:50–00:20 and 01:30–01:50. Between them, 00:40–01:30 is quiet (device mean 4–8 MB/s, except a ramp to 10 MB/s).
- Within the active windows, the device runs at 10–50 MB/s on average with bursts to 230–350 MB/s.

## 7. Does the termlink hub write meaningfully?

| Process | Mean kB/s | Peak kB/s | Notes |
|---|---:|---:|---|
| `termlink hub start --tcp 0.0.0.0:9100` (PID 3491244, live) | 27.1 | 132 | 21 MB over 2 h |
| All `termlink`-binary rows combined | 164 | 35,233 | Peak is one tick of `termlink register --shell` (PID 2575984) at 00:02:10 — a single-tick outlier, likely an attribution artifact |

**No.** The hub's own writes are about 0.1% of the device's mean write rate, so the hub does not cause the saturation. A hub write that needs a sync would still wait in the device queue during the stall ticks in section 8 (aqu-sz 84–230, w_await 240–500 ms). The hub's own write latency was not sampled, so this is an inference.

## 8. Latency stalls are not bandwidth-limited

| Tick (local) | w_await (ms) | %util | Device wkB/s (MB/s) | Device r/s | Read MB/s | Pattern |
|---|---:|---:|---:|---:|---:|---|
| 00:09:00–00:09:10 | 437–497 | 90 | 9–10 | 996–1498 | 26–72 | mixed heavy reads and writes, queue ~90 |
| 01:45:20–01:45:30 | 241–327 | 92 | 11 | 16–64 | <1 | **pure small writes**: ~650–700 w/s at ~16 KB each, aqu-sz 156–230 |

32 of the 33 ticks with %util > 80 also had w_await > 10 ms. The device is **IOPS- and latency-bound**, not bandwidth-bound. Small synchronous writes (fsync-heavy work: git, sqlite-backed indexing, test fixtures) queue behind each other.

## 9. What this means for T-3488 (hub write stalls)

1. The device latency spikes (00:09 and 01:45) happen when hub write volume is negligible. Any hub stall at those times is therefore queueing behind other writers, not the hub's own load. The hub stall timestamps from T-3488 were not in the sampler inputs, so I could not match them directly.
2. The biggest attributable writers are agent/tooling workloads: `claude` sessions, `fw`/`update-task.sh`/`git` hooks, bats test runs, headless Chromium, and the `reindex_incremental` job.
3. Recommended follow-ups (not done here):
   - Check the reindex schedule and whether it can run off-peak or with lower fsync frequency.
   - Trace the `fsync` rate of the 01:30–01:50 window.
   - Measure hub write latency directly during the 00:09 and 01:45 spikes.
   - Confirm pidstat attribution against `/proc/<pid>/io` on the next run (the 3× gap).

## 10. Limits of this analysis

- Read-only. No files were modified apart from this report. Nothing was committed.
- pidstat per-process numbers are approximate (table 3). Ranking is more reliable than absolute values.
- Command names come from pidstat's comm/cmdline column, which is truncated at 15 characters for comm and cut at a newline for some multi-line `bash -c` commands. Two `python3 -c` PIDs (2230868, 1364193) were identified by their pidstat command text; the 01:20 spike's PID was not resolved.
- `init` (PID 1 = systemd) shows 8–46 MB/s in some blocks. That is far too high for systemd itself and is probably writeback attributed to PID 1. I did not confirm this.
- One sampler outlier: `termlink register --shell` at 00:02:10 (35 MB/s, one tick) likely an artifact.
