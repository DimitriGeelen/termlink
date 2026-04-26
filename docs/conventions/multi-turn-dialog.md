# Multi-Turn Dialog Convention (T-243)

How agents compose multi-turn conversations over `channel.*` + `dialog.presence` without a new typed namespace.

**Source:** T-243 inception research (`docs/reports/T-243-multi-turn-agent-conversation-inception.md`) — Agent C's minimal-surface path, with Agent B's heartbeat-as-infrastructure framing folded in.

## Why this exists

Pickup envelopes are a messaging channel; TermLink sessions are an execution channel (PL-012). Neither is a multi-turn dialog channel. Without a convention for sustained conversation:

- Agents fall back to "send and wait" — fire-and-forget patterns that lose interactivity
- Long LLM-tool-use turns (>30s) hit silent timeouts because no keepalive flows
- Multi-agent coordination becomes ad-hoc: every consumer reinvents conversation_id tagging, presence tracking, and resume-after-disconnect logic

This convention sits on the building blocks already in the protocol — `channel.post`, `channel.subscribe`, `dialog.presence`, T-1287 metadata, T-1289 long-poll — and gives every consumer the same playbook.

## Building blocks

| Primitive | Purpose | Task |
|-----------|---------|------|
| `channel.post` with `metadata.conversation_id` | Tag a post as belonging to a specific dialog | T-1287 |
| `channel.post` with `metadata.event_type` | Tell subscribers what kind of post this is (routing hint, not enforcement) | T-1287 |
| `channel.subscribe` with `conversation_id` filter | Receive only posts in a specific dialog | T-1287 |
| `channel.subscribe` with `timeout_ms` (long-poll) | Block until a fresh post arrives or timeout — push-like wake latency | T-1289 |
| `dialog.presence(conversation_id)` | "Who has been seen in this dialog?" — passive hub-side tracker | T-1286 |
| `Bus::oldest_offset` (gap detection) | Subscriber detects when records were swept while disconnected; can reset cursor or surface a gap warning | T-1285 |

The hub is **routing-only** for these conventions — metadata is unsigned, not enforced, not validated. Agents agree on the catalog; the hub forwards bytes.

## `metadata.conversation_id`

A free-form string. Hub doesn't parse, validate, or canonicalize it. Subscribers filter exact-match.

**Recommended format:** `<initiator-agent-id>:<short-uuid>` — globally unique, easy to grep, identifies the originating agent.

Example: `alice:b3f2c1` or `orchestrator-1:t243-2026-04-26`.

**Lifecycle:** lasts as long as the conversation. There is no hub-side "open dialog / close dialog" — when nobody posts on a `conversation_id` for long enough, presence ages out by the operational definition the consumer chooses.

## `metadata.event_type` catalog

Five well-known values. Senders pick one; subscribers route or filter on it.

### `turn`

A meaningful payload — the actual content of the dialog (a message, a request, an answer).

- **Emitted by:** the agent currently producing the next utterance.
- **When:** once per logical turn, when the work for this turn is complete.
- **Subscribers do:** read `payload`, advance dialog state, decide whether to respond.
- **Example:** an agent finishes generating an answer and posts it as `event_type=turn`.

### `typing`

The "I'm still working" heartbeat. Lightweight — empty or near-empty payload. **Load-bearing**, not cosmetic.

- **Emitted by:** the agent currently working, every ~5s during processing.
- **When:** while a turn is in progress and likely to take longer than the subscriber's `timeout_ms` window.
- **Subscribers do:** wake from long-poll, observe activity, restart their `subscribe(timeout_ms=N)` cycle. The heartbeat resets the implicit timeout clock.
- **Why this matters:** without it, a 30s default `timeout_ms` kills any LLM turn that calls a tool. The typing post IS the keepalive — typing indicators emerge as a side effect, not the goal.

### `receipt`

"I saw your turn." Acknowledgement that doesn't carry new content.

- **Emitted by:** any subscriber that has consumed an upstream `turn`.
- **When:** optional — useful for confirming delivery in flows where the sender needs to know it landed before proceeding.
- **Subscribers do:** update their picture of who has caught up to which offset.

### `presence`

Explicit "I am here" announcement. Most consumers don't need to emit this — `dialog.presence` derives it passively from any post with `metadata.conversation_id`. Use it when an agent wants to mark itself active without a content turn (e.g., joining mid-conversation as an observer).

- **Emitted by:** an agent declaring itself a participant.
- **When:** rare — typically when joining a dialog without a `turn` to contribute yet.
- **Subscribers do:** update their `dialog.presence` view; or query `dialog.presence(cid)` directly.

### `member`

Membership change events — agent joins or leaves the dialog.

- **Emitted by:** the joining/leaving agent (or a coordinator).
- **When:** at the boundary of dialog membership.
- **Subscribers do:** track who's currently a participant. Optional `payload` carries `{action: "join"|"leave", agent_id}`.

## Worked example: 2-agent dialog

Alice asks Bob a question. Bob's answer takes ~10s. Alice keeps her subscription alive via long-poll while Bob heartbeats.

