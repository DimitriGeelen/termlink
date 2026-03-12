---
id: T-112
name: "Integrate tool call extractor into session lifecycle"
description: >
  Hook extract-tool-calls.py into PreCompact/session-end. Append to .context/telemetry/tool-calls.jsonl. Align retention with T-110.

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: [observability, telemetry, hooks, lifecycle]
components: []
related_tasks: [T-104, T-111, T-110]
created: 2026-03-12T06:31:11Z
last_update: 2026-03-12T06:31:11Z
date_finished: null
---

# T-112: Integrate tool call extractor into session lifecycle

## Context

Hook T-111's `extract-tool-calls.py` into the session lifecycle so tool call data
is automatically captured. Design from T-104 inception: batch extraction at PreCompact.

## Acceptance Criteria

### Agent
- [x] PreCompact hook script created (`capture-on-compact.sh`) — appends to `.context/telemetry/tool-calls.jsonl`
- [x] `.context/telemetry/` directory created if missing (mkdir -p in script)
- [x] Extraction runs silently (stderr to /dev/null, non-blocking — exits 0 on failure)
- [x] Manual extraction: `python3 agents/telemetry/extract-tool-calls.py --include-sidechains >> .context/telemetry/tool-calls.jsonl`
- [x] Integration tested: 1,008 valid JSONL records produced from current session
- [x] `.gitignore` updated to exclude `.context/telemetry/` (raw data, not committed)

## Verification

test -d .context/telemetry
python3 agents/telemetry/extract-tool-calls.py --help

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

### 2026-03-12T06:31:11Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /Users/dimidev32/001-projects/010-termlink/.tasks/active/T-112-integrate-tool-call-extractor-into-sessi.md
- **Context:** Initial task creation
