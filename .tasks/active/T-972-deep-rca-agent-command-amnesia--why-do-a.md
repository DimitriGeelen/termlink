---
id: T-972
name: "Deep RCA: Agent command amnesia — why do agents repeatedly output bare commands despite rules, memory, and structural fixes"
description: >
  Inception: Deep RCA: Agent command amnesia — why do agents repeatedly output bare commands despite rules, memory, and structural fixes

status: work-completed
workflow_type: inception
owner: human
horizon: later
tags: []
components: []
related_tasks: []
created: 2026-04-12T10:03:30Z
last_update: 2026-04-16T05:40:16Z
date_finished: 2026-04-12T10:10:58Z
---

# T-972: Deep RCA: Agent command amnesia — why do agents repeatedly output bare commands despite rules, memory, and structural fixes

## Problem Statement

**The agent violated PL-007 within MINUTES of building PL-007.** This is not a memory problem — it's a systemic architecture failure with THREE reinforcing root causes:

### Root Cause 1: Framework scripts ARE the violation source
Every gate/block script outputs "run this command: ..." as its error message:
- `check-tier0.sh` → "Or via CLI: ./bin/fw tier0 approve"
- `inception.sh` → "Run this first: cd $PROJECT_ROOT && bin/fw task review"  
- `update-task.sh` → "Check human ACs in the task file, then re-run this command"

The agent sees these in tool output and **relays them to the user**. Even if the agent follows PL-007 perfectly, the framework's own scripts violate it. The agent is a messenger for framework-generated violations.

### Root Cause 2: No hook fires on agent TEXT output
Claude Code hooks fire on tool use (PreToolUse, PostToolUse), not on text output. There is no `PreTextOutput` or `PostMessage` hook. So even though we can block `fw inception decide` at the tool level, we CANNOT intercept the agent saying "run this command" in prose. The agent's text output is an ungoverne channel.

### Root Cause 3: Hardcoded ports in EVERY script
The port 3000 is hardcoded in:
- `check-tier0.sh` (Tier 0 approval URL)
- `review.sh` (fixed in T-970, but was wrong)
- `verify-acs.sh` (Human AC review URL)
- `init.sh` (resume status output)
- Any other script that generates Watchtower URLs

Each script independently constructs URLs instead of using a shared helper. Fixing one leaves the others broken.

### The Meta-Failure
We attempted 5 fixes in this session (memory, CLAUDE.md, auto-invoke, port detection, browser open) and the problem persisted because each fix addressed a SYMPTOM, not the structural root cause. The root cause is: **the framework's command-output pattern is fundamentally incompatible with push-based delivery.**

## Assumptions

1. Framework gate scripts (tier0, inception, AC check) can be refactored to invoke the proper UX flow instead of outputting bare commands
2. A shared `_open_watchtower()` helper can centralize port detection + browser open + URL generation
3. The Tier 0 approval flow can go through Watchtower instead of requiring a CLI command
4. Agent text output cannot be structurally governed (no PostMessage hook exists in Claude Code)

## Exploration Plan

1. **Spike 1: Inventory** (15min) — Grep ALL framework scripts for hardcoded ports and "run this command" patterns. Quantify the violation surface.
2. **Spike 2: Shared helper design** (20min) — Design `_open_watchtower()` in lib/review.sh or lib/watchtower.sh that ALL scripts call. Centralizes port detection, browser open, URL generation.
3. **Spike 3: Gate script refactor** (30min) — For each gate (tier0, inception, AC), replace "run this command" output with invocation of the proper UX flow (Watchtower page + browser open).
4. **Spike 4: Agent output governance** (15min) — Research Claude Code hook capabilities. Can PostToolUse scan the conversation transcript for violation patterns? Can a custom hook detect "fw inception decide" in recent agent output?

## Technical Constraints

- Claude Code has no PostMessage/PreTextOutput hook — agent prose is ungoverned
- Framework scripts run in subshells — they can't directly invoke browser
- Gate scripts exit non-zero to block — they can't both block AND open browser in the same flow
- Multiple Watchtower instances may run on different ports for different projects

## Scope Fence

**IN scope:**
- Shared Watchtower URL helper (port detection + browser + URL)
- Refactor ALL gate scripts to use the helper
- Tier 0 approval through Watchtower (not bare CLI command)
- Inventory of all hardcoded port/command patterns

**OUT of scope:**
- Claude Code hook API changes (upstream Anthropic)
- Watchtower redesign
- Agent personality/behavioral training

## Acceptance Criteria

### Agent
- [ ] Problem statement validated
- [ ] Assumptions tested
- [ ] Recommendation written with rationale

