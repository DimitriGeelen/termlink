---
id: T-246
name: "Interactive session picker — prompt when no target given"
description: >
  Add shared session picker utility: when target-requiring interactive commands (attach, mirror, stream, ping, status, watch, topics, output, interact, inject, kv, events, wait, remote ping, remote status) are run without a target and stdin is TTY, list sessions numbered, auto-select if 1, prompt if 2+. Works for local and remote (--hub) sessions. Make target Optional in clap for these commands, call picker before dispatch.

status: work-completed
workflow_type: build
owner: human
horizon: now
tags: [ux, cli, T-245]
components: []
related_tasks: [T-245, T-244]
created: 2026-03-23T15:54:57Z
last_update: 2026-03-23T16:13:58Z
date_finished: 2026-03-23T16:13:58Z
---

# T-246: Interactive session picker — prompt when no target given

## Context

Interactive session picker when no target given. Derived from T-245 inception (GO).

## Acceptance Criteria

### Agent
- [x] resolve_target() utility in util.rs for local sessions
- [x] resolve_remote_target() utility in remote.rs for hub sessions
- [x] ~15 local commands accept optional target (attach, mirror, stream, ping, status, output, inject, interact, events, wait)
- [x] 4 remote commands accept optional session (status, inject, send-file, exec)
- [x] Auto-select when 1 session, numbered prompt when 2+
- [x] Error when no sessions or stdin not a TTY
- [x] All 297 workspace tests pass

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

/Users/dimidev32/.cargo/bin/cargo test --package termlink
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

### 2026-03-23T15:54:57Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /Users/dimidev32/001-projects/010-termlink/.tasks/active/T-246-interactive-session-picker--prompt-when-.md
- **Context:** Initial task creation

### 2026-03-23T15:58:39Z — status-update [task-update-agent]
- **Change:** status: captured → started-work

### 2026-03-23T16:13:58Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
