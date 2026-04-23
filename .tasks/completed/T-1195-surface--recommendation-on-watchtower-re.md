---
id: T-1195
name: "Surface ## Recommendation on Watchtower /review/T-XXX page (fw task review)"
description: >
  review.py (176 LoC) parses Human ACs, pending Tier 0, artifacts — but never touches the ## Recommendation section from the task body. review.html (284 LoC) has no recommendation rendering block. Result: human scanning a QR to review an inception sees the approval button but not the recommendation they're approving. Separate from T-939 (/approvals page — already shows recs). Fix: add _parse_recommendation helper in review.py, pass to template, render above the Tier 0 approval block. Mirror upstream via Channel 1.

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: [framework, watchtower, ui, structural-fix]
components: []
related_tasks: [T-939, T-1192, T-1194]
created: 2026-04-22T22:04:41Z
last_update: 2026-04-22T22:08:18Z
date_finished: 2026-04-22T22:08:18Z
---

# T-1195: Surface ## Recommendation on Watchtower /review/T-XXX page (fw task review)

## Context

`fw task review T-XXX` opens Watchtower's `/review/T-XXX` page — the mobile-first card the human uses to decide. For inception tasks, the whole point is seeing the recommendation + rationale before approving. Currently the page shows task name, Human ACs, Tier 0 approval buttons, and research artifact links — but **not** the `## Recommendation` section from the task body. The human scans the QR, sees Approve/Reject, and has to click through to the artifact link to find what they're approving. Verified via grep: `review.html` contains 0 references to "recommendation".

## Acceptance Criteria

### Agent
- [x] `review.py` has helper `_parse_recommendation(body_text)` (vendored line 75; smoke test extracted 1200 chars from T-1194 body)
- [x] `review()` route passes `recommendation` to `render_template` (vendored line 150)
- [x] `review.html` renders `<section class="recommendation-block">` above Tier 0 approval block (vendored lines 248-260)
- [x] Empty-path fallback renders "No recommendation recorded yet" with artifact link (jinja render smoke confirmed)
- [x] Upstream mirrored via Channel 1: commit `b74a1a3e` on `/opt/999-Agentic-Engineering-Framework`, pushed to onedev master (`f398d004..b74a1a3e`)
- [x] Both review.py parse clean via `ast.parse` (verified)

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

grep -q "_parse_recommendation" /opt/termlink/.agentic-framework/web/blueprints/review.py
grep -q "recommendation" /opt/termlink/.agentic-framework/web/templates/review.html
python3 -c "import ast; ast.parse(open('/opt/termlink/.agentic-framework/web/blueprints/review.py').read())"

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

### 2026-04-22T22:04:41Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1195-surface--recommendation-on-watchtower-re.md
- **Context:** Initial task creation

### 2026-04-22T22:08:18Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
