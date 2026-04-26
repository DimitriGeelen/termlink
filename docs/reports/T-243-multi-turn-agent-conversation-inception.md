# T-243 Inception — Multi-Turn Agent Conversation Primitive over TermLink

**Status:** decided — GO (2026-04-26)
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

### 2026-04-26 — Dialogue 2 (operator redirect: do the analysis, don't ask for it)

**Operator opening:** "1 oh on several times, sorry do not have the instances anymore just the characteristics that it often is send and wait instead of immediate response and interaction. 2 no idea, open for suggestion, even consider to adapt a opensource chat protocol aka signal style. 3 hello conversation, please analyse how a conversation normally flows, and you know agents best."

**Three signals from this turn:**
1. The failure characteristic is **send-and-wait** (asynchronous IM pattern) where live interaction is wanted.
2. Open to adopting an existing open-source chat protocol — "Signal style" (referring to Signal-the-app, but invitation is broader).
3. Operator wants the agent (me) to do the analysis of conversational flow, not ask the operator.

**Analysis produced** (preserved in this file): conversation-flow three-layer model (content / activity signal / presence), agent-specific characteristics, scored comparison of Signal/XMPP/IRC/Matrix/MQTT, working hypothesis that **typing/processing signal is the killer feature** because the missing element is the activity layer.

**Operator response:** "1 yes and we should three agent incept this. 2 also her three agent incept this --> use termlink !!!"

→ Three-agent fan-out inception via TermLink.

### 2026-04-26 — Dialogue 3 (three-agent inception via TermLink)

**Method.** Spawned 3 ephemeral agents via `termlink_batch_run` (after `termlink_spawn` failed for ad-hoc bash commands — see Meta data point below). Each agent received a role-specific prompt, ran `claude -p --model sonnet`, returned ~500-word analysis. Wallclock: ~3 minutes total.

**Roles:**
- **Agent A — Protocol Architect** — choose between Matrix / XMPP / IRC / MQTT / Signal; argue decisively
- **Agent B — Liveness Specialist** — validate or challenge "typing signal is the killer feature"
- **Agent C — Pragmatic Skeptic** — argue for convention-layer + minimal extension over new typed-method namespace

#### Agent A — Protocol Architect (verbatim)

**Recommendation: Borrow Matrix's room model. Nothing else.**

Matrix wins because its room is a *conversation container*, not a topic bus. The distinction matters: a topic bus delivers messages to whoever is subscribed now; a room accumulates an ordered event log that any participant can replay. LLM agents need replay — their context window is their only memory.

**Borrow:** room as conversation unit; typed event catalog (`dialog.message`, `dialog.typing`, `dialog.receipt`, `dialog.presence`, `dialog.member`); sync token model (`since` cursor for resume after disconnect); state events vs. timeline events split.

**Skip:** federation, homeserver architecture, identity server (TermLink has its own auth). E2E encryption (hub-level auth sufficient). HTTP Matrix API surface (we keep TermLink MCP transport). Presence polling (derive from `dialog.presence` events).

**Reject the others outright:** Signal — async IM is the very anti-pattern operator is escaping; crypto is irrelevant noise. XMPP MUC — correct concepts but stanza verbosity is accidental complexity in a typed Rust mesh. IRC — no typed events, no resumability. MQTT — pub-sub without request-reply; LWT useful but not enough to anchor.

**Single biggest implementation risk: ordered delivery under concurrent writers.** The append-only log is worthless if two agents post simultaneously and each receives a different ordering. TermLink's channel layer must provide total order per room, or we must add a hub-side monotonic sequencer per room. If that becomes a bottleneck or SPOF, conversation throughput degrades exactly when load is highest.

**NO-GO trigger:** discover that TermLink channel retention silently drops events under hub restart, or that event ordering breaks with multiple simultaneous publishers. If we cannot guarantee ordered durable delivery at the channel layer, the append-only log model collapses entirely. In that case the correct path is to fix channel durability first (a separate task), then build dialog rooms on top of a proven foundation — not to paper over delivery gaps with application-level retries.

#### Agent B — Liveness Specialist (verbatim)

**Top 3 missing pieces (priority order):**

