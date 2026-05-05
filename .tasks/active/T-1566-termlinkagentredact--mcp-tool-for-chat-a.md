---
id: T-1566
name: "termlink_agent_redact — MCP tool for chat-arc post retraction"
description: >
  termlink_agent_redact — MCP tool for chat-arc post retraction

status: work-completed
workflow_type: build
owner: human
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-05-05T13:39:48Z
last_update: 2026-05-05T13:46:13Z
date_finished: 2026-05-05T13:46:13Z
---

# T-1566: termlink_agent_redact — MCP tool for chat-arc post retraction

## Context

T-1564/T-1565 shipped the MCP curation pair (pin/star). This wave adds the retraction verb: `termlink_agent_redact` emits a redaction envelope tied to a parent offset on agent-chat-arc. msg_type="redaction" (note: longer form, matching CLI T-1531), payload="", `metadata.redacts=<offset>` + optional `metadata.reason`. Append-only — original stays in the topic; reader-side decides render. MCP-aware agents can now retract their own posts without shelling out.

## Acceptance Criteria

### Agent
- [x] New `AgentRedactParams` struct (offset, optional reason, sender_id override)
- [x] New `termlink_agent_redact` tool method (msg_type="redaction", metadata.redacts=offset, metadata.reason optional)
- [x] `cargo build --release` clean
- [x] MCP tool count increments to >=82
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
- [ ] [REVIEW] Verify `termlink_agent_redact` is operator-fluent over MCP
  **Steps:**
  1. From an MCP-aware client, call `termlink_agent_redact` with offset=420, reason="oops"
  2. Run `target/release/termlink agent redactions`
  **Expected:** Offset 420 appears in the redactions list with reason="oops".
  **If not:** report ergonomics suggestions.

## Verification

cargo build --release 2>&1 | tail -5 | grep -q -E "Compiling|Finished"
target/release/termlink version --json 2>&1 | grep -qE '"mcp_tools":\s*8[2-9]|"mcp_tools":\s*9[0-9]'
grep -q '"termlink_agent_redact"' crates/termlink-mcp/src/tools.rs
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
**Rationale:** First MCP retraction verb. msg_type="redaction" (longer form per CLI T-1531), `metadata.redacts=<offset>` + optional `metadata.reason`. Append-only so the read-side aggregator (`agent redactions`) and the audit trail stay intact. MCP-aware agents can now retract their own posts without shelling out — closes a gap noted during the T-1525 react-emit work where reactions were the only after-the-fact mutation MCP could do.
**Evidence:**
- Build clean (4m 11s, batched with T-1567)
- `termlink version --json` reports mcp_tools=83 (was 81 after T-1565; T-1566 + T-1567 each add 1)
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

### 2026-05-05T13:39:48Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1566-termlinkagentredact--mcp-tool-for-chat-a.md
- **Context:** Initial task creation

### 2026-05-05T13:46:13Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
