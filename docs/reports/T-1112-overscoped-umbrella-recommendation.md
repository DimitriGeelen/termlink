# T-1112 — Overscoped umbrella inception (NO-GO recommendation)

**Date:** 2026-04-18
**Status:** Recommendation NO-GO; awaiting human decision.
**Origin task:** `.tasks/active/T-1112-backfill-recommendation-sections-for-51-.md`

## What was proposed

T-1112 asked one agent session to backfill Recommendation sections for 51 stalled inception tasks in a single sweep, "mirroring T-1110 pattern but larger batch."

## Why NO-GO

Per CLAUDE.md task-sizing rule:

> "One inception = one question. Umbrella inceptions that bundle independent explorations create all-or-nothing decisions and coarse progress tracking."

The 51 inceptions cover independent problem domains (protocol design, hub auth, MCP tooling, framework hooks, deployment, packaging, observability, etc). Each requires problem-specific code/system exploration. A bulk pass would either:

1. Produce shallow boilerplate recommendations (silently bypassing the recommendation-as-thinking-trail intent), or
2. Exceed any single session's context budget (no upper bound on per-task exploration depth).

T-1110, the cited precedent, was completed with empty acceptance criteria and is itself an example of the anti-pattern, not a reusable model.

## Empirical evidence (same session)

Two adjacent inception recommendations were filled in this session under their own task IDs:

- **T-1071** — Framework improvements from termlink protocol-skew → required reading `crates/termlink-protocol/src/control.rs`, grepping for `protocol_version` enforcement, walking commit history for KeyEntry. Produced a GO with 3 follow-up tasks.
- **T-1122** — Migrate Watchtower from Werkzeug to production WSGI → required reading `web/app.py` socketio integration, weighing 3 WSGI servers against the multi-worker hook constraint. Produced a DEFER (the actual root cause is systemd, not the WSGI server).

Each took non-trivial focused effort. Multiplied by 51, this is at least N sessions of work, where N is large.

## Proposed replacement

Two follow-up tasks:

1. **`fw inception triage` tooling** — extend `fw inception status` to flag pending-decision inceptions whose Recommendation section is empty, sorted by age. Operator-facing triage tool, not bulk automation.
2. **Per-session triage habit** — each future agent session that has spare capacity picks ONE pending inception, fills the recommendation, runs `fw task review`, and stops. The backlog drains through normal work, not through a one-shot bulk task.

## What this report does NOT do

- It does not enumerate which 51 inceptions need recommendations (that is itself a triage step, not a finding).
- It does not bulk-assert recommendations on those 51 tasks. Doing so would defeat the purpose of the recommendation gate.

## Decision path

Human runs:
```
fw inception decide T-1112 no-go --rationale "Overscoped umbrella; replaced by per-session triage + fw inception triage tooling"
```
