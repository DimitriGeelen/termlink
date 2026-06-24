---
id: T-1447
name: "framework upgrade + full test suite + fix issues"
description: >
  framework upgrade + full test suite + fix issues

status: work-completed
workflow_type: build
owner: agent
horizon: null
tags: []
components: []
related_tasks: []
created: 2026-05-02T09:59:47Z
last_update: 2026-05-02T11:37:49Z
date_finished: 2026-05-02T11:37:49Z
---

# T-1447: framework upgrade + full test suite + fix issues

## Context

User requested upgrade of vendored Agentic Engineering Framework on /opt/termlink to latest upstream (https://github.com/DimitriGeelen/agentic-engineering-framework), full test-suite run, classification of all issues, fixes for environmental problems, and findings for framework/upstream bugs (per STEP 5: do NOT edit framework source locally — report instead).

Pre-state: fw 1.5.307 (vendored), pinned 1.5.307.
Post-state: fw 1.6.124 (vendored from upstream).

## Acceptance Criteria

### Agent
- [x] `fw upgrade` ran cleanly from upstream clone (5 changes applied, 1.5.307 → 1.6.124)
- [x] `fw doctor` post-upgrade returns 0 FAIL after enforcement-baseline reset (post-upgrade routine)
- [x] `fw test all` ran end-to-end; per-suite pass/fail counts captured
- [x] Each test failure classified (framework / termlink / environmental)
- [x] CLAUDE.md project-specific governance content preserved or restored after `fw upgrade` clobber
- [x] Findings captured as learnings + bug report posted via termlink (channel post agent-chat-arc)
- [x] Commit + push to OneDev only (memory: never push to GitHub)

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

## RCA

**Symptom:** `fw test all` ERRORs on every consumer install (bats unit path missing); web/playwright tests assume framework-source data; `fw upgrade` silently clobbered project-specific governance content in CLAUDE.md.

**Root cause:** Framework's vendor includes list (`do_vendor()` in `bin/fw` line ~245) excludes `tests/`, but the test runner hard-codes `$FRAMEWORK_ROOT/tests/unit/`. Web/playwright fixtures (`PROJECT_ROOT` env handling in `tests/playwright/conftest.py`) read consumer paths but expect framework-source layout. Governance-section refresh in `fw upgrade` does not preserve project additions inside template-managed regions.

**Why structurally allowed:** No upstream CI runs `fw test all` in consumer-install mode — only on framework-source checkout where `tests/` exists at `$FRAMEWORK_ROOT/tests`. Consumer-install path divergence is invisible to upstream verification. CLAUDE.md regen has no diff-and-prompt step on protected sections.

**Prevention:** Reported as 6 framework findings (F-1..F-6) with fix options to upstream `framework-agent` via `termlink emit-to` seq 122 + chat-arc broadcast offsets 55-56. Local mitigation: copied tests/playwright/ from upstream clone, applied `PROJECT_ROOT=$FRAMEWORK_ROOT` workaround for the run, captured 6 learnings (F-1..F-6) in `.context/project/learnings.yaml`, restored CLAUDE.md from `.bak`. Upstream fix is the structural answer — local test-suite green is the future signal.

## Decisions

## Updates

### 2026-05-02T09:59:47Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1447-framework-upgrade--full-test-suite--fix-.md
- **Context:** Initial task creation

### 2026-05-02T11:37:49Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
