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

## Topic stats and search (T-1335 / T-1336)

Two read-only observability commands for live operator triage:

```sh
# Per-topic content/meta breakdown — content envelopes vs. meta types
# (receipt, reaction, redaction, edit, topic_metadata) plus distinct
# senders and timestamp range.
termlink channel list --stats
# → broadcast:global  content=128  meta=0  senders=1  ts=1777131902274..1777308663711

# Same, JSON-formatted for piping to jq.
termlink channel list --stats --json --prefix dm:

# Payload grep across one topic. Default mode: case-insensitive substring.
termlink channel search dm:alice:bob "deadline"
# → [3] alice (chat): the deadline is friday

# Other modes:
termlink channel search dm:alice:bob "DEADLINE" --case-sensitive
termlink channel search dm:alice:bob 'error:\s+\d+' --regex
termlink channel search dm:alice:bob "🧪" --all   # include meta envelopes
termlink channel search dm:alice:bob "..." --limit 5 --json
```

**Tier-A:** the search pattern stays client-side — the hub never sees
the query, so secrets-in-payloads don't leak via search.

## Inbox views (T-1338 / T-1339 / T-1341)

Three commands that compose the primitives above into operator-friendly
inbox views:

```sh
# DM inbox — every DM topic the caller participates in, with per-topic
# unread count (delta from the caller's last receipt). Sorts unread-
# first.
termlink channel dm --list --unread
# → dm:alice:bob  (peer=bob)  unread=3  first=42
#   dm:alice:carol  (peer=carol)  unread=0  first=—

# Cross-topic @-mentions inbox — every envelope that mentions the
# caller (or `*` / @room) across every topic. Groups by topic.
termlink channel mentions
# Default --for is the caller's identity; switch to query someone else:
termlink channel mentions --for bob --prefix dm:

# Per-topic membership list — distinct senders with post-count and
# first/last activity ts. Lighter than `channel info`.
termlink channel members dm:alice:bob
# → alice  posts=12  first=1777313661544  last=1777316598002
#   bob    posts=8   first=1777313661548  last=1777316597927

# --include-meta counts reactions/edits/redactions/receipts too.
termlink channel members dm:alice:bob --include-meta --json
```

## Receipt anchoring (T-1337)

`channel ack` accepts either an explicit offset or a timestamp anchor:

```sh
# Mark "everything since 10 minutes ago" as seen — no need to look up
# the offset first.
TEN_MIN_AGO=$(python3 -c 'import time; print(int(time.time()*1000) - 600_000)')
termlink channel ack dm:alice:bob --since "$TEN_MIN_AGO"

# --up-to and --since are mutually exclusive (clap-enforced):
# error: the argument '--up-to <UP_TO>' cannot be used with '--since <MS>'
```

A future anchor (no envelope satisfies `ts >= since`) emits a friendly
hint with the topic's actual latest ts and the gap in ms.

## Thread navigation (T-1340)

`channel thread <topic> <offset>` walks a reply tree DOWN from a root.
T-1340 added the inverse:

```sh
# Trace the reply chain UP from a leaf back to the conversation's root.
# Output is indented by depth in root→leaf order.
termlink channel ancestors dm:alice:bob 17
# → [3] bob (chat): we should ship the patch
#     [9] alice (chat): agreed — but let's bench it first
#       [17] bob (chat): bench is in CI — passing
```

Cycle-safe: caps recursion at depth 1024. `--json` returns the chain
as `{topic, leaf, ancestors: [...]}`.

## Quote rendering (T-1344)

`channel quote <topic> <offset>` renders an envelope inline with its
parent quoted on a preceding `>` line. Useful when you've found a reply
in some other view (e.g. `channel mentions` or a thread walk) and want
context without juggling two terminal windows.

```sh
# Show offset 17 with its parent quoted above it.
termlink channel quote dm:alice:bob 17
# → > [9] alice chat: agreed — but let's bench it first
#   [17] bob chat: bench is in CI — passing
```

