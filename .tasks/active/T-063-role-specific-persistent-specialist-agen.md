---
id: T-063
name: "Role-specific persistent specialist agents — reviewer, tester, infra, git"
description: >
  Role-specific persistent specialist agents — reviewer, tester, infra, git

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-03-10T08:11:49Z
last_update: 2026-03-10T08:11:49Z
date_finished: null
---

# T-063: Role-specific persistent specialist agents — reviewer, tester, infra, git

## Context

Extends T-062's generic specialist-watcher with role-specific system prompts, tool permissions, and role identity in events. Builds toward permanent specialist agent fleet.

## Acceptance Criteria

### Agent
- [x] Role-specific system prompts exist for reviewer, tester, documenter, git-committer
- [x] Role-aware watcher (role-watcher.sh) loads role prompts and sets role-appropriate tools
- [x] Level 5 e2e test exists and is executable
- [x] Level 5 e2e test passes — 3 role-aware specialists complete with role-tagged events

### Human
- [ ] [RUBBER-STAMP] Review role prompt quality
  **Steps:**
  1. Read `tests/e2e/role-prompts/*.md`
  2. Verify each prompt has clear domain expertise and constraints
  **Expected:** Prompts are focused, concise, and appropriate for each role
  **If not:** Note which prompts need improvement

## Verification

test -f tests/e2e/role-prompts/reviewer.md
test -f tests/e2e/role-prompts/tester.md
test -f tests/e2e/role-prompts/documenter.md
test -f tests/e2e/role-prompts/git-committer.md
test -x tests/e2e/role-watcher.sh
test -x tests/e2e/level5-role-specialists.sh

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

### 2026-03-10T08:11:49Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /Users/dimidev32/001-projects/010-termlink/.tasks/active/T-063-role-specific-persistent-specialist-agen.md
- **Context:** Initial task creation
