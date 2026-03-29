---
id: T-770
name: "Fix update-homebrew-sha.sh to handle 4 platform variants (add Linux aarch64)"
description: >
  Fix update-homebrew-sha.sh to handle 4 platform variants (add Linux aarch64)

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-03-29T23:12:15Z
last_update: 2026-03-29T23:13:32Z
date_finished: 2026-03-29T23:13:32Z
---

# T-770: Fix update-homebrew-sha.sh to handle 4 platform variants (add Linux aarch64)

## Context

The script extracts 3 SHA256 hashes (darwin-aarch64, darwin-x86_64, linux-x86_64) but the Homebrew formula now has 4 platform variants after T-760 added Linux aarch64.

## Acceptance Criteria

### Agent
- [x] Script extracts SHA256 for all 4 platforms: darwin-aarch64, darwin-x86_64, linux-x86_64, linux-aarch64
- [x] Script validates all 4 hashes are present before proceeding
- [x] Script updates all 4 platform entries in the formula via sed
- [x] Script runs without errors: `bash -n scripts/update-homebrew-sha.sh`

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

bash -n scripts/update-homebrew-sha.sh
grep -q "linux-aarch64" scripts/update-homebrew-sha.sh

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

### 2026-03-29T23:12:15Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-770-fix-update-homebrew-shash-to-handle-4-pl.md
- **Context:** Initial task creation

### 2026-03-29T23:13:32Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
