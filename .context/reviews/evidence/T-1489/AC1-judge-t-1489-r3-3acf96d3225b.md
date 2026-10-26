# T-1489 AC1 — truncation footer readability

Reviewer dispatch: judge-t-1489-r3-3acf96d3225b, revision 839f4acba (binary target/release/termlink, built Oct 26 22:54).

## Ran (from /opt/termlink)
```
$ target/release/termlink agent presence --top 1 --window-secs 86400
PEER_FP                 LAST_SEEN    POSTS  TOP_PROJECT
d1993c2c3ec44c94          48m ago       30  010-termlink

1 of 2 peer(s) active in window=86400s
exit=0
```
Control without --top:
```
d1993c2c3ec44c94          48m ago       30  010-termlink
9219671e28054458           2h ago        4  -

2 peer(s) active in window=86400s
```
Source: crates/termlink-cli/src/commands/agent.rs:3591-3603 — `"{shown} of {total}"` only when `--top` truncates.

## Finding
- Mechanically met: the footer carries both the shown count (1) and the original total (2), and only when truncation applied.
- Readability: "1 of 2 peer(s) active" reads naturally as "only one of the two peers was active", which is false here — both peers were active in the window (the control run shows 2 active). Nothing in the footer says the list was cut by --top, so the truncation is not visible as truncation.

## Suggested phrasing
`showing top 1 of 2 peer(s) active in window=86400s`
(or `2 peer(s) active in window=86400s (showing top 1)`).
