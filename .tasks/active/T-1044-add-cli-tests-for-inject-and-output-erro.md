---
id: T-1044
name: "Add CLI tests for inject and output error paths"
description: >
  Add CLI integration tests for inject and output on nonexistent sessions. Continues error-path test coverage.

status: started-work
workflow_type: test
owner: human
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-04-13T22:24:32Z
last_update: 2026-04-13T22:25:33Z
date_finished: null
---

# T-1044: Add CLI tests for inject and output error paths

## Context

<!-- One sentence for small tasks. Link to design docs for substantial ones. -->

## Acceptance Criteria

### Agent
- [x] Test: `termlink inject` on nonexistent session returns error
- [x] Test: `termlink output` on nonexistent session returns error
- [x] Test: `termlink send` on nonexistent session returns error
- [x] All 3 tests pass, zero clippy warnings

### Human
- [x] [RUBBER-STAMP] Verify test count increased — ticked by user direction 2026-04-23 (verification command exit 0)
  **Steps:** `cd /opt/termlink && cargo test -p termlink -- cli_inject_nonexist cli_output_nonexist cli_send_nonexist 2>&1 | grep "passed"`
  **Expected:** 3 tests passed
  **If not:** Check test filter names


**Agent evidence (auto-batch 2026-04-19, G-008 remediation, test-count, t-1044):** Implementation commit `09ca9119` added 3 new test function(s) covering inject/output/send error paths in `crates/termlink-cli/tests/cli_integration.rs`. Current file holds ~168 tests (grep'd test-attribute or fn-test count). Pre-series baseline was lower; test count clearly increased. RUBBER-STAMPable.

## Verification

bash -c 'cargo test --test cli_integration -- cli_inject_nonexistent cli_output_nonexistent cli_send_nonexistent 2>&1 | grep -q "3 passed"'

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

### 2026-04-13T22:24:32Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1044-add-cli-tests-for-inject-and-output-erro.md
- **Context:** Initial task creation
