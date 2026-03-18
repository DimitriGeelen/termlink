---
id: T-184
name: "Send improvement prompt to framework agent on remote via TermLink"
description: >
  Use TermLink cross-machine communication (TOFU TLS + hub forwarding) to inject a task-creation prompt into a framework Claude session running on 192.168.10.107. Tests T-178 + T-182 end-to-end for the real use case.

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: [cross-machine, termlink, framework]
components: []
related_tasks: []
created: 2026-03-18T23:16:54Z
last_update: 2026-03-18T23:16:54Z
date_finished: null
---

# T-184: Send improvement prompt to framework agent on remote via TermLink

## Context

<!-- One sentence for small tasks. Link to design docs for substantial ones. -->

## Acceptance Criteria

### Agent
- [x] tofu_test example extended with inject + hub routing capability
- [x] Prompt successfully injected into remote session (681 bytes via command.inject)
- [x] tofu_test example compiles and runs

## Verification

test -f crates/termlink-session/examples/tofu_test.rs

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

### 2026-03-18T23:16:54Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /Users/dimidev32/001-projects/010-termlink/.tasks/active/T-184-send-improvement-prompt-to-framework-age.md
- **Context:** Initial task creation
