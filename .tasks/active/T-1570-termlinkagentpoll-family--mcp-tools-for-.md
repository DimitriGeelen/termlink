---
id: T-1570
name: "termlink_agent_poll family — MCP tools for chat-arc poll lifecycle (start/vote/end)"
description: >
  termlink_agent_poll family — MCP tools for chat-arc poll lifecycle (start/vote/end)

status: work-completed
workflow_type: build
owner: human
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-05-05T13:53:23Z
last_update: 2026-05-05T14:00:27Z
date_finished: 2026-05-05T14:00:19Z
---

# T-1570: termlink_agent_poll family — MCP tools for chat-arc poll lifecycle (start/vote/end)

## Context

This wave bundles the chat-arc poll lifecycle into one MCP tool family — three thin write verbs sharing a single deliverable: collaborative decision-making for MCP-aware agents. Mirrors CLI T-1543/T-1544/T-1545 envelope shapes:
- `termlink_agent_poll_start` — msg_type="poll_start", payload=question, metadata.poll_options=opt1|opt2|...
- `termlink_agent_poll_vote` — msg_type="poll_vote", payload="", metadata={poll_id, poll_choice}
- `termlink_agent_poll_end` — msg_type="poll_end", payload="", metadata.poll_id

Bundled as one task (per T-1559 precedent) because the three verbs are useless without each other — they form a single coherent surface, not three independent features. Read-side aggregator `agent poll-results` (T-1546) walks the topic and tallies via `compute_poll_state`; this MCP family plugs in unchanged.

## Acceptance Criteria

### Agent
- [x] New `AgentPollStartParams` (question, options Vec<String>, sender_id override)
- [x] New `AgentPollVoteParams` (poll_id u64, choice u64, sender_id override)
- [x] New `AgentPollEndParams` (poll_id u64, sender_id override)
- [x] Three new tool methods: `termlink_agent_poll_start`, `termlink_agent_poll_vote`, `termlink_agent_poll_end`
- [x] `cargo build --release` clean
- [x] MCP tool count increments to >=88 (3 new)
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
- [ ] [REVIEW] Verify `termlink_agent_poll_*` family is operator-fluent over MCP
  **Steps:**
  1. From an MCP-aware client, call `termlink_agent_poll_start` with question="Approve the cut?", options=["yes", "no", "wait"]
  2. Note returned offset (let's call it P)
  3. Call `termlink_agent_poll_vote` with poll_id=P, choice=0
  4. Call `termlink_agent_poll_end` with poll_id=P
  5. Run `target/release/termlink agent poll-results <P>`
  **Expected:** Poll renders with the question, options, 1 vote on "yes", and shows closed.
  **If not:** report ergonomics suggestions.

## Verification

cargo build --release 2>&1 | tail -5 | grep -q -E "Compiling|Finished"
target/release/termlink version --json 2>&1 | grep -qE '"mcp_tools":\s*(8[8-9]|9[0-9])'
grep -q '"termlink_agent_poll_start"' crates/termlink-mcp/src/tools.rs
grep -q '"termlink_agent_poll_vote"' crates/termlink-mcp/src/tools.rs
grep -q '"termlink_agent_poll_end"' crates/termlink-mcp/src/tools.rs
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
**Rationale:** First collaborative-decision surface on the MCP write side. Three tools form a complete poll lifecycle: start (open with question + options), vote (cast a choice — latest-wins per (poll_id, sender)), end (close, drop after-end votes). All thin write envelopes — `metadata.poll_options` is pipe-joined to match CLI T-1543's wire convention so the read-side aggregator (`agent poll-results`, T-1546) parses unchanged. Poll lifecycle previously required shelling out via three separate `termlink_exec` calls; now it's three native MCP calls.
**Evidence:**
- Build clean (4m 20s, batched with T-1569)
- `termlink version --json` reports mcp_tools=88 (was 84 after T-1568; T-1569 + T-1570 add 4 total)
- Verification gate 5/5 passed (3 grep checks + build + count)

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

### 2026-05-05T13:53:23Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1570-termlinkagentpoll-family--mcp-tools-for-.md
- **Context:** Initial task creation

### 2026-05-05T14:00:19Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
