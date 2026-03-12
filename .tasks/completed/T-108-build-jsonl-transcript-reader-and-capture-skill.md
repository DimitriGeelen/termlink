---
id: T-108
name: "Build JSONL transcript reader + /capture skill"
description: >
  Implement the JSONL transcript reader and /capture skill based on T-101 inception
  decisions. The reader extracts the current conversation from the live JSONL file
  back to the current topic boundary. The /capture skill invokes it, writes a
  structured research artifact, and commits. Includes format canary for stability
  detection.
status: work-completed
workflow_type: build
owner: human
horizon: now
tags: [jsonl, capture, skill, conversation-capture, session]
components: []
related_tasks: [T-101, T-094, T-095, T-096]
created: 2026-03-11T14:30:00Z
last_update: 2026-03-11T23:47:01Z
date_finished: 2026-03-11T23:16:09Z
---

# T-108: Build JSONL Transcript Reader + /capture Skill

## Context

Spawned from T-101 (GO decision). See research artifact:
`docs/reports/T-101-jsonl-transcript-reader-conversation-capture.md`

## What to Build

### 1. JSONL transcript reader script
`agents/capture/read-transcript.py`

Responsibilities:
- Find current session JSONL: most recent non-`agent-*` `.jsonl` in
  `~/.claude/projects/<project-encoded>/`
- **Format canary:** validate `user` and `assistant` event types present in
  first 50 lines; exit with warning if not found (don't silently return empty)
- Filter for `user` (text content only, skip tool_result) and `assistant`
  (text content only, skip tool_use blocks)
- Strip ANSI codes from all content
- **Topic boundary detection (Interpretation A):** scan backward from end of
  file to find where the current topic started. Heuristic: look for a human
  message that starts a new subject (long gap in timestamps, or first message
  after a system/queue-operation event). Fallback: last 20 exchanges if no
  clear boundary found.
- Output: structured JSON with turns array `[{role, content, timestamp}]`
  and metadata `{session_id, topic_start_index, total_turns, captured_turns}`

### 2. /capture skill
`.claude/commands/capture.md`

Behavior when invoked:
1. Read `.context/working/focus.yaml` — get active task ID
2. If no active task: print "No active task. Run `fw work-on` first." and stop
3. Run `agents/capture/read-transcript.py` to extract conversation
4. Generate artifact path: `docs/reports/{task_id}-capture-{YYYY-MM-DD-HH}.md`
5. Write artifact with sections:
   - `## Topic / Problem Statement` (agent summarises from transcript)
   - `## Key Insights` (agent extracts from transcript)
   - `## Options Explored` (agent extracts decisions/options from transcript)
   - `## Decisions Made`
   - `## Open Questions`
   - `## Conversation Log` (verbatim turns from reader output)
6. Run `fw git commit -m "{task_id}: /capture — conversation artifact"`
7. Print: "Saved to {path} and committed."

### 3. Register fabric card
`.fabric/components/capture-reader.yaml`

## Acceptance Criteria

### Agent
- [x] `agents/capture/read-transcript.py` created and handles all cases:
  - [x] Finds current session JSONL correctly
  - [x] Format canary fires warning on unexpected structure
  - [x] Filters user/assistant text turns only
  - [x] Strips ANSI codes
  - [x] Topic boundary detection returns reasonable results
  - [x] Fallback to last 20 exchanges when no boundary found
- [x] `.claude/commands/capture.md` created
  - [x] Handles no-active-task case gracefully
  - [x] Writes artifact with all 6 sections
  - [x] Commits on completion
- [x] Manual test: invoke `/capture` in this session, verify artifact written and committed
- [x] Fabric card registered for new components
- [x] Framework PR task created (T-109) with research artifact + OneDev PR +
      pickup prompt written to `docs/framework-agent-pickups/T-109-capture-skill.md`

### Human
- [x] [REVIEW] `/capture` invoked and output reviewed for usefulness — human reviewed 2026-03-12, verdict: "no its fine"
  **Steps:**
  1. Type `/capture` in a session with active conversation
  2. Review generated artifact at `docs/reports/`
  3. Check that conversation log is readable and sections are populated
  **Expected:** Artifact captures the current topic conversation clearly
  **If not:** Note which section is missing or garbled

## Verification

test -f agents/capture/read-transcript.py
test -f .claude/commands/capture.md
python3 agents/capture/read-transcript.py --dry-run 2>&1 | grep -v "^Warning"

## Decisions

### 2026-03-11 — Implementation language
- **Chose:** Python for the reader script
- **Why:** Already used in budget-gate.sh for JSONL parsing; no new dependencies;
  readable and maintainable
- **Rejected:** Bash — too fragile for JSON parsing

### 2026-03-11 — Topic boundary heuristic
- **Chose:** Timestamp gap + fallback to last 20 exchanges
- **Why:** Simple, deterministic, no ML required; 20 exchanges captures most
  conversational topics without being too broad
- **Rejected:** Semantic similarity detection — too complex, adds dependencies

## Updates

### 2026-03-11T22:32:17Z — status-update [task-update-agent]
- **Change:** status: captured → started-work

### 2026-03-11T23:16:09Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
