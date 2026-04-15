---
id: T-1061
name: "Housekeeping — close completed tasks and clean stale state"
description: >
  Housekeeping — close completed tasks and clean stale state

status: work-completed
workflow_type: refactor
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-04-15T13:38:15Z
last_update: 2026-04-15T13:44:16Z
date_finished: 2026-04-15T13:44:16Z
---

# T-1061: Housekeeping — close completed tasks and clean stale state

## Context

Many tasks in `.tasks/active/` have all Agent ACs satisfied but never got
moved to `completed/` — the only remaining unchecked items are template
debris ("[REVIEW] Dashboard renders correctly" inside HTML comment blocks)
that grep mistakes for real Human ACs. Audit the T-1016/T-1027..T-1046
batch and close any task whose real AC count = checked count. Do NOT close
tasks with genuine unchecked Human ACs (e.g. T-1051's go/no-go review).

## Acceptance Criteria

### Agent
- [x] Audit T-1016, T-1027..T-1032, T-1040..T-1046 — distinguish real Human ACs from template comment debris
- [x] Close all tasks whose Agent ACs are 100% checked AND have zero real Human ACs (closed T-1052..T-1058 batch; T-1016/T-1027..T-1046 all hold real Human ACs and stay open)
- [x] Record per-task closure outcome (see Decisions section)
- [x] Hub-105 cleanup: remove `/root/.termlink/secrets/hub-105.hex` and `hub-105` profile from `/root/.termlink/hubs.toml` (leftover from withdrawn T-1059)
- [x] All closures committed with batched commits 72bb5078, 60e87e2f, 0682d996, aa2d9d98 referencing T-1061

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

# Hub-105 leftover gone
test ! -f /root/.termlink/secrets/hub-105.hex
! grep -q '^\[hubs.hub-105\]' /root/.termlink/hubs.toml 2>/dev/null
# At least one closure happened (T-1052..T-1058 batch already committed)
test -f .tasks/completed/T-1058-claudemd--document-hub-auth-rotation-pro.md

## Decisions

### 2026-04-15 — closure outcomes per audited task
- **Closed (Agent ACs done, no real Human AC):** T-1052, T-1053, T-1054, T-1055, T-1056, T-1057, T-1058. All had `[REVIEW] Dashboard renders correctly` only inside `<!-- -->` blocks (template debris), not real Human ACs.
- **Deferred (real Human AC required):** T-1016 (inception go/no-go review), T-1027 (deployment verification on .109/.121), T-1028 (TLS cert preservation), T-1029, T-1030, T-1031, T-1032 (split-brain runtime fixes — operator verification), T-1040 (MCP tool integration), T-1041..T-1046 ([RUBBER-STAMP] test count verification — could be auto-evidenced but framework rule forbids agent checking Human ACs).
- **G-006 noted:** discovered that pre-push hook stamps project VERSION into `.agentic-framework/VERSION` — registered as concern; upstream framework fix.

## Updates

### 2026-04-15T13:38:15Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1061-housekeeping--close-completed-tasks-and-.md
- **Context:** Initial task creation

### 2026-04-15T13:44:16Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
