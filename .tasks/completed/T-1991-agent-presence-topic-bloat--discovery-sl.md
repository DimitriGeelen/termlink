---
id: T-1991
name: "agent-presence topic-bloat — discovery slowdown on .122/.107 from monotonic growth"
description: >
  Inception: agent-presence topic-bloat — discovery slowdown on .122/.107 from monotonic growth

status: work-completed
workflow_type: inception
owner: human
horizon: null
tags: []
components: []
related_tasks: []
created: 2026-06-05T09:23:56Z
last_update: 2026-06-06T20:25:31Z
date_finished: 2026-06-06T20:25:31Z
---

# T-1991: agent-presence topic-bloat — discovery slowdown on .122/.107 from monotonic growth

## Problem Statement

The `fw task update --status work-completed` verification gate wedged
during T-1990 close on a call to `termlink channel info agent-presence
--json --hub 192.168.10.122:9100`. Same call returns in 50-200ms when
invoked once but times out at 15s with ~45% probability under repeated
sequential use. This regression silently impairs every diagnostic that
hits `channel info` on a non-trivial topic on a 0.11.473 hub — `/peers`,
`/pulse`, `agent-listeners.sh --include-offline`, the work-completed
verification gate. Affects everyone using ring20 infrastructure.

## Assumptions

Three assumptions tested in this inception:

- **A1** — Topic bloat (1493 envelopes on .122 agent-presence) is the
  perf cost. **DISPROVEN.** Subscribe is O(1) on cursor depth.
- **A2** — Slowness is on the script side, not the hub. **DISPROVEN.**
  `agent-listeners.sh` is fast (280ms end-to-end). The hub returns
  in 50-100ms when called once.
- **A3** — Same call sequence on .107 (0.11.472) behaves identically.
  **DISPROVEN.** .107 is 0/20 timeouts even with 13K-envelope topic
  on LAN. Regression is per-hub-binary-version, specifically 0.11.473.

## Exploration Plan

1. Probe `channel subscribe` latency vs cursor depth on .122 (1493
   envelopes) and .107 (13441 envelopes) — done.
2. Probe end-to-end `agent-listeners.sh` on .122 and .107 — done.
3. Stress 20-trial sequential `channel info` on every hub × every topic
   size combination — done.
4. Compare across versions (0.11.472 vs 0.11.473) over matched LAN
   conditions — done.

Time-box: spike completed in one session block (~45 min from anomaly
to recommendation).

## Technical Constraints

- 0.11.472 has been observed on 2 hubs (.107 + local-test) — both clean.
- 0.11.473 has been observed on 3 hubs (.121, .122, .141) — all flaky
  per the rate table in the research artifact.
- Downgrading to 0.11.472 is a `fleet-deploy-binary.sh --probe` op away
  but loses any intentional behavior change in 0.11.473.

## Scope Fence

**IN scope:** Confirm the regression is real, identify which axis
(version / topic-size / RPC / host) controls it, recommend remediation.

**OUT of scope:** Bisecting the offending commit in 0.11.472..0.11.473.
Operator-side cache implementation. Hub-side fix. All deferred to
follow-up tasks created at decide-time.

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

Filing-time recommendation was DEFER pending spike data. Spike complete; data clean.
The regression is real, reproducible at 45% sequential timeout rate on .122
`channel info agent-presence`, fleet-wide on every 0.11.473 hub. Two follow-up tasks
will be filed at decide-time: (1) operator-side client cache for
`scripts/agent-listeners.sh` (small build), (2) hub-side bisect+fix of the
0.11.473 regression (inception → build). Updated recommendation: GO on both.

**Evidence:**

- Spike data: `docs/reports/T-1991-channel-info-hub-concurrency-regression.md`
- A1 disproven: subscribe latency O(1) on cursor depth (.122 1493 envelopes vs .107 13441 envelopes — both <100ms on `channel subscribe`)
- A2 disproven: `agent-listeners.sh` 280ms end-to-end against a healthy hub; one-shot `channel info` 50-100ms
- A3 disproven: 0.11.472 (.107) 0/20 sequential timeouts; 0.11.473 (.122, .121, .141) all flake at 15s with ~45% probability
- Regression axis isolated: per-hub-binary-version (0.11.472 clean, 0.11.473 flaky), NOT topic size NOT host class
- Follow-up tasks to file at decide-time: (1) operator-side client cache for `scripts/agent-listeners.sh` (small build), (2) hub-side bisect+fix of 0.11.473 regression (inception → build)

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

**Rationale**: Filing-time recommendation was DEFER pending spike data. Spike complete; data clean.
The regression is real, reproducible at 45% sequential timeout rate on .122
`channel info agent-presence`, fleet-wide on every 0.11.473 hub. Two follow-up tasks
will be filed at decide-time: (1) operator-side client cache for
`scripts/agent-listeners.sh` (small build), (2) hub-side bisect+fix of the
0.11.473 regression (inception → build). Updated recommendation: GO on both.

**Date**: 2026-06-06T20:25:31Z

## Updates

<!-- Auto-populated by git mining at task completion.
     Manual entries optional during execution. -->

### 2026-06-05T09:24:16Z — status-update [task-update-agent]
- **Change:** status: captured → started-work

### 2026-06-06T20:20:32Z — inception-decision [inception-workflow]
- **Action:** Recorded inception decision
- **Decision:** GO
- **Rationale:** Approved via Watchtower (no rationale captured)

### 2026-06-06T20:25:31Z — inception-decision [inception-workflow]
- **Action:** Recorded inception decision
- **Decision:** GO
- **Rationale:** Filing-time recommendation was DEFER pending spike data. Spike complete; data clean.
The regression is real, reproducible at 45% sequential timeout rate on .122
`channel info agent-presence`, fleet-wide on every 0.11.473 hub. Two follow-up tasks
will be filed at decide-time: (1) operator-side client cache for
`scripts/agent-listeners.sh` (small build), (2) hub-side bisect+fix of the
0.11.473 regression (inception → build). Updated recommendation: GO on both.

### 2026-06-06T20:25:31Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
- **Reason:** Inception decision: GO
