---
id: T-1881
name: "recent-dm: surface conversation_id in render output"
description: >
  recent-dm: surface conversation_id in render output

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-05-30T16:10:50Z
last_update: 2026-05-30T16:15:24Z
date_finished: 2026-05-30T16:15:24Z
---

# T-1881: recent-dm: surface conversation_id in render output

## Context

Post-T-1880 friction: `/recent-dm` renders TS/TOPIC/SENDER/PREVIEW but omits
`metadata.conversation_id` — the load-bearing thread key that `/reply` uses
to target. Without it visible the operator cannot (a) distinguish multiple
concurrent threads on the same DM topic before firing `/reply`, (b) decide
between `/reply` (uses latest cid) vs `/reply --ensure-cid` (mints new),
(c) shell out to `agent-respond.sh` directly with an explicit cid. This
task surfaces `conversation_id` through the read pipeline so the cid is
visible alongside the post.

## Acceptance Criteria

### Agent
- [x] `scripts/agent-chat-arc-recent.sh` per-post JSON object includes a `conversation_id` field (null when envelope has no metadata.conversation_id)
- [x] `scripts/recent-dm.sh` text mode renders a `CID` column between SENDER and PREVIEW (first 22 chars or `-` when null)
- [x] `scripts/recent-dm.sh --json` output preserves `conversation_id` on each post (additive field, no shape change to other fields)
- [x] Live smoke against `dm:d1993c2c3ec44c94:d1993c2c3ec44c94` shows the cid column populated for envelopes that carry one (e.g. T-1880 smoke entry with cid `reply-20260530T152332Z`)
- [x] Backward compat: chat-arc posts (no `metadata.conversation_id`) flow through with `conversation_id: null` in JSON without breaking `/recent-chat` text render

<!-- All criteria are mechanically verifiable — no Human section. -->

## Verification

# Reader emits conversation_id field on each post (key may be absent when null — accept either).
bash scripts/agent-chat-arc-recent.sh --topic dm:d1993c2c3ec44c94:d1993c2c3ec44c94 --limit 1 --since 24 --json --all-msg-types | jq -e '.posts[0] | has("conversation_id")' >/dev/null
# recent-dm --json carries conversation_id through to per-post objects.
bash scripts/recent-dm.sh d1993c2c3ec44c94:d1993c2c3ec44c94 --limit 1 --since 24 --json | jq -e '.posts[0] | has("conversation_id")' >/dev/null
# recent-dm text mode renders a CID column header.
bash scripts/recent-dm.sh d1993c2c3ec44c94:d1993c2c3ec44c94 --limit 1 --since 24 | grep -q "CID"
# Backward compat: agent-chat-arc-recent on chat-arc topic still renders without erroring (no jq null surprises).
bash scripts/agent-chat-arc-recent.sh --limit 1 --since 24 --json >/dev/null
# Syntax sanity: both scripts parse with bash -n.
bash -n scripts/agent-chat-arc-recent.sh
bash -n scripts/recent-dm.sh

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

### 2026-05-30T16:10:50Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1881-recent-dm-surface-conversationid-in-rend.md
- **Context:** Initial task creation

## Reviewer Verdict (v1.4)

- **Scan ID:** R-5638cef6
- **Timestamp:** 2026-05-30T16:16:18Z
- **Catalogue:** v1.3-seed
- **Overall:** CONCERN
- **Needs Human:** no
- **Findings:** 3

**Verification-level findings:**

  1. **empty-output-success** (partial, heuristic) @ Verification:line 2
     - evidence: `bash scripts/agent-chat-arc-recent.sh --topic dm:d1993c2c3ec44c94:d1993c2c3ec44c94 --limit 1 --since 24 --json --all-msg-types | jq -e '.posts[0] | has("conversation_id")' >/dev/null`
  2. **empty-output-success** (partial, heuristic) @ Verification:line 4
     - evidence: `bash scripts/recent-dm.sh d1993c2c3ec44c94:d1993c2c3ec44c94 --limit 1 --since 24 --json | jq -e '.posts[0] | has("conversation_id")' >/dev/null`
  3. **empty-output-success** (partial, heuristic) @ Verification:line 8
     - evidence: `bash scripts/agent-chat-arc-recent.sh --limit 1 --since 24 --json >/dev/null`

### 2026-05-30T16:15:24Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
