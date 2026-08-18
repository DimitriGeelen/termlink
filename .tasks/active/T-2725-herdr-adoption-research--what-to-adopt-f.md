---
id: T-2725
name: "Herdr adoption research — what to adopt from the herdr codebase"
description: >
  Inception: Herdr adoption research — what to adopt from the herdr codebase

status: started-work
workflow_type: inception
owner: human
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-08-15T07:47:26Z
last_update: '2026-08-18T18:58:40Z'
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
  - ts: '2026-08-18T18:55:38Z'
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
    rubric_sha: missing
cost_estimate_proposed:
  - ts: '2026-08-18T18:58:40Z'
    estimator: bvp-estimator-v1-heuristic
    cost_estimate:
      blast_radius: 3
      tier: 4
      effort: 7
    rationale: blast_radius=3 (no-signal); tier=4 (no-signal); effort=7 
      (no-signal)
    rubric_sha: missing
---

# T-2725: Herdr adoption research — what to adopt from the herdr codebase

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

- **IW-1: Which terminal-correctness edge cases does herdr's ~142-issue backlog
  name that TermLink's PTY layer does not handle?**
  confidence: 3
  disposition: answered
    rationale: worker 1 mapped 15 issue classes; 3 MEASURED live defects. Class A fixed in T-2727; class D filed as T-2728. .context/upstream/herdr-adopt-1-terminal-correctness.md

- **IW-2: Is a hybrid presence model (protocol when available, heuristic
  screen-scrape fallback when not) worth having, or does a heuristic that can
  lie break the liveness contract T-2387's canary depends on?**
  confidence: 3
  disposition: answered
    rationale: DO NOT RECOMMEND wiring a heuristic into agent-presence/find-idle/peers — a heuristic that lies breaks the liveness contract T-2387 depends on. .context/upstream/herdr-adopt-2-agent-state.md

- **IW-3: Is herdr's "CLI and socket API are the same surface" structurally
  better at preventing the CLI/MCP parity drift that went undetected from
  2026-08-12 until T-2686 wired cargo test into CI, and can we adopt it?**
  confidence: 3
  disposition: answered
    rationale: Yes, decisively — but because they have ONE implementation, not because of codegen (worker flagged the mechanism INFERRED). Our two crates are already siblings on termlink-protocol; only the response-envelope literals are duplicated. .context/upstream/herdr-adopt-3-parity.md

- **IW-4: Is herdr's client-server persistence architecture structurally more
  robust than our registration+heartbeat+pushwaker rail, given that three
  canaries (T-2239/T-2387/T-2405) exist because that rail keeps breaking?**
  confidence: 3
  disposition: answered
    rationale: No — worker found they are not looking rather than not failing. Two borrowable items (R1/R2); herdr --remote explicitly rejected. .context/upstream/herdr-adopt-4-persistence.md

- **IW-5: What specifically makes herdr's onboarding frictionless, and which of
  it is adoptable without giving up the guarantees our hub/secret/TLS-pinning
  complexity buys?**
  confidence: 3
  disposition: answered
    rationale: Several concrete items incl. preflight resolving the wrong runtime_dir (backlog rank 3). Default runtime_dir change is human-owned. .context/upstream/herdr-adopt-5-distribution.md

## Exploration Plan

Five independent read-only workers, one per open question, dispatched in
parallel via `fw termlink dispatch` (operator approved up to 8 termlink agents
on 2026-08-15). Each writes a findings file to `.context/upstream/` and returns
under 500 words. A sixth synthesis worker runs only after 1–5 return and
produces a ranked adoption backlog separating (a) ideas — free, (b) code —
needs Apache-2.0 attribution + NOTICE, (c) rejected with reason.

Time-box: workers are independent and bounded by their own dispatch timeout.
No source is modified by any worker — this is reconnaissance only.

**Standing methodology constraint carried into every worker brief:** cite a
source for every claim, and an asserted absence requires a cited file:line for
where you looked. A count of zero deserves more suspicion than a count of a few.

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

The replacement question is already settled NO (herdr socket API cannot return exit codes or deliver signals, both asserted by charter verb 4 prover session-selftest.sh; synthesis in .context/upstream/herdr-evaluation-synthesis-2026-08-15.md). This task asks the separate and still-open question of what to ADOPT — ideas, test cases, and architectural patterns — where I have no evidence yet beyond the recon. Five independent read-only workers gather it; the go/no-go per adoption item belongs to the human after synthesis. DEFER until worker findings land.

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

### 2026-08-15T07:48:41Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
