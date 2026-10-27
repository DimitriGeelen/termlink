# T-1502 AC1: independent review evidence

Reviewer: judge-t-1502-r1-eeb26d701d75 (rung-1-same-vendor-independent)
Revision: 01d2972266e4ab299c0ad3ac20d5f56592f008b9 (HEAD at review time)
Binary: /opt/termlink/target/release/termlink (built Oct 27 00:42)

## Step 1: `agent timeline --window-secs 86400 --n 10`: PASS
Showed 10 posts, each with real decoded text and no "(empty)". Examples:
- `@2754 msg_type=note` from 9219671e: "presence: ring20-management (9219671e28054458) online @ 2026-10-27T14:02:04Z on .122. Heartbeat is every 12h at most…". A **note** showed its decoded content, which is the core of the fix.
- `@2751 msg_type=chat thread=T-1438`: "ALERT: vendored chat-arc rollout — 1 hub(s) STALE: ring20-management…"
- `@2747..@2756 msg_type=chat thread=T-1438`: "T-1438 vendored-arc heartbeat from dimitrimintdev…"

## Step 2: `agent recent --target-fp d1993c2c3ec44c94 --window-secs 86400 --n 5`: PASS
Showed 5 posts (@2751, @2752, @2753, @2755, @2756), all with decoded text. The peer filter correctly left out @2754, which came from 9219671e.

## Step 3: `agent on-thread T-1500 --window-secs 86400`: expected output NOT reproduced
Output: `(no posts found on thread=T-1500 in window=86400s)`. The same happened with a wider window (capped at 604800s).

To find out whether this is a filter bug or missing data, I paged the whole topic with
`channel subscribe agent-chat-arc --cursor {0,1000,2000} --limit 1000 --json`. That returned 2757 envelopes (offsets 0..2756). I decoded every payload_b64 and counted the metadata thread values:
- `_thread` distribution: T-1438=2368, none=351, T-076=21, T-999=6, T-090=3, T-134=3, T-139=3, pen-inbound-archive-probe=2
- **No envelope has `_thread`=T-1500, and no decoded payload contains the string "T-1500".**
- Envelopes with an empty decoded payload: 0

Wire shape (offset 2756): `{"metadata":{"_from":…,"_thread":"T-1438","client_msg_id":…,"from_project":"010-termlink"},"msg_type":"chat","offset":2756,"payload_b64":"VC0xNDM4…","sender_id":"d1993c2c3ec44c94",…}`

Control check: `agent on-thread T-1438 --window-secs 86400` returned **30 posts** with decoded content. The thread filter reads the `_thread` metadata key correctly.

## Conclusion
The fixed reading verbs do show real decoded chat-arc content, notes included. The thread filter works, as T-1438 shows. Step 3 cannot show "matching posts" for T-1500 because the agent-chat-arc topic has no posts on that thread. This is a defect in the AC's test data, not in the code. I am not voting green because the expected result as written was not observed.
