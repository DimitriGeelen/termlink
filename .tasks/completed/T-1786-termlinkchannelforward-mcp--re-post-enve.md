---
id: T-1786
name: "termlink_channel_forward MCP — re-post envelope across topics (T-1166 wedge, NEW verb)"
description: >
  termlink_channel_forward MCP — re-post envelope across topics (T-1166 wedge, NEW verb)

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-05-21T17:30:22Z
last_update: 2026-05-21T17:32:16Z
date_finished: 2026-05-21T17:32:16Z
---

# T-1786: termlink_channel_forward MCP — re-post envelope across topics (T-1166 wedge, NEW verb)

## Context

T-1166 MCP-parity wedge: port `cmd_channel_forward` CLI verb (channel.rs:3455) to MCP tool wrapper. NEW verb in MCP — no sister `agent_forward` exists (forward inherently requires a destination topic, so it's topic-flex by nature). Walks source topic for the envelope at offset, then re-posts a new envelope on the destination topic with same msg_type + payload, but signed by the current identity (forwarder is sender-on-record). Metadata records `forwarded_from=<src_topic>:<offset>` + `forwarded_sender=<original>` for trace-back. Especially useful given G-060 (chat-arc federation gap) — operators can forward a chat-arc post to a project topic that DOES federate.

## Acceptance Criteria

### Agent
- [x] `ChannelForwardParams` struct exposes `src_topic`, `offset`, `dst_topic`, optional `sender_id`
- [x] `termlink_channel_forward` tool method walks src via `CHANNEL_SUBSCRIBE`, extracts payload+msg_type+original_sender, posts to dst via `CHANNEL_POST` with `metadata.forwarded_from=src:N` + `metadata.forwarded_sender=<original>`
- [x] Implementation mirrors CLI `cmd_channel_forward` semantics (forwarder is sender on record — NOT a faithful relay)
- [x] Returns clear error if source envelope at offset not found
- [x] Description doc references T-1786 + T-1166 + the no-sister-MCP note + the G-060 motivation
- [x] Unit tests: params deserialize + metadata-shape sanity matching CLI `build_forward_metadata`
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

### 2026-05-21T17:30:22Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1786-termlinkchannelforward-mcp--re-post-enve.md
- **Context:** Initial task creation

## Reviewer Verdict (v1.4)

- **Scan ID:** R-c9ef6df3
- **Timestamp:** 2026-05-21T17:32:33Z
- **Catalogue:** v1.3-seed
- **Overall:** PASS
- **Needs Human:** no
- **Findings:** none

### 2026-05-21T17:32:16Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
