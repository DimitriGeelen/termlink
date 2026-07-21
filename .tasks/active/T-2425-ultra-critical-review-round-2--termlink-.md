---
id: T-2425
name: "ultra-critical review round 2 — termlink purpose vs field reality"
description: >
  Inception: ultra-critical review round 2 — termlink purpose vs field reality

status: started-work
workflow_type: inception
owner: human
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-07-21T08:48:35Z
last_update: 2026-07-21T08:49:21Z
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

# T-2425: ultra-critical review round 2 — termlink purpose vs field reality

## Problem Statement

Round 1 (T-2419) shipped document fixes (README truth) and verbs (channel.delete),
but fresh field incidents between the two reviews (.121 T-1991 recurrence, .107
debris self-DoS, G-070 detached orphans, G-084 capability blindness) suggest the
substrate's CREATION-TIME DEFAULTS still encode Era-1 assumptions: retention=forever
on auto-created topics, unsupervised process launch, unbounded namespace growth.
Round 2 asks: does delivered behavior now match stated purpose, and which residual
or newly-exposed gaps warrant scoped correction now?

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
     FW_SKIP_DISPOSITION_GATE=1 (env-var, T-1890 producer/consumer parity). -->

- **IW-1: Do the round-1 gap closures (GAP-1..GAP-7) actually hold in the field, or are any of them document-only?**
  confidence: 3
  disposition: answered
  rationale: 5/7 genuinely closed (README truthful at README.md:3-60; T-2421/T-2424 completed; learning at learnings.yaml:2567); live remainders are arc-005 S3 (T-2408) and 4 loudness tasks orphaned by arc-003 closure — docs/reports/T-2425-ultra-critical-review-round-2.md §2.

- **IW-2: Do creation-time defaults still permit unbounded growth despite the shipped primitives (ensure_topic → Forever for generic names, hub create default Forever, debris namespaces unbounded)?**
  confidence: 3
  disposition: answered
  rationale: Yes — ensure_topic (channel.rs:2530) and hub create (hub/channel.rs:359-362) both default Forever for non-pattern names incl. all five debris namespaces; report §3.1/§3.4 → R2-GAP-A (T-2426).

- **IW-3: Should retention enforcement remain sweep-only (T-1155 "explicit, never implicit") given field evidence that the cron dependency itself is the recurring failure (T-1991 recurrence, canary #5 exists solely to watch the cron)?**
  confidence: 2
  disposition: answered
  rationale: No — per-host crons are empirically the least reliable estate component; middle path preserves explicitness via one opt-in env var (TERMLINK_SWEEP_INTERVAL_SECS, default OFF) → R2-GAP-B (T-2427); report §4.2.

- **IW-4: Do test/smoke namespaces (t-*, xhub-*, stress-*, scratch:*, smoke:*) need lifecycle defaults (auto-TTL via the existing Days retention) to prevent debris re-accumulation after the T-2424 sweep?**
  confidence: 3
  disposition: answered
  rationale: Yes — Retention::Days already exists (retention.rs:6-29) and is never auto-selected; T-2424's own allowlist enumerates the namespaces; report §3.2/§3.4 → T-2426.

## Exploration Plan

Three parallel evidence tracks (all complete, see docs/reports/T-2425-ultra-critical-review-round-2.md):
- Track A: closure-reality audit of round-1 gaps GAP-1..GAP-7 (repo sweep)
- Track B: defaults-vs-purpose audit — retention/lifecycle/creation defaults in code
- Track C: fresh field incidents between the two reviews (.121 T-1991 recurrence, .107 debris self-DoS, G-070, G-084)

## Technical Constraints

<!-- What platform, browser, network, or hardware constraints apply?
     For web apps: HTTPS requirements, browser API restrictions, CORS, device support.
     For hardware APIs (mic, camera, GPS, Bluetooth): access requirements, permissions model.
     For infrastructure: network topology, firewall rules, latency bounds.
     Fill this BEFORE building. Discovering constraints after implementation wastes sessions. -->

## Scope Fence

IN: audit of round-1 closures; creation/retention defaults; gap identification + scoped build tasks (T-2426 debris auto-TTL, T-2427 env-gated auto-sweep); arc governance repair for orphaned loudness tasks.
OUT: authz model (T-2422, awaiting operator), managed deploy (T-2423, awaiting operator), arc-005 S3 content itself (T-2408, existing task), any change to durable-topic defaults (channel:learnings etc. stay Forever).

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

**GO.** Round 2 found a coherent, buildable gap class — creation-time defaults and
enforcement — that round 1's verb-and-document work exposed but did not touch:

- **R2-GAP-A (T-2426, BUILD):** the five known test-debris namespaces (t-*/T-*/xhub-*/
  stress-*/scratch:*/smoke:*) are still born `Forever`; auto-pick `Days(7)` at
  creation (CLI ensure_topic + hub-side default) so the T-2424 debris sweep never
  needs repeating. `Retention::Days` already exists — this is a missing default,
  not a missing primitive.
- **R2-GAP-B (T-2427, BUILD):** retention is policy-without-enforcement — the hub
  never sweeps; per-host crons are empirically the least reliable estate component
  (T-1991 twice, canary #5 exists solely to watch the cron). Env-gated hub-side
  periodic sweep (`TERMLINK_SWEEP_INTERVAL_SECS`, default OFF = exact current
  behavior) preserves T-1155 explicitness while collapsing N crons to one unit-file
  line.
- **R2-GAP-C (governance, under this task):** file arc-006 comms-loudness bundling
  the four tasks orphaned by arc-003's closure (T-2385/T-2389/T-2402/T-2224).
- **R2-GAP-D:** advance T-2408 (arc-005 S3) as budget allows.
- **R2-GAP-E (re-surfaced to operator):** T-2422 (authz) and T-2423 (managed
  deploy) decisions remain the highest-leverage unpaid debts.

Both build items are low-risk: behavior changes only for known-debris names or
behind an opt-in env var. Full evidence: docs/reports/T-2425-ultra-critical-review-round-2.md.

**Recommendation:** GO

**Rationale:**

Operator re-issued the standing review directive after round 1 (T-2419) closed. Fresh field evidence since the GO — T-1991 recurrence on .121 (bespoke stale producer, retention never set), 851 debris topics on .107 accumulated with retention=forever defaults, G-070 detached-hub ghosts, G-084 capability-blind version floors — indicates the substrate's DEFAULTS still fight its stated purpose even where primitives exist. Round 2 audits closure-reality of round-1 gaps and identifies default/lifecycle gaps the first pass missed.

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

### 2026-07-21T08:49:21Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
