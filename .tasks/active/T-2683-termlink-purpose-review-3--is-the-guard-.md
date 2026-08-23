---
id: T-2683
name: "TermLink purpose review #3 — is the guard layer itself executed by anything?"
description: >
  Inception: TermLink purpose review #3 — is the guard layer itself executed by anything?

status: started-work
workflow_type: inception
owner: human
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-08-14T05:52:27Z
last_update: '2026-08-23T19:13:47Z'
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
  - ts: '2026-08-23T19:13:28Z'
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
cost_estimate_proposed:
  - ts: '2026-08-23T19:13:47Z'
    estimator: bvp-estimator-v1-heuristic
    cost_estimate:
      blast_radius: 3
      tier: 4
      effort: 7
    rationale: blast_radius=3 (no-signal); tier=4 (no-signal); effort=7 
      (no-signal)
    rubric_sha: e4a00f38e801
---

# T-2683: TermLink purpose review #3 — is the guard layer itself executed by anything?

## Problem Statement

<!-- What problem are we exploring? For whom? Why now? -->

Research artifact: `docs/reports/T-2683-guard-execution-review.md` (C-001).

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

- **IW-1: Does anything automatic execute the source-level guard layer — the 4 static
  checks, the 10 fixture suites, or the 2055 workspace tests?**
  confidence: 3
  disposition: answered
  rationale: No. `release.yml` runs `cargo build --release` with no `cargo test`;
  `doc-lint.yml` runs 2 of 28 check scripts; `.onedev-buildspec.yml` mirrors only;
  pre-push audit runs `Sections: structure`. Tree-wide grep for the four static checks
  outside their own implementation returns only their fixtures and `.context/episodic/*`.

- **IW-2: Do the canary crontabs preserve the exit-1 (finding) vs exit-2 (tooling
  error) split that every canary script implements and that T-2557 states is the
  reason a firing log stays meaningful?**
  confidence: 3
  disposition: answered
  rationale: No — 19/19 canary job lines use `>> <findings>.log 2>&1`, merging stderr
  into the findings log. Live proof: `.release-mirror-canary.log` contains
  `error: origin HEAD empty` while the script itself exits 0 (`GitHub mirror: synced`).

- **IW-3: Is a green report from a static check evidence, given T-2681 moved the
  allowlists to tracked paths?**
  confidence: 3
  disposition: answered
  rationale: Yes, and this worktree proves it — a fresh checkout with no
  `.context/working/` history scans clean on all four checks (101/6/14/39 sites),
  which is exactly the reproducibility property T-2681 was built to establish.

- **IW-4: Can the charter classify every live tool, or is there a surface it cannot
  see?**
  confidence: 3
  disposition: deferred
  rationale: It cannot. 32 live tools (`fleet` 12, `diagnostics` 12, `hub` 8) trace to
  no named verb and are invisible to both the drift canary and T-2548's scope.
  Deferred: extending the charter's wording is human-sovereign per T-2470.

- **IW-5: Should this review add new guards, as its two predecessors did?**
  confidence: 3
  disposition: dissolved
  rationale: The question dissolves on IW-1 — adding a sixth guard to a layer nothing
  invokes is not protection. Scope shifted from *more coverage* to *making existing
  coverage executable*. PL-271 applied to the reviewers themselves.

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

Third critical pass on TermLink's purpose and goals. The first two (T-2468 product-vs-charter, T-2678 guard-coverage) both concluded 'add more guards'; this pass asks whether the guards that exist are load-bearing at all. Three findings, all confirmed live on this tree: (F1) the source-level guard layer -- 4 static checks, 10 fixture suites, 2055 workspace tests -- is executed by NOTHING automatic: .github/workflows/release.yml runs cargo build and never cargo test, doc-lint.yml runs 2 of 28 check scripts, .onedev-buildspec.yml only mirrors, and the pre-push audit runs the 'structure' section only; grep confirms the four static checks are referenced by nothing but their own scripts, their own fixtures, and episodic memory. The guard layer has the exact disease the charter-drift canary docstring names as its reason to exist -- correction only when a human periodically asks for a hand review. (F2) 19 of 19 canary cron job lines append with '>> <findings>.log 2>&1', merging exit-2 tooling errors into the exit-1 findings log; a live instance exists right now -- .release-mirror-canary.log contains 'error: origin HEAD empty', which per CLAUDE.md directs an operator to rotate a GitHub token for what is actually an unrunnable check, and which permanently destroys the 'empty log = healthy' signal for that canary. (F3) the charter names four verbs but 32 live tools (fleet 12, diagnostics 12, hub 8) are pure observability tracing to no named verb, so they sit in a blind spot that neither T-2548's subtract decision nor the charter-drift canary can see. GO is recommended because F1 and F2 are mechanical, in-authority, and testable without any new product decision; F3's charter-wording half is human-sovereign and is proposed for deferral, not build.

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

### 2026-08-14T05:52:41Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
