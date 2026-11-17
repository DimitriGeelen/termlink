# T-2258 Human AC#1 — evidence report (rung-5 panel seat claude-code)

Revision reviewed: 2d0f172a9343aa3e8499a8b49d3995e1dbb73aa4. Verdict: **amber**.
Note: the brief names `bin/fw`; only `.agentic-framework/bin/fw` exists here, so that path was used.

## What I checked myself (read-only, from 192.168.10.107, 2026-11-17 ~13:57Z)

1. Fix is in the deployed revision. `git merge-base --is-ancestor db7c10a56 d167da0e2` succeeds
   (db7c10a56 = "T-2258: fix hub read-path stall — spawn_blocking for large-topic walks");
   `crates/termlink-hub/src/channel.rs` at d167da0e2 runs the walk via `tokio::task::spawn_blocking` (line 1619).
2. Fixed binary is the running hub. `termlink hub status` -> PID 2306830;
   `ps`: `/root/.cargo/bin/termlink hub start --tcp 0.0.0.0:9100 --json`, started Tue Nov 17 09:54:19 2026 local
   (= 08:54Z, consistent with the Measured block's 08:50Z operator run);
   `/proc/2306830/exe --version` -> `termlink 0.12.13+135.gd167da0e2`. Runtime dir /var/lib/termlink (persistent, not /tmp).
   Matches the Measured block's `hub_version 0.12.13+135.gd167da0e2`.
3. Step 2 reproduced. Three runs of `timeout -s KILL 12 termlink channel state framework:pickup --hub 192.168.10.107:9100`:
   rc=0 in 0.25 s, 0.14 s, 0.16 s, each returning the full topic (20096 output lines, offsets up to 360).
   Consistent with the five recorded reads (0.14-0.38 s).
4. The scratch topic exists: `channel state test-t3814-probe --hub 192.168.10.107:9100` returns
   `[0] ...: T-3814 post-then-read probe`, so the recorded post did happen.

## What the recorded output does not show

Step 3 says "Post a note to the topic, then immediately re-read the same topic", where the topic is
`framework:pickup`, and Expected says the read returns in bound "including immediately after a post".
The Measured block posted to a new scratch topic `test-t3814-probe` and re-read that one-record topic.
The task's own description says the stall is on "large/low-traffic topics" and is "aggravated by a recent
write to the same topic". A re-read of a one-record topic after a post does not exercise a large-topic walk
right after a write to that topic, so the aggravated condition the criterion names was not measured.
No read of framework:pickup immediately after a post to framework:pickup is recorded.

I did not post to framework:pickup myself: it is a shared fleet topic and this seat is read-only.

## Conclusion

Deploy: proven. Bounded reads of framework:pickup: proven (recorded and reproduced).
Read immediately after a post to the same large topic: not shown. Mostly met -> amber.
