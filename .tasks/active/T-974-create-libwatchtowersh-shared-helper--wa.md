---
id: T-974
name: "Create lib/watchtower.sh shared helper — _watchtower_url and _watchtower_open"
description: >
  Extract shared Watchtower URL detection and browser-open logic from review.sh into lib/watchtower.sh. Two functions: _watchtower_url (port detection + host detection) and _watchtower_open (URL + browser open with desktop-user awareness). Refactor review.sh to use them. From T-972 RC-3 fix.

status: work-completed
workflow_type: build
owner: human
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-04-12T10:27:15Z
last_update: 2026-04-12T10:35:44Z
date_finished: 2026-04-12T10:35:44Z
---

# T-974: Create lib/watchtower.sh shared helper — _watchtower_url and _watchtower_open

## Context

T-972 RCA identified RC-3: no shared Watchtower URL helper. Each script independently constructs URLs with hardcoded ports. See `docs/reports/T-972-command-amnesia-rca.md`.

## Acceptance Criteria

### Agent
- [x] `lib/watchtower.sh` exists with `_watchtower_url()` function (port detection + host detection)
- [x] `lib/watchtower.sh` has `_watchtower_open()` function (URL + browser open, desktop-user aware)
- [x] `review.sh` refactored to source and use `lib/watchtower.sh` instead of inline port/browser logic
- [x] `fw task review` still works end-to-end (correct port 3002, QR code, browser open)
- [x] Related tasks updated: T-975 (gate refactor), T-976 (PostToolUse hook)

### Human
- [ ] [RUBBER-STAMP] Verify `fw task review T-974` opens browser to correct Watchtower URL
  **Steps:**
  1. Run: `cd /opt/termlink && fw task review T-974`
  2. Verify browser opens to correct port (3002)
  3. Verify QR code renders in terminal
  **Expected:** Browser opens to http://192.168.10.107:3002/review/T-974, QR visible
  **If not:** Check `fw doctor` output and report port detection failure

## Verification

# Shell commands that MUST pass before work-completed. One per line.
bash -c "source /opt/termlink/.agentic-framework/lib/watchtower.sh && type _watchtower_url && type _watchtower_open"
bash -c "source /opt/termlink/.agentic-framework/lib/watchtower.sh && url=\$(_watchtower_url T-974) && [ -n \"\$url\" ] && echo \"OK: \$url\""
grep -q '_watchtower_url\|_watchtower_open' /opt/termlink/.agentic-framework/lib/review.sh

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

### 2026-04-12T10:27:15Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-974-create-libwatchtowersh-shared-helper--wa.md
- **Context:** Initial task creation

### 2026-04-12T10:35:44Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
