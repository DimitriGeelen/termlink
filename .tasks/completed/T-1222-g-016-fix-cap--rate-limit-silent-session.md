---
id: T-1222
name: "G-016 fix: cap + rate-limit silent-session scanner to prevent handover storm"
description: >
  Three mitigations per G-016: (1) add MAX_RECOVERIES per-invocation cap (default 10) to session-silent-scanner.sh inner loop, (2) add SESSION_SILENT_MAX_AGE_DAYS ceiling (default 7) to filter out ancient agent-acompact-* sessions, (3) optional: 30s rate limit between fw handover invocations. Also: trace root cause of DRY_RUN=0 in the 13:38 bootstrap that triggered the 92-commit storm — if shell snapshot carries it, every claude session re-triggers.

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: [G-016, T-1212, framework-health, safety]
components: []
related_tasks: []
created: 2026-04-24T15:44:57Z
last_update: 2026-04-24T15:49:09Z
date_finished: 2026-04-24T15:49:09Z
---

# T-1222: G-016 fix: cap + rate-limit silent-session scanner to prevent handover storm

## Context

G-016 trigger: 2026-04-24T13:38Z a `fw hook session-silent-scanner` invocation with `DRY_RUN=0` processed 757 stale `agent-acompact-*` sessions over 4 hours, generating 92 spurious handover commits on origin/main. Scanner killed manually at 17:42Z. Fix prevents recurrence.

Scanner source: `.agentic-framework/agents/context/session-silent-scanner.sh` (vendored) mirrored at `/opt/999-Agentic-Engineering-Framework/agents/context/session-silent-scanner.sh` (upstream). Both at same size (4574 bytes, Apr 24). Both need the fix.

The inner python loop iterates ALL matches with no cap, no age ceiling, no rate limit. When a bootstrap runs with `DRY_RUN=0` and finds a large backlog, it generates N handover commits serially with no backoff.

## Acceptance Criteria

### Agent
- [x] `SESSION_SILENT_MAX_RECOVERIES` env var added (default 10) — python inner loop breaks after this many successful `fw handover` invocations per run.
- [x] `SESSION_SILENT_MAX_AGE_DAYS` env var added (default 7) — matches with `age > MAX_AGE_DAYS*86400` skipped pre-queue AND logged as `skip-too-old`. Also emits `skip-too-old-total count=<n>` summary.
- [x] Cap log line `cap-reached N=<count> remaining=<n> max=<max>` emitted on break. DRY_RUN variant mirrors the same with `cap-would-hit`.
- [x] Verified: `SESSION_SILENT_MAX_RECOVERIES=2 SESSION_SILENT_MAX_AGE_DAYS=7 DRY_RUN=1` → log shows `skip-too-old-total count=829 max-age-days=7`, then 2 `DRY-RUN would-recover` lines, then `cap-would-hit candidates=17 max=2 remaining=15`. Behavior matches design.
- [x] Upstream mirror landed: `/opt/999-AEF/agents/context/session-silent-scanner.sh` at commit `2199ccba` on framework master, pushed to onedev. Direct bash run (no termlink dispatch — same machine).
- [x] Root-cause investigation of the 13:38 DRY_RUN=0 bootstrap — deferred to follow-up task T-1223. Deliverable for THIS task = the cap fix + the follow-up task capturing the open question.
- [x] `bash -n .agentic-framework/agents/context/session-silent-scanner.sh` clean. shellcheck not available locally; bash -n is the floor.

## Verification

bash -n .agentic-framework/agents/context/session-silent-scanner.sh
grep -q "SESSION_SILENT_MAX_RECOVERIES" .agentic-framework/agents/context/session-silent-scanner.sh
grep -q "SESSION_SILENT_MAX_AGE_DAYS" .agentic-framework/agents/context/session-silent-scanner.sh
grep -q "cap-reached" .agentic-framework/agents/context/session-silent-scanner.sh

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

### 2026-04-24T15:44:57Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1222-g-016-fix-cap--rate-limit-silent-session.md
- **Context:** Initial task creation

### 2026-04-24T15:46:13Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
- **Change:** horizon: next → now (auto-sync)

### 2026-04-24T15:49:09Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
