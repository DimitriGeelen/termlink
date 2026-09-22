---
id: T-3060
name: "Six unassessed plugins appeared after the T-3055 survey - assess them and why
  the set changed"
description: >
  Inception: Six unassessed plugins appeared after the T-3055 survey - assess them
  and why the set changed

status: started-work
workflow_type: inception
owner: human
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-09-22T08:09:10Z
last_update: '2026-09-22T14:57:32Z'
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
  - ts: '2026-09-22T08:22:03Z'
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
  - ts: '2026-09-22T14:57:32Z'
    estimator: bvp-estimator-v1-heuristic
    cost_estimate:
      blast_radius: 3
      tier: 4
      effort: 6
    rationale: blast_radius=3 (target_blast_radius:inception-T-2189); tier=4 
      (workflow:inception); effort=6 (lines=148,acs=4)
    rubric_sha: e4a00f38e801
---

# T-3060: Six unassessed plugins appeared after the T-3055 survey - assess them and why the set changed

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

- **IW-1: Does each of the six plugins satisfy the operator's standing standalone constraint — no dependency on any service, commercial or non-commercial?**
  confidence: 1
  disposition:
  rationale:

- **IW-2: Why did the enabled plugin set change from 13 to 15 between the T-3055 survey and the post-disable verification, with nothing recording the change?**
  confidence: 0
  disposition:
  rationale:

- **IW-3: For each plugin that passes IW-1, does its value to THIS repo justify its per-session context cost?**
  confidence: 1
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

**Recommendation:** GO

**Rationale:**

GO on assessing, not adopting. T-3055 surveyed 19 plugins on 2026-09-22 and the operator disabled four on its recommendation - confirmed disabled - yet the enabled count went 13 to 15, not 13 to 9, because SIX plugins absent from that survey are now present: semgrep, pyright-lsp, claude-md-management, hookify, mcp-server-dev, plugin-dev. None were assessed and several look materially more relevant than anything the survey covered. semgrep is industrial static analysis while this project hand-rolls eleven-plus source-level guard checks in bash and grep. pyright-lsp would cover the Python in web/ and lib/ the way rust-analyzer now covers the crates. claude-md-management bears on a 2500-line CLAUDE.md with a documented clobber defect (T-2015, 844 lines destroyed per upgrade). mcp-server-dev and plugin-dev bear on termlink-mcp, which this repo builds. Assess each against the operator standing constraint - stand-alone, no service dependency commercial or otherwise - via claude plugin details, where an MCP server is the tell for an external call and an LSP server is a local binary. Also establish WHY the set changed: a plugin set that shifts without anyone noticing makes every survey a snapshot with an unknown shelf life, which is the more durable finding.

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

### 2026-09-22 — semgrep: uninstall (operator decision)
- **Chose:** Uninstall the `semgrep` plugin.
- **Why:** Operator instruction, given before this inception assessed it. Sovereign
  call — recorded as a decision, not derived as a finding.
- **Rejected:** Holding the uninstall until IW-1/IW-3 were answered for semgrep. The
  operator did not ask for that assessment first, and the standing direction is theirs
  to set.
- **Scope:** Decides ONE of the six. It does not answer IW-1 or IW-3, which stay open
  for pyright-lsp, claude-md-management, hookify, mcp-server-dev and plugin-dev.

## Decision

<!-- Filled at completion via: fw inception decide T-XXX go|no-go --rationale "..." -->

## Updates

<!-- Auto-populated by git mining at task completion.
     Manual entries optional during execution. -->

### 2026-09-22T08:22:03Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
