---
id: T-1433
name: "Deploy 0.9.1638 (T-1426 deprecation print) to laptop-141 (.141)"
description: >
  Deploy 0.9.1638 (T-1426 deprecation print) to laptop-141 (.141)

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-05-01T07:34:26Z
last_update: 2026-05-01T07:34:26Z
date_finished: null
---

# T-1433: Deploy 0.9.1638 (T-1426 deprecation print) to laptop-141 (.141)

## Context

<!-- One sentence for small tasks. Link to design docs for substantial ones. -->

## Acceptance Criteria

### Agent
- [x] musl 0.9.1638 binary built locally with T-1426 deprecation print baked in (probe-verified `--version` against bogus push call shows `[DEPRECATED]` line)
- [x] fleet-deploy-binary.sh streamed to .141 with `--probe` and `--swap-restart`. Probe ran post-stage and confirmed binary executes cleanly on the WSL host
- [x] Hub on .141 came back UP within 90s of the swap (observed 10s in the deploy log)
- [x] Post-deploy version probe on .141: `/mnt/c/ntb-acd-plugin/termlink/target/release/termlink --version` reports `termlink 0.9.1638`
- [x] Deprecation nudge fires on .141: `termlink remote push 192.168.10.999:9100 bogus --message x` emits the canonical `[DEPRECATED] termlink remote push — use 'termlink channel post' instead.` line on stderr
- [x] No SSH used for the deploy — all transport via termlink remote exec (PL-015)

## Verification

target/release/termlink fleet doctor 2>&1 | grep -A 1 "laptop-141 (" | grep -q PASS
target/release/termlink remote exec laptop-141 tl-hmfi6wpa "/mnt/c/ntb-acd-plugin/termlink/target/release/termlink --version" 2>&1 | grep -q "0.9.1638"
target/release/termlink remote exec laptop-141 tl-hmfi6wpa "/mnt/c/ntb-acd-plugin/termlink/target/release/termlink remote push 192.168.10.999:9100 bogus --message x 2>&1 | head -1" 2>&1 | grep -q DEPRECATED

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

### 2026-05-01T07:34:26Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1433-deploy-091638-t-1426-deprecation-print-t.md
- **Context:** Initial task creation
