---
id: T-2380
name: "cross-agent comms confirm-ack field gap (hub-split + degraded-read)"
description: >
  Inception: cross-agent comms confirm-ack field gap (hub-split + degraded-read)

status: work-completed
workflow_type: inception
owner: human
horizon: null
tags: []
components: []
related_tasks: []
created: 2026-07-07T17:16:20Z
last_update: 2026-07-09T09:35:29Z
date_finished: 2026-07-09T09:35:29Z
# revisit_at: YYYY-MM-DD          # T-1451: set on DEFER decisions to enable G-053 daily revisit scan
# revisit_evidence_needed:        # T-1451: one-line description of what evidence makes the revisit actionable
# ── Inception scoring exception (T-2186 Slice 2 / T-2188). See 050-Inceptions.md §Scoring Exception. ──
target_blast_radius: 3            # int 0..9. Anticipated component count of the build work this inception would authorise on GO.
                                  # Substitutes for the absent components: list in the F8 cost formula (040). Required.
                                  # Guide: 0=docs only, 1=single file, 3=small subsystem (S), 5=cross-subsystem (M), 7=multi-arc (L), 9=framework-wide (XL).
voi_score: 0.5                    # float 0..1. Value of Information — expected value of resolving this question,
                                  # independent of build cost. Higher when answer affects many tasks or unblocks a strategic decision. Required.
---

# T-2380: cross-agent comms confirm-ack field gap (hub-split + degraded-read)

## Problem Statement

The reliable-comms arcs (arc-003 `reliable-comms`, arc-004 `push-transport`)
shipped a durable send + sub-second push-wake, proven in isolated single-hub E2E
(T-2318/T-2325/T-2320, 85–111 ms). But in the live fleet the **confirm/ack half
fails silently** — a sender gets a write success (`offset N`) then silent
uncertainty about whether the peer received or replied. Observed 3× in one
session (2026-07-07). Two proven root causes: **hub-split/no-federation (G-060)**
— reader and writer on different hubs see different histories of the same-named
topic (E1); and **degraded-read hubs** — .122 accepts writes but times out
message reads, so `--ack-required` false-timeouts forever (E2, the phantom
"2-hour wait"). Full evidence:
`docs/reports/T-2380-comms-confirm-ack-field-gap-inception.md`. For: any operator
or agent coordinating cross-host work (deploys, handoffs). Why now: it just cost
multiple sessions of phantom waiting and a wrong "message lost" conclusion.

## Assumptions

<!-- Key assumptions to test. Register with: fw assumption add "Statement" --task T-XXX -->
- A1: the .122 read-wedge is agent-presence bloat (PL-200), not a binary/version regression.
- A2: reader/writer hub disagreement (E1) is a common field pattern, not a one-off of the `--hub .122` choice.
- A3: no existing convention already says "reply on the sender's hub" that we're merely not following.

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

- **IW-1: Should the tooling enforce a "reply on the sender's hub" convention so reader and writer can't silently target different hubs (attacks E1)?**
  confidence: 1
  disposition: answered
  rationale: GO (operator "yes" 2026-07-09) — build T-2386 (reply-on-sender-hub routing).

- **IW-2: Should `--ack-required` (and the ack-poll generally) gain a hub-read-health precondition that fails fast instead of burning the full timeout against a degraded-read hub (attacks E2/E3)?**
  confidence: 2
  disposition: answered
  rationale: GO — hub-read-health fail-fast folded into the T-2385 preflight (the loud-contract centerpiece); strongest evidence (E2 reproduced 3×).

- **IW-3: Is actual cross-hub federation (or a relay) for dm: topics warranted, or is it out of scope vs the convention+fail-fast pair (C1+C2)?**
  confidence: 1
  disposition: dissolved
  rationale: OUT of scope — full cross-hub federation is high-cost; the convention+fail-fast pair (T-2386 reply-on-sender-hub + T-2385 preflight) removes the SILENT failure without it. Revisit only if hub-agreement routing proves insufficient.

- **IW-4: Are the adjacent guard-rails part of THIS fix or separate follow-on tasks — specifically: agent-vs-shell signal in `remote list` (F1), inject off-rail warning (F2), and the arc-004 finding that the push-waker is SHIPPED BUT NOT RUNNING on any host (E4/F3) — needing both auto-arm AND a "waker-liveness" canary ("host claims reachable but no waker process")?**
  confidence: 2
  disposition: answered
  rationale: PARTIALLY IN — agent-vs-shell signal (F1) + waker-liveness both fold into THIS arc: F1/waker-checks into the T-2385 preflight, plus a standalone T-2387 waker-liveness canary (E4/F3, G-069 shipped≠live guard). The send-side fp-mismatch found post-filing becomes T-2384. Raw-inject off-rail warning (F2) deferred as a separate follow-up (cosmetic vs the silent-delivery core).

## Exploration Plan

<!-- How will we validate assumptions? Spikes, prototypes, research? Time-box each. -->
1. **Validate A1 (30 min):** have ring20-manager run `channel sweep` / retention-reset on .122 locally; re-time `channel info <dm>` — does read speed restore? Confirms bloat vs regression.
2. **Validate A2/A3 (read-only):** audit how `agent contact` / `/reply` / push-waker pick a hub for a reply; grep for any existing sender-hub convention. Time-box 45 min.
3. **Prototype C1+C2 shape (paper only, no build pre-GO):** sketch where a reply-hub stamp + a hub-read-health probe would slot into `agent contact --ack-required`. Estimate blast radius.
4. **Decide:** present C1/C2/C3/C4-5 with cost + evidence for human go/no-go via `fw task review T-2380`.