For streaming reads, `subscribe --show-parent` does the same render-time
quote inline for every reply in the stream. A one-time topic walk seeds
the parent cache; live envelopes during `--follow` are added as they
arrive. JSON mode (`--show-parent --json`) attaches a `parent` field to
each emitted envelope (`null` when not a reply or the parent is missing
from the cache).

```sh
termlink channel subscribe dm:alice:bob --show-parent
# → [0] alice chat: hi bob, are you there?
#   > [0] alice chat: hi bob, are you there?
#   [1 ↳0] bob chat: yes alice, ready
```

## Pinned events (T-1345 — Matrix `m.room.pinned_events`)

`channel pin <topic> <offset>` emits a `msg_type=pin` envelope with
`metadata.pin_target=<offset>` and `metadata.action=pin`. The current pin
set is computed by walking the topic and applying pin/unpin events in
order — latest action per target wins. `channel pinned <topic>` walks
the topic and renders the active set, sorted by most-recently-pinned
descending.

```sh
# Pin a message.
termlink channel pin dm:alice:bob 0

# Show the current pin set.
termlink channel pinned dm:alice:bob
# → [0] pinned_by=alice ts=1761500000000: hi bob, are you there?

# Remove a pin (latest action wins).
termlink channel pin dm:alice:bob 0 --unpin
```

Append-only — the original posts and the pin/unpin envelopes are never
overwritten. Old peers ignore unknown `msg_type` values, so pin events
are visible only to clients that know to compute the pinned set.

## Render filters (T-1346 / T-1347)

Two render-side flags help focus a noisy topic without changing the
hub-side cursor or filter logic:

- **`--tail <N>`** — show only the last N envelopes after all aggregation
  passes. Pure render-side slice; pagination unchanged. Conflicts with
  `--follow` (tail of an unbounded stream is ill-defined).
- **`--senders <csv>`** — drop envelopes whose `sender_id` is not in the
  comma-separated allowlist. Strict equality (no substring match);
  composes with all other passes.

```sh
# Last 5 envelopes from the topic.
termlink channel subscribe dm:alice:bob --tail 5

# Only what alice said.
termlink channel subscribe dm:alice:bob --senders alice-fingerprint

# Both at once: alice's last 3.
termlink channel subscribe dm:alice:bob --tail 3 --senders alice-fingerprint
```

## Forwarding (T-1348 / T-1349 — Matrix forwarding)

`channel forward <src> <offset> <dst>` copies an envelope from one topic
to another while preserving provenance. The destination envelope keeps
the original `msg_type` and `payload` but is signed by the forwarder
(current identity); metadata records the source for trace-back:

- `forwarded_from=<src_topic>:<offset>`
- `forwarded_sender=<original sender_id>`

```sh
# Bob forwards alice's offset 0 from the DM into a fresh topic.
termlink channel forward dm:alice:bob 0 announcements
```

`subscribe --show-forwards` renders forwarded envelopes with a
`[fwd from <src>:<off> by <orig_sender>]` prefix line so a reader can
spot the origin at a glance:

```sh
termlink channel subscribe announcements --show-forwards
# → [fwd from dm:alice:bob:0 by alice-fingerprint]
#   [0] bob-fingerprint chat: hi bob, are you there?
```

Without the flag, forwards render as normal posts (the metadata is still
in the envelope; readers can opt in to the prefix).

## Typing indicators (T-1351 — Matrix `m.typing`)

`channel typing <topic> --emit [--ttl-ms N]` posts a `msg_type=typing`
envelope with `metadata.expires_at_ms=<now+ttl>`. Default TTL is 30000ms
(Matrix's 30s typing window). `channel typing <topic>` (default = list
mode) walks the topic, drops envelopes whose `expires_at_ms <= now_ms`,
and reports active typers — one row per sender, latest envelope wins.

