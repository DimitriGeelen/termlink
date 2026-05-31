---
id: T-1830
name: "Doorbell+mail adoption gap — drive zero active conversations to non-zero"
description: >
  Inception: Doorbell+mail adoption gap — drive zero active conversations to non-zero

status: work-completed
workflow_type: inception
owner: human
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-05-28T11:37:43Z
last_update: 2026-05-28T12:37:55Z
date_finished: 2026-05-28T12:37:55Z
---

# T-1830: Doorbell+mail adoption gap — drive zero active conversations to non-zero

## Problem Statement

<!-- What problem are we exploring? For whom? Why now? -->

## Assumptions

<!-- Key assumptions to test. Register with: fw assumption add "Statement" --task T-XXX -->

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

T-1829 live fleet validation 2026-05-28 PROVED runtime is healthy on all three reachable hubs (.107/.121/.122 selftest PASS in 51/391/453 ms) yet active conversation count = 0 across 91 topics. T-1807 validated end-to-end determinism in May. The gap is NOT infrastructure (selftest passes everywhere) — it's coordination: (1) no discovery primitive for who's listening, (2) no convention for always-on /check-arc respond listeners, (3) agent-send.sh requires --peer-fp/--to-session that operators don't have without prior coordination. GO recommended for inception because the runtime work is done; the remaining work is socio-technical (protocol design + adoption convention). Inception should explore: (a) heartbeat/listener-presence topic, (b) discovery verb listing active listeners, (c) agent-send.sh auto-discover. Each could be a small build task — but the wiring decisions need a deliberate design pass first.

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

**Decision**: GO

**Rationale**: T-1829 live fleet validation 2026-05-28 PROVED runtime is healthy on all three reachable hubs (.107/.121/.122 selftest PASS in 51/391/453 ms) yet active conversation count = 0 across 91 topics. T-1807 validated end-to-end determinism in May. The gap is NOT infrastructure (selftest passes everywhere) — it's coordination: (1) no discovery primitive for who's listening, (2) no convention for always-on /check-arc respond listeners, (3) agent-send.sh requires --peer-fp/--to-session that operators don't have without prior coordination. GO recommended for inception because the runtime work is done; the remaining work is socio-technical (protocol design + adoption convention). Inception should explore: (a) heartbeat/listener-presence topic, (b) discovery verb listing active listeners, (c) agent-send.sh auto-discover. Each could be a small build task — but the wiring decisions need a deliberate design pass first.

**Date**: 2026-05-28T12:37:54Z

## Updates

<!-- Auto-populated by git mining at task completion.
     Manual entries optional during execution. -->

### 2026-05-28T12:37:54Z — inception-decision [inception-workflow]
- **Action:** Recorded inception decision
- **Decision:** GO
- **Rationale:** T-1829 live fleet validation 2026-05-28 PROVED runtime is healthy on all three reachable hubs (.107/.121/.122 selftest PASS in 51/391/453 ms) yet active conversation count = 0 across 91 topics. T-1807 validated end-to-end determinism in May. The gap is NOT infrastructure (selftest passes everywhere) — it's coordination: (1) no discovery primitive for who's listening, (2) no convention for always-on /check-arc respond listeners, (3) agent-send.sh requires --peer-fp/--to-session that operators don't have without prior coordination. GO recommended for inception because the runtime work is done; the remaining work is socio-technical (protocol design + adoption convention). Inception should explore: (a) heartbeat/listener-presence topic, (b) discovery verb listing active listeners, (c) agent-send.sh auto-discover. Each could be a small build task — but the wiring decisions need a deliberate design pass first.

### 2026-05-28T12:37:55Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
- **Reason:** Inception decision in progress

## Reviewer Verdict (v1.4)

- **Scan ID:** R-8023346d
- **Timestamp:** 2026-05-28T12:37:55Z
- **Catalogue:** v1.3-seed
- **Overall:** PASS
- **Needs Human:** no
- **Findings:** none

### 2026-05-28T12:37:55Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
- **Reason:** Inception decision: GO
