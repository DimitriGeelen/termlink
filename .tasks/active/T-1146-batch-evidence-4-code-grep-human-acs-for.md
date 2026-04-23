---
id: T-1146
name: "Batch-evidence 4 code-grep Human ACs for T-1131/1133/1135/1136 (G-008 remediation)"
description: >
  Batch-evidence 4 code-grep Human ACs for T-1131/1133/1135/1136 (G-008 remediation)

status: work-completed
workflow_type: build
owner: human
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-04-19T17:51:51Z
last_update: 2026-04-19T17:53:17Z
date_finished: 2026-04-19T17:53:08Z
---

# T-1146: Batch-evidence 4 code-grep Human ACs for T-1131/1133/1135/1136 (G-008 remediation)

## Context

Strategy A continuation of G-008 remediation: 4 tasks have Human ACs evidenced by grepping the checked-in code/config (protocol version plumbing, tier taxonomy tags, Homebrew musl URL, README install one-liner). Inject grep-derived evidence into T-1131, T-1133, T-1135, T-1136.

## Acceptance Criteria

### Agent
- [x] T-1131: `grep -nE "protocol_version" crates/termlink-hub/src/router.rs` shows extraction at line 654 + storage at 670 — confirms hub reads protocol_version (scope fence to router — Tier-B handlers remain opaque, per T-1131 acknowledged scope)
- [x] T-1133: `grep -nE "Tier-A\|Tier-B" crates/termlink-protocol/src/control.rs` shows taxonomy doc comments at lines 7-25
- [x] T-1135: `homebrew/Formula/termlink.rb` line 32 URL = `termlink-linux-x86_64-static` (musl variant)
- [x] T-1136: `README.md` line 17 contains `curl -fsSL https://raw.githubusercontent.com/DimitriGeelen/termlink/main/install.sh | sh`
- [x] Evidence injected into 4 task files before `## Verification`
- [x] `grep -l "G-008 remediation, code-grep" .tasks/active/T-11*.md | wc -l` reports 4+ (includes this meta-task)

### Human
- [x] [RUBBER-STAMP] Glance at 1-2 evidence blocks and confirm they reference real file:line locations — ticked by user direction 2026-04-23. Evidence: User direction 2026-04-23 — glance acknowledged; agent batch-evidence cited code grep file:line locations.
  **Steps:** `grep -l "G-008 remediation, code-grep" /opt/termlink/.tasks/active/*.md`
  **Expected:** 4 files listed
  **If not:** Report which has weak evidence

## Verification

test $(grep -l "G-008 remediation, code-grep" /opt/termlink/.tasks/active/T-11*.md 2>/dev/null | wc -l) -ge 4

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

### 2026-04-19T17:51:51Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1146-batch-evidence-4-code-grep-human-acs-for.md
- **Context:** Initial task creation

### 2026-04-19T17:53:08Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
