---
id: T-258
name: "Context amnesia root cause — T-233 vision lost across sessions"
description: >
  The T-233 specialist orchestration GO decision (the most important architectural decision in the project)
  was lost across sessions. Five child tasks (T-239-T-242, T-256) were incorrectly NO-GO'd because the
  incoming agent had no access to the architectural vision. Root cause: 5 structural gaps in the context
  framework's decision persistence pipeline.

status: work-completed
workflow_type: inception
owner: agent
horizon: now
tags: [context, framework, critical]
components: []
related_tasks: [T-233, T-247, T-239, T-240, T-241, T-242, T-256]
created: 2026-03-24T07:25:29Z
last_update: 2026-03-24T08:22:16Z
date_finished: 2026-03-24T08:22:16Z
---

# T-258: Context amnesia root cause — T-233 vision lost across sessions

## Problem Statement

The T-233 specialist orchestration GO decision and its 5 architectural principles were lost across
session boundaries. A new agent session NO-GO'd all child tasks (T-239, T-240, T-241, T-242, T-256)
because it evaluated each as isolated features rather than building blocks of an approved architecture.

Five structural gaps identified:
1. T-233 never formally completed → no episodic generated (sovereignty gate blocked)
2. No auto-promotion from task `## Decisions` to `decisions.yaml`
3. `decisions.yaml` has zero project-specific decisions (173 lines, all framework-seeded)
4. Handovers are narrative prose, not queryable structured data
5. `/resume` doesn't load architectural context or decisions

## Scope Fence

**IN scope:**
- Fix A: Generate missing T-233/T-247 episodics
- Fix B: Capture T-233 decisions into `decisions.yaml`
- Fix C: Save vision to Claude memory for cross-session persistence
- Fix D-F: Local prototype fixes for framework gaps, then dispatch as inception pickup prompts

**OUT of scope:**
- Framework repo edits (separate project, dispatched via TermLink)

## Acceptance Criteria

- [x] T-233 episodic summary exists in `.context/episodic/`
- [x] T-233 architectural decisions captured in `decisions.yaml`
- [x] Claude memory file preserves T-233 vision for future sessions
- [x] Root cause analysis artifact at `docs/reports/T-258-context-amnesia.md`
- [x] Framework fix inception pickup prompts created and ready to dispatch
- [x] 5 incorrectly NO-GO'd tasks reversed to GO with full audit trail

## Verification

test -f .context/episodic/T-233.yaml
grep -q "T-233" .context/project/decisions.yaml
test -f docs/reports/T-258-context-amnesia.md

## Decisions

### 2026-03-24 — Root cause analysis scope
- **Decision:** GO — Fix what we can locally (episodics, decisions.yaml, memory) + dispatch framework fixes as inception pickup prompts
- **Why:** Framework repo is a separate project — can't edit directly from TermLink consumer
- **Rejected:** Waiting for framework fixes first (would block reversal of 5 incorrectly NO-GO'd tasks)

## Updates

### 2026-03-24T08:22:16Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
