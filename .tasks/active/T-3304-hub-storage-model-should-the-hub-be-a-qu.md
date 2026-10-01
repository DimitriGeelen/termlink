---
id: T-3304
name: "Hub storage model: should the hub be a queryable source of truth? (retention,
  pruning, query, topic discovery)"
description: >
  Inception: Hub storage model: should the hub be a queryable source of truth? (retention,
  pruning, query, topic discovery)

status: started-work
workflow_type: inception
owner: human
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-10-01T18:13:59Z
last_update: 2026-10-01T18:14:36Z
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
  - ts: '2026-10-01T18:14:36Z'
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

# T-3304: Hub storage model: should the hub be a queryable source of truth? (retention, pruning, query, topic discovery)

## Problem Statement

While walking SQ-11 (T-2573, subscribe deadline drops collected messages) the
operator raised the principle underneath it: does the hub store messages as a
**queryable source of truth**, or is it a transport where agents keep their own
history? That determines retention/pruning defaults, the query model (cursor +
filter today; no time-range, cross-topic or metadata index), and how agents
discover and select topics. Charter non-goal #2 currently says "Not a durable
database or system of record" — this inception tests that stance. The operator
asked to consult three non-Anthropic agents first ("there's real value in there").
Research artifact: `docs/reports/T-3304-hub-storage-model.md`.

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

- **IW-1: Should the hub be a queryable source of truth, or a transport where agents keep their own history?**
  confidence: 0
  disposition:
  rationale:
- **IW-2: What retention and pruning model (defaults, who decides, automatic vs explicit sweep)?**
  confidence: 1
  disposition:
  rationale:
- **IW-3: Is cursor + optional filter enough, or are time-range / cross-topic / metadata-index queries needed?**
  confidence: 0
  disposition:
  rationale:
- **IW-4: How should agents discover and select the topics relevant to them?**
  confidence: 0
  disposition:
  rationale:
- **IW-5: Does the answer change the SQ-11 (T-2573) fix choice?**
  confidence: 1
  disposition:
  rationale:

## Exploration Plan

1. Write the brief with measured facts (`docs/reports/T-3304-consult/brief.md`).
2. Send the identical brief to three non-Anthropic agents, read-only: Codex (OpenAI,
   ChatGPT subscription), GLM-5.3 via opencode (Z.AI Coding Plan), qwen3:14b local
   via Ollama. Answers saved verbatim under `docs/reports/T-3304-consult/`.
3. Synthesise agreements/disagreements; walk the operator through it one question at a time.

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
- The operator chooses a storage stance (transport vs queryable store) and the
  charter non-goal #2 is either reaffirmed or amended by the operator
- The resulting changes (retention defaults, query additions, topic discovery)
  decompose into bounded build tasks

**NO-GO if:**
- The current stance (retention-bounded coordination log) is reaffirmed and no
  query/discovery change is warranted

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

No evidence yet; operator asked to consult three non-Anthropic agents (Codex, GLM-5.3 via opencode, local qwen3) before forming a view. Charter non-goal #2 currently says the hub is not a system of record; this inception tests that.

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

### 2026-10-01T18:14:36Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
