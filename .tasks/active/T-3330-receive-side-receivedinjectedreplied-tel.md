---
id: T-3330
name: "Receive side: received/injected/replied telemetry as API calls, and ship all
  sidecars inside the termlink binary"
description: >
  Inception: Receive side: received/injected/replied telemetry as API calls, and ship
  all sidecars inside the termlink binary

status: started-work
workflow_type: inception
owner: human
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-10-02T23:18:52Z
last_update: 2026-10-02T23:27:53Z
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
  - ts: '2026-10-02T23:27:54Z'
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

# T-3330: Receive side: received/injected/replied telemetry as API calls, and ship all sidecars inside the termlink binary

## Problem Statement

1. **The receive side is invisible and partly missing (found 2026-10-03, T-3325).** AEF waited about a day (@62–@131) and 055's consult went unseen. The cause: the claude-termlink sidecar receipted mail as "delivered", but nothing injected it. Per the research artifact (docs/reports/T-3330-receive-side.md):
   1a. "received" is a per-topic watermark receipt (`stage=delivered`, `up_to`), with no per-message id or timestamp field;
   1b. "injected" is never emitted: the wake consumers have no action, and notify-injector is scheduled by nothing;
   1c. "replied" is not emitted at all;
   1d. the binary drops `stage`, keeping only the latest `{sender_id, up_to, ts}` per sender.
2. **Operator design (2026-10-03).** Each stage is a timestamped API call back to the sender: RECEIVED as soon as the message is stored, INJECTED when it is put into the session, REPLIED when the answer goes back. All timestamps are stored.
3. **Operator requirement (2026-10-03): a communication-telemetry STANDARD for all agents, ours and every vendored agent (AEF, 055, 832, ring20, …).**
   3a. Measure every step of sidecar and inbox delivery: how long each takes, and the delay between steps.
   3b. We can PULL this telemetry from other agents.
   3c. Each agent also OFFERS it regularly (about daily).
   3d. We work with the data and reflect on what working with it means.
4. **Operator question (2026-10-03): ship every sidecar inside the TermLink package**, so a deployment carries them. Today releases ship only the binary, and the ~13 sidecar scripts run only from this checkout.

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

- **IW-1: Who owns the receive-side state machine and its API calls (RECEIVED / INJECTED / REPLIED with timestamps): adopt AEF's receiver (T-3693), extend TermLink's own sidecar + injector, or move the receive side into the termlink binary?**
  confidence: 3
  disposition: answered
  rationale: operator ruling 2026-10-03, C amended (see ## Decisions): the operator's sidecar protocol carries the stages as API calls between sidecars, each a timestamped event; each sidecar copies its events to the hub; daily digest; urgent-only alarms; observability DB + learning later as their own inception. External review 4/4 C (T-3330-consult/).
- **IW-2: What is the cross-agent communication-telemetry standard: the per-message event record (states, timestamps, ids), where each agent stores it, how a peer PULLS it, and how each agent OFFERS a daily digest? Who defines it (TermLink, AEF, jointly)?**
  confidence: 0
  disposition:
  rationale:
- **IW-3: How do sidecars ship with every deployment: subcommands in the binary, or scripts bundled into the release?**
  confidence: 1
  disposition:
  rationale:
- **IW-4: Interim: until IW-1 is built, how does claude-termlink get a working wake path (its consumer is held back by the T-3065 receipt-identity mismatch)?**
  confidence: 1
  disposition:
  rationale:

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

**Rationale:**

Evidence gathering not done yet. Known so far (T-3325, 2026-10-03): the received stage exists only as a stage=delivered topic receipt; injected does not exist (wake consumer has no action, notify-injector scheduled by nothing, claude-termlink has no consumer); replied is a plain post. AEF has built a per-agent receiver API with SENT/RECEIVED/HANDED_OVER/REPLIED (T-3693). Releases ship only the termlink binary; ~13 sidecar scripts run from this checkout only. Recommendation follows the evidence in the research artifact.

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

### 2026-10-03 — IW-1: who owns the receive-side stages and the telemetry record (operator ruling)
- **Chose:** C amended, matched to the operator's protocol design (research artifact § "Operator's protocol design"):
  1. **Protocol:** sidecar-to-sidecar API calls carry RECEIVED, STORED, INJECTED and ANSWER READY. Push is primary; pull is the fallback on the standard polling ladder (15 s … 1 year, each rung twice; sent to AEF, pickup @309 / inbox @180).
  2. **Telemetry:** every such call is a timestamped event. Each sidecar records its own and copies them to the hub in the background (outbox, never in the message path). The hub keeps them for a retention window; any agent can pull; each agent posts a daily digest.
  3. **Alarms:** immediate alarms only for messages flagged urgent; how they surface is still to define. Everything else accumulates and escalates when it piles up, like audit warnings; how and where is still to define.
  4. **The observability database** and the learning from the data come later, as their own design inception (linked to T-3319). The hub stays the home for now.
- **Why:** it meets the operator's requirements (every step timestamped and visible to the sender; a standard for all agents; pullable plus a daily offer; reflection) while keeping the hub inside its charter (retention-bounded, not a second bus: cross-host sidecar calls are AEF's T-3688; TermLink supplies discovery and the telemetry record). External review chose C 4/4.
- **Rejected:** A (per-project ledgers cannot be pulled by other agents); B (discards AEF's working receiver; a large port before anything improves); D (the silent stall continues). gemma4's separate observability store is not rejected but deferred, by the operator, to the learning inception.
- **Left open:** IW-2 (event record, pull verb, digest), IW-3 (how sidecars ship; whether TermLink's own receive scripts are replaced by AEF's sidecar), IW-4 (interim wake path); how urgent alarms and escalations surface; the operator's "hub agent" idea (proposed as its own inception); sending our design to AEF for overlap feedback (operator, 2026-10-03).

## Decision

<!-- Filled at completion via: fw inception decide T-XXX go|no-go --rationale "..." -->

## Updates

<!-- Auto-populated by git mining at task completion.
     Manual entries optional during execution. -->

### 2026-10-02T23:27:53Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
