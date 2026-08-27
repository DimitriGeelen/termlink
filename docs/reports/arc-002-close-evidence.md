# arc-002 (arc-substrate-fitness) — close evidence

Captured 2026-08-27T12:58:09Z by T-2840. Evidence for the two halves of the arc's
headline_mechanic, gathered at the wire, not from task checkboxes.

## Half 1 — "presence shows only LIVE agents, no multi-day-stale ghosts"

```
$ bash scripts/agent-listeners-fleet.sh
Fleet agent-presence — 3 hubs scanned, 1 failed, 2 listeners (2 LIVE / 0 STALE / 0 OFFLINE)
Failed hubs:
  - laptop-141 (192.168.10.141:9100): Error: Hub rpc_call (channel.list) failed  Caused by:     0: I/O error: No route to host (os error 113)     1: No route to host (os error 113) agent-listeners: channel info failed (exit=1)

AGENT_ID                 ROLE       STATUS   AGE_S   HUB                    LISTEN_TOPICS
ring20-dashboard-agent   listener   LIVE     11      192.168.10.121:9100    dm:33df8954b2a9b70d:*,agent-chat-arc
ring20-concierge         claude-code LIVE     22      192.168.10.122:9100    dm:ring20-concierge:*,agent-chat-arc
```

```
$ termlink agent find-idle --json
{
  "idle": [],
  "ok": true
}
```

## Half 2 — "a discarded post surfaces in a recoverable dead-letter, not vanishing"

```
$ bash scripts/check-dead-letter-freshness.sh
check-dead-letter: healthy (0 pending, 0 dead-lettered)
exit=0
```

```
$ termlink channel queue-status --json
{
  "cap": 1000,
  "dead_letter_rows": [],
  "dead_letters": 0,
  "exists": true,
```

## Supporting: the retention bound R2 shipped (T-2245) is live

```
$ bash scripts/check-topic-growth-freshness.sh
topic-growth canary: healthy — no watched topic over 5000 records
exit=0
```

## How to read this, including where it is weak

**Half 1 is directly evidenced.** `2 LIVE / 0 STALE / 0 OFFLINE` is the mechanic firing
at the wire: the presence view returns only agents heartbeating now, with no multi-day
ghosts. That is the property T-2249/T-2245 shipped, observed rather than asserted. The
`.141` line is a genuinely unreachable host (no route, 13+ days) and is reported as a
failed hub, not silently folded into the LIVE count — which is itself the behaviour the
arc wanted.

**Half 2 is evidenced more weakly, and the difference matters.** `dead_letters: 0` proves
the dead-letter sink exists, is readable, and currently holds nothing. It does NOT prove
that a post rejected `POISON_THRESHOLD` times today would land there rather than vanish —
that would need an injected poison post, which is a live write to the substrate and not
something to do casually on a working hub. Read this half as "the recoverable sink is
present and observable", not as "loss-to-recovery was re-demonstrated end to end".

Whoever closes the arc should decide whether half 2 wants a demonstration before closing,
or whether T-2243's original proof plus a present, readable sink is sufficient. That is a
judgement, which is why this file stops at evidence and does not make it.

**T-2250 (R5 telemetry-plane design) is still open and parked** (`captured`, horizon
`later`). Closing the arc means deciding that R5 is out of scope for it, not that R5 is
done. The other 7 members are `work-completed`.
