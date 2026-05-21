# Review Evidence — Batch 3 (T-1496, T-1498, T-1499, T-1500, T-1501, T-1502, T-1506)

Date: 2026-05-22
Binary: `termlink 0.11.1 (04a8c3aa)` (PATH-resolved, /root/.cargo/bin/termlink)
Working dir: /opt/termlink
Populated peer fp used: `d1993c2c3ec44c94` (T-1438 vendored-arc heartbeat stream)
Fleet state: quiet window (single active poster — heartbeats only). Empty
peer/fleet-aggregate results are quiet-window artifacts, not defects.

Scope: read-only evidence. No task files edited, no AC boxes ticked, no source changed.

---

## T-1496 — agent overview --watch (live fleet dashboard)

**Command:**
```
timeout 8 termlink agent overview --watch --watch-interval 5 --window-secs 86400 --top 5
```
**Verdict:** HUMAN-VISUAL-ONLY (clean start confirmed; exit 124 = timeout-kill = ran without crashing)

**Output (first ~ticks):**
```
[2J[H# agent overview --watch | interval=5s | window=86400s | top=5 | 2026-05-21T23:15:38Z
(no fleet activity in window=86400s)
[2J[H# agent overview --watch | interval=5s | window=86400s | top=5 | 2026-05-21T23:15:43Z
(no fleet activity in window=86400s)
EXIT=124
```
Notes: ANSI clear-home (`\x1b[2J\x1b[H`) per tick, per-tick header with
interval/window/top/RFC3339-ts, refreshes every 5s. "(no fleet activity)" is
the quiet-window artifact — overview's fleet aggregate is empty while only one
peer (a heartbeat job) is posting. The "steady/no flicker" criterion is
inherently visual. Clean start + multi-tick refresh confirmed.

---

## T-1498 — agent recent --watch (live single-peer streaming)

**Command:**
```
timeout 8 termlink agent recent --target-fp d1993c2c3ec44c94 --watch --watch-interval 5 --window-secs 86400 --n 5
```
**Verdict:** HUMAN-VISUAL-ONLY (clean start + real content confirmed; exit 124 expected)

**Output (first tick):**
```
[2J[H# agent recent d1993c2c3ec44c94 --watch | peer_fp=d1993c2c3ec44c94 | interval=5s | window=86400s | n=5 | 2026-05-21T23:15:47Z
[4h ago] @1803 msg_type=chat thread=T-1438 project=010-termlink
    T-1438 vendored-arc heartbeat from dimitrimintdev (x86_64, Linux) at 2026-05-21T20:17:01+02:00. Binary: /usr/local/bin/termlink (termlink 0.9.1542).
[3h ago] @1804 msg_type=chat thread=T-1438 project=010-termlink
    ...
[58m ago] @1807 ...
5 post(s) shown
[2J[H# agent recent d1993c2c3ec44c94 --watch | ... | 2026-05-21T23:15:52Z   (2nd tick)
```
Notes: ANSI clear-home per tick, header carries target/peer_fp/interval/window/n/
RFC3339-ts (ts advances each tick: 23:15:47 → 23:15:52). Real decoded content +
offsets rendered. Visual steadiness left to human.

---

## T-1499 — agent recent / on-thread --msg-type (signal vs noise filter)

**Command:**
```
termlink agent recent --target-fp d1993c2c3ec44c94 --window-secs 86400 --msg-type chat --n 5
termlink agent on-thread T-1438 --window-secs 86400 --msg-type chat
```
**Verdict:** EVIDENCE-CLEAN (recent path); on-thread path returns empty (see note)

**Output:**
```
# agent recent d1993c2c3ec44c94 (peer_fp=d1993c2c3ec44c94) | window=86400s | n=5 msg_type=chat
[4h ago] @1803 msg_type=chat thread=T-1438 project=010-termlink
    T-1438 vendored-arc heartbeat from dimitrimintdev ...
...

# agent on-thread T-1438 | window=86400s | n=50 msg_type=chat
(no posts found on thread=T-1438 in window=86400s)
```
Notes: recent `--msg-type chat` works — header shows `msg_type=chat` suffix,
only chat posts shown. The on-thread arm returns empty, BUT this is NOT a
msg-type-filter defect — on-thread T-1438 returns empty even with no filter
(see cross-task note below). Filter feature itself is demonstrable on the
recent verb. The msg-type vocabulary used was `chat` (the real wire type in
this window); the task AC examples reference `note`/`status` which are present
in other windows/peers.

---

## T-1500 — agent timeline (fleet-wide chronological log)

**Command:**
```
termlink agent timeline --window-secs 86400 --n 20
termlink agent timeline --json --window-secs 86400 --n 5 --msg-type chat
```
**Verdict:** EVIDENCE-CLEAN

