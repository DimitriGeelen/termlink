---
id: T-254
name: "Failover optimization — parallel probe or circuit breaker for dead candidates"
description: >
  Serial failover with fixed timeout means N dead candidates = N * 5s latency. Add parallel probing, circuit breaker pattern, or liveness cache to avoid cascading timeouts. See docs/reports/T-247-scenarios-adversarial.md Scenario 3.

status: captured
workflow_type: build
owner: agent
horizon: next
tags: [T-247, orchestration, routing]
components: []
related_tasks: [T-247, T-237, T-233]
created: 2026-03-23T16:54:31Z
last_update: 2026-03-23T16:54:31Z
date_finished: null
---

# T-254: Failover optimization — parallel probe or circuit breaker for dead candidates

## Context

Serial failover with N dead candidates causes N * timeout_secs latency in `orchestrator.route`. A circuit breaker pattern per session avoids cascading timeouts by skipping known-dead sessions. See `docs/reports/T-247-scenarios-adversarial.md` Scenario 3. Implementation targets `crates/termlink-hub/src/router.rs` (candidate iteration) and a new circuit breaker module in `crates/termlink-hub/src/`.

## Acceptance Criteria

### Agent
- [ ] Circuit breaker state tracked per session: after 3 consecutive transport failures, mark session as "open" (skip for 60s)
- [ ] `orchestrator.route` skips open-circuit sessions during candidate iteration
- [ ] Circuit breaker auto-resets after cooldown period (half-open state: try one request, close circuit on success)
- [ ] Test: dead session gets circuit-opened after 3 consecutive failures; subsequent routes skip it
- [ ] Test: circuit auto-resets after cooldown; session is tried again and circuit closes on success
- [ ] All hub tests pass (`cargo test --package termlink-hub`)

## Verification

/Users/dimidev32/.cargo/bin/cargo test --package termlink-hub

## Decisions

<!-- Record decisions ONLY when choosing between alternatives.
     Skip for tasks with no meaningful choices.
     Format:
     ### [date] — [topic]
     - **Chose:** [what was decided]
     - **Why:** [rationale]
     - **Rejected:** [alternatives and why not]
-->

## Updates

### 2026-03-23T16:54:31Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /Users/dimidev32/001-projects/010-termlink/.tasks/active/T-254-failover-optimization--parallel-probe-or.md
- **Context:** Initial task creation
