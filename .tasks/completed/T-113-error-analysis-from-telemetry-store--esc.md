---
id: T-113
name: "Error analysis from telemetry store — escalation ladder auto-detection"
description: >
  Build error analysis on T-111 telemetry data. Query tool-calls.jsonl for is_error records, classify by tool/frequency/pattern, map to escalation ladder (A-D), output actionable recommendations. Delivers T-103 vision.

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: [antifragility, error-escalation, telemetry, healing]
components: []
related_tasks: [T-103, T-104, T-111, T-112]
created: 2026-03-12T06:41:54Z
last_update: 2026-03-12T06:51:07Z
date_finished: 2026-03-12T06:51:07Z
---

# T-113: Error analysis from telemetry store — escalation ladder auto-detection

## Context

Builds on T-111 telemetry store (`.context/telemetry/tool-calls.jsonl`). Delivers
T-103's deferred vision: proactive error pattern detection via escalation ladder mapping.

## Acceptance Criteria

### Agent
- [x] `agents/telemetry/analyze-errors.py` created and executable
- [x] Reads `.context/telemetry/tool-calls.jsonl` and filters `is_error: true`
- [x] Groups errors by tool name + error_summary pattern (17 classifiers)
- [x] Maps frequency to escalation ladder level (A/B/C/D) — 6D, 2C, 6B, 3A in test
- [x] Outputs actionable report to terminal + JSON mode
- [x] Tested: 1,008 calls, 126 errors (12%), 17 patterns detected

## Verification

test -x agents/telemetry/analyze-errors.py
python3 agents/telemetry/analyze-errors.py --help

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

### 2026-03-12T06:41:54Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /Users/dimidev32/001-projects/010-termlink/.tasks/active/T-113-error-analysis-from-telemetry-store--esc.md
- **Context:** Initial task creation

### 2026-03-12T06:51:07Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
