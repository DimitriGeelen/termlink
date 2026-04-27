---
id: T-1308
name: "fw metrics api-usage: multi-window trend output (1d/7d/30d/60d) for incremental feedback"
description: >
  fw metrics api-usage: multi-window trend output (1d/7d/30d/60d) for incremental feedback

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-04-27T12:01:40Z
last_update: 2026-04-27T12:03:58Z
date_finished: null
---

# T-1308: fw metrics api-usage: multi-window trend output (1d/7d/30d/60d) for incremental feedback

## Context

T-1304's `fw metrics api-usage` defaults to a 60-day window (matching T-1166's gate threshold). User correctly flagged this as a feedback-loop problem: operators should see the legacy% trajectory **incrementally** — daily, weekly, monthly — not wait two months for a binary go/no-go.

**Fix.** Default invocation (no `--last-Nd`) prints a *trend report* showing legacy% across 1d / 7d / 30d / 60d windows side-by-side, so operators see the trajectory in real time. `--last-Nd N` retains its existing single-window behavior for CI gates (e.g., T-1166 entry check stays exactly as specified). The exit code in trend mode reflects the 60d (gate) window so CI behavior is unchanged.

## Acceptance Criteria

### Agent
- [x] `.agentic-framework/agents/metrics/api-usage.sh` adds a "trend" mode (default when `--last-Nd` is not passed) that computes per-window stats for [1, 7, 30, 60] days and prints a trend table
- [x] Single-window mode (`--last-Nd N`) preserved exactly — same output, same exit code
- [x] Trend table shows: window | total RPCs | legacy count | legacy % | gate-status (PASS/FAIL/N-A) per row
- [x] Trend mode exit code = 60d row's gate status (so existing CI usage is unchanged when invoked with no flag)
- [x] Top-10 method tally still printed (using the 60d window, since that's the canonical T-1166 gate window)
- [x] Operator docs (`docs/operations/api-usage-metrics.md`) updated with: "Reading the report" section showing the trend output; "When to look" guidance — start checking from day 1, watch the 1d/7d trajectory toward zero
- [x] Smoke test: against T-1306-fixture-pass (1 legacy in 201 calls, all recent), trend mode shows PASS at all four windows; against T-1306-fixture (1 legacy in 10 over 7d, 4 in 13 over 30d), trend shows FAIL@1d/7d, FAIL@30d/60d
- [x] `bash .agentic-framework/agents/metrics/api-usage.sh --runtime-dir <fixture>` (no other flags) produces the trend report and exits with the correct gate code
- [x] Sync to upstream framework via patched copy + git -C commit + push (same as T-1304)

### Human
<!-- Criteria requiring human verification (UI/UX, subjective quality). Not blocking.
     Remove this section if all criteria are agent-verifiable.
     Each criterion MUST include Steps/Expected/If-not so the human can act without guessing.
     Optionally prefix with [RUBBER-STAMP] or [REVIEW] for prioritization.
     Example:
       - [x] [REVIEW] Dashboard renders correctly
         **Steps:**
         1. Open https://example.com/dashboard in browser
         2. Verify all panels load within 2 seconds
         3. Check browser console for errors
         **Expected:** All panels visible, no console errors
         **If not:** Screenshot the broken panel and note the console error
-->

## Verification

grep -q "trend (use --last-Nd N for single-window CI gate)" .agentic-framework/agents/metrics/api-usage.sh
grep -q "'Window':>8s" .agentic-framework/agents/metrics/api-usage.sh
grep -q "When to look" docs/operations/api-usage-metrics.md
test -x .agentic-framework/agents/metrics/api-usage.sh
.agentic-framework/bin/fw metrics api-usage --runtime-dir /tmp/T-1304-fixture-pass 2>&1 | grep -q "60d         201"

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

### 2026-04-27T12:01:40Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1308-fw-metrics-api-usage-multi-window-trend-.md
- **Context:** Initial task creation
