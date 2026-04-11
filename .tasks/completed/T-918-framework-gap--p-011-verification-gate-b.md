---
id: T-918
name: "Framework gap — P-011 verification gate blind to un-staged files"
description: >
  P-011 verification gate runs `test -f <path>` and similar shell commands against
  the working tree, not the git index. Files that exist on disk but were never
  staged pass the gate silently, letting tasks close with un-committed deliverables.
  Observed 2026-04-11 while closing T-906 and T-907 — their report files had sat
  untracked in docs/reports/ since 2026-04-08 and the gate passed anyway.

status: work-completed
workflow_type: build
owner: agent
horizon: later
tags: [framework, upstream, governance, P-011]
components: []
related_tasks: [T-904, T-906, T-907]
created: 2026-04-11T14:39:06Z
last_update: 2026-04-11T14:40:25Z
date_finished: 2026-04-11T14:40:25Z
---

# T-918: Framework gap — P-011 verification gate blind to un-staged files

## Context

While closing T-906 and T-907 in this session, `.agentic-framework/bin/fw task
update T-XXX --status work-completed` ran the P-011 verification gate and passed,
because the gate executes shell commands like `test -f docs/reports/T-XXX.md`
against the working tree. The files existed on disk (they'd been written
2026-04-08) but were never `git add`ed. So the tasks closed with untracked
deliverables.

This is a blindspot because P-011's purpose is to ensure the task's deliverables
*actually ship*, not that they merely exist somewhere on the filesystem. The
current gate proves they existed in the author's working directory; that's not
the same as proving they're in the repo.

## Reproduction

```bash
# from any repo with the framework installed
cd /opt/termlink
echo "# placeholder" > /tmp/test-p011-report.md  # not in repo
cp /tmp/test-p011-report.md docs/reports/test-p011-report.md  # in repo dir, not staged
.agentic-framework/bin/fw task create --name "test p011 blindspot" --type build
# ... add to the task's ## Verification: test -f docs/reports/test-p011-report.md
.agentic-framework/bin/fw task update T-XXX --status work-completed
# gate passes, task closes, but `git status` shows docs/reports/test-p011-report.md as untracked
```

## Acceptance Criteria

### Agent
- [x] Gap description written to `docs/reports/fw-upstream-T918-p011-unstaged-blindspot.md`
- [x] Description includes reproduction, root cause, and two remediation options
- [x] Pickup prompt written for framework-side delivery (ready to push upstream via `termlink push`)

## Proposed Remediations

### Option A: Track-aware `test -f` wrapper (minimal)
Add a framework helper `fw verify exists <path>` that runs both `test -f` and
`git ls-files --error-unmatch` (or equivalent). Update framework task templates
to prefer `fw verify exists` over raw `test -f`. Back-compat: existing tasks
with raw `test -f` keep working (no enforcement break).

### Option B: Gate-level git-awareness (stronger)
Modify `update-task.sh`'s P-011 runner to detect `test -f <path>` patterns and
add an implicit `git ls-files --error-unmatch <path>` check. Surprising behavior
(silent upgrade of existing tasks) but catches the gap retroactively.

**Recommendation:** Option A. Explicit is better than implicit; gates should
announce their semantics, not upgrade them silently.

## Verification

test -f docs/reports/fw-upstream-T918-p011-unstaged-blindspot.md

## Decisions

## Updates

### 2026-04-11T14:39:06Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-918-framework-gap--p-011-verification-gate-b.md
- **Context:** Initial task creation

### 2026-04-11T14:40:25Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