## Technical Constraints

<!-- What platform, browser, network, or hardware constraints apply?
     For web apps: HTTPS requirements, browser API restrictions, CORS, device support.
     For hardware APIs (mic, camera, GPS, Bluetooth): access requirements, permissions model.
     For infrastructure: network topology, firewall rules, latency bounds.
     Fill this BEFORE building. Discovering constraints after implementation wastes sessions. -->

## Scope Fence

**IN:** decide the permanent shape for making the cross-agent comms *confirm/ack*
half non-silent under hub-split (E1) + degraded-read (E2) — candidates C1
(reply-on-sender-hub), C2 (hub-read-health fail-fast), C3 (federation, evaluate
in/out).

**OUT (this inception):** the operational instance-fix of .122's bloat (C6 —
ring20-manager's host, folded into the live coordination message, not our code);
building any of the candidates (post-GO build tasks); the adjacent guard-rails
C4/C5 unless IW-4 pulls them in.

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
- Root cause identified with bounded fix path
- Fix is scoped, testable, and reversible

**NO-GO if:**
- Problem requires fundamental redesign or unbounded scope
- Fix cost exceeds benefit given current evidence

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

Field evidence this session: arc-003/004 shipped durable send + push-wake (proven in isolated single-hub E2E), but the confirm/ack half fails silently in the real fleet. Two proven causes: (1) hub-split/no-federation (G-060) — a handoff written to .122 (offset 52) is invisible to a co-resident reader on .107 reading the same-named topic (113 msgs, different history), so recent_dm showed 0 and looked lost; (2) a degraded-read hub — .122 does metadata reads fine (channel list = 105 topics instant) but per-topic message reads time out, so --ack-required polling it false-timeouts forever (the phantom 2-hour wait). Net: arc-003 headline "confirmed delivery, no silent loss" has a field hole it assumed away (you must be able to read the hub you wrote to, and reader+writer must agree on hub). Recurred across 3 sessions this session alone. Worth an inception to decide the permanent shape (reply-on-sender-hub convention vs hub-read-health fail-fast vs actual federation) before building — I have changed the diagnosis twice, so scope needs validation not a jump-to-fix.

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

**Decision**: GO

**Rationale**: Field evidence this session: arc-003/004 shipped durable send + push-wake (proven in isolated single-hub E2E), but the confirm/ack half fails silently in the real fleet. Two proven causes: (1) hub-split/no-federation (G-060) — a handoff written to .122 (offset 52) is invisible to a co-resident reader on .107 reading the same-named topic (113 msgs, different history), so recent_dm showed 0 and looked lost; (2) a degraded-read hub — .122 does metadata reads fine (channel list = 105 topics instant) but per-topic message reads time out, so --ack-required polling it false-timeouts forever (the phantom 2-hour wait). Net: arc-003 headline "confirmed delivery, no silent loss" has a field hole it assumed away (you must be able to read the hub you wrote to, and reader+writer must agree on hub). Recurred across 3 sessions this session alone. Worth an inception to decide the permanent shape (reply-on-sender-hub convention vs hub-read-health fail-fast vs actual federation) before building — I have changed the diagnosis twice, so scope needs validation not a jump-to-fix.

**Date**: 2026-07-09T09:35:28Z

## Updates

<!-- Auto-populated by git mining at task completion.
     Manual entries optional during execution. -->

### 2026-07-07T17:17:32Z — status-update [task-update-agent]
- **Change:** status: captured → started-work

### 2026-07-09T09:35:28Z — inception-decision [inception-workflow]
- **Action:** Recorded inception decision
- **Decision:** GO
- **Rationale:** Field evidence this session: arc-003/004 shipped durable send + push-wake (proven in isolated single-hub E2E), but the confirm/ack half fails silently in the real fleet. Two proven causes: (1) hub-split/no-federation (G-060) — a handoff written to .122 (offset 52) is invisible to a co-resident reader on .107 reading the same-named topic (113 msgs, different history), so recent_dm showed 0 and looked lost; (2) a degraded-read hub — .122 does metadata reads fine (channel list = 105 topics instant) but per-topic message reads time out, so --ack-required polling it false-timeouts forever (the phantom 2-hour wait). Net: arc-003 headline "confirmed delivery, no silent loss" has a field hole it assumed away (you must be able to read the hub you wrote to, and reader+writer must agree on hub). Recurred across 3 sessions this session alone. Worth an inception to decide the permanent shape (reply-on-sender-hub convention vs hub-read-health fail-fast vs actual federation) before building — I have changed the diagnosis twice, so scope needs validation not a jump-to-fix.

## Reviewer Verdict (v1.5)

- **Scan ID:** R-66ec3618
- **Timestamp:** 2026-07-09T09:35:29Z
- **Catalogue:** v1.3-seed
- **Overall:** CONCERN
- **Needs Human:** no
- **Findings:** 1

**Verification-level findings:**

  1. **disposition-incomplete** (partial, heuristic) @ ## Open Questions: IW-1
     - evidence: `IW-1 disposition='answered' but rationale has no evidence citation (T-NNNN, file:line, docs/reports/, G-/L-/D-id, dialogue-log, or commit hash)`

### 2026-07-09T09:35:29Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
- **Reason:** Inception decision: GO
