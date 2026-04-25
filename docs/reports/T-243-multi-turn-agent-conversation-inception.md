# T-243 Inception — Multi-Turn Agent Conversation Primitive over TermLink

**Status:** in-progress (Dialogue 1 partial, awaiting concrete failure scenarios)
**Re-scoped:** 2026-04-26 (from "script error yielding" → "multi-turn agent conversation primitive")
**Origin:** captured 2026-03-23 under T-233 (specialist agent orchestration) as the one open problem deferred from that decomposition.
**Companion task:** T-1284 (G-011 auth structural fix — foundation, must land first or in parallel)

---

## Why this exists

TermLink's stated purpose is agent-mesh coordination across machines and sessions. The first-order test of that mission is: **can two agents hold a reliable multi-turn conversation?** Today the answer is no. The operator's stated pain (2026-04-26):

> "We keep having key rotation issues, authentication issues, and interactive multi-turn conversation between two or more agents is absolutely not working."

That sentence breaks into two coupled problems:

1. **Auth flake** — recurring secret rotation, TOFU drift, cache divergence. Every multi-turn exchange has N opportunities for auth to fail vs. 1 for a single-shot RPC. This is a foundation problem and is being addressed under T-1284 + the existing T-1051..T-1058 line + G-011.
2. **No multi-turn primitive** — even with stable auth, TermLink's existing primitives are single-shot. There is no session-scoped dialog, no yield/resume, no clean way for one side to say "hold on, asking my LLM" without the other side timing out.

T-243 owns problem 2. T-1284 owns problem 1. They are deliberately split because building (2) on top of unstable (1) produces unreliable conversations *and* wrong root-cause attribution — every failure looks like it could be either layer.

---

## What exists today (current TermLink primitives, by surface)

| Primitive | Shape | Multi-turn? |
|---|---|---|
| `agent.ask` / `agent.listen` | Typed request/response, single shot | No — each call is independent |
| `request` (event request-reply) | Single request, single reply, correlation by id | No — no notion of follow-up |
| `channel.post` / `channel.subscribe` | Pub-sub topic with retention, signed envelopes | Partial — multi-message, but no conversation grouping |
| `termlink interact` / `pty inject/output` | Drives a PTY; types input, reads output | Yes for human↔terminal — wrong layer for agent↔agent |
| `event.emit_to` | Targeted event delivery | No |

PL-012 names the seam exactly:

> *"Pickup envelopes are a messaging channel, TermLink sessions are an execution channel — agents must know the distinction."*

A multi-turn agent conversation lives on the **messaging** side but wants **execution-channel** durability (state survives one peer being briefly busy or restarting).

---

## The shape of the missing primitive (working hypothesis)

A `dialog` primitive scoped to a `conversation_id`:

| RPC | Purpose |
|---|---|
| `dialog.open` | Initiator declares peer + conversation_id; hub binds session pair, returns ack |
| `dialog.send` | Either side posts a turn into the conversation; bus persists per retention |
| `dialog.yield` | Sender signals "I'm pausing — expect my next turn within Δt" — receiver displays/waits without timing out |
| `dialog.resume` | Sender's signal that they are back and processing |
| `dialog.close` | Either side ends; hub releases binding |

**State machine (sketch):**
```
[opened] --send--> [active]
[active] --yield(Δt)--> [yielded]
[yielded] --resume--> [active]
[yielded] --timeout(Δt)--> [stale]
[active|yielded] --close--> [closed]
```

**Open questions for the spikes:**
- Is this a new typed Tier-B method block, or does `channel.post` with `conversation_id` metadata + a `yield` event type cover 80%? (NO-GO criterion)
- Where does the state machine live — hub authoritative, or agents authoritative with hub as transport?
- How does yield interact with hub restart? Conversation must survive auth heal mid-yield.
- What's the retention policy for an idle dialog? (TTL after last activity?)

---

## Use cases this primitive enables (downstream)

These are deferred — none are in scope for the inception itself, but the protocol must support them.

1. **Two LLM agents collaborating on a problem** (orchestrator ↔ specialist mid-task; the original T-233 motivation).
2. **Script error yielding** (the original T-243 framing) — script hits a partial failure mid-run, opens a dialog with a supervisor agent, gets a decision, resumes.
3. **Cross-host agent dialog** — agent on host A asking agent on host B for context that lives only on B.
4. **Human-in-the-loop interjection** — agent yields to a human via dialog; human types a decision; agent resumes. Same primitive, different peer.
5. **Confirmation-required actions** — agent about to take a Tier-1 action opens a dialog with the operator, awaits confirmation.

---

## Dialogue Log

### 2026-04-26 — Dialogue 1 (re-scoping)

**Operator opening:** "please what are we working that actually delivers value for our goals, purpose and intent of termlink?"

**Agent triage:** Scored active queue against TermLink's mission (agent-mesh coordination). Three tasks score for product value: T-243 (this), T-212 (homebrew distribution), T-1166 (legacy primitive retirement). Five score for hygiene/operational. Recommended T-243 as highest-leverage product work in the queue.

**Operator:** "yes" (proceed with T-243 inception).

**Agent first framing (wrong):** Took T-243 at face value — script error yielding via three options (checkpoints / error-stream events / PTY pause-and-ask). Presented a comparison table.

**Operator correction:** "no not completely, i wanted a means to simulate keyboard input and capture console output, because this enables several use cases for agentic engineering."

**Agent second framing (closer but still wrong):** Surveyed existing primitives — confirmed `termlink interact`, `pty inject/output`, `mirror`, `send` already cover input simulation + output capture for human↔terminal. Asked which gap was the real one: ergonomics / discoverability / reliability / yielding / something else.

**Operator clarification:** "3 we keep having key rotation issues, authentication issues, and interactive multi turn conversation between two or more agents is absolutely not working also related to 4."

**Agent reframing (current):** The real pain is two-layered — auth flake + missing multi-turn primitive. The "drive a CLI" framing was a side door; the actual goal is agent↔agent dialogue. Recommended split: T-1284 owns auth (G-011), T-243 owns the multi-turn primitive design.

**Operator:** "follow your recommendation."

**Agent action:** Created T-1284. Re-scoped T-243's task file. Created this artifact.

**Outstanding for Dialogue 1 continuation:**
- Concrete failing scenarios — which two agents have tried to talk, with what protocol, where did it break?
- Has anyone tried bootstrapping multi-turn with `channel.post` + a conversation_id convention? If so, what didn't work?
- What's the desired peer model — two named sessions, or session-discovery-by-capability?

### 2026-04-26 — Dialogue 2 (planned, not yet held)

Surface review of `agent.ask` / `request` / `channel.post`. Decide build-new vs. extend-existing.

---

## Spike plan (after dialogues complete)

- **Spike A** (1-2h): One-page protocol sketch for `dialog.*` RPCs. No code. Validates that the surface fits cleanly into the Tier-B typed-method pattern alongside `orchestrator.*` and `channel.*`.
- **Spike B** (2-3h): Minimal hub stub — accept `dialog.open`, store conversation_id binding in-memory, accept `dialog.send`, route to peer. No persistence yet. Validates that hub state can hold the binding without conflicting with existing routing.

Both spikes write findings back into this file before Go/No-Go decision.

---

## Decision (pending)

To be filled by `fw inception decide T-243 go|no-go --rationale "..."` after Spike B completes.
