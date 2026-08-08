---
id: T-2549
name: "Charter non-goal #4: termlink dispatch orchestration-in-substrate subtract-vs-keep"
description: >
  Off-charter subtract candidate from T-2468: termlink dispatch (dispatch.rs 1138 LOC + MCP termlink_dispatch/_status) self-describes as multi-worker orchestration 'replacing dispatch scripts' — non-goal #4 (substrate stays mechanism not policy). Distinct from orchestrator.route (T-2540). Composes charter-legal primitives, so counter-case is 'convenience wrapper'.

status: started-work
workflow_type: inception
owner: human
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-08-08T19:37:48Z
last_update: 2026-08-08T19:39:15Z
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

# T-2549: Charter non-goal #4: termlink dispatch orchestration-in-substrate subtract-vs-keep

## Problem Statement

Charter non-goal #4 (docs/CHARTER.md): *"Not a workflow or orchestration engine.
TermLink provides the primitives... The substrate stays mechanism, not policy."*
The `termlink dispatch` verb (crates/termlink-cli/src/commands/dispatch.rs, 1138
LOC) + its MCP tools (`termlink_dispatch`, `termlink_dispatch_status`) is, by its
own module docstring, *"atomic spawn+tag+collect for multi-worker orchestration...
Provides a structural guarantee that collect is always wired, replacing manual
dispatch scripts."* This is a workflow/orchestration engine baked into the
substrate — the exact thing non-goal #4 reserves for the AEF layer. It is a
distinct surface from `orchestrator.route` (filed T-2540). The honest
counter-case: `dispatch` only *composes* charter-legal session primitives
(spawn / tag / collect), so it can be read as a convenience wrapper rather than
policy. The subtract-vs-keep call is a human sovereignty product-identity
decision.

## Assumptions

<!-- Key assumptions to test. Register with: fw assumption add "Statement" --task T-XXX -->

- A-1: `dispatch` implements orchestration *policy* (fan-out N workers + wire
  collect), not a mechanism-only primitive. (verify: dispatch.rs docstring +
  DispatchOpts.) — CONFIRMED: docstring self-describes as orchestration.
- A-2: MCP `termlink_dispatch` / `_status` are LIVE (not deprecated). — CONFIRMED.
- A-3: Near-zero first-party callers (at most 1 demo/eval script).
- A-4 (UNVERIFIABLE here): no EXTERNAL peer/AEF process calls dispatch (T-559).
  This is the removal GATE.

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

- **IW-1: Are there any EXTERNAL (non-first-party) callers of `termlink dispatch`
  or its MCP tools across the fleet or the AEF layer?**
  confidence: 1
  disposition: deferred
  rationale: First-party callers near-zero (at most 1 eval script). External
  callers cannot be confirmed from /opt/termlink (T-559). Removal GATE.

- **IW-2: Is `dispatch` policy (subtract) or a thin convenience wrapper over
  charter-legal primitives (keep)?**
  confidence: 2
  disposition: deferred
  rationale: The module docstring self-describes as "orchestration... replacing
  dispatch scripts" (policy read). But it only composes spawn/tag/collect (all
  charter-legal), so a convenience-wrapper read is defensible. This is the crux
  of the human decision — more nuanced than the analytics family (T-2548) or
  orchestrator.route (T-2540), both of which carry bespoke policy logic dispatch
  does not.

## Exploration Plan

<!-- How will we validate assumptions? Spikes, prototypes, research? Time-box each. -->

1. Verify A-1/A-2: read dispatch.rs docstring + confirm MCP tools live. (DONE.)
2. Verify A-3: grep scripts/ + .claude/commands/ for dispatch callers.
3. Clear IW-1 (GATE): cross-project consumer check — not doable here (T-559).
4. Human decides IW-2 (subtract / keep-as-wrapper / grandfather).

## Technical Constraints

<!-- What platform, browser, network, or hardware constraints apply?
     For web apps: HTTPS requirements, browser API restrictions, CORS, device support.
     For hardware APIs (mic, camera, GPS, Bluetooth): access requirements, permissions model.
     For infrastructure: network topology, firewall rules, latency bounds.
     Fill this BEFORE building. Discovering constraints after implementation wastes sessions. -->

## Scope Fence

**IN scope:** confirm dispatch is orchestration policy (not mechanism), confirm
near-zero first-party callers, frame subtract-vs-keep with the wrapper counter-case.
**OUT of scope:** the actual removal (GO build task), the cross-project consumer
check (IW-1 gate), any charter amendment. The underlying session primitives
(spawn/tag/collect/session discover/channel claim) are charter-legal and stay
regardless — only the `dispatch` composition verb is in question.

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

**GO (subtract) if:**
- Non-goal #4 is read strictly (self-described "orchestration" belongs in AEF),
  AND IW-1 clears (no external callers). Then remove the verb + 2 MCP tools +
  dispatch.rs; document the spawn/tag/collect recipe in AEF for anyone who wants
  the composition.

**NO-GO (keep-as-wrapper) if:**
- The human judges dispatch a thin, charter-legal convenience over existing
  primitives worth keeping for usability (it wires collect so callers can't
  forget) → keep, optionally add a doc note that it is composition sugar, not a
  new primitive.

**INTERMEDIATE (deprecate-then-remove) if:**
- IW-1 cannot be cleared quickly → deprecate (T-1166 pattern), soak, remove.

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

**Recommendation:** GO (subtract), gated on IW-1 — but with LOWER confidence than
T-2540/T-2548; the wrapper counter-case (IW-2) is genuine and the human may
reasonably keep it.

**Rationale:** Verified in code (2026-08-08): dispatch.rs (1138 LOC) + MCP
`termlink_dispatch`/`_status` are LIVE and self-describe as "multi-worker
orchestration... replacing manual dispatch scripts" — a workflow/orchestration
engine in the substrate, which non-goal #4 names verbatim as a non-goal. This is
a distinct second instance from `orchestrator.route` (T-2540). Strict charter
reading → subtract. HOWEVER, unlike orchestrator.route (bespoke adaptive routing
policy) and the analytics family (bespoke metrics), dispatch only *composes*
charter-legal primitives (spawn/tag/collect), so it is the most defensible of the
three off-charter surfaces to KEEP as usability sugar. Recommendation is GO-to-
subtract to hold the line, but flagged as the human's most-legitimately-debatable
of the subtract set.

**Evidence:**
- LIVE: `termlink_dispatch` / `termlink_dispatch_status` have no deprecation
  marker in their MCP descriptions.
- Self-described orchestration: dispatch.rs:1-5 docstring.
- Near-zero first-party callers (at most 1 eval script).
- Distinct from T-2540 (orchestrator.route) and T-2548 (analytics family).
- Counter-case: composes only charter-legal session primitives → wrapper read.


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
