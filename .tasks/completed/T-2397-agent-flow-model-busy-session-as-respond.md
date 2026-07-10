---
id: T-2397
name: "Agent flow model: busy-session-as-responder vs dedicated responder"
description: >
  Inception: Agent flow model: busy-session-as-responder vs dedicated responder

status: work-completed
workflow_type: inception
owner: human
horizon: null
tags: []
components: []
related_tasks: []
created: 2026-07-10T18:33:23Z
last_update: 2026-07-10T19:20:48Z
date_finished: 2026-07-10T19:20:48Z
# revisit_at: YYYY-MM-DD          # T-1451: set on DEFER decisions to enable G-053 daily revisit scan
# revisit_evidence_needed:        # T-1451: one-line description of what evidence makes the revisit actionable
# ── Inception scoring exception (T-2186 Slice 2 / T-2188). See 050-Inceptions.md §Scoring Exception. ──
target_blast_radius: 3            # int 0..9. Anticipated component count of the build work this inception would authorise on GO.
                                  # Substitutes for the absent components: list in the F8 cost formula (040). Required.
                                  # Guide: 0=docs only, 1=single file, 3=small subsystem (S), 5=cross-subsystem (M), 7=multi-arc (L), 9=framework-wide (XL).
voi_score: 0.5                    # float 0..1. Value of Information — expected value of resolving this question,
                                  # independent of build cost. Higher when answer affects many tasks or unblocks a strategic decision. Required.
---

# T-2397: Agent flow model: busy-session-as-responder vs dedicated responder

## Problem Statement

An interactive claude session is turn-based: input → output → await next input,
with no idle loop of its own. So a session cannot be both busy-on-its-own-work
AND an instant peer responder — the two properties are in tension in one session.
This is the wall under every prior comms fix (WAKE/DISCOVERY/SEND-rail/
continuation/consumption-confirmation each fixed a link; flow still stalled).
Decide the flow MODEL before adding more plumbing. Five models (A dedicated
responder, B single armed auto-accept interrupt-and-resume, C daemon+claude -p,
D poll-within-turn, E human-in-the-loop). Full analysis:
`docs/reports/T-2397-agent-flow-model-inception.md`.

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

- **IW-1: Interruption coherence — can a session mid-task on job X, interrupted by a peer wake about job Y, respond and then RESUME job X without derailing?**
  confidence: 1
  disposition: deferred
  rationale: The load-bearing unknown for Model B; answerable only by a live run of two armed agents (prove-first step 2), not by code.
- **IW-2: Who relaunches the live agents armed — aef + designer run on /opt/999 under another operator; validate on scratch .107 agents first, or coordinate a relaunch of theirs?**
  confidence: 1
  disposition: deferred
  rationale: Cross-host/operator coordination; scratch-agent validation on .107 avoids disrupting live /opt/999 work.
- **IW-3: One session or two — if interruption derails work (IW-1 = no), is Model A's 2-session cost acceptable given the responder can only ack/route, not advance the peer's task?**
  confidence: 1
  disposition: deferred
  rationale: Contingent on IW-1; only relevant if interrupt-and-resume proves incoherent.
- **IW-4: Is flow even the goal for design dialogue — some exchanges (GO gates) SHOULD stop for a human; is "flow through mechanical hops, stop at decisions" the target, or is per-turn human review actually wanted for design work?**
  confidence: 2
  disposition: deferred
  rationale: Relay-loop premise assumes the former; operator sovereignty over design decisions (T-175 GO gates) is explicit and correct — needs confirmation this is the intended scope.

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

Provisional GO on Model B (single armed auto-accept session, interrupt-and-resume) — already built via T-2388, just not deployed; validate live before building dedicated responders (Model A). The comms saga bottoms out here: an interactive claude session is turn-based and cannot be both busy-on-own-work AND an instant peer responder.

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

**Rationale**: Provisional GO on Model B (single armed auto-accept session, interrupt-and-resume) — already built via T-2388, just not deployed; validate live before building dedicated responders (Model A). The comms saga bottoms out here: an interactive claude session is turn-based and cannot be both busy-on-own-work AND an instant peer responder.

**Date**: 2026-07-10T19:20:47Z

## Updates

<!-- Auto-populated by git mining at task completion.
     Manual entries optional during execution. -->

### 2026-07-10T18:33:46Z — status-update [task-update-agent]
- **Change:** status: captured → started-work

### 2026-07-10T19:20:47Z — inception-decision [inception-workflow]
- **Action:** Recorded inception decision
- **Decision:** GO
- **Rationale:** Provisional GO on Model B (single armed auto-accept session, interrupt-and-resume) — already built via T-2388, just not deployed; validate live before building dedicated responders (Model A). The comms saga bottoms out here: an interactive claude session is turn-based and cannot be both busy-on-own-work AND an instant peer responder.

## Reviewer Verdict (v1.5)

- **Scan ID:** R-067cbdc6
- **Timestamp:** 2026-07-10T19:20:48Z
- **Catalogue:** v1.3-seed
- **Overall:** CONCERN
- **Needs Human:** no
- **Findings:** 1

**Verification-level findings:**

  1. **disposition-incomplete** (partial, heuristic) @ ## Open Questions: IW-1
     - evidence: `IW-1 disposition='answered' but rationale has no evidence citation (T-NNNN, file:line, docs/reports/, G-/L-/D-id, dialogue-log, or commit hash)`

### 2026-07-10T19:20:48Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
- **Reason:** Inception decision: GO
