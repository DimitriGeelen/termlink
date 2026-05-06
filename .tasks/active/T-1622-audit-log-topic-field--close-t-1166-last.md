---
id: T-1622
name: "audit-log topic field — close T-1166 last-mile visibility for legacy event.broadcast residue"
description: >
  audit-log topic field — close T-1166 last-mile visibility for legacy event.broadcast residue

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-05-06T13:27:35Z
last_update: 2026-05-06T13:27:35Z
date_finished: null
---

# T-1622: audit-log topic field — close T-1166 last-mile visibility for legacy event.broadcast residue

## Context

T-1166 cut-readiness now passes for 1d/2d windows but the operator can't ID *which topics* are still being broadcast via legacy `event.broadcast`. The audit log (T-1304) records `method` only — the params (which carry `topic`) are dropped. For event.broadcast, the method-level "still seeing N calls" tells operator nothing about which channel migration is incomplete. T-1417 (legacy fanout migration) needs that data to know what's left.

This ship: capture `topic` from request params at the dispatch site, persist as additive `"topic":"..."` field in `rpc-audit.jsonl`, surface in `fw metrics api-usage` legacy breakdown so operators can answer "which topics are the residue going to?" without SSH+jq.

Schema is additive (omitted when None / non-event.broadcast) — existing readers ignore the new field.

## Acceptance Criteria

### Agent
- [ ] `rpc_audit::build_audit_line` accepts an additional `topic: Option<&str>` and emits `"topic":"..."` when Some+non-empty (additive — omitted when None)
- [ ] Server dispatch site (server.rs ~597) extracts `params.topic` (best-effort) and threads it to `record()`
- [ ] Existing rpc_audit unit tests still pass (no semantic regression)
- [ ] New unit test: `build_audit_line` with topic Some emits the field; with None omits it
- [ ] `fw metrics api-usage --json` for legacy event.broadcast surfaces a `top_topics` array (additive) bucketed by topic when present in audit lines
- [ ] `cargo build -p termlink-hub` succeeds
- [ ] `bash tests/test_t1619_metrics_trend_smoke.sh` still passes (no metrics regression)

## Verification

cargo build -p termlink-hub --quiet 2>&1 | tail -5
cargo test -p termlink-hub --lib --quiet rpc_audit 2>&1 | tail -10
bash tests/test_t1619_metrics_trend_smoke.sh 2>&1 | tail -8
grep -q '"topic":' crates/termlink-hub/src/rpc_audit.rs
grep -qE 'params\.(get\("topic"\)|topic)' crates/termlink-hub/src/server.rs

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

### 2026-05-06T13:27:35Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1622-audit-log-topic-field--close-t-1166-last.md
- **Context:** Initial task creation
