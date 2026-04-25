---
id: T-1283
name: "Extend fw promote to accept PL- prefix for consumer-namespace learnings"
description: >
  Extend fw promote to accept PL- prefix for consumer-namespace learnings

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-04-25T21:47:26Z
last_update: 2026-04-25T21:52:52Z
date_finished: 2026-04-25T21:52:52Z
---

# T-1283: Extend fw promote to accept PL- prefix for consumer-namespace learnings

## Context

`fw promote` accepts only `L-` prefix (lib/promote.sh line 22 case + line 213 handler). Consumer-namespace learnings carry `PL-` prefix and cannot be promoted to practices through the CLI today. As of 2026-04-25 there are 6 PL- candidates with 3+ applications: PL-007 (11 apps — bare-command output), PL-022 (5 — fw upgrade clobbers patches), PL-011 (4), PL-053 (4 — handler build pattern), PL-021 (3), PL-040 (3).

PL-007 in particular has 11 applications and embeds a structural-enforcement insight worth permanent practice status.

Fix is two lines in lib/promote.sh: (1) `case "$subcmd"` line 22 → add `|PL-*` to the suggest|status|L-* alternation, (2) `elif subcmd.startswith('L-'):` line 213 → also accept `PL-`. The find-by-id loop already does string match against the L-prefix-or-PL-prefix.

Park as `next` for a future session that can do the upstream patch + mirror dispatch. Budget-gate at 170K critical blocked the patch attempt this session.

## Acceptance Criteria

### Agent
- [x] /opt/999-Agentic-Engineering-Framework/lib/promote.sh case at line 22 accepts `PL-*` alongside `L-*`
- [x] Handler check at line 213 accepts `subcmd.startswith('PL-')` too
- [x] Vendored copy synced
- [x] `fw promote PL-007 ...` exits 0 and created PP-012
- [x] Upstream commit pushed to onedev master (949bd3ec)

## Verification

test -n "$(grep 'PL-\*' .agentic-framework/lib/promote.sh)"
test -n "$(grep promoted_from: PL-007 .context/project/practices.yaml)"

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

### 2026-04-25T21:47:26Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1283-extend-fw-promote-to-accept-pl--prefix-f.md
- **Context:** Initial task creation

### 2026-04-25T21:47:56Z — status-update [task-update-agent]
- **Change:** status: started-work → captured
- **Change:** horizon: now → next

### 2026-04-25T21:51:34Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
- **Change:** horizon: next → now (auto-sync)

### 2026-04-25T21:52:52Z — status-update [task-update-agent]
- **Change:** horizon: now → now

### 2026-04-25T21:52:52Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
