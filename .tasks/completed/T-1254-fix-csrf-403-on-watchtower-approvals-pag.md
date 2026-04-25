---
id: T-1254
name: "Fix CSRF 403 on Watchtower /approvals page — 5 hx-post forms missing _csrf_token field"
description: >
  Fix CSRF 403 on Watchtower /approvals page — 5 hx-post forms missing _csrf_token field

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-04-25T15:37:50Z
last_update: 2026-04-25T15:44:52Z
date_finished: 2026-04-25T15:44:52Z
---

# T-1254: Fix CSRF 403 on Watchtower /approvals page — 5 hx-post forms missing _csrf_token field

## Context

User reported clicking GO on T-1253 from `/approvals` returned `403 Forbidden` 7 times. RCA: T-1343/G-048 removed the `/api/*` blanket CSRF exemption (`app.py:92-111` `csrf_protect()`). Five `hx-post` forms in `web/templates/_approvals_content.html` (lines 42, 129, 164, 240, 277) never included a `_csrf_token` hidden field, so any state-changing POST from `/approvals` returns 403. Symptom in logs: `"POST /inception/T-1253/decide HTTP/1.1" 403 -` repeated. Same root cause affects: `/api/approvals/decide`, `/api/approvals/complete-batch`, `/api/task/<id>/toggle-ac`, `/api/task/<id>/complete`.

## Acceptance Criteria

### Agent
- [x] All 5 `hx-post` forms in `.agentic-framework/web/templates/_approvals_content.html` include `<input type="hidden" name="_csrf_token" value="{{ csrf_token() }}">` immediately after the opening `<form>` tag (matches the pattern from `inception_detail.html:349`).
- [x] `grep -c '_csrf_token' .agentic-framework/web/templates/_approvals_content.html` returns at least 5 (one per form).
- [x] Live POST test: `curl -s -b cookies http://localhost:3100/inception/T-1253/decide -d "_csrf_token=...&decision=defer&rationale=test"` returns 302 redirect, not 403.
- [x] Upstream mirror to `/opt/999-Agentic-Engineering-Framework` via `termlink dispatch --workdir` (Channel 1 pattern) — same 5 forms patched in upstream `web/templates/_approvals_content.html`, committed to onedev master.
- [x] Watchtower restarted (Flask dev server doesn't auto-reload without --reload flag) so live page picks up the fix.

### Human
<!-- Criteria requiring human verification (UI/UX, subjective quality). Not blocking.
     Remove this section if all criteria are agent-verifiable.
     Each criterion MUST include Steps/Expected/If-not so the human can act without guessing.
     Optionally prefix with [RUBBER-STAMP] or [REVIEW] for prioritization.
     Example:
       - [x] [REVIEW] Dashboard renders correctly
         **Steps:**
         1. Open https://example.com/dashboard in browser
         2. Verify all panels load within 2 seconds
         3. Check browser console for errors
         **Expected:** All panels visible, no console errors
         **If not:** Screenshot the broken panel and note the console error
-->

## Verification

# Shell commands that MUST pass before work-completed.
test 5 -le "$(grep -c '_csrf_token' .agentic-framework/web/templates/_approvals_content.html)"
test 5 -le "$(grep -c '_csrf_token' /opt/999-Agentic-Engineering-Framework/web/templates/_approvals_content.html)"
curl -sf http://localhost:3100/approvals -o /dev/null

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

### 2026-04-25T15:37:50Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1254-fix-csrf-403-on-watchtower-approvals-pag.md
- **Context:** Initial task creation

### 2026-04-25T15:44:52Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
