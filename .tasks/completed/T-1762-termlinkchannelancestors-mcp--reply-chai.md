---
id: T-1762
name: "termlink_channel_ancestors MCP — reply-chain root→leaf for arbitrary topic (T-1166 wedge)"
description: >
  Port channel ancestors CLI verb to MCP. agent_ancestors is hardcoded to agent-chat-arc; channel_ancestors accepts topic param. Extract pure compute_ancestors_mcp helper for unit-testability.

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: [termlink, mcp, t-1166]
components: []
related_tasks: []
created: 2026-05-21T12:23:15Z
last_update: 2026-05-21T12:28:57Z
date_finished: 2026-05-21T12:28:57Z
---

# T-1762: termlink_channel_ancestors MCP — reply-chain root→leaf for arbitrary topic (T-1166 wedge)

## Context

T-1166 MCP-parity wedge. CLI verb `cmd_channel_ancestors` (channel.rs:2626) walks the topic with `walk_topic_full`, builds an offset→envelope map, then uses pure helper `build_ancestors` (channel.rs:2592) to chain via `metadata.in_reply_to` until reaching a root. MCP side has `termlink_agent_ancestors` (tools.rs:14720) but with the loop inlined and hardcoded to `agent-chat-arc`. This wedge ships the topic-flexible variant AND extracts a `compute_ancestors_mcp` pure helper that mirrors `build_ancestors` one-to-one — gains unit-testability.

## Acceptance Criteria

### Agent
- [x] `compute_ancestors_mcp(by_offset: &HashMap<u64, &Value>, leaf: u64, max_depth: usize) -> Vec<u64>` pure helper added (tools.rs:2678)
- [x] Helper guards against cycles (visited set) and missing-parent (returns chain so far, root→leaf order)
- [x] `ChannelAncestorsParams { topic: String, offset: u64, max_depth: Option<u64> }` struct
- [x] `termlink_channel_ancestors` tool method uses `walk_topic_full_mcp` + `compute_ancestors_mcp`; refuses if leaf offset absent
- [x] Returns `{ok, topic, leaf, ancestors: [{offset, sender_id, msg_type, ts, payload}, ...]}`
- [x] 6 helper unit tests: linear chain, leaf-with-no-parent, missing-leaf, cycle-detection, max-depth cap, missing-parent break
- [x] Plus 2 params deserialize tests (basic + max_depth override) — all pass
- [x] `cargo build -p termlink-mcp` clean (only pre-existing cur_run_end warning)

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
cd /opt/termlink && cargo test -p termlink-mcp ancestors 2>&1 | tail -5 | grep -q "test result: ok" && echo TESTS_OK

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

### 2026-05-21T12:23:15Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1762-termlinkchannelancestors-mcp--reply-chai.md
- **Context:** Initial task creation

### 2026-05-21T12:23:44Z — status-update [task-update-agent]
- **Change:** status: captured → started-work

## Reviewer Verdict (v1.4)

- **Scan ID:** R-80ef39c4
- **Timestamp:** 2026-05-21T12:28:58Z
- **Catalogue:** v1.3-seed
- **Overall:** PASS
- **Needs Human:** no
- **Findings:** none

### 2026-05-21T12:28:57Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
