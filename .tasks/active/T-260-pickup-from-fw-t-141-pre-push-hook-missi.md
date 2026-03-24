---
id: T-260
name: "Pickup from fw T-141: Pre-push hook missing PROJECT_ROOT (framework-side)"
description: >
  Framework-side fix from fw-agent. Pre-push hook doesn't pass PROJECT_ROOT to audit
  script. Also: declare -A breaks on macOS bash 3.2, D2 should be WARN not FAIL for
  upstream-blocked tasks. This is a framework repo fix — track here, dispatch back to .107.

status: captured
workflow_type: build
owner: human
horizon: later
tags: [pickup, framework]
components: []
related_tasks: [T-160]
created: 2026-03-24T08:42:18Z
last_update: 2026-03-24T08:42:18Z
date_finished: null
---

# T-260: Pickup from fw T-141 — Pre-push hook missing PROJECT_ROOT

## Context

Framework-side fix. Tracked here for visibility. Overlaps with T-160 (declare -A macOS bash 3.2 issue). Needs to be dispatched to the framework agent on .107 for implementation.

## Pickup Message (from fw-agent)

Pre-push hook doesn't pass PROJECT_ROOT to audit script. One-line fix in `agents/git/lib/hooks.sh` line ~328: change `'"\"'` to `'PROJECT_ROOT="\" "\"'`. Also: `declare -A` breaks on macOS bash 3.2, and D2 should be WARN not FAIL for upstream-blocked tasks. Full report at: termlink project `docs/reports/T-141-upstream-fix-request.md`.

## Acceptance Criteria

### Agent
- [ ] Pickup dispatched to framework agent on .107 (already sent as part of T-258)

## Updates

### 2026-03-24T08:42:18Z — task-created [pickup from fw-agent on .107]
- **Source:** `/pickup T-141` via termlink remote inject output read
- **Note:** Framework-side fix, not TermLink code. Related to T-160 (declare -A).
