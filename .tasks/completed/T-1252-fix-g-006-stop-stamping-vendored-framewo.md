---
id: T-1252
name: "Fix G-006: stop stamping vendored framework VERSION on push"
description: >
  Pre-push hook lib/hooks.sh:410-412 stamps the project's git-derived version into .agentic-framework/VERSION, overwriting the framework's own version. Remove the second stamp; only the project root VERSION should be touched. Fix in consumer + upstream framework via Channel 1.

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-04-25T13:37:06Z
last_update: 2026-04-25T13:41:00Z
date_finished: 2026-04-25T13:41:00Z
---

# T-1252: Fix G-006: stop stamping vendored framework VERSION on push

## Context

G-006 (low severity) — vendored `.agentic-framework/VERSION` should track the framework release that was vendored, not the consumer project's git-derived version. The pre-push hook in `lib/hooks.sh:410-412` overwrites it on every push. Fix: drop the conditional second stamp. Channel 1 mirror to upstream `/opt/999-Agentic-Engineering-Framework`.

## Acceptance Criteria

### Agent
- [x] `.agentic-framework/agents/git/lib/hooks.sh` no longer writes to `$PROJECT_ROOT/.agentic-framework/VERSION` in the version-stamp block. Only `$PROJECT_ROOT/VERSION` is stamped.
- [x] `/opt/999-Agentic-Engineering-Framework/agents/git/lib/hooks.sh` receives the same fix (Channel 1 upstream sync), committed and pushed to upstream `onedev` remote.
- [x] `.context/project/concerns.yaml` G-006 `mitigation_candidate` updated with closure note pointing to T-1252 + the two commits (consumer + upstream).
- [x] `grep -n "PROJECT_ROOT/.agentic-framework/VERSION" .agentic-framework/agents/git/lib/hooks.sh` returns no match.
- [x] `grep -n "PROJECT_ROOT/.agentic-framework/VERSION" /opt/999-Agentic-Engineering-Framework/agents/git/lib/hooks.sh` returns no match.

## Verification

! grep -q "PROJECT_ROOT/.agentic-framework/VERSION" /opt/termlink/.agentic-framework/agents/git/lib/hooks.sh
! grep -q "PROJECT_ROOT/.agentic-framework/VERSION" /opt/999-Agentic-Engineering-Framework/agents/git/lib/hooks.sh
grep -q "T-1252" /opt/termlink/.context/project/concerns.yaml

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

### 2026-04-25T13:37:06Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1252-fix-g-006-stop-stamping-vendored-framewo.md
- **Context:** Initial task creation

### 2026-04-25T13:41:00Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
