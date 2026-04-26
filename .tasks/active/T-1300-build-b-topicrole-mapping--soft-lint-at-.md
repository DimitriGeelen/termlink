---
id: T-1300
name: "Build B: Topic↔role mapping + soft-lint at emit (hub-side)"
description: >
  Per T-1297 GO: hub-side YAML mapping (~/var/lib/termlink/topic_roles.yaml or similar) + soft-lint at event emit. 10 prefix rules + 4 exempt categories cover 95% of current topic catalog. Warning-only (NEVER reject); emit a sentinel event (e.g. routing.lint.warning) to subscribed channels. Hot-reload on SIGHUP. Compares topic prefix against caller session's roles (and payload.relay_target/needs/from when present per Spike 1 design signal). Estimate: 1 dev-day. Reversible: lint can be globally disabled via config. Evidence: docs/reports/T-1297-termlink-agent-routing-discipline.md § Spike 3.

status: captured
workflow_type: build
owner: human
horizon: next
tags: [termlink, routing, lint, T-1297-child, hub]
components: []
related_tasks: [T-1297]
created: 2026-04-26T21:19:39Z
last_update: 2026-04-26T21:19:39Z
date_finished: null
---

# T-1300: Build B: Topic↔role mapping + soft-lint at emit (hub-side)

## Context

<!-- One sentence for small tasks. Link to design docs for substantial ones. -->

## Acceptance Criteria

### Agent
<!-- Criteria the agent can verify (code, tests, commands). P-010 gates on these. -->
- [ ] [First criterion]
- [ ] [Second criterion]

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

### 2026-04-26T21:19:39Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1300-build-b-topicrole-mapping--soft-lint-at-.md
- **Context:** Initial task creation
