---
id: T-2229
name: "Triage ring20 cross-hub federation + heartbeat-freeze RCA (framework:pickup)"
description: >
  Inception: Triage ring20 cross-hub federation + heartbeat-freeze RCA (framework:pickup)

status: work-completed
workflow_type: inception
owner: human
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-06-21T09:37:18Z
last_update: 2026-07-02T15:40:51Z
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

# T-2229: Triage ring20 cross-hub federation + heartbeat-freeze RCA (framework:pickup)

## Problem Statement

<!-- What problem are we exploring? For whom? Why now? -->

ring20-management's agent filed a high-severity RCA to the `framework:pickup`
hub topic (~offset 43); it sat ~27h unprocessed because termlink has no
consumer of `framework:pickup` (G-063) AND the reported heartbeat-freeze froze
the .107 agent sessions that would have read it (self-reinforcing). A live
termlink agent (this session) picked it up 2026-06-21.

**Fault 1 (ring20's framing): cross-hub federation broken** — registry +
channel (agent-chat-arc, DM topics) no longer replicate across hubs.

**Fault 2: heartbeat freezes on hub restart** — on restart the hub reloads the
persisted session with its ORIGINAL registration heartbeat; `termlink register`
never re-handshakes with the new hub instance, so presence freezes at
registration time. This is the "frozen husks" symptom.

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

- **IW-1: Is cross-hub federation a regression, or working-as-designed (no primitive)?**
  confidence: 3
  disposition: answered
  rationale: PL-176 + G-060 + CLAUDE.md §"Channel Topic Semantics" all state TermLink has NO inter-hub federation primitive — never existed. Client-driven cross-post (`channel post --hub`) is the intended pattern; ring20's "beacon" IS that pattern, not a band-aid. Fault 1 is a consumer-expectation mismatch, not a termlink regression.

- **IW-2: Is the heartbeat-freeze-on-hub-restart (fault 2) a real, bounded termlink bug worth a fix task?**
  confidence: 3
  disposition: answered
  rationale: CONFIRMED and broader than diagnosed — `termlink register` never advanced heartbeat_at at all (touch_heartbeat had zero production callers); the freeze was permanent, not restart-specific. Fixed in T-2230 (periodic heartbeat task + strict-advance regression test; cargo check + 24 session tests pass). PL-221 captured.

- **IW-3: Should termlink expose an operator-facing federation enable/status verb, or is documenting client-driven cross-post sufficient?**
  confidence: 1
  disposition: deferred
  rationale: docs/operations/channel-topic-semantics.md already documents the no-federation design (G-060). Whether to add a discoverability verb so consumers stop mis-filing "federation broken" is an operator call.

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

**IN:** Confirm/dispose each fault; correct ring20's federation framing;
spawn bounded follow-up tasks (repro+fix for fault 2; optional discoverability
verb); reply to ring20.

**OUT:** Building a federation primitive (none exists by design). Implementing
the fault-2 fix under this inception ID (spawn a separate build task on GO).

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

Confirmed high-severity, durable filing on framework:pickup (offset ~43) sitting ~27h unprocessed because termlink has no consumer for that topic (G-063) AND the reported fault_2 heartbeat-freeze itself froze the .107 agent sessions that would have picked it up — self-reinforcing. Two distinct faults: (1) cross-hub federation/registry+channel replication broken; (2) termlink register heartbeat freezes on hub restart (never re-handshakes). ring20 has a beacon band-aid in place. Triage is GO to confirm each fault, decide federation-off-vs-regressed, and spawn the right bounded build/bug tasks — but the federation fix itself is NOT yet authorized (needs operator GO per pickup=proposal rule).

**Evidence:**

<!-- Add evidence bullets as exploration progresses (file paths,
     commit hashes, test results). The filing-time recommendation
     can be revised before fw inception decide. -->

- Fault 1 = working-as-designed: PL-176 / G-060 / CLAUDE.md §Channel Topic Semantics — no federation primitive exists; client-driven cross-post is the intended pattern. No code change.
- Fault 2 = real bug, FIXED: T-2230 (commit b182a803) — periodic self-heartbeat in cmd_register; root cause was touch_heartbeat had zero production callers (registration.rs:325).
- G-063 (root cause of the 27h drop) = closed: T-2231 framework:pickup freshness canary (empty-log=healthy, /canaries HEALTHY).
- Replied to ring20 hub agent-chat-arc @ .122 (offset 2299).
- Fault 3 (operator-facing federation status verb) = open operator decision (IW-3 deferred).

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

**Rationale**: triage complete; fault1=WAD, fault2 fixed (T-2230), G-063 closed 
  (T-2231)

**Date**: 2026-06-21T10:35:42Z

## Updates

<!-- Auto-populated by git mining at task completion.
     Manual entries optional during execution. -->

### 2026-06-21T09:37:47Z — status-update [task-update-agent]
- **Change:** status: captured → started-work

### 2026-06-21T10:26:50Z — inception-decision [inception-workflow]
- **Action:** Recorded inception decision
- **Decision:** GO
- **Rationale:** triage complete; fault1=WAD, fault2 fixed (T-2230), G-063 closed (T-2231)

### 2026-06-21T10:35:42Z — inception-decision [inception-workflow]
- **Action:** Recorded inception decision
- **Decision:** GO
- **Rationale:** triage complete; fault1=WAD, fault2 fixed (T-2230), G-063 closed 
  (T-2231)
