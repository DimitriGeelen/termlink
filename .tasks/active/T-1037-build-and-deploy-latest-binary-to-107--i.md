---
id: T-1037
name: "Build and deploy latest binary to .107 — includes T-1033 through T-1036 improvements"
description: >
  Build and deploy latest binary to .107 — includes T-1033 through T-1036 improvements

status: work-completed
workflow_type: build
owner: human
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-04-13T18:52:30Z
last_update: 2026-04-15T13:47:09Z
date_finished: 2026-04-13T19:02:33Z
---

# T-1037: Build and deploy latest binary to .107 — includes T-1033 through T-1036 improvements

## Context

Build musl static binary with all improvements from T-1033 through T-1036 and upgrade the locally installed binary at /usr/local/bin/termlink. Includes: tofu CLI, fleet-doctor diagnostics, hub status tests, TOFU hints.

## Acceptance Criteria

### Agent
- [x] Musl static binary built successfully (14.7MB static-pie, v0.9.835)
- [x] Binary deployed to /usr/local/bin/termlink via atomic swap
- [x] Hub restarted with new binary (PID 453871)
- [x] `termlink remote ping local-test` succeeds (115ms)
- [x] `termlink tofu list` works on installed binary (shows 3 entries)
- [x] `termlink fleet doctor` shows new diagnostic output (secret source + hints)

### Human
- [ ] [RUBBER-STAMP] Verify installed version
  **Steps:** `termlink --version`
  **Expected:** Version >= 0.9.833
  **If not:** Check if binary was correctly swapped

## Verification

/usr/local/bin/termlink --version
/usr/local/bin/termlink remote ping local-test
/usr/local/bin/termlink tofu list

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

### 2026-04-13T18:52:30Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1037-build-and-deploy-latest-binary-to-107--i.md
- **Context:** Initial task creation

### 2026-04-13T19:02:33Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed

### 2026-04-16T19:00:39Z — programmatic-evidence [T-1087]
- **Evidence:** termlink version reports 0.9.53 (15fe47e4) with 67 MCP tools
- **Verified by:** automated command execution

