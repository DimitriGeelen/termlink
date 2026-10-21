---
id: T-3493
name: "Inject ACL for registered sessions: signed inject + per-session allowlist of
  identity fingerprints (ring20-dashboard request)"
description: >
  ring20-dashboard (msg 0d962a47): a registered --shell session accepts command.inject
  from anyone reaching it; over TCP the only gate is the hub secret, which many peers
  hold. Verified 2026-10-20: open session injects; --token-secret refuses inject at
  observe, but the CLI inject cannot present a token, so it would also lock out the
  session's own sidecar.

status: started-work
workflow_type: inception
owner: human
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-10-20T14:38:15Z
last_update: 2026-10-21T15:44:46Z
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
  - ts: '2026-10-21T15:44:46Z'
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

# T-3493: Inject ACL for registered sessions: signed inject + per-session allowlist of identity fingerprints (ring20-dashboard request)

## Problem Statement

Anyone holding the hub secret (most fleet peers) can inject keystrokes into any open agent session, and nothing records
who did it; the one existing lock (`--token-secret`) cannot be used because `termlink inject`, and so the sidecar's
doorbell, cannot present a token. O-7 (ratified 2026-10-21) names injection explicitly: traced to an identity, accepted
only from an allowed identity. Facts verified 2026-10-21: crates/termlink-session/src/server.rs:250-253 (same-UID gets
Execute without a token secret), crates/termlink-cli/src/cli.rs:364-372 (inject has no token argument),
crates/termlink-hub/src/channel.rs (only channel posts carry and verify identity signatures).

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

- **IW-1: Which option: A defer, B token on the inject path now (C shaped next), C full signed-inject design now, D no-go?**
  confidence: 2
  disposition: deferred
  rationale: brief presented 2026-10-21 (recommend B); operator asked for external review first (docs/reports/T-3493-consult/)
- **IW-2: How does an inject RPC carry a signature the session can verify (today only channel posts are signed, T-1427)?**
  confidence: 0
  disposition: deferred
  rationale: open; input wanted from the external review and from AEF/ring20 (their supervisors inject)
- **IW-3: Where does a session's allowlist of identity fingerprints come from (spawner, hub, config), and how does the operator attach?**
  confidence: 0
  disposition: deferred
  rationale: open
- **IW-4: How does a worker session spawned by another project's supervisor get the right allowlist from its first second without locking that supervisor out?**
  confidence: 0
  disposition: deferred
  rationale: open; touches the shared worker contract (AEF T-4439)

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

**Recommendation:** DEFER

**Rationale:** Shaping in progress: the risk is verified (open sessions accept inject from any hub-secret holder), but the design has open questions not yet answered: how an inject request carries a signature the session can check (RPC calls are not identity-signed today, only channel posts are, T-1427), how the session learns its allowlist and the operator's attach, and how the sidecar injector authenticates. Interim mitigation exists and is ring20's call: --token-secret blocks foreign inject but today also their own sidecar.

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

### 2026-10-21T15:44:46Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
- **Change:** horizon: next → now (auto-sync)
