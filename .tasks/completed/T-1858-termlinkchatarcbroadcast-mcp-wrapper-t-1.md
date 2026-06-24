---
id: T-1858
name: "termlink_chat_arc_broadcast MCP wrapper (T-1856/T-1857 follow-on)"
description: >
  termlink_chat_arc_broadcast MCP wrapper (T-1856/T-1857 follow-on)

status: work-completed
workflow_type: build
owner: agent
horizon: null
tags: []
components: [crates/termlink-mcp/src/tools.rs]
related_tasks: []
created: 2026-05-28T22:19:59Z
last_update: 2026-05-28T22:21:47Z
date_finished: 2026-05-28T22:21:47Z
---

# T-1858: termlink_chat_arc_broadcast MCP wrapper (T-1856/T-1857 follow-on)

## Context

T-1856 shipped `scripts/chat-arc-broadcast.sh` and T-1857 added the
`/broadcast-chat` slash skill. The MCP layer of the interactive arc
already has wrappers for the four DISCOVERY corners (T-1839, T-1853,
T-1847, T-1852). The BROADCAST corner has no MCP wrapper — an agent
reasoning over MCP cannot fan a chat-arc message to the fleet without
shelling out. This task closes the last script→MCP gap in the arc.

Unlike the read-only wrappers, this is the FIRST mutating chat-arc
MCP tool (writes one envelope per hub). Tool description must surface
the write-side semantics so consuming agents don't accidentally
spam-broadcast.

## Acceptance Criteria

### Agent
- [x] `ChatArcBroadcastParams` struct added to `crates/termlink-mcp/src/tools.rs` with `payload: String` (required), `from: Option<String>`, `hubs_file: Option<String>`, `timeout_secs: Option<u64>`, `#[derive(Deserialize, JsonSchema)]`
- [x] `#[tool] async fn termlink_chat_arc_broadcast` method added on existing impl, reuses `resolve_t1836_script("chat-arc-broadcast.sh")` + `run_t1836_subprocess`
- [x] Tool description distinguishes this from the four read-only wrappers (T-1839/T-1847/T-1852/T-1853) by explicitly stating it MUTATES state on every reachable hub
- [x] Tool description references the sender resolution priority (--from / $TERMLINK_AGENT_ID / be-reachable.state) so agents understand attribution
- [x] `cargo check -p termlink-mcp` exits 0
- [x] `cargo build -p termlink-mcp` exits 0

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
grep -q "termlink_chat_arc_broadcast" crates/termlink-mcp/src/tools.rs
grep -q "ChatArcBroadcastParams" crates/termlink-mcp/src/tools.rs
cargo check -p termlink-mcp 2>&1 | tail -3 | grep -qE "Finished|warning"
cargo build -p termlink-mcp 2>&1 | tail -3 | grep -qE "Finished|warning"

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

### 2026-05-28T22:19:59Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1858-termlinkchatarcbroadcast-mcp-wrapper-t-1.md
- **Context:** Initial task creation

## Reviewer Verdict (v1.4)

- **Scan ID:** R-bfe1f655
- **Timestamp:** 2026-05-28T22:21:58Z
- **Catalogue:** v1.3-seed
- **Overall:** PASS
- **Needs Human:** yes
- **Findings:** none

- **Layer-1 escalations:** 1
  1. **external-publish** (high) — External publish or release
     - matched: `broadcast`

### 2026-05-28T22:21:47Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
