# T-2397 — Agent flow model: busy-session-as-responder vs dedicated responder (inception)

**Status:** inception / exploration
**Recommendation (advisory):** GO on **Model B**, validated live before Model A
**Decision owner:** human (Dimitri)
**Date:** 2026-07-10

---

## The question this closes

Every prior comms task fixed a *link* (WAKE T-2388, DISCOVERY T-2390/91/92, SEND
rail T-2394, continuation T-2395, consumption-confirmation T-2396). The operator
asked "why doesn't it flow?" three times. Each answer was true and each time it
*still* didn't flow. This inception names the thing underneath all of them:

> **An interactive claude session is turn-based. It processes input → emits
> output → awaits the next input. It has no idle loop of its own.** So "an agent
> that does its own work AND instantly responds to peers" is in direct tension
> with the execution model. You cannot have both properties in one session for
> free.

Decide the *model* for autonomous flow. Then plumbing follows the model, instead
of us adding one more link and re-discovering the same wall.

## The five models

| # | Model | How a peer message becomes a turn | Cost | Verdict |
|---|-------|-----------------------------------|------|---------|
| **A** | **Dedicated responder session** — each reachable agent runs a 2nd, idle, auto-accept session whose only job is to consume wakes | Wake hits the always-idle responder → it acks + relays | 2 sessions/agent; responder lacks the worker's task context (can ack + route, can't advance the work) | Robust reachability, weak *flow* |
| **B** | **Single armed auto-accept session, interrupt-and-resume** — one session under `tl-claude.sh --reachable`; a wake interrupts its current work, it handles the peer turn, resumes | Wake submits (auto-accept) → current turn is the peer message → relay-loop B2 contract advances-or-declares | 1 session, full context; interrupts own flow | **Already built (T-2388); untested live** |
| C | Daemon + `claude -p` per message | Daemon consumes inbox, spawns headless claude per msg | Re-pays context each call (memory: avoid for stateful design dialogue) | Good for cheap fan-out, bad here |
| D | Poll-within-turn | Agent checks inbox at end of each turn before yielding | Only works if something drives the *next* turn → circular for interactive sessions | Doesn't close it alone |
| E | Human-in-the-loop (status quo) | Human advances each session to read | Zero build; the nudging we're trying to remove | The baseline we're rejecting |

## Why B is the provisional GO

- **It already exists.** `tl-claude.sh start --reachable --agent-id <id>` (T-2388)
  launches claude in an auto-accept injectable PTY. The live agents (aef,
  designer) are **not** launched this way — they run plain `claude` in
  manual-accept mode, which is *exactly* why injected wakes land unsubmitted
  (T-2396 root cause). So the dominant failure today is a **deployment gap in
  Model B, not a missing model.**
- **It composes with the relay loop.** B2's continuation contract
  ("advance to your next real blocker, then reply-on-rail or declare + stop") is
  precisely the behavior a woken auto-accept session needs. B3's hop-budget
  bounds it. T-2396's wake-confirm makes a non-consume loud. All three were built
  *for* Model B — they just never ran against an armed agent.
- **It keeps full context.** One coherent session that both does the work and
  answers about it — unlike Model A, whose responder can ack but can't advance.

**The one real unknown B carries:** when agent A is mid-task on T-168 and a wake
about T-175 interrupts it, does A coherently context-switch, respond, and *resume
T-168* — or does it derail? That is empirical and **cannot be answered by more
code**. It needs a live run of two armed agents. Hence: GO on B *as a validation*,
not as a blind build.

## The proposed path (prove-first, mirroring T-2396)

1. **Deploy B on two real agents** — relaunch aef + designer (or two scratch
   agents) via `tl-claude.sh start --reachable`. Operator/cross-host action.
2. **Run one real exchange** — A sends B a genuine question via `agent-send.sh`
   (which now confirms consumption, T-2396). Observe: does B wake, respond on the
   rail, ring A back (B1), and does A advance — for ≥2 hops (B3) — without a human
   nudge, stopping loudly only at a real blocker (a GO gate)?
3. **Decide A vs B on the evidence.** If interrupt-and-resume produces coherent
   flow → B is the model, done. If agents derail on interruption → escalate to
   Model A (dedicated responders) for reachability + keep B's worker for the work.

