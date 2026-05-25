# T-1800 Inception — Interactive Agent Conversation Runtime (deterministic doorbell+mail auto-pickup loop)

**Status:** exploring — advisory recommendation GO (2026-05-25)
**Focus question (one inception = one question):** How do we make interactive multi-turn conversation between 2+ agents *actually happen* at runtime, given that T-243's message/protocol layer is already shipped but nothing wakes a turn-based agent when a turn arrives?
**Parent arc:** T-243 (multi-turn dialog protocol, GO 2026-04-26) → T-690 (push event delivery, GO) → T-256 (interactive multi-agent comms). This inception owns the **runtime/behavioral loop**, which those explicitly scoped OUT.
**Operator framing (2026-05-25):** "interactive conversation between two or more agents… send-and-wait instead of immediate response… we want this interactive style of conversation to take place." Called "key functionality."

---

## 1. Why this exists — the runtime gap

The protocol is done. The behavior is not.

**Shipped and live** (verified in `crates/termlink-hub/src/channel.rs`, not inferred):

| Capability | Where | Task |
|---|---|---|
| `channel.subscribe` long-poll (`timeout_ms`, blocks until a post arrives) | channel.rs:554 | T-1289 |
| `conversation_id` filter on subscribe | channel.rs:563 | T-1287 |
| `event_type` catalog: `turn`/`typing`/`receipt`/`presence`/`member` | channel.rs:465 | T-1288 |
| `dialog.presence` passive tracker (who's active in a dialog) | channel.rs:29 | T-1286 |
| ordered durable delivery + gap detection (`oldest_offset`) | — | T-1285 |
| threaded replies (`in_reply_to`) + MCP metadata exposure | — | T-1313 / T-1692 |
| convention spec with worked Alice↔Bob heartbeat example | `docs/conventions/multi-turn-dialog.md` | T-1288 |

**The gap:** the only receive-side machinery is two skills — `/agent-handoff` (send) and `/check-arc` (a **manual** "show me unread" command). There is **no** listen-loop, **no** auto-pickup, **no** auto-respond, and **no** mechanism that wakes/invokes an agent when a turn arrives. (`bus-handler.sh`, called "dormant… never activated" in the T-256 report, no longer exists.)

Consequence: an agent can *post* a turn and the hub will *push* it instantly to anyone long-polling — but **nobody long-polls, because a Claude Code agent only acts when the human prompts it.** The keepalive, presence, and conversation_id machinery is dressed up with no runtime to drive it. That is precisely why it still feels like "send-and-wait."

**This inception's single deliverable:** a decision (+ design) on the runtime loop that turns the shipped protocol into a live A↔B (and A↔B↔C) conversation.

---

## 2. Design space

The hard constraint: **Claude Code is turn-based** — it runs, then halts until invoked. Something must drive "a turn arrived → the agent reads it → responds → re-arms." Candidate mechanisms:

| # | Mechanism | Wake source | Transport for message content | Pros | Cons / risk |
|---|---|---|---|---|---|
| **1 Doorbell + mail** *(leading)* | `command.inject` a one-line nudge into receiver's PTY | `channel.*` (structured turn) | Solves auto-pickup with shipped primitives; reply comes back **structured** (no screen-scrape); keeps auth/identity/threading | Receiver must be a PTY-backed hub session; injection collides if receiver mid-turn / dialog open (eventual, not instant pickup) |
| **2 Pure-PTY** | `command.inject` the **whole message** | the PTY itself (scrollback) | No channel needed; dead simple to start | **No reliable turn-completion** for an LLM (can't append `; echo MARKER`); must **screen-scrape the TUI** for the reply; loses all structure/auth; the "wrong layer" T-243 warned about |
| **3 Background long-poll + harness wake** | agent backgrounds `channel.subscribe --timeout`; harness notifies on completion | `channel.*` | Pure in-session; no terminal injection | Tied to a live interactive session; couples to harness notification semantics |
| **4 External daemon** | a wrapper watches the topic, pipes each turn into `claude -p`, posts reply back | `channel.*` | Fully autonomous A↔B with no human; clean headless model | New long-running supervised component; no PTY (so no live TUI to observe) |
| **5 Hook-based injection** | Stop/Notification hook drains pending turns at turn boundaries | `channel.*` | Lightest | Only nudges *between* turns — not truly live |

**Operator decision so far (2026-05-25):** pursue **#1 doorbell+mail**, with a deep inception. #2 pure-PTY explicitly questioned for the scraping/turn-completion fragility. #4 daemon is the likely "hands-off autonomous" follow-on once #1 proves the loop.

---

## 3. Leading architecture — doorbell + mail

**Roles split cleanly:**
- **Mail = `channel.*`** — the *content* of every turn (signed envelope, `conversation_id`, `event_type=turn`, threading). Already built. Never changes.
- **Doorbell = `command.inject`** — a *wake* signal only. A tiny, fixed nudge (e.g. inject `/check-arc` + Enter) into the receiver's PTY. Never carries message content.

**Happy-path round-trip (A → B):**
1. A: `channel.post` a `turn` on the conversation topic → hub returns accepted offset.
2. A: `command.inject` the doorbell (`/check-arc`) into B's PTY session.
3. B wakes (injected stdin = a turn), runs `/check-arc`, reads the **structured** turn via `channel.subscribe`, responds with its own `channel.post turn`, and emits a `receipt`.
4. A's pending `channel.subscribe` (long-poll, conversation_id filter) wakes on B's reply. Loop.

Why this beats pure-PTY: **B's reply never has to be scraped from the terminal** — it comes back as a structured envelope. The PTY is only ever a doorbell, so its fragility is bounded to "did a one-liner land," not "can I parse a 2000-token answer out of an ANSI TUI."

---

## 4. The two operator questions, as design decisions

### Q-A — Does this work with `claude`, or must `claude-fw` be started?

**Finding (verified):** `command.inject` / `query.output` require the target session to have a PTY — `ctx.pty: Option<Arc<PtySession>>` in `crates/termlink-session/src/handler.rs:25`; `has_pty` is `false` when `None` (handler.rs:273). A plain `claude` launched in your own terminal is **not a TermLink session at all** — the hub cannot target it, so the doorbell cannot ring.

**Decision shape:**
- The **receiver** must run inside a **hub-registered PTY session** — launched via `termlink spawn --backend tmux … claude …` (T-256 learning: tmux backend + `--permission-mode bypassPermissions` works for Claude instances; `--backend background` does not — no real terminal).
- **`claude-fw` is orthogonal.** It is the auto-restart wrapper (T-179): runs `claude`, auto-restarts on the handover/restart signal. It does **not** create PTY registration. It *is* valuable for a long-lived *listening* agent (survives context-compaction restarts so the conversation participant doesn't vanish mid-dialog), but it is not what enables injectability.
- **Open (spike S-3):** the exact, reproducible recipe to bring up an injectable Claude listener — `termlink spawn` flags, permission mode, and whether `claude` vs `claude-fw` should be the spawned command for a durable participant.

### Q-B — Can we wire it so it's a *deterministic* workflow (doorbell guaranteed rung after message sent)?

**Yes — zero protocol redesign**, by composing existing pieces into one verb plus the catalog's `receipt`:

1. **Atomic send verb** — a single `termlink agent <send-verb>` that sequences: `channel.post turn` → **confirm hub-accepted offset** → only then `command.inject` the doorbell referencing that offset. Post fails ⇒ no ring (no false doorbell). Ring fails ⇒ error returned to caller (caller knows to retry). This removes the naive two-step race ("nudge before post is visible" / "post ok but nudge lost").
2. **Closed-loop ack** — receiver emits `event_type=receipt` (already in the catalog) on pickup. Sender can wait for the receipt ⇒ positive confirmation B got it.
3. **Bounded re-ring** — if no receipt within Δt, re-ring (B may have been mid-turn). Bounded retries, then surface "peer unreachable."

**Determinism ceiling (must characterize — spike S-2):** if B is mid-turn or showing a permission dialog, injected text **queues** until B is prompt-ready. So delivery is *eventual*, not *instant*, and a doorbell injected during a permission prompt could be mis-consumed. The receipt+re-ring loop converts this into eventual-consistent-with-confirmation, but the inception must measure the worst-case latency and the permission-dialog failure mode and decide whether that bound is acceptable for "interactive."

---

## 5. Assumptions to validate

- **A-1:** A Claude Code session spawned via `termlink spawn --backend tmux` is reliably injectable (`has_pty=true`) and a `/check-arc`-style doorbell injected at its prompt is picked up as a normal turn. *(Spike S-1)*
- **A-2:** When the receiver is mid-turn, an injected doorbell queues and is consumed cleanly after the current turn (no corruption, bounded latency). *(Spike S-2)*
- **A-3:** A receipt-confirmed, re-ring-on-timeout send verb yields deterministic delivery (sender always learns delivered-or-failed) without races. *(Spike S-4)*
- **A-4:** The reply round-trips as a **structured** `channel.*` envelope — the PTY is never scraped for content. *(design invariant; verified by S-1)*
- **A-5:** Permission-dialog / TUI-state collision is either avoidable (prompt-ready detection) or acceptably rare. *(Spike S-2)*

## 6. Spikes (each carries NO-GO authority)

| Spike | Question | NO-GO trigger |
|---|---|---|
| **S-1** Injectable listener | Spawn a Claude listener via `termlink spawn --backend tmux`; inject `/check-arc`; confirm it wakes + reads a structured turn | If a spawned Claude cannot be reliably woken by injection at all → doorbell+mail collapses; fall back to #4 daemon |
| **S-2** Mid-turn robustness | Inject while receiver is mid-turn / has a permission dialog; measure pickup latency + corruption | If injection corrupts state or is silently dropped with no recovery → determinism unachievable via PTY; reconsider #3/#4 |
| **S-3** Registration recipe | Nail the exact spawn flags / permission mode / claude-vs-claude-fw for a durable participant | (informational; no kill) |
| **S-4** Deterministic verb | Prototype atomic post→accept→ring + receipt ack + bounded re-ring; prove no race, sender always learns outcome | If receipts can't be correlated to the originating turn reliably → determinism story weaker, document the gap |
| **S-5** Two-way live demo | Two real Claude sessions hold a ≥3-turn conversation end-to-end with heartbeats | If a live 3-turn exchange cannot complete → the runtime loop is not yet viable; iterate before GO |

## 7. Go / No-Go criteria

**GO (build the runtime loop) if:** S-1 + S-5 pass (a spawned listener wakes on a doorbell and two agents complete a ≥3-turn structured conversation), AND S-4 shows the deterministic send verb has no fundamental race. The new surface is then small: one `agent` send-verb (atomic post+ring+ack) + making `/check-arc` the standard pickup-and-respond ritual + a documented spawn recipe. **No protocol changes.**

**NO-GO / pivot to #4 daemon if:** S-1 or S-2 shows PTY injection into a Claude session is fundamentally unreliable (can't wake, or corrupts state with no bounded recovery). In that case the message layer is still correct; only the wake mechanism changes to an out-of-session daemon driving `claude -p`.

**Scope OUT:** any change to `channel.*` / the dialog protocol (already shipped); the fully-autonomous no-human daemon (#4) is a *follow-on* build, not this inception.

---

## 8. Dialogue Log

### 2026-05-25 — Dialogue 1 (locating the arc)
- **Operator:** "didn't we have an arc for improving dynamic, responsive chat between agents, where we were thinking of using SSE? — first reflect and play back."
- **Agent:** Located the arc: T-243 (multi-turn conversation, GO) + T-690 (push delivery, GO) + T-256 (interactive comms). **Correction surfaced:** the chat arc chose long-poll + broadcast channel, **not** SSE; SSE lives only in unrelated contexts (T-908 API-relay, T-1122 Watchtower web UI, T-004 MCP transport). Operator's "SSE" memory was bridging the web/relay work into the chat arc.
- **Operator:** confirmed the arc; "SSE is not mandatory — the goal is push messages + automatic pickup for truly dynamic, interactive conversation."

### 2026-05-25 — Dialogue 2 (the real gap)
- **Agent:** Verified T-243's entire build decomposition is CLOSED (T-1285/86/87/88/89 + T-1313/1692) — protocol shipped end-to-end. The gap is the runtime loop: nothing wakes a turn-based agent; only `/check-arc` (manual) exists.
- **Operator:** "this interactive style of conversation [is what we] want to take place." Confirmed: protocol done, the missing thing is the auto-pickup/respond runtime loop.

### 2026-05-25 — Dialogue 3 (PTY injection redirect)
- **Operator:** "what can we do with pty inject or other terminal inject — do we need to redesign the protocol?"
- **Agent:** Verified `command.inject`/`query.output`/`interact` exist, cross-host, require `has_pty`. **No protocol redesign needed.** Proposed **doorbell+mail** split: PTY inject as the *wake*, `channel.*` as the *content* — avoids the pure-PTY scraping/turn-completion trap.
- **Operator:** "1 YES [to the split]. A: does this work with claude or does claude-fw need to be started? B: can we wire it so it becomes a deterministic workflow (ensure the doorbell is rung after the message is sent)? 2 yes make it a deep inception, it's key functionality."
- **Agent:** Answered A (receiver must be a PTY-backed hub session via `termlink spawn --backend tmux`; claude-fw orthogonal/resilience-only) and B (deterministic via atomic post→accept→ring verb + receipt ack + bounded re-ring; ceiling = receiver runtime state). Filed this inception (T-1800, advisory GO).

---

## 9. Decision

Pending. Advisory recommendation: **GO** to pursue (build the doorbell+mail runtime loop), contingent on spikes S-1/S-4/S-5. Human decides via `fw inception decide T-1800 go|no-go|defer`. Per inception discipline, no build artifacts until that decision.
