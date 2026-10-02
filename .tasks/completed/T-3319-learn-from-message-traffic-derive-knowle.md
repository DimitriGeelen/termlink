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

status: work-completed
workflow_type: inception
owner: human
horizon: null
tags: [arc:arc-012]
components: [runme.sh]
related_tasks: [T-3310, T-3321, T-3309, T-3304]
created: 2026-10-02T14:53:39Z
last_update: 2026-10-02T17:51:10Z
date_finished: 2026-10-02T17:51:10Z
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

The hubs carry a large flow of filings and messages (framework:pickup alone: 301 records), and
nothing turns it into answered work or shared knowledge. Measured 2026-10-02: 98 inbound filings
unprocessed for a week while the canary fired; AEF has posted nothing and left no receipt in weeks;
AEF's inbound learnings feed holds 1 of 407. Five defect classes recur across 2+ projects (T-3324).
For whom: every project filing to the framework, and the operator who reads "posted" as "told".

## Assumptions

- A1 (tested, held): the motivating failure is routing, not missing analysis — T-3324 found 98
  unread filings and a silent receiver.
- A2 (tested, held): the flow holds recurring cross-project classes worth mining — stop rule
  passed with 5 classes (T-3324).
- A3 (tested, failed as stated): "who said what" is reliable — every agent on a host signs with
  one identity; attribution must be fixed before per-agent claims (T-3325, 4/4 reviewers).
- A4 (tested, held): T-3310 retention will not destroy the raw material — framework:pickup and
  channel:learnings are operator-durable forever.

## Open Questions

<!-- T-2190 (T-2186 Slice 4): every IW-N question must be disposed before
     --status work-completed. Disposition gate (agents/task-create/update-task.sh
     check_disposition_gate) refuses on under-disposed inceptions.

     Per-question shape:

       - **IW-1: <question text>**
         confidence: 0-3      (your confidence in your current answer; 0=guess, 3=verified)
         disposition: answered
         rationale: operator ruling 2026-10-02 on IW-2 = C: derived knowledge lives in AEF memory; termlink publishes evidence only

     Never bare yes/no — the gate refuses bare checkboxes. See 050-Inceptions.md
     §Disposition Gate. Bypass: --skip-disposition-gate "rationale" (direct) or
     FW_SKIP_DISPOSITION_GATE=1 (env-var, T-1890 producer/consumer parity).
-->

- **IW-0: Goal — make sure filings get answered (routing), or learn patterns from them (mining)?**
  confidence: 3
  disposition: answered
  rationale: operator ruling 2026-10-02 = C (routing first, mining folded into triage); see ## Decisions and the research artifact Dialogue Log
- **IW-1: Where does derived knowledge live — AEF project memory, small hub views, or a separate store?**
  confidence: 3
  disposition: answered
  rationale: operator ruling 2026-10-02 on IW-2 = C: derived knowledge lives in AEF memory; termlink publishes evidence only (4/4 reviewers concur)
- **IW-2: Who runs any miner — AEF or TermLink?**
  confidence: 3
  disposition: answered
  rationale: operator ruling 2026-10-02 = C: termlink runs the extraction and publishes a digest on framework:pickup
- **IW-3: Capture before trim — snapshot anything before T-3310 retention trims it?**
  confidence: 2
  disposition: dissolved
  rationale: framework:pickup and channel:learnings are operator-durable forever; T-3310 never trims them
- **IW-4: Hub-side "unanswered age" view — coordination state on the hub, or analytics kept in AEF?**
  confidence: 1
  disposition: deferred
  rationale: the digest needs no hub view; revisit after the first digest shows whether 'unanswered age' is wanted live
- **IW-5: LLM involvement in summarising/clustering — none, local-only (Ollama), or remote?**
  confidence: 0
  disposition: deferred
  rationale: the first digest is deterministic (offsets, projects, receipts); no LLM needed

## Exploration Plan

Done: research artifact + read-only measurement; four isolated external reviews; Q0 ruling; pickup
backlog triage as the stop-rule test (T-3324); IW-2 ruling.

## Technical Constraints

<!-- What platform, browser, network, or hardware constraints apply?
     For web apps: HTTPS requirements, browser API restrictions, CORS, device support.
     For hardware APIs (mic, camera, GPS, Bluetooth): access requirements, permissions model.
     For infrastructure: network topology, firewall rules, latency bounds.
     Fill this BEFORE building. Discovering constraints after implementation wastes sessions. -->

## Scope Fence

IN: termlink extracts transport evidence (classes with offsets and raising projects, unanswered
filings and ages, read receipts, attribution confidence) and publishes it as a digest.
OUT: termlink writing into AEF memory or tasks; hub-side analytics views (IW-4); LLM classification
(IW-5); any archive of raw messages.

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
- The stop rule passes (>= 2 recurring cross-project classes in the flow) — it did: 5
- A digest can be produced from data the hub already holds, without new hub analytics state

**NO-GO if:**
- Fewer than 2 recurring cross-project classes (nothing to mine)
- Producing it would need termlink to interpret or write AEF's own memory (G-062)

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

