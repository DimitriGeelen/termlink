---
id: T-579
name: "Add --json output to agent ask command"
description: >
  Add --json output to agent ask command

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: [crates/termlink-cli/src/cli.rs, crates/termlink-cli/src/commands/agent.rs, crates/termlink-cli/src/main.rs]
related_tasks: []
created: 2026-03-28T15:39:19Z
last_update: 2026-03-28T15:40:54Z
date_finished: 2026-03-28T15:40:54Z
---

# T-579: Add --json output to agent ask command

## Context

<!-- One sentence for small tasks. Link to design docs for substantial ones. -->

## Acceptance Criteria

### Agent
- [x] --json flag added to AgentAction::Ask in cli.rs
- [x] main.rs passes json to cmd_agent_ask
- [x] cmd_agent_ask outputs structured JSON on success and error
- [x] cargo build succeeds

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

### 2026-03-28T15:39:19Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-579-add---json-output-to-agent-ask-command.md
- **Context:** Initial task creation

### 2026-03-28T15:40:54Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
