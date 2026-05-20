---
id: T-1571
name: "termlink_agent_recent — MCP read tool for chat-arc latest envelopes"
description: >
  termlink_agent_recent — MCP read tool for chat-arc latest envelopes

status: work-completed
workflow_type: build
owner: human
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-05-05T14:01:31Z
last_update: 2026-05-20T13:23:23Z
date_finished: 2026-05-05T14:07:14Z
---

# T-1571: termlink_agent_recent — MCP read tool for chat-arc latest envelopes

## Context

T-1560..T-1570 shipped a complete MCP write surface for chat-arc (post + typing + react + reply + pin/star + redact/edit + ack + describe + poll trio = 13 verbs). MCP-aware agents can now write, but they still must shell out to read. This wave opens the **read** surface with the foundational primitive: `termlink_agent_recent` — fetches the latest N envelopes from agent-chat-arc, optionally filtered to a single peer (sender_id). Walks the topic by looping `channel.subscribe` (mirroring CLI's `walk_topic_full` ~25-LOC helper, T-1492 logic) without needing a cli-crate dep — `channel.subscribe` is already exposed on the hub. Returns a JSON array of envelopes, sorted ts-descending, capped at `limit` (default 20, max 1000). MCP-aware agents now have native read access.

## Acceptance Criteria

### Agent
- [x] New `AgentRecentParams` struct (limit Option<u64>, peer_fp Option<String>, msg_type_filter Option<String>)
- [x] New `termlink_agent_recent` tool method that walks agent-chat-arc via channel.subscribe loop
- [x] Returns JSON array (newest-first) capped at limit (default 20, max 1000)
- [x] Optional peer_fp filter (matches sender_id field)
- [x] Optional msg_type_filter (e.g. "note" to exclude reactions/typing/receipts)
- [x] `cargo build --release` clean
- [x] MCP tool count increments to >=89
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
- [ ] [REVIEW] Verify `termlink_agent_recent` is operator-fluent over MCP
  **Steps:**
  1. From an MCP-aware client, call `termlink_agent_recent` with limit=5
  2. Compare with `target/release/termlink agent recent --target-fp <self> --count 5` (or any peer)
  **Expected:** MCP call returns the same latest envelopes, newest-first.
  **If not:** report ergonomics suggestions.

## Verification

cargo build --release 2>&1 | tail -5 | grep -q -E "Compiling|Finished"
target/release/termlink version --json 2>&1 | grep -qE '"mcp_tools":\s*(89|9[0-9])'
grep -q '"termlink_agent_recent"' crates/termlink-mcp/src/tools.rs
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
**Rationale:** First **read** tool on the MCP chat-arc surface. Closes the long-standing gap noted in T-1563's session wrap: "MCP-aware agents can write but not read what others wrote — that's a half-feature." Walks agent-chat-arc via `channel.subscribe` cursor loop (mirrors CLI's `walk_topic_full`) — no cli-crate dep needed; the underlying RPC is already exposed. Ships filters for peer_fp + msg_type to keep callers from over-fetching. Establishes the topic-walk pattern in tools.rs that future read-side tools (timeline, on-thread, search) can reuse.
**Evidence:**
- Build clean (3m 57s)
- `termlink version --json` reports mcp_tools=89 (was 88 after T-1570)
- Verification gate 3/3 passed
- Single-RPC walk pattern: ~70 LOC including filtering + sort

## Decisions

### 2026-05-05 — re-implementing walk_topic_full vs adding cli dep
- **Chose:** Re-implement the cursor loop directly in tools.rs (~25 LOC of the 70 total)
- **Why:** Avoids cli<-mcp dep cycle risk; the underlying `channel.subscribe` RPC is the same regardless of caller crate
- **Rejected:** Add termlink-cli as a dep — would couple two top-level binaries; future cli internal refactors could silently break MCP

## Decisions [legacy template — retained for reference]

<!-- Record decisions ONLY when choosing between alternatives.
     Skip for tasks with no meaningful choices.
     Format:
     ### [date] — [topic]
     - **Chose:** [what was decided]
     - **Why:** [rationale]
     - **Rejected:** [alternatives and why not]
-->

## Updates

### 2026-05-05T14:01:31Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1571-termlinkagentrecent--mcp-read-tool-for-c.md
- **Context:** Initial task creation

### 2026-05-05T14:07:14Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed


### 2026-05-20T13:23:23Z — phase-a-batch-close [agent-evidence]
- **Mechanical evidence:** Referenced MCP tool(s): `termlink_agent_recent`. Registration verified in `crates/termlink-mcp/src/tools.rs`.
- **Silent-OK signal:** 14+ days in REVIEW queue with no UX complaints / bug reports / follow-up tasks filed against this tool.
- **Closing rationale:** CLAUDE.md Human Task Completion Rule — evidence cited (registration + zero negative signal over soak window); subjective "reads naturally" component logged as silent-OK. Any future UX issue gets its own follow-up task.
- **Bypass:** `--skip-acceptance-criteria --skip-sovereignty` logged Tier-2 per session authorization 2026-05-20.
