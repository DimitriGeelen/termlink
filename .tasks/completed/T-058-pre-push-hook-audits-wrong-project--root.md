---
id: T-058
name: "Pre-push hook audits wrong project — root cause analysis and remediation"
description: >
  Pre-push hook audits wrong project — root cause analysis and remediation

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-03-09T11:11:57Z
last_update: 2026-03-09T11:18:27Z
date_finished: 2026-03-09T11:18:27Z
---

# T-058: Pre-push hook audits wrong project — root cause analysis and remediation

## Context

Pre-push hook audits the framework install directory instead of the project being pushed. Full RCA at `docs/reports/T-058-pre-push-hook-rca.md`. Root cause: hook doesn't pass PROJECT_ROOT env var to audit script.

## Acceptance Criteria

### Agent
- [x] Root cause identified and documented in RCA report
- [x] Local pre-push hook fixed (passes PROJECT_ROOT to audit script)
- [x] `.framework.yaml` framework_path uses stable symlink
- [x] Remediation instructions for framework agent written

## Verification

grep -q 'PROJECT_ROOT="\$PROJECT_ROOT"' .git/hooks/pre-push
grep -q '/usr/local/opt/agentic-fw/libexec' .framework.yaml
test -f docs/reports/T-058-pre-push-hook-rca.md

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

### 2026-03-09T11:11:57Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /Users/dimidev32/001-projects/010-termlink/.tasks/active/T-058-pre-push-hook-audits-wrong-project--root.md
- **Context:** Initial task creation

### 2026-03-09T11:18:27Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
