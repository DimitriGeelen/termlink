---
id: T-291
name: "Housekeeping — fix all audit warnings before push to origin"
description: >
  Housekeeping — fix all audit warnings before push to origin

status: started-work
workflow_type: refactor
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-03-26T11:04:11Z
last_update: 2026-03-26T21:25:59Z
date_finished: null
---

# T-291: Housekeeping — fix all audit warnings before push to origin

## Context

Pre-push audit has 1 FAIL (CTL-009 T-258 inception without decision) and ~50 warnings (missing episodics, missing research artifacts, stale gaps.yaml, etc.). Fix all to get a clean push to origin before switching development to .107.

## Acceptance Criteria

### Agent
- [x] CTL-009 FAIL on T-258 resolved (decision added — prior session)
- [x] All missing episodic summaries generated (0 missing now)
- [x] 5 inception research artifacts created (T-205, T-206, T-208, T-209, T-245 — all exist)
- [x] Stale gaps.yaml removed (no longer present)
- [x] T-283 and T-287 research artifact references added to task Updates (prior session)
- [x] Pre-push audit passes with 0 FAILs (T-293 resolved all)
- [x] All changes committed and pushed to origin (f0879af pushed to onedev)

## Verification

# No missing episodics
test "$(python3 -c "import glob,os; completed=glob.glob('.tasks/completed/T-*.md'); missing=[f for f in completed if not os.path.exists('.context/episodic/'+os.path.basename(f).split('-')[0]+'-'+os.path.basename(f).split('-')[1]+'.yaml')]; print(len(missing))")" = "0"
# Research artifacts exist
test -f docs/reports/T-205-pyyaml-phantom-dependency.md
test -f docs/reports/T-206-remove-sudo-from-installer.md

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

### 2026-03-26T11:04:11Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /Users/dimidev32/001-projects/010-termlink/.tasks/active/T-291-housekeeping--fix-all-audit-warnings-bef.md
- **Context:** Initial task creation
