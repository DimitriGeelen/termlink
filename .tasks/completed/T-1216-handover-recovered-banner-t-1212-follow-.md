---
id: T-1216
name: "Handover RECOVERED banner (T-1212 follow-up)"
description: >
  Deferred follow-up from T-1212: teach `fw handover` to detect `RECOVERED=1` env (set by session-silent-scanner.sh) and prepend a `[recovered, no agent context]` banner to the generated handover document. Currently the scanner triggers handover for idle sessions but the output is indistinguishable from a normal end-of-session handover, so the next agent cannot tell whether the agent context reflects live state or a post-mortem recovery.

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: [hook, handover, framework-bridge, antifragility]
components: []
related_tasks: [T-1212]
created: 2026-04-24T12:03:15Z
last_update: 2026-04-24T12:08:03Z
date_finished: 2026-04-24T12:08:03Z
---

# T-1216: Handover RECOVERED banner (T-1212 follow-up)

## Context

When session-silent-scanner.sh (T-1212) triggers `fw handover` for an idle
session, it sets `RECOVERED=1`, `RECOVERED_SESSION_ID`, `RECOVERED_AGE_MIN`,
and `RECOVERED_TRANSCRIPT` in the subprocess env. `handover.sh` currently
ignores them. This task makes the banner visible so the NEXT session
immediately knows the handover does not reflect live agent context.

Referenced by `.tasks/completed/T-1212-*.md` §Human ACs + by scanner code
in `.agentic-framework/agents/context/session-silent-scanner.sh`.

## Acceptance Criteria

### Agent
- [x] `.agentic-framework/agents/handover/handover.sh` detects `RECOVERED=1`
      env var before the handover-markdown heredoc and computes a
      `RECOVERED_BANNER` string containing the session_id, age, and
      transcript path.
- [x] Heredoc interpolates `${RECOVERED_BANNER}` immediately after the
      `# Session Handover:` title and before `## Where We Are`. Empty
      string when not recovered — no visual diff for normal handovers.
- [x] Banner uses blockquote markdown (`> `) so it renders distinctively in
      Watchtower and any markdown viewer. (Dropped emoji — handover body is
      markdown plain, consistent with rest of framework output.)
- [x] Smoke test from CLI: running `RECOVERED=1 RECOVERED_SESSION_ID=test-id
      RECOVERED_AGE_MIN=90 RECOVERED_TRANSCRIPT=/tmp/t.jsonl` under a sandbox
      PROJECT_ROOT produced banner at line 33 with `test-id`, `90 min`,
      and `/tmp/t.jsonl` interpolated.
- [x] Smoke test: current `.context/handovers/LATEST.md` (generated
      without RECOVERED env) contains 0 occurrences of the banner phrase
      — confirms empty expansion path is clean.
- [x] Upstream mirror via termlink dispatch: commit `8df5c484` on upstream
      framework master; onedev ref aligned. GitHub mirror lags one commit
      (normal — OneDev PushRepository buildspec auto-mirrors).

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

# handover.sh recognizes RECOVERED env vars
grep -q "RECOVERED:-0" .agentic-framework/agents/handover/handover.sh
# Banner text references the well-known phrase
grep -q "recovered, no agent context" .agentic-framework/agents/handover/handover.sh
# Heredoc interpolates RECOVERED_BANNER after the title
grep -q '${RECOVERED_BANNER}## Where We Are' .agentic-framework/agents/handover/handover.sh

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

### 2026-04-24T12:03:15Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1216-handover-recovered-banner-t-1212-follow-.md
- **Context:** Initial task creation

### 2026-04-24T12:08:03Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
