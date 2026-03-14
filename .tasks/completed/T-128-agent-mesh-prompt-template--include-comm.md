---
id: T-128
name: "Agent mesh prompt template — include commit instructions and output format"
description: >
  Standardized prompt template for mesh workers that includes: commit your work, output format, cargo path, error handling. From T-123 retrospective.

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: [agent-mesh]
components: []
related_tasks: []
created: 2026-03-13T10:05:23Z
last_update: 2026-03-14T12:08:35Z
date_finished: 2026-03-14T12:08:35Z
---

# T-128: Agent mesh prompt template — include commit instructions and output format

## Context

From T-123 retrospective: agents dispatched via mesh didn't commit, used wrong cargo path, returned inconsistent output. A prompt template wraps the user's task prompt with standard instructions.

## Acceptance Criteria

### Agent
- [x] Prompt template file exists at `agents/mesh/prompt-template.sh`
- [x] Template includes: commit instructions, cargo path, output format, error handling
- [x] dispatch.sh wraps user prompt through the template before passing to agent-wrapper.sh
- [x] Template is shell-sourceable (function that takes prompt and returns wrapped prompt)

## Verification

test -f agents/mesh/prompt-template.sh
grep -q 'commit' agents/mesh/prompt-template.sh
grep -q 'cargo' agents/mesh/prompt-template.sh
# dispatch.sh uses the template
grep -q 'prompt-template\|wrap_prompt' agents/mesh/dispatch.sh
bash -n agents/mesh/prompt-template.sh

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

### 2026-03-13T10:05:23Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /Users/dimidev32/001-projects/010-termlink/.tasks/active/T-128-agent-mesh-prompt-template--include-comm.md
- **Context:** Initial task creation

### 2026-03-14T12:04:47Z — status-update [task-update-agent]
- **Change:** horizon: later → now

### 2026-03-14T12:04:48Z — status-update [task-update-agent]
- **Change:** status: captured → started-work

### 2026-03-14T12:08:35Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
