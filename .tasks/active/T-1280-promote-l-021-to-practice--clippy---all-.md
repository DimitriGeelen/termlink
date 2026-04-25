---
id: T-1280
name: "Promote L-021 to practice — clippy --all-targets in CI catches await_holding_lock"
description: >
  Promote L-021 to practice — clippy --all-targets in CI catches await_holding_lock

status: started-work
workflow_type: refactor
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-04-25T21:20:52Z
last_update: 2026-04-25T21:20:52Z
date_finished: null
---

# T-1280: Promote L-021 to practice — clippy --all-targets in CI catches await_holding_lock

## Context

L-021 (clippy --all-targets catches await_holding_lock) — 3 documented applications. Promote to D2 Reliability practice: catching deadlock-prone async patterns at CI time prevents production-load-only failures.

## Acceptance Criteria

### Agent
- [x] `fw promote L-021 --name "..." --directive D2` exits 0 (created PP-009)
- [x] PP-009 entry exists with `promoted_from: L-021`
- [x] L-021 `application` field references PP-009
- [x] Side-discovery: PP-008 (T-1277 duplicate of L-027 → PP-001) reverted, L-027 application back-filled to PP-001

## Verification

test -n "$(grep 'promoted_from: L-021' .context/project/practices.yaml)"
test -n "$(grep -A6 '^- id: L-021' .context/project/learnings.yaml | grep 'application:' | grep -oE 'PP-[0-9]+')"

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

### 2026-04-25T21:20:52Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1280-promote-l-021-to-practice--clippy---all-.md
- **Context:** Initial task creation
