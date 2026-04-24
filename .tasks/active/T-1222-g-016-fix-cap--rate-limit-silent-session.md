---
id: T-1222
name: "G-016 fix: cap + rate-limit silent-session scanner to prevent handover storm"
description: >
  Three mitigations per G-016: (1) add MAX_RECOVERIES per-invocation cap (default 10) to session-silent-scanner.sh inner loop, (2) add SESSION_SILENT_MAX_AGE_DAYS ceiling (default 7) to filter out ancient agent-acompact-* sessions, (3) optional: 30s rate limit between fw handover invocations. Also: trace root cause of DRY_RUN=0 in the 13:38 bootstrap that triggered the 92-commit storm — if shell snapshot carries it, every claude session re-triggers.

status: captured
workflow_type: build
owner: agent
horizon: next
tags: [G-016, T-1212, framework-health, safety]
components: []
related_tasks: []
created: 2026-04-24T15:44:57Z
last_update: 2026-04-24T15:44:57Z
date_finished: null
---

# T-1222: G-016 fix: cap + rate-limit silent-session scanner to prevent handover storm

## Context

G-016 trigger: 2026-04-24T13:38Z a `fw hook session-silent-scanner` invocation with `DRY_RUN=0` processed 757 stale `agent-acompact-*` sessions over 4 hours, generating 92 spurious handover commits on origin/main. Scanner killed manually at 17:42Z. Fix prevents recurrence.

Scanner source: `.agentic-framework/agents/context/session-silent-scanner.sh` (vendored) mirrored at `/opt/999-Agentic-Engineering-Framework/agents/context/session-silent-scanner.sh` (upstream). Both at same size (4574 bytes, Apr 24). Both need the fix.

The inner python loop iterates ALL matches with no cap, no age ceiling, no rate limit. When a bootstrap runs with `DRY_RUN=0` and finds a large backlog, it generates N handover commits serially with no backoff.

## Acceptance Criteria

### Agent
- [ ] `SESSION_SILENT_MAX_RECOVERIES` env var added (default 10) — python inner loop breaks after this many successful `fw handover` invocations per run.
- [ ] `SESSION_SILENT_MAX_AGE_DAYS` env var added (default 7) — matches with `age-min > MAX_AGE_DAYS*1440` are skipped AND logged as `skip-too-old session=<id> age-min=<n>`. Ancient sessions carry zero context; a banner handover is worthless.
- [ ] When the cap is hit, log an explicit `cap-reached N=<count> remaining=<remaining>` line so operators can see the backlog and run again if needed.
- [ ] Verify the fix: set `SESSION_SILENT_MAX_RECOVERIES=2 DRY_RUN=1 fw hook session-silent-scanner` → log shows at most 2 `recovered` lines + (if backlog) 1 `cap-reached` line. DRY_RUN=1 so no git pollution.
- [ ] Upstream mirror to `/opt/999-AEF` via termlink_dispatch (PL-053 pattern, T-1063-approved). Verify via `git log` on framework master.
- [ ] Root-cause investigation: document in task file why the 13:38 bootstrap ran with `DRY_RUN=0`. Check `/root/.claude/shell-snapshots/*.sh` for DRY_RUN export, check `.claude/settings.local.json` for hook env. If found: open a separate task to fix the bootstrap config.
- [ ] `bash -n .agentic-framework/agents/context/session-silent-scanner.sh` passes + `shellcheck` (if available) clean.

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
