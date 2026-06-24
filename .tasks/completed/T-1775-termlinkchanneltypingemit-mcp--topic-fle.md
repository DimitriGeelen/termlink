---
id: T-1775
name: "termlink_channel_typing_emit MCP — topic-flexible typing-indicator emit"
description: >
  termlink_channel_typing_emit MCP — topic-flexible typing-indicator emit

status: work-completed
workflow_type: build
owner: agent
horizon: null
tags: []
components: [crates/termlink-mcp/src/tools.rs]
related_tasks: []
created: 2026-05-21T15:18:50Z
last_update: 2026-05-21T15:21:27Z
date_finished: 2026-05-21T15:21:27Z
---

# T-1775: termlink_channel_typing_emit MCP — topic-flexible typing-indicator emit

## Context

T-1166 MCP-parity wedge. Port `termlink channel typing-emit` (CLI `cmd_channel_typing_emit` at channel.rs:3303) to MCP. Write companion to T-1773 (`channel_typing_list` read side). Sister `termlink_agent_typing` (tools.rs:14368) is hardcoded to `agent-chat-arc`; this variant adds **topic-flexibility** so MCP agents can emit typing indicators on DM topics (`dm:*`) and project topics. Closes the typing read+write loop at the channel-flex layer.

## Acceptance Criteria

### Agent
- [x] `ChannelTypingEmitParams { topic: String, ttl_ms: Option<u64>, sender_id: Option<String> }` params struct.
- [x] `termlink_channel_typing_emit` tool method registered. Builds `msg_type=typing` envelope with empty payload, `metadata.expires_at_ms = now_ms + ttl_ms` (default 5000ms), signs via local identity, POSTs via `CHANNEL_POST` RPC. Mirrors `termlink_agent_typing` at tools.rs:14371 with caller-supplied topic.
- [x] Unit test: params struct deserializes with topic-only (defaults), topic+ttl_ms, topic+sender_id override (3 tests).
- [x] `cargo build -p termlink-mcp` clean.
- [x] `cargo test -p termlink-mcp channel_typing_emit` all pass.

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
cd /opt/termlink && cargo build -p termlink-mcp 2>&1 | tail -5
cd /opt/termlink && cargo test -p termlink-mcp channel_typing_emit 2>&1 | tail -10

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

### 2026-05-21T15:18:50Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1775-termlinkchanneltypingemit-mcp--topic-fle.md
- **Context:** Initial task creation

## Reviewer Verdict (v1.4)

- **Scan ID:** R-1292bc97
- **Timestamp:** 2026-05-21T15:21:40Z
- **Catalogue:** v1.3-seed
- **Overall:** PASS
- **Needs Human:** no
- **Findings:** none

### 2026-05-21T15:21:27Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
