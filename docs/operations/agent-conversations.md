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

For 1:1 conversations the easiest entry point is `channel dm <peer>`
(T-1319) — auto-resolves the canonical topic from your identity
fingerprint plus the peer's, auto-creates it on first use, and opens
read mode (`--resume --reactions`) by default:

```sh
# Alice sends a message (resolves topic + auto-creates).
TERMLINK_IDENTITY_DIR=/tmp/alice termlink channel dm <bob-fingerprint> \
    --send "ready for the design review?"

# Bob replies (note --reply-to threads it).
TERMLINK_IDENTITY_DIR=/tmp/bob termlink channel dm <alice-fingerprint> \
    --send "yes, walking in now" --reply-to 0

# Alice reads (default mode = --resume --reactions; cursor advances).
TERMLINK_IDENTITY_DIR=/tmp/alice termlink channel dm <bob-fingerprint>

# Bob's identity gets an independent cursor.
TERMLINK_IDENTITY_DIR=/tmp/bob termlink channel dm <alice-fingerprint>

# Discover existing DMs for the current identity (T-1320).
termlink channel dm --list
# dm:alice-fp:d1993c2c3ec4...  (peer=alice-fp)
# dm:bob-fp:d1993c2c3ec4...    (peer=bob-fp)
```

The peer identifier should be a stable string both ends agree on —
typically the peer's identity fingerprint (`termlink identity show`).
Topic name is deterministic regardless of which side runs the command:
`channel dm` sorts `[my_fp, peer]` alphabetically and joins as `dm:<a>:<b>`.
`channel dm --list` filters `channel.list` to topics involving the
caller's identity fingerprint, printing the *other* fingerprint as `peer`.

For lower-level use or non-DM conversations, pick a topic name yourself.
For 1:1 use the convention `dm:<a>:<b>` (alphabetical sort to keep both
ends agreeing on a single topic).

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

For a recursive Matrix-style thread view (root + all descendants, indented):

```sh
termlink channel thread topic 0
# [0] alice chat: Q?
#   [1] bob chat: A!
#     [2] alice chat: follow-up
#   [3] carol chat: alt-answer
```

Walks the topic once, builds the parent→children map from
`metadata.in_reply_to`, DFS-renders the subtree from `<root>`. Children are
visited in ascending offset order so output is deterministic. Sub-rooting
(`channel thread topic 1`) renders just that branch (T-1328).

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

## Mentions (T-1325 — Matrix `m.mention`)

Flag a post as relevant to specific recipients. The `--mention <id>` flag is
repeatable on `channel post` and `channel dm --send`; the ids are joined into
`metadata.mentions=<csv>` on the envelope.

```sh
termlink channel post alpha:design --payload "alice please review" --mention alice
termlink channel post alpha:design --payload "pair up on this" --mention alice --mention bob
termlink channel dm <peer-fp> --send "urgent" --mention agent-1
```

Reader-side, the marker shows inline (truncated to 3 ids):

```
[5 @alice] sender chat: alice please review
[6 @alice,bob] sender chat: pair up on this
```

To filter to lines that mention a specific id:

```sh
termlink channel subscribe alpha:design --filter-mentions alice
```

The match is strict: comma-split + whitespace trim, no substring matching.
Use the receiver's stable identifier (typically their identity fingerprint)
on both ends so filtering is deterministic.

**Wildcards (T-1333).** Use `*` for `@room`-style "everyone":

```sh
# Post that tags everyone:
termlink channel post alpha:design --payload "Standup in 5" --mention "*"

# Filter to "any post that mentioned anyone":
termlink channel subscribe alpha:design --filter-mentions "*"
```

A post with `metadata.mentions=*` matches any specific subscriber's
`--filter-mentions <id>` filter (so `--mention "*"` reaches everyone the
way it should). Conversely, `--filter-mentions "*"` matches any post with
a non-empty mentions metadata. Empty mentions still don't match.

## Channel description (T-1323 — Matrix `m.room.topic`)

