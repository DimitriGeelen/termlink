---
id: T-1623
name: "PL-152 static check — counter-arity drift detection for stats_for_window unpack sites"
description: >
  PL-152 static check — counter-arity drift detection for stats_for_window unpack sites

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-05-06T13:38:36Z
last_update: 2026-05-06T13:40:49Z
date_finished: 2026-05-06T13:40:49Z
---

# T-1623: PL-152 static check — counter-arity drift detection for stats_for_window unpack sites

## Context

PL-152 (aggregation-counter regression rule) has fired three times in one
week: T-1615, T-1619 (metrics trend mode crash), T-1620 (MCP fail_count
tautology). The pattern: a function returning N values is updated to return
N+M, but one or more call sites silently keep the old unpack arity,
producing either a ValueError (Python) or a correctness bug (Rust).

T-1621 added a smoke test specifically for `fw metrics api-usage` trend
mode. T-1622 *deliberately* added a separate helper rather than extending
`stats_for_window`'s 10-tuple — discipline I executed manually. This task
converts that discipline into a structural test: scan `api-usage.sh`,
extract `stats_for_window`'s return arity, find every call site, assert
all unpacks match.

Level C escalation per Error Escalation Ladder. Target: make the next
counter-arity drift fail at test-time instead of operator-time.

## Acceptance Criteria

### Agent
- [x] `tests/test_pl152_counter_arity_static.sh` exists and is executable
- [x] Test parses `.agentic-framework/agents/metrics/api-usage.sh`, extracts the tuple arity of `stats_for_window`'s `return (...)` expression (extracted: 10)
- [x] Test finds every `= stats_for_window(...)` call site and verifies the unpack-LHS arity matches the return arity (7 sites found, all match)
- [x] Test exits 0 when all sites match, prints PASS/FAIL summary
- [x] Test exits non-zero when any site mismatches, prints the offending line + expected/got arity (negative control: line 357 corrupted to arity 7 → flagged with `arity 7 != expected 10`)
- [x] Passes today against the current api-usage.sh (7 call sites, 10-tuple return — 9/9 PASS)
- [x] Negative-control verified: corrupting one site to 7-arity made the test fail with exit 1; restoring brought it back to 9/9 PASS

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

# Shell commands that MUST pass before work-completed. One per line.
# Lines starting with # are comments (skipped). Empty lines ignored.
# The completion gate runs each command — if any exits non-zero, completion is blocked.
#
# Toolchain hint (L-291): if you edited *.vbproj/*.csproj/*.xaml add `dotnet build`;
# *.go → `go build ./...`; Cargo.toml → `cargo check`; tsconfig.json → `tsc --noEmit`;
# pom.xml → `mvn -q compile`. P-011 runs only what you write — broken builds slip
# past otherwise (origin: 003-NTB-ATC-Plugin T-077, broken WPF DLL on master 5 days).

bash tests/test_pl152_counter_arity_static.sh
test -x tests/test_pl152_counter_arity_static.sh
bash tests/test_t1619_metrics_trend_smoke.sh 2>&1 | tail -3

## RCA

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

### 2026-05-06T13:38:36Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1623-pl-152-static-check--counter-arity-drift.md
- **Context:** Initial task creation

### 2026-05-06T13:40:49Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
