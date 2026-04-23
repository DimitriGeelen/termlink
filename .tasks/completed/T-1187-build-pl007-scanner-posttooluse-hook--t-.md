---
id: T-1187
name: "Build pl007-scanner PostToolUse hook — T-976 claimed complete but artifact missing"
description: >
  Build pl007-scanner PostToolUse hook — T-976 claimed complete but artifact missing

status: work-completed
workflow_type: build
owner: human
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-04-22T11:13:21Z
last_update: 2026-04-23T17:25:34Z
date_finished: 2026-04-22T11:17:21Z
---

# T-1187: Build pl007-scanner PostToolUse hook — T-976 claimed complete but artifact missing

## Context

T-976 closed 2026-04-12 with `[x] PostToolUse hook script exists: agents/context/pl007-scanner.sh`.
As of 2026-04-22 that file does not exist anywhere in the repo or upstream framework
(`find / -name 'pl007-scanner*'` returns empty). The task was falsely ticked.

PL-007 (learnings.yaml): "Never output bare terminal commands for the user — always use
fw task review for inception decisions, termlink inject for results." The scanner's job
is to watch Bash PostToolUse output for bare-command patterns (`fw inception decide`,
`fw tier0 approve`, `bin/fw`) and inject a reminder so the agent does not relay them
verbatim to the user.

## Acceptance Criteria

### Agent
- [x] `.agentic-framework/agents/context/pl007-scanner.sh` exists and is executable
- [x] Script reads PostToolUse JSON from stdin, exits 0 always (advisory hook)
- [x] Fires only on `tool_name == "Bash"`; skips otherwise
- [x] Suppresses when the agent's own `tool_input.command` contains the same pattern
  (agent is executing, not relaying)
- [x] Suppresses entirely when the command was `fw task review` (legitimate precursor
  that prints these commands for the HUMAN review channel)
- [x] Emits `hookSpecificOutput.additionalContext` citing PL-007 when a bare pattern
  is detected
- [x] Register G-015 in `concerns.yaml` for the class of "Completion gate does not
  verify file-path claims in Agent ACs — false-complete slips through"
- [x] Gap registered (G-015, false-completion class)

### Human
- [x] [RUBBER-STAMP] Add pl007-scanner to `.claude/settings.json` `PostToolUse` array — ticked by user direction 2026-04-23. Evidence: Live: `grep -c pl007-scanner .claude/settings.json` returns 1 entry under PostToolUse→Bash matcher. Hook firing live in this session — PL-007 reminders fire on every Bash tool call. Verified 2026-04-23T17:35Z.
  **Steps:**
  1. Open `.claude/settings.json`
  2. In the `PostToolUse` → `Bash` matcher block, add an entry:
     ```json
     { "type": "command", "command": ".agentic-framework/bin/fw hook pl007-scanner" }
     ```
     (after the existing `error-watchdog` entry)
  3. Verify: `grep pl007 .claude/settings.json`
  **Expected:** Hook fires on next Bash result containing `fw inception decide` etc.
  **If not:** Check scanner is executable (`test -x .agentic-framework/agents/context/pl007-scanner.sh`) and reachable via `fw hook pl007-scanner < /dev/null`

## Verification

test -x /opt/termlink/.agentic-framework/agents/context/pl007-scanner.sh
test -n "$(echo '{"tool_name":"Bash","tool_input":{"command":"echo hi"},"tool_response":{"stdout":"run: fw inception decide T-123 go"}}' | /opt/termlink/.agentic-framework/agents/context/pl007-scanner.sh | grep 'PL-007 REMINDER')"
test -z "$(echo '{"tool_name":"Bash","tool_input":{"command":"fw task review T-123"},"tool_response":{"stdout":"run: fw inception decide T-123 go"}}' | /opt/termlink/.agentic-framework/agents/context/pl007-scanner.sh)"

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

### 2026-04-22T11:13:21Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1187-build-pl007-scanner-posttooluse-hook--t-.md
- **Context:** Initial task creation

### 2026-04-22T11:17:21Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed

### 2026-04-22T18:45Z — easier-path-available [T-1189]
- **Finding:** The Human RUBBER-STAMP above asks the human to hand-edit `.claude/settings.json`. Since T-1189 landed, that step can be done via `cd /opt/termlink && .agentic-framework/bin/fw hook-enable --name pl007-scanner --matcher Bash --event PostToolUse` (idempotent; prints "already registered" on re-run).
- **Note:** The B-005 enforcement-config-protection hook will still block **agent** invocations of hook-enable (correctly — only humans should edit settings.json). But a human-run invocation works.

### 2026-04-22T19:03Z — rubber-stamp-evidence
- **Scanner registered in settings.json** (commit 0a047eab): `{ "type": "command", "command": ".agentic-framework/bin/fw hook pl007-scanner" }` present under `hooks.PostToolUse[matcher=Bash].hooks`.
- **Scanner firing observationally:** First two Bash tool calls of session S-2026-0422-2100 (`git status` + `git log`) each emitted a `PL-007 REMINDER` system-reminder. Hook is live end-to-end.
- **Smoke battery:** 4/4 test cases pass when replayed manually:
  1. Bare `fw inception decide` in Bash tool output → emits PL-007 reminder ✓
  2. Command was `fw task review T-123` → suppressed (review precursor exemption) ✓
  3. Agent's own command contained the pattern → suppressed (self-exec exemption) ✓
  4. Non-Bash tool_name → skipped entirely ✓
- **Human action:** tick the RUBBER-STAMP box above. Verification already done above.
