---
id: T-1208
name: "SessionEnd hook design — auto-handover with bug-fallbacks (T-174 parent)"
description: >
  Inception: SessionEnd hook design — auto-handover with bug-fallbacks (T-174 parent)

status: work-completed
workflow_type: inception
owner: human
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-04-24T09:14:25Z
last_update: 2026-04-24T09:50:23Z
date_finished: 2026-04-24T09:50:23Z
---

# T-1208: SessionEnd hook design — auto-handover with bug-fallbacks (T-174 parent)

## Problem Statement

Claude Code's `SessionEnd` hook fires on session termination with a `reason` field. The goal: auto-trigger `fw handover` on every exit so no session ends without a handover document. Today the handover depends on (a) agent discipline, (b) budget-gate auto-handover at critical, or (c) the PreCompact hook — all partial. Sessions that end by `/exit`, terminal close, or API 500 still slip through. Known Claude Code bugs **#17885** (SessionEnd doesn't fire on `/exit`) and **#20197** (API 500 skips it) mean the hook alone cannot be the sole trigger — a fallback is required.

Full research: `docs/reports/T-1208-sessionend-hook-inception.md`.

## Assumptions

- A1: SessionEnd fires reliably on `clear` and `logout` reasons; only `prompt_input_exit` (#17885) is unreliable on the current Claude Code version.
- A2: Idempotency check (compare `LATEST.md` `session_id` frontmatter against `.context/working/session.yaml`) is sufficient to avoid duplicate handovers.
- A3: A 15-min silent-session cron (scanning `.claude/sessions/*.jsonl` for sessions idle >30 min) is a viable antifragility fallback for hook-missed exits.

## Exploration Plan

- **S1 (1h):** Passive reason-field logger — wire a no-op SessionEnd handler that logs `reason` across 3 real session endings. Confirms which reasons actually fire on current Claude Code.
- **S2 (2h):** Handover-trigger prototype — idempotent invoke of `fw handover` with wall-clock measurement under clean-exit and simulated-kill scenarios.
- **S3 (2h):** Silent-session cron fallback — scan every 15 min for sessions whose last event is >30 min old AND whose `session_id` doesn't appear in any handover; generate a recovery handover marked `[recovered, no agent context]`.

## Technical Constraints

- Hook runs in "shutting down" state — can write files, cannot prompt user. Must be fast (<10s) or risk being killed.
- Framework-side script; consumer projects wire via `.agentic-framework/bin/fw hook session-end` in `.claude/settings.json`.
- D1-D4 alignment: S3 is the antifragility piece (D1); S2 measures reliability (D2); recovery handovers are clearly labeled (D3); scripts never reach into Claude Code internals beyond documented payloads (D4).

## Scope Fence

**IN:** hook handler for `clear`/`logout`/`prompt_input_exit` reasons, idempotency guard, silent-session cron fallback, recovery-handover labeling.

**OUT:** replacing the PreCompact handover path; forensic transcript analysis (just attach transcript path in recovery); cross-session analytics.

## Acceptance Criteria

### Agent
- [x] Problem statement validated
- [x] Assumptions tested
- [x] Recommendation written with rationale

### Human
- [x] [REVIEW] Review exploration findings and approve go/no-go decision
  **Steps:**
  1. Run: `fw task review T-XXX` (opens Watchtower with recommendation, assumptions, research artifacts)
  2. Review the Agent Recommendation section and go/no-go criteria evaluation
  3. Record decision via the Watchtower form or the command shown alongside the QR code
  **Expected:** Decision recorded, task completed
  **If not:** Ask agent for clarification on specific findings

## Go/No-Go Criteria

**GO if:**
- S2 generates a handover on clean exits with ≥95% success rate and wall-clock under 10s.
- S3 recovers any session skipped by SessionEnd within 30 min of silence.
- Idempotency guard prevents duplicate handovers when both hook and cron fire.

**NO-GO if:**
- SessionEnd fires on <50% of real exits — in that case retire the hook work and ship only the silent-session cron (S3) as its own build task.
- Recovery handover quality is so low (no actionable content) that humans ignore them.

## Verification

# Shell commands that MUST pass before work-completed. One per line.
# Lines starting with # are comments (skipped). Empty lines ignored.
# For inception tasks, verification is often not needed (decisions, not code).

## Recommendation

**Recommendation:** GO (S1+S2+S3 all in scope)

**Rationale:** The hook alone is structurally insufficient due to known Claude Code bugs (#17885, #20197). A silent-session cron (S3) is not optional — it's the antifragility piece that makes the whole system reliable. Shipping S2 without S3 would create false confidence (handovers only for the exit modes Claude Code supports), exactly the blind spot that left previous sessions without handovers. The plan is bounded: one hook script, one idempotency check, one cron script — all with concrete exit criteria and framework-directive alignment (D1-D4).

**Evidence:**
- T-174 task description explicitly flags #17885 and #20197 — hook unreliability is a known constraint, not an assumption.
- Framework already has `fw handover` as a mature one-shot command (invoked by PreCompact hook, budget-gate auto-handover, and manually) — S2 just wires an existing flow to a new trigger.
- `.claude/sessions/*.jsonl` exist on disk per session with timestamps — S3's silent-session detection has clean signal, no new instrumentation needed.
- `.context/working/session.yaml` carries `session_id` — idempotency guard has zero new state to invent.

**Human direction (2026-04-24):** "Proceed as you see fit, considering framework directives." Accepted with D1-D4 alignment documented; S3 kept in scope as the antifragility piece.

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

**Decision**: GO

**Rationale**: The hook alone is structurally insufficient due to known Claude Code bugs (#17885, #20197). A silent-session cron (S3) is not optional — it's the antifragility piece that makes the whole system reliable. Shipping S2 without S3 would create false confidence (handovers only for the exit modes Claude Code supports), exactly the blind spot that left previous sessions without handovers. The plan is bounded: one hook script, one idempotency check, one cron script — all with concrete exit criteria and framework-directive alignment (D1-D4).

**Date**: 2026-04-24T09:50:23Z

## Updates

<!-- Auto-populated by git mining at task completion.
     Manual entries optional during execution. -->

### 2026-04-24T09:16:14Z — status-update [task-update-agent]
- **Change:** status: captured → started-work

### 2026-04-24T09:50:23Z — inception-decision [inception-workflow]
- **Action:** Recorded inception decision
- **Decision:** GO
- **Rationale:** The hook alone is structurally insufficient due to known Claude Code bugs (#17885, #20197). A silent-session cron (S3) is not optional — it's the antifragility piece that makes the whole system reliable. Shipping S2 without S3 would create false confidence (handovers only for the exit modes Claude Code supports), exactly the blind spot that left previous sessions without handovers. The plan is bounded: one hook script, one idempotency check, one cron script — all with concrete exit criteria and framework-directive alignment (D1-D4).

### 2026-04-24T09:50:23Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
- **Reason:** Inception decision: GO
