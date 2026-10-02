---
id: T-3319
name: "Learn from message traffic: derive knowledge from the hub's message and event
  flow (with AEF, external review, operator discussion)"
description: >
  Operator 2026-10-02 during T-3310: the hub carries a huge flow of messages and events;
  we can learn from it rather than only retain or prune it. Its own inception, involving
  the Agentic Engineering Framework (post to framework:pickup, invite AEF into the
  discussion), an external review (non-Anthropic harnesses, as in T-3304) and a discussion
  with the operator soon. Horizon now. Raw material already recorded: T-3309 per-topic
  activity (writers, readers, unread), framework:pickup filings, inbox rails, audit
  warnings that AEF folds into tasks.

status: started-work
workflow_type: inception
owner: human
horizon: now
tags: [arc:arc-012]
components: []
related_tasks: [T-3310, T-3321, T-3309, T-3304]
created: 2026-10-02T14:53:39Z
last_update: 2026-10-02T17:28:25Z
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
  - ts: '2026-10-02T17:28:25Z'
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

# T-3319: Learn from message traffic: derive knowledge from the hub's message and event flow (with AEF, external review, operator discussion)

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

- **IW-0: Goal — make sure filings get answered (routing), or learn patterns from them (mining)?**
  confidence: 3
  disposition: answered
  rationale: operator ruling 2026-10-02 = C (routing first, mining folded into triage); see ## Decisions and the research artifact Dialogue Log
- **IW-1: Where does derived knowledge live — AEF project memory, small hub views, or a separate store?**
  confidence: 2
  disposition:
  rationale: 4/4 reviewers say AEF memory (docs/reports/T-3319-learn-from-message-traffic.md § Synthesis); awaits operator
- **IW-2: Who runs any miner — AEF or TermLink?**
  confidence: 1
  disposition:
  rationale: only asked if the Q0 triage passes the stop rule (>=2 recurring cross-project classes)
- **IW-3: Capture before trim — snapshot anything before T-3310 retention trims it?**
  confidence: 2
  disposition:
  rationale: framework:pickup and channel:learnings are operator-durable forever and never trimmed by T-3310; likely dissolves
- **IW-4: Hub-side "unanswered age" view — coordination state on the hub, or analytics kept in AEF?**
  confidence: 1
  disposition:
  rationale: reviewers split (GLM hub; qwen/Codex AEF-only)
- **IW-5: LLM involvement in summarising/clustering — none, local-only (Ollama), or remote?**
  confidence: 0
  disposition:
  rationale: not yet discussed

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

**Rationale:** The operator asked for it explicitly and wants it soon; the data exists (T-3309 activity tables, ~1000 records across inbox/dm, pickup filings) and nothing derives knowledge from it today. GO here authorises exploration only: research artifact, AEF filing, external consultation, operator dialogue. No build.

## Decisions

### 2026-10-02 — Q0: routing or mining (operator ruling)
- **Chose:** C — routing first, mining folded into the triage. Triage the framework:pickup
  backlog by addressee (answer/file what concerns TermLink, acknowledge the rest), coding each
  filing's class (addressee, class, already-seen-elsewhere) as the mining yield test; propose an
  "unanswered filing" audit to AEF. Stop rule set in advance: fewer than 2 recurring
  cross-project classes -> no miner.
- **Why:** the proven failure is routing: 98 inbound filings unprocessed since 2026-09-25 while the
  pickup canary fired daily; AEF's inbound learnings feed holds 1 of 407; AEF's own router exists
  because a bug report sat unread for three months. Codex and GLM diagnose routing; the backlog
  must be read anyway, so coding classes costs minutes.
- **Rejected:** A routing only (+43: answers repeats one at a time forever); B mining only (+21:
  learns from filings nobody answers); D defer (-38: how the 98 accumulated). C scored +58.
- **Left open:** IW-1..IW-5.

## Decision

<!-- Filled at completion via: fw inception decide T-XXX go|no-go --rationale "..." -->

## Updates

<!-- Auto-populated by git mining at task completion.
     Manual entries optional during execution. -->

### 2026-10-02T17:28:25Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
