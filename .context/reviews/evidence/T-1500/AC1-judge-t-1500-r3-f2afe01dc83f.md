# T-1500 AC1 — timeline operator-readability (reviewer judge-t-1500-r3-f2afe01dc83f)

Revision under review: c605253a580fb466f400dae6ffda250a02704516 (HEAD at review time).
Binary: /opt/termlink/target/release/termlink (built 2026-10-27 00:42).

## Step 1 — `agent timeline --window-secs 86400 --n 20`
- Exit 0. Header `# agent timeline | window=86400s | n=20`, footer `20 post(s) shown`.
- Posts are in ascending chronological order: offsets 13h → 12h → 11h … → 1h → 15m ago; offsets @2737 → @2756 strictly increasing.
- Every entry has the line `[<age>] [<peer-short 8 hex>] @<offset> msg_type=<t> [thread=..] [project=..]`, and the content is indented on the next line. The two peers (`9219671e` presence notes, `d1993c2c` heartbeats/alerts) are easy to tell apart at a glance.

## Step 2 — `agent timeline --window-secs 3600 --watch` (run under `script` pty, interval 2s, SIGINT after 7s)
- 4 frames rendered at 2s intervals (header timestamps 15:32:19Z, :21Z, :23Z, :25Z). Each frame starts with `ESC[2J ESC[H` (clear + home) and is followed by a header showing interval/window/n/filters/UTC time. SIGINT exited cleanly.
- `--watch --json` is rejected as incompatible (code: agent.rs ~2474).
- Minor note (agent.rs:2491): the screen is cleared *before* the hub fetch (`fetch_topic_msgs_paginated`), so for the length of the fetch the screen shows only the header. On the local hub this lasts milliseconds and I saw no flicker. It could show as a brief blank on a slow remote `--hub` or a large `--depth`. Optional improvement: fetch and build the frame first, then clear and print in one write.

## Step 3 — `agent timeline --msg-type note --window-secs 86400 --n 10`
- Header shows `msg_type=note`. It returned 4 posts, all `msg_type=note` (presence notes from 9219671e, at 19h/13h/7h/1h ago), in chronological order. The chat-type heartbeats seen in step 1 were correctly excluded.

## Conclusion
All expected behaviours hold: chronological order, peer-short prefix, watch refresh, and the msg-type filter. GREEN, with the optional suggestion above (render after the fetch so slow hubs don't flicker).
