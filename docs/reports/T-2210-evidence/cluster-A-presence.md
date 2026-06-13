# T-2210 Evidence — Cluster A (agent presence/who/ping/contact)

Fresh G-008 resmoke captured 2026-06-13T13:41:18Z from /opt/termlink via target/release/termlink (built Jun 13 12:14).
Peer fingerprint used: d1993c2c3ec44c94 (live). Watch commands wrapped in `timeout 6` (exit 124 = success-with-partial).

## T-1482
```
$ target/release/termlink agent presence --window-secs 86400
PEER_FP                 LAST_SEEN    POSTS  TOP_PROJECT
d1993c2c3ec44c94          23m ago       19  010-termlink
9219671e28054458           3h ago        2  proxmox-ring20-management

2 peer(s) active in window=86400s
[exit=0]
```

## T-1483
```
$ target/release/termlink agent who --target nonexistent-session-xyz
error: Session 'nonexistent-session-xyz' not found: session not found: nonexistent-session-xyz
[exit=1]
$ target/release/termlink agent who --target some-name --target-fp deadbeefdead
Error: specify either --target or --target-fp, not both
[exit=1]
$ target/release/termlink agent who
Error: must specify either --target <name> or --target-fp <hex>
[exit=1]
```

## T-1484
```
$ target/release/termlink agent presence --filter-project nonexistent-xyz
(no peers active in window=3600s matching project=nonexistent-xyz)
[exit=0]
```

## T-1485
```
$ target/release/termlink agent contact --target-fp deadbeefdeadbeef --message hi --ack-required --ack-timeout-secs 5
Error: agent contact: posting to dm topic for peer fp=deadbeefdeadbeef failed

Caused by:
    channel.create failed: JSON-RPC error -32603: channel.create: topic "dm:d1993c2c3ec44c94:deadbeefdeadbeef" already exists with a different retention policy (existing=Forever, requested=Messages(1000))
[exit=1]
```

## T-1486
```
$ timeout 6 target/release/termlink agent presence --watch --watch-interval 3 --window-secs 86400
[2J[H# agent presence --watch | view=by-peer | interval=3s | window=86400s | 2026-06-13T13:40:43Z
PEER_FP                 LAST_SEEN    POSTS  TOP_PROJECT
d1993c2c3ec44c94          23m ago       19  010-termlink
9219671e28054458           3h ago        2  proxmox-ring20-management

2 peer(s) active in window=86400s
[2J[H# agent presence --watch | view=by-peer | interval=3s | window=86400s | 2026-06-13T13:40:46Z
PEER_FP                 LAST_SEEN    POSTS  TOP_PROJECT
d1993c2c3ec44c94          23m ago       19  010-termlink
9219671e28054458           3h ago        2  proxmox-ring20-management

2 peer(s) active in window=86400s
[exit=124]
```

## T-1487
```
$ target/release/termlink agent ping --target-fp d1993c2c3ec44c94 --window-secs 86400
d1993c2c3ec44c94 (d1993c2c3ec44c94): online — last seen 23m ago (window=86400s)
[exit=0]
$ target/release/termlink agent ping --target-fp deadbeefdeadbeef --window-secs 60
deadbeefdeadbeef (deadbeefdeadbeef): offline — last seen never (window=60s)
[exit=1]
```

## T-1488
```
$ target/release/termlink agent who --target-fp d1993c2c3ec44c94 --window-secs 86400
peer_fp:           d1993c2c3ec44c94
last_seen:         1428s ago (ts_ms=1781356621735)
posts_in_window:   19 (window_secs=86400)
from_projects:
  010-termlink                       18
  100-Video-riper-and-translation-app      1
[exit=0]
$ target/release/termlink agent who --target-fp d1993c2c3ec44c94 --thread T-1487 --window-secs 86400
# filter_thread=T-1487
peer_fp:           d1993c2c3ec44c94
last_seen:         1428s ago (ts_ms=1781356621735)
posts_in_window:   0 (window_secs=86400)
from_projects:     (none observed in window)
[exit=0]
```

## T-1489
```
$ target/release/termlink agent presence --top 1 --window-secs 86400
PEER_FP                 LAST_SEEN    POSTS  TOP_PROJECT
d1993c2c3ec44c94          23m ago       19  010-termlink

1 of 2 peer(s) active in window=86400s
[exit=0]
```

## T-1490
```
$ target/release/termlink agent presence --thread T-1487 --window-secs 86400
(no peers active in window=86400s matching thread=T-1487)
[exit=0]
```

## T-1491
```
$ target/release/termlink agent presence --by-project --window-secs 86400
PROJECT                     POSTS    PEERS TOP_PEER            LAST_SEEN
010-termlink                   18        1 d1993c2c3ec44c94    23m ago
100-Video-riper-and-translation-app        1        1 d1993c2c3ec44c94    1h ago
proxmox-ring20-management        1        1 9219671e28054458    4h ago
termlink                        1        1 9219671e28054458    3h ago

4 project(s) active in window=86400s
[exit=0]
```

