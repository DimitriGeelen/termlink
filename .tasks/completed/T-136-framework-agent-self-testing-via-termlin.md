---
id: T-136
name: "Framework agent self-testing via TermLink — spawn, observe, fix loop"
description: >
  Inception: Can the framework agent use TermLink to spawn terminals, test its
  own scripts (fw doctor, hooks, init), observe output, diagnose failures, fix
  them, and retry — all without human intervention?

status: work-completed
workflow_type: inception
owner: human
horizon: now
tags: [framework, self-test, agent-autonomy]
components: []
related_tasks: [T-121, T-011, T-100]
created: 2026-03-14T16:38:43Z
last_update: 2026-03-16T07:46:45Z
date_finished: 2026-03-16T07:46:45Z
---

# T-136: Framework agent self-testing via TermLink — spawn, observe, fix loop

## Problem Statement

The framework agent (Claude Code + agentic-fw) cannot autonomously test its own
tooling. When `fw doctor` fails, or a hook script has a bug, or `fw context init`
breaks, the human must manually open another terminal, run the command, paste
output back, and iterate. This creates a bottleneck in framework development and
prevents the agent from building a self-healing loop.

TermLink already has all the primitives: session spawning, command execution with
captured output, event-based coordination, and streaming observation. The question
is whether these can be composed into a reliable automated test loop.

## Assumptions

- A1: VALIDATED — TermLink `command.execute` returns structured {exit_code, stdout, stderr}
- A2: VALIDATED — Three observation mechanisms available (execute, scrollback, streaming)
- A3: VALIDATED — Claude Code Bash tool can invoke TermLink CLI commands
- A4: VALIDATED — Fresh TermLink sessions provide clean isolated environments
- A5: PARTIALLY VALIDATED — Agent can fix and retry; limitation: CLAUDE.md/settings reload needs new session

## Exploration Plan

1. ~~Research observation capabilities~~ — DONE (see inception report)
2. ~~Validate assumptions A1-A5~~ — DONE
3. ~~Evaluate options (direct bash vs execute vs streaming vs hybrid)~~ — DONE
4. Record go/no-go decision — PENDING human review

## Technical Constraints

- TermLink binary must be on PATH (true in dev environment)
- `command.execute` has 30s default timeout (configurable)
- Hook changes take effect immediately; CLAUDE.md changes need session restart
- Interactive commands (requiring stdin) need inject+streaming, not execute

## Scope Fence

**IN scope:**
- Validating that TermLink primitives support the self-test loop
- Documenting the protocol (spawn → exec → observe → fix → retry)
- Decision: go/no-go on building the integration

**OUT of scope:**
- Building the actual /self-test skill (Phase 1 — separate build task)
- Interactive PTY testing (Phase 3 — separate build task)
- Cross-machine self-testing (depends on T-011 Phase 2)

## Acceptance Criteria

- [x] Problem statement validated
- [x] Assumptions tested (A1-A5)
- [x] Research artifact written (docs/reports/T-136-framework-self-testing-inception.md)
- [x] Options evaluated (4 options, hybrid chosen)
- [x] Go/No-Go decision recorded

## Go/No-Go Criteria

**GO if:**
- All required TermLink primitives exist (execute, observe, spawn) — YES
- No new TermLink code needed for basic loop — YES
- Integration effort is bounded (≤3 sessions for Phase 0+1) — YES

**NO-GO if:**
- Critical capability missing (e.g., can't capture output) — NOT THE CASE
- Requires architectural changes to TermLink — NOT THE CASE
- Integration effort exceeds 5 sessions — NOT THE CASE

## Verification

test -f docs/reports/T-136-framework-self-testing-inception.md
test -f docs/reports/T-136-termlink-status-report.md

## Decisions

**Decision**: GO

**Rationale**: Spike validated all 7 test cases: inject, scrollback, fw doctor E2E, marker sync, state persistence, cleanup. All primitives work. No new TermLink code needed for Phase 0.

**Date**: 2026-03-14T16:59:00Z
## Decision

**Decision**: GO

**Rationale**: Spike validated all 7 test cases: inject, scrollback, fw doctor E2E, marker sync, state persistence, cleanup. All primitives work. No new TermLink code needed for Phase 0.

**Date**: 2026-03-14T16:59:00Z

## Dialogue Log

### Human question: How does the agent see what happens in the other terminal?
Three mechanisms validated:
1. `command.execute` → structured JSON with exit_code + stdout + stderr (simplest)
2. `query.output` → scrollback buffer read (1 MiB ring, poll after execution)
3. Data plane streaming → real-time binary frames (for long/interactive commands)

For the self-test use case, `command.execute` is sufficient — it returns
everything the agent needs in a single RPC call.

### Human concern: Do we need streaming?
Not for the basic loop. `command.execute` captures output synchronously.
Streaming is only needed for:
- Commands that run >30s (override timeout or stream)
- Interactive programs (vim, REPLs) where you need real-time observation
- Debugging scenarios where you want to see output as it happens

## Updates

### 2026-03-14T16:38:43Z — task-created
- Created inception task for framework self-testing via TermLink

### 2026-03-14T16:45:00Z — research complete
- Validated all 5 assumptions
- Wrote status report: docs/reports/T-136-termlink-status-report.md
- Wrote inception report: docs/reports/T-136-framework-self-testing-inception.md
- Evaluated 4 options, chose hybrid approach
- Verdict: GO — all primitives exist, no new TermLink code needed

### 2026-03-14T16:59:00Z — inception-decision [inception-workflow]
- **Action:** Recorded inception decision
- **Decision:** GO
- **Rationale:** Spike validated all 7 test cases: inject, scrollback, fw doctor E2E, marker sync, state persistence, cleanup. All primitives work. No new TermLink code needed for Phase 0.

### 2026-03-16T07:46:45Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
