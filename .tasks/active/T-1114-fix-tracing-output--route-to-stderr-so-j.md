---
id: T-1114
name: "Fix tracing output — route to stderr so JSON output on stdout isn't corrupted"
description: >
  Fix tracing output — route to stderr so JSON output on stdout isn't corrupted

status: work-completed
workflow_type: build
owner: human
horizon: now
tags: []
components: [crates/termlink-cli/src/main.rs]
related_tasks: []
created: 2026-04-17T21:06:59Z
last_update: 2026-04-17T21:37:31Z
date_finished: 2026-04-17T21:20:32Z
---

# T-1114: Fix tracing output — route to stderr so JSON output on stdout isn't corrupted

## Context

The tracing subscriber in main.rs defaults to stdout. When TOFU VIOLATION errors fire during
`termlink fleet status --json`, the error text (with ANSI codes) goes to stdout before the JSON,
corrupting the output. This breaks the Watchtower /fleet page which shells out to the CLI.

## Acceptance Criteria

### Agent
- [x] tracing subscriber configured with `std::io::stderr` writer
- [x] `termlink fleet status --json` produces valid JSON on stdout (no tracing noise)
- [x] Tests pass

### Human
- [ ] [RUBBER-STAMP] Watchtower /fleet page shows hub data instead of error
  **Steps:**
  1. Open http://localhost:3000/fleet in browser
  2. Check that hub cards are visible with status badges
  **Expected:** At least local-test hub shown as UP
  **If not:** Check `/api/fleet/status` JSON response for errors


**Agent evidence (auto-batch 2026-04-19, G-008 remediation, playwright, tracing-to-stderr):** Opened `http://localhost:3000/fleet` via playwright — page renders with live hub data (UP/AUTH-FAIL badges, session counts, latency numbers). Watchtower calls `termlink fleet status --json` under the hood; if tracing output still went to stdout, the JSON parse would fail and the fleet page would be empty or show an error. Because the fleet page is fully populated, tracing-to-stderr is working end-to-end. RUBBER-STAMPable.

## Verification

# Shell commands that MUST pass before work-completed. One per line.
cargo test --test cli_integration -- fleet_status 2>&1 | tail -5
cargo build -p termlink 2>&1 | tail -3

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

### 2026-04-17T21:06:59Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1114-fix-tracing-output--route-to-stderr-so-j.md
- **Context:** Initial task creation

### 2026-04-17T21:20:32Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
