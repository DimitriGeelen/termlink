---
id: T-3477
name: "Container isolation for dispatched agent workers: per-claim container instead
  of bare clone or worktree"
description: >
  Inception: Container isolation for dispatched agent workers: per-claim container
  instead of bare clone or worktree

status: started-work
workflow_type: inception
owner: human
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-10-19T13:37:19Z
last_update: 2026-10-19T18:39:38Z
date_finished:
# revisit_at: YYYY-MM-DD          # T-1451: set on DEFER decisions to enable G-053 daily revisit scan
# revisit_evidence_needed:        # T-1451: one-line description of what evidence makes the revisit actionable
# ── Inception scoring exception (T-2186 Slice 2 / T-2188). See 050-Inceptions.md §Scoring Exception. ──
target_blast_radius: 3            # int 0..9. Anticipated component count of the build work this inception would authorise on GO.
                                  # Substitutes for the absent components: list in the F8 cost formula (040). Required.
                                  # Guide: 0=docs only, 1=single file, 3=small subsystem (S), 5=cross-subsystem (M), 7=multi-arc (L), 9=framework-wide (XL).
voi_score: 0.8                    # float 0..1. Value of Information — expected value of resolving this question,
                                  # independent of build cost. Higher when answer affects many tasks or unblocks a strategic decision. Required.
bvp_scores_proposed:
  - ts: '2026-10-19T13:38:40Z'
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

# T-3477: Container isolation for dispatched agent workers: per-claim container instead of bare clone or worktree

## Problem Statement

Parallel agent workers keep colliding with shared state (session cwd, untracked framework
files and hooks, basename-derived names, the local task counter). The five T-3469 reviewers
agreed a bare clone fixes only the shared `.git`; two named a per-task container as the
prior-art answer. Operator, 2026-10-19: explore containers for more isolation.
Research: docs/reports/T-3477-container-isolation-workers.md.

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

- **IW-1: Can a Claude Code / Codex worker authenticate inside a container without copying the operator's credential files?**
  confidence: 0
  disposition:
  rationale:
- **IW-2: Can the image carry the framework and hooks so governance fails closed inside the container, with task IDs from the central allocator (T-3476)?**
  confidence: 1
  disposition:
  rationale:
- **IW-3: Does TermLink work from inside (hub TLS, presence, claims, DM, waker)?**
  confidence: 1
  disposition:
  rationale:
- **IW-4: What does it cost (start time, image size, disk with build cache) on a host at 88% disk?**
  confidence: 0
  disposition:
  rationale:
- **IW-5: Docker container or a lighter bubblewrap/unshare sandbox; and what about hosts without Docker?**
  confidence: 0
  disposition:
  rationale:
- **IW-6: Placement: how does a dispatcher pick a host with spare capacity (CPU, memory, disk) and start the worker there?**
  confidence: 0
  disposition:
  rationale: operator input 2026-10-21 (below): the reason for containers is spreading worker load across hosts
- **IW-7: Which run model: (a) a container spawned per task on the chosen host, (b) standing containers per host that accept tasks, (c) a central definition (image + config) each host pulls and spawns from; or a mix?**
  confidence: 0
  disposition:
  rationale: operator named all three on 2026-10-21
- **IW-8: Which hub owns the dispatch record of a worker running on another host? Hubs do not federate (G-060) and the shared contract says the record's hub is the hub the worker registers with.**
  confidence: 0
  disposition:
  rationale:
- **IW-9: How do the worker's results come back across hosts (push a branch to OneDev, or a patch/artifact through the hub) and land safely (verify before land, CAS)?**
  confidence: 0
  disposition:
  rationale:
- **IW-10: Who may start a worker on my host? Remote spawn is a privileged action and ties to the inject/identity work (T-3493 C, O-7).**
  confidence: 0
  disposition:
  rationale:

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

Shaping in progress. Two of five T-3469 reviewers (claude, glm) named a per-task container as the prior-art isolation (OpenHands, Codex cloud, Copilot agent) that also removes the hook/cwd/basename failure modes; operator asked to explore it. Docker 29.9 and bubblewrap are present on .107. Evidence to gather: credentials and auth inside a container without copying operator secrets, governance (hooks, untracked framework files) baked into an image, build-cache and disk cost, cross-host.

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

### 2026-10-19T13:38:40Z — status-update [task-update-agent]
- **Change:** status: captured → started-work

### 2026-10-21 — operator input: containers are for spreading work across hosts
- Operator, verbatim in substance: "I want containers for multiple reasons ... I've got multiple hosts and I'm running
  out of processing power. What's the ability then to spawn off a container in another host and fire off a task? A worker
  task." And: "Or we have standing containers where work tasks can be spawned, or we have configuration centrally which
  can be pulled and then spawned up, right?"
- Consequence for scope: this inception is no longer only per-claim isolation on one host; the main goal is cross-host
  worker placement for capacity. New open questions IW-6 (placement by capacity), IW-7 (per-task spawn vs standing pool vs
  centrally defined image pulled per host), IW-8 (which hub owns the record, given no federation), IW-9 (results back across
  hosts), IW-10 (who may start a worker on a host).
- Hard constraint carried from the operator's standing rules: the operator's own credentials (e.g. ~/.claude/.credentials.json)
  are never copied into a container or onto another host (IW-1 must find another route).
- VOI raised to 0.8 by the operator (decision 4, 2026-10-21).
