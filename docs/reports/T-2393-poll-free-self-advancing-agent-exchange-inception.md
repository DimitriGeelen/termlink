# T-2393 — Poll-free self-advancing agent exchange (inception)

**Status:** inception / exploration
**Recommendation (advisory):** GO — build the minimal "relay loop" mechanism
**Decision owner:** human (Dimitri)
**Date:** 2026-07-10

---

## The fair challenge: "you tested and verified — how can it not work?"

Both are true, and here is why they don't contradict. I tested and verified the
**individual links**. Flow needs a **chain that was never assembled** — and you
can't test a chain that doesn't exist.

What I actually proved this session, each genuinely true:
- **WAKE works** — a woken PTY rings. (T-2388 live E2E.)
- **DISCOVERY works** — presence reads correctly. (T-2390/91/92 count-seek fix.)
- **SEND works** — a DM lands on the per-fp rail and rings the recipient.

What I never tested — because it is **not a code path** — is multi-hop
autonomous **flow**. Flow is not a function you call; it is emergent behavior
that requires a *driver* looping the verified links together. That driver does
not exist. So no test failed, because there was nothing to fail — the gap is an
**absence**, not a defect. Testing each link green and concluding "the chain
holds" was the overclaim. "Doorbell complete on .107" was accurate for the four
transport *layers* I fixed; it was **not** a claim that collaboration now flows,
and I should not have let it read that way.

That is the honest reconciliation: everything verified works; the thing that is
missing was never built.

---

## Problem statement

Two agents collaborating on a task (e.g. aef ↔ workflow-designer on T-175) do
**not** progress on their own. Every hand-off requires a human to manually
"nudge". The conversation *stops* after each message instead of *flowing* to its
next real blocker. We designed and shipped **wake**. We did **not** design
**flow**.

## What already works (do not rebuild)

- **WRITE** — durable offset append.
- **WAKE** — push-waker rings an idle PTY on the per-fp DM rail. Proven live.
- **DISCOVERY** — sender can see who is reachable. Fixed across all three code
  faces this session (T-2390 shell / T-2391 CLI / T-2392 MCP).

Necessary, but **not sufficient** for flow.

## Root cause — two addressable gaps + one correct stop

### Gap 1 — Return-leg rail mismatch (routing)
The waker rings **only** on the private per-fp DM rail. Design discussions happen
on **shared thread topics** (where the conversation accumulates). Thread posts
ring **nobody**:
- A → B on a thread: B not rung, sees it on next manual check.
- B → A reply on the thread: A not rung — the **return leg never fires.**

### Gap 2 — Woken-agent continuation (autonomy)
Interactive claude sessions. Being woken buys **one turn**; the agent does its
piece then **idles again**. Nothing tells it "work to your next real blocker and
fire the next hop." Result: `hop → idle → hop → idle`, each hop needing a fresh
manual ring. Not transport per se — it is *what the transport injects* + *how the
session is driven*.

### The correct stop (NOT a bug)
T-175 is parked on an **AEF-agent decision + human GO**. Flow must **self-
terminate loudly at real blockers** and pull the human in — never bulldoze a
decision that is the human's. This is the line the mechanism must respect.

## Proposed mechanism — the "relay loop"

Self-advance through mechanical/agent-resolvable steps; self-terminate loudly at
real blockers.

1. A contacts B on the **per-fp DM rail** (`agent contact`) — rings B. *(works)*
2. Injected payload carries a **continuation preamble** (Gap 2): "You were woken
   by <A> on thread <T> re <task>. Do your piece. When done: reply to <A> via
   `/reply` on this DM rail (rings them); or if blocked on a human decision /
   another agent, state the blocker and stop. Do not idle silently — advance or
   declare."
3. B works, replies via **`/reply` on the DM rail** (Gap 1) — rings A.
4. A woken with the same preamble → advances or declares.
5. Loop runs until a real blocker is declared → **stops loudly**; human sees why.

Converges on human decision points + external deps (where it *should* stop);
self-drives the mechanical middle (where nudging was pure waste).

## Minimal build on GO (~3 build tasks, not one umbrella)

- **B1 — Reply-on-ringing-rail default.** A woken agent's reply defaults to the
  DM rail + `conversation_id` that rang it, not a thread post. `/reply` already
  does per-fp DM; stamp reply-rail metadata into the injected payload so the
  reply tooling has zero ambiguity. **B1 alone kills the "say check" symptom.**
- **B2 — Continuation preamble.** Standard bounded wrapper the inject path
  (`agent-send.sh` / doorbell) prepends to every delivered turn: advance-or-
  declare + reply-on-rail.
- **B3 — Circuit-breaker + loud blocker surfacing.** Hop budget (advance up to N
  hops or until a declared blocker, then surface) so two agents can't ping-pong
  forever, and a loud "blocker declared" signal so a stop is *visible*, never a
  silent idle.

## Open questions the human owns (mirror of IW-1..IW-4 in the task file)

1. **IW-1 Autonomy budget** — how many hops between human checkpoints? Recommend
   bounded N with the circuit-breaker surfacing at the cap.
2. **IW-2 Cost tolerance** — each hop is a full claude turn; autonomous multi-hop
   burns tokens with no human in the loop. Acceptable for design threads, or
   gated per arc?
3. **IW-3 Preamble-as-instruction** — it instructs another agent's session; must
   be framework-owned, bounded, non-spoofable. Standard preamble, or per-contract?
4. **IW-4 B1 sufficiency** — does reply-on-ringing-rail alone remove "say check",
   independent of B2/B3? (Validate empirically — it is the fast symptom-killer.)

## Recommendation

**GO** on the relay loop, built B1 → B2 → B3, with **bounded** autonomy
(circuit-breaker on) as the default so we don't trade the nudging problem for a
runaway problem. B1 ships fast and removes the "say check" symptom immediately;
B2 + B3 deliver true self-advance.

## Dialogue Log

### 2026-07-10 — inception opened
- **Human:** "why does the communication between the two not flow and stop
  without manual nudging — that is not what we designed?" → "yes focus on getting
  this to work!!!" → "how [is it] impossible, you tested and verified…"
- **Agent finding:** the tested links (WAKE / DISCOVERY / SEND) all genuinely
  work; flow is an *un-built chain*, not a broken link. Two gaps (return-leg rail
  mismatch + woken-agent continuation) + one correct stop (human GO gates).
  Proposed the relay-loop mechanism.
- **Outcome:** awaiting human go/no-go on the mechanism + the three autonomy
  questions before any build.
