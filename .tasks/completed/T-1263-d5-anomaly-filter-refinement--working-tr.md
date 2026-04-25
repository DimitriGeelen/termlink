---
id: T-1263
name: "D5 anomaly filter refinement + working tree cleanup"
description: >
  D5 anomaly filter refinement + working tree cleanup

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-04-25T19:25:46Z
last_update: 2026-04-25T19:36:27Z
date_finished: 2026-04-25T19:36:27Z
---

# T-1263: D5 anomaly filter refinement + working tree cleanup

## Context

D5 lifecycle detector flags 11 human-owned completed tasks as "fast cycle <5min" anomalies, but all 11 have exactly 1 commit referencing them — the retroactive-workflow signature (work done first, task created+closed to capture it). Real anomaly is 0 commits + fast cycle (work without captured artifact). Filter 3 currently requires `commits >= 2`; relaxing to `commits >= 1` eliminates the false-positive class without weakening detection of the real signal. Also: clean stray `/opt/termlink/sys` PostScript file (accidental ImageMagick output redirect from 2026-04-20).

## Acceptance Criteria

### Agent
- [x] Filter 3 in `.agentic-framework/agents/audit/audit.sh` D5 detector relaxed from `commits >= 2` to `commits >= 1` with comment explaining retroactive-workflow rationale — applied 2026-04-25; commit `1955eb5d`.
- [x] `fw audit` D5 anomaly count drops from 15 → ≤4 (stuck-active tasks only, no fast-completion false positives) — verified post-patch: `D5: Task lifecycle — 4 anomaly(s): T-174(37d-active) T-173(37d-active) T-212(35d-active) T-175(37d-active)` (all stuck-active).
- [x] Stray `/opt/termlink/sys` PostScript file removed — `rm /opt/termlink/sys`; verified `test ! -e /opt/termlink/sys`.
- [x] Upstream mirror: same audit.sh patch landed in `/opt/999-Agentic-Engineering-Framework`, pushed to onedev — commit `b15ed567`. Path was direct git-via-`-C` (not termlink dispatch — equivalent end state, push succeeded after fixing pre-existing L-275 YAML quoting).

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

# Filter 3 patched
grep -q "commits >= 1" .agentic-framework/agents/audit/audit.sh
# Stray sys file removed
test ! -e /opt/termlink/sys
# D5 false positives gone — no "min,human" patterns in the warning line
test "$(.agentic-framework/bin/fw audit 2>&1 | grep 'D5: Task lifecycle' | grep -cE 'min,human')" = "0"

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

### 2026-04-25T19:25:46Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1263-d5-anomaly-filter-refinement--working-tr.md
- **Context:** Initial task creation

### 2026-04-25T19:36:27Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
