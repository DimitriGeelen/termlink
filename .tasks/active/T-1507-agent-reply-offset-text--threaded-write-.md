---
id: T-1507
name: "agent reply offset text — threaded write verb on chat-arc"
description: >
  agent reply offset text — threaded write verb on chat-arc

status: work-completed
workflow_type: build
owner: human
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-05-05T06:52:40Z
last_update: 2026-05-05T06:58:44Z
date_finished: 2026-05-05T06:58:44Z
---

# T-1507: agent reply offset text — threaded write verb on chat-arc

## Context

T-1505 (`agent quote <offset>`) reveals parent/child structure on the chat-arc — `cmd_channel_quote` reads `metadata.in_reply_to` and renders the parent line. But there's been no agent-namespace write verb that creates that link. `agent reply <offset> <text>` closes the cohesion gap: post a note with `metadata.in_reply_to=<offset>` so subsequent `agent quote` / `agent on-thread` / Matrix-style traversal can see the reply chain. Reuses cmd_agent_post's focus-aware metadata resolution (thread, from_project) and delegates to cmd_channel_post with `reply_to: Some(offset)`.

## Acceptance Criteria

### Agent
- [x] New `AgentAction::Reply { offset, text, thread, project, msg_type, hub, json }` variant in cli.rs
- [x] main.rs dispatch arm calling `cmd_agent_reply`
- [x] New `cmd_agent_reply` in agent.rs: resolves focus thread + framework project, calls `cmd_channel_post(..., reply_to=Some(offset), ...)`
- [x] `cargo build --release -p termlink` clean
- [x] `agent reply --help` shows `<OFFSET>` and `<TEXT>` positionals plus `--thread` / `--project` / `--msg-type` / `--hub` / `--json`
- [x] Live smoke: reply to a real offset, then `agent quote <new-offset>` shows parent line
- [x] Live smoke: `--json` returns the post envelope including new offset
- [x] All existing tests still pass

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
- [ ] [REVIEW] Verify `agent reply` is operator-fluent
  **Steps:**
  1. `target/release/termlink agent timeline --window-secs 3600 --n 3` — note an `@<offset>`
  2. `target/release/termlink agent reply <offset> "thread test"`
  3. `target/release/termlink agent quote <new-offset-from-step-2>` — should show parent
  **Expected:** quote output has `> [<parent-offset>] ...` line above the new reply.
  **If not:** report the discrepancy.

## Verification

cargo build --release -p termlink 2>&1 | tail -5 | grep -q -E "Compiling|Finished"
target/release/termlink agent reply --help 2>&1 | grep -q -- "--hub"
target/release/termlink agent reply --help 2>&1 | grep -qi "OFFSET"
target/release/termlink agent reply --help 2>&1 | grep -qi "TEXT"
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
**Rationale:** Closes the chat-arc conversational cohesion loop opened by T-1505 quote rendering parents. `agent reply <offset>` is the inverse — write a child whose `metadata.in_reply_to=<offset>` makes it discoverable from `agent quote`, `cmd_channel_thread`, and `cmd_channel_replies`. Pure delegation: ~30 LOC mirror of `cmd_agent_post` with `reply_to: Some(offset)` threaded through. Smoke proven: posted offset 319 as reply to 318, then `agent quote 319` rendered parent line `> [318] ...` above child. Nested reply (320 → 319) also works, so reply chains compose.
**Evidence:**
- Build clean
- Verification gate 4/4 passed (build + 3 help-shape checks)
- Live smoke: reply offset 319 → quote shows `> [318] ...` parent line
- Live smoke JSON: `{delivered: {offset: 320, ts: ...}}`
- Reply chain composes (320 → 319 → 318)

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

### 2026-05-05T06:52:40Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1507-agent-reply-offset-text--threaded-write-.md
- **Context:** Initial task creation

### 2026-05-05T06:58:44Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