No new mechanism is built until step 2 tells us which model survives contact.

## Open questions (IW — human owns; full text mirrored in the task file)

- **IW-1 Interruption coherence:** Is a mid-task session that gets interrupted by
  a peer wake able to resume its own task afterward, or does the context-switch
  corrupt its flow? (The load-bearing unknown; answer empirically in step 2.)
- **IW-2 Who relaunches the live agents:** aef + designer run on /opt/999 under
  another operator's control. Validate on two scratch agents on .107 first, or
  coordinate a relaunch of theirs?
- **IW-3 One session or two:** If interruption derails work (IW-1 = no), is the
  2-session cost of Model A acceptable, given the responder can only ack/route
  (not advance the peer's actual task)?
- **IW-4 Is flow even the goal for design dialogue:** Some exchanges (T-175
  GO gates) *should* stop for a human. Is the target "flow through mechanical
  hops, stop at decisions" (relay-loop premise) — or is per-turn human review
  actually wanted for design work, making E acceptable there?

## Recommendation

**GO on Model B as a live validation.** It is already built, composes with the
relay loop + consumption-confirmation just shipped, and its dominant failure mode
today is a *deployment* gap, not a design flaw. Prove interruption-coherence
(IW-1) on two armed agents before spending anything on Model A. Do **not** build
new plumbing under this inception — the deliverable is the go/no-go on the model
plus, on GO, a build task for whatever step-2 reveals is missing.

## Dialogue Log

### 2026-07-10 — inception opened
- **Human:** after T-2396 shipped (consumption-confirmation loud), chose option
  "2" — the deeper design fork: can a session doing its own work also be an
  instant responder, or does flow need a different model?
- **Agent finding:** the whole comms saga bottoms out at the turn-based execution
  model. Five models enumerated; Model B (armed auto-accept, interrupt-and-resume)
  is already built (T-2388) and just undeployed — the live agents run manual-mode
  plain claude, which is why wakes land unsubmitted. Provisional GO on B as a
  live validation; the one unknown (interruption-coherence, IW-1) needs two armed
  agents, not more code.
- **Outcome:** awaiting human go/no-go on the model + IW-1..IW-4.

### 2026-07-10 — step-2 live validation ran (T-2398)

Two scratch agents (relay-test-a/-b) launched armed + `IS_SANDBOX=1 claude
--dangerously-skip-permissions` (auto-accept) on .107. Full evidence:
`docs/reports/T-2398-findings.md`.

- **Model B premise PROVEN:** an armed auto-accept session **consumes an injected
  wake and replies** — relay-test-b acked at rail offset 1 (+47s); in the two-hop
  test B answered "17×23=391" **and advanced with a follow-up question** at
  offset 3 (+41s). Two autonomous hops, no manual nudge. Contrast with T-2396
  manual-mode silent-fail: manual claude never submits the doorbell.
- **IW-1 PASSED (no derail):** relay-test-a, rung mid-task, **completed its own
  task** (counted `.sh` files → 22,470 total lines) **and** answered the peer
  interrupt ("5+5=10"), correctly treating peer content as a G-020 *proposal*.
  The injected wake was handled as queued input after the running turn — coherent.
- **Dominant finding — the blocker is NOT the model, it's identity (T-1693):**
  `agent-send.sh` reported `FAILED — receiver never acked` on BOTH the premise and
  two-hop sends even though the rail shows the replies landed. Cause: all
  co-resident .107 sessions sign envelopes with the shared host key `d1993c2c`, so
  receipt/`--await-reply` cannot tell sender from receiver → false negatives. This
  false negative is what made A *abandon* the thread instead of taking hop 3. So
  the autonomous chain stalls at 2 hops on the **confirmation layer**, not the
  execution model.
- **A-vs-B recommendation: keep GO on Model B (it passed its trial), but it is
  necessary-not-sufficient.** The highest-priority build item is **per-agent
  envelope identity (T-1693)**, then making `tl-claude.sh --reachable` actually
  arm auto-accept (`IS_SANDBOX=1 …`, + fix launch/teardown bugs F1/F2/F4) and
  pre-arming a relay agent's active task (F5). **Model A not needed** to make B's
  worker advance a thread — B advanced it here.
