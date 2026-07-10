# /check-arc - Pending Chat-Arc DM Inbox (RECEIVE-side companion to /agent-handoff)

Surface pending agent-chat-arc DMs targeted at this agent. Read-side counterpart
to `/agent-handoff`. Walks `dm:<self-fp>:*` topics, computes unread counts via
`termlink channel unread`, renders a Slack-style summary.

**Invocation:**

- `/check-arc` (no arguments) → **Browse mode** (read-only).
- `/check-arc respond` → **Respond mode** (ack + reply). This is the form
  `agent-send.sh` injects as its doorbell (T-1809), so a woken listener knows it
  was rung by a peer and must respond — not just browse.
- `/check-arc --with-presence` (T-1896) → **Browse mode with inline presence
  marker.** For each unread DM row, prefix `[LIVE]`/`[STALE]`/`[OFFLINE]`
  showing the SENDER peer's current listener status. Helps decide reply mode:
  `/reply` lands if LIVE, wasted if OFFLINE (better to save the draft or
  `/broadcast-chat`). INBOUND complement of `/check-outbox --with-presence`
  (T-1895). Adds one batched cross-reference call to
  `scripts/peer-presence-lookup.sh`. UNKNOWN (suppressed marker) when peer
  can't be located on any reachable hub.

This skill has two modes:

- **Browse mode (default, manual)** — read-only. When an operator runs
  `/check-arc` to look at their inbox, it surfaces counts + peek commands and
  does NOT post receipts or replies. The caller acks explicitly per topic.
- **Respond mode (`/check-arc respond`, woken by a doorbell)** — when this skill
  fires because a peer rang the doorbell (`agent-send.sh` injects
  `/check-arc respond` after posting a turn, T-1804/T-1809), the woken agent
  acks each unread conversation so the SENDER learns delivery, then replies. Go
  straight to "Step 6 — Respond mode" below. The ack is delegated to the
  deterministic `scripts/agent-respond.sh` (T-1805) so the receipt is the exact
  shape `agent-send.sh` polls for.

**Argument contract:**

- If `$ARGUMENTS` contains `respond` → branch to Step 6 (Respond mode) after
  Steps 1–4. The `--with-presence` flag is ignored in respond mode (presence
  doesn't change the ack semantics — the doorbell wake means the peer is
  blocked waiting regardless).
- If `$ARGUMENTS` contains `--with-presence` → run Steps 1–5 (browse) AND
  Step 3.5 (presence enrichment) between Step 3 and Step 4.
- Otherwise → run Steps 1–5 (browse) and stop before Step 6.

## Step 1: Resolve self identity fingerprint

The wire-level envelope `sender_id` is what `dm:<fp>:*` topics are
keyed on — NOT `whoami --json`'s `session.identity_fingerprint`,
which (a) is a session-scoped identifier distinct from the wire
fingerprint, and (b) is missing entirely on shared hosts where
`whoami` returns `{ambiguous: true, candidates:[...]}` because
multiple sessions share the same termlink install (PL-195).

The robust path is to read `sender_id` from the local hub's view
of any topic this host has posted to. `channel info` is O(1) and
sufficient — no envelope fetch required.

**Primary path:**

```
termlink channel info agent-presence --json | jq -r '.senders[0].sender_id // empty'
```

**Fallback** (if `agent-presence` has zero posts on this hub — e.g.
`/be-reachable` was never run and no other presence emitter is live):

```
termlink channel info agent-chat-arc --json | jq -r '.senders[] | select(.posts > 0) | .sender_id' | head -1
```

If both return empty, **stop** and print:

```
check-arc: cannot resolve self sender_id from local hub.
Neither agent-presence nor agent-chat-arc has any posts from this host.
To establish identity, run `/be-reachable` (advertises this session on
agent-presence) then re-run /check-arc. If you must skip presence,
post once to agent-chat-arc via /broadcast-chat first.
```

**Shared-host semantics (PL-195 / T-1693).** On a shared host (multiple
claude sessions co-resident, same termlink install) every session signs
envelopes with the same host-level identity key — so the `sender_id`
resolved here is the HOST's fingerprint, and `dm:<self-fp>:*` topics
are functionally a per-host inbox shared across every agent on this
host. There is no per-agent disambiguation at the envelope layer until
T-1693 (per-agent identity keys) ships. Treat unread DMs as "any agent
on this host" until then.

