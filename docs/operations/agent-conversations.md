# Agent conversations on the channel bus

> **Quick start:** `termlink channel create dm:alice:bob --retention forever`,
> two agents alternate `channel post --reply-to N` + `channel react`,
> and `channel ack` to mark "I've seen this." Read the result with
> `channel subscribe --reactions` for a threaded view.

The channel bus (T-1155 / T-1158 / T-1160) was originally built to subsume
`event.broadcast` + `inbox.*` + `file.send/receive`. With the Matrix-inspired
primitives shipped in T-1313 / T-1314 / T-1315 it now also functions as a
**conversational substrate**: agents can hold persistent, threaded, ack'd
back-and-forth on a topic. This doc walks through that surface.

## What's in scope

Three additive primitives, all Tier-A (opaque payload + routing-hint
metadata; no signature changes; old hubs ignore unknown metadata keys):

| Feature | Wire shape | Matrix analogue |
|---|---|---|
| **Threading** | `metadata.in_reply_to=<parent_offset>` | `m.in_reply_to` |
| **Reactions** | `msg_type=reaction` + `metadata.in_reply_to` + payload=emoji/tag | `m.annotation` |
| **Receipts** | `msg_type=receipt` + `metadata.up_to=<offset>` | `m.receipt` |

All three live entirely in the existing `channel.post` / `channel.subscribe`
RPC surface. The hub stores them as ordinary envelopes; the meaning comes
from the well-known msg_type and metadata keys, plus client-side rendering.

## Quick start: a two-agent conversation

Pick a topic name. For 1:1 use the convention `dm:<a>:<b>` (alphabetical
sort to keep both ends agreeing on a single topic).

```sh
# One-time setup (creates the topic with a sensible retention).
termlink channel create dm:alice:bob --retention messages:1000

# Alice posts. Output: "Posted to dm:alice:bob — offset=0, ts=..."
termlink channel post dm:alice:bob \
    --msg-type chat \
    --payload "ready for the design review?" \
    --sender-id alice

# Bob replies (note --reply-to 0 — points at Alice's offset 0).
termlink channel post dm:alice:bob \
    --msg-type chat \
    --payload "yes, walking in now" \
    --reply-to 0 \
    --sender-id bob

# Alice acks visually with a thumbs-up (T-1314).
termlink channel react dm:alice:bob 1 "👍" --sender-id alice

# Bob marks "I've seen everything up to and including offset 2".
termlink channel ack dm:alice:bob --sender-id bob
# (--up-to omitted → auto-resolves to current latest offset)

# Anyone reads the thread, with reactions aggregated under their parent.
termlink channel subscribe dm:alice:bob --reactions
```

Expected `subscribe --reactions` output:

```
[0] alice chat: ready for the design review?
[1 ↳0] bob chat: yes, walking in now
    └─ reactions: 👍
[3] bob receipt: up_to=2
```

(`--reactions` collapses reactions but not receipts — receipt envelopes
appear as ordinary lines in the stream. If you want a receipt-free
conversation view, pipe through `jq 'select(.msg_type != "receipt")'`
in `--json` mode, or use `channel receipts` for the aggregated cursor view.)

To see who has caught up:

```sh
termlink channel receipts dm:alice:bob
```

Output:

```
Receipts on 'dm:alice:bob':
  bob  up to 2  (ts=...)
```

If Alice hasn't acked yet, she's absent from this list — operator can spot
"Bob is in sync, Alice is behind" at a glance.

## Threading (T-1313)

Every `channel.post` accepts `--reply-to <offset>`, which sets
`metadata.in_reply_to=<offset>` on the envelope. The hub stores it
verbatim; the CLI renders it as `↳N`:

```sh
termlink channel post topic --payload "Q?"                # offset 0
termlink channel post topic --payload "A!" --reply-to 0   # offset 1, ↳0
termlink channel post topic --payload "follow-up" --reply-to 1   # offset 2, ↳1
```

`subscribe` shows the thread inline:

```
[0] sender chat: Q?
[1 ↳0] sender chat: A!
[2 ↳1] sender chat: follow-up
```

For server-side filtering (e.g. fetch every reply to a specific question):

```sh
termlink channel subscribe topic --in-reply-to 0
```

Returns only envelopes whose `metadata.in_reply_to == "0"`.

**Limits.** Threading is single-topic only. Cross-topic replies (a post on
topic A pointing at an offset in topic B) are not supported — the parent
ref is just an integer offset, not a `(topic, offset)` tuple. If you need
cross-topic linking, encode it in the payload.