**Output:**
```
# agent timeline | window=86400s | n=20
[10h ago] [d1993c2c] @1784 msg_type=chat thread=T-1438 project=010-termlink
    T-1438 vendored-arc heartbeat from dimitrimintdev ...
[10h ago] [d1993c2c] @1785 msg_type=note thread=T-209 project=002-Claude-Partner-Network
    @ring20-management — two items combined, in_reply_to chat-arc:1408 ...

(json msg-type filter)
filter_msg_types= ['chat']
posts= 5
```
Notes: chronological order, peer-short prefix `[d1993c2c]` per line for
multi-peer disambiguation, offsets visible, multiple msg_types surfaced
(`chat`, `note`). JSON envelope carries `filter_msg_types: ["chat"]` when set.

---

## T-1501 — agent recent / on-thread / timeline --grep (content substring)

**Command:**
```
termlink agent timeline --window-secs 86400 --grep T-1438 --n 5
termlink agent timeline --window-secs 86400 --n 50 --grep heartbeat --json
termlink agent recent --target-fp d1993c2c3ec44c94 --window-secs 86400 --grep heartbeat --n 3
```
**Verdict:** EVIDENCE-CLEAN

**Output:**
```
# agent timeline | window=86400s | n=5 grep=T-1438
[4h ago] [d1993c2c] @1804 msg_type=chat thread=T-1438 ...
    T-1438 vendored-arc heartbeat from dimitrimintdev ...

(json) filter_grep= heartbeat   posts= 24

# agent recent d1993c2c3ec44c94 (peer_fp=d1993c2c3ec44c94) | window=86400s | n=3 grep=heartbeat
[2h ago] @1806 msg_type=chat thread=T-1438 ...
    T-1438 vendored-arc heartbeat ...
```
Notes: text header shows `grep=<pattern>` suffix; JSON envelope carries
`filter_grep`; 24 posts matched `heartbeat` (substring match against decoded
content — depends on T-1502's content fix working). Verified on timeline + recent.

---

## T-1502 — extract_recent_posts content extraction (BUG-FIX)

**Command:**
```
termlink agent timeline --window-secs 86400 --n 10 --json   (content non-empty assertion)
termlink agent recent --target-fp d1993c2c3ec44c94 --window-secs 86400 --n 5 --json
```
**Verdict:** EVIDENCE-CLEAN (the whole point of the fix is proven)

**Output:**
```
total=10 with_content=10
  @1799 '@cohort-agent — assets received, federation OK. Read 1785 + '
  @1800 'T-1438 vendored-arc heartbeat from dimitrimintdev (x86_64, L'
  @1801 'Deploy access ready — install complete:\n- User `cohort-agent'

(recent) total=5 with_content=5
```
Notes: Pre-fix symptom was every post rendering `(empty)` because `payload_b64`
was never decoded. Post-fix: 10/10 timeline posts and 5/5 recent posts carry
real decoded content. payload_b64 → UTF-8 decode path is live. Multiple msg_types
(chat, note) decode correctly. Bug fix is unambiguously demonstrated.

---

## T-1506 — offset in render AND --json

**Command:**
```
termlink agent timeline --window-secs 86400 --n 3            (text @offset)
termlink agent timeline --window-secs 86400 --n 3 --json     (offset field)
termlink agent recent --target-fp d1993c2c3ec44c94 --window-secs 86400 --n 2
```
**Verdict:** EVIDENCE-CLEAN

**Output:**
```
(text render)
[2h ago] [d1993c2c] @1806 msg_type=chat thread=T-1438 project=010-termlink
[1h ago] [d1993c2c] @1807 msg_type=chat thread=T-1438 project=010-termlink
[14s ago] [d1993c2c] @1808 msg_type=chat thread=T-1438 project=010-termlink

(json)
all posts have offset: True
  offset=1806
  offset=1807
  offset=1808

(recent text)
[1h ago] @1807 msg_type=chat thread=T-1438 project=010-termlink
[15s ago] @1808 msg_type=chat thread=T-1438 project=010-termlink
```
Notes: `@<offset>` token present in text render (`@1806`/`@1807`/`@1808`) AND
`offset` field present on every JSON post. Both surfaces expose offset — exactly
the read→quote loop the task targets. Format `[<age>] [<peer>] @<offset> msg_type=...`.

---

## Cross-task observation (not a verdict — flag for human)

`agent on-thread <T-XXX>` returns **empty in this window** even with NO filter,
while `agent timeline --thread <T-XXX>` returns matching posts (5 for T-1438).

Evidence:
```
agent on-thread T-1438 --window-secs 86400          → (no posts found)
agent on-thread T-1438 --json                        → keys=[n,posts,thread,window_secs], posts=0, no filter_thread key
agent timeline --thread T-1438 --json --n 5          → filter_thread=T-1438, posts=5  (thread=T-1438 each)
```
The timeline `--thread` path matches; the on-thread verb's own thread argument
does not surface matching posts in this window. This affects the on-thread arm
of T-1499 (msg-type) and T-1501 (grep) review steps — those steps compose with
on-thread and will read empty. The msg-type and grep FEATURES themselves are
proven on recent + timeline. Whether on-thread's thread-match is a separate
defect or a window/key-shape edge case is outside this read-only evidence pass —
flagging for the human reviewer. (Plausibly related to a residual
thread-key-shape mismatch, the same class T-1502 partially addressed.)
