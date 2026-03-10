---
id: T-068
name: "CLI restructuring — command grouping, --json output, shell completions"
description: >
  Group 28 flat subcommands into pty.* and event.* namespaces. Add --json output flag. Generate shell completions via clap_complete.

status: work-completed
workflow_type: refactor
owner: human
horizon: next
tags: []
components: []
related_tasks: []
created: 2026-03-10T08:44:38Z
last_update: 2026-03-10T17:33:49Z
date_finished: 2026-03-10T17:22:10Z
---

# T-068: CLI restructuring — command grouping, --json output, shell completions

## Context

UX issues found by reflection fleet cli-ux agent. 28 flat subcommands need grouping (pty.*, event.*), missing --json output and shell completions. See [docs/reports/reflection-result-cli.md].

## Acceptance Criteria

### Agent
- [x] PTY commands nested under `pty` subcommand: `termlink pty attach`, `pty inject`, `pty resize`, `pty stream`, `pty output`
- [x] Event commands nested under `event` subcommand: `termlink event watch`, `event emit`, `event broadcast`, `event wait`, `event topics`
- [x] Old flat command names remain as hidden aliases for backward compatibility (one release cycle)
- [x] `--json` flag added to `list`, `status`, `info`, `events`, `topics` commands — outputs structured JSON
- [x] Shell completions generated via `clap_complete` for bash, zsh, fish
- [x] `termlink --help` shows grouped commands with clear descriptions
- [x] All existing e2e tests pass with the restructured CLI (may need command updates)

### Human
- [ ] [REVIEW] CLI help output is readable and logically grouped
  **Steps:** Run `termlink --help` and `termlink pty --help`
  **Expected:** Commands grouped by domain, descriptions clear
  **If not:** Note which grouping feels wrong

## Verification

/Users/dimidev32/.cargo/bin/cargo build -p termlink 2>&1 | tail -3
# Verify nested subcommands exist
/Users/dimidev32/.cargo/bin/cargo run -p termlink -- pty --help 2>&1 | grep -q "attach\|inject"
/Users/dimidev32/.cargo/bin/cargo run -p termlink -- event --help 2>&1 | grep -q "watch\|emit"

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

### 2026-03-10T08:44:38Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /Users/dimidev32/001-projects/010-termlink/.tasks/active/T-068-cli-restructuring--command-grouping---js.md
- **Context:** Initial task creation

### 2026-03-10T17:13:59Z — status-update [task-update-agent]
- **Change:** status: captured → started-work

### 2026-03-10T17:22:10Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
