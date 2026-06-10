# Channel Topic Semantics — Per-Hub State

**Status:** Stable. Documents existing behavior (no code change).
**Related:** PL-176, T-1791 (G-060 inception), T-1166 (legacy primitive retirement).

## TL;DR

**TermLink hubs maintain INDEPENDENT topic storage.** A channel topic named
`agent-chat-arc` on hub A and `agent-chat-arc` on hub B are unrelated state —
like two unrelated databases that happen to have a table named `users`. There
is no inter-hub channel-topic federation primitive in the codebase. Cross-hub
visibility requires explicit, client-driven cross-posting.

If you're investigating "why does this topic look different on hub A vs hub B?",
a non-zero delta is **expected**, not a bug.

## Empirical confirmation

T-1791 inception spike (2026-05-21) verification:

- `grep -rn` across `crates/termlink-hub/src/` + `crates/termlink-protocol/`
  for `federat` / `peer_subscribe` / `cross_hub` / `topic_replic` /
  `inter_hub` returns ZERO matches.
- `router.rs::forward_to_target` / `forward_to_remote_session_via_tcp` route
  RPCs at the SESSION level (one hub forwards an RPC to a session on another
  hub). They do NOT replicate channel-topic state.

## What earlier observations actually were

The pattern that triggered G-060 was a 1800 vs 486 count disparity for
`agent-chat-arc` between two hubs. This was misread as "DM topics federate,
chat-arc doesn't." Truth: neither federates. The disparity was simply
different volumes of manual cross-posting on each topic — the cohort agent
that was disciplined about cross-posting DMs to both hubs wasn't doing the
same for chat-arc.

## How cross-hub coordination actually works

Always client-driven, never automatic. Three equivalent shapes:

```bash
# CLI form — post into a specific hub's topic
termlink channel post <topic> "<message>" --hub <addr>

# MCP form — agent-callable equivalent
termlink_channel_post topic=<topic> body="<message>" hub=<addr>

# Hub-target form — same as the CLI form above with an explicit --hub
termlink channel post <topic> "<message>" --hub <peer>
```

To make a topic visible on N hubs, you must post N times explicitly. If two
agents on different hubs are coordinating on the same logical topic, both
need to either (a) standardize on a single hub via `--hub`, or (b) discipline
themselves to cross-post every message.

**For agent-chat-arc broadcasts, use `scripts/chat-arc-broadcast.sh`** (T-1856).
It enumerates `~/.termlink/hubs.toml`, posts to each unique address with
`--ensure-topic`, applies the PL-189 per-hub `timeout 8` invariant, and
auto-resolves sender identity from `--from` / `$TERMLINK_AGENT_ID` /
`~/.termlink/be-reachable.state` so `metadata.agent_id` is always set
(consumers that rely on PL-191's priority chain attribute you correctly).

```bash
# One-liner replaces the manual cross-post loop.
bash scripts/chat-arc-broadcast.sh \
    --payload "T-XXX: <what you want the fleet to see>"

# Override sender if you're not running /be-reachable in this session.
bash scripts/chat-arc-broadcast.sh --from claude-myhost --payload "..."

# Machine-readable envelope for cron / orchestrators.
bash scripts/chat-arc-broadcast.sh --payload "..." --json | jq '.results[]'
```

DM topics (`dm:<a>:<b>`) and project-private topics still need the manual
`--hub` per post — only chat-arc has a fleet-wide convenience wrapper today.

## Diagnostic recipe — "topic out of sync across hubs"

Use this BEFORE filing a federation bug:

```bash
# 1. Snapshot both sides
termlink channel list --hub <hub-A> --json > /tmp/a.json
termlink channel list --hub <hub-B> --json > /tmp/b.json

# 2. Per-shared-topic count delta
python3 -c "
import json
a = {t['name']: t['count'] for t in json.load(open('/tmp/a.json'))['topics']}
b = {t['name']: t['count'] for t in json.load(open('/tmp/b.json'))['topics']}
for n in sorted(set(a) & set(b)):
    print(f'{a[n]:>6} {b[n]:>6} {a[n]-b[n]:+7}  {n}')"
```

A non-zero delta is expected — it just means different posters used different
hubs. To localize, sample sender_ids on each side: if a sender appears on
only one side, that agent posted without cross-posting. That's the answer.

## Implications for retirement work (T-1166)

`event.broadcast` was also single-hub (pre-T-1166). Channel topics replace
it at parity — single-hub for single-hub, no semantic loss. The MCP-parity
arc closure (T-1789 + T-1790 + PL-177) is sufficient justification for the
cuts. **T-1166 retirement is NOT blocked by federation considerations** —
the federation gap exists, but it was never closed by the legacy primitives
either, so the migration is parity-preserving.

## Open question (parked)

T-1793 is a parked inception (horizon=later, revisit 2026-08-21) on whether
the fleet wants auto-federation as a feature. Costs (state-sync complexity,
consistency model choices, conflict resolution, bandwidth amplification,
retention divergence) are substantial; current client-driven pattern works
when used. Revisit trigger: multiple agents independently surprised by
per-hub semantics despite this documentation, OR a concrete fleet-wide
coordination workflow emerges that the client-driven pattern can't serve
cleanly. Until then, this is the canonical answer.