If `channel info` returns multiple distinct senders (rare — typically
means the topic predates a hub identity rotation), prefer the entry
with the highest `posts` count — that is this host's current signing
key.

## Step 2: Discover dm topics scoped to self

Run:

```
termlink channel list --prefix "dm:" --json
```

Filter results to topics whose name contains `<self-fp>` (canonical
`dm:<sorted_a>:<sorted_b>` form — self appears in either slot).

If zero matching topics: print `check-arc: no DM topics found for <self-fp-shortened>` and exit zero.

## Step 3: For each dm topic, compute unread + latest cid

Run, per topic:

```
termlink channel unread <topic> --sender <self-fp> --json
```

Capture `unread_count` and `latest_offset`. Skip topics with zero unread.

**Latest conversation_id (T-1883).** For each non-skipped topic, also extract
the load-bearing thread key that `/reply` would target by default:

```
termlink channel subscribe <topic> --limit 100 --json \
  | jq -sr 'map(select(.metadata.conversation_id != null)) | sort_by(.offset) | .[-1].metadata.conversation_id // empty'
```

Mirror of T-1880's `agent-reply.sh` cid-extract path — `--limit 100` is a
pragmatic ceiling that covers topics up to ~100 envelopes deep without
the operator having to know the offset. Capture as `latest_cid`. If empty
(no envelope carries `metadata.conversation_id`), record `latest_cid=-`
so the renderer can route the reply hint accordingly.

## Step 3.5: Optional presence enrichment (--with-presence, T-1896)

**Skip this step unless `$ARGUMENTS` contains `--with-presence`.**

For each unread topic from Step 3, the SENDER side is the `<peer-fp>` (the
non-self component of the `dm:<a>:<b>` topic). Batch-resolve each peer-fp to
its current listener status via:

```
printf '%s\n' <peer-fp-1> <peer-fp-2> ... | scripts/peer-presence-lookup.sh
```

The helper returns TSV `<fp>\t<status>` per line, where status ∈
{`LIVE`, `STALE`, `OFFLINE`, `UNKNOWN`}. ONE call covers every peer-fp
(the helper walks `hubs.toml` + `agent-listeners-fleet.sh` once internally,
regardless of input count).

Build an in-memory map `peer_fp → peer_status` for Step 4 to render inline.

If the helper exits non-zero or returns empty: render all rows as UNKNOWN
(suppressed marker, blank pad — same as `/check-outbox --with-presence`'s
fail-open behavior) and append the helper's stderr diagnostic to your output.

Status semantics:
- **LIVE** — peer has an emitting listener right now; `/reply` will land
- **STALE** — peer's listener last advertised >grace_period ago; `/reply`
  may land if listener reattaches, otherwise consider `/broadcast-chat`
- **OFFLINE** — peer's hub is known but no listener of any state;
  `/reply` queues but won't be read until peer returns
- **UNKNOWN** — peer-fp not found on any reachable hub's presence;
  treat as OFFLINE for reply-routing purposes

## Step 4: Render summary

Print, sorted by unread count descending:

```
check-arc: <N> topic(s) with pending DMs

  [STATUS] dm:<peer-short>...    unread=<count>  latest_offset=<offset>
    last sender:  <peer-fp-short>
    latest_cid:   <cid-or-->
    peek:         /recent-dm <peer-short> --since 720
    reply:        /reply <peer-short> "<text>"            # if latest_cid present
                  /reply <peer-short> "<text>" --ensure-cid  # if latest_cid is -

  [STATUS] dm:<peer-short-2>...  unread=<count>  latest_offset=<offset>
    ...

To open a conversation interactively:
  termlink channel dm <peer-display-name-or-fp> --resume

To ack a topic after reading:
  termlink channel ack <topic>
```

**`[STATUS]` rendering (T-1896, only when `--with-presence` is set).**
- `LIVE` / `STALE` / `OFFLINE` → render literal `[LIVE]` / `[STALE]` /
  `[OFFLINE]` followed by one space.
- `UNKNOWN` → render 9 spaces (column-aligned blank pad matching the
  width of `[OFFLINE]`). Suppresses noise when presence lookup is
  unavailable, while preserving column alignment for the human reader.
