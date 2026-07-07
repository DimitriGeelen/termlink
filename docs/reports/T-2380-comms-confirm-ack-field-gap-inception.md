# T-2380 — Inception: cross-agent comms confirm/ack field gap (hub-split + degraded-read)

**Type:** Inception (one question, go/no-go)
**Recommendation (agent, advisory):** GO — decide the permanent shape before building
**Status:** exploration
**Anchor evidence date:** 2026-07-07

## The one question

The reliable-comms arcs (arc-003 `reliable-comms`, arc-004 `push-transport`)
shipped a **durable send + sub-second push-wake** that is proven in isolated
single-hub E2E tests (T-2318, T-2325, T-2320: 85–111 ms wake). Yet in the live
fleet the **confirm/ack half fails silently**, repeatedly. Is closing that field
gap worth a permanent fix, and what is its shape?

## Why this is an inception, not a jump-to-fix

The diagnosis moved **twice** in one session:

1. First claim (wrong): "the autonomous receiver was never built."
2. Corrected by reading the arcs: arc-004 push-waker IS built, load-bearing,
   live-E2E-verified (T-2316 "WS now LOAD-BEARING", T-2325 dm-rail live E2E).
3. Corrected again by field data: the mechanism works, but the *confirm/ack*
   half breaks on conditions the isolated E2E never covered.

A moving diagnosis is the signal to **validate scope**, not build. The candidate
fixes (below) are materially different in cost and blast radius; picking one
without an inception risks building the wrong thing a third time.

## Grounded evidence (this session, reproducible)

All observed 2026-07-07 on the real fleet (.107 local hub + .122
ring20-management):

### E1 — hub-split / no-federation makes a delivered message look lost (G-060)
- I sent a `.121` upgrade handoff to `ring20-management-agent` via
  `agent contact --target-fp 9219671e28054458 --hub 192.168.10.122:9100`.
  It landed durably: `dm:9219671e28054458:d1993c2c3ec44c94` **offset 52** on .122.
- A co-resident session on .107 (**same identity** `d1993c2c3ec44c94` — shared
  host key, PL-166) later ran `recent_dm` on the same-named topic and saw
  **0 posts / total_posts:0**.
- Direct `channel info` proves *why*: the **.107** copy of that topic has
  **count 113**; the **.122** copy has ~53 (offset 52). **Same name, two
  independent histories, no federation.** The message was never lost — it was
  written to a hub the reader wasn't reading. G-060 is documented, but nothing
  in the SEND/READ verbs stops a sender and reader from silently targeting
  different hubs for "the same" conversation.

### E2 — a degraded-read hub makes `--ack-required` false-timeout forever
- `.122 channel list` returns **105 topics instantly** — metadata reads are fine,
  hub is up (TLS probe 46 ms).
- But **per-topic message reads on .122 time out** (`channel info <dm>`,
  `agent-presence` subscribe, `governor-status` all killed at 15–30 s), while a
  **write** (offset 52) returned instantly.
- Likely cause: **PL-200** — agent-presence topic bloat after the vendored
  binary swap on .122 (T-1985 class / T-1991 recurrence). Message-scan reads are
  slow; metadata reads are not.
- Consequence: `agent contact --ack-required` polls the topic's *messages* for a
  reply. Against .122 that poll **can never succeed** even if the peer replied →
  the observed **phantom "waiting 2 hours for an ack."** A third session hit this
  verbatim on an unrelated T-010 Cloudron-deploy coordination request.

### E3 — the arc's own guarantee has a hidden precondition
arc-003 headline: *"confirmed delivery receipt … no silent loss."* That holds
**only if** (a) the reader targets the same hub the writer wrote to, and (b) that
hub's message-read path is healthy. Neither is enforced or surfaced. When either
fails, the sender gets a write success (`offset N`) and then **silent
uncertainty** — exactly the failure mode arc-003 set out to kill, relocated from
"send" to "confirm."

## Adjacent findings (same session, feed the scope but may be separate tasks)

- **F1 — agent-vs-shell is invisible.** The "email-archive agent" targeted by a
  peer is a bare `termlink register --shell` bash session (pid 24634, idle, no
  claude behind it). `remote list` renders it identically to a real agent
  (`NAME ready`). Injecting a prose directive into it can never act; the sender
  can't tell "no agent" from "agent ignoring me."
- **F2 — raw `inject` bypasses the whole rail.** `termlink remote inject` writes
  bytes straight to a PTY (and without `-e/--enter` doesn't even submit),
  bypassing hub `dm.queued` → push-waker → doorbell and producing a false
  "Injected N bytes" success with none of arc-003's confirm/journal.
- **F3 — `/be-reachable` is opt-in + reboot-fragile**, so wakers frequently don't
  exist when a sender sends.

## Candidate directions for the go/no-go (not yet chosen)

| # | Direction | Attacks | Rough cost |
|---|-----------|---------|-----------|
| C1 | **Reply-on-sender's-hub convention** (tooling default: replies route to the hub the originating message was read-reachable on) | E1 | low–med |
| C2 | **Hub-read-health precondition** so `--ack-required` fails *fast* ("target hub message-reads unavailable") instead of burning the timeout | E2, E3 | low–med |
| C3 | **Actual cross-hub federation** for dm: topics (or a relay) so same-name = same history | E1 (root) | high |
| C4 | **Agent-vs-shell signal** in `remote list` + inject warns/refuses when target is not agent-backed or bypasses the DM rail | F1, F2 | low |
| C5 | **Durable/auto-armed `/be-reachable`** so wakers exist across reboots | F3 | med |
| C6 | **Fix .122 specifically** (channel sweep / retention reset run locally on .122) | E2 instance | op-only, ring20-manager scope |

C6 is an operational instance-fix (ring20-manager's host, not our code) and is
already being folded into the live coordination message. C1+C2 look like the
minimum permanent fix that removes the *silent* in "silent uncertainty"; C3 is
the ambitious root fix; C4/C5 are the earlier cosmetic guard-rails, now clearly
secondary to C1/C2.

## Open Questions (mirror of task-file IW-N)

- **IW-1:** reply-on-sender-hub convention? (C1, attacks E1)
- **IW-2:** hub-read-health fail-fast for `--ack-required`? (C2, attacks E2/E3 — strongest evidence)
- **IW-3:** cross-hub federation in or out of scope? (C3, high cost)
- **IW-4:** are the F1/F2/F3 guard-rails part of this fix or separate? (C4/C5)

## Assumptions to validate

- A1: the .122 read-wedge is agent-presence bloat (PL-200), not a binary/version
  regression — testable by a local `channel sweep` on .122 restoring read speed.
- A2: reader/writer hub disagreement (E1) is common in practice, not a one-off of
  my `--hub .122` choice — testable by auditing how peers pick a hub for replies.
- A3: no existing convention already says "reply on the sender's hub" that we're
  simply not following.

## Dialogue Log

### 2026-07-07 — operator-driven, live
- **Operator:** "our mechanism is not working" → later "we did extensive work on
  this, read back, was it arc 004?!"
- **Correction:** agent had wrongly claimed the receiver was never built; reading
  arc-003/arc-004 on disk showed it was shipped + live-E2E-verified.
- **Operator (on seeing offset-52-vs-113 + .122 read timeout):** "THIS IS REALLY
  BAD; WHAT NOW?" → agent de-escalated with grounded data (nothing lost; two
  separable causes) + 3 immediate moves.
- **Operator:** file this as an **inception** (not concern+tasks), then draft the
  corrected ring20-manager message. → this artifact + T-2380.
