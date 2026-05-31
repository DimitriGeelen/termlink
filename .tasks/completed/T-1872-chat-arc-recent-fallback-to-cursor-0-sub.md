---
id: T-1872
name: "chat-arc-recent: fallback to cursor-0 subscribe when channel info times out (PL-194 mitigation)"
description: >
  chat-arc-recent: fallback to cursor-0 subscribe when channel info times out (PL-194 mitigation)

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: [scripts/agent-chat-arc-recent.sh]
related_tasks: [T-1870, T-1871, T-1851]
created: 2026-05-30T06:41:36Z
last_update: 2026-05-30T06:49:36Z
date_finished: 2026-05-30T06:49:36Z
---

# T-1872: chat-arc-recent: fallback to cursor-0 subscribe when channel info times out (PL-194 mitigation)

## Context

PL-194 (just registered): `channel info` on agent-chat-arc has asymmetric
read/write cost on slow hubs — write completes within 8s ceiling but
read (scanning backlog to compute count) times out. T-1870 made the
failure visible; this task closes the actual gap so /pulse and
/recent-chat return data from slow hubs instead of marking them failed.

Investigation (just now on .107):

- `channel info` on .121: 10s timeout (rc=124)
- `channel info` on .141: 10s timeout (rc=124)
- `channel subscribe --hub <same> --since SINCE_MS --limit 500`: 95-118ms
- BUT — confirmed via `channel subscribe --help`: `--since`/`--tail` are
  pure **render-side** filters. Cursor still starts at 0. With `--limit
  500` on a 2000-post topic, subscribe returns 500 from offset 0, then
  --since drops them all → empty result. So we can't just drop the info
  probe; the seek-to-tail (PL-188) is structurally necessary for big
  topics.

What works for SMALL/MEDIUM topics (e.g. .121's 277 posts, .141's 429
posts): if we skip the seek-to-tail and run subscribe with `--cursor 0
--limit SCAN_LIMIT --since SINCE_MS`, the server returns the whole
topic within --limit, and --since filters to the window. For these
hubs that fits comfortably in <500 posts.

Strategy: when `channel info` exits non-zero (timeout OR network) AND
the previous behavior would mark the hub failed, fall through to a
**no-seek subscribe** with cursor=0. Returns degraded but useful data
on slow hubs. Hub still counts as scanned (not failed). Surface the
fallback distinctly in the failed_hubs array (reason: `info-timeout-fallback`
when the fallback succeeded; `network`/`timeout` only when the fallback
also failed).

Risk: for hubs with > SCAN_LIMIT posts whose recent activity is past
SCAN_LIMIT from offset 0, the fallback returns empty. Acceptable —
better than the current "hub marked failed, zero posts surfaced".
Surface this distinctly via the reason field.

## Acceptance Criteria

### Agent
- [x] `scripts/agent-chat-arc-recent.sh` adds a fallback path: when `channel info` exits non-zero AND the error is NOT `-32013|unknown topic|Not found`, attempt `channel subscribe --cursor 0 --limit SCAN_LIMIT --since SINCE_MS` instead of marking the hub failed
- [x] If the fallback subscribe succeeds (rc=0), the hub is counted as `hubs_scanned`, posts are merged into the result, and the hub is NOT in `failed_hubs` (it succeeded, just via fallback)
- [x] If the fallback subscribe ALSO fails (rc != 0), the hub is marked failed as before — reason is `timeout` (rc=124 on subscribe) or `network` (anything else)
- [x] An optional `fallback: true` field is added to scanned hubs that used the fallback path — surfaced in `--json` envelope's `summary.fallback_hubs: [<name>]` array so /pulse can hint "data may be partial — seek-to-tail unavailable"
- [x] Human format adds one line `  fallback: <name1>, <name2>` when any hub used the fallback path (omitted when zero)
- [x] Verification: on .107 fleet today, running with default scan-limit produces non-empty results for at least one of .121 or .141 (currently both fail silently)
- [x] Existing semantics preserved: hubs that succeed on the first `channel info` path still take the seek-to-tail (PL-188) optimization — no behavior change for fast hubs

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

bash scripts/agent-chat-arc-recent.sh --json --limit 1 --since 1 | jq -e '.summary.fallback_hubs | type == "array"'
bash scripts/agent-chat-arc-recent.sh --json --limit 1 --since 1 | jq -e '.summary | has("failed_hubs") and has("fallback_hubs")'
bash scripts/agent-chat-arc-recent.sh --help >/dev/null

## RCA

<!-- REQUIRED for bug-class tasks (workflow_type=build with bug-tag, OR title matches
     fix/bug/rca/broken/crash/error/regression/fail/hotfix).
     Non-bug-class tasks may leave this section empty or remove it.

     For bug-class, fill in:
       **Symptom:** what was observed (the user-facing manifestation).
       **Root cause:** the specific structural/logical gap — not "the code was wrong".
       **Why structurally allowed:** what in the framework/code/tooling let this go undetected.
       **Prevention:** what catches the next instance (test/lint/gate/doc/learning) — distinct from the fix itself.

     The completion gate (T-1550, G-019) blocks --status work-completed when
     bug-class AND this section is empty/template-only. Use --skip-rca to bypass (logged).
-->

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

### 2026-05-30T06:41:36Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1872-chat-arc-recent-fallback-to-cursor-0-sub.md
- **Context:** Initial task creation

## Reviewer Verdict (v1.4)

- **Scan ID:** R-f1e7a655
- **Timestamp:** 2026-05-30T06:50:34Z
- **Catalogue:** v1.3-seed
- **Overall:** CONCERN
- **Needs Human:** no
- **Findings:** 1

**Verification-level findings:**

  1. **empty-output-success** (partial, heuristic) @ Verification:line 3
     - evidence: `bash scripts/agent-chat-arc-recent.sh --help >/dev/null`

### 2026-05-30T06:49:36Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
