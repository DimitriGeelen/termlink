# T-3004 — Chat-arc traffic collapse investigation (C-40)

**Task:** T-3004 (inception, arc-009, S-29b/C-40)
**Source finding:** consolidated value review C-40 — 0 posts + 0 unique speakers for 10 consecutive
daily fleet-adoption snapshots after progressive decline; historically 92% single-sender
(`d1993c2c3ec44c94`, 871/950).
**Measured:** 2026-09-20, live against the fleet + `.context/working/.fleet-adoption-snapshot.log`
(91 daily blocks, 36 consecutive zeros at the tail).

## Verdict up front

**The rail is not dead — the counter is blind on the one hub where the rail actually lives.**
agent-chat-arc on workstation-107-public carries posts from TODAY (hourly vendored heartbeats,
a peer-project note from 832-Workflow-designer at 08:05Z, presence notes, rollout alerts; 4 unique
speakers in the 30-day window), while the fleet-adoption snapshot has reported 0/0 fleet-wide for
36 consecutive days. The zero-streak is a compound of one measurement defect and two real per-hub
declines.

## Findings

**F1 — Streak timing.** 36 consecutive zero blocks at the log tail; at daily cadence that starts
**2026-08-16 — the commit date of T-2758** (`84df1c239`, "seek from tail offset, not count"),
the very fix meant to cure this class in the read path.

**F2 — .107 (the live copy): counter structurally blind, and the T-2758 port is inert here.**
The local hub does not serve `latest_offset` (pre-T-2533 binary — `channel info` returns only
`count`/`receipts`/`retention`). The snapshot's fallback tail is `max(count-1, max receipt up_to)`
= max(1000, 923) = 1000, but the topic is retention-trimmed (messages/1000): live offsets run
**553..~1553**. Cursor = 1000−500 = 500 ⇒ the 500-envelope scan covers offsets 553..~1052 — all
older than 24h ⇒ windowed count 0. **Reproduced live:** `subscribe --cursor 500 --since <24h>` →
0 envelopes; `--cursor 1000` → a Sep-02 envelope at offset 1000; the true tail holds today's
posts. Note .107 read CHAT_ARC=0 even *before* the streak — the fleet number was only ever
counting the other hubs' copies.

**F3 — .122 and .121: real per-hub declines (G-060 semantics).** ring20-management (.122) is
reachable but its copy stopped growing (count static at 2012; its metronomic poster
`9219671e28054458` — the presence-note poster — is stale; today's rollout-detector alert names
ring20-management STALE). Its `up_to=2329` receipt exceeds its own tail 2011 (stale epoch), a
receipts-beyond-tail hazard for the fallback on any hub in this state. ring20-dashboard (.121),
the pre-streak 143/day metronome, is now **partial-unreachable** (channel info times out) —
today's block says so. 167/day pre-streak = 143 (.121) + 24 (.122) + 0 (.107, already blind).

**F4 — `d1993c2c3ec44c94` identified.** It is the shared HOST identity on the dimitrimintdev dev
host: the same fingerprint signs the hourly `dimitrimintdev-vendored` T-1438 heartbeats (874 of
1001 retained posts on .107) AND 832-Workflow-designer's cross-project notes (self-labeled
"[832-Workflow-designer -> 999-AEF, cc 010-termlink]"). The "92% single-sender" statistic
measures a host keypair, not an agent — exactly the T-2838 caveat. Traffic diversity is low
(mostly automated heartbeats) but is multi-logical-sender.

## Recommendation (GO — one small fix task + two already-owned handoffs)

1. **Fix the snapshot counter** (small build task): derive the tail the way
   `agent-chat-arc-recent.sh` does post-T-2758 (page forward from `count-1` when `latest_offset`
   is absent, and never trust a receipt `up_to` beyond the derived tail — F3 shows receipts can
   exceed it). Fixture: a trimmed-topic + no-latest_offset case pinning today's reproduce.
2. **Hub upgrade path:** .107's hub predates T-2533 (`latest_offset` missing) — that is exactly
   arc-009's **T-2977** (binary reinstall/hub restart) already filed; note the dependency, don't
   duplicate.
3. **.122 staleness / .121 unreachability** are already surfaced by the rollout detector and
   fleet doctor respectively; no new task.

C-40's implied severity ("fleet's main broadcast rail dead") is corrected: the rail is alive on
the hub of record; what died is the measurement (Directive #2 violated inside the metrics layer —
same class as T-2680's canary scope lesson).
