---
id: T-1156
name: "RCA + remediate recurring bin/fw path mistake in agent output"
description: >
  RCA + remediate recurring bin/fw path mistake in agent output

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-04-20T13:43:29Z
last_update: 2026-04-20T13:43:29Z
date_finished: null
---

# T-1156: RCA + remediate recurring bin/fw path mistake in agent output

## Context

Agent keeps giving users `cd /opt/termlink && bin/fw ...` commands that fail with `No such file or directory`. Reproduced 3+ times in this session alone, most recently with T-1155 inception decide. Root cause traced to two layers:

1. **Framework layer:** CLAUDE.md rule at line 715 literally says "Use `bin/fw` not `fw`". That rule is correct inside the framework repo (where `bin/fw` is the entry point) but wrong for consumer projects like termlink where the vendored path is `.agentic-framework/bin/fw`. Template source: `.agentic-framework/lib/templates/claude-project.md:621`. `lib/init.sh:580` already has logic to pick the right path but the template copies unchanged.

2. **Agent layer:** I obey CLAUDE.md rules literally. Fixing the rule fixes the root cause.

This is a close cousin of T-915 F2 (task-review prompt shows wrong consumer path). Same class: framework-authored text that doesn't adapt to consumer context.

## Acceptance Criteria

### Agent
- [x] CLAUDE.md line 715 rule clarified to say `.agentic-framework/bin/fw` for consumer projects, with example
- [x] Memory entry saved documenting the rule (feedback type) so it survives CLAUDE.md drift
- [x] Pickup envelope drafted for framework agent documenting the template bug (source: `lib/templates/claude-project.md:621`) and proposed fix (substitute path in `fw init` like `lib/init.sh:580` already does for other emissions)
- [x] Pickup successfully sent via termlink to the framework project
- [x] Verification: grep confirms no `bin/fw` (standalone, not `.agentic-framework/bin/fw`) as a runnable-command hint in the updated CLAUDE.md line

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
# Lines starting with # are comments (skipped). Empty lines ignored.
# The completion gate runs each command — if any exits non-zero, completion is blocked.
grep -q "agentic-framework/bin/fw" /opt/termlink/CLAUDE.md
! grep -qE "^3\. \*\*Use .bin/fw. not" /opt/termlink/CLAUDE.md
test -f /opt/999-Agentic-Engineering-Framework/.context/pickup/inbox/P-T-1156-bug-report.yaml
test -f /root/.claude/projects/-opt-termlink/memory/feedback_fw_path_consumer.md

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

### 2026-04-20T13:43:29Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1156-rca--remediate-recurring-binfw-path-mist.md
- **Context:** Initial task creation
