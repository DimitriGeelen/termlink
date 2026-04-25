---
id: T-1279
name: "Fix fw promote dedup: check promoted_from not derived_from (PL-083)"
description: >
  Fix fw promote dedup: check promoted_from not derived_from (PL-083)

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-04-25T21:18:02Z
last_update: 2026-04-25T21:18:02Z
date_finished: null
---

# T-1279: Fix fw promote dedup: check promoted_from not derived_from (PL-083)

## Context

PL-083 — fw promote dedup uses `derived_from` (which holds the directive D1/D2/...) instead of `promoted_from` (which holds the L-XXX origin). Practices from `fw promote` always set derived_from=Dx, so dedup never matches. Result: an already-promoted L-XXX with TBD application field gets re-suggested and re-promoted. Hit on T-1278 (L-007 → duplicate PP-009).

Fix: at lib/promote.sh:62-72, also collect `promoted_from` into the `promoted_ids` set. One-place change fixes both `suggest` (line 138-149) and the explicit-promote dedup gate (line 249-251).

## Acceptance Criteria

### Agent
- [ ] /opt/999-Agentic-Engineering-Framework/lib/promote.sh patched
- [ ] Vendored /opt/termlink/.agentic-framework/lib/promote.sh updated to match
- [ ] `fw promote suggest` no longer lists L-007 in output
- [ ] `fw promote L-007 ...` exits with "already promoted" message
- [ ] Upstream commit pushed to onedev master

## Verification

test ! -z "$(grep -E 'promoted_from' .agentic-framework/lib/promote.sh | grep -v '^\s*#')"
test -z "$(.agentic-framework/bin/fw promote suggest 2>&1 | grep -E '^\s*L-007 ')"
.agentic-framework/bin/fw promote L-007 --name x --directive D2 2>&1 | grep -q "already promoted"

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

### 2026-04-25T21:18:02Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1279-fix-fw-promote-dedup-check-promotedfrom-.md
- **Context:** Initial task creation
