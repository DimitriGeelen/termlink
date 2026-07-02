---
id: T-2315
name: "arc-004 wake-path integration: consume WS push in the live doorbell/be-reachable path to replace the 15s poll floor"
description: >
  Inception: arc-004 wake-path integration: consume WS push in the live doorbell/be-reachable path to replace the 15s poll floor

status: work-completed
workflow_type: inception
owner: human
horizon: null
tags: [arc:push-transport]
arc_id: push-transport
components: []
related_tasks: [T-2303, T-2309, T-2313, T-2314]
created: 2026-07-02T20:51:50Z
last_update: 2026-07-02T21:10:46Z
date_finished: 2026-07-02T21:10:46Z
# revisit_at: YYYY-MM-DD          # T-1451: set on DEFER decisions to enable G-053 daily revisit scan
# revisit_evidence_needed:        # T-1451: one-line description of what evidence makes the revisit actionable
# ── Inception scoring exception (T-2186 Slice 2 / T-2188). See 050-Inceptions.md §Scoring Exception. ──
target_blast_radius: 3            # int 0..9. Anticipated component count of the build work this inception would authorise on GO.
                                  # Substitutes for the absent components: list in the F8 cost formula (040). Required.
                                  # Guide: 0=docs only, 1=single file, 3=small subsystem (S), 5=cross-subsystem (M), 7=multi-arc (L), 9=framework-wide (XL).
voi_score: 0.5                    # float 0..1. Value of Information — expected value of resolving this question,
                                  # independent of build cost. Higher when answer affects many tasks or unblocks a strategic decision. Required.
---

# T-2315: arc-004 wake-path integration: consume WS push in the live doorbell/be-reachable path to replace the 15s poll floor

## Problem Statement

arc-004 shipped a working, proven hub→client WebSocket push (`channel subscribe <topic>
--push`): a DM produces an `inbox.queued` frame the instant it is posted (~88 ms TCP /
31 ms Unix), it degrades to poll on drop, and it now auto-reconnects after a blip (T-2314).
**But nothing in the live agent wake path consumes it** — the `--push` verb is exercised
only by demo scripts. A reachable agent (`/be-reachable`) is still woken by the doorbell+mail
loop's poll on the **15 s floor** the arc set out to remove (T-2303 §10.1). The arc's
headline value — instant DM delivery to a live agent — is therefore **demonstrated but not
delivered**. Question: should the live wake path adopt the WS push, and if so how does a
pushed `inbox.queued` frame wake a Claude Code PTY session without regressing the T-1800
doorbell+mail durability? Full analysis: `docs/reports/T-2315-arc-004-wake-path-integration-inception.md`.

## Assumptions

- The hub already emits an `inbox.queued` aggregator event for every `inbox:<self>` DM
  (the exact frame `--push` streams) — so no hub-side change is needed (validated by the
  T-2310/T-2313 demos, which push on `inbox.queued`).
- The existing PTY doorbell ring (`be-reachable --pty-session`, T-1834) can be fired by a
  background process on demand — so a push waker reuses it rather than inventing a new wake.

## Open Questions

- **IW-1: Should the live wake path adopt the WS push at all (vs keep the 15 s poll)?**
  confidence: 2
  disposition: deferred
  rationale: sensitive wake-path area (T-1800/T-2285); human go/no-go via `fw task review T-2315`.

- **IW-2: Which shape — add a push-waker beside the poll (Option A) or replace the sender-side poll (Option B)?**
  confidence: 2
  disposition: deferred
  rationale: analysis favours A (additive, reversible, lower blast radius); confirm in build. See artifact §Options.

- **IW-3: How is double-wake avoided when both push and poll can fire for the same DM?**
  confidence: 1
  disposition: deferred
  rationale: the receipt/ack is the natural idempotency key; WP2 loopback test must prove no double-wake, no lost DM. Ties to T-2285 ack-gap.

<!-- T-2190 (T-2186 Slice 4): every IW-N question must be disposed before
     --status work-completed. Disposition gate (agents/task-create/update-task.sh
     check_disposition_gate) refuses on under-disposed inceptions.

     Per-question shape:

       - **IW-1: <question text>**
         confidence: 0-3      (your confidence in your current answer; 0=guess, 3=verified)
         disposition: answered | deferred | dissolved
         rationale: <one-line evidence — file:line, decision id, dialogue ref>

     Never bare yes/no — the gate refuses bare checkboxes. See 050-Inceptions.md
     §Disposition Gate. Bypass: --skip-disposition-gate "rationale" (direct) or
     FW_SKIP_DISPOSITION_GATE=1 (env-var, T-1890 producer/consumer parity).
-->

## Exploration Plan

Code-grounded (already done during filing — see artifact §"The wake path today"):
1. Map the current wake path: `be-reachable.sh` (`--pty-session` ring target),
   `agent-send.sh` (sender poll + PTY inject), `agent-respond.sh` (receiver half). ✅
2. Confirm `inbox.queued` is the frame a self-inbox `--push` streams (T-2310/T-2313). ✅
3. Enumerate integration options (A add-a-waker / B replace-the-poll / C null) + score. ✅
Remaining validation deferred to build (WP2 loopback test): sub-second PTY wake via push +
clean degrade-to-poll with no double-wake.

## Technical Constraints

- **A Claude Code session is not a daemon** — it cannot itself hold a WS open; a *separate*
  background process (the be-reachable lifecycle already runs one) must own the subscription
  and ring the PTY. This is the same constraint that shaped the doorbell design (A1, T-2303).
