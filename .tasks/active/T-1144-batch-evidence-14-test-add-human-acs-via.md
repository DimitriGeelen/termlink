---
id: T-1144
name: "Batch-evidence 14 test-add Human ACs via git-derived test counts (G-008 remediation)"
description: >
  Batch-evidence 14 test-add Human ACs via git-derived test counts (G-008 remediation)

status: work-completed
workflow_type: build
owner: human
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-04-19T17:45:37Z
last_update: 2026-04-19T17:47:49Z
date_finished: 2026-04-19T17:47:40Z
---

# T-1144: Batch-evidence 14 test-add Human ACs via git-derived test counts (G-008 remediation)

## Context

Strategy A of G-008 remediation: for 14 test-adding tasks (T-1007, T-1033, T-1041–T-1046, T-1091–T-1098), the Human AC is `[RUBBER-STAMP] Verify test count increased`. Evidence can be derived mechanically from the task's implementation commit (+N test functions) plus the current total test count in the modified file. Inject this evidence above `## Verification` in each task file.

## Acceptance Criteria

### Agent
- [x] Per-task implementation commits identified via `git log --grep`
- [x] Per-task `+N tests` count derived via `git show <commit> | grep '^+.*fn '`
- [x] Current total test counts confirmed: 168 in crates/termlink-cli/tests/cli_integration.rs, 198+ in crates/termlink-hub/src/router.rs
- [x] Evidence blocks injected into all 14 task files before `## Verification`
- [x] `grep -l "G-008 remediation, test-count" .tasks/active/T-10*.md | wc -l` reports 14

### Human
- [ ] [RUBBER-STAMP] Glance at the evidence blocks in any 2-3 task files and confirm the test-count numbers look reasonable
  **Steps:**
  1. `grep -l "G-008 remediation, test-count" /opt/termlink/.tasks/active/*.md | head -3`
  2. Read the evidence block in one of them
  **Expected:** Block cites commit hash + test delta + current file total
  **If not:** Report which task has dubious evidence

## Verification

test $(grep -l "G-008 remediation, test-count" /opt/termlink/.tasks/active/T-10*.md 2>/dev/null | wc -l) -ge 14
test -f crates/termlink-cli/tests/cli_integration.rs
test -f crates/termlink-hub/src/router.rs

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

### 2026-04-19T17:45:37Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1144-batch-evidence-14-test-add-human-acs-via.md
- **Context:** Initial task creation

### 2026-04-19T17:47:40Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
