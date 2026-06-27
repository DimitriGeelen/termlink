---
id: T-2291
name: "Permanent cross-agent comms fix — identity + delivery"
description: >
  Inception: Permanent cross-agent comms fix — identity + delivery

status: started-work
workflow_type: inception
owner: human
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-06-27T08:50:06Z
last_update: 2026-06-27T08:50:54Z
date_finished: null
# revisit_at: YYYY-MM-DD          # T-1451: set on DEFER decisions to enable G-053 daily revisit scan
# revisit_evidence_needed:        # T-1451: one-line description of what evidence makes the revisit actionable
# ── Inception scoring exception (T-2186 Slice 2 / T-2188). See 050-Inceptions.md §Scoring Exception. ──
target_blast_radius: 3            # int 0..9. Anticipated component count of the build work this inception would authorise on GO.
                                  # Substitutes for the absent components: list in the F8 cost formula (040). Required.
                                  # Guide: 0=docs only, 1=single file, 3=small subsystem (S), 5=cross-subsystem (M), 7=multi-arc (L), 9=framework-wide (XL).
voi_score: 0.5                    # float 0..1. Value of Information — expected value of resolving this question,
                                  # independent of build cost. Higher when answer affects many tasks or unblocks a strategic decision. Required.
---

# T-2291: Permanent cross-agent comms fix — identity + delivery

## Problem Statement

Cross-agent communication repeatedly fails. Triggering incident: a `card-redirect`
agent's handoff to the manager was lost. Two **compounding structural root causes**:

1. **Shared host fingerprint (identity layer).** All agents co-resident on host
   `.107` authenticate with the host key → share ONE identity `d1993c2c64…`.
   Manager↔.107-agent DMs collapse onto a single `dm:` topic, discriminated only
   by a human-typed `[manager -> X]` body tag. No per-agent addressability.
   T-1693 (per-agent keys) is the designed-but-unshipped fix.
2. **No inter-hub federation — BY DESIGN (delivery layer).** A post on hub A
   (`.122`) never reaches a reader on hub B (`.107`); there is no mirror that
   "lags" — it never syncs (G-060 / T-2229 / PL-176). Senders don't know which
   hub a peer reads; "sent" ≠ "delivered." The handoff sat at offset 48 on `.122`
   while the manager read `.107`.

Systemic, not a one-off: ring20 T-1259/T-1264/T-1296 (re-filed 3×), G-155
(false-green probe), G-156 (no peer registry). Full RCA + 5 directive-scored
remediation variants in `docs/reports/T-2291-cross-agent-comms-inception.md`.

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

- **IW-1: Is controlled cross-hub relay in scope, or a non-goal?**
  confidence: 1
  disposition: deferred
  rationale: G-060 is a deliberate design choice (T-2229); any federation variant must be opt-in + explicit + loud — passive replication is a known invalid premise. Resolve via A3/A5 findings.
- **IW-2: Identity-fix vs delivery-fix sequencing — which unblocks more, can one ship without the other?**
  confidence: 1
  disposition: deferred
  rationale: Both root causes compound; need A2+A3 RCA to decide whether per-agent identity (V1) or delivery addressing/confirm (V2/V3) is the higher-leverage first move.
- **IW-3: Is single-hub convergence (V5) acceptable given its portability/SPOF cost?**
  confidence: 1
  disposition: deferred
  rationale: Eliminates cross-hub delivery entirely but trades against Portability directive + single point of failure. Score in the directive matrix (A5) before deciding.

## Exploration Plan

5 termlink research agents (parallel where independent), findings written to disk
(`docs/reports/T-2291-*` + bus), concise summaries returned:
- **A1** Cross-agent issue harvester — reach LIVE peers via termlink, build failure catalogue.
- **A2** Identity-layer RCA — shared fingerprint, T-1693 design, T-1427 verified identity, blast radius.
- **A3** Delivery-layer RCA — no-federation (G-060/T-2229), peer-registry gap (G-156), delivery-confirm (T-2286), false-green probe (G-155).
- **A4** Substrate-capability scout — inventory primitives (kv, cv_index, claim, ack-retry, find-idle, presence) → map to root causes.
- **A5** Prior-art + directive-scoring lead — collect prior filings/decisions, finalize rubric, pre-score where evidence exists.

Then synthesize RCA (T5–T6), develop + score 5 variants (T7), present to human (T8).

## Technical Constraints

<!-- What platform, browser, network, or hardware constraints apply?
     For web apps: HTTPS requirements, browser API restrictions, CORS, device support.
     For hardware APIs (mic, camera, GPS, Bluetooth): access requirements, permissions model.
     For infrastructure: network topology, firewall rules, latency bounds.
     Fill this BEFORE building. Discovering constraints after implementation wastes sessions. -->

## Scope Fence

**IN:** RCA of the two root causes; 5 structural remediation variants developed
and scored against the 4 Constitutional Directives + cost; a recommendation
(composite likely) presented for human go/no-go.
**OUT:** Any build artifact (no production code before GO — inception discipline).
Variant selection + GO decision (human sovereignty). Cross-project (AEF) writes
beyond relay unless separately authorized.

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

Two compounding structural root causes are well-documented and recurring (shared host fingerprint on .107 collapsing per-agent DM addressability — T-1693; and no inter-hub federation by design — G-060/T-2229, where posts on hub A never reach readers on hub B so 'sent' != 'delivered'). A permanent fix is clearly warranted, but the SELECTION among 5 structural remediation variants (per-agent keys / peer registry / mandatory delivery-confirm / opt-in cross-hub relay / single-hub convergence) requires directive-scoring evidence this inception will produce via 5 research agents. Deferring the GO + variant choice to the human pending that RCA + scored-variant matrix.

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

### 2026-06-27T08:50:54Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
