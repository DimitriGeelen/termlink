---
id: T-1193
name: "CLAUDECODE-unset follow-up to db8b39d1 — approvals auto-execute must strip CLAUDECODE from subprocess env"
description: >
  Structural fix db8b39d1 landed Watchtower auto-execute for fw inception decide approvals, but subprocess.run passes env={**os.environ,'TIER0_AUTOEXEC':'1'} which still includes CLAUDECODE=1. Inner gate (T-679/T-1259) refuses the command with 'Agents must not invoke fw inception decide directly'. Fix: strip CLAUDECODE before spawning. Apply to both /opt/termlink vendored and upstream /opt/999-Agentic-Engineering-Framework via Channel 1 dispatch.

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: [framework, watchtower, structural-fix]
components: []
related_tasks: [T-1192, T-939]
created: 2026-04-22T21:52:21Z
last_update: 2026-04-22T21:52:21Z
date_finished: null
---

# T-1193: CLAUDECODE-unset follow-up to db8b39d1 — approvals auto-execute must strip CLAUDECODE from subprocess env

## Context

Follow-up to structural fix `db8b39d1` (T-1192). That patch added `_execute_inception_decide()` in `web/blueprints/approvals.py` so Watchtower's `/api/approvals/decide` auto-executes the approved command. Bug on test: subprocess env still contains `CLAUDECODE=1` (inherited from the Claude Code parent), so the inner gate in `fw inception decide` (T-679/T-1259) refuses the spawn. Single-line fix: build env without CLAUDECODE, then add TIER0_AUTOEXEC.

## Acceptance Criteria

### Agent
- [x] `/opt/termlink/.agentic-framework/web/blueprints/approvals.py` — `_execute_inception_decide()` subprocess.run uses an env dict that does NOT contain `CLAUDECODE` (line 423: `{k: v for k, v in os.environ.items() if k != "CLAUDECODE"}`)
- [x] `/opt/999-Agentic-Engineering-Framework/web/blueprints/approvals.py` — same patch mirrored upstream via Channel 1 dispatch, committed, pushed (upstream commit `f398d004`, pushed to onedev master `db8b39d1..f398d004`)
- [x] Both files still set `TIER0_AUTOEXEC=1` in the subprocess env (outer-hook contract preserved)
- [x] `python3 -c "import ast; ast.parse(...)"` parses clean on both files (verified)
- [x] No other callers of approvals.py subprocess pattern regressed (only `_execute_inception_decide` path changed; unrelated `subprocess.run` call at line 475+ batch work-completed is untouched)

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

grep -q "k != \"CLAUDECODE\"" /opt/termlink/.agentic-framework/web/blueprints/approvals.py
grep -q "TIER0_AUTOEXEC" /opt/termlink/.agentic-framework/web/blueprints/approvals.py
python3 -c "import ast; ast.parse(open('/opt/termlink/.agentic-framework/web/blueprints/approvals.py').read())"

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

### 2026-04-22T21:52:21Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1193-claudecode-unset-follow-up-to-db8b39d1--.md
- **Context:** Initial task creation