**#1 — Heartbeat/keepalive during processing.** The 30-second request timeout is not a UX problem — it is an actual failure mode. When an LLM turn takes 25s (tool use + reasoning), the caller's blocking `agent.ask` RPC can timeout, surface as an error, and the conversation breaks. The operator says "absolutely not working" — this is not a "feels slow" complaint, it's a "it dies" complaint. The fix is a progress heartbeat: every ~5s during processing, the responding agent emits a lightweight signal on the conversation channel. This serves two jobs simultaneously: (a) resets the caller's timeout clock, and (b) gives you the typing indicator for free as a side effect. **The typing/processing signal from the working hypothesis is correct but for the wrong reason. It's not cosmetic presence feedback — it's a keepalive mechanism that prevents real conversation death.**

**#2 — Push delivery (event-driven, not poll-on-response).** If `agent.ask` is polled (caller checks periodically for reply), then "immediate response" is structurally impossible — there's a polling-interval floor on every turn. With push, agent B's message arrives at agent A the instant it's sent. For a 2s LLM call, poll-at-1s = 0-1s extra latency; that compounds across a 10-turn conversation to 0-10s of pure overhead. Observable: shorten a 10-turn conversation clock by 20-30% with zero code changes to agents.

**#3 — Presence model (ready / busy / offline).** Without presence, agent A cannot know whether B is alive before starting a conversation. The operator's "send and wait" may include sends to agents that are offline or mid-reboot. A simple last-heartbeat TTL (>30s = presumed offline) lets the caller fail fast rather than block until timeout.

**Is typing signal really #1? No.** It's a downstream rendering concern. The upstream problem is: if transport is polling, add push then indicators are trivial; if timeout is 30s hard cut, add heartbeat then indicators are trivial. **Shipping a typing indicator without fixing polling or timeout gives the operator a progress bar on a conversation that still dies at 29 seconds. That's worse — it raises expectations and then fails visibly.**

