---
id: T-1145
name: "Batch-evidence 4 remote/inbox Human ACs via live termlink (G-008 remediation)"
description: >
  Batch-evidence 4 remote/inbox Human ACs via live termlink (G-008 remediation)

status: work-completed
workflow_type: build
owner: human
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-04-19T17:49:08Z
last_update: 2026-04-19T17:50:49Z
date_finished: 2026-04-19T17:50:43Z
---

# T-1145: Batch-evidence 4 remote/inbox Human ACs via live termlink (G-008 remediation)

## Context

Strategy B continuation of G-008 remediation: 4 tasks have Human ACs that can be evidenced by live termlink commands against the running hub fleet (ring20-management on 192.168.10.122:9100).

## Acceptance Criteria

### Agent
- [x] `termlink remote ping ring20-management` returns PONG with latency (evidence for T-1012)
- [x] GitHub Releases v0.9.1 checksums.txt contains `termlink-linux-x86_64-static` (evidence for T-1019)
- [x] `termlink remote doctor ring20-management` returns 3 PASS, 0 fail (evidence for T-1020)
- [x] `termlink remote inbox ring20-management --help` shows 4 subcommands (evidence for T-1076)
- [x] Evidence injected into 4 task files before `## Verification`
- [x] `grep -l "G-008 remediation, live-termlink" .tasks/active/T-10*.md | wc -l` reports 4

### Human
- [ ] [RUBBER-STAMP] Glance at evidence blocks in the 4 task files and confirm outputs look correct
  **Steps:** `grep -l "G-008 remediation, live-termlink" /opt/termlink/.tasks/active/*.md`
  **Expected:** 4 files listed
  **If not:** Report which task has dubious or missing evidence

## Verification

test $(grep -l "G-008 remediation, live-termlink" /opt/termlink/.tasks/active/T-10*.md 2>/dev/null | wc -l) -ge 4

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

### 2026-04-19T17:49:08Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1145-batch-evidence-4-remoteinbox-human-acs-v.md
- **Context:** Initial task creation

### 2026-04-19T17:50:43Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