To attach a free-text description to a topic (think: "this channel is for
the design review thread, archived after Apr-30"):

```sh
termlink channel describe alpha:design "Q2 design review thread (closes 2026-04-30)"
```

Append-only — repeat calls add new records, the latest by ts_ms wins. The
synthesized `channel info <topic>` view surfaces the latest description; raw
`subscribe` shows every description record so you can audit who changed it
and when.

```sh
termlink channel info alpha:design
# Topic: alpha:design
# Retention: messages:100
# Posts: 42
# Description: Q2 design review thread (closes 2026-04-30)
# Senders: 3
#   alice  (24 posts)
#   bob    (12 posts)
#   carol  (6 posts)
# Receipts: 2
#   bob    up to 41  (ts=...)
#   carol  up to 38  (ts=...)
```

Pass `--json` for machine-readable output (T-1324).

**Bounded view (T-1331).** For long-lived topics, `--since <ms>` restricts
description / senders / receipts to records with `ts_unix_ms >= <ms>`. The
total `Posts:` count remains unbounded so you can see the slice in context:

```sh
# Activity in the last hour:
NOW=$(date -u +%s%3N); HOUR_AGO=$((NOW - 3600000))
termlink channel info alpha:design --since $HOUR_AGO
# Posts: 42 (5 since 1777300000000)
# Senders: 2
# ...
```

In `--json` mode, two extra fields appear: `since` (the bound) and
`posts_since` (count in slice).

## Edits (T-1321 — Matrix `m.replace`)

When you need to correct a previously-sent message, emit a new envelope with
`msg_type=edit` and `metadata.replaces=<original-offset>`:

```sh
termlink channel post topic --payload "ready by EOD"          # offset 0
termlink channel edit topic 0 "ready by 4pm — corrected"      # offset 1, edits 0
termlink channel edit topic 0 "ready now"                     # offset 2, edits 0 again
```

Hub stores all three records (append-only). The reader chooses the view:

```sh
# Raw view: full audit trail
termlink channel subscribe topic --limit 50
# [0] alice chat: ready by EOD
# [1] alice edit: ready by 4pm — corrected
# [2] alice edit: ready now

# Collapsed view: latest version against the original offset
termlink channel subscribe topic --limit 50 --collapse-edits
# [0] alice chat: ready now (edited)
```

Composes with `--reactions` (the parent line shows the latest text and the
reactions summary attaches as usual). Old peers that don't pass
`--collapse-edits` see all three records — strictly additive.

`channel dm` opens read mode with `--collapse-edits` on by default, since
that's the natural conversational view; pass the explicit `subscribe`
form if you want to see the full edit history.

## Redactions (T-1322 — Matrix `m.redaction`)

When you need to *retract* a message (vs edit it), use redaction:

```sh
termlink channel post topic --payload "Slack token: xoxb-..."        # offset 4
termlink channel redact topic 4 --reason "leaked secret"
```

The hub keeps the original record (append-only audit trail). Two render
modes:

```sh
# Default: redaction shown explicitly so the audit trail is visible
termlink channel subscribe topic --limit 50
# [4] alice chat: Slack token: xoxb-...
# [5 redact] alice → offset 4 (reason: leaked secret)

# --hide-redacted: clean view that suppresses both target AND the redaction envelope
termlink channel subscribe topic --limit 50 --hide-redacted
# (offset 4 and 5 both gone)
```

**Caveat (trusted-mesh threat model):** the redaction envelope is just a
metadata pointer. The original payload remains in the hub's storage. If the
content was sensitive (a leaked secret, PII), redaction is *not* a delete —
it's a "please don't render this" hint that compliant readers honor. For
true purges you need to truncate the topic or rewrite the channel.

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

**Removing a reaction (T-1330).** Matrix `m.annotation` removal: emit
an `m.redaction` against the reaction event. The CLI provides
`channel react <topic> <parent> <reaction> --remove` which walks the
topic, finds the latest reaction by this identity matching
(parent, payload), and emits a redaction targeting that offset:

```sh
termlink channel react topic 0 "👍" --remove
```

`subscribe --reactions` aggregation always skips redacted reactions
(regardless of `--hide-redacted`) so the removed reaction simply
disappears from the inline summary. Errors with a clear message if no
matching reaction exists for this identity.

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

**Server-side aggregation (T-1329).** `channel receipts` first calls the
hub-side `channel.receipts` RPC, which walks the topic once on the server
and returns the aggregated `{sender_id, up_to, ts_unix_ms}` list. Old
hubs (pre-T-1329) return JSON-RPC `MethodNotFound` (-32601), at which
point the CLI transparently falls back to the legacy client-side walker
(paginated `channel.subscribe` from offset 0). Output is identical
between the two paths — operators don't need to know which fired.

**Quick reply (T-1334).** For interactive use, `channel reply` auto-resolves
the latest content offset and threads to it — skipping reactions, edits,
redactions, receipts, and topic_metadata when picking the parent:

```sh
termlink channel reply alpha:design "got it"
# Posted to alpha:design — offset=42, ts=...
# (envelope's metadata.in_reply_to is the latest content offset, e.g. 41)
```

Errors with a clear message when the topic has no content yet. Pass
`--mention <id>` (or `*`) to attach mentions to the reply.

**Unread count (T-1332).** Companion to receipts — "what's new for me?"
`channel unread <topic>` resolves the caller's (or `--sender`'s) latest
receipt up_to, walks the topic past that offset, and counts content
envelopes (excluding meta types: receipt, reaction, redaction, edit,
topic_metadata):

```sh
termlink channel unread dm:alice:bob
# Topic 'dm:alice:bob': 3 unread for bob (first new offset 7, last 9, last receipt up_to=6)
```

`--json` emits `{topic, sender_id, up_to, unread_count, first_unread, last_offset}`
for dashboards. If the sender has never posted a receipt, `up_to=0` —
i.e., everything counts.

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

## End-to-end test

A self-contained walkthrough exercising every feature above with two real
identities lives at `tests/e2e/agent-conversation.sh`. Run it against any
local hub to catch regressions:

```sh
PATH=$PWD/target/release:$PATH bash tests/e2e/agent-conversation.sh
```

The script provisions transient `alice` and `bob` identity dirs under `/tmp`,
walks all 10 steps (canonical DM, send/read, threading, reactions, edits,
redactions, description+info, mentions, receipts, dm --list), and exits 0
on success. Each assertion is content-level (`grep -F` for expected
substrings) so re-runs are safe even though the canonical DM topic
accumulates state across runs.

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
