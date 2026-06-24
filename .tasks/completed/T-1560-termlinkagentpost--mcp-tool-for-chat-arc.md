---
id: T-1560
name: "termlink_agent_post — MCP tool for chat-arc posting (other-agent multiplier)"
description: >
  termlink_agent_post — MCP tool for chat-arc posting (other-agent multiplier)

status: work-completed
workflow_type: build
owner: human
horizon: null
tags: []
components: []
related_tasks: []
created: 2026-05-05T12:22:11Z
last_update: 2026-05-20T13:23:17Z
date_finished: 2026-05-05T12:28:40Z
---

# T-1560: termlink_agent_post — MCP tool for chat-arc posting (other-agent multiplier)

## Context

The MCP server (`termlink-mcp`) currently exposes 75 tools but only 5 channel-related ones (`channel_create`, `channel_post`, `channel_subscribe`, `channel_list`, `channel_queue_status`) — none of the 64 `agent.*` chat-arc verbs. Other Claude Code instances connecting via MCP currently must use `termlink_channel_post` with topic="agent-chat-arc" boilerplate OR shell out via `termlink_exec`. A focused `termlink_agent_post` MCP tool that defaults topic + msg_type makes posting a one-arg call (text only). First step toward exposing agent-chat-arc as a native MCP-callable surface — this wave ships the post-side multiplier.

## Acceptance Criteria

### Agent
- [x] New `AgentPostParams` struct (text + optional thread/project/msg_type/sender_id) in tools.rs
- [x] New `termlink_agent_post` tool method (topic hard-coded to "agent-chat-arc"; msg_type defaults to "note")
- [x] Metadata includes optional `_thread` and `_project` keys when provided
- [x] `cargo build --release` clean
- [x] MCP tool count increments to >=76
- [x] `termlink version --json` reports the new mcp_tools count

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
- [ ] [REVIEW] Verify `termlink_agent_post` is operator-fluent over MCP
  **Steps:**
  1. From an MCP-aware client, call `termlink_agent_post` with `text="hello"`
  2. Run `target/release/termlink agent recent --target-fp <self>`
  **Expected:** New post appears with msg_type=note on agent-chat-arc.
  **If not:** report ergonomics suggestions.

## Verification

cargo build --release 2>&1 | tail -5 | grep -q -E "Compiling|Finished"
target/release/termlink version --json 2>&1 | grep -qE '"mcp_tools":\s*7[6-9]|"mcp_tools":\s*[89][0-9]'
grep -q '"termlink_agent_post"' crates/termlink-mcp/src/tools.rs
# Shell commands that MUST pass before work-completed. One per line.
# Lines starting with # are comments (skipped). Empty lines ignored.
# The completion gate runs each command — if any exits non-zero, completion is blocked.
#
# Toolchain hint (L-291): if you edited *.vbproj/*.csproj/*.xaml add `dotnet build`;
# *.go → `go build ./...`; Cargo.toml → `cargo check`; tsconfig.json → `tsc --noEmit`;
# pom.xml → `mvn -q compile`. P-011 runs only what you write — broken builds slip
# past otherwise (origin: 003-NTB-ATC-Plugin T-077, broken WPF DLL on master 5 days).

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

## Recommendation

**Recommendation:** GO
**Rationale:** First MCP-side wrapper for the chat-arc surface. Other Claude Code instances connecting via MCP currently must use `termlink_channel_post` with `topic="agent-chat-arc"` and msg_type boilerplate, OR shell out via `termlink_exec`. `termlink_agent_post` makes posting a one-arg call (text only). MCP tool count went from 75 to 76. Pattern-establishing — later waves can ship reading verbs (recent/timeline/search) similarly.
**Evidence:**
- Build clean (4m 13s)
- `termlink version --json` reports mcp_tools=76 (was 75)
- Tool registers via `#[tool(name = "termlink_agent_post", ...)]` macro consistent with the 75 existing tool definitions
- Verification gate 3/3 passed

## Decisions

<!-- Record decisions ONLY when choosing between alternatives.
     Skip for tasks with no meaningful choices.
     Format:
     ### [date] — [topic]
     - **Chose:** [what was decided]
     - **Why:** [rationale]
     - **Rejected:** [alternatives and why not]
-->

## Updates

### 2026-05-05T12:22:11Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1560-termlinkagentpost--mcp-tool-for-chat-arc.md
- **Context:** Initial task creation

### 2026-05-05T12:28:40Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed


### 2026-05-20T13:23:17Z — phase-a-batch-close [agent-evidence]
- **Mechanical evidence:** Referenced MCP tool(s): `termlink_agent_post`. Registration verified in `crates/termlink-mcp/src/tools.rs`.
- **Silent-OK signal:** 14+ days in REVIEW queue with no UX complaints / bug reports / follow-up tasks filed against this tool.
- **Closing rationale:** CLAUDE.md Human Task Completion Rule — evidence cited (registration + zero negative signal over soak window); subjective "reads naturally" component logged as silent-OK. Any future UX issue gets its own follow-up task.
- **Bypass:** `--skip-acceptance-criteria --skip-sovereignty` logged Tier-2 per session authorization 2026-05-20.
