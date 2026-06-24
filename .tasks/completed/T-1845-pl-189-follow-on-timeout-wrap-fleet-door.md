---
id: T-1845
name: "PL-189 follow-on: timeout-wrap fleet-doorbell-mail-canary + selftest"
description: >
  Apply timeout 8/30 bounding to scripts/check-fleet-doorbell-mail-health.sh + scripts/agent-conversation-selftest.sh. PL-189 captured the root cause (termlink channel info/subscribe has NO client-side timeout); T-1843 fixed fleet-adoption-snapshot. This closes the symmetric gap in the canary path — a single hanging hub must not wedge the whole sweep.

status: work-completed
workflow_type: build
owner: agent
horizon: null
tags: [bug, doorbell-mail, canary, pl-189]
components: []
related_tasks: []
created: 2026-05-28T18:32:23Z
last_update: 2026-05-28T18:35:47Z
date_finished: 2026-05-28T18:35:47Z
---

# T-1845: PL-189 follow-on: timeout-wrap fleet-doorbell-mail-canary + selftest

## Context

PL-189 (captured during T-1843) documents that `termlink channel info|subscribe` has no client-side timeout — a TCP read against an unreachable hub hangs indefinitely. T-1843 wrapped every termlink RPC in `fleet-adoption-snapshot.sh` with `timeout 8`; this task applies the symmetric fix to the doorbell+mail canary path (T-1831). The canary calls `agent-conversation-selftest.sh` per hub; the selftest in turn calls `termlink channel create/post`. Either layer can wedge.

## Acceptance Criteria

### Agent
- [x] `scripts/agent-conversation-selftest.sh`: every `"$TERMLINK"` call wrapped with `$TIMEOUT_CMD` (default `timeout 8`, env-overridable via `TERMLINK_SELFTEST_TIMEOUT`). Falls back gracefully when `timeout(1)` is absent.
- [x] `scripts/check-fleet-doorbell-mail-health.sh`: the per-profile `bash "$SELFTEST"` call wrapped with `$TIMEOUT_CMD` (default `timeout 30` per-hub, env-overridable via `FLEET_DM_CANARY_TIMEOUT`). Treat exit code 124 as "unreachable", not "tooling error".
- [x] Live run against the current fleet (`bash scripts/check-fleet-doorbell-mail-health.sh --json`) emits valid JSON and completes within bounded time even if any hub silently hangs.

      Live (2026-05-28): 5/5 pass in 1.87s wall-clock; per-hub elapsed_ms 214–494.
- [x] Tests: existing `scripts/test-fleet-adoption-snapshot.sh` still 9/9 pass. No regression to selftest happy path against local hub.

      Verified: 9/9 pass (T1–T9 + T10 + T11 from T-1844 regression). Live canary run also confirms selftest still passes against local-test.
- [x] Commit message references `PL-189`.

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

grep -qE 'TIMEOUT_CMD|timeout ' scripts/agent-conversation-selftest.sh
grep -qE 'TIMEOUT_CMD|timeout ' scripts/check-fleet-doorbell-mail-health.sh
bash scripts/check-fleet-doorbell-mail-health.sh --json | jq -e '.summary.total >= 1 and (.profiles | length) == .summary.total' >/dev/null
bash scripts/test-fleet-adoption-snapshot.sh >/dev/null

## RCA

**Symptom:** `scripts/check-fleet-doorbell-mail-health.sh` (T-1831 cron canary) can wedge indefinitely if any hub in `hubs.toml` is silently unresponsive at the TCP layer. Same failure shape that T-1843 hit during the first run of `fleet-adoption-snapshot.sh` (50+ zombie processes from a week of prior hangs).

**Root cause:** The `termlink` binary's `channel info` / `channel subscribe` / `channel post` commands set no client-side TCP read timeout. The selftest (`scripts/agent-conversation-selftest.sh`) called by the canary uses `channel create` + `channel post` per hub; either can hang. The canary inherits.

**Why structurally allowed:** T-1843's fix landed in fleet-adoption-snapshot only; PL-189 explicitly tagged the canary path as the symmetric "next instance" but no test exercised it against a frozen hub.

**Prevention:** PL-189 stays as the broader learning; this task applies the documented mitigation (wrap every termlink RPC with `timeout 8` in selftest, wrap the per-hub selftest call with `timeout 30` in canary). Future fleet sweepers should grep `timeout` against the agent-listeners pattern. A binary-side fix (real client timeout in `termlink` channel RPCs) is the long-term Level-D and remains an open follow-on under PL-189.

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

### 2026-05-28T18:32:23Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1845-pl-189-follow-on-timeout-wrap-fleet-door.md
- **Context:** Initial task creation

## Reviewer Verdict (v1.4)

- **Scan ID:** R-8a4f95aa
- **Timestamp:** 2026-05-28T18:36:06Z
- **Catalogue:** v1.3-seed
- **Overall:** CONCERN
- **Needs Human:** no
- **Findings:** 2

**Verification-level findings:**

  1. **empty-output-success** (partial, heuristic) @ Verification:line 3
     - evidence: `bash scripts/check-fleet-doorbell-mail-health.sh --json | jq -e '.summary.total >= 1 and (.profiles | length) == .summary.total' >/dev/null`
  2. **empty-output-success** (partial, heuristic) @ Verification:line 4
     - evidence: `bash scripts/test-fleet-adoption-snapshot.sh >/dev/null`

### 2026-05-28T18:35:47Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
