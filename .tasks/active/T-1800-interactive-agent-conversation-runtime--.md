---
id: T-1800
name: "Interactive agent conversation runtime — deterministic doorbell+mail auto-pickup loop (close T-243 runtime gap)"
description: >
  Inception: Interactive agent conversation runtime — deterministic doorbell+mail auto-pickup loop (close T-243 runtime gap)

status: started-work
workflow_type: inception
owner: human
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-05-25T09:41:44Z
last_update: 2026-05-25T09:42:47Z
date_finished: null
---

# T-1800: Interactive agent conversation runtime — deterministic doorbell+mail auto-pickup loop (close T-243 runtime gap)

## Problem Statement

Two or more agents cannot hold a live, interactive conversation, even though
T-243's message/protocol layer is fully shipped (channel.* with
`conversation_id`/`event_type`/long-poll/`dialog.presence`/threading — verified
in `crates/termlink-hub/src/channel.rs`). The gap is purely **runtime**: a
Claude Code agent is turn-based (acts only when prompted), and there is **no**
mechanism that wakes it when a turn arrives. The only receive-side machinery is
`/check-arc` (manual). So the hub pushes turns instantly but nobody is
listening — the operator experiences "send-and-wait instead of immediate
response." For: any multi-agent workflow (orchestrator↔specialist, cross-host
dialog, human-in-the-loop). Why now: operator flagged it as "key functionality."

Full design artifact: `docs/reports/T-1800-interactive-agent-conversation-runtime-inception.md`.

## Assumptions

- **A-1:** A Claude session spawned via `termlink spawn --backend tmux` is reliably injectable (`has_pty=true`) and an injected `/check-arc` doorbell is picked up as a normal turn.
- **A-2:** When the receiver is mid-turn, an injected doorbell queues and is consumed cleanly after the current turn (no corruption, bounded latency).
- **A-3:** Atomic post→accept→ring + `receipt` ack + bounded re-ring gives deterministic delivery (sender always learns delivered-or-failed) without races.
- **A-4:** The reply round-trips as a **structured** `channel.*` envelope — the PTY is never scraped for content (design invariant).
- **A-5:** Permission-dialog / TUI-state collision is avoidable (prompt-ready detection) or acceptably rare.

## Exploration Plan

Spikes (each carries NO-GO authority; detail + triggers in the artifact):
- **S-1** Spawn a Claude listener via tmux backend; inject `/check-arc`; confirm wake + structured read. (timebox 45m)
- **S-2** Inject mid-turn / during a permission dialog; measure pickup latency + corruption. (45m)
- **S-3** Nail the exact spawn recipe (flags / permission mode / claude vs claude-fw). (30m)
- **S-4** Prototype the deterministic send verb (atomic post→accept→ring + receipt ack + bounded re-ring); prove no race. (60m)
- **S-5** Two real Claude sessions hold a ≥3-turn conversation end-to-end with heartbeats. (60m)

## Technical Constraints

- **Receiver must be a hub-registered PTY session.** `command.inject`/`query.output` require `ctx.pty: Some(..)` (`crates/termlink-session/src/handler.rs:25`, `has_pty` at :273). Plain `claude` in a user terminal is not a TermLink session and cannot be targeted.
- **Claude Code is turn-based** — injected stdin is consumed only when the input box is prompt-ready; mid-turn or permission-dialog states defer/contend with pickup (eventual, not instant).
- **No protocol changes permitted** — `channel.*` / dialog protocol is shipped and frozen for this work; the runtime loop composes existing primitives only.
- `claude-fw` (auto-restart wrapper, T-179) is orthogonal — resilience for a long-lived listener, not what enables injection.

## Scope Fence

