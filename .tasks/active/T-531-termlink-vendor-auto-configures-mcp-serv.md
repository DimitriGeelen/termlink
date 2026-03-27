---
id: T-531
name: "termlink vendor auto-configures MCP server in Claude Code settings"
description: >
  termlink vendor auto-configures MCP server in Claude Code settings

status: work-completed
workflow_type: build
owner: human
horizon: now
tags: []
components: [crates/termlink-cli/src/commands/vendor.rs]
related_tasks: []
created: 2026-03-27T13:30:39Z
last_update: 2026-03-27T14:02:27Z
date_finished: 2026-03-27T14:02:27Z
---

# T-531: termlink vendor auto-configures MCP server in Claude Code settings

## Context

`termlink vendor` copies the binary to `.termlink/bin/termlink` but does not configure Claude Code's MCP settings. The MCP server (`termlink mcp serve`) is fully functional but invisible to Claude Code in consumer projects. This task adds auto-configuration of `.claude/settings.local.json` during vendor.

## Acceptance Criteria

### Agent
- [x] `cmd_vendor` writes a `termlink` entry to `.claude/settings.local.json` under `mcpServers` with command `.termlink/bin/termlink` and args `["mcp", "serve"]`
- [x] Existing `.claude/settings.local.json` content is preserved (merges, not overwrites)
- [x] If `.claude/` dir doesn't exist, it is created
- [x] If MCP entry already exists and matches, no-op (no duplicate writes)
- [x] Dry-run mode reports the MCP config that would be written
- [x] `cargo build` succeeds
- [x] `cargo clippy` passes

### Human
- [ ] [REVIEW] Run `termlink vendor` in a test project and verify `.claude/settings.local.json` has the MCP entry
  **Steps:**
  1. `mkdir /tmp/test-vendor && cd /tmp/test-vendor && termlink vendor`
  2. `cat .claude/settings.local.json`
  **Expected:** File contains `"mcpServers": { "termlink": { "command": ".termlink/bin/termlink", "args": ["mcp", "serve"] } }`
  **If not:** Check stderr for errors from the vendor command

## Verification

PATH="$HOME/.cargo/bin:$PATH" cargo build 2>&1 | tail -1
! PATH="$HOME/.cargo/bin:$PATH" cargo clippy 2>&1 | grep -q "^error"

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

### 2026-03-27T13:30:39Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-531-termlink-vendor-auto-configures-mcp-serv.md
- **Context:** Initial task creation

### 2026-03-27T14:02:27Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
