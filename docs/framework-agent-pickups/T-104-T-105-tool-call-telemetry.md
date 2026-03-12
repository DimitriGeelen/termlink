# Framework Agent Pickup: Tool Call Telemetry — Capture Store + Watchtower Reporting

> Tasks: T-104, T-105 | Generated: 2026-03-12

## What You Need To Do

Create a PR in the agentic-engineering-framework repo on OneDev to add tool call
telemetry: a capture store (data layer) and reporting integration with Watchtower
(the framework's Flask web UI).

This is two tightly coupled deliverables in one PR:
1. **Capture store** — extract and persist tool call data from Claude Code JSONL transcripts
2. **Reporting** — surface tool call stats in Watchtower and terminal

## OneDev Location

- Instance: `onedev.docker.ring20.geelenandcompany.com`
- Repo: `agentic-engineering-framework`
- Branch: create `feature/tool-call-telemetry`

## Problem

No persistent record of tool call activity exists across sessions. Each session's
tool calls live in the Claude Code JSONL transcript (`~/.claude/projects/*/`), but
that transcript is session-scoped and not aggregated. After context compaction or
session end, the data is effectively inaccessible.

Without telemetry:
- No visibility into which tools are called most or fail most
- No error trend detection across sessions
- No way to correlate tool usage with context budget burn
- No data to feed the Error Escalation Ladder (Level D improvements)

## Design Decisions (validated in 010-termlink)

### Storage: Append-only JSONL

- File: `.context/telemetry/tool-calls.jsonl`
- One JSON record per tool call
- ~350 bytes/record, ~30 MB/month at one session/day
- Simple, queryable with Python, no dependencies

### Schema (per record)

```json
{
  "tool": "Bash",
  "timestamp": "2026-03-12T15:30:00Z",
  "session_id": "S-2026-0312-1400",
  "task": "T-117",
  "is_error": false,
  "error_content": null,
  "input_summary": "git status --short",
  "token_count": 45000
}
```

- Tool inputs: metadata only (name + truncated summary), NOT full content
- Errors: always store full error output (diagnostic value outweighs size)
- Token count: from budget state at time of call

### Capture timing: Batch at PreCompact

- Extract from JSONL at PreCompact hook (session end / manual compact)
- No per-tool-call overhead during the session
- Python extractor script reads transcript, appends to JSONL store

### Reporting: Terminal + Watchtower

- `fw tool-stats` — terminal command for quick checks
- Watchtower page — filter by session, tool, error/success, time range
- Handover integration — add tool call summary to handover output

## Files to Create

### Capture layer

**`agents/telemetry/extract-tool-calls.py`**
Python script that:
1. Reads the current session JSONL transcript
2. Extracts tool_use / tool_result event pairs
3. Builds records matching the schema above
4. Appends to `.context/telemetry/tool-calls.jsonl`

**`agents/telemetry/capture-on-compact.sh`**
PreCompact hook wrapper. Calls extract-tool-calls.py with the current session's
JSONL path. Register in `.claude/settings.json` as a PreCompact hook.

### Reporting layer

**`agents/telemetry/tool-stats.py`**
Terminal reporter. Reads `.context/telemetry/tool-calls.jsonl`, produces:
- Total calls, error count, error rate
- Top tools by usage
- Top error tools
- Per-session breakdown
- Flags: `--session S-XXX`, `--tool Bash`, `--errors-only`, `--last N`

**Watchtower integration**
Add a `/telemetry` or `/tool-stats` route to the Watchtower Flask app:
- Table view: filterable by session, tool, error/success
- Drill-down: click session → see all calls; click error → see full error content
- Summary cards: total calls, error rate, most-used tool, trend vs previous sessions

### Hook registration

Add to `.claude/settings.json` under PreCompact hooks:
```json
{
  "matcher": "",
  "hooks": [{
    "type": "command",
    "command": "$PROJECT_ROOT/agents/telemetry/capture-on-compact.sh"
  }]
}
```

### Fabric cards

- `agents-telemetry-extract.yaml`
- `agents-telemetry-capture-hook.yaml`
- `agents-telemetry-tool-stats.yaml`

### Handover integration

Modify `agents/handover/handover.sh` to include a `## Tool Call Summary` section:
- Total calls this session
- Error count and rate
- Most-used tool
- Trend vs session average

## PR Description Template

**Title:** `feat: tool call telemetry — capture store + Watchtower reporting`

**Body:**
```
## Problem

No persistent record of tool call activity across sessions. Tool calls live
in Claude Code JSONL transcripts but are session-scoped and not aggregated.
No visibility into error trends, tool usage patterns, or budget burn correlation.

## Solution

Two layers:
1. **Capture store** — PreCompact hook extracts tool calls from JSONL transcript,
   appends to `.context/telemetry/tool-calls.jsonl` (~350 bytes/record)
2. **Reporting** — `fw tool-stats` terminal command + Watchtower web page with
   filter/drill-down. Handover agent gets tool call summary section.

## Schema

tool, timestamp, session_id, task, is_error, error_content, input_summary, token_count

## Validation

Design validated in project 010-termlink across 12+ sessions. Volume estimate
confirmed against real JSONL data. Error reporting (analyze-errors.py) already
proven useful for identifying recurring failure patterns.
```

## Key Constraints

- Framework is bash + Python (Flask for Watchtower) — no TermLink dependency
- JSONL path encoding: `project_root.replace('/', '-')` — same as /capture skill
- PreCompact hook needs the `B-005` protection pattern (agent cannot modify
  `.claude/settings.json` — document manual installation step)
- Watchtower routes live in the Flask app under the web UI subsystem

## After Creating the PR

1. Post the PR URL in the project's T-104 and T-105 tasks as comments
2. Update both task statuses if not already completed
