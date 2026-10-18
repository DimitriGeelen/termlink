---
id: T-3455
name: "arc-011 PH-3 input: cross-hub sidecar delivery gaps measured by dimitri-mint-dev"
description: >
  Inception: arc-011 PH-3 input: cross-hub sidecar delivery gaps measured by dimitri-mint-dev

status: captured
workflow_type: inception
owner: human
horizon: later
tags: [arc:arc-011, ph-3, cross-hub]
components: []
related_tasks: [T-3454, T-3426]
created: 2026-10-17T21:55:43Z
last_update: '2026-10-18T22:32:58Z'
date_finished:
# revisit_at: YYYY-MM-DD          # T-1451: set on DEFER decisions to enable G-053 daily revisit scan
# revisit_evidence_needed:        # T-1451: one-line description of what evidence makes the revisit actionable
# ── Inception scoring exception (T-2186 Slice 2 / T-2188). See 050-Inceptions.md §Scoring Exception. ──
target_blast_radius: 3            # int 0..9. Anticipated component count of the build work this inception would authorise on GO.
                                  # Substitutes for the absent components: list in the F8 cost formula (040). Required.
                                  # Guide: 0=docs only, 1=single file, 3=small subsystem (S), 5=cross-subsystem (M), 7=multi-arc (L), 9=framework-wide (XL).
voi_score: 0.6                    # agent-set under operator delegation 2026-10-18 ("same for the rest"): decides whether agents on other hubs (.122) can be reached at all; below T-3423 (0.7) because gap 1 is already fixed upstream and the work is sequenced after PH-2. — float 0..1. Value of Information — expected value of resolving this question,
                                  # independent of build cost. Higher when answer affects many tasks or unblocks a strategic decision. Required.
bvp_scores_proposed:
  - ts: '2026-10-18T22:32:58Z'
    estimator: bvp-estimator-v1-heuristic
    scores:
      D1: 3
      D2: 3
      D3: 3
      D4: 3
      F-RECALL: 3
      F-ORCH: 3
    rationale: D1=3 (no-signal); D2=3 (no-signal); D3=3 (no-signal); D4=3 
      (no-signal); F-RECALL=3 (no-signal); F-ORCH=3 (no-signal)
    rubric_sha: e4a00f38e801
---

# T-3455: arc-011 PH-3 input: cross-hub sidecar delivery gaps measured by dimitri-mint-dev

## Problem Statement

**Parked by operator ruling (T-3454, option B, 2026-10-17): shaping starts at arc-011 PH-2 exit, run WITH the operator** (dimitri-mint-dev asked on his behalf, msg 649c4ff1, conv tl-xhub-sidecar, 2026-10-03). Cross-host delivery is PH-3 scope (`docs/design/arc-011-ph2-architecture.md:328`).

Today an agent on another hub (e.g. ring20-management, .122) is reachable only by `agent contact` DM plus `remote inject`: no retry ladder, no ack tracking, presence often OFFLINE while the agent is alive.

Gaps as measured by dimitri-mint-dev (framework 1.7.424, termlink 0.12.103), with current status:
1. Remote hub version probe could not use credentials — **resolved upstream since** (AEF T-3806, in vendored 1.8.10: `probe_hub` resolves a hubs.toml profile name and does an authenticated floor read, `.agentic-framework/lib/sidecar/termlink_transport.py:187-203`). Re-measure live at shaping time.
2. Addressing: the remote inbox topic is built with OUR hub id (`inbox:cacc73ea32b121dd/<remote-project>`); no remote hub-id discovery. Open.
3. The remote hub (.122) has no `inbox:<hubid>/<project>` sidecar topic, only legacy `inbox:ring20-management`. Open.
4. No retry ladder or delivery stages across hubs. Open.
5. Minor: `fw sidecar whoami` showed identity "- (termlink unreachable)" when `termlink whoami` is ambiguous with several sessions on the hub. Re-measure.

Inputs that PH-2 is still producing: P10 hub signing key, revocation (OQ-h), W2 measurements. When PH-3 starts, tell dimitri-mint-dev on conv tl-xhub-sidecar.

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

**Recommendation:** DEFER

**Rationale:**

Operator ruled option B on 2026-10-17 (T-3454): cross-hub delivery is arc-011 PH-3 scope (docs/design/arc-011-ph2-architecture.md:328) and depends on PH-2 outputs still being built (P10 hub signing key, revocation, W2 measurements). Shaping starts at PH-2 exit, with the operator.

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
