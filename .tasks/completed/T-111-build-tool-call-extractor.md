---
id: T-111
name: "Build tool call extractor"
description: >
  Python script to extract tool call metadata from session JSONL transcripts (main + sidechain). Outputs metadata-only records to stdout as JSONL. Schema from T-104.

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: [observability, tool-calls, telemetry, jsonl]
components: []
related_tasks: [T-104, T-103, T-105]
created: 2026-03-12T06:20:18Z
last_update: 2026-03-12T06:31:02Z
date_finished: 2026-03-12T06:31:02Z
---

# T-111: Build tool call extractor

## Context

Python extractor for tool call metadata from Claude Code JSONL transcripts.
Schema and design from T-104 inception. See `docs/reports/T-104-tool-call-capture-store.md`.

## Acceptance Criteria

### Agent
- [x] `agents/telemetry/extract-tool-calls.py` created and executable
- [x] Parses main JSONL: extracts tool_use + tool_result pairs with metadata-only records
- [x] Parses sidechain JSONL: extracts same from `subagents/agent-*.jsonl`
- [x] Output schema matches T-104 spec (ts, session_id, task, tool, tool_use_id, is_error, error_summary, input_size, output_size, model, tokens_in, tokens_out, is_sidechain, agent_id, cwd)
- [x] Flags: `--session ID`, `--task T-XXX`, `--project-dir DIR`
- [x] Dry-run test against real session JSONL produces valid output (985 calls, 124 errors, 646 sidechain)
- [x] Fabric card registered

## Verification

test -x agents/telemetry/extract-tool-calls.py
python3 agents/telemetry/extract-tool-calls.py --help
test -f .fabric/components/extract-tool-calls.yaml

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

### 2026-03-12T06:20:18Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /Users/dimidev32/001-projects/010-termlink/.tasks/active/T-111-build-tool-call-extractor.md
- **Context:** Initial task creation

### 2026-03-12T06:31:02Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
