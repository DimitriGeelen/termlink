---
id: T-2249
name: "R1 rescope register-as-presence-producer design decision"
description: >
  Inception: R1 rescope register-as-presence-producer design decision

status: work-completed
workflow_type: inception
owner: human
horizon: null
arc_id: arc-substrate-fitness
tags: [arc:arc-substrate-fitness]
components: []
related_tasks: [T-2242, T-2245, T-2107, T-2239]
created: 2026-06-23T07:39:06Z
last_update: 2026-06-25T06:31:50Z
date_finished: 2026-06-25T06:31:50Z
# revisit_at: YYYY-MM-DD          # T-1451: set on DEFER decisions to enable G-053 daily revisit scan
# revisit_evidence_needed:        # T-1451: one-line description of what evidence makes the revisit actionable
# ── Inception scoring exception (T-2186 Slice 2 / T-2188). See 050-Inceptions.md §Scoring Exception. ──
target_blast_radius: 3            # int 0..9. Anticipated component count of the build work this inception would authorise on GO.
                                  # Substitutes for the absent components: list in the F8 cost formula (040). Required.
                                  # Guide: 0=docs only, 1=single file, 3=small subsystem (S), 5=cross-subsystem (M), 7=multi-arc (L), 9=framework-wide (XL).
voi_score: 0.5                    # float 0..1. Value of Information — expected value of resolving this question,
                                  # independent of build cost. Higher when answer affects many tasks or unblocks a strategic decision. Required.
---

# T-2249: R1 rescope register-as-presence-producer design decision

## Problem Statement

The arc-002 ingestion plan (FLAG 1) re-scoped **R1** to "extend `cv_key` emission
to the `register`/PTY presence producer path" as a **minor build**. Source
verification reverses that premise: `termlink register` **never posts to
`agent-presence`** — its heartbeat is a local JSON `touch_heartbeat` only
(`session.rs:294-307`, `registration.rs:325`). `agent-presence` is fed solely by
the opt-in `/be-reachable` producer (`listener-heartbeat.sh`, cv_key-wired
T-2107) and the MCP `listener_heartbeat` tool. So R1-as-described does not exist
as a minor edit; the genuine change would give `register` a NEW presence-
publishing role — a substrate-semantics decision (presence becomes "every
registered session" vs today's "agents that opted in"). The `cv_index count=0`
the discovery saw was **operational** (`/be-reachable` not running), not a code
gap. Full analysis: `docs/reports/T-2249-r1-rescope-register-presence-producer.md`.

## Assumptions

- The plan's FLAG 1 assumed `register` already posts presence — **contradicted**
  by source (the repo wins per plan §1 / handoff §7).
- Presence is intentionally opt-in today (a worker advertises via `/be-reachable`);
  this is a design property, not an accident.

## Open Questions

- **IW-1: Should `termlink register` gain a new role as an `agent-presence`
  producer (changing presence semantics from "opted-in agents" to "all registered
  sessions")?**
  confidence: 2
  disposition: deferred
  rationale: Sovereign design decision — agent recommends NO-GO on always-on; human decides via `fw task review T-2249`. Evidence: report §4-§6.
- **IW-2: Was the discovery's `cv_index count=0` a register code gap or
  operational?**
  confidence: 3
  disposition: answered
  rationale: Operational — `/be-reachable` producer not running (`be-reachable.log`="Terminated", no `listener-heartbeat` in ps); register never fed agent-presence by design. Report §3.

## Exploration Plan

Read-only source verification (done): confirm register's heartbeat path
(`session.rs`, `registration.rs`, `endpoint.rs`), the producers that DO feed
`agent-presence` (`listener-heartbeat.sh`, MCP tool), and the bus-side
read/retention references. No spikes/prototypes — the question is design intent,
resolved by reading code + surfacing options. Time-box: complete.

## Technical Constraints

<!-- What platform, browser, network, or hardware constraints apply?
     For web apps: HTTPS requirements, browser API restrictions, CORS, device support.
     For hardware APIs (mic, camera, GPS, Bluetooth): access requirements, permissions model.
     For infrastructure: network topology, firewall rules, latency bounds.
     Fill this BEFORE building. Discovering constraints after implementation wastes sessions. -->

## Scope Fence

**IN:** verify R1's premise against source; surface the design question (should
`register` publish presence?) with options + an agent recommendation for the
human. **OUT:** any code change to `register`/heartbeat (no producer added); any
GO/NO-GO decision (Sovereign — human only); live-host operational steps (R7).

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
- The human wants `register` sessions discoverable in `agent-presence` AND
  accepts the semantic shift (or scopes it to an opt-in `--publish-presence`
  flag, default OFF — option 2).

**NO-GO if:**
- Presence should stay opt-in (the `/be-reachable` producer owns it) and the
  `count=0` is correctly attributed to operations (run `/be-reachable`) — option 1,
  the agent recommendation. Re-scopes R1 to operational, folding into R7.

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

**Recommendation:** NO-GO

**Rationale:**

Source verification (session.rs:294-307 + registration.rs:325) reverses the plan's FLAG-1 R1 framing: termlink register NEVER posts to agent-presence — its heartbeat is a local JSON touch_heartbeat only. agent-presence is fed solely by the opt-in /be-reachable producer (listener-heartbeat.sh, cv_key-wired T-2107) and the MCP listener_heartbeat tool. So R1 'add cv_key to the register presence post' does not exist as a minor edit; the genuine change is giving register a NEW presence-publishing role, which alters substrate semantics (presence would become 'every registered session' vs today's 'agents that opted in'). The cv_index count=0 the discovery saw was OPERATIONAL (/be-reachable not running), not a register code gap. Recommend NO-GO on auto-publishing from register; keep presence opt-in (dedicated producer owns it). Human may instead choose an explicit opt-in flag variant. Re-scopes R1 from minor-build to operational (run /be-reachable) — folds into R7.

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

