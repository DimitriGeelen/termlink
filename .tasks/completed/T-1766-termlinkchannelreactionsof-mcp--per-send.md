---
id: T-1766
name: "termlink_channel_reactions_of MCP — per-sender reaction history with parent_payload on arbitrary topic (T-1166 wedge)"
description: >
  Port channel reactions-of CLI verb to MCP. compute_reactions_of_mcp helper exists (T-1735) but is unused by agent_reactions_by which inlines a simpler shape. This wedge ships topic-flexible variant using the richer helper.

status: work-completed
workflow_type: build
owner: agent
horizon: null
tags: [termlink, mcp, t-1166]
components: []
related_tasks: []
created: 2026-05-21T12:41:37Z
last_update: 2026-05-21T12:43:45Z
date_finished: 2026-05-21T12:43:45Z
---

# T-1766: termlink_channel_reactions_of MCP — per-sender reaction history with parent_payload on arbitrary topic (T-1166 wedge)

## Context

T-1166 MCP-parity wedge. CLI verb `cmd_channel_reactions_of` (channel.rs:4682) uses pure helper `compute_reactions_of` (channel.rs:4631), returning `ReactionsOfRow { reaction_offset, parent_offset, emoji, parent_payload: Option<String>, ts }`. MCP has the pure helper `compute_reactions_of_mcp` (tools.rs:3319) already ported under T-1735 — but `termlink_agent_reactions_by` (tools.rs:16434) doesn't use it; it inlines a simpler shape `{emoji, in_reply_to, ts_unix_ms}` with NO parent_payload and NO redaction handling.

Three value-adds:
1. Topic-flexible (any DM/topic)
2. Richer row shape with `parent_payload` (preview of original post)
3. Respects redactions via helper

## Acceptance Criteria

### Agent
- [x] `ChannelReactionsOfParams { topic: String, sender_id: Option<String> }` struct
- [x] `termlink_channel_reactions_of` tool method uses `walk_topic_full_mcp` + existing `compute_reactions_of_mcp` helper
- [x] Returns `{ok, topic, sender_id, rows: [{reaction_offset, parent_offset, emoji, parent_payload, ts}, ...]}` sorted newest-first
- [x] sender_id defaults to local Identity fingerprint when not supplied
- [x] 2 params deserialize tests — both pass
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
cd /opt/termlink && cargo test -p termlink-mcp channel_reactions_of_params 2>&1 | tail -5 | grep -q "test result: ok" && echo TESTS_OK

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

### 2026-05-21T12:41:37Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1766-termlinkchannelreactionsof-mcp--per-send.md
- **Context:** Initial task creation

### 2026-05-21T12:42:10Z — status-update [task-update-agent]
- **Change:** status: captured → started-work

## Reviewer Verdict (v1.4)

- **Scan ID:** R-9edebd25
- **Timestamp:** 2026-05-21T12:43:46Z
- **Catalogue:** v1.3-seed
- **Overall:** PASS
- **Needs Human:** no
- **Findings:** none

### 2026-05-21T12:43:45Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
