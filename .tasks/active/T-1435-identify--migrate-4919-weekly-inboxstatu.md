---
id: T-1435
name: "Identify + migrate 4919 weekly inbox.status pollers on .107 (T-1166 last-mile)"
description: >
  Identify + migrate 4919 weekly inbox.status pollers on .107 (T-1166 last-mile)

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-05-01T09:34:49Z
last_update: 2026-05-01T09:34:49Z
date_finished: null
---

# T-1435: Identify + migrate 4919 weekly inbox.status pollers on .107 (T-1166 last-mile)

## Context

T-1432 fleet doctor --legacy-usage on 2026-05-01 surfaced 5121 legacy invocations on .107 in a 7-day window — 4919 (96%) are `inbox.status` calls, all from `from=(unknown)` (no caller-attribution label). Some background loop is polling. Likely Watchtower's panel render or a cron/api-usage script. Until this is migrated, the T-1166 cut verdict stays at WAIT instead of CUT-READY. This is the "last mile" for the cut.

The remaining traffic on .107: 197 event.broadcast (mix of unknown + tl-* sessions, several tl-t1407-test which suggests test fixtures), 5 inbox.list (all unknown).

## Acceptance Criteria

### Agent
- [ ] Identify the 4919 inbox.status callers — search for `inbox.status` invocations in: Watchtower (`.context/watchtower/` + any python source), agentic-framework scripts (`.agentic-framework/agents/**/*.sh`), cron entries, any process polling once a minute
- [ ] Identify the 197 event.broadcast callers — distinguish test-fixtures (tl-t1407-test) from real callers; real callers should migrate to channel.post or event.emit_to
- [ ] Migrate each identified caller to its T-1166 canonical replacement: `inbox.status`→`channel.info` or simply remove the poll (if it was just rendering an empty inbox indicator); `inbox.list`→`channel.subscribe`; `event.broadcast`→`channel.post` (broadcast) or `event.emit_to` (unicast)
- [ ] After migration, re-run `target/release/termlink fleet doctor --legacy-usage --legacy-window-days 1` (1d window so it shows post-migration traffic only). Verdict should be on track to CUT-READY: total_legacy on .107 should drop to near-zero within 24h
- [ ] Add caller-attribution (`from=<label>`) to any remaining legitimate callers so the breakdown isn't mostly `(unknown)` — discoverability for the next iteration
- [ ] No fix-with-suppression: setting `TERMLINK_NO_DEPRECATION_WARN=1` to silence the symptom does NOT count as migrating the caller. The audit-log count is what the verdict reads

### Human
- [ ] [REVIEW] Verify the 24h post-migration window shows zero legacy traffic on .107
  **Steps:**
  1. Wait 24h after the last migration commit
  2. `target/release/termlink fleet doctor --legacy-usage --legacy-window-days 1`
  3. Look for `CLEAN (1d): workstation-107-public, local-test, laptop-141`
  **Expected:** verdict reads CUT-READY for 1-day window. Once it holds for 7d, T-1166 cut is safe
  **If not:** new callers have appeared since migration — re-run the breakdown via `--json | jq .legacy_summary` and identify the new offenders

## Verification

target/release/termlink fleet doctor --legacy-usage --legacy-window-days 1 2>&1 | grep -E "CLEAN|WITH TRAFFIC" | head -5

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

### 2026-05-01T09:34:49Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1435-identify--migrate-4919-weekly-inboxstatu.md
- **Context:** Initial task creation
