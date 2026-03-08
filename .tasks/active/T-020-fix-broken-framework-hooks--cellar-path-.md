---
id: T-020
name: "Fix broken framework hooks — Cellar path + PROJECT_ROOT"
description: >
  Fix broken framework hooks — Cellar path + PROJECT_ROOT

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-03-08T17:38:49Z
last_update: 2026-03-08T17:38:49Z
date_finished: null
---

# T-020: Fix broken framework hooks — Cellar path + PROJECT_ROOT

## Context

`brew upgrade fw` (1.1.0 → 1.2.4) removed old Cellar dir. All 10 hooks in `.claude/settings.json` hardcoded `/usr/local/Cellar/fw/1.1.0/libexec/...` — silently broke every framework gate. Additionally `PROJECT_ROOT` referenced old project name `010-Ag-Framework-Brew-Test`.

## Acceptance Criteria

### Agent
- [x] All hook paths use `/usr/local/opt/fw/libexec` (upgrade-proof symlink)
- [x] PROJECT_ROOT updated to `010-termlink`
- [ ] All uncommitted task housekeeping committed (T-015–T-019 completions, episodics)
- [ ] Hooks verified working (no errors on tool use)

## Verification

# Verify no hardcoded Cellar paths remain
! grep -q "Cellar" .claude/settings.json
# Verify no old project name remains
! grep -q "Ag-Framework-Brew-Test" .claude/settings.json
# Verify settings.json is valid JSON
python3 -c "import json; json.load(open('.claude/settings.json'))"

## Decisions

### 2026-03-08 — Use opt symlink vs Cellar path
- **Chose:** `/usr/local/opt/fw/libexec` (Homebrew stable symlink)
- **Why:** Survives `brew upgrade` — symlink always points to active version
- **Rejected:** Hardcoded new Cellar path (`1.2.4`) — same problem on next upgrade

## Updates

### 2026-03-08T17:38:49Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /Users/dimidev32/001-projects/010-termlink/.tasks/active/T-020-fix-broken-framework-hooks--cellar-path-.md
- **Context:** Initial task creation
