---
id: T-2698
name: "TermLink purpose review #6 — is the refusal taxonomy real, or partly fiction?"
description: >
  Inception: TermLink purpose review #6 — is the refusal taxonomy real, or partly fiction?

status: started-work
workflow_type: inception
owner: human
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-08-14T08:23:32Z
last_update: 2026-08-14T08:25:02Z
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

# T-2698: TermLink purpose review #6 — is the refusal taxonomy real, or partly fiction?

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

- **IW-1: Is Directive #1 (Antifragility) unproven — are the backpressure paths untested?**
  confidence: 3
  disposition: dissolved
  rationale: Not supported. `governor.rs` carries 20 unit tests; `offline_queue.rs`
  asserts `QueueError::QueueFull { cap: 3 }`. The thesis was investigated first, found
  unevidenced, and **dropped rather than forced** — recorded for calibration.

- **IW-2: Can every documented error code actually be emitted?**
  confidence: 3
  disposition: answered
  rationale: No. 23 codes defined in `control.rs::error_code`; three have zero emission
  sites across all product crates — `SESSION_BUSY` (-32002), `MESSAGE_EXPIRED`
  (-32004), `PROTOCOL_VERSION_TOO_OLD` (-32011).

- **IW-3: Is PROTOCOL_VERSION_TOO_OLD merely reserved, or is it unwired machinery?**
  confidence: 3
  disposition: answered
  rationale: Unwired machinery. `control.rs` ships `check_protocol_version()` which
  builds the structured error with `{declared, required, method}`, plus a passing unit
  test `check_protocol_version_rejects_when_declared_is_older`. Grep finds **zero
  callers** outside `control.rs`. Defined, tested, documented, invoked by nothing.

- **IW-4: Does the documentation present these as reserved?**
  confidence: 3
  disposition: answered
  rationale: No. `docs/reports/T-005-message-protocol-design.md` lists -32002 and
  -32004 in the protocol's error table as ordinary refusals ("Target cannot accept
  commands (already executing)", "TTL exceeded before delivery"), with no indication
  they are unimplemented. A client written from the docs would handle errors it can
  never receive and assume protections that are not applied.

- **IW-5: Should version negotiation be wired into the request path in this pass?**
  confidence: 3
  disposition: deferred
  rationale: No. Calling `check_protocol_version` on a live method begins REJECTING
  peers whose declared version is below the requirement — a wire-behaviour change that
  could cut off running fleet hosts. Which methods require which version is a product
  decision with fleet consequences, so it is filed for the human rather than enforced
  unilaterally (same reasoning as T-2692's non-blocking macOS gate).

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

Sixth critical pass. Prior axes: breadth (T-2468), non-goal guards (T-2678), guard execution (T-2683), Usability+Portability (T-2690), positive claims vs provers (T-2694). This pass audits the ERROR TAXONOMY — the set of refusals TermLink documents itself as able to issue — against what the code can actually emit. Method: enumerate every constant in control.rs error_code, then grep every product crate for an emission site. Result: 23 codes defined, THREE never emitted anywhere. (1) SESSION_BUSY -32002, documented in the T-005 protocol design as 'Target cannot accept commands (already executing)' — never emitted, so whatever concurrency condition it names is either impossible or silently permitted. (2) MESSAGE_EXPIRED -32004, documented as 'TTL exceeded before delivery' — never emitted, so message TTL is not enforced or is enforced silently. (3) PROTOCOL_VERSION_TOO_OLD -32011 is the sharpest: control.rs ships a builder check_protocol_version() that constructs the structured error with {declared, required, method}, and a unit test check_protocol_version_rejects_when_declared_is_older asserting it — but grep finds ZERO callers outside control.rs. Version negotiation is defined, tested, documented, and invoked by nothing. That is the T-2683 pattern (a guard nothing executes) reproduced at the code level rather than the script level, and it means a client implementing from the docs would assume a protection the hub never applies. Also checked and NOT a finding, recorded for calibration: the backpressure mechanisms are genuinely tested (governor.rs carries 20 unit tests; offline_queue asserts QueueFull{cap:3}), so an 'antifragility is unproven' thesis is not supported by evidence and was dropped rather than forced. GO on the in-authority subset: make the dead-refusal class structurally visible and stop the documented contract overstating what is enforced. Wiring version negotiation into the request path is explicitly NOT in scope — it changes wire behaviour and could begin rejecting live fleet peers, which is a human decision.

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

### 2026-08-14T08:23:46Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
