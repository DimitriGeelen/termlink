---
id: T-1416
name: "api-usage --cut-ready: binary gate on attributable-only legacy traffic"
description: >
  api-usage --cut-ready: binary gate on attributable-only legacy traffic

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-04-30T07:12:28Z
last_update: 2026-04-30T07:14:47Z
date_finished: 2026-04-30T07:14:47Z
---

# T-1416: api-usage --cut-ready: binary gate on attributable-only legacy traffic

## Context

T-1166 entry gate (`fw metrics api-usage`) currently does a statistical
check: `legacy_pct ≤ gate_pct` over a rolling window. T-1414 split the
legacy total into `legacy_attributable` + `legacy_unattributable_pre_t1409`,
but the gate decision still uses the muddled aggregate.

This adds a `--cut-ready` flag that is a stricter binary gate: exit 0 iff
`legacy_attributable == 0` in the chosen window. That's the operator's
real cut-readiness question: "is ANYONE still hitting legacy methods, after
ignoring the pre-deploy backlog that ages out on its own?"

Use cases:
- T-1415 prelude — operator runs `--cut-ready` on every hub before
  authorizing the Tier-2 cut
- CI gate for the post-cut binary build (assert no caller is still active)
- Watchtower future page rendering "X hubs cut-ready, Y not yet" status

## Acceptance Criteria

### Agent
- [x] `api-usage.sh --cut-ready` flag added (works alongside existing flags)
- [x] When `--cut-ready` is present: exit 0 iff `legacy_attributable == 0` in the chosen window (default 7d if `--last-Nd` not given) — verified live: 575 attributable on .107 → exit=1, NOT READY
- [x] When `--cut-ready` is present + `--json`: emits `{"cut_ready": bool, "window_days": N, "legacy_attributable": K, "legacy_unattributable_pre_t1409": K, "audit_file": ...}`
- [x] When `--cut-ready` not present: behavior unchanged (regression-safe — verified `--last-Nd 1 --json` returns full shape)
- [x] Help text (`-h` / `--help`) describes `--cut-ready` distinct from `--gate-pct`
- [x] Vendored copy mirrored to `/opt/999-Agentic-Engineering-Framework/agents/metrics/api-usage.sh`

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

# Help text mentions --cut-ready
.agentic-framework/agents/metrics/api-usage.sh --help 2>&1 | grep -q -- '--cut-ready'
# --cut-ready --json shape is correct (only checking shape, not pass/fail)
bash -c '.agentic-framework/agents/metrics/api-usage.sh --cut-ready --json 2>/dev/null || true' | python3 -c "import json,sys;d=json.load(sys.stdin);assert 'cut_ready' in d, 'missing cut_ready';assert 'window_days' in d, 'missing window_days';assert 'legacy_attributable' in d, 'missing legacy_attributable';print('OK shape', d)"
# Mirrored upstream
diff -q /opt/termlink/.agentic-framework/agents/metrics/api-usage.sh /opt/999-Agentic-Engineering-Framework/agents/metrics/api-usage.sh

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

### 2026-04-30T07:12:28Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1416-api-usage---cut-ready-binary-gate-on-att.md
- **Context:** Initial task creation

### 2026-04-30T07:14:47Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
