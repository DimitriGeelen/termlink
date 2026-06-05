---
id: T-1991
name: "agent-presence topic-bloat — discovery slowdown on .122/.107 from monotonic growth"
description: >
  Inception: agent-presence topic-bloat — discovery slowdown on .122/.107 from monotonic growth

status: started-work
workflow_type: inception
owner: human
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-06-05T09:23:56Z
last_update: 2026-06-05T09:24:16Z
date_finished: null
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

## Recommendation

**GO** — close T-1991 with two follow-up tasks; see
`docs/reports/T-1991-channel-info-hub-concurrency-regression.md` for the
full spike data.

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

Need spike: actual measurements of channel subscribe latency vs cursor depth on .107 (13441 envelopes) and .122 (1493 envelopes). T-1844 already added cursor=count-limit windowing client-side, but if the hub seeks to cursor by scanning, the cost is still O(count). Decision points: (a) does hub seek require scan? (b) right fix — retention policy / topic rotation / hub-side index. Cannot recommend GO/NO-GO until spike data exists.

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

### 2026-06-05T09:24:16Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
