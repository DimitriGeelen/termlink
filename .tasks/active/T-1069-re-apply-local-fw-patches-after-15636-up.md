---
id: T-1069
name: "Re-apply local fw patches after 1.5.636 upgrade"
description: >
  Re-apply local fw patches after 1.5.636 upgrade

status: started-work
workflow_type: refactor
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-04-15T21:02:11Z
last_update: 2026-04-15T21:02:11Z
date_finished: null
---

# T-1069: Re-apply local fw patches after 1.5.636 upgrade

## Context

`fw upgrade` (vendored 0.9.23 → upstream 1.5.636) overwrote two local patches:
T-1066 (`fw task review-queue` subcommand) and T-1068 (handover partial-complete
tags + age sort). Both patches have recipes in `docs/patches/T-1066-*.md` and
`docs/patches/T-1068-*.md`. This task re-applies them to the freshly-vendored
framework so local tooling continues to work until the patches are propagated
upstream.

## Acceptance Criteria

### Agent
- [ ] `fw task review-queue` subcommand runs and lists partial-complete tasks
- [ ] `fw task review-queue --count` prints an integer
- [ ] `fw task help` output references `review-queue`
- [ ] Handover's `PARTIAL_COMPLETE_SECTION` uses date_finished-ASC sort + tag prefixes
- [ ] Both patch files (`docs/patches/T-1066-*.md`, `docs/patches/T-1068-*.md`) kept as upstream-propagation recipes

### Human
<!-- Criteria requiring human verification (UI/UX, subjective quality). Not blocking.
     Remove this section if all criteria are agent-verifiable.
     Each criterion MUST include Steps/Expected/If-not so the human can act without guessing.
     Optionally prefix with [RUBBER-STAMP] or [REVIEW] for prioritization.
     Example:
       - [ ] [REVIEW] Dashboard renders correctly
         **Steps:**
         1. Open https://example.com/dashboard in browser
         2. Verify all panels load within 2 seconds
         3. Check browser console for errors
         **Expected:** All panels visible, no console errors
         **If not:** Screenshot the broken panel and note the console error
-->

## Verification

# Shell commands that MUST pass before work-completed. One per line.
./.agentic-framework/bin/fw task review-queue --count 2>&1 | grep -qE '^[0-9]+$'
grep -q 'date_finished ASC' .agentic-framework/agents/handover/handover.sh 2>/dev/null || grep -q "sort.*date_finished" .agentic-framework/agents/handover/handover.sh

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

### 2026-04-15T21:02:11Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1069-re-apply-local-fw-patches-after-15636-up.md
- **Context:** Initial task creation
