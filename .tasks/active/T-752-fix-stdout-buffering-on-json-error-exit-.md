---
id: T-752
name: "Fix stdout buffering on JSON error exit — 222 process::exit calls lose output when piped"
description: >
  Fix stdout buffering on JSON error exit — 222 process::exit calls lose output when piped

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-03-29T18:39:09Z
last_update: 2026-03-29T18:39:09Z
date_finished: null
---

# T-752: Fix stdout buffering on JSON error exit — 222 process::exit calls lose output when piped

## Context

`println!()` followed by `process::exit(1)` loses output when stdout is piped (block-buffered). Discovered via failing `cli_broadcast_no_hub_json` test. Fix: create `json_error_exit()` helper that flushes before exit, replace all 222 occurrences.

## Acceptance Criteria

### Agent
- [ ] `json_error_exit()` helper exists in commands/mod.rs with stdout flush
- [ ] All `println!(...); process::exit(1)` replaced with `json_error_exit()`
- [ ] `cargo clippy --workspace -- -D warnings` passes
- [ ] `cargo test --workspace` passes
- [ ] Broadcast no-hub JSON error test passes (was previously broken)

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

### 2026-03-29T18:39:09Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-752-fix-stdout-buffering-on-json-error-exit-.md
- **Context:** Initial task creation
