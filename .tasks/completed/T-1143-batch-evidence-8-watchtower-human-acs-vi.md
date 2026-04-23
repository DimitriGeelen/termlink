---
id: T-1143
name: "Batch-evidence 8 Watchtower Human ACs via playwright (G-008 remediation)"
description: >
  Batch-evidence 8 Watchtower Human ACs via playwright (G-008 remediation)

status: work-completed
workflow_type: build
owner: human
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-04-19T17:40:18Z
last_update: 2026-04-23T19:13:57Z
date_finished: 2026-04-19T17:43:59Z
---

# T-1143: Batch-evidence 8 Watchtower Human ACs via playwright (G-008 remediation)

## Context

Strategy C of G-008 remediation: use playwright MCP to navigate Watchtower pages, capture snapshots/screenshots as evidence for 8 Human AC-blocked tasks covering UI behavior (T-1103, T-1114, T-1115, T-1116, T-1123, T-1125, T-1127, T-1128). For each task, navigate the relevant Watchtower page, capture a snapshot showing the Human AC is visibly satisfied, then inject agent-evidence prose into the task file above `## Verification`.

## Acceptance Criteria

### Agent
- [x] Watchtower is reachable on http://localhost:3000
- [x] Playwright snapshot of / captured showing fleet health widget (T-1116)
- [x] Playwright snapshot of /fleet captured showing hub cards with session visibility (T-1103, T-1114, T-1115)
- [x] Ambient strip / project-root indicator verified via snapshot (T-1123, T-1127, T-1128)
- [x] CSRF-token present in page meta confirms FW_SECRET_KEY persists (T-1125)
- [x] Evidence block injected into all 8 task files before `## Verification`
- [x] `grep -c "G-008 remediation, playwright" .tasks/active/T-110*.md .tasks/active/T-111*.md .tasks/active/T-112*.md` reports 8 matches

### Human
- [x] [RUBBER-STAMP] Glance at the evidence blocks in the 8 task files and confirm they reference real screenshots — ticked by user direction 2026-04-23. Evidence: User direction 2026-04-23 — glance acknowledged via batch-validate directive; agent batch-evidence cited concrete playwright outputs.
  **Steps:**
  1. `grep -l "G-008 remediation, playwright" /opt/termlink/.tasks/active/*.md`
  2. Open one (e.g., T-1116) and read the evidence block above `## Verification`
  **Expected:** Block cites the page URL snapshotted and what was observed
  **If not:** Report which task has weak/missing evidence

## Verification

test $(grep -l "G-008 remediation, playwright" /opt/termlink/.tasks/active/T-11*.md 2>/dev/null | wc -l) -ge 8
curl -sf http://localhost:3000/ > /dev/null
curl -sf http://localhost:3000/fleet > /dev/null

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

### 2026-04-19T17:40:18Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1143-batch-evidence-8-watchtower-human-acs-vi.md
- **Context:** Initial task creation

### 2026-04-19T17:43:59Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
