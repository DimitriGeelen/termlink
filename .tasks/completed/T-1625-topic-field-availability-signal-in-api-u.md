---
id: T-1625
name: "topic-field availability signal in api-usage.sh — disambiguate no-traffic vs pre-T-1622 hub"
description: >
  topic-field availability signal in api-usage.sh — disambiguate no-traffic vs pre-T-1622 hub

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-05-06T16:04:09Z
last_update: 2026-05-06T16:12:50Z
date_finished: 2026-05-06T16:12:50Z
---

# T-1625: topic-field availability signal in api-usage.sh — disambiguate no-traffic vs pre-T-1622 hub

## Context

T-1622 added a per-(method, topic) breakdown to `fw metrics api-usage`
("Legacy callers by topic"). When the table is silent, the operator
cannot tell two states apart:

  (a) **No traffic** — `legacy_total > 0` but `event.broadcast` is gone:
      cut is ready for that channel.
  (b) **Pre-T-1622 hub** — legacy traffic exists, but the audit log
      entries don't carry the `topic` field yet (hub binary predates
      T-1622).

Both render as "no rows" today. That ambiguity is the last UX wart
on the T-1622 ship and the operator-facing surface the T-1166 cut
authorization will read.

Fix: a `legacy_topic_coverage_for_window(days)` helper that returns
`(legacy_total, with_topic)`. Use it to:
- emit a `legacy_topic_coverage` JSON field in both single-window and
  trend modes (downstream tooling reads this as a gate signal),
- in the text output, print one explicit hint line when the table is
  empty AND `legacy_total > 0` ("(topic field unavailable — hub may
  predate T-1622)"), or when partial-coverage ("X/Y entries carry
  topic — older entries lack the field").

PL-152 discipline: helper kept SEPARATE from `stats_for_window`'s
10-tuple return. Same isolation rule that T-1622 followed and
T-1623 enforces statically.

Channel 1 upstream-mirror pattern: edit consumer-side first to test,
then dispatch matching patch to `/opt/999-Agentic-Engineering-Framework`.

## Acceptance Criteria

### Agent
- [x] `legacy_topic_coverage_for_window(days)` helper added to api-usage.sh, returns `(legacy_total, with_topic)` for the window
- [x] Helper is SEPARATE from `stats_for_window` — does not mutate the 10-tuple return (PL-152)
- [x] `legacy_topic_coverage` field present in single-window JSON: `{"total": N, "with_topic": M}` (verified: `mode: single-window` shows `{'total': 4724, 'with_topic': 0}`)
- [x] `legacy_topic_coverage` field present in trend JSON (60d window) (verified: top-level field, `{'total': 7870, 'with_topic': 0}`)
- [x] Single-window text: when `legacy_total > 0` and topic table empty, prints `"(topic field unavailable — hub may predate T-1622)"` (verified live on `--last-Nd 7`)
- [x] Single-window text: when `legacy_total > 0` and `with_topic < legacy_total` and `with_topic > 0`, prints `"(topic field present on X/Y legacy entries)"` (path coded; live hub has 0 with-topic so other branch fires today)
- [x] Trend mode text (60d): same two hint lines (verified live on default invocation — "(topic field unavailable…)" appears)
- [x] Existing T-1623 PL-152 static check still passes: `bash tests/test_pl152_counter_arity_static.sh` (9/9 PASS, 7 call sites, arity 10 — helper isolation discipline held)
- [x] api-usage.sh JSON parses end-to-end (`python3 -c 'import json,sys; json.load(sys.stdin)'` → JSON OK)
- [x] Upstream `/opt/999-Agentic-Engineering-Framework/agents/metrics/api-usage.sh` carries the same patch (commit `1664057dd` on origin/master, 5/5 patches applied via /tmp/patcher_t1625.py, sha256 `f98b3de4…`)

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
grep -q 'def legacy_topic_coverage_for_window' /opt/termlink/.agentic-framework/agents/metrics/api-usage.sh
grep -q '"legacy_topic_coverage"' /opt/termlink/.agentic-framework/agents/metrics/api-usage.sh
grep -q 'topic field unavailable' /opt/termlink/.agentic-framework/agents/metrics/api-usage.sh
grep -q 'topic field present on' /opt/termlink/.agentic-framework/agents/metrics/api-usage.sh
bash /opt/termlink/tests/test_pl152_counter_arity_static.sh

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

### 2026-05-06T16:04:09Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1625-topic-field-availability-signal-in-api-u.md
- **Context:** Initial task creation

### 2026-05-06T16:12:50Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