**IN:** the wake/pickup/respond runtime loop (doorbell+mail); one atomic send verb (post+ring+receipt ack + bounded re-ring); making `/check-arc` the standard pickup-and-respond ritual; a documented injectable-listener spawn recipe.
**OUT:** any change to `channel.*` or the dialog protocol (already shipped); the fully-autonomous no-human daemon driving `claude -p` (candidate #4 — a *follow-on* build if/when hands-off is wanted); auth hardening (T-1284/G-011).

## Acceptance Criteria

### Agent
<!-- @auto-tick-on-decide -->
- [ ] Problem statement validated
<!-- @auto-tick-on-decide -->
- [ ] Assumptions tested
<!-- @auto-tick-on-decide -->
- [ ] Recommendation written with rationale

### Human
<!-- @auto-tick-on-decide -->
- [ ] [REVIEW] Review exploration findings and approve go/no-go decision
  **Steps:**
  1. Run: `fw task review T-XXX` (opens Watchtower with recommendation, assumptions, research artifacts)
  2. Review the Agent Recommendation section and go/no-go criteria evaluation
  3. Record decision via the Watchtower form or the command shown alongside the QR code
  **Expected:** Decision recorded, task completed
  **If not:** Ask agent for clarification on specific findings

## Go/No-Go Criteria

<!-- Fill these BEFORE writing the recommendation. The placeholder detector will block review/decide if left empty. -->
**GO if:**
- S-1 + S-5 pass: a spawned Claude listener wakes on an injected doorbell AND two agents complete a ≥3-turn structured conversation with heartbeats
- S-4 shows the deterministic send verb (post→accept→ring + receipt ack + re-ring) has no fundamental race
- New surface stays small: one send verb + `/check-arc` ritual + spawn recipe — NO protocol changes

**NO-GO / pivot to daemon (#4) if:**
- S-1 or S-2 shows PTY injection into a Claude session is fundamentally unreliable (can't wake, or corrupts state with no bounded recovery)
- In that case the message layer is still correct; only the wake mechanism changes to an out-of-session daemon driving `claude -p`

## Verification

# Shell commands that MUST pass before work-completed. One per line.
# Lines starting with # are comments (skipped). Empty lines ignored.
# For inception tasks, verification is often not needed (decisions, not code).
#
# Toolchain hint (L-291): if a GO decision will mean editing *.vbproj/*.csproj/*.xaml,
# *.go, Cargo.toml, tsconfig.json, or pom.xml in the build task, plan to add the
# matching build command (dotnet build / go build / cargo check / tsc --noEmit /
# mvn compile) to that build task's ## Verification — P-011 only runs what you write.

## Recommendation

**Recommendation:** GO

**Rationale:**

T-243's message/protocol layer is fully shipped and live (channel.* with conversation_id/event_type/long-poll/presence + dialog.presence + threading; verified in crates/termlink-hub/src/channel.rs). The remaining gap is purely runtime: nothing wakes a turn-based Claude agent when a turn arrives. The doorbell+mail design (structured turn over channel.* + command.inject doorbell into the receiver's PTY + receipt-based ack) composes ONLY already-shipped primitives — command.inject/query.output exist and are cross-host, receipt is in the catalog — so no protocol redesign is needed. High operator value ('key functionality'). GO to pursue; named spikes (injection robustness vs turn-completion, determinism ceiling when receiver mid-turn, exact PTY-session registration recipe) carry NO-GO authority if the runtime loop proves too fragile.

**Evidence:**

<!-- Add evidence bullets as exploration progresses (file paths,
     commit hashes, test results). The filing-time recommendation
     can be revised before fw inception decide. -->

## Decisions

<!-- Record decisions ONLY when choosing between alternatives.
     Skip for tasks with no meaningful choices.
     Format:
     ### [date] — [topic]
     - **Chose:** [what was decided]
     - **Why:** [rationale]
     - **Rejected:** [alternatives and why not]
-->

## Decision

<!-- Filled at completion via: fw inception decide T-XXX go|no-go --rationale "..." -->

## Updates

<!-- Auto-populated by git mining at task completion.
     Manual entries optional during execution. -->

### 2026-05-25T09:42:47Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
