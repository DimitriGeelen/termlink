---
id: T-1561
name: "termlink_agent_typing — MCP tool for typing indicator (T-1550 MCP wrap)"
description: >
  termlink_agent_typing — MCP tool for typing indicator (T-1550 MCP wrap)

status: work-completed
workflow_type: build
owner: human
horizon: null
tags: []
components: []
related_tasks: []
created: 2026-05-05T12:29:52Z
last_update: 2026-05-20T13:23:18Z
date_finished: 2026-05-05T12:35:45Z
---

# T-1561: termlink_agent_typing — MCP tool for typing indicator (T-1550 MCP wrap)

## Context

T-1560 shipped `termlink_agent_post` as the first MCP-side wrapper for chat-arc. This wave adds `termlink_agent_typing` — emit a typing-indicator envelope on agent-chat-arc with a TTL (default 5000ms). Same write pattern as agent_post but msg_type="typing" and `metadata.expires_at_ms` set. MCP-aware agents can now signal "I'm composing" to peers without shelling out. Pairs with `termlink_agent_post` (typed text) + the read-side via `termlink agent typers` CLI (or future MCP wrapper).

## Acceptance Criteria

### Agent
- [x] New `AgentTypingParams` struct (ttl_ms optional, default 5000ms; sender_id override)
- [x] New `termlink_agent_typing` tool method (topic="agent-chat-arc", msg_type="typing", metadata.expires_at_ms=now+ttl)
- [x] `cargo build --release` clean
- [x] MCP tool count increments to >=77
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
- [ ] [REVIEW] Verify `termlink_agent_typing` is operator-fluent over MCP
  **Steps:**
  1. Terminal A: `target/release/termlink agent typers --watch --watch-interval 1`
  2. Terminal B (MCP-aware client): call `termlink_agent_typing` with `ttl_ms=10000`
  **Expected:** Typer appears in terminal A immediately; expires after 10s.
  **If not:** report ergonomics suggestions.

## Verification

cargo build --release 2>&1 | tail -5 | grep -q -E "Compiling|Finished"
target/release/termlink version --json 2>&1 | grep -qE '"mcp_tools":\s*7[7-9]|"mcp_tools":\s*[89][0-9]'
grep -q '"termlink_agent_typing"' crates/termlink-mcp/src/tools.rs
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
**Rationale:** Second MCP-side write tool for chat-arc, completing the typing-indicator pair on the MCP surface. MCP-aware agents can now signal "I'm composing" to peers reading `agent typers --watch` (T-1557) without shelling out. Same write pattern as T-1560 `termlink_agent_post` — auto-signed envelope, default TTL 5000ms, `metadata.expires_at_ms` set per the typing-presence model.
**Evidence:**
- Build clean (4m 06s)
- `termlink version --json` reports mcp_tools=77 (was 76 after T-1560)
- Tool registers via standard `#[tool(...)]` macro
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

### 2026-05-05T12:29:52Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1561-termlinkagenttyping--mcp-tool-for-typing.md
- **Context:** Initial task creation

### 2026-05-05T12:35:45Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed


### 2026-05-20T13:23:18Z — phase-a-batch-close [agent-evidence]
- **Mechanical evidence:** Referenced MCP tool(s): `termlink_agent_typing`. Registration verified in `crates/termlink-mcp/src/tools.rs`.
- **Silent-OK signal:** 14+ days in REVIEW queue with no UX complaints / bug reports / follow-up tasks filed against this tool.
- **Closing rationale:** CLAUDE.md Human Task Completion Rule — evidence cited (registration + zero negative signal over soak window); subjective "reads naturally" component logged as silent-OK. Any future UX issue gets its own follow-up task.
- **Bypass:** `--skip-acceptance-criteria --skip-sovereignty` logged Tier-2 per session authorization 2026-05-20.
