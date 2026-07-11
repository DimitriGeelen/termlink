---
id: T-2402
name: "Woken-but-silent: make a rung-yet-unanswered agent loud + self-healing"
description: >
  Push-waker rings PTY once on dm.queued; a rung agent that does not reply is invisible (no re-ring, no operator signal). Demo T-2400: rang wfd offset=7, silent 15min. Explore re-ring-on-no-receipt / awaiting-ack registration / woken-but-silent surface.

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-07-11T07:37:06Z
last_update: 2026-07-11T09:31:38Z
date_finished: null
# revisit_at: YYYY-MM-DD          # T-1451: set on DEFER decisions to enable G-053 daily revisit scan
# revisit_evidence_needed:        # T-1451: one-line description of what evidence makes the revisit actionable
# ── Inception scoring exception (T-2186 Slice 2 / T-2188). See 050-Inceptions.md §Scoring Exception. ──
target_blast_radius: 3            # int 0..9. Anticipated component count of the build work this inception would authorise on GO.
                                  # Substitutes for the absent components: list in the F8 cost formula (040). Required.
                                  # Guide: 0=docs only, 1=single file, 3=small subsystem (S), 5=cross-subsystem (M), 7=multi-arc (L), 9=framework-wide (XL).
voi_score: 0.5                    # float 0..1. Value of Information — expected value of resolving this question,
                                  # independent of build cost. Higher when answer affects many tasks or unblocks a strategic decision. Required.
---

# T-2402: Woken-but-silent: make a rung-yet-unanswered agent loud + self-healing

## Context

**Deterministic-attention design (promoted from inception 2026-07-11).** The comms
mechanism is proven end-to-end (durable mail T-1800/1804, WS push-wake arc-004
`channel subscribe --push`, correct identity T-2399, hands-free auto-accept
T-2400/T-2401 — the last VERIFIED live: wfd relaunched with NO
`--dangerously-skip-permissions` auto-posted off=9 "wfd ack no-flag 6a646ce8").
The ONLY non-deterministic node left is the agent's *cognition* (choosing to act).
You cannot make LLM cognition deterministic — so this task makes the ENVELOPE
around it deterministic: delivery-into-the-REPL, and loud+self-healing failure
detection, so "a message went unattended" is a guaranteed-detected, retried, and
surfaced event, never a silent stop.

Live evidence this is real (not theoretical): T-2400 demo — push-waker `rang
workflow-designer offset=7`, wfd never replied, silent 15 min; the ring fired once,
nothing re-rang, nothing got loud. Root of the silence = two gaps (stages 3 + 5
below) plus a soft wake obligation (stage 6). See
[[project_comms_loud_contract]] and the control-loop table in the T-2401 session.

The six-stage control loop (1,2 already deterministic; this task builds 3,5,6):
1 durable obligation ✅ · 2 push wake ✅ · **3 idle-gated injection ❌** ·
4 receipt-or-re-ring ⚠️bounded · **5 escalate-if-stuck ❌** ·
**6 wake-protocol obligation ⚠️soft**.

## Acceptance Criteria

### Agent
- [x] **Stage 3 — idle-gated injection.** `scripts/be-reachable-pushwaker.sh` (and/or the inject step in `agent-send.sh`) rings the PTY only when it is at a READY prompt; if the REPL is busy (mid-turn, resume-picker, tool-call) it defers and re-injects (bounded retry + backoff) until the input is accepted — instead of injecting blind. Verify: an injection issued while the REPL is busy is NOT swallowed (integration test or documented manual test showing the doorbell lands after the REPL returns to idle). Detection mechanism (PTY-state probe) documented. **DONE 2026-07-11:** blind inject at rail-loop replaced by `pushwaker_ring_when_ready` (probe→defer→inject-at-idle, rc=3 loud give-up, never blind). Pure `pushwaker_pty_state` (7 unit fixtures) + hermetic BUSY→READY integration test (`test-pushwaker-ready-loop.sh`: inject fires only after READY, 0 blind injects) + live-probe validated vs wfd/aef/claude-master. Mechanism doc: `docs/operations/pushwaker-idle-gating.md`. Commits 340a03df + integration test.
- [ ] **Stage 5 — escalating re-ring, no silent stop.** The send/receipt loop re-rings on a schedule until a receipt is observed; after N attempts it ESCALATES to a loud, operator-visible signal (a `*-canary.log` entry AND/OR a registered `awaiting-ack` obligation) instead of exiting silently at `--max-rings`. Verify: against a deliberately non-responding recipient, the loop produces a surfaced artifact (canary log line or awaiting-ack row visible to `/canaries` or `channel awaiting-ack`) rather than a silent non-zero exit.
- [ ] **Stage 6 — wake-protocol obligation.** The `/check-arc respond` skill mandates that a woken agent drains ALL unread topics, posts a receipt per topic, and for each either replies OR posts an explicit "acknowledged, no action needed" — so silence always means a bug (caught by stage 5), never a valid unlogged choice. Verify: the skill text encodes the always-ack + reply-or-explicit-defer obligation (grep-able), AND the T-2295 unconfirmed-delivery canary is confirmed to fire on a woken-but-silent thread (i.e. `/check-arc respond` registers or leaves an awaiting-ack obligation the canary reads).

## Assumptions

<!-- Key assumptions to test. Register with: fw assumption add "Statement" --task T-XXX -->

## Open Questions

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

<!-- How will we validate assumptions? Spikes, prototypes, research? Time-box each. -->

## Technical Constraints

<!-- What platform, browser, network, or hardware constraints apply?
     For web apps: HTTPS requirements, browser API restrictions, CORS, device support.
     For hardware APIs (mic, camera, GPS, Bluetooth): access requirements, permissions model.
     For infrastructure: network topology, firewall rules, latency bounds.
     Fill this BEFORE building. Discovering constraints after implementation wastes sessions. -->

## Scope Fence

<!-- What's IN scope for this exploration? What's explicitly OUT? -->

## Acceptance Criteria (inception template — SUPERSEDED by the three stage ACs above)

<!-- Vestigial inception ACs removed on promotion to build (T-2402, 2026-07-11)
     so they do not trip the P-010 unchecked-AC gate. The authoritative ACs are
     the three deterministic-attention stages under the FIRST ### Agent above. -->

### Agent (superseded)
<!-- intentionally empty — see stage ACs above -->

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
- Root cause identified with bounded fix path
- Fix is scoped, testable, and reversible

**NO-GO if:**
- Problem requires fundamental redesign or unbounded scope
- Fix cost exceeds benefit given current evidence

## Verification

# Shell commands that MUST pass before work-completed. One per line.
# Stage 3 (idle-gated injection): syntax + unit + hermetic integration tests.
bash -n scripts/be-reachable-pushwaker.sh
bash scripts/test-pushwaker-filter.sh
bash scripts/test-pushwaker-ready-loop.sh

## Recommendation

<!-- REQUIRED before fw inception decide. Write your recommendation here (T-974).
     Watchtower reads this section — if it's empty, the human sees nothing.
     Format:
     **Recommendation:** GO / NO-GO / DEFER
     **Rationale:** Why (cite evidence from exploration)
     **Evidence:**
     - Finding 1
     - Finding 2
-->

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