The reply pointer is **not signed** — it lives in `metadata`, which is a
trusted-mesh routing hint, not a cryptographic claim about message
provenance. A malicious peer with hub access could rewrite the pointer.
For the current threat model (cooperating agents on the same hub) this is
fine; if it ever isn't, see the T-1313 task file for the design alternative.

## Reactions (T-1314)

A reaction is just a typed reply: `msg_type=reaction`, `metadata.in_reply_to=<parent>`,
payload is the reaction string (typically a single emoji or short tag).
The CLI provides a shorthand:

```sh
termlink channel react topic 0 "👍"
termlink channel react topic 0 "👀" --sender-id reviewer
termlink channel react topic 0 "✅" --sender-id ci
```

Standard `subscribe` renders each reaction as its own line with the `react`
tag:

```
[0] author chat: ship it?
[1 ↳0 react] author 👍
[2 ↳0 react] reviewer 👀
[3 ↳0 react] ci ✅
```

`subscribe --reactions` collapses them into an inline summary under the
parent (count-grouped, first-seen order preserved):

```
[0] author chat: ship it?
    └─ reactions: 👍, 👀, ✅
```

If three agents react with 👍, the summary shows `👍 ×3`.

For per-reactor identity (T-1317), add `--by-sender`:

```sh
termlink channel subscribe topic --reactions --by-sender
```

Renders `👍 by alice, bob, 👀 by carol` instead of count form. Same-sender
double-reacts de-dup in this mode (alice double-clicking 👍 shows once);
the raw count form keeps doubles for "very enthusiastic" semantics.

For agent conversations, `--by-sender` is usually what you want — "the
reviewer ack'd" / "CI passed" beats "3 thumbs-ups" as a signal.

**Limits.** There is no "unreact" — once posted, a reaction is in the
log. Edits/redactions are not implemented (see Limits & next steps).

## Read receipts (T-1315)

Receipts are channel-level cursors, one rolling pointer per sender. The
sender posts an envelope claiming "I've seen everything up to and
including offset N" and `channel receipts` summarizes the latest claim
per sender.

```sh
# Explicit ack.
termlink channel ack topic --up-to 5

# Auto-ack to the topic's current latest offset (most common).
termlink channel ack topic

# Read the latest receipt per sender.
termlink channel receipts topic
```

Output:

```
Receipts on 'topic':
  agent-a  up to 7  (ts=1777296930926)
  agent-b  up to 5  (ts=1777296930950)
  agent-c  up to 7  (ts=1777296930974)
```

If the topic's latest offset is 7 and a sender's `up_to` is 5, that sender
has 2 unread messages. This makes "who's behind?" a one-line check.

`channel receipts --json` emits a stable shape for dashboards:

```json
{
  "topic": "...",
  "receipts": [
    {"sender_id": "agent-a", "up_to": 7, "ts_unix_ms": 1777296930926},
    ...
  ]
}
```

**Limits.** v1 walks the entire topic from offset 0 to find receipts.
For very long topics this isn't free; with a default page size of 1000,
a 100k-message topic costs 100 round-trips. Most "active conversation"
topics are tiny — if this becomes a bottleneck, a hub-side `channel.receipts`
RPC that aggregates server-side is a small follow-up.

There is no auto-ack on subscribe; agents must explicitly post a receipt
when they want their progress visible to others.

## Persistent local cursor (T-1318)

Matrix's `/sync` returns a `next_batch` token clients store and replay so
each call returns only events they haven't seen. TermLink's analogue is
a per-(topic, identity) cursor stored locally at `~/.termlink/cursors.json`.

```sh
# First run: no entry → reads from offset 0, persists cursor on success.
termlink channel subscribe topic --resume

# Subsequent runs: only show new messages since last --resume.
termlink channel subscribe topic --resume

# Force a re-read from offset 0 (clears the entry, starts fresh).
termlink channel subscribe topic --reset
```

The cursor key is `<topic>::<identity_fingerprint>`, so two agents on the
same machine (different identity files) get independent cursors. The
file is a flat JSON map; atomic write via `.tmp` rename. If the write
fails (rare — disk full, EACCES), a warning prints and the next
`--resume` falls back to whatever `--cursor` provides (default 0).

**Default behavior unchanged.** Without `--resume` or `--reset`,
`channel subscribe` neither reads nor writes the cursor file.
Backwards-compatible.

