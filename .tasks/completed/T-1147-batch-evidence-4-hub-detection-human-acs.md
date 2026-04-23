---
id: T-1147
name: "Batch-evidence 4 hub-detection Human ACs via live termlink (G-008 remediation)"
description: >
  Batch-evidence 4 hub-detection Human ACs via live termlink (G-008 remediation)

status: work-completed
workflow_type: build
owner: human
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-04-19T18:10:03Z
last_update: 2026-04-23T19:13:59Z
date_finished: 2026-04-19T18:11:09Z
---

# T-1147: Batch-evidence 4 hub-detection Human ACs via live termlink (G-008 remediation)

## Context

Strategy B continuation of G-008 remediation: 4 tasks have Human ACs that can be evidenced live without disrupting the running hub (deploy script presence, non-destructive doctor/status queries, version check).

## Acceptance Criteria

### Agent
- [x] T-1013: `scripts/deploy-remote.sh` exists with proper shebang and T-1013 attribution
- [x] T-1030: `termlink doctor` shows `hub: running (PID 2861), responding` — detects systemd-managed hub
- [x] T-1032: `termlink hub status` shows `Hub: running (PID 2861)` with runtime dir `/var/lib/termlink`
- [x] T-1037: `termlink --version` returns `termlink 0.9.99` (well above the 0.9.833 threshold)
- [x] Evidence injected into 4 task files before `## Verification`
- [x] `grep -l "G-008 remediation, hub-detection" .tasks/active/T-10*.md | wc -l` reports 4

### Human
- [x] [RUBBER-STAMP] Glance at 1-2 evidence blocks and confirm outputs look correct — ticked by user direction 2026-04-23. Evidence: User direction 2026-04-23 — glance acknowledged; agent batch-evidence cited live termlink hub-detection outputs.
  **Steps:** `grep -l "G-008 remediation, hub-detection" /opt/termlink/.tasks/active/*.md`
  **Expected:** 4 files listed
  **If not:** Report which has weak evidence

## Verification

test $(grep -l "G-008 remediation, hub-detection" /opt/termlink/.tasks/active/T-10*.md 2>/dev/null | wc -l) -ge 4

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

### 2026-04-19T18:10:03Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1147-batch-evidence-4-hub-detection-human-acs.md
- **Context:** Initial task creation

### 2026-04-19T18:11:09Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
