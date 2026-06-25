---
id: T-2285
name: "Substrate ack-with-retry — enforce delivery receipts for parallel-exec harness (§9 hard-dep #5)"
description: >
  Inception: Substrate ack-with-retry — enforce delivery receipts for parallel-exec harness (§9 hard-dep #5)

status: captured
workflow_type: inception
owner: human
horizon: now
tags: []
components: []
related_tasks: [T-2018, T-2051, T-1485, T-2049]
tags_note: arc-parallel-substrate, collaboration-seam, harness, ack-retry
created: 2026-06-25T14:16:59Z
last_update: 2026-06-25T14:16:59Z
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

# T-2285: Substrate ack-with-retry — enforce delivery receipts for parallel-exec harness (§9 hard-dep #5)

## Problem Statement

The AEF parallel-execution harness (AEF ADR §5) depends on a **sender-side
ack-with-retry** primitive from the substrate: when a worker reports
"complete" (or a collision warning is sent) and the recipient's listener is
dead, the *sender* must detect the missing acknowledgment and retry — one of
two independent detectors of the one dangerous failure mode (receiver detects
via stale heartbeat timestamp; sender detects via missing ack). Today
**TermLink receipts are advisory** — `channel.receipts` records who acked but
nothing enforces delivery or retries on a missing ack. Substrate ADR §6 #5
explicitly scopes this as "the substrate half of the sender-side retry the AEF
layer relies on," but only the *receiver-side* durability (offline outbound
queue, T-2051) and a *synchronous* `--ack-required` send (T-1485) have shipped.
The sender-detects-dead-recipient-and-retries semantic is uncaptured.

**For whom:** the orchestrator + workers of the parallel-execution arc (the
governance plane cannot silently drop a completion/ledger message). **Why
now:** AEF un-parked its harness inceptions T-2323/T-2324 this session and
flagged this gap back to the substrate via pickup P-051; it is the one open
**hard** dependency (§9) for the arc, so leaving it uncontracted will surface
as rising consultation volume (the §9 "re-contract, not grind" smell).

## Assumptions

<!-- Register with: fw assumption add "Statement" --task T-2285 -->
- The harness needs *exactly-once* delivery semantics, so any retry must
  compose with the T-2049 idempotency dedupe (retried sends must not
  double-apply).
- "Advisory receipts + a sender retry loop" may be sufficient; a hub-side
  redelivery guarantee may be unnecessary. To be tested.

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

- **IW-1: Enforced vs. advisory+wrapper — does the substrate promote receipts to
  enforced delivery (sender blocks/retries until ack or dead-letter), or stay
  advisory with an opt-in sender-side retry loop built on existing
  `channel.receipts`?**
  confidence: 1
  disposition: deferred
  rationale: awaiting human spike dialogue; lean advisory+wrapper (smaller blast
  radius, preserves the strict-star/append-log invariants) but unverified.

- **IW-2: At what layer does retry live — hub-side redelivery, client-side
  outbound-queue replay (extend T-2051), or a sender poll-loop on the receipt
  topic?**
  confidence: 1
  disposition: deferred
  rationale: T-2051 already owns client-side durable replay; extending it is the
  cheapest path, but the dead-recipient-detection trigger (no ack within window)
  is new. Unverified.

- **IW-3: How does retry compose with T-2049 idempotency dedupe so a retried
  send is exactly-once, and what is the retry policy (backoff, max attempts,
  dead-letter sink)?**
  confidence: 1
  disposition: deferred
  rationale: dedupe LRU is keyed on `(sender_id, client_msg_id)`; a retry must
  reuse the same id. Policy numbers (window, attempts) co-discovered with AEF
  §6 heartbeat tick/threshold. Unverified.

## Exploration Plan

1. **Read the current receipt path** (`channel.receipts` / `channel.ack` in
   `crates/termlink-*`) — confirm what "advisory" means in code and where an ack
   lands. Time-box: 30 min.
2. **Map against T-2051 outbound queue + T-2049 dedupe** — can the existing
   client-side replay + idempotency be the retry substrate, with only a
   missing-ack trigger added? Time-box: 30 min.
3. **Sketch the two candidate designs** (enforced hub-side redelivery vs.
   advisory+sender-retry-loop) with blast radius + invariant impact for each.
   Time-box: 45 min.
4. **Co-discover retry policy numbers with AEF** (§6 heartbeat tick/threshold) —
   soft dependency, consult via the pickup channel. Time-box: dialogue.

## Technical Constraints

- **Invariant: strict star + append-log durability/ordering must be preserved**
  (substrate ADR §10). Retry cannot introduce a peer-to-peer surface.
- **Exactly-once:** any retry MUST reuse the original `client_msg_id` so the
  T-2049 hub-side dedupe LRU absorbs the duplicate (no double-apply).
- **Same-host vs cross-host transports differ** (UDS vs authenticated TCP) —
  retry semantics must hold identically across both.
- **Receipts are advisory today** — promoting them is a protocol-level change
  with backward-compat obligations for older clients (must degrade, not break).

## Scope Fence

**IN:** the design decision (enforced vs. advisory+wrapper), the layer (hub vs.
client-queue vs. sender-loop), the retry policy shape, and composition with
T-2049/T-2051. Produces a GO/NO-GO + a bounded build-task list on GO.
**OUT:** writing the implementation (separate build tasks post-GO); the AEF-side
harness poll loop (AEF owns, T-2323); the conservative→optimistic flip
(orthogonal, gated separately); filesystem-write observation (substrate gap #4,
unrelated).

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
- One of the two candidate designs (enforced vs. advisory+sender-retry-loop) is
  selected with a bounded, reversible build-task list
- The selected design composes cleanly with T-2049 dedupe (exactly-once) and
  T-2051 outbound queue without violating the strict-star / append-log invariants
- Retry policy shape (trigger, backoff, max attempts, dead-letter) is specified
  well enough for AEF to build its §5 harness against the contract

**NO-GO if:**
- Advisory receipts + AEF-side sender retry already suffice with no substrate
  change (the gap is an AEF-layer concern, not a substrate one)
- Enforced delivery would require dissolving the strict star or unbounded
  protocol surface (re-contract the §9 dependency instead)

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

AEF ADR §5 names ack-with-retry as the one open substrate dependency ('TermLink receipts advisory today'); substrate ADR §6 #5 calls it 'the substrate half of the sender-side retry the AEF layer relies on'. AEF agent flagged it back via pickup P-051 (2026-06-25). No existing substrate task captures the sender-detects-missing-ack-and-retries semantic — T-1485 was synchronous send-ack (done), T-2051 shipped receiver-side outbound queue. Recommend scoping the enforced-receipt + retry design as the producer-side closure of the §9 seam.

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
