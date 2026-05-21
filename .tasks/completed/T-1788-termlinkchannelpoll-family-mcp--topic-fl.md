---
id: T-1788
name: "termlink_channel_poll family MCP — topic-flexible poll lifecycle (T-1166 wedge)"
description: >
  termlink_channel_poll family MCP — topic-flexible poll lifecycle (T-1166 wedge)

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-05-21T17:39:16Z
last_update: 2026-05-21T17:41:35Z
date_finished: 2026-05-21T17:41:35Z
---

# T-1788: termlink_channel_poll family MCP — topic-flexible poll lifecycle (T-1166 wedge)

## Context

T-1166 MCP-parity wedge: port the `channel_poll_*` lifecycle CLI verbs (channel.rs:3858/3893/3921) to MCP tool wrappers. Topic-flex variants of the hardcoded chat-arc `termlink_agent_poll_*` sisters (tools.rs:16821/16898/16969 — shipped T-1570). Per T-1570 precedent, the three write-side verbs (start/vote/end) are bundled into one task — they share structure and are tiny (each is a thin envelope emit). Results aggregator (channel.poll_results) is a separate read-side wedge, deferred.

## Acceptance Criteria

### Agent
- [x] `ChannelPollStartParams` exposes `topic`, `question`, `options: Vec<String>`, optional `sender_id`
- [x] `ChannelPollVoteParams` exposes `topic`, `poll_id`, `choice`, optional `sender_id`
- [x] `ChannelPollEndParams` exposes `topic`, `poll_id`, optional `sender_id`
- [x] `termlink_channel_poll_start` posts `msg_type=poll_start` with payload=question, `metadata.poll_options=a|b|c`, validates ≥2 options + no `|` in labels
- [x] `termlink_channel_poll_vote` posts `msg_type=poll_vote` with `metadata.poll_id` + `metadata.poll_choice`
- [x] `termlink_channel_poll_end` posts `msg_type=poll_end` with `metadata.poll_id`
- [x] Description docs reference T-1788 + T-1166 + sister agent_poll_* MCPs
- [x] Unit tests per verb: params deserialize + sanity for metadata shape + poll_start option validation
- [x] `cargo build -p termlink-mcp` passes
- [x] `cargo test -p termlink-mcp --lib` passes

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

# Shell commands that MUST pass before work-completed. One per line.
# Lines starting with # are comments (skipped). Empty lines ignored.
# The completion gate runs each command — if any exits non-zero, completion is blocked.
#
# Toolchain hint (L-291): if you edited *.vbproj/*.csproj/*.xaml add `dotnet build`;
# *.go → `go build ./...`; Cargo.toml → `cargo check`; tsconfig.json → `tsc --noEmit`;
# pom.xml → `mvn -q compile`. P-011 runs only what you write — broken builds slip
# past otherwise (origin: 003-NTB-ATC-Plugin T-077, broken WPF DLL on master 5 days).

cd /opt/termlink && cargo build -p termlink-mcp
cd /opt/termlink && cargo test -p termlink-mcp --lib

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

### 2026-05-21T17:39:16Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1788-termlinkchannelpoll-family-mcp--topic-fl.md
- **Context:** Initial task creation

## Reviewer Verdict (v1.4)

- **Scan ID:** R-62a75686
- **Timestamp:** 2026-05-21T17:41:53Z
- **Catalogue:** v1.3-seed
- **Overall:** PASS
- **Needs Human:** no
- **Findings:** none

### 2026-05-21T17:41:35Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
