---
id: T-1900
name: "Watchtower 403 CSRF error UX — actionable recovery steps on stale-session POST"
description: >
  Watchtower 403 CSRF error UX — actionable recovery steps on stale-session POST

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: [watchtower, bug, ux]
components: []
related_tasks: [T-1306, T-1343, T-1898, T-1899]
created: 2026-05-31T18:58:11Z
last_update: 2026-05-31T18:58:11Z
date_finished: null
---

# T-1900: Watchtower 403 CSRF error UX — actionable recovery steps on stale-session POST

## Context

Operator hit `Forbidden — termlink` on Watchtower at end of S-2026-0531-2005 when trying to act on T-1898/T-1899 inception decisions. Reproduced as a CSRF mismatch on POST `/inception/<id>/decide` (server-side path proven clean — Set-Cookie issued correctly on GET, 25-day-uptime gunicorn with persisted secret_key). Symptom: browser session had stale `_csrf_token` (signed cookie pre-dating the persisted key, or cookie-blocking, or referrer policy stripping the session cookie). The 403 page rendered today says **"CSRF token missing or invalid"** with nothing else — no recovery path, no plain-language explanation, no CLI fallback. Operators don't know to clear cookies. This task improves the 403 error page so a CSRF-class 403 surfaces actionable recovery steps **without weakening CSRF protection**.

Scope: error-page UX only. Do NOT relax csrf_protect logic (regenerate-on-mismatch would silently neuter CSRF). Server-side fix surface is `app.py forbidden()` errorhandler + `_error.html` (or a new `_csrf_error.html` partial). Reproduction recipe captured in T-1900 Updates.

## Acceptance Criteria

### Agent
- [x] CSRF-class 403 (description starts with "CSRF token") renders a distinct error template path with: (a) plain-language explanation that the form session expired, (b) "Refresh and retry" instruction with browser keystrokes (Cmd-R / F5), (c) "Clear cookies for this site if refresh doesn't fix it" instruction with browser-menu hint, (d) CLI fallback path (link to docs OR generic `fw` hint).
- [x] Non-CSRF 403 (e.g. fabric.py:406 "Forbidden") still renders the existing plain error page — recovery hints do NOT show for non-CSRF 403s. Verified: CSRF body=34448 bytes (with panel), non-CSRF body=32986 bytes (terse), `session expired` only in CSRF body.
- [x] `csrf_protect()` logic in `app.py:92-111` is UNCHANGED — POST without valid token still 403s (no security regression). Verified: pytest `TestCSRF` 4 tests pass + curl POST returns `403`.
- [x] CSRF 403 page returns HTTP 403 (status code unchanged — only body changes).
- [x] No untested template variables — rendering CSRF 403 page does not raise Jinja `UndefinedError`. Verified by grep for `{{` in rendered body (none).
- [x] Existing 404/500 errorhandlers untouched — only `forbidden()` block edited; verified by reading the diff.

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

# CSRF-class POST without token still returns 403 (security unchanged):
test "$(curl -s -X POST http://localhost:3003/inception/T-1898/decide -d 'decision=defer&rationale=x' -w '%{http_code}' -o /dev/null)" = "403"
# CSRF 403 page now contains the new recovery-step language:
curl -s -X POST http://localhost:3003/inception/T-1898/decide -d 'decision=defer&rationale=x' | grep -qi "refresh"
curl -s -X POST http://localhost:3003/inception/T-1898/decide -d 'decision=defer&rationale=x' | grep -q "session expired"
# CSRF 403 page contains the CLI fallback hint:
curl -s -X POST http://localhost:3003/inception/T-1898/decide -d 'decision=defer&rationale=x' | grep -q "fw inception decide"
# Existing 404 path still works:
test "$(curl -s -o /dev/null -w '%{http_code}' http://localhost:3003/inception/T-9999999)" = "404"
# GET on a valid inception page still works (no regression):
test "$(curl -s -o /dev/null -w '%{http_code}' http://localhost:3003/inception/T-1898)" = "200"

## RCA

**Symptom:** Operator hit `Forbidden — termlink` plain page on Watchtower 2026-05-31 when trying to act on T-1898/T-1899 inception decisions. Localhost curl returned 200; operator browser returned 403. Reproduction proves the 403 fires only on `csrf_protect()` mismatch on POST/PATCH/PUT/DELETE — GET path is clean.

**Root cause (UX, not security):** The 403 errorhandler at `app.py:355` renders `_error.html` with `error_message=str(e.description)`, surfacing the raw CSRF abort message `"CSRF token missing or invalid"` with no recovery guidance. An operator landing on this page has no in-product path to recover — they have to ask an agent, search docs, or give up. T-1306 (persisted secret_key, prevented one class of CSRF breakage after restarts) was the structural fix on the back end; this task is the matching front-end UX fix.

**Why structurally allowed:** T-1306 closed the "every restart breaks CSRF" class but assumed the residual 403 surface (genuinely stale cookies, browser-cookie-blocking modes, very old visitors) was rare enough to not warrant UX investment. The 2026-05-31 incident shows even one 403 stalls an operator at the moment they're trying to act on a pending decision — a structural multiplier on the cost of the rare event. No lint/check catches "raw exception strings reach the operator" because Flask's errorhandler model encourages exactly that pattern.

**Prevention:** Distinct error template path for CSRF-class 403 (`description` startswith `CSRF token`) renders plain-English explanation + browser keystroke for refresh + cookie-clear instruction + CLI fallback. Non-CSRF 403s (e.g. fabric.py:406) keep the existing terse path so security messaging isn't diluted. A small unit-test-equivalent verification curl in the task's `## Verification` block re-runs the recovery-text check on every completion — if a future refactor regresses the UX, the gate blocks. No new framework lint required; the verification recipe IS the catch.

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

## Decision

<!-- Filled at completion of inception tasks via:
     fw inception decide T-XXX go|no-go|defer --rationale "..."

     For non-inception tasks this section is ignored. Kept in template
     so `fw inception decide` (lib/inception.sh) finds the anchor heading
     without auto-creating; T-1832 added auto-create as fallback for
     legacy tasks lacking this section. -->

## Updates

### 2026-05-31T18:58:11Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1900-watchtower-403-csrf-error-ux--actionable.md
- **Context:** Initial task creation
