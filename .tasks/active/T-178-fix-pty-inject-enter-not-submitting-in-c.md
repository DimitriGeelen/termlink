---
id: T-178
name: "Fix pty inject Enter not submitting in Claude Code TUI"
description: >
  pty inject sends text+Enter as one write. Ink TUI needs Enter (0x0D) as separate write with small delay. Root cause: batched write means ink sees multi-char chunk, not a keypress. Fix: split text write and Enter into two separate pty.write() calls. Also check ICRNL termios flag. See docs/reports/T-163-cross-machine-rca-findings.md for full RCA. Related: Claude Code issue #15553, ink useInput batching.

status: captured
workflow_type: build
owner: agent
horizon: now
tags: [bug, cli, inject, pty]
components: []
related_tasks: [T-137, T-156, T-163, T-177]
created: 2026-03-18T22:19:38Z
last_update: 2026-03-18T22:19:38Z
date_finished: null
---

# T-178: Fix pty inject Enter not submitting in Claude Code TUI

## Context

<!-- One sentence for small tasks. Link to design docs for substantial ones. -->

## Acceptance Criteria

### Agent
<!-- Criteria the agent can verify (code, tests, commands). P-010 gates on these. -->
- [ ] [First criterion]
- [ ] [Second criterion]

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

### 2026-03-18T22:19:38Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /Users/dimidev32/001-projects/010-termlink/.tasks/active/T-178-fix-pty-inject-enter-not-submitting-in-c.md
- **Context:** Initial task creation
