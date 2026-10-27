# T-1495 Human AC#1 — overview operator-readable as session "first command"

Reviewer: judge-t-1495-r3-bde97ce1eeef  Revision: 5c962841cf4dea1daf63c4a4dd126726c2d6d1bb

## Executed (from /opt/termlink, 2026-10-27)
`target/release/termlink agent overview --window-secs 86400` (binary mtime Oct 27 00:42; help text cites T-1495/T-1496/T-1819)

## Output (verbatim)
```
## Top Peers (window=86400s, top=5)
PEER_FP                 LAST_SEEN    POSTS  TOP_PROJECT
d1993c2c3ec44c94          44m ago       30  010-termlink
9219671e28054458          59m ago        4  -

## Top Projects (window=86400s, top=5)
PROJECT                     POSTS    PEERS TOP_PEER            LAST_SEEN
010-termlink                   24        1 d1993c2c3ec44c94    44m ago
root                            6        1 d1993c2c3ec44c94    3h ago

## Recent Posts (window=86400s, top=5)
[3h ago] peer=d1993c2c3ec4 msg_type=chat thread=T-1438 project=root
    ALERT: vendored chat-arc rollout — 1 hub(s) STALE: ring20-management at 2026-10-27T12:43:23+01:00. M…
[2h ago] ... (3 heartbeats, 1 presence note)
[44m ago] peer=d1993c2c3ec4 msg_type=chat thread=T-1438 project=010-termlink
    T-1438 vendored-arc heartbeat ...

# overview: window=86400s, top=5, total_peers=2, total_projects=2
```

## Assessment
- Three clearly delimited sections: `## Top Peers`, `## Top Projects`, `## Recent Posts`, each with window/top in header, blank line separators, plus a one-line `# overview:` footer summary. MET.
- Scannable <5s: aligned column tables with relative times ("44m ago"), message previews truncated with ellipsis to one line each. Fleet state (2 peers, 2 projects, most recent activity 44m ago, one STALE alert) is graspable at a glance. MET.
- Data consistency: per-peer post total 30 = 24+6 project posts for d1993c...; peer 9219671e posts without project shown as "-" (correct, excluded from projects table).

## Non-blocking suggestions
- Recent Posts are oldest-first; newest-first (or putting the totals footer at the top) would make the most important line appear first when output is long.
- An ALERT line in Recent Posts is not visually distinguished; could be highlighted.
