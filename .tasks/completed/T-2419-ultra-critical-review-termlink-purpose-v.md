---
id: T-2419
name: "Ultra-critical review: termlink purpose vs current state — gap identification"
description: >
  Inception: Ultra-critical review: termlink purpose vs current state — gap identification

status: work-completed
workflow_type: inception
owner: human
horizon: null
tags: []
components: [crates/termlink-bus/src/lib.rs, crates/termlink-bus/src/meta.rs, crates/termlink-cli/src/cli.rs, crates/termlink-cli/src/commands/channel.rs, crates/termlink-cli/src/main.rs, crates/termlink-hub/src/channel.rs, crates/termlink-hub/src/cv_index.rs, crates/termlink-hub/src/router.rs, crates/termlink-hub/src/server.rs, crates/termlink-protocol/src/control.rs]
related_tasks: []
created: 2026-07-19T20:43:16Z
last_update: 2026-07-19T21:43:31Z
date_finished: 2026-07-19T21:43:31Z
# revisit_at: YYYY-MM-DD          # T-1451: set on DEFER decisions to enable G-053 daily revisit scan
# revisit_evidence_needed:        # T-1451: one-line description of what evidence makes the revisit actionable
# ── Inception scoring exception (T-2186 Slice 2 / T-2188). See 050-Inceptions.md §Scoring Exception. ──
target_blast_radius: 3            # int 0..9. Anticipated component count of the build work this inception would authorise on GO.
                                  # Substitutes for the absent components: list in the F8 cost formula (040). Required.
                                  # Guide: 0=docs only, 1=single file, 3=small subsystem (S), 5=cross-subsystem (M), 7=multi-arc (L), 9=framework-wide (XL).
voi_score: 0.5                    # float 0..1. Value of Information — expected value of resolving this question,
                                  # independent of build cost. Higher when answer affects many tasks or unblocks a strategic decision. Required.
---

# T-2419: Ultra-critical review: termlink purpose vs current state — gap identification

## Problem Statement

Operator directive: ultra-critically review termlink's purpose and goals, identify gaps
or needed adjustments, and drive scoped corrections to completion. Question: where has
termlink drifted from — or failed to deliver — its core purpose (reliable parallel-agent
coordination substrate), and which gaps warrant correction now? Full evidence + synthesis:
docs/reports/T-2419-ultra-critical-purpose-review.md

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

- **IW-1: What is termlink's stated purpose, and is it coherent across README/ADR/docs — or has the goalpost silently moved?**
  confidence: 3
  disposition: answered
  rationale: Two unreconciled eras: README/Cargo era-1 terminal-control (37 tools claimed) vs ADR substrate mission (276 live) — docs/reports/T-2419 §1

- **IW-2: Does the feature surface (CLI verbs, MCP tools, ops docs, canaries) serve that purpose proportionally, or has observability/parity sprawl outgrown core-capability investment?**
  confidence: 3
  disposition: answered
  rationale: No: CLI+MCP=71% of 164K LOC, 276 MCP tools ~5x verb fan-out, log/history layers zero field usage — §2/§4

- **IW-3: What failure classes recur in the concerns/gaps/learnings registers, and do they indicate structural purpose-level gaps (vs incidental bugs)?**
  confidence: 3
  disposition: answered
  rationale: Four structural classes recur: silent-failure (49L/10 concerns), deploy-staleness (68L), identity/authz (38L+G-064), federation absence — §3

- **IW-4: What does the fleet actually USE (traffic, primitives, verbs) versus what has been built — where is dead weight and where is unmet need?**
  confidence: 3
  disposition: answered
  rationale: Core usage = presence/DM/chat-arc/file/exec; observability logs ABSENT; 60% of 1581 topics are test debris self-DoSing walk verbs — §4

