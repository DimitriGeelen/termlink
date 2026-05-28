---
id: T-1827
name: "agent-conversation-list.sh — enumerate cids on a topic with per-cid roll-up"
description: >
  agent-conversation-list.sh — enumerate cids on a topic with per-cid roll-up

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: [crates/termlink-cli/src/commands/remote.rs]
related_tasks: []
created: 2026-05-28T09:46:16Z
last_update: 2026-05-28T09:49:59Z
date_finished: 2026-05-28T09:49:59Z
---

# T-1827: agent-conversation-list.sh — enumerate cids on a topic with per-cid roll-up

## Context

Natural sibling to T-1826 (`agent-conversation-status.sh`). That verb answers
"how is conversation cid-X doing?" — but to ask that, you have to know cid-X
exists. Without a list view, an orchestrator agent supervising N autonomous
threads can't enumerate its own active set, and operators can't sweep a
topic for stalled/abandoned conversations.

`agent-conversation-list.sh`: given a topic (and optional `--hub`), scan
recent envelopes, group by `metadata.conversation_id`, and emit a per-cid
roll-up — turn count, receipt count, distinct senders, last activity ts.
Read-only; composes `channel subscribe --json` + jq. No protocol change.

Envelopes lacking `metadata.conversation_id` (legacy chat-arc, etc.) are
either bucketed under a single `(no-cid)` sentinel row or skipped entirely.
Configurable via `--include-no-cid` flag (default OFF; focus on
doorbell+mail loop threads).

## Acceptance Criteria

### Agent
- [x] `scripts/agent-conversation-list.sh` exists, executable, `bash -n` + `shellcheck` clean.
- [x] Accepts `--topic <T>` (required) + `--hub <addr>` (optional) + `--limit <N>` (optional, default 500) + `--include-no-cid` (optional flag, default OFF) + `--json` (optional flag) + `--sort <field>` (optional, one of: `last_activity` (default) / `turn_count` / `cid`).
- [x] In text mode, prints a table with one row per distinct `conversation_id`: `cid | turns | receipts | senders | last_activity`. Header row + one row per cid.
- [x] In `--json` mode, emits a single JSON object: `{ok, topic, conversation_count, conversations: [{conversation_id, turn_count, receipt_count, sender_count, senders: [...], last_activity, first_activity}], summary: {total_envelopes_scanned, with_cid, without_cid}}`. Sorted per `--sort`.
- [x] `--include-no-cid` enables a sentinel entry with `conversation_id="(no-cid)"`; default OFF.
- [x] Usage errors (missing `--topic`, unknown args, invalid `--sort`) exit 2 with usage to stderr.
- [x] Empty topic / no matching envelopes yields exit 0 with `conversation_count=0`.
- [x] `scripts/test-agent-conversation-list.sh` covers all 7 paths (T1..T7). **7 PASS / 0 FAIL / 0 SKIP** at HEAD.
- [x] Test suite ALL PASS; `bash -n` + `shellcheck` clean on both files.

## Recommendation

**Ship.** T-1826 (status, single-cid) + T-1827 (list, all-cids) form the read-side
observability pair for the doorbell+mail loop. Together they let an orchestrator
agent enumerate its active threads and inspect any one in detail without shelling
out to channel subscribe + grep.

**Side observation:** `termlink channel subscribe` exits **1** on unknown topic
(JSON-RPC -32013), not 0 with empty output. Both T-1826 and T-1827 correctly detect
this and exit 3 ("subscribe failed"). Documented here for future-me; not a bug.

Follow-up candidates (file as needed):
- **MCP parity (T-1828 candidate):** `termlink_agent_conversation_list` MCP tool.
  Same PL-167 silent-strip risk — bash verb is invisible to agent callers without
  MCP wrapper.
- **Watch mode (T-1829 candidate):** `--watch <secs>` follow loop. Cron-replacement
  for "alert me when a new cid appears on this topic".
- **Health classifier (T-1830 candidate):** classify each cid as
  `active/idle/stalled/delivered` based on pending count + age.

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

bash -n scripts/agent-conversation-list.sh
shellcheck scripts/agent-conversation-list.sh
bash -n scripts/test-agent-conversation-list.sh
shellcheck scripts/test-agent-conversation-list.sh
bash scripts/test-agent-conversation-list.sh

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

### 2026-05-28T09:46:16Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1827-agent-conversation-listsh--enumerate-cid.md
- **Context:** Initial task creation

## Reviewer Verdict (v1.4)

- **Scan ID:** R-f207edd9
- **Timestamp:** 2026-05-28T09:49:59Z
- **Catalogue:** v1.3-seed
- **Overall:** PASS
- **Needs Human:** no
- **Findings:** none

### 2026-05-28T09:49:59Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
