---
id: T-1276
name: "Fix fw context add-learning to escape embedded double-quotes (PL-080/PL-081)"
description: >
  Fix fw context add-learning to escape embedded double-quotes (PL-080/PL-081)

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-04-25T20:44:37Z
last_update: 2026-04-25T20:46:22Z
date_finished: 2026-04-25T20:46:22Z
---

# T-1276: Fix fw context add-learning to escape embedded double-quotes (PL-080/PL-081)

## Context

PL-080 and PL-081 both got corrupted on insert because
`agents/context/lib/learning.sh` lines 75 and 87 print
`learning: "$learning"` without escaping any embedded `"` chars in the
input. Result: any learning text containing inner double-quotes (shell
quoting examples, CLI snippets) breaks YAML parsing on the next
`fw audit` cycle, surfacing as `[FAIL] YAML parse error: learnings.yaml`.
Hit 3 times in 2 sessions (L-275 upstream, PL-080, PL-081). Fix the
serializer in awk by escaping `"` → `\"` before printing.

## Acceptance Criteria

### Agent
- [x] `learning.sh` upstream: both awk print blocks (header + END) escape `"` → `\"` in $learning before printing
- [x] Local sanity test: live `fw context add-learning` with embedded `"double"` quotes produces valid-YAML learnings.yaml
- [x] Mirrored to /opt/999-AEF + committed + pushed to onedev master (commit 7c8579d6, pushed 2026-04-25T20:46Z)

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

grep -q 'gsub.*\\\\"' /opt/999-Agentic-Engineering-Framework/agents/context/lib/learning.sh
python3 -c 'import yaml; yaml.safe_load(open(".context/project/learnings.yaml")); print("OK")'
test -n "$(git -C /opt/999-Agentic-Engineering-Framework log --oneline -5 | grep 'T-1276')"

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

### 2026-04-25T20:44:37Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1276-fix-fw-context-add-learning-to-escape-em.md
- **Context:** Initial task creation

### 2026-04-25T20:46:22Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
