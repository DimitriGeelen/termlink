---
id: T-739
name: "Add ok:true to tag update, kv set, kv get, kv list, kv del JSON success responses"
description: >
  Add ok:true to tag update, kv set, kv get, kv list, kv del JSON success responses

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-03-29T13:23:55Z
last_update: 2026-03-29T13:25:38Z
date_finished: 2026-03-29T13:25:38Z
---

# T-739: Add ok:true to tag update, kv set, kv get, kv list, kv del JSON success responses

## Context

Add `"ok": true` wrapping to raw RPC result JSON outputs in metadata.rs (tag update, kv set/get/list/del).

## Acceptance Criteria

### Agent
- [x] `cmd_tag` JSON success includes `"ok": true`
- [x] `kv set` JSON success includes `"ok": true`
- [x] `kv get` JSON success includes `"ok": true`
- [x] `kv list` JSON success includes `"ok": true`
- [x] `kv del` JSON success includes `"ok": true`
- [x] Project compiles with `cargo build`

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

<!-- Shell commands that MUST pass before work-completed. One per line.
     Lines starting with # are comments. Empty lines ignored.
     The completion gate runs each command — if any exits non-zero, completion is blocked.
     Examples:
       python3 -c "import yaml; yaml.safe_load(open('path/to/file.yaml'))"
       curl -sf http://localhost:3000/page
       grep -q "expected_string" output_file.txt
-->

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

### 2026-03-29T13:23:55Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-739-add-oktrue-to-tag-update-kv-set-kv-get-k.md
- **Context:** Initial task creation

### 2026-03-29T13:25:38Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
