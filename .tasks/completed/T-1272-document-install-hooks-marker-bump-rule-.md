---
id: T-1272
name: "Document install-hooks marker-bump rule in lib/hooks.sh (PL-078)"
description: >
  Document install-hooks marker-bump rule in lib/hooks.sh (PL-078)

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-04-25T20:32:50Z
last_update: 2026-04-25T20:40:50Z
date_finished: 2026-04-25T20:40:50Z
---

# T-1272: Document install-hooks marker-bump rule in lib/hooks.sh (PL-078)

## Context

PL-078 (T-1270) found that install-hooks short-circuits on the commit-msg
`# VERSION=` marker alone — a fix to pre-push or post-commit content
won't deploy unless the commit-msg marker also bumps. Without an inline
docstring, future framework editors will hit the same trap T-1252 hit
(fix sat dormant for days). Add a comment block right above the
commit-msg `# VERSION=` line in `agents/git/lib/hooks.sh` documenting
the rule. Mirror only — fix lives in /opt/999-AEF.

## Acceptance Criteria

### Agent
- [x] `lib/hooks.sh` upstream contains a comment near the commit-msg `# VERSION=` line referencing PL-078 + the bump-on-any-hook-change rule
- [x] Comment is BEFORE the HOOK_EOF heredoc-end so it stays in the source file (not embedded into the deployed hook)
- [x] Change committed + pushed to onedev (master) — pushed at 2026-04-25T20:43Z (after ~9min OneDev outage); refs c9086ea4..29c10012

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

grep -q 'PL-078' /opt/999-Agentic-Engineering-Framework/agents/git/lib/hooks.sh
test -n "$(git -C /opt/999-Agentic-Engineering-Framework log --oneline -5 | grep 'T-1272')"

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

### 2026-04-25T20:32:50Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1272-document-install-hooks-marker-bump-rule-.md
- **Context:** Initial task creation

### 2026-04-25T20:40:50Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