```sh
# Emit a 30s typing indicator.
termlink channel typing dm:alice:bob --emit
# → Posted to dm:alice:bob — offset=42, ts=…

# Who's currently typing?
termlink channel typing dm:alice:bob
# → alice-fingerprint: typing (expires in 27412ms)
```

Append-only. Old peers see typing envelopes as unknown `msg_type` and
ignore them. The latest-per-sender rule means a fresh emit replaces a
previous one — including replacing an active indicator with a shorter
TTL (the most recent intent wins, even if it's about to expire).

## Windowed reads (T-1343 / T-1352)

`subscribe --since <ms>` and `subscribe --until <ms>` are render-side
timestamp filters that compose into an inclusive `[since, until]`
window. Pagination/cursor behavior is unchanged — both filters drop
envelopes from the printed output only.

```sh
# Yesterday's posts only (since/until in ms-since-epoch).
SINCE=$(date -d 'yesterday 00:00' +%s)000
UNTIL=$(date -d 'today 00:00' +%s)000
termlink channel subscribe dm:alice:bob \
  --since "$SINCE" --until "$UNTIL"
```

Defensive keep: ts-less envelopes (rare; e.g. pre-T-1287 envelopes from
old hubs that didn't carry timestamps) pass through both filters
unchanged. If you only want envelopes that explicitly fall inside the
window, post-filter with `--json | jq` on the `ts` field.

## Per-user bookmarks (T-1354 — Matrix `m.bookmark` flavour)

`channel star` is the per-user analogue of `channel pin`: scoped to the
calling identity, latest action per (sender_id, target) wins. Anyone can
star anyone else's message, but `channel starred` defaults to the
caller's own stars. Use `--all` to see every user's bookmarks.

```sh
# Bookmark a message someone else posted.
termlink channel star dm:alice:bob 5

# Just my stars.
termlink channel starred dm:alice:bob
# → [5] starred_by=alice-fingerprint ts=…: <decoded payload>

# Everyone's stars.
termlink channel starred dm:alice:bob --all

# Remove my star.
termlink channel unstar dm:alice:bob 5
```

Implementation parallels pin: `metadata.star_target=<offset>` +
`metadata.star=true|false`. Aggregator: `compute_starred_set` keys on
`(sender_id, target)`. Matrix mapping: there's no first-class
`m.bookmark` event type — Matrix clients use account-data, but the same
"per-user, per-message marker" pattern applies.

## Polls (T-1355 — Matrix `m.poll`)

Three additive envelope types implement Matrix `m.poll.start` /
`m.poll.response` / `m.poll.end`:

```sh
# Open a poll. The envelope's offset becomes the poll id.
termlink channel poll start dm:alice:bob \
  --question "Lunch?" --option "Pizza" --option "Salad" --option "Sushi"
# → Posted to dm:alice:bob — offset=42, ts=…

# Vote. Re-voting replaces the prior vote (latest action per sender wins).
termlink channel poll vote dm:alice:bob 42 --choice 0
termlink channel poll vote dm:alice:bob 42 --choice 2  # changed mind

# Close. Aggregator drops votes with ts > poll_end.ts.
termlink channel poll end dm:alice:bob 42

# Tallies.
termlink channel poll results dm:alice:bob 42
# → Poll #42 [CLOSED]: Lunch?
#     [0] Pizza — 1 vote(s)
#          · alice-fingerprint
#     [1] Salad — 0 vote(s)
#     [2] Sushi — 1 vote(s)
#          · bob-fingerprint
#   Total votes: 2
```

`--json` returns `{poll_id, question, options:[{label,count,voters}], closed, total_votes}`.

Constraints: at least 2 options required at start; option labels must
not contain `|` (used as the metadata delimiter); out-of-range choice
indices are silently dropped from the tally; votes posted after
`poll_end` are ignored even if their offset is later.

## Activity digest (T-1356)

`channel digest` is "I was away — what did I miss?" — a synthesized
view scoped to a time window. Distinct from `channel info` (no time
window) and `channel stats` (global counts).

```sh
# Last 60 minutes (default).
termlink channel digest dm:alice:bob

# Custom relative window.
termlink channel digest dm:alice:bob --since-mins 5

# Absolute lower bound (epoch ms).
termlink channel digest dm:alice:bob --since 1761500000000
```

Output sections:
- Posts count (content msg_types: `post`/`chat`/`note`)
- Distinct senders
- Forwards in (envelopes carrying `metadata.forwarded_from`)
- Pins added/removed
- Top 3 senders by content-post count
- Top 3 reactions by count
- Last 3 chat snippets in offset order

`--json` returns the structured object with all sections. Pure helper
`compute_digest(envelopes, since_ms)` does the aggregation; ts-less
envelopes are dropped (defensive).

## Cross-topic inbox (T-1358)

`channel inbox` is "what did I miss across every topic I follow?" — a
one-shot summary built on the T-1318 cursor system.

```sh
# Set a cursor by reading with --resume.
termlink channel subscribe dm:alice:bob --limit 10 --resume

# Later, see what's new everywhere.
termlink channel inbox
# → 2 topic(s) with unread content:
#     channel:learnings — 5 unread (latest=42, cursor=37)
#     dm:alice:bob       — 1 unread (latest=11, cursor=10)
```

Read-only; does not advance cursors. `--json` returns
`[{topic, unread, latest, cursor}]`. Topics whose cursor is at-or-ahead
of latest are excluded; topics deleted on the hub are silently dropped
(stale cursor entries persist locally — clean by hand if desired).

Distinct from `channel unread <topic>` which is single-topic and
receipt-based (uses `m.receipt.up_to`, not the local cursor).

## Per-topic emoji breakdown (T-1359)

`channel emoji-stats` walks a topic, tallies every active (non-redacted)
reaction by emoji, and renders sorted-by-count rows. Distinct from
`channel digest` (top 3 only) and from `subscribe --reactions`
(per-message aggregation).

```sh
# Default: all emojis, descending by count.
termlink channel emoji-stats dm:alice:bob
# → Emoji stats for 'dm:alice:bob':
#     👍 ×7 (4 reactor(s))
#     ❤ ×3 (2 reactor(s))
#     🚀 ×1 (1 reactor(s))

# Per-reactor expansion.
termlink channel emoji-stats dm:alice:bob --by-sender
# → 👍 ×7 (4 reactor(s))
#       · alice ×3
#       · bob ×2
#       · carol ×1
#       · dave ×1
#     ...

# Top N only.
termlink channel emoji-stats dm:alice:bob --top 3
```

`--json` returns `[{emoji, count, distinct_reactors, reactors:[{sender_id,count}]}]`.
Redactions targeting reaction envelopes are honoured (the reaction is
excluded from the tally), so `react --remove` is reflected here.

## Read-receipt dashboard (T-1361)

`channel ack-status` is the read-receipt overview: per-sender, where they
are vs. the topic's latest offset, with a "lag" delta. Surfaces members
who never sent a receipt (their `ack` shows `-`).

```sh
# Default: every member.
termlink channel ack-status dm:alice:bob
# → Ack status on 'dm:alice:bob' (latest offset = 12):
#     bob-fingerprint    ack=-   lag=13  ts=0           # never acked
#     carol-fingerprint  ack=4   lag=8   ts=...
#     alice-fingerprint  ack=12  lag=0   ts=...

# Only members who are behind.
termlink channel ack-status dm:alice:bob --pending-only
```

Distinct from `channel receipts` (raw list, no lag) and `channel unread
<topic>` (single-sender count). Note: a sender's own receipt envelope is
itself in the topic, so each `ack` advances `latest` by 1; the lag drifts
back to ≥1 unless caller acks past the receipt envelope.

## Per-sender reaction reverse view (T-1362)

`channel reactions-of` is the inverse of `subscribe --reactions`: instead
of "what reactions exist on each message", it answers "what did sender X
react to". Defaults to the calling identity; `--sender <fp>` overrides.

```sh
# What did I react to?
termlink channel reactions-of dm:alice:bob
# → Reactions by alice-fingerprint on 'dm:alice:bob':
#     🚀 → offset 5 (let's ship it)
#     ❤  → offset 3 (I love this idea)
#     👍 → offset 0 (initial proposal)

# What did bob react to?
termlink channel reactions-of dm:alice:bob --sender bob-fingerprint
```

Active reactions only — redacted reactions (via `react --remove`) are
excluded. Sort: reaction-offset descending (most recent first). `--json`
returns `[{reaction_offset, parent_offset, emoji, parent_payload, ts}]`.

## Quotable snippets (T-1363)

`channel snippet` is "give me a markdown-friendly excerpt I can paste
into a task description, inception, or PR". Walks the topic, finds the
target offset, renders it with N envelopes of context above and below
as a fenced code block.

```sh
termlink channel snippet dm:alice:bob 42 --lines 2 --header
# → From `dm:alice:bob` @ offset 42:
#   ```
#      [40] alice-fp: discussion of approach A
#      [41] bob-fp: prefer approach B because of …
#   >> [42] alice-fp: agreed — let's go with B
#      [43] bob-fp: I'll draft the spec
#      [44] alice-fp: thanks
#   ```
```

Skips meta envelopes (reactions, edits, redactions, receipts,
topic_metadata) so the snippet stays content-focused. Default `--lines
2`. `--json` returns `{topic, target_offset, lines:[{offset, sender,
payload, is_target}]}`. Errors when the target offset doesn't exist as
content.

## Threads index — `channel threads`

`channel threads <topic>` lists every offset that has at least one
non-redacted reply (a thread root) with reply count, distinct
participants, last activity, and a preview. Matrix m.thread room-overview
analog. Sister of `channel thread <topic> <offset>` (T-1328) which drills
into ONE thread; this one is the index. Sorted by last_ts_ms desc.

```sh
termlink channel threads dm:alice:bob

#   Threads on 'dm:alice:bob' (2 roots):
#     [42] replies=3 participants=2 last_ts=1729880000000: shall we ship today?
#     [17] replies=1 participants=2 last_ts=1729810000000: design doc draft
```

`--top N` truncates after sort. `--json` returns `[{root_offset,
reply_count, participants, last_ts_ms, root_payload}]`. Redacted roots
drop the row entirely; redacted replies don't count toward
`reply_count`/`participants`. Roots with zero non-redacted replies don't
appear (use `channel info` if you want a flat envelope listing).

## Edit history — `channel edits-of`

`channel edits-of <topic> <offset>` shows the full edit chain for one
target — the original post followed by every `msg_type=edit` envelope
whose `metadata.replaces=<offset>`, in chronological order. Matrix
m.replace history analog. Useful for audit and forensic correlation when
you only ever see the latest text via `subscribe --collapse-edits`.

```sh
termlink channel edits-of dm:alice:bob 42

#   Edits of offset 42 on 'dm:alice:bob' (2 edits):
#     [original 42 ts=1729880000000 alice-fp] shall we ship today?
#     [edit 51 ts=1729880060000 alice-fp] shall we ship today (just the bus part)?
#     [edit 67 ts=1729880240000 alice-fp] shall we ship today (just the bus part)? Y/N
```

Errors when target offset is missing or is itself redacted. Redacted
edits silently dropped. `--json` returns `{original: {...}, edits:
[{...}, ...]}`. Sort: ts_ms asc with edit offset asc tiebreak.

## Forwards reverse view — `channel forwards-of`

`channel forwards-of <topic> [sender]` is the reverse view of `channel
forward` (T-1346) — list every forward envelope on `<topic>` whose
`sender_id` is the given fingerprint (defaults to caller). Each row:
forward_offset, origin (topic + offset), original sender, payload
preview, ts. Sort: forward_offset desc.

```sh
termlink channel forwards-of dst-topic alice-fp

#   Forwards by alice-fp on 'dst-topic':
#     [forward 12] from src-topic:5 (orig sender bob-fp): hub restart needed
#     [forward 11] from src-topic:3 (orig sender carol-fp): cert rotation alert
```

A subtlety: `channel forward` keeps the *original* `msg_type` (e.g.
"chat") on the destination envelope — only `metadata.forwarded_from` /
`metadata.forwarded_sender` differentiate it from a regular post. The
helper detects forwards via metadata, NOT via msg_type. `--json`
returns `[{forward_offset, origin_topic, origin_offset, origin_sender,
payload, ts}]`.

## Topic dashboard — `channel topic-stats`

`channel topic-stats <topic>` rolls up every counter into a single
read: total envelopes (excluding redacted), distinct senders,
breakdown by msg_type, top-5 senders, distinct + top-5 emojis, thread
roots, active pins (last-write-wins), forwards-in (via metadata),
edits, redactions, and lifetime time span. Like `channel digest`
(T-1356) but unconstrained by time and focused on cumulative totals.

```sh
termlink channel topic-stats dm:alice:bob

#   Topic-stats for 'dm:alice:bob':
#     total envelopes:     7
#     distinct senders:    2
#     thread roots:        1
#     active pins:         1
#     forwards in:         0
#     edits:               1
#     redactions:          0
#     distinct emojis:     1
#     time span (ms):      1729880000000 → 1729880240000  (240000 ms)
#     by msg_type:
#       chat: 3
#       reaction: 2
#       pin: 1
#       edit: 1
#     top senders:
#       alice-fp: 4
#       bob-fp:   3
#     top emojis:
#       👍: 2
```

Naming note: the helper is `compute_full_topic_stats` (the lighter
`TopicStats` / `compute_topic_stats` from T-1335 is reserved for
`channel list --stats` summaries). `--json` returns a structured object
with all counters.

## Replies reverse view — `channel replies-of`

`channel replies-of <topic> [sender]` is the reverse view of `channel
reply` (T-1313) — list every reply envelope on `<topic>` whose
`sender_id` matches the given fingerprint (defaults to caller). Each row
shows reply_offset, parent (offset + sender + payload preview), reply
payload preview, and ts. Sort: reply_offset desc.

```sh
termlink channel replies-of dm:alice:bob bob-fp

#   Replies by bob-fp on 'dm:alice:bob':
#   [reply 5] yes I can take that
#     ↳ to [3] alice-fp: can you cover the deploy?
#   [reply 4] hub looks healthy from here
#     ↳ to [2] alice-fp: hub status?
```

A reply is identified by `metadata.in_reply_to` parsing as a u64 AND
`msg_type != "reaction"`. Reactions also carry `in_reply_to` (T-1314)
but live in `channel reactions-of` (T-1362). Redacted replies are
dropped. If the parent is missing or itself redacted, `parent_sender` /
`parent_payload` come back empty and the row is annotated `(parent
missing or redacted)`. `--json` returns `[{reply_offset, parent_offset,
parent_sender, parent_payload, reply_payload, ts_ms}]`.

## Mentions reverse view — `channel mentions-of`

`channel mentions-of <topic> <user>` complements `channel mentions`
(T-1339): mentions is *cross-topic and locked to the caller fingerprint*,
mentions-of is *per-topic for any user*. Use it to answer "who pinged
Alice in this thread" or audit `@room` traffic.

```sh
termlink channel mentions-of design-channel alice-fp

#   Mentions of alice-fp on 'design-channel':
#     [@ 17] bob-fp (mentions=alice,carol): can you take this?
#     [@ 12] carol-fp (mentions=*): @room standup in 5
```

The match rule is exactly the T-1333 `mentions_match` semantics: literal
CSV equality on parts, with two wildcards — querying `*` matches every
non-empty `mentions` csv ("did this post tag *anyone*?"), and a csv
containing `*` matches every specific user (the post tagged everyone, so
every individual filter fires). Meta envelopes (receipts, reactions,
edits, redactions, topic-metadata) are skipped — only content posts are
considered. Redacted offsets dropped. Sort: mention_offset desc.
`--json` returns `[{mention_offset, sender_id, payload, mentions_csv,
ts_ms}]`.

## Pin audit log — `channel pin-history`

`channel pinned` (T-1345) shows the *current* pin set after collapsing
last-write-wins. `channel pin-history <topic>` is the audit complement:
it preserves *every* pin/unpin envelope as a chronological row, so you
can answer "when did this get pinned, by whom, and was it ever undone?"

```sh
termlink channel pin-history alpha:design

#   Pin history for 'alpha:design':
#     [12] PIN   → [3] by alice-fp: api proposal v3 — comments?
#     [17] UNPIN → [3] by bob-fp:   api proposal v3 — comments?
#     [22] PIN   → [3] by alice-fp: api proposal v3 — comments?
```

Each row carries event_offset (where in the topic timeline the toggle
happened), action ("pin" or "unpin", default + missing both render as
"pin"), target_offset, actor_sender, ts_ms, and best-effort
target_payload (None when the target isn't in the snapshot — redacted,
truncated, or itself a meta envelope). Sort: event_offset asc.
Malformed pin envelopes (missing or non-numeric `pin_target`, missing
metadata) are silently skipped. `--json` returns the array of rows.

## Redaction audit log — `channel redactions`

`channel redact` (T-1322) posts a redaction; `subscribe --hide-redacted`
suppresses redacted bodies; this command renders the *audit trail*. One
row per `msg_type=redaction` envelope with parseable `metadata.redacts`,
chronological by event_offset.

```sh
termlink channel redactions alpha:design

#   Redactions on 'alpha:design':
#     [12] redacts → [3] by alice-fp reason="wrong channel": draft proposal v0
#     [25] redacts → [7] by bob-fp: forgot to anonymise dataset notes
```

Each row carries event_offset (where the redaction was posted),
target_offset, redactor_sender, optional reason (passed through from
`metadata.reason`), ts_ms, and best-effort `target_payload` (None when
the target isn't in the snapshot). Sort: event_offset asc. Symmetric
with `pin-history` (T-1372). `--json` returns the array of rows.

## Per-message reaction rollup — `channel reactions-on`

Three reaction views complete the matrix: `subscribe
--aggregate-reactions` (live, inline next to the parent), `emoji-stats`
(T-1359, topic-wide tally), `reactions-of` (T-1362, per-sender). The
fourth, `reactions-on <topic> <offset>`, is the *per-target* rollup —
"how is THIS message being received?" — equivalent to Matrix's
annotation rollup for a specific event.

```sh
termlink channel reactions-on alpha:design 17

#   Reactions on 'alpha:design':[17]:
#     👍 ×3 — alice-fp, bob-fp, carol-fp
#     🎉 ×2 — alice-fp, dave-fp
#     👀 ×1 — bob-fp
```

Filters: `msg_type=reaction` AND `metadata.in_reply_to == target_offset`
AND not redacted. `count` is total taps (a sender hitting 👍 twice
counts twice), `senders` is the deduplicated sorted set (set semantics
for "who reacted"). Sort: count desc, emoji asc tiebreak. `--json`
returns `[{emoji, count, senders}]`.

## End-to-end test

A self-contained walkthrough exercising every feature above with two real
identities lives at `tests/e2e/agent-conversation.sh`. Run it against any
local hub to catch regressions:

```sh
PATH=$PWD/target/release:$PATH bash tests/e2e/agent-conversation.sh
```

The script provisions transient `alice` and `bob` identity dirs under `/tmp`,
walks all 46 steps (canonical DM, send/read, threading, reactions, edits,
redactions, description+info, mentions, receipts, dm --list, thread view,
react --remove, channel list --stats, search, ack --since, dm --list
--unread, mentions inbox, ancestors, members, subscribe --since, quote,
subscribe --show-parent, pin/pinned, subscribe --tail, subscribe --senders,
forward, subscribe --show-forwards, typing emit/list/expiry, subscribe
--until window, star/unstar/starred per-user bookmarks, poll
start/vote/end/results lifecycle, digest synthesis, cross-topic inbox,
per-topic emoji-stats, ack-status dashboard, reactions-of reverse view,
snippet excerpt, threads index, edits-of history, forwards-of reverse
view, topic-stats dashboard), and exits 0 on success.
Each assertion is content-level (`grep -F` for expected substrings) so
re-runs are safe even though the canonical DM topic accumulates state
across runs.

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
- T-1335 — `channel list --stats` (per-topic content/meta breakdown)
- T-1336 — `channel search` (read-only payload grep)
- T-1337 — `channel ack --since` (timestamp-anchored receipt)
- T-1338 — `channel dm --list --unread` (DM inbox view)
- T-1339 — `channel mentions` (cross-topic @-mentions inbox)
- T-1340 — `channel ancestors` (root→leaf reply chain)
- T-1341 — `channel members` (per-sender activity summary)
- T-1343 — `subscribe --since` (timestamp render filter)
- T-1344 — `channel quote` + `subscribe --show-parent` (parent quoting)
- T-1345 — `channel pin` / `channel pinned` (Matrix `m.room.pinned_events`)
- T-1346 — `subscribe --tail N` (last-N render slice)
- T-1347 — `subscribe --senders <csv>` (per-sender filter)
- T-1348 — `channel forward` (Matrix-style forwarding with provenance)
- T-1349 — `subscribe --show-forwards` (forward provenance prefix)
- T-1351 — `channel typing` (Matrix `m.typing` ephemeral indicator with TTL)
- T-1352 — `subscribe --until <ms>` (upper-bound timestamp filter, pairs with --since)
- T-1354 — `channel star` / `unstar` / `starred` (per-user message bookmarks, Matrix `m.bookmark` flavour)
- T-1355 — `channel poll start` / `vote` / `end` / `results` (Matrix `m.poll` lifecycle)
- T-1356 — `channel digest` (synthesized recent activity, time-windowed)
- T-1358 — `channel inbox` (cross-topic unread summary via T-1318 cursors)
- T-1359 — `channel emoji-stats` (per-topic reaction breakdown)
- T-1361 — `channel ack-status` (read-receipt dashboard with lag)
- T-1362 — `channel reactions-of` (per-sender reaction reverse view)
- T-1363 — `channel snippet` (quotable text excerpt with surrounding context)
- T-1365 — `channel threads` (index of threads with reply counts, Matrix m.thread overview)
- T-1366 — `channel edits-of` (Matrix m.replace history for one target offset)
- T-1367 — `channel forwards-of` (per-sender forwards reverse view, parallel to reactions-of)
- T-1368 — `channel topic-stats` (full per-topic statistics dashboard, distinct from T-1335 list summary)
- T-1370 — `channel replies-of` (reverse view of `channel reply`, mirror of T-1367)
- T-1371 — `channel mentions-of` (per-topic mention reverse view, complements T-1339 cross-topic mentions)
- T-1372 — `channel pin-history` (chronological pin/unpin audit log, complements T-1345 LWW pinned set)
- T-1373 — `channel redactions` (chronological redaction audit log, symmetric with pin-history; mirror of T-1322 redact)
- T-1374 — `channel reactions-on` (per-message reaction rollup, fourth axis after live/topic-wide/per-sender)
- `docs/reports/T-1155-agent-communication-bus.md` — full inception report
