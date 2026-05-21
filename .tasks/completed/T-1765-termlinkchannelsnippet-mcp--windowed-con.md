---
id: T-1765
name: "termlink_channel_snippet MCP — windowed content preview around offset on arbitrary topic (T-1166 wedge)"
description: >
  Port channel snippet CLI verb to MCP. compute_snippet_mcp helper already exists (used by agent_snippet); this wedge adds topic-flexible tool wrapper.

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: [termlink, mcp, t-1166]
components: []
related_tasks: []
created: 2026-05-21T12:38:32Z
last_update: 2026-05-21T12:40:47Z
date_finished: 2026-05-21T12:40:47Z
---

# T-1765: termlink_channel_snippet MCP — windowed content preview around offset on arbitrary topic (T-1166 wedge)

## Context

T-1166 MCP-parity wedge. CLI verb `cmd_channel_snippet` (channel.rs:4555) uses pure helper `compute_snippet` (channel.rs:4514). MCP side already has `compute_snippet_mcp` helper (tools.rs:872) used by `termlink_agent_snippet` (tools.rs:13641), but agent_snippet hardcodes `topic="agent-chat-arc"`. This is a thin port: reuse the existing helper, parameterize the topic. Value-add = topic-flexibility (preview around a DM message offset, not just chat-arc).

## Acceptance Criteria

### Agent
- [x] `ChannelSnippetParams { topic: String, target: u64, lines: Option<u64> }` struct
- [x] `termlink_channel_snippet` tool method reuses `walk_topic_full_mcp` + existing `compute_snippet_mcp` helper
- [x] `lines` defaults to 3, clamped 1..=50 (parity with agent_snippet)
- [x] Errors when target absent or is a meta msg-type (via compute_snippet_mcp returning None)
- [x] Returns `{ok, topic, target_offset, lines: [...]}`
- [x] 2 params deserialize tests (minimal + with lines) — both pass
- [x] `cargo build -p termlink-mcp` clean

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
cd /opt/termlink && cargo build -p termlink-mcp 2>&1 | tail -3 | grep -qE "Finished|warning" && echo OK
cd /opt/termlink && cargo test -p termlink-mcp channel_snippet_params 2>&1 | tail -5 | grep -q "test result: ok" && echo TESTS_OK

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

### 2026-05-21T12:38:32Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1765-termlinkchannelsnippet-mcp--windowed-con.md
- **Context:** Initial task creation

### 2026-05-21T12:39:04Z — status-update [task-update-agent]
- **Change:** status: captured → started-work

## Reviewer Verdict (v1.4)

- **Scan ID:** R-f90ba465
- **Timestamp:** 2026-05-21T12:40:47Z
- **Catalogue:** v1.3-seed
- **Overall:** PASS
- **Needs Human:** no
- **Findings:** none

### 2026-05-21T12:40:47Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