- **IW-5: Which identified gaps are correctable with scoped, testable work now, and which need dedicated inceptions or operator decisions?**
  confidence: 3
  disposition: answered
  rationale: GAP-1 README + GAP-2 delete-topic buildable now; GAP-4 authz + GAP-5 deploy need inceptions; GAP-3/6/7 route to existing arcs — §6

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
- [x] Problem statement validated
<!-- @auto-tick-on-decide -->
- [x] Assumptions tested
<!-- @auto-tick-on-decide -->
- [x] Recommendation written with rationale

### Human
<!-- @auto-tick-on-decide -->
- [x] [REVIEW] Review exploration findings and approve go/no-go decision
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

Review conducted (4 evidence sweeps, all IW questions answered at confidence 3). Seven
purpose-level gaps identified with clear per-gap dispositions: two are scoped, testable,
reversible corrections buildable now (GAP-1 front-door purpose truth: README/Cargo state
era-1 identity + 37-vs-276 tool count, no guarantees/trust/federation statement; GAP-2
no topic-deletion primitive: 60% of the production hub's 1,581 topics are test debris
that self-DoSes walk verbs). Two need their own inceptions + operator decisions (GAP-4
authorization model / G-064; GAP-5 managed deploy story — #1 incident source at 68
learnings). Three route to existing arcs/policy (GAP-3 arc-005 MCP slimming; GAP-6
5-slice sprawl policy; GAP-7 loud-contract arc). GO authorizes the two scoped builds +
the two inception filings; NO fundamental redesign is proposed.

**Evidence:**

- docs/reports/T-2419-ultra-critical-purpose-review.md §1-§6 (full sweeps + synthesis)
- README.md:3-4 vs docs/architecture/parallel-execution-substrate.md §1 (purpose split)
- LOC: CLI 66,819 + MCP 49,804 = 71% of 164K; core 29%
- ~/.termlink/{find-idle,governor,claims,queue,heal}.log ALL ABSENT (unused surface)
- channel list: 1,581 topics, ~617 t-*/107 xhub-*/96 stress-* debris; claims-summary
  --all hits -32008 rate-limit mid-walk
- Learning clusters: deploy 68, parsing 61, silent-failure 49, auth 43, identity 38

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

**Decision**: GO

**Rationale**: Approved via Watchtower (no rationale captured)

**Date**: 2026-07-19T21:43:31Z

## Updates

<!-- Auto-populated by git mining at task completion.
     Manual entries optional during execution. -->

### 2026-07-19T20:43:46Z — status-update [task-update-agent]
- **Change:** status: captured → started-work

### 2026-07-19T21:43:31Z — inception-decision [inception-workflow]
- **Action:** Recorded inception decision
- **Decision:** GO
- **Rationale:** Approved via Watchtower (no rationale captured)

## Reviewer Verdict (v1.5)

- **Scan ID:** R-7a684cd6
- **Timestamp:** 2026-07-19T21:43:32Z
- **Catalogue:** v1.3-seed
- **Overall:** CONCERN
- **Needs Human:** no
- **Findings:** 4

**Verification-level findings:**

  1. **disposition-incomplete** (partial, heuristic) @ ## Open Questions: IW-1
     - evidence: `IW-1 disposition='answered' but rationale has no evidence citation (T-NNNN, file:line, docs/reports/, G-/L-/D-id, dialogue-log, or commit hash)`
  2. **disposition-incomplete** (partial, heuristic) @ ## Open Questions: IW-2
     - evidence: `IW-2 disposition='answered' but rationale has no evidence citation (T-NNNN, file:line, docs/reports/, G-/L-/D-id, dialogue-log, or commit hash)`
  3. **disposition-incomplete** (partial, heuristic) @ ## Open Questions: IW-4
     - evidence: `IW-4 disposition='answered' but rationale has no evidence citation (T-NNNN, file:line, docs/reports/, G-/L-/D-id, dialogue-log, or commit hash)`
  4. **disposition-incomplete** (partial, heuristic) @ ## Open Questions: IW-5
     - evidence: `IW-5 disposition='answered' but rationale has no evidence citation (T-NNNN, file:line, docs/reports/, G-/L-/D-id, dialogue-log, or commit hash)`

### 2026-07-19T21:43:31Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
- **Reason:** Inception decision: GO
