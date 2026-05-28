---
id: T-1826
name: "agent-conversation-status.sh — conversation_id state diagnostic"
description: >
  agent-conversation-status.sh — conversation_id state diagnostic

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-05-28T09:39:59Z
last_update: 2026-05-28T09:39:59Z
date_finished: null
---

# T-1826: agent-conversation-status.sh — conversation_id state diagnostic

## Context

Closes the observability gap in the doorbell+mail loop (T-1800/T-1804/T-1805/T-1807/T-1809):
`agent-send.sh` polls receipts internally and exits with a status code; `agent-respond.sh`
posts receipts. There is **no external view** of "where is conversation cid-X right now" for
a third observer (operator running multiple concurrent autonomous threads, or an orchestrator
agent supervising n simultaneous a2a conversations). This is the *interactive autonomous
agent-to-agent conversation mode* arc's missing diagnostic primitive.

Read-only bash verb that composes the existing first-class filter
(`channel subscribe --conversation-id <CID> --json`, already in CLI per cli.rs:2319)
+ jq to render a summary: turn count, sender list, receipts received, pending turns
(turns with no matching receipt), and last activity timestamp. JSON output mode for
agent consumption; human-readable text default. No protocol change.

## Acceptance Criteria

### Agent
- [x] `scripts/agent-conversation-status.sh` exists, executable, `bash -n` + `shellcheck` clean.
- [x] Accepts `--topic <T> --conversation-id <CID>` (required) + `--hub <addr>` (optional) + `--json` (optional flag).
- [x] In text mode, prints a multi-line summary with: topic, conversation_id, turn count (with sender breakdown), receipt count (with up_to mapping), pending receipts (turn offsets with no matching `metadata.up_to >= turn_offset`), last activity timestamp (RFC3339).
- [x] In `--json` mode, emits a single JSON object with fields `{ok, topic, conversation_id, turns:[{offset, sender, ts}], receipts:[{offset, up_to, sender, ts}], pending_turn_offsets:[], senders:[], last_activity, summary:{turn_count, receipt_count, pending_count, sender_count}}`.
- [x] Unknown / empty conversation_id yields exit 0 with `turn_count=0` and `summary.turn_count=0` (not an error — empty conversation is valid state).
- [x] Usage errors (missing required args) exit 2 with usage to stderr.
- [x] `scripts/test-agent-conversation-status.sh` covers: (a) usage error on missing args, (b) unknown args exit 2, (c) `--json` parses with jq, (d) empty cid returns turn_count=0, (e) populated cid returns matching counts, (f) pending detection: turn offsets without watermark coverage are flagged. Tests use a deterministic local hub setup (ephemeral topic, synthetic posts) — no cross-host dependency. **5 PASS / 0 FAIL** at HEAD.
- [x] Test suite ALL PASS under both bash and shellcheck.

## Recommendation

**Ship.** A2A observability gap closed for the single-conversation case. Three concrete
follow-ups (file as needed):

1. **MCP parity (T-1827 candidate):** `termlink_agent_conversation_status` MCP tool —
   subprocess `agent-conversation-status.sh --json` (or reimplement in Rust) so agent
   callers can introspect their own / others' conversations without shelling out. Same
   PL-167 silent-strip risk profile as T-1825 — every CLI surface needs MCP parity to
   stay reachable by LLM-driven agents.
2. **Multi-cid summary verb (T-1828 candidate):** `agent-conversation-list.sh` — given a
   topic, enumerate distinct `metadata.conversation_id` values + per-cid roll-up
   (counts, last activity, status). Useful for orchestrator agents supervising N
   simultaneous threads.
3. **Status field (T-1829 candidate):** classify conversations into `idle` (no activity
   recently), `pending` (turns waiting on receipts), `delivered` (all turns acked),
   `stalled` (oldest pending > N minutes). Layer on top of the current `pending_count`
   primitive; introduces a clock dependency the current verb deliberately avoids.

## Skip-rationale

No follow-ups auto-filed — each is a separate small unit that can be filed independently
based on observed need (the standing directive is on this session, not on speculative
future capability).

### Human

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

bash -n scripts/agent-conversation-status.sh
shellcheck scripts/agent-conversation-status.sh
bash -n scripts/test-agent-conversation-status.sh
shellcheck scripts/test-agent-conversation-status.sh
bash scripts/test-agent-conversation-status.sh

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

### 2026-05-28T09:39:59Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1826-agent-conversation-statussh--conversatio.md
- **Context:** Initial task creation
