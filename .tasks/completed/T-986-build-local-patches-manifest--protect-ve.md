---
id: T-986
name: "Build .local-patches manifest — protect vendored framework fixes from fw upgrade"
description: >
  Build .local-patches manifest — protect vendored framework fixes from fw upgrade

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-04-12T20:46:30Z
last_update: 2026-04-12T20:49:31Z
date_finished: 2026-04-12T20:49:31Z
---

# T-986: Build .local-patches manifest — protect vendored framework fixes from fw upgrade

## Context

<!-- One sentence for small tasks. Link to design docs for substantial ones. -->

## Context

T-984 GO. `fw upgrade` / `do_vendor` silently reverts locally-patched framework files. Build a `.local-patches` manifest that `do_vendor` checks before overwriting. See `docs/reports/T-984-fw-upgrade-local-patch-reversion.md`.

## Acceptance Criteria

### Agent
- [x] `.agentic-framework/.local-patches` YAML manifest exists with entries for all 6 known patched files
- [x] `do_vendor()` in `bin/fw` reads `.local-patches` before copying and skips listed files (rsync --exclude)
- [x] Skipped files produce visible summary: "N locally-patched file(s) preserved"
- [x] Non-rsync path (cp fallback) also checks manifest for single-file copies
- [x] `cargo build --workspace` succeeds (no Rust changes)
- [x] Manifest includes: file path, task ID, description for each patched file
- [x] 6 known patched files registered (T-911, T-913, T-949, T-938x2, T-981)

## Verification

# Shell commands that MUST pass before work-completed. One per line.
test -f /opt/termlink/.agentic-framework/.local-patches
grep -q 'local-patches' /opt/termlink/.agentic-framework/bin/fw
python3 -c "import yaml; d=yaml.safe_load(open('/opt/termlink/.agentic-framework/.local-patches')); assert len(d.get('patches',[])) >= 6, f'Expected >=6 patches, got {len(d.get(\"patches\",[]))}'"

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

### 2026-04-12T20:46:30Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-986-build-local-patches-manifest--protect-ve.md
- **Context:** Initial task creation

### 2026-04-12T20:49:31Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
- **Reason:** All verification passes, 6 patches registered, do_vendor modified