**Decision**: NO-GO

**Rationale**: Recommendation: NO-GO

Rationale:

Source verification (session.rs:294-307 + registration.rs:325) reverses the plan's FLAG-1 R1 framing: termlink register NEVER posts to agent-presence — its heartbeat is a local JSON touch_heartbeat only. agent-presence is fed solely by the opt-in /be-reachable producer (listener-heartbeat.sh, cv_key-wired T-2107) and the MCP listener_heartbeat tool. So R1 'add cv_key to the register presence post' does not exist as a minor edit; the genuine change is giving register a NEW presence-publishing role, which alters substrate semantics (presence would become 'every registered session' vs today's 'agents that opted in'). The cv_index count=0 the discovery saw was OPERATIONAL (/be-reachable not running), not a register code gap. Recommend NO-GO on auto-publishing from register; keep presence opt-in (dedicated producer owns it). Human may instead choose an explicit opt-in flag variant. Re-scopes R1 from minor-build to operational (run /be-reachable) — folds into R7.

Evidence:

**Date**: 2026-06-25T06:31:50Z

## Updates

<!-- Auto-populated by git mining at task completion.
     Manual entries optional during execution. -->

### 2026-06-23T07:40:03Z — status-update [task-update-agent]
- **Change:** status: captured → started-work

### 2026-06-25T06:31:50Z — inception-decision [inception-workflow]
- **Action:** Recorded inception decision
- **Decision:** NO-GO
- **Rationale:** Recommendation: NO-GO

Rationale:

Source verification (session.rs:294-307 + registration.rs:325) reverses the plan's FLAG-1 R1 framing: termlink register NEVER posts to agent-presence — its heartbeat is a local JSON touch_heartbeat only. agent-presence is fed solely by the opt-in /be-reachable producer (listener-heartbeat.sh, cv_key-wired T-2107) and the MCP listener_heartbeat tool. So R1 'add cv_key to the register presence post' does not exist as a minor edit; the genuine change is giving register a NEW presence-publishing role, which alters substrate semantics (presence would become 'every registered session' vs today's 'agents that opted in'). The cv_index count=0 the discovery saw was OPERATIONAL (/be-reachable not running), not a register code gap. Recommend NO-GO on auto-publishing from register; keep presence opt-in (dedicated producer owns it). Human may instead choose an explicit opt-in flag variant. Re-scopes R1 from minor-build to operational (run /be-reachable) — folds into R7.

Evidence:

### 2026-06-25T06:31:50Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
- **Reason:** Inception decision: NO-GO
