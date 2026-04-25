---
id: T-1263
name: "D5 anomaly filter refinement + working tree cleanup"
description: >
  D5 anomaly filter refinement + working tree cleanup

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-04-25T19:25:46Z
last_update: 2026-04-25T19:25:46Z
date_finished: null
---

# T-1263: D5 anomaly filter refinement + working tree cleanup

## Context

D5 lifecycle detector flags 11 human-owned completed tasks as "fast cycle <5min" anomalies, but all 11 have exactly 1 commit referencing them — the retroactive-workflow signature (work done first, task created+closed to capture it). Real anomaly is 0 commits + fast cycle (work without captured artifact). Filter 3 currently requires `commits >= 2`; relaxing to `commits >= 1` eliminates the false-positive class without weakening detection of the real signal. Also: clean stray `/opt/termlink/sys` PostScript file (accidental ImageMagick output redirect from 2026-04-20).

## Acceptance Criteria

### Agent
- [ ] Filter 3 in `.agentic-framework/agents/audit/audit.sh` D5 detector relaxed from `commits >= 2` to `commits >= 1` with comment explaining retroactive-workflow rationale
- [ ] `fw audit` D5 anomaly count drops from 15 → ≤4 (stuck-active tasks only, no fast-completion false positives)
- [ ] Stray `/opt/termlink/sys` PostScript file removed
- [ ] Upstream mirror: same audit.sh patch landed in `/opt/999-Agentic-Engineering-Framework` via termlink dispatch --workdir, pushed to onedev

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
# D5 false positives gone (≤4 stuck-active anomalies acceptable, no fast-completion ones)
.agentic-framework/bin/fw audit 2>&1 | grep "D5: Task lifecycle" | grep -vE "anomaly\(s\): T-[0-9]+\([0-9]+min,human\)"

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