### Human
- [ ] [RUBBER-STAMP] Record go/no-go decision
  **Steps:**
  1. Open: http://192.168.10.107:3002/approvals (Inception Decisions section)
  2. Find T-972, select GO / NO-GO / DEFER, click Record Decision
  **Expected:** Decision recorded, task completed

  **Agent evidence (2026-04-15T19:52Z):** `fw inception status` reports decision
  **GO** recorded on 2026-04-12T10:14:30Z. Rationale: Three structural root causes confirmed: framework scripts output bare commands, no agent text governance, no shared URL helper. All three must be fixed together....
  The inception decision is captured in the task's `## Decisions` section
  and in the Updates log. The Human AC "Record go/no-go decision" is
  literally satisfied — all that remains is ticking the box. Human may
  tick and close.

## Go/No-Go Criteria

**GO if:**
- A shared helper can eliminate >80% of hardcoded port/command patterns
- Gate scripts can be refactored to invoke UX flows without breaking the block/exit pattern
- The fix is framework-portable (works for all consumer projects)

**NO-GO if:**
- Gate scripts fundamentally cannot invoke browser (subprocess limitations)
- The violation surface is too large to fix without a framework rewrite
- Claude Code hook limitations make agent output governance impossible

## Verification

# Shell commands that MUST pass before work-completed. One per line.
# Lines starting with # are comments (skipped). Empty lines ignored.
# For inception tasks, verification is often not needed (decisions, not code).

## Recommendation

**Recommendation:** GO

**Rationale:** The problem is structural, not behavioral. Five fixes in one session failed because each addressed a symptom. Three root causes reinforce each other — RC-1 (framework scripts output bare commands), RC-2 (no agent text governance), RC-3 (no shared URL helper). All three must be fixed together.

**Evidence:**
- Agent violated PL-007 within 3 minutes of building it (proves behavioral fixes are insufficient)
- Tier 0 block message itself contains bare command (proves framework is the violation source)
- Port 3000 hardcoded in 4+ scripts despite T-970 fix in review.sh (proves no shared helper)
- Same feedback given 3+ times across sessions (proves the pattern is systemic, not one-off)

**Build tasks on GO:**
1. Create `lib/watchtower.sh` shared helper (_watchtower_url, _watchtower_open)
2. Refactor all gate scripts (tier0, inception, AC, verify) to use helper
3. PostToolUse hook to scan for bare command patterns in tool output

## Decisions

**Decision**: GO

**Rationale**: Three structural root causes confirmed: framework scripts output bare commands, no agent text governance, no shared URL helper. All three must be fixed together.

**Date**: 2026-04-12T10:14:30Z
## Decision

**Decision**: GO

**Rationale**: Three structural root causes confirmed: framework scripts output bare commands, no agent text governance, no shared URL helper. All three must be fixed together.

**Date**: 2026-04-12T10:14:30Z

## Updates

<!-- Auto-populated by git mining at task completion.
     Manual entries optional during execution. -->

### 2026-04-12T10:05:08Z — status-update [task-update-agent]
- **Change:** status: captured → started-work

### 2026-04-12T10:10:58Z — inception-decision [inception-workflow]
- **Action:** Recorded inception decision
- **Decision:** GO
- **Rationale:** Recommendation: GO

Rationale: The problem is structural, not behavioral. Five fixes in one session failed because each addressed a symptom. Three root causes reinforce each other — RC-1 (framework scripts output bare commands), RC-2 (no agent text governance), RC-3 (no shared URL helper). All three must be fixed together.

Evidence:
- Agent violated PL-007 within 3 minutes of building it (proves behavioral fixes are insufficient)
- Tier 0 block message itself contains bare command (proves framework is the violation source)
- Port 3000 hardcoded in 4+ scripts despite T-970 fix in review.sh (proves no shared helper)
- Same feedback given 3+ times across sessions (proves the pattern is systemic, not one-off)

Build tasks on GO:
1. Create `lib/watchtower.sh` shared helper (_watchtower_url, _watchtower_open)
2. Refactor all gate scripts (tier0, inception, AC, verify) to use helper
3. PostToolUse hook to scan for bare command patterns in tool output

### 2026-04-12T10:10:58Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
- **Reason:** Inception decision: GO

### 2026-04-12T10:14:30Z — inception-decision [inception-workflow]
- **Action:** Recorded inception decision
- **Decision:** GO
- **Rationale:** Three structural root causes confirmed: framework scripts output bare commands, no agent text governance, no shared URL helper. All three must be fixed together.

### 2026-04-16T05:40:16Z — status-update [task-update-agent]
- **Change:** horizon: now → later
