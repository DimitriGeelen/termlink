---
id: T-939
name: "Approvals page missing recommendation display — RCA + fix"
description: >
  Approvals page missing recommendation display — RCA + fix

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-04-12T07:27:33Z
last_update: 2026-04-12T07:31:39Z
date_finished: null
---

# T-939: Approvals page missing recommendation display — RCA + fix

## Context

Watchtower /approvals page shows inception tasks pending GO/NO-GO but doesn't display the agent's recommendation or argumentation. The `rationale_hint` is only pre-filled into the textarea — the reviewer sees no visible recommendation before deciding.

**RCA (two root causes):**
1. **UI gap (approvals template):** `_approvals_content.html` lines 63-106 never render `rationale_hint` as visible text. It's only stuffed into the textarea pre-fill. A reviewer opening the page sees task name + problem excerpt + radio buttons — no recommendation.
2. **Data gap (16/24 tasks):** 16 inception tasks have empty `## Recommendation` sections (placeholder HTML comments). These predate the framework enforcing recommendation writing (T-974). Even with a perfect UI, these would show "No recommendation yet."

**Fix plan:**
- A: Add recommendation display block to `_approvals_content.html` (GO/NO-GO badge + argumentation text above the form)
- B: Framework-side bug report for the data gap (pickup to fw-agent)

## Acceptance Criteria

### Agent
- [x] `_approvals_content.html` shows recommendation text prominently above the decision form for each inception task
- [x] When recommendation is empty, shows "No recommendation yet — review task file" with link
- [x] When recommendation exists, shows GO/NO-GO/DEFER badge + truncated argumentation
- [x] Approvals page loads without error after template change
- [x] Pickup task created and sent to framework agent via termlink (P-009 delivered to /opt/999-Agentic-Engineering-Framework/.context/pickup/inbox/)

### Human
- [ ] [REVIEW] Verify approvals page shows recommendations visually
  **Steps:**
  1. Open http://192.168.10.107:3002/approvals
  2. Check inception decision cards — each should show recommendation or "No recommendation yet"
  3. Cards with recommendations (T-908, T-930) should show GO badge + rationale preview
  **Expected:** Recommendation visible above the decision form, not buried in textarea
  **If not:** Check browser console for template errors

## Verification

curl -sf http://localhost:3002/approvals | grep -q "recommendation-display"
python3 -c "import yaml; yaml.safe_load(open('.context/cron-registry.yaml'))"

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

### 2026-04-12T07:27:33Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-939-approvals-page-missing-recommendation-di.md
- **Context:** Initial task creation
