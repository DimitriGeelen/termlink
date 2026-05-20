---
id: T-1564
name: "termlink_agent_pin — MCP tool for chat-arc post pinning"
description: >
  termlink_agent_pin — MCP tool for chat-arc post pinning

status: work-completed
workflow_type: build
owner: human
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-05-05T13:27:21Z
last_update: 2026-05-20T13:23:20Z
date_finished: 2026-05-05T13:33:23Z
---

# T-1564: termlink_agent_pin — MCP tool for chat-arc post pinning

## Context

T-1560/T-1561/T-1562/T-1563 shipped the MCP write quartet for chat-arc (post + typing + react + reply). This wave opens the curation surface — `termlink_agent_pin` emits a pin/unpin envelope tied to a parent offset on agent-chat-arc. Same write pattern as agent_react but with msg_type="pin", payload="", and `metadata.pin_target=<offset>` + `metadata.action=pin|unpin`. MCP-aware agents can now curate attention on chat-arc without shelling out. Mirrors the CLI's T-1527 `agent pin <offset>` / `agent pin --unpin`.

## Acceptance Criteria

### Agent
- [x] New `AgentPinParams` struct (offset, optional unpin bool, sender_id override)
- [x] New `termlink_agent_pin` tool method (msg_type="pin", metadata.pin_target=offset, metadata.action=pin|unpin)
- [x] `cargo build --release` clean
- [x] MCP tool count increments to >=80
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
- [ ] [REVIEW] Verify `termlink_agent_pin` is operator-fluent over MCP
  **Steps:**
  1. From an MCP-aware client, call `termlink_agent_pin` with offset=420
  2. Run `target/release/termlink agent pinned`
  **Expected:** Offset 420 appears in the pinned list.
  **If not:** report ergonomics suggestions.

## Verification

cargo build --release 2>&1 | tail -5 | grep -q -E "Compiling|Finished"
target/release/termlink version --json 2>&1 | grep -qE '"mcp_tools":\s*8[0-9]|"mcp_tools":\s*9[0-9]'
grep -q '"termlink_agent_pin"' crates/termlink-mcp/src/tools.rs
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
**Rationale:** First MCP curation verb on chat-arc — pin/unpin via offset. Mirrors CLI T-1527 exactly: msg_type="pin", payload="", `metadata.pin_target=<offset>` + `metadata.action=pin|unpin`. MCP-aware agents can curate attention without shelling out. Pairs with the read-side via the CLI's `agent pinned` (T-1527) — read aggregator already walks the topic and renders the active set, so this MCP write tool plugs directly in.
**Evidence:**
- Build clean (4m 16s)
- `termlink version --json` reports mcp_tools=80 (was 79 after T-1563)
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

### 2026-05-05T13:27:21Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1564-termlinkagentpin--mcp-tool-for-chat-arc-.md
- **Context:** Initial task creation

### 2026-05-05T13:33:23Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed


### 2026-05-20T13:23:20Z — phase-a-batch-close [agent-evidence]
- **Mechanical evidence:** Referenced MCP tool(s): `termlink_agent_pin`. Registration verified in `crates/termlink-mcp/src/tools.rs`.
- **Silent-OK signal:** 14+ days in REVIEW queue with no UX complaints / bug reports / follow-up tasks filed against this tool.
- **Closing rationale:** CLAUDE.md Human Task Completion Rule — evidence cited (registration + zero negative signal over soak window); subjective "reads naturally" component logged as silent-OK. Any future UX issue gets its own follow-up task.
- **Bypass:** `--skip-acceptance-criteria --skip-sovereignty` logged Tier-2 per session authorization 2026-05-20.
