---
id: T-2476
name: "Consumption-confirmation for the WAKE layer (T-2468 P2, G-083)"
description: >
  Inception: Consumption-confirmation for the WAKE layer (T-2468 P2, G-083)

status: work-completed
workflow_type: inception
owner: human
horizon: null
tags: []
components: []
related_tasks: []
created: 2026-08-01T19:49:44Z
last_update: 2026-08-01T21:08:26Z
date_finished: 2026-08-01T21:08:26Z
# revisit_at: YYYY-MM-DD          # T-1451: set on DEFER decisions to enable G-053 daily revisit scan
# revisit_evidence_needed:        # T-1451: one-line description of what evidence makes the revisit actionable
# ── Inception scoring exception (T-2186 Slice 2 / T-2188). See 050-Inceptions.md §Scoring Exception. ──
target_blast_radius: 3            # int 0..9. Anticipated component count of the build work this inception would authorise on GO.
                                  # Substitutes for the absent components: list in the F8 cost formula (040). Required.
                                  # Guide: 0=docs only, 1=single file, 3=small subsystem (S), 5=cross-subsystem (M), 7=multi-arc (L), 9=framework-wide (XL).
voi_score: 0.5                    # float 0..1. Value of Information — expected value of resolving this question,
                                  # independent of build cost. Higher when answer affects many tasks or unblocks a strategic decision. Required.
---

# T-2476: Consumption-confirmation for the WAKE layer (T-2468 P2, G-083)

## Problem Statement

The comms round-trip's weakest link is CONSUME, and it fails SILENTLY (G-083, high).
TermLink guarantees the WRITE (durable append), and WAKE fires (pushwaker rings the
PTY), but nothing confirms the recipient session actually **consumed** the wake. Field
evidence (2026-07-10, aef↔workflow-designer T-175 thread): when the recipient session
is busy on its own task or in manual-accept mode, the injected wake text lands in the
input box **unsubmitted** and is discarded on the next `claude --continue`. The message
sits durably written (offset 21) but never read — which the operator experiences as
"comms does not flow without manual nudging".

The root is that `agent-presence` heartbeat comes from a background script
(`listener-heartbeat.sh`) **fully decoupled** from the claude session's actual
availability: `LIVE + armed (pty set)` is true while the session consumes nothing.
The framework checks reachable-**BEFORE** (T-2385 `classify_reachability` /
`--require-reachable`) but has **no** check that the recipient consumed-**AFTER** (that
its read-cursor / receipt advanced within a bounded window post-ring). `agent-send.sh`
re-rings and waits for a receipt up to `max_rings` — so a wake that IS consumed gets
confirmed — but a raw `termlink inject` bypasses that wait, and even agent-send cannot
*make* a busy/manual session consume. This is the sibling of G-069 (shipped≠live):
here **LIVE≠listening**. Blind since the doorbell shipped (T-1800, months ago),
surfaced only by operator frustration — never by any framework surface. Why now:
T-2468 IW-2 ranked this the highest Reliability gap in the register.

## Assumptions

<!-- Key assumptions to test. Register with: fw assumption add "Statement" --task T-XXX -->

## Open Questions

- **IW-1: What is the load-bearing "consumed" signal — read-cursor advance, receipt
  post, or a session-level turn-consumed ack?**
  confidence: 1
  disposition: deferred
  rationale: The receipt/read-cursor advance is the observable candidate (agent-send
  already waits on it), but the failure case is exactly where it does NOT advance
  (busy/manual). Need to confirm which cursor moves when a wake IS consumed vs not,
  against the live thread — G-083 says "prove against the live thread first".

- **IW-2: Where does the confirmation loop live — in the sender path (poll after ring),
  a canary, or a new primitive — and what is the bounded window N?**
  confidence: 1
  disposition: deferred
  rationale: Cheapest is extending the sender-side ring loop to LOUD-fail with the
  "busy/manual-mode, unread at offset X" diagnosis if no cursor advance within N secs.
  A canary catches the raw-inject bypass case. Design tradeoff for the human.

