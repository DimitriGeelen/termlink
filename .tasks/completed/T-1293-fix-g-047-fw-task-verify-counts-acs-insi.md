---
id: T-1293
name: "Fix G-047: fw task verify counts ACs inside HTML comments"
description: >
  Patch fw task verify Python parser to strip HTML <!-- ... --> blocks from the Human AC region before counting checkboxes. Default template's example AC '[REVIEW] Dashboard renders correctly' currently surfaces as a real unchecked Human AC for every freshly-templated task, inflating the G-008 partial-complete signal with false positives. One-line re.sub fix in .agentic-framework/bin/fw line ~1981; mirror to upstream framework via Channel 1 (T-1192 dispatch pattern).

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-04-26T11:57:36Z
last_update: 2026-04-26T12:01:17Z
date_finished: 2026-04-26T12:01:17Z
---

# T-1293: Fix G-047: fw task verify counts ACs inside HTML comments

## Context

G-047 (registered 2026-04-26): the `fw task verify` Python parser counts `- [ ]` checkboxes inside HTML `<!--...-->` blocks. The default task template stores an example Human AC `- [ ] [REVIEW] Dashboard renders correctly` inside such a comment to show authors how to format their own Human ACs. As a result every freshly-templated task with the example block left in surfaces in the Awaiting-Human-Verification queue with one fake AC, inflating the G-008 backlog with false positives.

The bash polling counter at `fw task verify <id> --poll` (around line 2055) has the same blindness — fix both for consistency.

## Acceptance Criteria

### Agent
- [x] `fw task verify` Python parser strips HTML `<!--...-->` blocks from each task's `### Human` section before counting `- [ ]` and `- [x]` checkboxes
- [x] Bash poll-mode counter inside `fw task verify <id> --poll` similarly skips lines that fall between `<!--` and `-->` markers
- [x] After the patch, `fw task verify` no longer lists T-1124, T-1261, T-1293 (these three have only template-comment ACs); T-1290 (real Human AC) and T-1137 (two real Human ACs) are still listed correctly — verified, queue dropped 5→3, only real-AC tasks remain
- [x] Vendored `.agentic-framework/bin/fw` carries the patch
- [x] Upstream framework (`/opt/999-Agentic-Engineering-Framework/bin/fw`) carries the same patch via Channel 1 dispatch — landed as commit `c6efdc0db T-1293/G-047: ...` on master, verified via `termlink exec framework-agent`
- [x] G-047 closed with `closure_evidence` referencing this task and the upstream commit hash

## Verification

# Confirm Python regex change present in vendored copy
grep -q 'G-047: strip HTML' .agentic-framework/bin/fw
# Confirm fw task verify no longer false-positives on the three template-only tasks
! .agentic-framework/bin/fw task verify 2>&1 | grep -qE 'T-1124|T-1261|T-1293'
# Confirm fw task verify still surfaces tasks with real Human ACs
.agentic-framework/bin/fw task verify 2>&1 | grep -q 'T-1137'
.agentic-framework/bin/fw task verify 2>&1 | grep -q 'T-1290'

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

### 2026-04-26T11:57:36Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1293-fix-g-047-fw-task-verify-counts-acs-insi.md
- **Context:** Initial task creation

### 2026-04-26T11:57:43Z — status-update [task-update-agent]
- **Change:** status: captured → started-work

### 2026-04-26T12:01:17Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
