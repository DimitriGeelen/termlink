---
id: T-1277
name: "Promote L-027 to practice — consumer hooks pass PROJECT_ROOT explicitly"
description: >
  Promote L-027 to practice — consumer hooks pass PROJECT_ROOT explicitly

status: started-work
workflow_type: refactor
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-04-25T21:13:07Z
last_update: 2026-04-25T21:13:07Z
date_finished: null
---

# T-1277: Promote L-027 to practice — consumer hooks pass PROJECT_ROOT explicitly

## Context

L-027 (Consumer project hooks must pass PROJECT_ROOT explicitly) has 15 documented applications, the highest in the registry. Promote to a permanent practice (PP-XXX) under D2 (Reliability) so it's enforced by audit rather than rediscovered task-by-task.

## Acceptance Criteria

### Agent
- [ ] `fw promote L-027 --name "..." --directive D2` exits 0
- [ ] New PP-XXX entry exists in `.context/project/practices.yaml`
- [ ] L-027 `application` field updated with promotion reference

## Verification

# PP entry exists referencing L-027
test -n "$(grep -B1 'L-027' .context/project/practices.yaml | grep -E '^\s*-\s*id:\s*PP-')"
# L-027 application field is no longer TBD
test -n "$(awk '/^- id: L-027/,/^- id: /' .context/project/learnings.yaml | grep 'application:' | head -1 | grep -v 'TBD')"

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

### 2026-04-25T21:13:07Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1277-promote-l-027-to-practice--consumer-hook.md
- **Context:** Initial task creation
