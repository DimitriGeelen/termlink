# T-1991 — Spike: `channel info` hub concurrency regression in 0.11.473

**Date:** 2026-06-05
**Owner:** claude (agent), human (decision-maker)
**Status:** Inception — recommendation: GO on two follow-ups.

## Origin

Steady-state verification of T-1985 (`.122 presence-heartbeat cron`) wedged
the `fw task update --status work-completed` verification gate. Investigation
of the wedge revealed that the gate's `agent-listeners.sh` call was waiting
on `termlink channel info agent-presence --json --hub 192.168.10.122:9100`.

Initial hypothesis: agent-presence topic-bloat (1493 envelopes accumulated
in 24h of per-minute cron firing) was causing client-side scan slowdown.

## Spike Plan

1. Measure `channel subscribe --cursor N --limit 200` latency vs `N` on
   .122 (1493 envelopes) and .107 (13441 envelopes). If subscribe scales
   with cursor depth, T-1844's windowing isn't actually effective.
2. Measure end-to-end `agent-listeners.sh` latency on .122 and .107.
3. Measure repeated-call latency of `channel info`, `hub status`, and
   `ping` on each hub. Compare topics of different sizes.
4. Compare hosts by version (0.11.472 vs 0.11.473) under matched network
   conditions to isolate "topic size" vs "hub version" as the dominant
   factor.

## Findings

### 1. `channel subscribe` is O(1) on cursor depth — bloat doesn't slow reads

| Hub | count | cursor | limit | wall |
|---|---|---|---|---|
| .122 | 1493 | 0 | 200 | 50ms |
| .122 | 1493 | 1490 | 200 | 40ms |
| .107 | 13441 | 0 | 200 | 70ms |
| .107 | 13441 | 13240 | 200 | 60ms |

The hub has an efficient cursor index. T-1844's `cursor=count-limit`
windowing in `agent-listeners.sh` is correctly designed and effective.
The topic-bloat hypothesis is **disproven** as a perf cost on the read path.

### 2. End-to-end `agent-listeners.sh` is fast — when called in isolation

| Hub | total run |
|---|---|
| .122 | 0.28s |
| .107 (LAN) | 0.79s |

Not the source of the gate wedge in isolation.

### 3. Sequential `channel info` on the .473 hubs is flaky

20-trial sequential timeout-at-15s rate, all calls clean topics matching
the install on each host:

| Hub | Version | Topic | Count | Timeout rate |
|---|---|---|---|---|
| .107 (loopback) | 0.11.472 | agent-presence | 13441 | 0/10 |
| .107 (LAN) | 0.11.472 | agent-presence | 13441 | 0/20 |
| .121 (LAN) | 0.11.473 | agent-presence | 785 | 1/10 (10%) |
| .122 (LAN) | 0.11.473 | agent-presence | 1503 | 9/20 (45%) |
| .141 (LAN) | 0.11.473 | agent-presence | 797 | 4/10 (40%) |
| .122 (LAN) | 0.11.473 | agent-chat-arc | 894 | 4/20 (20%) |
| .122 (LAN) | 0.11.473 | inbox (missing) | 0 | 0/20 |
| .122 (LAN) | 0.11.473 | hub status | n/a | 0/20 |
| .122 (LAN) | 0.11.473 | ping | n/a | 0/20 |

Two clean facts:

- **0.11.472 is reliable, 0.11.473 is flaky.** Loopback vs LAN doesn't
  explain it — .107 over LAN with a 9× larger topic still scored 0/20.
- **Topic size amplifies the bug.** Within the .122 0.11.473 hub:
  empty topic = 0% timeout, 894 envelopes = 20%, 1503 envelopes = 45%.
- **The bug is specific to `channel info`** on .473. `hub status` and
  `ping` are clean.

`channel info`'s help text says it "Walks the topic once" — that's
expected to be O(count), and 0.11.472 handled it serially. Something
changed in 0.11.473 that makes this walk wedge under sequential load,
likely a lock or scheduling issue. The TCP timeout at exactly 15.00s
on every failure (no partial returns) suggests the hub stops responding
mid-stream rather than just being slow.

## Recommendation

**GO** on two follow-ups; close T-1991 as the scoping decision.

### Follow-up 1 — Operator-side immediate mitigation (small task)

Roll agent-presence on .121/.122/.141: redact-with-retention is one
option but n² in envelope count and the .473 hubs ARE the affected
fleet (chicken-and-egg). The more tractable workaround is **client-side
caching of the listener probe**: have `agent-listeners.sh` cache its
JSON output for N seconds under `~/.termlink/cache/`, so back-to-back
calls don't hit the hub at all. Side benefit: closes the
`/agent-handoff`-resolution flake observed during peer DM.

Scope estimate: 1 small build task, ~1 session, scripts/ only.

### Follow-up 2 — Hub-side bisect + fix (substantial task)

Bisect commits between 0.11.472 → 0.11.473 (single commit on the build.rs
counter, but possibly multiple code commits if version numbering is
nondeterministic vs `git describe`). Find the change that introduces the
wedge in `channel info`'s topic-walk path. Likely candidates per CLAUDE.md
fabric: `crates/termlink-hub/` topic-state serialization or rpc dispatcher.

Scope estimate: 1 inception → 1-2 build tasks, ~2-3 sessions.

## Decision criteria for human review

- Is the operator-side cache (follow-up 1) acceptable as the immediate
  fix while follow-up 2 lands?
- Should hubs running 0.11.473 be downgraded to 0.11.472 in the meantime?
  (`fleet-deploy-binary.sh` supports this; 0.11.472 is what .107 runs and
  is proven reliable.)
- Or accept the flake until follow-up 2 produces a fix?

## Disproven theories

- ~~Topic bloat causes slow subscribe latency.~~ Subscribe is O(1) on
  cursor; bloat is irrelevant on the read side. The bloat triggers the
  bug indirectly by feeding more topic-walk work to a buggy `channel
  info` code path, but the bloat itself is harmless.
- ~~`agent-listeners.sh` needs a windowing fix.~~ T-1844 already added
  the correct windowing.
- ~~The .122 hub is uniquely broken.~~ It's a fleet-wide bug on every
  0.11.473 hub.

## Dialogue Log

(No human dialogue this round — pure agent spike. Findings posted here
for review.)