**Correct sequence: heartbeat → push → presence → typing indicator (emerges from #1 automatically).**

**Concrete observable for day 1 of heartbeat/keepalive:** A conversation where agent B uses 3 tools before replying currently breaks ~40% of the time at the 30s mark. With heartbeat, that conversation completes 100% of the time, and the caller sees "B is processing..." instead of silence. No UI changes required — just fewer broken conversations.

**What would convince me I'm wrong:** if the operator confirms *"Conversations always complete — I just can't tell if the other agent received my message."* That would mean transport and timeout are fine, and the problem is purely perceptual presence. In that case the typing indicator is the right and sufficient fix. But "absolutely not working" is not perceptual language — it's failure language.

#### Agent C — Pragmatic Skeptic (verbatim)

**The Real Diagnosis.** The failure ("send and wait instead of immediate response") is a **usage pattern bug**, not a missing primitive. Agents are calling `agent.ask` or `request` (blocking) instead of `channel.subscribe` (async). That's a documentation and convention problem. Building a new namespace on top of a misuse pattern entrenches the misuse.

**Cheapest possible extension. Zero new methods. One metadata convention. Three event types.**

What changes in code: `channel.post` accepts optional `metadata.conversation_id: string` (one field). Optional `metadata.event_type: "turn" | "typing" | "receipt"` (routing hint, not enforcement).

What is pure convention (zero code): threads = channels scoped by `conversation_id` filter on subscribe; turn-taking = post `turn`, await `turn` from peer; human-in-the-loop = human posts `turn` to channel; confirmation = post `turn` with `requires_ack: true`, peer posts `receipt`. Covers the four stated use cases using infrastructure that already exists, is already signed, has retention, and supports multi-subscriber.

**Coverage estimate:** ~85% of stated use cases. Agent-to-agent multi-turn 95%; script-yielding to human 85%; human-in-the-loop confirmation 85%; collaboration (N agents) 75%. Overall good enough to ship, learn, iterate.

**One concrete failure: liveness under crash.** Agent A posts `typing`. Agent B crashes before posting `turn`. Agent A subscribes and waits. Forever. The convention-layer `typing` event has no heartbeat, no TTL, no eviction. Application code must implement its own timeout loop — and every consumer reimplements it differently. **When you find two or more independent consumers writing heartbeat-timeout logic to work around this, the infrastructure has a gap the convention layer can't paper over.**

**Decision criterion — one trigger: replication of liveness logic.** If a post-ship audit finds that ≥2 independent consumers have written their own "typing-timeout / dead-participant eviction" code, a first-class conversation primitive with infrastructure-owned heartbeat is justified. Not before.

Secondary trigger: if Watchtower needs to display "active conversations" without replaying full channel history, a server-side conversation object becomes necessary. Observable state that requires event replay is a data model smell.

Until either trigger fires: ship the metadata convention, document the subscriber pattern, fix the agents that are calling `agent.ask` in a loop. **The `dialog.*` namespace is a solution looking for a problem that hasn't been measured yet.**

---

## Synthesis (orchestrator)

**Convergence (all 3 agree):**
- Multi-turn dialog needs *some* structural support beyond current single-shot RPCs
- Typing/processing/heartbeat signals are central to the fix
- `channel.*` is the right foundation
- Ordered, durable delivery is the underlying enabler — A names it as the NO-GO check; C names it as "liveness under crash"; B's heartbeat depends on it

**Divergence:**
- A: new typed-method surface (`dialog.*`), borrow Matrix decisively
- C: convention layer + one metadata field, new surface only when ≥2 consumers reimplement timeout logic
- B: reframes priority — typing signal is real but **for the wrong reason**; it's a *keepalive* preventing 30s-timeout death, not cosmetic presence feedback

**Resolution.** B's reframing tips the A vs C debate. C's own conversion trigger is *"if ≥2 independent consumers reimplement timeout logic."* B's analysis says every consumer **will** need that timeout loop because the 30s default kills real conversations. **C's trigger is met before deployment.** But A's full `dialog.*` surface is more than needed.

The honest synthesis is a *thin* first-class layer:

| Layer | Decision | Rationale |
|---|---|---|
| **Channel foundation** (`channel.*`) | Audit + harden ordered/durable delivery first | A's NO-GO + C's crash gap + B's heartbeat all depend on it |
| **Heartbeat / keepalive** | **Must be infrastructure** — single new typed RPC or hub-tracked invariant | B's "real failure mode"; C's own trigger; cannot be convention |
| **Event type catalog** (`turn`, `typing`, `receipt`, `presence`, `member`) | **Convention** over `channel.post` with `metadata.conversation_id` + `event_type` | C's minimal-surface path; works once heartbeat exists |
| **Push delivery / streaming subscribe** | Verify it exists; if poll-based, separate enabling task | B's #2 — without push, "immediate response" is structurally impossible |

**Keepalive is the only piece that *must* be infrastructure. Everything else is convention.** This is a smaller surface than Matrix-borrow-everything, larger than convention-only.

---

## Decision

**GO** (recorded 2026-04-26).

**Rationale.** Three-agent inception via TermLink converged on a clear protocol shape. Operator confirmed direction. Synthesis identifies a small infrastructure delta (heartbeat as first-class) plus convention-layer extensions on `channel.*` — minimal new surface for maximum coverage. T-1284 is in flight for the auth foundation. Build decomposition is clear and independently testable.

**Build decomposition (child tasks, see related_tasks):**
1. Audit + harden `channel.*` ordered durable delivery — A's NO-GO check and C's crash gap. Foundation.
2. `dialog.heartbeat` typed RPC (or hub-tracked invariant) — the only must-be-infrastructure piece per B + C.
3. Extend `channel.post` with `metadata.conversation_id` + `metadata.event_type` — C's one-field change.
4. Convention catalog documentation — `turn`, `typing`, `receipt`, `presence`, `member` event types and recommended subscriber patterns. Documentation, no code.
5. Verify push delivery is the default subscription mode; if poll-based, separate task to flip it.

---

## Meta data point: TermLink dispatch friction during this very inception

`termlink_spawn` for ad-hoc bash commands timed out waiting for hub registration — sessions need to *be* TermLink agents (speak the protocol, register with hub) to register cleanly. Tried twice; both rounds: `ok: true, status: timeout`, no output produced.

`termlink_batch_run` worked perfectly first try — it's designed for ephemeral fan-out with result collection.

**This is the same shape mismatch T-243 itself addresses.** TermLink today is great at "fire command, collect result" (one-shot fan-out) and weak at "two agents hold a conversation" (long-running, multi-turn, registered peers). The 3-agent fan-out succeeded; a 3-agent *dialog* would not have. The dispatch experience is itself evidence supporting GO.

---

## Spike plan (superseded by GO decision)

The originally planned spikes (Spike A protocol sketch, Spike B hub stub) are no longer needed — the three-agent inception produced equivalent or better material. Build proceeds via the child tasks listed above.
