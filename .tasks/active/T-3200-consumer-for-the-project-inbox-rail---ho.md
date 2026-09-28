---
id: T-3200
name: "Consumer for the project inbox rail - how a durable mailbox reaches a session
  prompt"
description: >
  Inception: Consumer for the project inbox rail - how a durable mailbox reaches a
  session prompt

status: started-work
workflow_type: inception
owner: human
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-09-28T15:44:52Z
last_update: 2026-09-28T15:45:49Z
date_finished:
# revisit_at: YYYY-MM-DD          # T-1451: set on DEFER decisions to enable G-053 daily revisit scan
# revisit_evidence_needed:        # T-1451: one-line description of what evidence makes the revisit actionable
# ── Inception scoring exception (T-2186 Slice 2 / T-2188). See 050-Inceptions.md §Scoring Exception. ──
target_blast_radius: 3            # int 0..9. Anticipated component count of the build work this inception would authorise on GO.
                                  # Substitutes for the absent components: list in the F8 cost formula (040). Required.
                                  # Guide: 0=docs only, 1=single file, 3=small subsystem (S), 5=cross-subsystem (M), 7=multi-arc (L), 9=framework-wide (XL).
voi_score: 0.5                    # float 0..1. Value of Information — expected value of resolving this question,
                                  # independent of build cost. Higher when answer affects many tasks or unblocks a strategic decision. Required.
bvp_scores_proposed:
  - ts: '2026-09-28T15:45:49Z'
    estimator: bvp-estimator-v1-heuristic
    scores:
      D1: 2
      D2: 2
      D3: 2
      D4: 2
      F-RECALL: 2
      F-ORCH: 2
    rationale: D1=2 (no-signal); D2=2 (no-signal); D3=2 (no-signal); D4=2 
      (no-signal); F-RECALL=2 (no-signal); F-ORCH=2 (no-signal)
    rubric_sha: e4a00f38e801
---

# T-3200: Consumer for the project inbox rail - how a durable mailbox reaches a session prompt

## Problem Statement

<!-- What problem are we exploring? For whom? Why now? -->

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

- **IW-1: Should an inbound peer message interrupt a working session?**
  confidence: 0
  disposition:
  rationale:
  Option C (wake on `inbox.queued`) requires yes. That is a claim about who
  controls a session's attention, not an engineering detail, and it is the
  operator's to make. Options A and B deliberately do not interrupt — they make
  the backlog visible and leave the timing to a human.

- **IW-2: Do we ack on READ, or on ACTION?**
  confidence: 1
  disposition:
  rationale:
  AEF's T-3434 ladder stops on ack. If we ack when a message is merely
  SURFACED, their retries stop while the work is still undone — converting a
  loud, correct escalation into silence, which is the failure this whole task
  exists to fix, inverted. Acking only on action keeps their pressure honest but
  leaves the ladder climbing while we are simply busy. Confidence 1 because the
  trade is clear but the right answer depends on IW-1.

- **IW-3: Does a consumer generalise beyond AEF?**
  confidence: 1
  disposition:
  rationale:
  Measured: 22 `inbox:*` topics exist across 5 real peer roots
  (832-Workflow-designer 54, 010-termlink 49, 1409-sprind 27, AEF 6,
  framework-agent 1). A consumer built around AEF's sidecar.consult convention
  is one that breaks on the second correspondent. Unknown: whether the other
  four peers post with comparable metadata, which is cheap to check and has not
  been checked.

- **IW-4: What is the real inbound rate?**
  confidence: 0
  disposition:
  rationale:
  Interrupt cost is a function of volume and NOBODY HAS MEASURED IT. 232 records
  accumulated over an unknown window. Without a rate, option C's cost is a guess
  and IW-1 cannot be answered honestly. This is the cheapest thing exploration
  could settle and it gates the expensive decision.

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

Advisory opening position, not a decision. Measured this session: nothing consumes inbox:cacc73ea32b121dd/010-termlink, so 49 AEF consults sat unread while their T-3434 retry ladder climbed to rung 4 of 10 attempts; one was a bug report against our own agent search verb, unread 3+ weeks. Delivery is proven (AEF measured a 73h05m round trip and their own harness called FAIL at 1800s); consumption is the gap. The same blindness nearly caused data loss: inbox status labelled those records pending transfers and inbox clear resolves to channel.trim, so a routine drain would have deleted 232 live messages. This is arc-011 declared subject. GO to EXPLORE the design - cron surfacing into handover vs a canary vs waking on inbox.queued vs a skill-layer verb - because the options differ in sovereignty and cost and picking one is a human call.

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

### 2026-09-28T15:45:49Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
