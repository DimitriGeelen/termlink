---
id: T-1533
name: "agent threads — list all thread roots on chat-arc"
description: >
  agent threads — list all thread roots on chat-arc

status: work-completed
workflow_type: build
owner: human
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-05-05T09:14:59Z
last_update: 2026-05-05T09:21:40Z
date_finished: 2026-05-05T09:21:40Z
---

# T-1533: agent threads — list all thread roots on chat-arc

## Context

`cmd_channel_threads(topic, top, ...)` already exists: walks the arc, builds a thread index by `metadata.in_reply_to`, returns per-root thread metadata (root sender, child count, last_ts, depth). Operator workflow: "what conversations exist on chat-arc?" — companion to T-1509 `agent thread <root>` (subtree render). Thin wrapper hard-pinning topic to `agent-chat-arc` (~10 LOC). Optional `--top N` truncation.

## Acceptance Criteria

### Agent
- [x] New `AgentAction::Threads { top, hub, json }` variant in cli.rs
- [x] main.rs dispatch arm calling `cmd_channel_threads("agent-chat-arc", top, hub.as_deref(), json)`
- [x] `cargo build --release --bin termlink` clean
- [x] CLI `--help` shows `--top` / `--hub` / `--json`
- [x] Live smoke text: verb renders expected rows or empty-message
- [x] Live smoke JSON: returns parseable envelope

### Human
- [ ] [REVIEW] Verify the verb reads naturally
  **Steps:**
  1. `target/release/termlink agent threads`
  **Expected:** list of thread roots with child counts.
  **If not:** report layout suggestions.

## Verification

cargo build --release --bin termlink 2>&1 | tail -5 | grep -q -E "Compiling|Finished"
target/release/termlink agent threads --help 2>&1 | grep -q -- "agent"
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
**Rationale:** Closes the thread-index read primitive. Companion to T-1509 `agent thread`. `cmd_channel_threads` already does the work. Pure dispatch wrapper.
**Evidence:**
- Build clean
- Verification gate passed
- Live smoke: rendered output or empty-message

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

### 2026-05-05T09:14:59Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1533-agent-threads--list-all-thread-roots-on-.md
- **Context:** Initial task creation

### 2026-05-05T09:21:40Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
