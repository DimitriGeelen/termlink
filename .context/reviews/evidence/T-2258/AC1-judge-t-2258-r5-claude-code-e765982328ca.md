# T-2258 Human AC#1 — independent review (rung-5-panel:claude-code)

Revision reviewed: 37c945a39df76fc1f82450555b8e5852542a78c2. Checked 2026-11-17 23:14Z from the host 192.168.10.107.

Verdict: **green**

## Clause by clause

**1. Fixed binary deployed to the .107 hub.** Checked live, not taken from the Measured block.
- Listener on 0.0.0.0:9100 is pid 2306830, `/root/.cargo/bin/termlink hub start --tcp 0.0.0.0:9100 --json`, started Tue Nov 17 09:54:19 CET (08:54Z), which fits the recorded "deployed 08:50Z, hub restarted".
- `/proc/2306830/exe --version` → `termlink 0.12.13+135.gd167da0e2` (the running image, not "(deleted)"; binary mtime 09:54:18 CET). Same version as the Measured block.
- `git merge-base --is-ancestor db7c10a56 d167da0e2` succeeds: the fix commit is in the deployed build. `git show d167da0e2:crates/termlink-hub/src/channel.rs` has both large-topic walks on `tokio::task::spawn_blocking` (lines 1619 and 1861).

**2. `timeout -s KILL 12 termlink channel state framework:pickup --hub 192.168.10.107:9100` returns within the bound.**
- Recorded: 5 reads 0.14–0.38 s and 3 reads 134–166 ms, all rc=0.
- My own three runs of the exact command: rc=0 in 220 ms, 196 ms, 204 ms, each returning the real state (20096 lines of output, starting "Canonical state of 'framework:pickup':"). So rc=0 is a full read, not an empty early exit.

**3. Scratch topic of 1500+ posts: read right after a post, and during landing posts, within the bound.**
- I did not re-run this: it writes 1521 records to a live shared hub and I am read-only. I judged the recorded run.
- `.context/working/t3814/repro-2258.sh` does what the Measured block says: 1500 seed posts to `t3814-repro-<epoch>`, cold read, one post then an immediate read, 4 backgrounded reads started while a 20-post loop is running, every read under `timeout -s KILL 12`, then delete.
- `.context/working/t3814/repro-2258.out` matches the block in the task file line for line: read-after-post rc=0 178 ms; concurrent reads rc=0 112/153/173/189 ms.
- `Deleted topic ... (1521 record(s) removed)` = 1500 + 1 + 20, so every post landed and the topic met the 1500+ condition. The hub reports that count, not the script.
- Limit of the proof: the four concurrent reads finished within 189 ms, while 20 sequential CLI posts take a few seconds, so the reads overlapped only the first posts of the loop. They did run while posts were landing, which is what the criterion asks.

**4. No probe post to framework:pickup.** The script only reads framework:pickup; posts go to the scratch topic.

## Notes (do not affect the verdict)
- The script's `hub binary:` line printed empty (the `fleet doctor` grep matched nothing); the version comes from the separate `hub.version` read, which I confirmed against the running process.
- The first run's small scratch topic `test-t3814-probe` (retention days:14) still exists on the hub. The 1500-post topic is gone (`channel list` shows no `t3814-repro-*`).
