---
id: T-3007
name: "Investigate ack-lag read side (unconfirmed-delivery aging)"
description: >
  S-29/C-43: awaiting-ack rows age without a read-side consumer acting on them; investigate
  whether the lag is recipient-side or tracker-side. Evidence: docs/reports/VALUE-REVIEW-repo-2026-09-19-consolidated.md
  C-43.

status: work-completed
workflow_type: inception
owner: agent
horizon: null
tags: [value-review, arc:arc-009]
components: []
related_tasks: []
created: 2026-09-19T22:29:24Z
last_update: 2026-09-21T11:07:57Z
date_finished: 2026-09-21T11:07:57Z
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

# T-3007: Investigate ack-lag read side (unconfirmed-delivery aging)

## Problem Statement

C-43: the receiver-ack-lag guard reports that on `agent-chat-arc` 4 of 5 identities have NEVER
acked and one lags by 1,533 — the G-063 write-only-sink shape on the fleet's main broadcast topic.
But an unacked identity is not necessarily an unread one: most read paths (recent-chat, snapshot
scans, subscribes) never emit receipts. Separate "nobody reads" from "readers don't ack" before
treating this as a consumption failure. For: the operator (whether broadcast reaches anyone) and
the guard's own credibility. Why now: arc-009 S-29e; sibling T-3004 just showed the same topic's
POST side being mismeasured.

## Assumptions

- A-1: `channel ack-status` rows on the local hub are inspectable and keyed by identity
  fingerprint (T-2838 item 1 caveat applies — a row is a host, not an agent).
- A-2: Read paths that don't ack (plain subscribe) exist and are the dominant consumption mode.
- A-3: T-3004's evidence (live posts today, shared host keypair) carries over to the read side.

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

- **IW-1: What does the ack-lag guard report right now, and which identities are the never-ackers?**
  confidence: 3
  disposition: answered
  rationale: rc=1 (permanently red): agent-chat-arc 4 NEVER-ACKED at lag=1554 (1da4fd99, 33df8954, 9219671e, fd794e54) + d1993c2c BEHIND (up_to=923, lag=630); framework:pickup also fires (2 never-acked lag=126, one behind by 44) — measured 2026-09-20

- **IW-2: Do unacked identities actually read the topic (unacked subscribe consumption), or is the topic genuinely unread by them?**
  confidence: 2
  disposition: answered
  rationale: Unanswerable from receipts BY DESIGN — every monitoring/read path (recent-chat, adoption snapshot, plain subscribe, this investigation) reads without acking and leaves no trace; T-3004 proved live readers exist on the topic the same day the guard called it unconsumed

- **IW-3: Which consumption paths emit receipts at all — is acking structurally rare by design, making never-acked the expected state for most identities?**
  confidence: 3
  disposition: answered
  rationale: Only the addressed flows ack (agent-respond receipt+reply via /check-arc, check-addressed-posts.sh); even framework:pickup's consumption contract acks via a LOCAL seen-offset file (T-2231) invisible to hub receipts — so yes, never-acked is the structural default

- **IW-4: Is the guard's firing semantics right for this rail (broadcast, mostly-automated posts), or does it need a topic-class exemption / different threshold to stay credible?**
  confidence: 3
  disposition: answered
  rationale: The 5 rows ARE the topic's 5 SENDERS (channel info .senders match exactly) — a sender-keyed check on a broadcast rail fires forever on its own posting bots (fd794e54: 1 post ever, permanent NEVER-ACKED row); permanently-red guard = T-2556/T-2818 fatigue class. Scope it to ack-contract topics (dm:* + include list), print exclusions

## Exploration Plan

1. Run `check-receiver-ack-lag.sh` + `channel ack-status` on agent-chat-arc; capture rows. Time-box: 10 min.
2. Grep the codebase for receipt-emitting paths (`channel ack` / ack-emitting verbs) vs plain-subscribe readers. Time-box: 15 min.
3. Correlate with T-3004's live-read evidence (who reads without acking). Time-box: 10 min.
4. Recommendation on guard semantics. Time-box: 15 min.

## Technical Constraints

<!-- What platform, browser, network, or hardware constraints apply?
     For web apps: HTTPS requirements, browser API restrictions, CORS, device support.
     For hardware APIs (mic, camera, GPS, Bluetooth): access requirements, permissions model.
     For infrastructure: network topology, firewall rules, latency bounds.
     Fill this BEFORE building. Discovering constraints after implementation wastes sessions. -->

## Scope Fence

