---
id: T-1790
name: "termlink_channel_info MCP — topic-flexible synthesized topic view (T-1166 wedge)"
description: >
  termlink_channel_info MCP — topic-flexible synthesized topic view (T-1166 wedge)

status: work-completed
workflow_type: build
owner: agent
horizon: null
tags: [mcp, T-1166, channel]
components: [crates/termlink-mcp/src/tools.rs]
related_tasks: [T-1166, T-1324, T-1323, T-1331]
created: 2026-05-21T18:23:00Z
last_update: 2026-05-21T18:26:42Z
date_finished: 2026-05-21T18:26:42Z
---

# T-1790: termlink_channel_info MCP — topic-flexible synthesized topic view (T-1166 wedge)

## Context

Topic-level metadata reader — MCP parity for `termlink channel info <topic> [--since <ms>]` (CLI verb at `crates/termlink-cli/src/commands/channel.rs:2791`, T-1324). Synthesizes a single-shot topic summary: retention, count, latest description, top senders, and latest read-receipt per sender. Topic-flexible by construction — works on `agent-chat-arc`, `dm:a:b`, and project topics.

Three CLI helpers need MCP mirrors:
- `filter_msgs_since` (channel.rs:2185, T-1331) — slice by `ts >= since`
- `summarize_senders` (channel.rs:2767) — sender → post_count, excluding meta msg_types
- `latest_description` (channel.rs:3190, T-1323) — pick most recent `topic_metadata.description`

Tool method: CHANNEL_LIST (retention/count) + paginated CHANNEL_SUBSCRIBE walk (full topic) + optional `--since` bound on the slice. Returns JSON `{topic, retention:{kind, value}, count, description, senders:[...], receipts:[...]}` with `{since, posts_since}` extras when bounded. Shape byte-identical to CLI `--json`.

## Acceptance Criteria

### Agent
- [x] `ChannelInfoParams { topic: String, since: Option<i64> }` struct added.
- [x] `filter_msgs_since_mcp`, `summarize_senders_mcp`, `latest_description_mcp` ported as private helpers — semantically equivalent to CLI originals.
- [x] `termlink_channel_info` tool method registered: CHANNEL_LIST for retention/count, paginated CHANNEL_SUBSCRIBE walk, optional `--since` bounding on description/senders/receipts.
- [x] Tool description references T-1790 + MCP parity for `termlink channel info <topic>`, notes "Topic-flexible variant" (no hardcoded chat-arc sister exists).
- [x] At least 4 unit tests covering: filter_msgs_since_mcp (bound inclusivity), summarize_senders_mcp (meta exclusion), latest_description_mcp (latest wins), ChannelInfoParams JSON parse.
- [x] `cargo build -p termlink-mcp` succeeds.
- [x] `cargo test -p termlink-mcp` succeeds (full crate suite).

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

cd /opt/termlink && cargo build -p termlink-mcp --quiet
cd /opt/termlink && cargo test -p termlink-mcp --quiet

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

### 2026-05-21T18:23:00Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1790-termlinkchannelinfo-mcp--topic-flexible-.md
- **Context:** Initial task creation

## Reviewer Verdict (v1.4)

- **Scan ID:** R-326ed69a
- **Timestamp:** 2026-05-21T18:27:26Z
- **Catalogue:** v1.3-seed
- **Overall:** PASS
- **Needs Human:** no
- **Findings:** none

### 2026-05-21T18:26:42Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
