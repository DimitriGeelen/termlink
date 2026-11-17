# T-2258 Human AC#1 — evidence report (rung-5 panel seat claude-code)

Revision reviewed: 879b46eac7a53bc656c448e75930c3d4c880e99e. Verdict: **amber** (literal step 3 is out of date; restatement proposed, evidence meets it).
Note: the brief names `bin/fw`; only `.agentic-framework/bin/fw` exists here, so that path was used.
All checks below were read-only, run from 192.168.10.107 on 2026-11-17 ~14:10Z.

## What I verified myself

1. **Fixed binary is the running .107 hub.** `termlink hub status` -> PID 2306830, runtime dir `/var/lib/termlink`
   (persistent). `ps`: `/root/.cargo/bin/termlink hub start --tcp 0.0.0.0:9100 --json`, started Tue Nov 17 09:54:19 local.
   `/proc/2306830/exe --version` -> `termlink 0.12.13+135.gd167da0e2`, matching the Measured blocks.
   `git merge-base --is-ancestor db7c10a56 d167da0e2` succeeds; `channel.rs` at d167da0e2 runs both large-topic
   walks through `tokio::task::spawn_blocking` (lines 1619, 1861).
2. **Step 2 reproduced.** Three `timeout -s KILL 12 termlink channel state framework:pickup --hub 192.168.10.107:9100`:
   rc=0 in 126 ms, 186 ms, 171 ms, each returning the full topic (20096 output lines).
3. **The recorded large-topic run really happened; checked against the hub's own audit log, not the producer's file.**
   `/var/lib/termlink/rpc-audit.jsonl` holds, for topic `t3814-repro-1794924411` (1794924411 = 14:06:51Z):
   1521 `channel.post`, 13 `channel.subscribe`, and `rpc-admin.jsonl` holds the `channel.delete` result
   `{"deleted_records":1521,"ok":true}` at ts 1794924450302. This matches the recorded output (1500 seeds + 1 + 20).
   The topic is gone now (`unknown topic`), as recorded.
4. **Post-then-immediate-read ordering, from the audit log.** Post at ts ...446784, the re-read's subscribes at
   ...446936/...446964 (about 150 ms after the post) — the recorded `rc=0 178ms`. A read straight after a write to
   the same 1500-record topic returned in bound.
5. `.context/working/t3814/repro-2258.sh` at the reviewed revision is the script that produced
   `.context/working/t3814/repro-2258.out`; each read is under `timeout -s KILL 12` and prints its own rc.

## Where the recorded output falls short of the criterion as written

- **Step 3 literal not performed.** Steps 2-3 say: read `framework:pickup`, "post a note to the topic, then immediately
  re-read the same topic". No post to `framework:pickup` is recorded. The post-then-read was done on a scratch topic
  built to 1500 records. The producer's stated reason: a post to `framework:pickup` is now read by AEF agents across
  the fleet as a pickup to act on, so a probe note there is no longer a harmless act (CLAUDE.md does show the topic
  used for filings that agents act on, e.g. "filed ... at framework:pickup offset 250"). That makes the step's literal
  out of date; it is not mine to rewrite, so this is amber with a proposed restatement, not green.
- **The scratch topic is a fair stand-in but not identical.** Original symptom: `framework:pickup` at ~1639 rows.
  Scratch: 1500 records with tiny payloads ("seed N"). `framework:pickup` today prints 20096 lines. Record count is
  in the same class as the original; record bytes are far smaller.
- **"4 concurrent reads while 20 posts land" is overstated.** In the audit log the four concurrent reads' subscribes
  fall at ...447056 to ...447162, and only the first 2 of the 20 landing posts (...447109, ...447231) fall inside the
  window in which those reads ran; the other 18 landed after the reads had returned. Reads concurrent with writes are
  shown, but with 2 overlapping posts, not 20. This does not change the verdict.

## Conclusion

Deploy: proven (checked on the live process). Bounded reads of `framework:pickup`: proven (recorded and reproduced).
Read immediately after a post to the same large topic: proven on a 1500-record scratch topic, corroborated by the
hub's audit log; not done on `framework:pickup` itself, which is what step 3 literally says. Amber, with:

PROPOSED RESTATEMENT: [REVIEW] Fixed binary deployed to the `.107` hub and the live repro no longer hangs: `timeout -s KILL 12 termlink channel state framework:pickup --hub 192.168.10.107:9100` returns within the bound, and on a scratch topic of at least 1500 posts on the same hub a read returns within the bound immediately after a post to that topic and while posts to it are landing (no probe post is sent to `framework:pickup`, which agents act on).
The recorded evidence meets this restatement.
