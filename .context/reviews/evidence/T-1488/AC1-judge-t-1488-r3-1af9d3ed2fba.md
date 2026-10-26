# T-1488 Human AC#1 — thread-filter output for `agent who` (reviewer judge-t-1488-r3-1af9d3ed2fba)

Revision: e7e3876149ffd7285217f92bcc6cdf71047e6e4d. Binary: /opt/termlink/target/release/termlink (built Oct 26 22:54).

## Step 1: unfiltered, 24h
```
peer_fp:           d1993c2c3ec44c94
last_seen:         2783s ago (ts_ms=1793049421249)
posts_in_window:   30 (window_secs=86400)
from_projects:
  010-termlink                       24
  root                                6
```
## Step 2: --thread T-1487, 24h
```
# filter_thread=T-1487
peer_fp:           d1993c2c3ec44c94
last_seen:         2784s ago (ts_ms=1793049421249)
posts_in_window:   0 (window_secs=86400)
from_projects:     (none observed in window)
```
## Step 3: comparison
- 0 ≤ 30, as expected. `agent recent --json` for the same peer and window shows that all 30 posts are on thread T-1438 (24 from 010-termlink, 6 from root), so 0 posts on T-1487 is correct.
- Positive control `--thread T-1438` (24h) gives 30 posts and the same projects. That is correct too, because every post in the window is on T-1438.
- Narrowing control with a 7-day window (604800). Ground truth from `agent recent`: 195 posts. T-1438 has 191 (152 from 010-termlink, 39 from root), T-076 has 3 (002-Claude-Partner-Network) and T-139 has 1 (0506-Voxtype-extention).
  - Unfiltered: 195 posts, 4 projects.
  - `--thread T-1438`: 191 posts. from_projects narrows to 010-termlink 152 and root 39. The other two projects are dropped. This matches the ground truth exactly.
- Readability: the `# filter_thread=` header makes the scope clear, and an empty result gives an explicit "(none observed in window)". `--json` echoes `filter_thread`.

## Minor non-blocking notes
- In text mode, `last_seen` is deliberately left unfiltered, but the output doesn't label it that way (only `--help` says so). A label such as "(any thread)" would avoid confusion.
- A zero result could suggest checking the thread id or widening the window.

Verdict: green.
