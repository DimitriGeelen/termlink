---
id: T-1008
name: "Improve inbox CLI help text and add doctor --runtime-dir flag"
description: >
  Improve inbox CLI help text and add doctor --runtime-dir flag

status: work-completed
workflow_type: build
owner: human
horizon: now
tags: []
components: [crates/termlink-cli/src/cli.rs, crates/termlink-cli/src/main.rs]
related_tasks: []
created: 2026-04-13T09:23:03Z
last_update: 2026-04-23T17:25:29Z
date_finished: 2026-04-13T09:26:42Z
---

# T-1008: Improve inbox CLI help text and add doctor --runtime-dir flag

## Context

Two UX improvements: (1) clarify inbox clear help text about --all vs target precedence, (2) add --runtime-dir flag to doctor so it can check the persistent hub at /var/lib/termlink without env var override.

## Acceptance Criteria

### Agent
- [x] Inbox::Clear help text clarifies --all vs target usage
- [x] Doctor accepts --runtime-dir flag to override default runtime directory
- [x] Doctor --runtime-dir /var/lib/termlink shows persistent hub checks (9/9 passed)
- [x] cargo test --workspace passes (1003 tests)
- [x] cargo clippy --workspace passes (0 warnings)

### Human
- [x] [RUBBER-STAMP] Verify `termlink inbox clear --help` reads clearly — ticked by user direction 2026-04-23. Evidence: Live: `termlink inbox clear --help` shows clear per-arg descriptions for `[TARGET]` and `--all` flag (mutually exclusive). Verified 2026-04-23T17:35Z.
  **Steps:**
  1. `cd /opt/termlink && cargo run -- inbox clear --help`
  **Expected:** Help text explains --all vs target clearly
  **If not:** Suggest improved wording


**Agent evidence (auto-batch 2026-04-19, G-008 remediation, inbox-cli-help-text):** `termlink inbox clear --help` shows clear per-arg descriptions and the new `--all` flag; `termlink doctor --help` lists `--runtime-dir <RUNTIME_DIR>` as documented. Both RUBBER-STAMPable.

## Verification

cargo clippy --workspace -- -D warnings 2>&1 | tail -1
cargo test --workspace 2>&1 | grep "^test result" | grep -v "0 passed"

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

### 2026-04-13T09:23:03Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1008-improve-inbox-cli-help-text-and-add-doct.md
- **Context:** Initial task creation

### 2026-04-13T09:26:42Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed

### 2026-04-16T19:00:39Z — programmatic-evidence [T-1087]
- **Evidence:** termlink inbox clear --help shows clear usage with target/--all options
- **Verified by:** automated command execution

