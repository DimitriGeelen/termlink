---
id: T-1621
name: "T-1619 regression smoke — pin metrics api-usage trend mode (PL-152 prevention)"
description: >
  T-1619 regression smoke — pin metrics api-usage trend mode (PL-152 prevention)

status: work-completed
workflow_type: test
owner: agent
horizon: null
tags: []
components: []
related_tasks: []
created: 2026-05-06T12:22:16Z
last_update: 2026-05-06T12:24:16Z
date_finished: 2026-05-06T12:24:16Z
---

# T-1621: T-1619 regression smoke — pin metrics api-usage trend mode (PL-152 prevention)

## Context

T-1619 fixed a multi-call-site arity drift in `agents/metrics/api-usage.sh` (line 382 trend-loop). PL-152 prevention requires a regression smoke test that exercises the actual binary in the actual environment and pins the fix mechanically. Modeled on `tests/test_g054_completion_smoke.sh` — same shape, different bug.

## Acceptance Criteria

### Agent
- [x] `tests/test_t1619_metrics_trend_smoke.sh` exists and is executable. Evidence: `test -x` succeeds, file is +x via `chmod +x`.
- [x] Test runs `fw metrics api-usage` (default invocation, no flags) and asserts output contains the trend table headers and at least one window row, AND asserts no `Traceback`, no `ValueError`, no Python exception text. Evidence: live test output shows 3 PASS lines for the trend-mode block.
- [x] Test runs `fw metrics api-usage --cut-ready --last-Nd 1` and asserts exit code is 0 or 1 (gate PASS/NOT-READY are both legitimate; non-zero non-1 means crash). Evidence: live test output shows `cut-ready mode exit=1 (legitimate gate outcome, not a crash)`.
- [x] Running the test passes today: `bash tests/test_t1619_metrics_trend_smoke.sh` exits 0. Evidence: 4/4 PASS, 0 FAIL.

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

test -x tests/test_t1619_metrics_trend_smoke.sh
bash tests/test_t1619_metrics_trend_smoke.sh

# Toolchain hint (L-291): if you edited *.vbproj/*.csproj/*.xaml add `dotnet build`;
# *.go → `go build ./...`; Cargo.toml → `cargo check`; tsconfig.json → `tsc --noEmit`;
# pom.xml → `mvn -q compile`. P-011 runs only what you write — broken builds slip
# past otherwise (origin: 003-NTB-ATC-Plugin T-077, broken WPF DLL on master 5 days).

## RCA

(This task is a *preventive* test, not a bug fix. The bug itself was diagnosed and resolved in T-1619; this task ships the regression smoke that pins it. RCA below restates T-1619's analysis to satisfy G-019, since the title contains "regression".)

**Symptom:** `fw metrics api-usage` (default invocation = trend mode) crashed with `ValueError: too many values to unpack (expected 7)` at `agents/metrics/api-usage.sh:382`.

**Root cause:** `stats_for_window()` returns a 10-tuple. Five call-sites had been updated to 10-unpack when the return shape grew (T-1414 added `last_seen_callers/pids/ips`). Line 382 in the trend-mode loop was missed.

**Why structurally allowed:** Bash-embedded Python has no static type checker. Trend mode is the *default* invocation but no test exercised it — operators only used `--cut-ready --last-Nd N` (different code path). PL-152 generalizes: when a multi-value return shape changes, all unpack call-sites must be enumerated.

**Prevention:** This task ships `tests/test_t1619_metrics_trend_smoke.sh` — a real-binary smoke that runs `fw metrics api-usage` (default + --cut-ready paths) and asserts no Python traceback, table renders, exit codes are sane. Distinct from T-1619's grep-based AC verification — this catches *any* future regression of the unpack arity (or of any code in stats_for_window's call chain) by exercising the live tool.
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

### 2026-05-06T12:22:16Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1621-t-1619-regression-smoke--pin-metrics-api.md
- **Context:** Initial task creation

### 2026-05-06T12:24:16Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