- **Durability layer is off-limits** (T-2303 §10.3): offline-queue / idempotency /
  delivery-confirm / journal / receipts stay authoritative. This changes only *when* the
  wake fires, never the durable inbox.
- **Wake path is sensitive:** T-1800 doorbell+mail, T-2285 ack-with-retry gap, and a
  §5-rejected flag-poll design all live here — any change must be additive + reversible.

## Scope Fence

**IN:** whether/how the live wake path consumes the already-shipped WS push; the shape
(add-a-waker vs replace-the-poll); the double-wake dedup question.
**OUT:** webhooks (A1 — separate deferred external-only inception); any change to the
durability layer; the hub side (already emits `inbox.queued`); building the waker (that is
a post-GO build task, WP1/WP2).

## Acceptance Criteria

### Agent
<!-- @auto-tick-on-decide -->
- [x] Problem statement validated
<!-- @auto-tick-on-decide -->
- [x] Assumptions tested
<!-- @auto-tick-on-decide -->
- [x] Recommendation written with rationale

### Human
<!-- @auto-tick-on-decide -->
- [x] [REVIEW] Review exploration findings and approve go/no-go decision
  **Steps:**
  1. Run: `fw task review T-XXX` (opens Watchtower with recommendation, assumptions, research artifacts)
  2. Review the Agent Recommendation section and go/no-go criteria evaluation
  3. Record decision via the Watchtower form or the command shown alongside the QR code
  **Expected:** Decision recorded, task completed
  **If not:** Ask agent for clarification on specific findings

## Go/No-Go Criteria

<!-- Fill these BEFORE writing the recommendation. The placeholder detector will block review/decide if left empty. -->
**GO if:**
- The integration is additive (adds a faster wake trigger beside the existing poll) and
  reversible (stop the waker → back to the current poll), leaving the durable substrate untouched.
- A bounded build path exists reusing the proven push consumer + the proven PTY ring
  (Option A, slices WP1/WP2).
- Double-wake can be made idempotent via the existing receipt/ack (IW-3 answerable in build).

**NO-GO if:**
- Delivering the value would require re-architecting the load-bearing T-1800 doorbell+mail
  handshake (e.g. Option B on the sender side) rather than adding a trigger.
- The 15 s→sub-second wake gain does not justify a second long-lived per-session process.
- Double-wake / lost-DM cannot be ruled out without touching the durability layer.

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

GO (advisory): the WS push mechanism is shipped + proven (S1-S4, S3b, T-2313, T-2314) but no live agent consumes it — the doorbell/be-reachable wake path still polls on the 15s floor, so the arc's headline value (instant DM delivery) is demonstrated but not delivered. Wiring a background 'channel subscribe inbox:<self> --push' consumer that rings the existing PTY doorbell on inbox.queued (degrading to the current poll if WS drops — T-2314 auto-recovers) is bounded, additive, reversible. Sensitive wake-path area (T-1800 doorbell+mail, T-2285 ack-gap, prior rejected flag-poll design) needs a human GO before build.

**Evidence:**

- Research artifact: `docs/reports/T-2315-arc-004-wake-path-integration-inception.md`
  (grounded in `be-reachable.sh`, `agent-send.sh`, `agent-respond.sh`).
- Shipped + proven push mechanism this arc depends on: T-2310 (TCP demo, 88 ms),
  T-2313 (WS-over-Unix, 31 ms), T-2314 (active reconnect after blip — commit ed956222).
- Hub already emits `inbox.queued` for self-inbox DMs (channel.rs:753, T-1637) — no
  hub change needed; the demos push on exactly this frame.
- The 15 s poll floor being replaced is the T-2303 §10.1 named target.

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

**Decision**: GO

**Rationale**: GO (advisory): the WS push mechanism is shipped + proven (S1-S4, S3b, T-2313, T-2314) but no live agent consumes it — the doorbell/be-reachable wake path still polls on the 15s floor, so the arc's headline value (instant DM delivery) is demonstrated but not delivered. Wiring a background 'channel subscribe inbox:<self> --push' consumer that rings the existing PTY doorbell on inbox.queued (degrading to the current poll if WS drops — T-2314 auto-recovers) is bounded, additive, reversible. Sensitive wake-path area (T-1800 doorbell+mail, T-2285 ack-gap, prior rejected flag-poll design) needs a human GO before build.

**Date**: 2026-07-02T21:10:46Z

## Updates

<!-- Auto-populated by git mining at task completion.
     Manual entries optional during execution. -->

### 2026-07-02T20:52:08Z — status-update [task-update-agent]
- **Change:** status: captured → started-work

### 2026-07-02T21:10:46Z — inception-decision [inception-workflow]
- **Action:** Recorded inception decision
- **Decision:** GO
- **Rationale:** GO (advisory): the WS push mechanism is shipped + proven (S1-S4, S3b, T-2313, T-2314) but no live agent consumes it — the doorbell/be-reachable wake path still polls on the 15s floor, so the arc's headline value (instant DM delivery) is demonstrated but not delivered. Wiring a background 'channel subscribe inbox:<self> --push' consumer that rings the existing PTY doorbell on inbox.queued (degrading to the current poll if WS drops — T-2314 auto-recovers) is bounded, additive, reversible. Sensitive wake-path area (T-1800 doorbell+mail, T-2285 ack-gap, prior rejected flag-poll design) needs a human GO before build.

### 2026-07-02T21:10:46Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
- **Reason:** Inception decision: GO
