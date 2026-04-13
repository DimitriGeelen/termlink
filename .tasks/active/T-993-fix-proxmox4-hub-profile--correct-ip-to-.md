---
id: T-993
name: "Fix remote hub profile — rename proxmox4 to ring20-management on .109"
description: >
  Proxmox4 profile had wrong IP (.122) and wrong name. Renamed to ring20-management,
  corrected to 192.168.10.109:9100, updated secret and TOFU fingerprint.

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-04-13T06:39:57Z
last_update: 2026-04-13T06:39:57Z
date_finished: null
---

# T-993: Fix proxmox4 hub profile — correct IP to .109

## Context

<!-- One sentence for small tasks. Link to design docs for substantial ones. -->

## Acceptance Criteria

### Agent
<!-- Criteria the agent can verify (code, tests, commands). P-010 gates on these. -->
- [x] Hub profile renamed from proxmox4 to ring20-management pointing to 192.168.10.109:9100
- [x] `termlink remote ping ring20-management` succeeds (86ms)
- [x] T-991 pickup delivered to ring20-manager session via inject + file send

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

# Shell commands that MUST pass before work-completed. One per line.
termlink remote ping ring20-management 2>&1 | grep -q "PONG"

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

### 2026-04-13T06:39:57Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-993-fix-proxmox4-hub-profile--correct-ip-to-.md
- **Context:** Initial task creation