**Setup.** They agree on `conversation_id = "alice:t243-demo"`. The transport channel is `inbox:bob` (Bob's existing inbox topic) for inbound, `inbox:alice` for outbound.

**Step 1 — Alice posts a turn (her question):**

```json
{
  "method": "channel.post",
  "params": {
    "topic": "inbox:bob",
    "msg_type": "dialog",
    "payload_b64": "<base64 of {\"question\": \"What is 2+2?\"}>",
    "ts": 1714123456000,
    "sender_id": "alice",
    "sender_pubkey_hex": "...",
    "signature_hex": "...",
    "metadata": {
      "conversation_id": "alice:t243-demo",
      "event_type": "turn"
    }
  }
}
```

**Step 2 — Alice subscribes for Bob's reply with long-poll + filter:**

```json
{
  "method": "channel.subscribe",
  "params": {
    "topic": "inbox:alice",
    "cursor": 0,
    "timeout_ms": 30000,
    "conversation_id": "alice:t243-demo"
  }
}
```

This blocks for up to 30s. Wakes immediately on any post matching `conversation_id=alice:t243-demo`.

**Step 3 — Bob receives and starts heartbeating** (every ~5s while computing):

```json
{
  "method": "channel.post",
  "params": {
    "topic": "inbox:alice",
    "msg_type": "dialog",
    "payload_b64": "",
    "ts": 1714123461000,
    "sender_id": "bob",
    "sender_pubkey_hex": "...",
    "signature_hex": "...",
    "metadata": {
      "conversation_id": "alice:t243-demo",
      "event_type": "typing"
    }
  }
}
```

Alice's long-poll wakes (typing post matches her filter), her client sees `event_type=typing`, ignores the empty payload, and **immediately re-subscribes for another 30s**. The keepalive prevents her client from giving up.

**Step 4 — Bob posts the answer turn:**

```json
{
  "method": "channel.post",
  "params": {
    "topic": "inbox:alice",
    "msg_type": "dialog",
    "payload_b64": "<base64 of {\"answer\": \"4\"}>",
    "ts": 1714123466000,
    "sender_id": "bob",
    "sender_pubkey_hex": "...",
    "signature_hex": "...",
    "metadata": {
      "conversation_id": "alice:t243-demo",
      "event_type": "turn"
    }
  }
}
```

Alice's pending long-poll wakes. She reads the payload. Dialog complete.

## Worked example: N-agent collaboration

Three agents (alice, bob, carol) collaborate on a task. Alice is coordinator; bob and carol are workers.

**Conversation:** `conversation_id = "alice:roundtable-1"` on broadcast topic `dialog:roundtable-1` (bob + carol both subscribe with the cid filter).

**Discovering presence.** Alice queries who has spoken in the conversation so far:

```json
{
  "method": "dialog.presence",
  "params": { "conversation_id": "alice:roundtable-1" }
}
```

Response:

```json
{
  "presences": [
    { "agent_id": "alice", "last_seen_ms": 1714123466000 },
    { "agent_id": "bob",   "last_seen_ms": 1714123470000 },
    { "agent_id": "carol", "last_seen_ms": 1714123455000 }
  ]
}
```

Alice sees carol's last activity was 15s ago — older than alice's "still active" threshold. She decides to nudge carol with a `member` post:

```json
{
  "method": "channel.post",
  "params": {
    "topic": "dialog:roundtable-1",
    "msg_type": "dialog",
    "payload_b64": "<base64 of {\"action\": \"ping\", \"target\": \"carol\"}>",
    "ts": 1714123471000,
    "sender_id": "alice",
    "metadata": {
      "conversation_id": "alice:roundtable-1",
      "event_type": "member"
    }
  }
}
```

If carol is alive she replies (a `typing` heartbeat at minimum). If she's silent past N retries, alice can choose to drop her from the active set. The hub does not enforce membership — alice's behavioral model is the source of truth.

## Resume after disconnect

If a subscriber's connection drops mid-conversation, on reconnect:

1. **Resubscribe** with the last-known `next_cursor` and the `conversation_id` filter.
2. **Check for gaps.** If `cursor` is below the topic's `oldest_offset` (T-1285), the bus has swept records that were live when you last subscribed. You missed messages. The convention: log a gap warning, advance cursor to `oldest_offset`, accept that some history is lost.
3. **Long-poll resumes normally** — the per-topic `tokio::sync::Notify` (T-1289) is independent of subscriber identity.

A subscriber that stays disconnected long enough to lose retention is a subscriber that needs out-of-band catchup (e.g., a coordinator can replay key turns). The protocol does not promise replay — only "pick up where you left off, if your spot still exists."

## Heartbeat as infrastructure

This convention treats `event_type=typing` posts as **load-bearing keepalive**, not visual decoration. The chain:

1. Subscriber calls `channel.subscribe(timeout_ms=30000, conversation_id=cid)`. Blocks for up to 30s.
2. Sender is mid-work, every ~5s emits an empty-payload `event_type=typing` post.
3. Per-topic `Notify` wakes the subscriber's long-poll within ~ms of each post.
4. Subscriber sees the typing post, ignores its content, re-subscribes for another 30s.
5. Keepalive cycle continues until the sender posts an `event_type=turn` (or the dialog dies).

Without typing posts, step 2 doesn't happen, step 1 hits the 30s wall, the subscriber gives up. With them, the dialog stays alive across arbitrary processing time.

The framing comes from T-243 Agent B: *"the typing signal IS the killer feature, but for the wrong reason. Without heartbeat, the 30s default timeout literally kills long LLM-tool-use turns."*

## See also

- `docs/conventions/agent-delegation-events.md` — typed-topic delegation pattern (`task.delegate` / `task.accepted` / `task.completed`)
- `docs/reports/T-243-multi-turn-agent-conversation-inception.md` — full design rationale and 3-agent inception transcript
- T-1285, T-1287, T-1289, T-1286 — the four wedges this convention sits on
