---
id: T-3055
name: "Which available plugins and MCP servers could materially help this project"
description: >
  Inception: Which available plugins and MCP servers could materially help this project

status: started-work
workflow_type: inception
owner: human
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-09-22T06:51:42Z
last_update: 2026-09-22T06:55:05Z
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
  - ts: '2026-09-22T06:52:55Z'
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

# T-3055: Which available plugins and MCP servers could materially help this project

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

- **IW-1: What is actually installed and enabled, versus merely available?**
  confidence: 3
  disposition: answered
  rationale: 19 plugins, 13 enabled / 6 disabled, via `claude plugin list`. rust-analyzer-lsp
    appears TWICE. docs/reports/T-3055-plugin-survey.md.

- **IW-2: Which of them serve work this project actually does — guard/verification,
  agent comms, docs accuracy — rather than work it does not do?**
  confidence: 3
  disposition: answered
  rationale: Measured by repo references: playwright 330 files (mandated by CLAUDE.md),
    context7 23, qdrant 9, rust-analyzer 4; chrome-devtools / data-engineering /
    frontend-design / superdesign / modern-web-guidance / ralph-loop all ZERO.

- **IW-3: What does each enabled plugin COST? Tool-list budget, context per call, and
  a maintenance owner. An enabled plugin nobody uses is not free — it competes for the
  same attention as the guard layer.**
  confidence: 3
  disposition: answered
  rationale: Measured via `claude plugin details` after the operator's standalone
    constraint made it decisive — ~2,275 tok always-on total, of which 1,835 (81%) is the
    four zero-reference purpose-mismatch plugins. Also surfaced that context7 is the only
    enabled plugin with an external SERVICE dependency (1 MCP server, remote docs API),
    which reverses its endorsement. See the ADDENDUM in docs/reports/T-3055-plugin-survey.md.

- **IW-4: Is anything enabled that duplicates something the repo already has? The repo
  ships its own guard layer, review tooling and comms rail; a plugin doing the same job
  worse is a DELETE candidate, not an addition.**
  confidence: 2
  disposition: answered
  rationale: Yes — chrome-devtools-mcp overlaps playwright, which CLAUDE.md already
    mandates. Separately, github and commit-commands would CONFLICT with governance
    (never-push-to-GitHub; fw git commit task traceability) and are correctly disabled.

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

GO on a read-only survey, not on adopting anything. The session already exposes several MCP surfaces (fw, termlink, context7, playwright, chrome-devtools, Claude Docs) plus whatever the plugin system has installed, and no one has assessed which of them serve this project's actual work versus which are ambient noise. The question is worth asking now specifically because the last hours surfaced two classes these tools bear on: guard/verification work that a browser-automation or docs surface could make cheaper, and agent-to-agent comms where termlink is already load-bearing. Adoption of any plugin is a separate decision with its own cost - dependency surface, tool-list budget, and a maintenance owner - so this survey classifies and recommends only.

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

### 2026-09-22T06:52:55Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
