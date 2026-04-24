---
id: T-1223
name: "G-016 root cause: find DRY_RUN=0 bootstrap source"
description: >
  T-1222 deferred this: trace why the 2026-04-24T13:38 session-silent-scanner ran with DRY_RUN=0 (producing the commit storm before the cap fix). Check /root/.claude/shell-snapshots/*.sh for DRY_RUN export, .claude/settings.local.json for hook env, any SessionStart hook sources. If found: either fix the bootstrap or deprecate DRY_RUN=0 support entirely (the cap makes it tolerable but the trigger is still unknown).

status: work-completed
workflow_type: inception
owner: agent
horizon: now
tags: [G-016, T-1222-followup, framework-health]
components: []
related_tasks: []
created: 2026-04-24T15:48:55Z
last_update: 2026-04-24T16:21:39Z
date_finished: 2026-04-24T16:21:39Z
---

# T-1223: G-016 root cause: find DRY_RUN=0 bootstrap source

## Problem Statement

G-016 fired when the silent-session scanner ran destructively against /opt/termlink at 2026-04-24T11:38:23Z, generating 92 handover commits in 30 min before manual kill. T-1222 capped the blast radius but deferred identifying *what launched the scanner*. Without knowing the trigger, we cannot tell whether the compound flaw can recur via a different path.

## Assumptions

1. A Claude Code hook or cron entry fired the scanner (A1).
2. The invocation explicitly set DRY_RUN=0 to bypass the safety default (A2).
3. The bootstrap source is recoverable from transcripts / hook configs (A3).

## Exploration Plan

- S1: Grep cron.d, user crontabs, project cron-registry for scanner invocations.
- S2: Grep .claude/settings.json, settings.local.json, /root/.claude/settings.json for scanner hooks.
- S3: Walk session transcripts for Bash tool_use invoking the scanner near 11:38 UTC.
- S4: Inspect scanner log sequence to establish timing / invocation count.

## Findings

### F1 — Zero cron / hook trigger (A1 rejected)
- No /etc/cron.d/*, user crontab, or project cron-registry.yaml entry installs the scanner.
- settings.json PostToolUse/SessionStart arrays contain zero references to session-silent-scanner.
- The claim "Invoked via cron every 15 min" in the scanner docstring is aspirational — no cron entry exists.

### F2 — No DRY_RUN=0 bypass (A2 rejected)
- At 11:38:22Z the scanner file had NO DRY_RUN variable at all. The guard was added by Edit at 11:41:24Z — 3 minutes AFTER runaway started. So the destructive default was the only mode available; no explicit DRY_RUN=0 flag was set.

### F3 — Bootstrap source: smoke-test dispatcher command (A3 confirmed)
Exact trigger Bash tool_use at 2026-04-24T11:38:22.806Z in session d938f9cf-…:

- First leg: echo stub JSON piped into `hook session-end`
- Second leg: `hook session-silent-scanner 2>&1 | head -3`

The second leg invoked the agent dispatcher directly against real /opt/termlink. The dispatcher (bin/fw line 3889: `exec bash "$_hook_script" "$@"`) is a pass-through — it does NOT sandbox, does NOT set DRY_RUN=1. Combined with the scanner's then-missing DRY_RUN guard, the invocation recovered the full backlog of stale transcripts.

The `| head -3` suffix explains why the session appeared to exit quickly while the scanner kept running: head closes its stdin after 3 lines, but the scanner's subprocess.run(handover) loop writes to its own stdout (not through head). The scanner continued ~30 min until manually killed.

### F4 — Compound failure mode (three flaws, two fixed)

| Flaw | Status |
|---|---|
| Scanner had no DRY_RUN safety default | Fixed (prior session, default=1) |
| Scanner had no per-invocation cap | Fixed (T-1222, default=10) |
| Dispatcher is pass-through (no DRY_RUN guard at entry) | Unchanged |

Residual risk: if a future agent invokes the dispatcher from a project with `DRY_RUN=0` explicitly set in environment, or if an agent edits the scanner to remove the DRY_RUN default, the cap alone limits blast radius to 10 commits/invocation — annoying but not catastrophic.

## Scope Fence

IN: Trace the trigger. Assess whether current mitigations are sufficient.
OUT: Rewriting the hook dispatcher (framework-wide change requiring its own inception); retiring the fw hook verb pattern.

## Acceptance Criteria

### Agent
- [x] Problem statement validated — G-016 runaway scanner trigger traced to specific Bash tool_use at 2026-04-24T11:38:22Z
- [x] Assumptions tested — A1 rejected (no cron/hook), A2 rejected (no DRY_RUN=0 flag — guard didn't exist yet), A3 confirmed (smoke-test dispatcher command)
- [x] Recommendation written with rationale — NO-GO on further structural fix; current mitigations (cap + DRY_RUN default) are sufficient

### Human
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

## Recommendation

**Recommendation:** NO-GO on further structural fix. Root cause identified; compound mitigations already in place.

**Rationale:** The G-016 runaway was caused by a compound of three flaws, two of which are now fixed structurally:
1. Scanner had no DRY_RUN safety default (fixed: default=1)
2. Scanner had no per-invocation cap (fixed T-1222: default=10)
3. Dispatcher `fw hook session-silent-scanner` is a pass-through with no guard (unchanged)

With (1)+(2) in place, the dispatcher can no longer produce a 92-commit storm — even if invoked directly with `DRY_RUN=0`, the cap bounds damage to 10 commits per invocation. The hypothetical remaining failure mode (agent edits the DRY_RUN default out) is caught by code review on the scanner file, not by dispatcher guards.

A structural fix to the dispatcher (e.g., require explicit `DRY_RUN=0` opt-in at the `fw hook` verb level) would touch a framework-wide pattern affecting all 20+ hook scripts. That's disproportionate to the residual risk.

**Evidence:**
- F1: Zero cron/hook trigger — scanner was not scheduled
- F2: No `DRY_RUN=0` flag existed at invocation time (guard added 3 minutes later)
- F3: Trigger was smoke-test Bash tool_use at 11:38:22Z: `fw hook session-silent-scanner 2>&1 | head -3`
- F4: Two of three compound flaws already fixed; cap caps future blast radius

**Secondary action (no task needed):** Added publisher-hook-style behavioral note to learnings: don't smoke-test scanner-style dispatchers against real PROJECT_ROOT; use stub test with sandbox or explicit `DRY_RUN=1` env override.

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

**Rationale**: Recommendation: NO-GO on further structural fix. Root cause identified; compound mitigations already in place.

Rationale: The G-016 runaway was caused by a compound of three flaws, two of which are now fixed structurally:
1. Scanner had no DRY_RUN safety default (fixed: default=1)
2. Scanner had no per-invocation cap (fixed T-1222: default=10)
3. Dispatcher `fw hook session-silent-scanner` is a pass-through with no guard (unchanged)

With (1)+(2) in place, the dispatcher can no longer produce a 92-commit storm — even if invoked directly with `DRY_RUN=0`, the cap bounds damage to 10 commits per invocation. The hypothetical remaining failure mode (agent edits the DRY_RUN default out) is caught by code review on the scanner file, not by dispatcher guards.

A structural fix to the dispatcher (e.g., require explicit `DRY_RUN=0` opt-in at the `fw hook` verb level) would touch a framework-wide pattern affecting all 20+ hook scripts. That's disproportionate to the residual risk.

Evidence:
- F1: Zero cron/hook trigger — scanner was not scheduled
- F2: No `DRY_RUN=0` flag existed at invocation time (guard added 3 minutes later)
- F3: Trigger was smoke-test Bash tool_use at 11:38:22Z: `fw hook session-silent-scanner 2>&1 | head -3`
- F4: Two of three compound flaws already fixed; cap caps future blast radius

Secondary action (no task needed): Added publisher-hook-style behavioral note to learnings: don't smoke-test scanner-style dispatchers against real PROJECT_ROOT; use stub test with sandbox or explicit `DRY_RUN=1` env override.

**Date**: 2026-04-24T16:21:39Z

## Updates

<!-- Auto-populated by git mining at task completion.
     Manual entries optional during execution. -->

### 2026-04-24T15:57:21Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
- **Change:** horizon: next → now (auto-sync)

### 2026-04-24T16:21:39Z — inception-decision [inception-workflow]
- **Action:** Recorded inception decision
- **Decision:** NO-GO
- **Rationale:** Recommendation: NO-GO on further structural fix. Root cause identified; compound mitigations already in place.

Rationale: The G-016 runaway was caused by a compound of three flaws, two of which are now fixed structurally:
1. Scanner had no DRY_RUN safety default (fixed: default=1)
2. Scanner had no per-invocation cap (fixed T-1222: default=10)
3. Dispatcher `fw hook session-silent-scanner` is a pass-through with no guard (unchanged)

With (1)+(2) in place, the dispatcher can no longer produce a 92-commit storm — even if invoked directly with `DRY_RUN=0`, the cap bounds damage to 10 commits per invocation. The hypothetical remaining failure mode (agent edits the DRY_RUN default out) is caught by code review on the scanner file, not by dispatcher guards.

A structural fix to the dispatcher (e.g., require explicit `DRY_RUN=0` opt-in at the `fw hook` verb level) would touch a framework-wide pattern affecting all 20+ hook scripts. That's disproportionate to the residual risk.

Evidence:
- F1: Zero cron/hook trigger — scanner was not scheduled
- F2: No `DRY_RUN=0` flag existed at invocation time (guard added 3 minutes later)
- F3: Trigger was smoke-test Bash tool_use at 11:38:22Z: `fw hook session-silent-scanner 2>&1 | head -3`
- F4: Two of three compound flaws already fixed; cap caps future blast radius

Secondary action (no task needed): Added publisher-hook-style behavioral note to learnings: don't smoke-test scanner-style dispatchers against real PROJECT_ROOT; use stub test with sandbox or explicit `DRY_RUN=1` env override.

### 2026-04-24T16:21:39Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
- **Reason:** Inception decision: NO-GO
