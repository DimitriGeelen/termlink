---
id: T-3759
name: "Non-root design session needs dispatch: own hub, root, a root-side dispatch
  service, or delegation (external consultation)"
description: >
  Inception: Non-root design session needs dispatch: own hub, root, a root-side dispatch
  service, or delegation (external consultation)

status: started-work
workflow_type: inception
owner: human
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-11-12T23:13:17Z
last_update: 2026-11-13T00:18:45Z
date_finished:
# revisit_at: YYYY-MM-DD          # T-1451: set on DEFER decisions to enable G-053 daily revisit scan
# revisit_evidence_needed:        # T-1451: one-line description of what evidence makes the revisit actionable
# ── Inception scoring exception (T-2186 Slice 2 / T-2188). See 050-Inceptions.md §Scoring Exception. ──
target_blast_radius: 3            # int 0..9. Anticipated component count of the build work this inception would authorise on GO.
                                  # Substitutes for the absent components: list in the F8 cost formula (040). Required.
                                  # Guide: 0=docs only, 1=single file, 3=small subsystem (S), 5=cross-subsystem (M), 7=multi-arc (L), 9=framework-wide (XL).
voi_score: 0.6                    # float 0..1. Value of Information — expected value of resolving this question,
                                  # independent of build cost. Higher when answer affects many tasks or unblocks a strategic decision. Required.
                                  # T-3172: deliberately EMPTY — set a considered value. 0.5 (and 0.3/0.4) score 2, the same as "not assessed".
bvp_scores_proposed:
  - ts: '2026-11-12T23:14:15Z'
    estimator: bvp-estimator-v1-heuristic
    scores:
      D1: 2
      D2: 2
      D3: 2
      D4: 2
      F-RECALL: 2
    rationale: D1=2 (no-signal); D2=2 (no-signal); D3=2 (no-signal); D4=2 
      (no-signal); F-RECALL=2 (no-signal)
    rubric_sha: e4a00f38e801
---

# T-3759: Non-root design session needs dispatch: own hub, root, a root-side dispatch service, or delegation (external consultation)

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

- **IW-1: How should a non-root agent session (user dimitri-mint-dev on .107) dispatch worker sessions, given the hub's runtime dir is root-only (0700, holds hub.secret) and register refuses a dir the hub does not serve (T-3749)? Options A own user hub, B run the session as root, C root-side dispatch service with an allow list, D delegate to a root project.**
  confidence: 1
  disposition: deferred
  rationale: operator asked for external consultation 2026-11-13 before ruling; panel brief docs/reports/T-3759-dispatch-nonroot/brief.md

## Exploration Plan

1. External consultation (operator, 2026-11-13): the four subscription seats (claude-code, codex, opencode, antigravity) answer the same brief, stored unedited in docs/reports/T-3759-dispatch-nonroot/<seat>.md. Time-box: one round, ~1 h.
2. Synthesis: where they agree and disagree, quoted from disk; revised recommendation to the operator as one decision.

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

Panel (claude-code, codex, antigravity; opencode unavailable) agrees a worker must never run as root for a non-root
requester (repo hooks, build scripts and settings in a user-writable checkout give root code execution), and that
execution and hub registration are separate problems. Build: the worker runs as the requesting user and registers on
the main hub over TCP (hub side exists: session.register_remote, router.rs:774, scope interact server.rs:796-798) with a
dedicated, user-bound registration capability so no one can impersonate another session. Interim: delegate to a root
session with a fixed request form only.

**Evidence:**

- docs/reports/T-3759-dispatch-nonroot/claude-code.md (E > D > A > C > B), codex.md (A > D > C > B, E = separate
  execution from registration), antigravity.md (C > D > A > B with E1 TCP register_remote, E2 privilege drop)
- Verified: hub implements session.register_remote/heartbeat/deregister_remote; no CLI client uses them yet

## Decisions

### 2026-11-13 — dispatch for the non-root design session (operator ruling)
- **Chose:** A + D. A: worker runs as the requesting user and signs in at the main hub over the network with its own
  user-bound registration permission (no root execution, workers visible). D (interim until A): the design session asks a
  root session using a fixed request form only (task id + reviewer kind), never commands taken from message text.
- **Why:** all three panel answers converge on separating execution (unprivileged) from registration; A is the only
  option with no root execution and visible workers; D works today.
- **Rejected:** B root doorman that drops privileges (adds a root service; more to get right than A); C the user's own
  hub (invisible workers, ambiguous hub detection); original C (root starts the worker: privilege escalation).
- Operator words in chat: "A + D". Assumption recorded (overturnable): the interim neighbour is 1409's own root main
  session (the design session belongs to project 1409). Left open: expiry of the registration permission (comes back
  with the build task).

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

### 2026-11-12T23:14:14Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