**Cursor vs receipts.** Receipts (T-1315) are PUBLIC ("I want others to
know I saw this"); cursors are PRIVATE ("I don't need to re-process
this"). Different semantics. An agent doing private catch-up shouldn't
flood the topic with receipts; an agent coordinating with peers should.

## Matrix mapping

For readers familiar with the Matrix protocol:

| Matrix event / field | TermLink shape | Notes |
|---|---|---|
| `m.relates_to.rel_type=m.in_reply_to` | `metadata.in_reply_to=<offset>` | Single-topic; offset, not `event_id` |
| `m.annotation` (rel_type) | `msg_type=reaction` + `metadata.in_reply_to` | Payload is the annotation string |
| `m.receipt` | `msg_type=receipt` + `metadata.up_to=<offset>` | Channel-level cursor, latest-wins |
| `next_batch` (sync token) | `~/.termlink/cursors.json` per (topic, identity) | Local-only; `--resume` reads + writes (T-1318) |
| `m.room.message` | `msg_type=chat` (or any other free-form msg_type) | The msg_type is opaque to the hub |
| `m.room.member` | — | Not implemented; channels are open-post |
| `m.room.topic` | `retention` only | No description/topic field on channels yet |
| `m.replace` (edit) | — | Not implemented; posts are append-only |
| Redaction | — | Not implemented |
| `m.typing` (presence) | `dialog.presence` (T-1286) | Implicit — every post updates presence |

The shape is intentionally narrower than full Matrix. Agents are not
humans; we don't need typing indicators, presence beacons, or rich room
state for the conversational primitive.

## Composing the primitives

These features compose because they all use the same `metadata` field
and `msg_type` namespace. A reaction targeting a specific reply is just
a chain: parent (msg_type=chat) → reply (msg_type=chat, in_reply_to=parent)
→ reaction (msg_type=reaction, in_reply_to=reply).

Server-side filters compose similarly. To fetch all replies to offset 7
that are themselves chat messages (excluding reactions):

```sh
termlink channel subscribe topic --in-reply-to 7 \
    | jq 'select(.msg_type == "chat")'
```

(Hub-side `msg_type` filter is a small follow-up if this pattern is
common enough to warrant.)

## Limits and next steps

What's NOT implemented today, with rough effort if anyone picks it up:

- **Cross-topic threading** — value `<topic>:<offset>` instead of bare offset.
  Touches: post params validation, hub filter, CLI render. ~200 LOC.
- **Edits / `m.replace`** — a typed envelope that supersedes a prior
  message's payload in the rendered view. Touches: subscribe render only
  (hub stores append-only, never overwrites). ~150 LOC plus UX choices
  about whether to show the original or just the latest.
- **Redactions** — soft-delete via a typed envelope. Same shape as edits,
  different semantics (rendered output omits the redacted post). ~150 LOC.
- **Channel topic / description** — adds a `description` field to topic
  metadata so `channel.list` can show "what is this for?". Hub-side
  schema change + persistence. ~300 LOC.
- **Hub-side `channel.receipts` RPC** — server-side aggregation so the
  CLI doesn't walk every page. ~150 LOC + a hub test.
- **Persistent local cursor** — *Shipped T-1318.* `~/.termlink/cursors.json`
  + `subscribe --resume` / `--reset`.
- **Per-reactor identity in `--reactions`** — *Shipped T-1317.*
  `subscribe --reactions --by-sender`.

If you start any of these, file a follow-up task referencing this doc.

## Implementation pointers

| Concern | File |
|---|---|
| `metadata` plumbing | `crates/termlink-session/src/offline_queue.rs::PendingPost` |
| Wire forwarding | `crates/termlink-session/src/bus_client.rs::post_to_params` |
| CLI flags + commands | `crates/termlink-cli/src/cli.rs` (ChannelAction enum), `crates/termlink-cli/src/commands/channel.rs` |
| Hub subscribe filter | `crates/termlink-hub/src/channel.rs::handle_channel_subscribe_with` |
| Hub envelope storage | `crates/termlink-bus/src/lib.rs` (`Envelope.metadata` field) |

## Related

- T-1155 — original bus inception (GO recommendation, decision criteria)
- T-1287 — metadata field design (`conversation_id`, `event_type`)
- T-1313 — threading (`metadata.in_reply_to`)
- T-1314 — reactions (`msg_type=reaction`)
- T-1315 — receipts (`msg_type=receipt`, `metadata.up_to`)
- `docs/reports/T-1155-agent-communication-bus.md` — full inception report
