---
id: T-1789
name: "termlink_channel_poll_results MCP — topic-flexible poll results aggregator (T-1166 wedge)"
description: >
  termlink_channel_poll_results MCP — topic-flexible poll results aggregator (T-1166 wedge)

status: work-completed
workflow_type: build
owner: agent
horizon: null
tags: [mcp, T-1166, channel, poll]
components: [crates/termlink-mcp/src/tools.rs]
related_tasks: [T-1166, T-1788, T-1570, T-1355]
created: 2026-05-21T18:15:49Z
last_update: 2026-05-21T18:21:29Z
date_finished: 2026-05-21T18:21:29Z
---

# T-1789: termlink_channel_poll_results MCP — topic-flexible poll results aggregator (T-1166 wedge)

## Context

Completes the channel-poll lifecycle triad shipped in T-1788 (`channel_poll_start` / `_vote` / `_end`). The read-side aggregator was deferred — this task ports `cmd_channel_poll_results` (CLI verb at `crates/termlink-cli/src/commands/channel.rs:4082`) plus its pure helper `compute_poll_state` (line 3969) into the MCP crate as `termlink_channel_poll_results` + `compute_poll_state_mcp`. Topic-flex variant of `termlink_agent_poll_results` (hardcoded chat-arc).

Pattern mirrors prior wedges: pure helper inlined into `crates/termlink-mcp/src/tools.rs` (the MCP crate has no `termlink-cli` dependency, so cross-crate use is not an option), tool method calls `walk_topic_full_mcp` then `compute_poll_state_mcp`, returns `{poll_id, question, options:[{label, count, voters[]}], closed, total_votes}` JSON.

## Acceptance Criteria

### Agent
- [x] `ChannelPollResultsParams { topic: String, poll_id: u64 }` struct added in tools.rs (alongside other Channel*Params).
- [x] `compute_poll_state_mcp` + `PollOptionRowMcp` + `PollStateMcp` ported as private helpers inside tools.rs, semantically equivalent to CLI's `compute_poll_state` (same poll_start/_vote/_end semantics, latest-wins-by-offset-order, drop-out-of-range-choice, drop-vote-after-poll_end-ts).
- [x] `termlink_channel_poll_results` tool method registered via `#[tool(name = ...)]`, walks `walk_topic_full_mcp(&hub_socket, &p.topic)`, computes state, returns JSON identical in shape to CLI's `--json` mode.
- [x] Tool description references T-1789, MCP parity for `termlink channel poll results <topic> <poll_id>`, and notes "Topic-flexible variant of `termlink_agent_poll_results`".
- [x] At least 3 unit tests covering: (a) compute_poll_state_mcp on a synthetic envelope set matching the CLI test pattern, (b) ChannelPollResultsParams JSON parse, (c) closed-poll boundary (votes after poll_end ts dropped).
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

### 2026-05-21T18:15:49Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1789-termlinkchannelpollresults-mcp--topic-fl.md
- **Context:** Initial task creation

## Reviewer Verdict (v1.4)

- **Scan ID:** R-b9c83da0
- **Timestamp:** 2026-05-21T18:22:16Z
- **Catalogue:** v1.3-seed
- **Overall:** PASS
- **Needs Human:** no
- **Findings:** none

### 2026-05-21T18:21:29Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
