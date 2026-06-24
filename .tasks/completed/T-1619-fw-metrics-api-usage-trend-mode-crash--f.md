---
id: T-1619
name: "fw metrics api-usage trend mode crash — F-7 unpack mismatch"
description: >
  fw metrics api-usage trend mode crash — F-7 unpack mismatch

status: work-completed
workflow_type: build
owner: agent
horizon: null
tags: []
components: []
related_tasks: []
created: 2026-05-06T12:03:14Z
last_update: 2026-05-06T12:08:40Z
date_finished: 2026-05-06T12:08:40Z
---

# T-1619: fw metrics api-usage trend mode crash — F-7 unpack mismatch

## Context

PL-130: `fw metrics api-usage` (default invocation, trend mode) crashes at `agents/metrics/api-usage.sh:382` with `ValueError: too many values to unpack (expected 7)`. `stats_for_window()` returns 10 values; line 382 unpacks into 7. Other call-sites (273, 303, 337, 398, 452) were updated to 10-tuple — line 382 was missed. Workaround: `--cut-ready --last-Nd N`. F-7 finding from T-1166 cut-readiness audit.

## Acceptance Criteria

### Agent
- [x] Line 382 unpacks 10 values matching `stats_for_window()` return shape (PL-152: aggregation/return-shape regression). Evidence: grep verifies `_, total, legacy_total, _, _, _, legacy_unattr, _, _, _ = stats_for_window(d)` present.
- [x] `fw metrics api-usage` (default invocation, trend mode) prints the 4-window table without traceback. Evidence: live output shows `60d  195690  7867  4.02%  FAIL` — table renders, no traceback.
- [x] `--cut-ready --last-Nd 1` still works (regression check on the workaround path). Evidence: command exits 1 (correct: gate is NOT READY) — exit code in expected range, no crash.
- [x] Upstream /opt/999-Agentic-Engineering-Framework patched + pushed to onedev (Channel 1 mirror pattern) so the fix survives next `fw upgrade`. Evidence: commit `adf465d76ec3087662578798257ef46ba52f1160` on origin/master, HEAD == origin/master verified.

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

bash -c 'output=$(.agentic-framework/bin/fw metrics api-usage 2>&1); echo "$output" | grep -q "Traceback" && exit 1; echo "$output" | grep -qE "60d  *[0-9]" && exit 0; exit 1'
.agentic-framework/bin/fw metrics api-usage --cut-ready --last-Nd 1 >/dev/null 2>&1; [ $? -le 1 ]
grep -n "stats_for_window(d)" .agentic-framework/agents/metrics/api-usage.sh | grep -q "_, total, legacy_total, _, _, _, legacy_unattr, _, _, _ ="

## RCA

**Symptom:** `fw metrics api-usage` (default invocation = trend mode) crashes with `ValueError: too many values to unpack (expected 7)` at `agents/metrics/api-usage.sh:382`. Operator cannot see the 4-window legacy-traffic trend table without first remembering the `--cut-ready --last-Nd N` workaround.

**Root cause:** `stats_for_window(days)` returns a 10-tuple. Five call-sites (lines 273, 303, 317, 337, 398, 452) were updated when the return shape grew (last_seen_callers/pids/ips were appended). Line 382 in the trend loop was missed and still unpacks into 7 vars.

**Why structurally allowed:** No automated check that all call-sites of a function in a heredoc Python script unpack the same arity as the `return (...)` statement. Bash-embedded Python has no static type checker. Trend mode is the *default* invocation path but no test exercises it — the test surface (if any) only covered `--cut-ready` (which works). PL-152 (aggregation-counter regression rule) generalizes: any cross-cutting structural change needs all call-sites enumerated and verified.

**Prevention:** PL-152 lessons applied — when growing a multi-value return, enumerate all unpack sites via grep before commit. Verification step in this task explicitly grep-checks the line shape, so a future regression on the same line fails the gate.
<!-- REQUIRED for bug-class tasks (workflow_type=build with bug-tag, OR title matches
     fix/bug/rca/broken/crash/error/regression/fail/hotfix).
     Non-bug-class tasks may leave this section empty or remove it.

     For bug-class, fill in:
       **Symptom:** what was observed (the user-facing manifestation).
       **Root cause:** the specific structural/logical gap — not "the code was wrong".
       **Why structurally allowed:** what in the framework/code/tooling let this go undetected.
       **Prevention:** what catches the next instance (test/lint/gate/doc/learning) — distinct from the fix itself.

     The completion gate (T-1550, G-019) blocks --status work-completed when
     bug-class AND this section is empty/template-only. Use --skip-rca to bypass (logged).
-->

## Evolution

<!-- REQUIRED for arc-tagged build tasks (tags include arc:*). Captures how
     understanding evolved during build — what was learned that wasn't known at
     filing, what in the original plan no longer fits, what triggered pivots
     or new sub-tasks. Mandatory at slice boundaries (when applicable) and
     before --status work-completed.

     Origin: T-1717 grill Q4 — "the understanding of what we need and want
     evolves with the process of materialisation." Structural counter to §ACD:
     spec-vs-build divergence is logged as soon as it happens, not lost as
     folklore.

     Format (one entry per slice boundary or significant insight):
       ### YYYY-MM-DD — [topic]
       - **What changed:** [what we learned that we didn't know at filing]
       - **Plan impact:** [what in the plan no longer fits]
       - **Triggered:** [new sub-task / pivot / scope cut, with task ID if filed]

     The completion gate (T-1718) blocks --status work-completed when this
     section exists but is empty/template-only. Use --skip-evolution to bypass
     (logged Tier-2). Non-arc tasks may leave this empty.
-->

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

### 2026-05-06T12:03:14Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1619-fw-metrics-api-usage-trend-mode-crash--f.md
- **Context:** Initial task creation

### 2026-05-06T12:08:40Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
