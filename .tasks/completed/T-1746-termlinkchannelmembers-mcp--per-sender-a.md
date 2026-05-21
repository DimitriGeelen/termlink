---
id: T-1746
name: "termlink_channel_members MCP — per-sender activity for arbitrary topic (T-1166 MCP-parity)"
description: >
  MCP wrapper for channel.rs cmd_channel_members. Sister to termlink_agent_peers (which is hardcoded to agent-chat-arc with a 3-field row) — channel_members works on ANY topic and returns the richer 4-field MemberRow (sender_id, posts, first_ts, last_ts). Supports as_of_ms cutoff for historical snapshots and include_meta toggle. Mirrors summarize_members + summarize_members_as_of pure helpers.

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-05-21T07:51:40Z
last_update: 2026-05-21T07:54:28Z
date_finished: 2026-05-21T07:54:28Z
---

# T-1746: termlink_channel_members MCP — per-sender activity for arbitrary topic (T-1166 MCP-parity)

## Context

MCP wrapper for channel.rs `cmd_channel_members` (CLI T-1341). Existing
`termlink_agent_peers` is hardcoded to `agent-chat-arc` and emits a 3-field row
{sender_id, post_count, last_post_ts}. CLI's MemberRow has 4 fields including
first_ts and supports as_of_ms historical snapshots + include_meta toggle. This
verb fills the channel-membership gap for arbitrary topics. Helpers:
`summarize_members` (channel.rs:2476) + `summarize_members_as_of` (channel.rs:2520).

## Acceptance Criteria

### Agent
- [x] `MemberRowMcp` struct + `to_json_mcp` — 4-field one-to-one mirror of CLI MemberRow
- [x] Pure helpers `summarize_members_mcp` + `summarize_members_as_of_mcp` — placed near compute_state_mcp
- [x] `ChannelMembersParams { topic, include_meta?, as_of_ms? }`
- [x] `termlink_channel_members` tool method — fetch_topic_msgs_mcp(topic, 2000) + helper, returns `{ok, topic, include_meta, as_of_ms, members, count}`
- [x] 10 unit tests added (8 helper, 2 params): empty, multi-sender accumulation, meta-skipped-by-default, include_meta=true, empty-sender-id-skipped, no-ts handling, as_of cutoff, as_of=None equivalence
- [x] `cargo build -p termlink-mcp` clean
- [x] `cargo test -p termlink-mcp` 383 → 393 passing, 0 regressions

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
cargo build -p termlink-mcp
cargo test -p termlink-mcp --lib 2>&1 | tail -5 | grep -q "test result: ok"

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

### 2026-05-21T07:51:40Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1746-termlinkchannelmembers-mcp--per-sender-a.md
- **Context:** Initial task creation

## Reviewer Verdict (v1.4)

- **Scan ID:** R-7ae7e5e0
- **Timestamp:** 2026-05-21T07:54:28Z
- **Catalogue:** v1.3-seed
- **Overall:** PASS
- **Needs Human:** no
- **Findings:** none

### 2026-05-21T07:54:28Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
