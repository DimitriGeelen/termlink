---
id: T-1783
name: "termlink_channel_ack MCP — topic-flexible read-receipt emit (T-1166 wedge)"
description: >
  termlink_channel_ack MCP — topic-flexible read-receipt emit (T-1166 wedge)

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-05-21T17:16:39Z
last_update: 2026-05-21T17:20:02Z
date_finished: 2026-05-21T17:20:02Z
---

# T-1783: termlink_channel_ack MCP — topic-flexible read-receipt emit (T-1166 wedge)

## Context

T-1166 MCP-parity wedge: port `cmd_channel_ack` CLI verb (channel.rs:1931) to MCP tool wrapper. Topic-flex variant of the hardcoded chat-arc `termlink_agent_ack` sister (tools.rs:14978). Caller-supplied topic enables read-receipts on DM channels (`dm:a:b`) and project topics, not just agent-chat-arc. Pure thin write: explicit `up_to` (CLI's auto-resolve via topic walk not exposed here, matching the sister tool's policy note).

## Acceptance Criteria

### Agent
- [x] `ChannelAckParams` struct exposes `topic`, `up_to`, optional `sender_id`
- [x] `termlink_channel_ack` tool method posts a `msg_type=receipt` envelope with payload `up_to=N` and `metadata.up_to=N` via `CHANNEL_POST`
- [x] Implementation mirrors `termlink_agent_ack` but uses caller-supplied `p.topic` instead of hardcoded `"agent-chat-arc"`
- [x] Description doc references T-1783 + T-1166 + the sister `termlink_agent_ack` tool
- [x] Unit tests: params deserialize round-trip + sanity test for the receipt envelope shape
- [x] `cargo build -p termlink-mcp` passes
- [x] `cargo test -p termlink-mcp` passes

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

### 2026-05-21T17:16:39Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1783-termlinkchannelack-mcp--topic-flexible-r.md
- **Context:** Initial task creation

## Reviewer Verdict (v1.4)

- **Scan ID:** R-d34e97bd
- **Timestamp:** 2026-05-21T17:20:02Z
- **Catalogue:** v1.3-seed
- **Overall:** PASS
- **Needs Human:** no
- **Findings:** none

### 2026-05-21T17:20:02Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
