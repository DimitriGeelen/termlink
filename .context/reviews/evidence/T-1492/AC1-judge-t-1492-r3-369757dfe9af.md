# T-1492 AC1 — recent-post output operator-readable

Reviewer: judge-t-1492-r3-369757dfe9af · Revision 0501adf2b08c06f055868fbfa990184412e9947e
Binary: /opt/termlink/target/release/termlink (built Oct 26 22:54)

## Command as specified (exit 0)
```
$ target/release/termlink agent recent --target-fp d1993c2c3ec44c94 --window-secs 3600 --n 5
# agent recent d1993c2c3ec44c94 (peer_fp=d1993c2c3ec44c94) | window=3600s | n=5
[52m ago] @2731 msg_type=chat thread=T-1438 project=010-termlink
    T-1438 vendored-arc heartbeat from dimitrimintdev (x86_64, Linux) at 2026-10-26T22:17:01+01:00. Binary: /usr/local/bin/termlink (termlink 0.12.1160).

1 post(s) shown
```
Only 1 post fell in the 1h window, so to judge delimitation between multiple posts I also ran
the same command with `--window-secs 86400` (supplementary, same formatter):

```
# agent recent d1993c2c3ec44c94 (peer_fp=d1993c2c3ec44c94) | window=86400s | n=5
[3h ago] @2726 msg_type=chat thread=T-1438 project=010-termlink
    T-1438 vendored-arc heartbeat from dimitrimintdev ... (termlink 0.12.1160).

[2h ago] @2727 msg_type=chat thread=T-1438 project=010-termlink
    T-1438 vendored-arc heartbeat from dimitrimintdev ... (termlink 0.12.1160).

[2h ago] @2728 msg_type=chat thread=T-1438 project=root
    ALERT: vendored chat-arc rollout — 1 hub(s) STALE: ring20-management ... Topic…

[1h ago] @2730 ...
[52m ago] @2731 ...

5 post(s) shown
```

## Assessment
- Header line states target, fp, window and n.
- Each post: one metadata line led by a bracketed relative timestamp (`[52m ago]`), offset `@N`,
  msg_type/thread/project; content indented 4 spaces on the next line; blank line between posts.
  Posts are clearly delimited and scannable.
- Long content is truncated with `…` (post @2728), keeping each block to 2 lines.
- Chronological order (oldest→newest), footer count.

Verdict: green. Optional polish (not blocking): relative times are coarse (two posts both "[2h ago]");
an absolute HH:MM alongside the relative age would help disambiguate.