**Rationale:** Both GO criteria hold. The stop rule passed with 5 recurring cross-project classes (T-3324), and the hub already holds what a digest needs: filing offsets, raising projects, receipts, T-3309 activity. Operator rulings: Q0 = C (routing first, mining folded into triage, done in T-3324); IW-2 = C (termlink extracts and publishes an evidence digest; AEF owns conclusions and memory). GO authorises one build task: a first evidence digest posted to framework:pickup, plus per-agent attribution (T-3325) before any per-agent claim.

**Evidence:**
- docs/reports/T-3324-pickup-triage.md: 98 filings coded; 5 classes raised by 2+ projects, all AEF's
- docs/reports/T-3319-learn-from-message-traffic.md § Synthesis: 4/4 reviewers say AEF memory, no hub archive
- framework:pickup offsets 299-301: our receipt, the AEF unanswered-filing audit proposal, our read receipt

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

### 2026-10-02 — IW-2: who does the mining (operator ruling)
- **Chose:** C — termlink extracts and publishes an evidence digest (recurring classes with offsets
  and raising projects, unanswered filings and ages, read receipts, attribution confidence) on
  framework:pickup; AEF interprets it and owns the knowledge in its memory; termlink never writes
  into AEF's learnings or tasks.
- **Why:** only the hub sees delivery and readership; all 5 recurring classes are AEF's defects;
  AEF is currently silent, so a joint start would not start (Codex: "the hub owns transport facts;
  AEF owns interpreted knowledge").
- **Rejected:** A joint co-design first (-2, blocked on a silent partner); B mine and share
  unspecified "outcomes" (+35, drifts into interpreting AEF's code, G-062); D AEF alone (-20, cannot
  see receipts). C scored +60.
- **Left open:** digest cadence; IW-4 (hub view) and IW-5 (LLM) deferred.

### 2026-10-02 — Q0 stop-rule result (from T-3324)
- The pre-set stop rule PASSED: 5 classes were raised by 2+ projects in offsets 163-298. All five
  are AEF's defect classes; only 1 of 98 inbound filings was addressed to termlink. So mining has
  material, and the material is about the framework — which bears directly on IW-1/IW-2.

## Decision

**Decision**: GO

**Rationale**: Operator rulings 2026-10-02: Q0 = C (routing first, mining folded into triage, done in T-3324: 98 filings coded, stop rule passed with 5 recurring cross-project classes); IW-2 = C (termlink extracts and publishes an evidence digest on framework:pickup, AEF owns the conclusions and its memory, termlink never writes AEF memory). GO authorises one build task: the first evidence digest, with per-agent attribution (T-3325) before any per-agent claim.

**Date**: 2026-10-02T17:51:10Z

## Updates

### 2026-10-02T18:05Z — operator rulings recorded [claude]
- Q0: operator "Proceed is suggested" -> C. IW-2: operator "Proceed as suggested" -> C.
- GO is Tier 0 (human): queued in runme.sh action 3 for the operator to record.

<!-- Auto-populated by git mining at task completion.
     Manual entries optional during execution. -->

### 2026-10-02T17:28:25Z — status-update [task-update-agent]
- **Change:** status: captured → started-work

### 2026-10-02T17:51:10Z — inception-decision [inception-workflow]
- **Action:** Recorded inception decision
- **Decision:** GO
- **Rationale:** Operator rulings 2026-10-02: Q0 = C (routing first, mining folded into triage, done in T-3324: 98 filings coded, stop rule passed with 5 recurring cross-project classes); IW-2 = C (termlink extracts and publishes an evidence digest on framework:pickup, AEF owns the conclusions and its memory, termlink never writes AEF memory). GO authorises one build task: the first evidence digest, with per-agent attribution (T-3325) before any per-agent claim.

## Reviewer Verdict (v1.5)

- **Scan ID:** R-722f8f6a
- **Timestamp:** 2026-10-02T17:51:12Z
- **Catalogue:** v1.3-seed
- **Overall:** CONCERN
- **Needs Human:** no
- **Reviewer:** inline
- **Findings:** 2

**Verification-level findings:**

  1. **disposition-incomplete** (partial, heuristic) @ ## Open Questions: IW-1
     - evidence: `IW-1 disposition='answered' but rationale has no evidence citation (T-NNNN, file:line, docs/reports/, G-/L-/D-id, dialogue-log, or commit hash)`
  2. **disposition-incomplete** (partial, heuristic) @ ## Open Questions: IW-2
     - evidence: `IW-2 disposition='answered' but rationale has no evidence citation (T-NNNN, file:line, docs/reports/, G-/L-/D-id, dialogue-log, or commit hash)`

## Recommendation Verdict (v1.0)

- **Scan ID:** RC-a80a23c2
- **Timestamp:** 2026-10-02T17:51:12Z
- **Overall:** CONFIRMED
- **Claims:** 3

| Claim | Type | Status |
|-------|------|--------|
| `T-3324` | task | ✓ pass |
| `T-3309` | task | ✓ pass |
| `T-3325` | task | ✓ pass |

### 2026-10-02T17:51:10Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
- **Reason:** Inception decision: GO
