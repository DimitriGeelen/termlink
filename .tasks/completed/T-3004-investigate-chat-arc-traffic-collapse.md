---
id: T-3004
name: "Investigate chat-arc traffic collapse"
description: >
  S-29/C-40: agent-chat-arc traffic collapsed in the observed window; diagnose (participants
  gone vs rail broken vs measurement artifact). Evidence: docs/reports/VALUE-REVIEW-repo-2026-09-19-consolidated.md
  C-40.

status: started-work
workflow_type: inception
owner: agent
horizon: now
tags: [value-review, arc:arc-009]
components: []
related_tasks: []
created: 2026-09-19T22:26:41Z
last_update: 2026-09-20T08:55:53Z
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
  - ts: '2026-09-20T08:45:11Z'
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
  - ts: '2026-09-20T08:45:20Z'
    estimator: bvp-estimator-v1-heuristic
    cost_estimate:
      blast_radius: 3
      tier: 4
      effort: 6
    rationale: blast_radius=3 (target_blast_radius:inception-T-2189); tier=4 
      (workflow:inception); effort=6 (lines=115,acs=4)
    rubric_sha: e4a00f38e801
---

# T-3004: Investigate chat-arc traffic collapse

## Problem Statement

C-40: fleet-adoption snapshots show agent-chat-arc at 0 posts + 0 unique speakers for 10
consecutive daily snapshots after a progressive decline (270.4 → 12.1 → 31.9 → zeros), and the
topic was historically 92% single-sender (871/950 posts from `d1993c2c3ec44c94`). Is the fleet's
main broadcast rail dead, was it ever a real multi-agent rail, and is the zero-streak a traffic
fact or a measurement artifact (hub-dedup fix / snapshot pipeline change)? For: the operator
(whether "broadcast to the fleet" still reaches anyone). Why now: arc-009 S-29b slice.

## Assumptions

- A-1: The fleet-adoption snapshot series is on disk locally and can date the zero-streak start.
- A-2: The hub's own topic state (`channel list` / chat-arc recent) can distinguish "no posts
  arriving" from "snapshot pipeline stopped counting".
- A-3: `d1993c2c3ec44c94` is resolvable to an identity via local tooling (whoami/tofu/receipts).

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

- **IW-1: Does the zero-streak reproduce on the live topic, and when exactly did it start?**
  confidence: 3
  disposition: answered
  rationale: 36 consecutive zero blocks in .fleet-adoption-snapshot.log; daily cadence dates the start to 2026-08-16 — the T-2758 commit date (84df1c239). The TOPIC is not zero: posts exist from today on .107 (artifact F1/F2)

- **IW-2: Is the collapse a traffic fact or a measurement artifact — does the zero-streak start align with the hub-dedup fix commit or any snapshot-pipeline change?**
  confidence: 3
  disposition: answered
  rationale: Compound: .107 counter structurally blind (no latest_offset on pre-T-2533 hub; fallback tail=count-1 lands scan window in trimmed past — reproduced live, cursor 500 → 0 while tail holds today's posts); .122 copy genuinely quiet (stale poster, rollout-detector confirms); .121 now partial-unreachable (artifact F2/F3)

- **IW-3: Who is `d1993c2c3ec44c94` (92% of historical posts) — which host/agent identity, and is it still alive?**
  confidence: 3
  disposition: answered
  rationale: Shared HOST keypair on the dimitrimintdev dev host — signs both hourly dimitrimintdev-vendored T-1438 heartbeats AND 832-Workflow-designer notes (self-labeled in payload); alive today 08:05Z. T-2838 caveat confirmed: the stat measures a host, not an agent (artifact F4)

- **IW-4: What does the answer imply — is agent-chat-arc a live rail worth guarding, or a single-sender artifact whose zero-state is the honest reading?**
  confidence: 2
  disposition: answered
  rationale: Alive on the hub of record (.107), low-diversity (mostly automated heartbeats + peer notes); the zero is the measurement lying, not the rail dying — fix the counter (artifact Recommendation), hub upgrade dependency already owned by T-2977

## Exploration Plan

1. Locate fleet-adoption snapshot series on disk; date the decline and zero-streak. Time-box: 15 min.
2. Read live topic state (channel list count, chat-arc recent window) on the local hub. Time-box: 10 min.
3. Date the hub-dedup fix commit; compare to streak start. Time-box: 10 min.
4. Resolve the dominant sender fingerprint via local identity tooling. Time-box: 10 min.
5. Recommendation. Time-box: 15 min.

## Technical Constraints

<!-- What platform, browser, network, or hardware constraints apply?
     For web apps: HTTPS requirements, browser API restrictions, CORS, device support.
     For hardware APIs (mic, camera, GPS, Bluetooth): access requirements, permissions model.
     For infrastructure: network topology, firewall rules, latency bounds.
     Fill this BEFORE building. Discovering constraints after implementation wastes sessions. -->

## Scope Fence

IN: dating and attributing the chat-arc decline/zero-streak from local snapshots + local-hub reads;
identifying the dominant sender; recommendation on the rail's status.
OUT: reviving traffic, changing topic retention, fleet-wide hub probes beyond what reachability
allows, the ack-lag read side (T-3007's slice).

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

**Rationale:** Diagnosis complete — the rail is ALIVE on .107 (posts from today, 4 speakers/30d) while the snapshot reported 0/0 for 36 days: the counter is structurally blind on hubs without latest_offset (pre-T-2533 binary; fallback tail=count-1 lands the scan window entirely in the trimmed past — reproduced live). .122's copy genuinely quiet (stale poster, known), .121 partial-unreachable (known). GO on ONE small build task: fix the snapshot's tail derivation (page-forward fallback + never trust receipt up_to beyond derived tail) with a fixture pinning today's reproduce. Hub upgrade half is already owned by T-2977; .122/.121 already surfaced by rollout-detector/fleet doctor. Full evidence: docs/reports/T-3004-chat-arc-collapse-investigation.md

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

### 2026-09-19T22:35:35Z — status-update [task-update-agent]
- **Change:** tags: +arc:arc-009

### 2026-09-20T08:55:53Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
- **Change:** horizon: later → now (auto-sync)
