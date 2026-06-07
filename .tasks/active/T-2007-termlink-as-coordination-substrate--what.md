---
id: T-2007
name: "TermLink-as-coordination-substrate — what exists today (research, not design)"
description: >
  Inception: TermLink-as-coordination-substrate — what exists today (research, not design)

status: captured
workflow_type: inception
owner: human
horizon: later
tags: []
components: []
related_tasks: []
created: 2026-06-05T19:16:55Z
last_update: 2026-06-06T12:32:28Z
date_finished: null
---

# T-2007: TermLink-as-coordination-substrate — what exists today (research, not design)

## Problem Statement

The human is evaluating whether TermLink, as it exists today at HEAD `4b69db68`,
can serve as the **coordination substrate** for parallel multi-agent task
execution across a distributed homelab (multiple hosts, multiple agents per host,
communicating over IP including loopback for same-host agents). The intended
model is hub-and-spoke: isolated code work on per-host checkouts, all
shared/governance state (task ledger, audit logs, arc YAML, episodic memory)
serialized through one central hub; spokes never write it directly. Safety from
**disjoint write-sets between concurrently-running tasks**, not from merging.

The agent's job is to **report what is, not design**. Where TermLink cannot do
something today, say so plainly — "it doesn't do this" is a valuable finding,
not a failure. The design decision belongs to the human.

## Assumptions

None registered — pure research. Hypotheses are explicitly avoided per the
"report what is" framing.

## Exploration Plan

Read the actual TermLink source across 10 specific question areas:
1. Messaging shape (point-to-point vs broadcast vs pubsub vs blackboard)
2. Topology (star-enforced vs peer-to-peer possible)
3. Transport uniformity (same-host vs cross-host)
4. Serialization primitive (queue / lock / ordered log)
5. Work distribution (push vs pull, idle/busy tracking)
6. Live state / announcements + filesystem observation
7. Git integration in the runtime
8. Failure modes (heartbeat, durability, reconnect)
9. Agent lifecycle (spawn surface, working dir, env)
10. Scale & limits (connections, message size, throughput)

Five parallel Explore subagents covered the 10 questions; findings synthesized
into the report at `docs/reports/T-2007-termlink-as-coordination-substrate-findings.md`.

## Technical Constraints

- Source-of-truth is the Rust workspace at `/opt/termlink/crates/` plus shell
  wrappers in `scripts/`. CLAUDE.md prose is context but not authoritative.
- Every claim must cite file:line. Absences must be stated plainly, not glossed.
- The agent does not propose a design or extend TermLink; that is the human's
  call after reading the findings.

## Scope Fence

**IN scope:**
- Reading TermLink source code
- Producing a findings report with file:line citations
- Enumerating gaps relative to the hub-and-spoke / disjoint-write-set model

**OUT of scope:**
- Recommending whether to extend TermLink or change the design
- Filing build tasks against any gap
- Prototyping or spiking any of the missing capabilities

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

**GO if:** N/A — this task does not propose a build.

**NO-GO if:** N/A — there is nothing to reject.

**DEFER if:** Research deliverable produced; the architectural choice (extend
TermLink, change the design, or accept the gaps) is the human's decision and
this inception's job is to brief that decision, not make it. — **THIS APPLIES.**

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

The task scope was explicit: "REPORT WHAT IS, not design or recommend an
architecture — the design decisions belong to the human." The findings report
delivers the evidence the human needs to make that decision. DEFER is therefore
not a holding pattern but the **terminal stance** for this inception — the
agent's job ended at producing the briefing.

**Evidence:**

- `docs/reports/T-2007-termlink-as-coordination-substrate-findings.md` — full
  10-question report with file:line citations and a 10-item gap enumeration.
- TermLink supports point-to-point, broadcast, pub-sub, and blackboard-style
  shared state — but **only** through append-only ordered logs; no
  exclusive-delivery / claim / lock / lease primitives exist
  (`crates/termlink-bus/src/log.rs:27-65`).
- The hub does not track agent idle/busy state; LIVE/STALE/OFFLINE is
  client-side `jq` over the `agent-presence` topic
  (`scripts/agent-listeners.sh:236-238`).
- Same-host and cross-host clients can use identical IP transport (loopback
  TCP), but the default same-host path is auth-bypassed Unix socket
  (`crates/termlink-hub/src/server.rs:500-515`).
- Channel logs + inbox spool survive hub restart; presence tracker + circuit
  breaker do not (`crates/termlink-bus/src/lib.rs:51-61` vs
  `crates/termlink-hub/src/channel.rs:36-70`).
- The hub has **no** facility to observe agent filesystem writes — path-claim
  coordination is honour-system only.
- `termlink dispatch --isolate` is the ONLY git-aware verb in the runtime
  (`crates/termlink-cli/src/commands/dispatch.rs:129-134, 533-540, 579-667`).
- 10 specific "TermLink would need X" gaps are listed in priority order in the
  report's GAPS section.

The architectural choice (extend TermLink, change the design, or accept the
gaps) is the human's call.

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

**Decision**: DEFER

**Rationale**: The task scope was explicit: "REPORT WHAT IS, not design or recommend an
architecture — the design decisions belong to the human." The findings report
delivers the evidence the human needs to make that decision. DEFER is therefore
not a holding pattern but the **terminal stance** for this inception — the
agent's job ended at producing the briefing.

**Date**: 2026-06-06T12:32:28Z

## Updates

<!-- Auto-populated by git mining at task completion.
     Manual entries optional during execution. -->

### 2026-06-05T19:19:07Z — status-update [task-update-agent]
- **Change:** status: captured → started-work

### 2026-06-06T12:32:28Z — inception-decision [inception-workflow]
- **Action:** Recorded inception decision
- **Decision:** DEFER
- **Rationale:** The task scope was explicit: "REPORT WHAT IS, not design or recommend an
architecture — the design decisions belong to the human." The findings report
delivers the evidence the human needs to make that decision. DEFER is therefore
not a holding pattern but the **terminal stance** for this inception — the
agent's job ended at producing the briefing.

### 2026-06-06T12:32:28Z — status-update [task-update-agent]
- **Change:** horizon: now → later
- **Change:** status: started-work → captured (auto-sync)
- **Reason:** Inception decision: DEFER — parking task

### 2026-06-07T11:15Z — design-output-linked [agent autonomous, T-2018]

The DEFER on this task was "terminal stance" because the agent's job ended at producing
the briefing — design ownership belongs to the human. The human has now produced that
design as a separate authored artifact, persisted at:

> `docs/architecture/parallel-execution-substrate.md` — *Architecture — Parallel
> Execution: TermLink Substrate Layer*

That ADR cites T-2007's findings as its substrate-reality grounding (§2) and the gap
list as the source of its §6 build manifest. So T-2007 is closed-as-research with this
artifact as the downstream design output. The build-track lineage (one task per §6
primitive, when operator decides to spend on it) will reference the ADR, not T-2007
directly. Persistence + linkage owned by T-2018.
