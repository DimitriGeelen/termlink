---
id: T-1399
name: "arc-suite --quick mode + --help — dev-loop ergonomics"
description: >
  arc-suite --quick mode + --help — dev-loop ergonomics

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-04-28T22:04:49Z
last_update: 2026-04-28T22:04:49Z
date_finished: null
---

# T-1399: arc-suite --quick mode + --help — dev-loop ergonomics

## Context

`arc-suite.sh` runs all 7 e2e scripts (~13s). For quick dev-loop iterations a developer often wants to skip the slower phases (stress-soak, mention-stream which has a 5s wait). This task adds two flags:
- `--quick` — skips stress-soak and mention-stream (runs the 5 correctness scripts only, ~7s)
- `--help` / `-h` — usage text

Documented in the runbook + the suite's own header.

## Acceptance Criteria

### Agent
- [x] `--help`/`-h` flag prints usage + script list + env vars
- [x] `--quick` flag runs the 5 fast scripts only (skips stress-soak + mention-stream)
- [x] Default behaviour unchanged (all 7 scripts)
- [x] Help text references the runbook
- [x] runbook updated to mention the flags
- [x] `arc-suite.sh --quick` exits 0 with `ARC SUITE GREEN`
- [x] `arc-suite.sh` (no flag) still exits 0 with `ARC SUITE GREEN`
- [x] Work committed with task reference

### Human
<!-- Criteria requiring human verification (UI/UX, subjective quality). Not blocking.
     Remove this section if all criteria are agent-verifiable.
     Each criterion MUST include Steps/Expected/If-not so the human can act without guessing.
     Optionally prefix with [RUBBER-STAMP] or [REVIEW] for prioritization.
     Example:
       - [ ] [REVIEW] Dashboard renders correctly
         **Steps:**
         1. Open https://example.com/dashboard in browser
         2. Verify all panels load within 2 seconds
         3. Check browser console for errors
         **Expected:** All panels visible, no console errors
         **If not:** Screenshot the broken panel and note the console error
-->

## Verification

out_help=$(./tests/e2e/arc-suite.sh --help 2>&1) && echo "$out_help" | grep -q "Usage:"
out_quick=$(BIN=./target/release/termlink ./tests/e2e/arc-suite.sh --quick 2>&1) && echo "$out_quick" | grep -q "ARC SUITE GREEN"
out_full=$(BIN=./target/release/termlink ./tests/e2e/arc-suite.sh 2>&1) && echo "$out_full" | grep -q "ARC SUITE GREEN"
grep -q "\-\-quick" docs/operations/agent-conversation-arc-e2e.md

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

### 2026-04-28T22:04:49Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1399-arc-suite---quick-mode----help--dev-loop.md
- **Context:** Initial task creation