IN: measuring current ack-status on agent-chat-arc, enumerating receipt-emitting vs plain read
paths, verdict on whether the guard's semantics fit a broadcast rail; recommendation only.
OUT: changing the guard, hub-side ack enforcement (T-2838's own territory), the POST-side counter
(T-3004's slice), per-agent identity keys (T-2838 item 1).

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

**Rationale:** C-43's G-063 framing over-reaches: the guard's rows are the topic's SENDERS (posters who never acked), not readers — on a broadcast rail its never-ackers are its own posting bots, and unacked subscribe-reads are invisible to receipts by design (T-3004 proved live readers exist the same day the guard called the topic unconsumed). The guard is permanently red (rc=1), the T-2556/T-2818 fatigue class. GO on ONE small build task: scope check-receiver-ack-lag.sh to ack-contract topics (dm:* + explicit include list), print exclusions counted-not-silent (T-2483 pattern), keep the sender-keyed caveat, document that broadcast consumption belongs to the adoption snapshot (post-T-3004 fix). Fixture: broadcast topic with never-acked bots must not fire; behind-threshold dm topic must. Findings + lag semantics (lag=1554 is frontier length, not backlog growth) recorded in the IW dispositions above; full artifact write deferred at session budget stop.

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

**Rationale**: Recommendation: GO

Rationale: C-43's G-063 framing over-reaches: the guard's rows are the topic's SENDERS (posters who never acked), not readers — on a broadcast rail its never-ackers are its own posting bots, and unacked subscribe-reads are invisible to receipts by design (T-3004 proved live readers exist the same day the guard called the topic unconsumed). The guard is permanently red (rc=1), the T-2556/T-2818 fatigue class. GO on ONE small build task: scope check-receiver-ack-lag.sh to ack-contract topics (dm: + explicit include list), print exclusions counted-not-silent (T-2483 pattern), keep the sender-keyed caveat, document that broadcast consumption belongs to the adoption snapshot (post-T-3004 fix). Fixture: broadcast topic with never-acked bots must not fire; behind-threshold dm topic must. Findings + lag semantics (lag=1554 is frontier length, not backlog growth) recorded in the IW dispositions above; full artifact write deferred at session budget stop.

**Date**: 2026-09-21T11:07:57Z

## Updates

<!-- Auto-populated by git mining at task completion.
     Manual entries optional during execution. -->

### 2026-09-19T22:35:36Z — status-update [task-update-agent]
- **Change:** tags: +arc:arc-009

### 2026-09-20T09:04:25Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
- **Change:** horizon: later → now (auto-sync)

### 2026-09-21T11:07:57Z — inception-decision [inception-workflow]
- **Action:** Recorded inception decision
- **Decision:** GO
- **Rationale:** Recommendation: GO

Rationale: C-43's G-063 framing over-reaches: the guard's rows are the topic's SENDERS (posters who never acked), not readers — on a broadcast rail its never-ackers are its own posting bots, and unacked subscribe-reads are invisible to receipts by design (T-3004 proved live readers exist the same day the guard called the topic unconsumed). The guard is permanently red (rc=1), the T-2556/T-2818 fatigue class. GO on ONE small build task: scope check-receiver-ack-lag.sh to ack-contract topics (dm: + explicit include list), print exclusions counted-not-silent (T-2483 pattern), keep the sender-keyed caveat, document that broadcast consumption belongs to the adoption snapshot (post-T-3004 fix). Fixture: broadcast topic with never-acked bots must not fire; behind-threshold dm topic must. Findings + lag semantics (lag=1554 is frontier length, not backlog growth) recorded in the IW dispositions above; full artifact write deferred at session budget stop.

## Reviewer Verdict (v1.5)

- **Scan ID:** R-fbf55829
- **Timestamp:** 2026-09-21T11:07:58Z
- **Catalogue:** v1.3-seed
- **Overall:** CONCERN
- **Needs Human:** no
- **Findings:** 1

**Verification-level findings:**

  1. **disposition-incomplete** (partial, heuristic) @ ## Open Questions: IW-1
     - evidence: `IW-1 disposition='answered' but rationale has no evidence citation (T-NNNN, file:line, docs/reports/, G-/L-/D-id, dialogue-log, or commit hash)`

## Recommendation Verdict (v1.0)

- **Scan ID:** RC-d9e83a9a
- **Timestamp:** 2026-09-21T11:07:58Z
- **Overall:** CONFIRMED
- **Claims:** 4

| Claim | Type | Status |
|-------|------|--------|
| `T-3004` | task | ✓ pass |
| `T-2556` | task | ✓ pass |
| `T-2818` | task | ✓ pass |
| `T-2483` | task | ✓ pass |

### 2026-09-21T11:07:57Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
- **Reason:** Inception decision: GO