- **IW-3: Can the framework distinguish "busy" from "manual-accept" from "dead" so the
  LOUD failure carries the right remediation?**
  confidence: 1
  disposition: deferred
  rationale: The remediation differs (busy → wait/re-ring; manual-accept → the
  IS_SANDBOX auto-accept fix per reference memory; dead → relaunch through T-2388
  launcher). The diagnosis must classify, not just say "unread".

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

IN: designing the consumed-AFTER confirmation (which signal, where the loop lives, the
bounded window N, the LOUD-failure diagnosis + per-class remediation); validating the
"consumed" signal against the live aef↔designer thread before building; GO/NO-GO
recommendation.
OUT: building blindly without live-thread validation (G-083 explicit warning);
fixing manual-accept auto-submit inside claude itself (harness territory — the
framework can only detect + remediate, not force a manual session to submit);
the WAKE transport itself (already shipped, T-1800/arc-004).

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

T-2468 IW-2 identified the comms round-trip's weakest link: WAKE fires and the message is durably written, but a busy/manual-accept recipient session consumes NOTHING and the framework has no consumed-AFTER check (only reachable-BEFORE via T-2385). This is the operator's own 'comms does not flow without manual nudging' complaint (G-083, high severity, blind since T-1800). Highest Reliability value in the register. GO to inception because the fix (poll recipient read-cursor/receipt post-ring; fail LOUD with busy/manual-mode + unread-offset diagnosis if no advance within N secs) is bounded and testable but MUST be designed + proven against the live aef<->designer thread before building — G-083 explicitly warns 'do not build blindly'.

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

**Rationale**: Recommendation: GO

Rationale:

T-2468 IW-2 identified the comms round-trip's weakest link: WAKE fires and the message is durably written, but a busy/manual-accept recipient session consumes NOTHING and the framework has no consumed-AFTER check (only reachable-BEFORE via T-2385). This is the operator's own 'comms does not flow without manual nudging' complaint (G-083, high severity, blind since T-1800). Highest Reliability value in the register. GO to inception because the fix (poll recipient read-cursor/receipt post-ring; fail LOUD with busy/manual-mode + unread-offset diagnosis if no advance within N secs) is bounded and testable but MUST be designed + proven against the live aef<->designer thread before building — G-083 explicitly warns 'do not build blindly'.

Evidence:

**Date**: 2026-08-01T21:08:22Z

## Updates

<!-- Auto-populated by git mining at task completion.
     Manual entries optional during execution. -->

### 2026-08-01T19:50:24Z — status-update [task-update-agent]
- **Change:** status: captured → started-work

### 2026-08-01T21:08:22Z — inception-decision [inception-workflow]
- **Action:** Recorded inception decision
- **Decision:** GO
- **Rationale:** Recommendation: GO

Rationale:

T-2468 IW-2 identified the comms round-trip's weakest link: WAKE fires and the message is durably written, but a busy/manual-accept recipient session consumes NOTHING and the framework has no consumed-AFTER check (only reachable-BEFORE via T-2385). This is the operator's own 'comms does not flow without manual nudging' complaint (G-083, high severity, blind since T-1800). Highest Reliability value in the register. GO to inception because the fix (poll recipient read-cursor/receipt post-ring; fail LOUD with busy/manual-mode + unread-offset diagnosis if no advance within N secs) is bounded and testable but MUST be designed + proven against the live aef<->designer thread before building — G-083 explicitly warns 'do not build blindly'.

Evidence:

## Reviewer Verdict (v1.5)

- **Scan ID:** R-1967da3d
- **Timestamp:** 2026-08-01T21:08:27Z
- **Catalogue:** v1.3-seed
- **Overall:** CONCERN
- **Needs Human:** no
- **Findings:** 1

**Verification-level findings:**

  1. **disposition-incomplete** (partial, heuristic) @ ## Open Questions: IW-1
     - evidence: `IW-1 disposition='answered' but rationale has no evidence citation (T-NNNN, file:line, docs/reports/, G-/L-/D-id, dialogue-log, or commit hash)`

### 2026-08-01T21:08:26Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
- **Reason:** Inception decision: GO
