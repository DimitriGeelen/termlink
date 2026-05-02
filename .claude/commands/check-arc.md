# /check-arc - Pending Chat-Arc DM Inbox (RECEIVE-side companion to /agent-handoff)

Surface pending agent-chat-arc DMs targeted at this agent. Read-side counterpart
to `/agent-handoff`. Walks `dm:<self-fp>:*` topics, computes unread counts via
`termlink channel unread`, renders a Slack-style summary.

**Invocation:** `/check-arc` (no arguments)

The skill is read-only — it does NOT post receipts. To ack after reading, the
caller runs `termlink channel ack <topic>` explicitly per topic.

## Step 1: Resolve self identity fingerprint

Run:

```
termlink whoami --json
```

If the output's `session.identity_fingerprint` is populated, use it.
Otherwise fall back to discovering self-fp from any recent
`agent-chat-arc` post owned by this caller — but if neither path resolves,
**stop** and print:

```
check-arc: cannot resolve self identity_fingerprint.
Either register this session with a fingerprint, or run:
  termlink channel info agent-chat-arc --json | jq '.senders'
to identify yourself manually.
```

## Step 2: Discover dm topics scoped to self

Run:

```
termlink channel list --prefix "dm:" --json
```

Filter results to topics whose name contains `<self-fp>` (canonical
`dm:<sorted_a>:<sorted_b>` form — self appears in either slot).

If zero matching topics: print `check-arc: no DM topics found for <self-fp-shortened>` and exit zero.

## Step 3: For each dm topic, compute unread

Run, per topic:

```
termlink channel unread <topic> --sender <self-fp> --json
```

Capture `unread_count` and `latest_offset`. Skip topics with zero unread.

## Step 4: Render summary

Print, sorted by unread count descending:

```
check-arc: <N> topic(s) with pending DMs

  dm:<peer-short>...    unread=<count>  latest_offset=<offset>
    last sender: <peer-fp-short>
    peek: termlink channel subscribe <topic> --since-offset <last-acked> --limit <count>

  dm:<peer-short-2>...  unread=<count>  latest_offset=<offset>
    ...

To open a conversation interactively:
  termlink channel dm <peer-display-name-or-fp> --resume

To ack a topic after reading:
  termlink channel ack <topic>
```

If zero unread across all dm topics:

```
check-arc: <M> dm topic(s), all read up to current offset.
```

## Step 5: Optional — also surface pending agent-chat-arc broadcasts

The `agent-chat-arc` topic carries fleet-wide announcements (milestones,
rollouts, soak findings). These are not DMs but vendored agents are
expected to read them when picking up.

Run:

```
termlink channel unread agent-chat-arc --sender <self-fp> --json
```

If `unread_count > 0`, append to the summary:

```
+ agent-chat-arc broadcast: unread=<count>  latest_offset=<offset>
    peek: termlink channel subscribe agent-chat-arc --since-offset <last-acked> --limit <count>
    ack: termlink channel ack agent-chat-arc
```

## Rules

- **NEVER** auto-ack topics. The user decides when to mark messages read.
- **NEVER** print the full payload of every unread message — only counts +
  peek commands. The caller decides what to read.
- **NEVER** post a reply from this skill. Use `/agent-handoff` for that.
- **Fail fast** if `termlink` is not on PATH or the local hub is unreachable.
  No silent degradation.

## Smoke test

```
/check-arc
```

Expected: exits zero with either an unread summary or "all read" line.
The agent-chat-arc soak section is appended only if there are unread
broadcasts.
