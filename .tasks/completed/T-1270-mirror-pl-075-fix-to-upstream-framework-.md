---
id: T-1270
name: "Mirror PL-075 fix to upstream framework + bump hooks version marker to force consumer redeploy"
description: >
  Mirror PL-075 fix to upstream framework + bump hooks version marker to force consumer redeploy

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-04-25T20:21:43Z
last_update: 2026-04-25T20:24:00Z
date_finished: 2026-04-25T20:24:00Z
---

# T-1270: Mirror PL-075 fix to upstream framework + bump hooks version marker to force consumer redeploy

## Context

T-1269 confirmed PL-075's deployment lag bug exists on /opt/999-AEF too:
its deployed `.git/hooks/pre-push` line 57 still has the buggy
`echo "$_stamped" > "$PROJECT_ROOT/.agentic-framework/VERSION"` block.
Template (`agents/git/lib/hooks.sh`) was already fixed by T-1252 but
install-hooks short-circuits on commit-msg `# VERSION=1.6` marker —
no consumer auto-redeploys.

Two-phase fix:
1. install-hooks --force on /opt/999-AEF (immediate upstream cleanup)
2. Bump commit-msg marker 1.6 → 1.7 in `lib/hooks.sh` so all OTHER
   consumers auto-redeploy on next install attempt.

## Acceptance Criteria

### Agent
- [x] /opt/999-AEF deployed pre-push no longer has `> .agentic-framework/VERSION` write
- [x] `lib/hooks.sh` commit-msg `# VERSION=` marker bumped 1.6 → 1.7
- [x] Change committed + pushed to onedev (master)

### Human
<!-- Criteria requiring human verification (UI/UX, subjective quality). Not blocking.
     Remove this section if all criteria are agent-verifiable.
     Each criterion MUST include Steps/Expected/If-not so the human can act without guessing.
     Optionally prefix with [RUBBER-STAMP] or [REVIEW] for prioritization.
     Example:
       - [ ] [REVIEW] Dashboard renders correctly
         **Steps:**
         1. Open https://example.com/dashboard in browser
         2. Verify all panels load within 2 seconds
         3. Check browser console for errors
         **Expected:** All panels visible, no console errors
         **If not:** Screenshot the broken panel and note the console error
-->

## Verification

test "$(grep -cE '> .*agentic-framework/VERSION' /opt/999-Agentic-Engineering-Framework/.git/hooks/pre-push)" = "0"
test "$(grep -cE '^# VERSION=1\.7$' /opt/999-Agentic-Engineering-Framework/agents/git/lib/hooks.sh)" -ge "1"
test "$(git -C /opt/999-Agentic-Engineering-Framework log --oneline -1 --format=%s | grep -c 'T-1270')" = "1"

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

### 2026-04-25T20:21:43Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1270-mirror-pl-075-fix-to-upstream-framework-.md
- **Context:** Initial task creation

### 2026-04-25T20:24:00Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
