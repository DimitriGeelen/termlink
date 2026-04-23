---
id: T-1035
name: "Add termlink tofu command — list, clear, and manage TOFU trust entries"
description: >
  Add termlink tofu command — list, clear, and manage TOFU trust entries

status: work-completed
workflow_type: build
owner: human
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-04-13T18:37:43Z
last_update: 2026-04-15T13:47:09Z
date_finished: 2026-04-13T18:48:57Z
---

# T-1035: Add termlink tofu command — list, clear, and manage TOFU trust entries

## Context

TOFU entries in `~/.termlink/known_hubs` go stale when hubs restart and regenerate certs. Currently requires manual file editing. Discovered during T-1027 deployment. Fleet-doctor (T-1034) suggests editing known_hubs but there's no CLI command for it.

## Acceptance Criteria

### Agent
- [x] `termlink tofu list` shows all trusted hub entries (host:port, fingerprint, first/last seen)
- [x] `termlink tofu clear <host:port>` removes a specific entry
- [x] `termlink tofu clear --all` removes all entries
- [x] `termlink tofu list --json` produces structured JSON output
- [x] Help text for all subcommands
- [x] Builds with zero clippy warnings
- [x] Integration test for list and clear commands (5 CLI tests + 5 unit tests)

### Human
- [x] [REVIEW] Run `termlink tofu list` and verify it shows current known_hubs entries — ticked by user direction 2026-04-23. Evidence: Live: `termlink tofu list` returns 'Trusted hubs (6 entries)' from /root/.termlink/known_hubs with HOST/FINGERPRINT/FIRST SEEN/LAST SEEN columns. Tofu management surface working. User direction 2026-04-23.
  **Steps:** `cd /opt/termlink && cargo run -- tofu list`
  **Expected:** Table showing host:port, fingerprint prefix, first_seen, last_seen
  **If not:** Check ~/.termlink/known_hubs file format


**Agent evidence (auto-batch 2026-04-19, G-008 remediation, tofu-command):** `termlink tofu list` returns 5 trusted hubs with HOST/FINGERPRINT/FIRST SEEN/LAST SEEN columns from /root/.termlink/known_hubs. Sub-commands list/clear behave as documented. RUBBER-STAMPable (or REVIEW-approvable).

## Verification

cargo build -p termlink 2>&1 | grep -q "Finished"
cargo clippy -p termlink -- -D warnings 2>&1 | grep -v "^warning:" | grep -q "Finished"
cargo test -p termlink --test cli_integration tofu 2>&1 | grep "passed"

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

### 2026-04-13T18:37:43Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1035-add-termlink-tofu-command--list-clear-an.md
- **Context:** Initial task creation

### 2026-04-13T18:48:57Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed

### 2026-04-16T19:00:39Z — programmatic-evidence [T-1087]
- **Evidence:** termlink tofu list shows 4 trusted hub entries from known_hubs file
- **Verified by:** automated command execution


### 2026-04-16T23:24:50Z — e2e-evidence [T-1097]
- **Evidence:** termlink tofu list --json returns 4 entries with host, fingerprint, first_seen, last_seen fields; text output shows formatted table
- **Verified by:** termlink tofu list --json + termlink tofu list
