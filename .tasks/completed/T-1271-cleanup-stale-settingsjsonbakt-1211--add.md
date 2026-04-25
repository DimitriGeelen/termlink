---
id: T-1271
name: "Cleanup stale settings.json.bak.T-1211 + add .bak.* to .gitignore (prevent future accidental commits)"
description: >
  Cleanup stale settings.json.bak.T-1211 + add .bak.* to .gitignore (prevent future accidental commits)

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-04-25T20:26:31Z
last_update: 2026-04-25T20:29:28Z
date_finished: 2026-04-25T20:29:28Z
---

# T-1271: Cleanup stale settings.json.bak.T-1211 + add .bak.* to .gitignore (prevent future accidental commits)

## Context

`.claude/settings.json.bak.T-1211` left over from T-1211's hook install
(pre-T-1211 snapshot). Existing `.gitignore` line 24 only matches the
exact name `.claude/settings.json.bak`, not the `.bak.T-XXX` suffix
variant. File showed in `git status` as untracked, eligible for accidental
commit. Add glob pattern + remove file.

## Acceptance Criteria

### Agent
- [x] `.gitignore` contains pattern matching `.claude/settings.json.bak.*`
- [x] `.claude/settings.json.bak.T-1211` no longer exists
- [x] `git check-ignore .claude/settings.json.bak.X` returns 0 for arbitrary suffix

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

grep -qE '^\.claude/settings\.json\.bak\.\*$' .gitignore
test ! -e .claude/settings.json.bak.T-1211
touch .claude/settings.json.bak.test && git check-ignore .claude/settings.json.bak.test >/dev/null && rm .claude/settings.json.bak.test

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

### 2026-04-25T20:26:31Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1271-cleanup-stale-settingsjsonbakt-1211--add.md
- **Context:** Initial task creation

### 2026-04-25T20:29:28Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