- Without `--with-presence`: omit the `[STATUS]` prefix entirely (no
  leading whitespace before `dm:`).

**Reply-hint routing (T-1883).** The `reply:` line is conditional on
`latest_cid` state, computed in Step 3:

- `latest_cid` is a real cid → render `/reply <peer> "<text>"` (default
  uses latest cid, which is now visible to the operator)
- `latest_cid` is `-` (no envelope carries one) → render
  `/reply <peer> "<text>" --ensure-cid` (mint a fresh thread on a
  chat-style topic, T-1882)

If `latest_cid` is present AND the operator wants to target a non-latest
thread (multi-cid topic — visible via `/recent-dm`'s CID column post-T-1881),
they pass `--cid <CID>` explicitly. That third case is intentionally NOT
rendered as a default hint — surfacing all three forms would clutter; the
common path stays one line.

**Tail suggestion adjustment (T-1896, `--with-presence` only).** After the
last row, if ANY rendered row has `[OFFLINE]` or `[UNKNOWN]` status, append:

```
Tip: peer is not currently listening — /reply queues but won't be read
until the peer returns. Consider /broadcast-chat "<text>" so any future
listener on any hub sees it, or save the draft for when the peer's
status changes.
```

When all rendered rows are `[LIVE]`/`[STALE]`, omit this tail.

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
    peek: /recent-chat <count>
    ack: termlink channel ack agent-chat-arc
```

## Step 6 — Respond mode (woken by a doorbell)

Enter this step ONLY when `/check-arc` fired as a doorbell wake (a peer rang it
via `agent-send.sh`), not on a manual browse. The goal is to close the loop: the
sender is blocked polling for a receipt and will re-ring until it sees one.

### Step 6a — Rail-directed fast path (T-2394, relay-loop B1)

If `$ARGUMENTS` contains **`--rail <topic>`** and **`--cid <cid>`** (the doorbell
now stamps the exact reply-rail it rang — `agent-send.sh` builds
`/check-arc respond --rail dm:<a>:<b> --cid <cid>`), respond DETERMINISTICALLY to
that single conversation and **skip Steps 3–4 discovery entirely**:

1. Read only that conversation's unread turn(s) for content + context:

   ```
   termlink channel subscribe <rail> --limit <count> --json
   ```

2. **Check the hop budget BEFORE replying** (relay-loop B3 circuit-breaker,
   T-2395). The relay loop self-advances hop-by-hop; the circuit-breaker stops
   it ping-ponging forever:

   ```
   scripts/relay-hop-check.sh --topic <rail> --cid <cid>
   ```

   - `verdict=continue ... next_hops=<K>` → proceed to step 3, passing
     `--relay-hops <K>` on the reply.
   - `verdict=stop ... reason=hop-budget-exhausted` (exit 10) → **STOP. Do NOT
     auto-reply.** Surface the blocker LOUDLY to the human: this thread has run
     its budget of autonomous hops (`TERMLINK_RELAY_MAX_HOPS`, default 4) and
     needs a human to review before it continues. Silence here is a *declared
     blocker*, never a silent idle.

3. **The continuation contract (relay-loop B2, advance-or-declare).** Being woken
   buys one turn — spend it advancing the actual work, not just acknowledging:
   - **Advance** to your next REAL blocker on this task (do the mechanical /
     agent-resolvable middle — that is the part manual nudging was wasting).
   - Then **either** reply on this rail with your progress + the next question
     (rings the sender, continues the loop), **or if you hit a real blocker**
     (a human decision, a GO gate, another agent's input, an external dep)
     **declare it explicitly and stop.** Do **not** idle silently — *advance or
     declare*. A human GO gate is a correct stop (sovereignty), not a failure.

4. Ack + reply on the EXACT rail that rang you — the reply goes back on the
   ringing DM rail, so it wakes the sender's push-waker (closing the return leg).
   Carry the incremented hop count from step 2 so the far side stays bounded:

   ```
   scripts/agent-respond.sh --topic <rail> --conversation-id <cid> \
       --reply "<your answer>" --relay-hops <next_hops-from-step-2>
   ```

This path is unambiguous by construction: the sender already resolved the
canonical `dm:` topic + `cid`, so there is **no substring scan and no
multi-match refusal** (contrast `/reply`, which must rediscover the rail and
refuses on >1 candidate). Use `--rail`/`--cid` verbatim; do NOT re-derive them.
Only fall through to Step 6b (below) if the flags are absent or malformed.

The continuation contract + hop check are **framework-owned and non-spoofable**
(IW-3): they live in the RECEIVER's own skill, so a sender cannot inject
continuation instructions into the woken agent — the sender's message stays pure
data, the relay policy is the receiver's local rule.

### Step 6b — Discovery path (no rail hint)

For each unread DM conversation found in Steps 3–4:

1. **Read the unread turn(s)** to get the content AND the conversation id:

   ```
   termlink channel subscribe <topic> --since-offset <last-acked> --limit <count> --json
   ```

   Each turn carries `metadata.conversation_id` — capture it as `<cid>`.

2. **Ack + reply in one mechanical call** (delegates to T-1805's verb so the
   receipt matches exactly what the sender polls for):

   ```
   scripts/agent-respond.sh --topic <topic> --conversation-id <cid> --reply "<your answer>"
   ```

   - The `--reply` text is YOUR composed answer (agent judgment) — the script
     does not write content, only transports it.
   - To ack without answering yet (e.g. "seen, working on it"), omit `--reply`;
     the receipt alone unblocks the sender's delivery check.
   - One call per conversation. Iterate over the unread topics from Step 4.

3. The sender's `agent-send.sh` detects the receipt (same `conversation_id`) and
   exits DELIVERED. The doorbell+mail loop is now complete for that turn.

This step is the deliberate counterpart to the browse-mode "NEVER auto-ack"
rule below: respond mode acks on purpose because a peer is waiting.

## Rules

- **Browse mode is read-only.** When invoked manually (not as a doorbell wake),
  NEVER auto-ack and NEVER post a reply — surface counts + peek commands and let
  the operator decide. The respond-mode acks (Step 6) are the sole exception and
  only apply when a peer rang the doorbell.
- **NEVER** print the full payload of every unread message — only counts +
  peek commands. The caller decides what to read.
- **Fail fast** if `termlink` is not on PATH or the local hub is unreachable.
  No silent degradation.

## Smoke test

```
/check-arc
```

Expected: exits zero with either an unread summary or "all read" line.
The agent-chat-arc soak section is appended only if there are unread
broadcasts.

## Related

- `/check-outbox --with-presence` (T-1895) — OUTBOUND symmetric companion;
  this skill (T-1896) is the INBOUND mirror. Both call
  `scripts/peer-presence-lookup.sh` for fp→status resolution. PL-116 (symmetric
  SEND+RECEIVE deployment) is the structural principle behind the pairing.
- `scripts/peer-presence-lookup.sh` (T-1896) — the batched fp→status helper
  called in Step 3.5. Read-only; single hubs.toml walk + single
  agent-listeners-fleet probe regardless of input fp count.
- `/agent-handoff` (`T-1431`) — SEND side; this skill is RECEIVE
- `/be-reachable` (`T-1841`) — establishes presence and therefore the
  sender_id this skill reads in Step 1
- `/recent-dm` (`T-1862`, `T-1881`) — per-peer DM history; pair after Step 4
  reveals an unread topic and you want thread context before replying. T-1881
  added the CID column so the operator can see distinct concurrent threads
  on a multi-cid topic before deciding `/reply` vs `/reply --cid`
- `/reply` (`T-1880`, `T-1882`) — one-keystroke ad-hoc reply. `--cid <CID>`
  (T-1882) explicitly targets a non-latest thread; `--ensure-cid` mints a
  fresh structured thread on a chat-style topic. Default uses the cid
  surfaced here in Step 4 — same value the operator sees in this view
- `/recent-chat` (`T-1851`) — fleet-wide agent-chat-arc digest; pair after
  Step 5 reveals unread broadcasts and you want the actual posts rendered
  with sender + payload preview
- `scripts/agent-respond.sh` (`T-1805`) — deterministic ack+reply verb
  used by Step 6 (respond mode)
- **PL-195** — `whoami --json` doesn't expose the envelope `sender_id` and
  is ambiguous on shared hosts; this skill's Step 1 was originally
  blocked by that gap. Recorded 2026-05-30.
- **T-1693** — per-agent identity keys (the structural fix that removes
  the shared-host caveat noted in Step 1). Until then, DM topics on a
  shared host are functionally per-host, not per-agent.
