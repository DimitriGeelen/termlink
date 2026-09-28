# T-3200 — A consumer for the project inbox rail

**Status:** inception, exploration phase. No build artifacts. No spikes run yet.
**Arc:** arc-011 (*Agent-to-agent message delivery: mailbox to prompt*) — this is
that arc's literal subject.

## The question

TermLink delivers durable messages to `inbox:<project>` correctly and provably.
**Nothing reads them.** How should a durable mailbox reach a session's prompt?

That is a design question with a sovereignty dimension, not a missing poller,
which is why this is an inception rather than a build.

## Evidence (measured 2026-09-28, not asserted)

- `inbox:cacc73ea32b121dd/010-termlink` held **49 unread `sidecar.consult`
  envelopes**, 48 of them from AEF.
- AEF's T-3434 retry ladder had climbed to **rung 4, 10 attempts**, re-posting
  because we never acked.
- One unread message (offset 30) was **a bug report against our own
  `agent search`**, which sat unread for **3+ weeks** while five of AEF's
  search-driven passes reported the artefacts it named as absent.
- Delivery is not in question. AEF measured a **73h05m** round trip and
  concluded, correctly: *"The rail was fine the whole time; the verdict was the
  defect."*
- The blindness was near-destructive. `inbox status` labelled those records
  "pending transfer(s)" and `inbox clear` resolves to `channel.trim`, so a
  routine-looking drain would have **permanently deleted 232 live messages**
  (T-3196; the metric is fixed in T-3197, the consumer is not).

The through-line: **every guard we have watches whether messages ARRIVE. None
watches whether anyone READS them.** G-063 is the write-only-sink class, and the
existing canaries (T-2231 pickup freshness, T-2295 unconfirmed-delivery) guard
the *outbound* and *rail* sides. The inbound-consumption side has no member.

## Options

Four shapes, genuinely different in cost and in who they interrupt.

### A. Cron canary — "empty log = healthy"
Daily check fires when `inbox:<project>` has records newer than a last-acked
marker. Exactly the T-2231 framework-pickup pattern, which already exists and
works.
- **For:** cheapest; one script plus a crontab; matches seventeen existing
  siblings, so nobody has to learn a new idiom; `/canaries` discovers it free.
- **Against:** surfaces to whoever reads canary output, which is a human at a
  terminal. A consult arriving at 02:00 waits for a person. It detects the
  backlog; it does not deliver the message.

### B. Surface into the handover
The handover generator already renders "Revisits Ripe Today". Add unread inbound.
- **For:** lands in the one document every session actually reads at start; zero
  new infrastructure; survives session boundaries, which is where this failed.
- **Against:** only fires at session start. A long session never sees it. And the
  handover is already long — this competes for attention with everything else on
  it (the T-2818 fatigue risk).

### C. Wake on `inbox.queued`
What AEF actually asked for: a live subscriber that wakes the session when a
message lands, via the existing push-waker rail (T-2387 / T-2404).
- **For:** the only option with a latency answer measured in seconds. Directly
  answers AEF's ask (1). The doorbell machinery already exists.
- **Against:** the largest build, and it interrupts a working session — which is
  a sovereignty question (IW-1), not just an engineering one. Also inherits the
  waker fleet's own failure modes (frozen husks, stale code), which we already
  run three canaries to watch.

### D. A skill-layer verb (`/inbox`)
An operator-invoked read, sibling to `/check-arc` and `/peers`.
- **For:** trivial; composes with the existing daily-verb set; no scheduling.
- **Against:** **it is the status quo dressed up.** Nothing makes anyone run it.
  This is precisely the PL-168 shape — a capability with no trigger is dormant
  tooling — and it is how we got here.

## The honest read

These are not exclusive, and the useful split is *detection* vs *delivery*.

A or B makes the backlog **visible**. C makes the message **arrive**. D alone
changes nothing, and should be rejected as a standalone answer however convenient
it looks.

My advisory position — explicitly not a decision — is **B + C**: put unread
inbound in the handover immediately because it is cheap and closes the
cross-session hole that actually bit us, and scope C separately because it is the
real fix and the one AEF asked for. A is redundant with B if B ships.

## Open questions

Filed in the task register as IW-1..IW-4, summarised here:

| id | question | why it blocks |
|---|---|---|
| IW-1 | Should an inbound peer message interrupt a working session? | Option C requires yes. A claim about who owns a session's attention — operator's call. |
| IW-2 | Do we ack on READ or on ACTION? | Acking on read stops AEF's ladder while the work is undone, converting correct escalation into silence. |
| IW-3 | Does a consumer generalise beyond AEF? | 22 topics, 5 peers. One built around AEF's convention breaks on the second. |
| IW-4 | What is the real inbound rate? | Unmeasured. Interrupt cost is a function of volume, so IW-1 cannot be answered honestly without it. |

IW-4 is the cheapest to settle and it gates the expensive decision, so it should
be first if exploration is authorised.

## What exploration would actually settle

- Whether the push-waker rail can carry `inbox.queued` without new hub work
  (read `channel.subscribe` semantics; do not build).
- The inbound rate across all 5 peer topics (IW-4).
- Whether the other four peers post with metadata comparable to AEF's (IW-3).
- Whether the handover generator can take a new section without the T-2015
  re-vendor problem — it is vendored, so B is not free.

**No spikes have been run.** Per Inception Discipline this document is the
artefact of record and exists before the research, not after it.
