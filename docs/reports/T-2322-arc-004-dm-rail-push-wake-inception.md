# T-2322 — Extend push-wake to the dm rail (inception)

**Arc:** arc-004 `push-transport` (follow-on; arc already closed=shipped)
**Workflow:** inception (one question, one go/no-go)
**Status:** exploration complete — recommendation written, awaiting human `fw inception decide`

---

## The one question

Should we extend the shipped arc-004 push-waker to also ring the receiver on a
direct `dm:<self>:<peer>` post made by a **non-live-sender** — and if so, by what
live-topic mechanism?

## The verified gap

The shipped push-waker (T-2316) rings the receiver on `inbox.queued` aggregator
frames. The hub emits `inbox.queued` **only** for `channel.post → inbox:<id>`
topics:

- `crates/termlink-hub/src/channel.rs:752` — `if let Some(addressee) =
  topic.strip_prefix("inbox:")` … `agg.inject(inbox.queued …)`. The emit fires
  only on a *successful* post (after the `Ok` arm; a failed post errors at
  line 771/777 before reaching it).
- `crates/termlink-hub/src/channel.rs:3038` — negative test
  `channel_post_non_inbox_topic_does_not_fire` proves a non-`inbox:` topic emits
  nothing.

A `dm:<self>:<peer>` topic is a non-inbox topic ⇒ **no `dm.queued` frame ⇒ the
waker never rings** for it. Today a direct `dm:` post wakes the receiver **only**
if the SENDER performs the ring-1 inject (`scripts/agent-send.sh` /
`termlink agent contact`).

### What is already covered (important — narrows the gap)

The shipped arc already covers the two primary agent-to-agent paths:

1. **inbox deposits** — `/agent-handoff` → `termlink agent contact` deposits to
   `inbox:<id>`, which **does** fire `inbox.queued` (line 752). Covered.
2. **live-sender dm** — a live termlink session posting to `dm:` performs the
   ring-1 inject itself (`agent-send.sh`). Covered.

### What is NOT covered (the actual hole)

A `dm:<self>:<peer>` post whose poster does **not** ring:

- a raw `termlink channel post dm:… --payload …` from a shell/cron,
- a remote peer posting cross-hub via `--hub`,
- the MCP `termlink_channel_post` tool driving a `dm:` topic (orchestrator/peer
  automation that talks to the durable rail but is not a live PTY sender).

These reach the durable `dm:` topic (message is safe, receipts work, `/check-arc`
poll will surface it) but do **not** push-wake the receiver — they fall back to
poll latency instead of the arc's sub-second wake.

## Design candidates

| # | Mechanism | Hub change? | Waker complexity | Portability (D4) |
|---|-----------|-------------|------------------|------------------|
| **A** | Hub-side `dm.queued` aggregator emit (mirror of `inbox.queued`) for `dm:` posts | Yes — one emit block | Lowest (subscribe one aggregator topic) | Good |
| B | Client wildcard/prefix `dm:<self>:*` push subscribe | No (if hub supports prefix push) | Low | Depends on a push feature that may not exist |
| C | Client discovery loop: list `dm:<self>:*`, `--push`-subscribe each | No | High (re-subscribe churn) + reintroduces a discovery poll floor | Pure-client but self-defeating |

**Candidate A is a near-verbatim mirror of the existing T-1637 emit** at
channel.rs:752 — add a sibling `else if let Some(_) = topic.strip_prefix("dm:")`
block emitting a `dm.queued` frame with the same `{addressee, channel,
message_offset, enqueued_at}` shape (addressee derived from the `dm:` topic's
non-self participant). The waker adds one `subscribe dm.queued --push` alongside
its existing `inbox.queued` subscribe and filters frames to self. Bounded,
testable (a positive sibling of the channel.rs:3038 negative test), reversible
(delete the emit block), portable.

## Value-of-information — the honest counter-argument

- **For GO:** reliability directive dislikes silent gaps; the fix is cheap,
  well-understood, and closes a verified reliable-comms hole before the next
  cross-host/MCP integration relies on it. arc-011 (parallel-dispatch) and
  MCP-driven peers are plausible near-term consumers of the uncovered path.
- **For DEFER:** the two *primary* agent-to-agent paths are already covered
  (inbox deposit + live-sender ring). I could **not** confirm a current live
  consumer that actually posts to a `dm:` topic without ringing. Building now is
  mildly speculative; because the fix is cheap and well-scoped, little is lost by
  waiting for the first real occurrence to promote it.

This tension (IW-2) is exactly why the go/no-go is a human decision. Recommendation
below states a default with the caveat made explicit.

## Recommendation

**Recommendation:** GO — Candidate A (hub-side `dm.queued` emit), **conditioned**
on the human's read of demand.

**Rationale:** The gap is verified (channel.rs:752 + the 3038 negative test) and
sits in the reliable-comms core. Candidate A is a bounded, reversible, testable
mirror of an already-proven pattern (T-1637), so the build risk is low and the
portability profile is good. The one soft spot is VOI — I could not confirm a
live consumer hitting the uncovered path today — so a DEFER is defensible if you
prefer to wait for a concrete consumer. Given the low fix cost and the arc's
"no silent wake gaps" goal, my default is GO, decomposed into three slices
mirroring T-2316→T-2320:
  - S1: hub `dm.queued` emit + positive/negative unit tests
  - S2: waker `dm.queued --push` subscribe + self-filter
  - S3: live E2E (real spawn → raw `dm:` post by a non-live poster → PTY rings)

**If DEFER instead:** set `revisit_at` + `revisit_evidence_needed: "first
confirmed non-live-sender dm: post that a receiver missed at poll latency"` so the
G-053 cron re-surfaces it when a real consumer appears.

## Reproduce the gap

```bash
# Negative proof the hub emits nothing for a non-inbox topic:
grep -n "channel_post_non_inbox_topic_does_not_fire" crates/termlink-hub/src/channel.rs
# The emit that DOES fire (inbox only):
sed -n '748,768p' crates/termlink-hub/src/channel.rs
```
