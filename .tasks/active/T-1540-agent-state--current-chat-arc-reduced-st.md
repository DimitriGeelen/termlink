---
id: T-1540
name: "agent state — current chat-arc reduced state"
description: >
  agent state — current chat-arc reduced state

status: work-completed
workflow_type: build
owner: human
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-05-05T09:22:38Z
last_update: 2026-05-05T09:28:13Z
date_finished: 2026-05-05T09:28:13Z
---

# T-1540: agent state — current chat-arc reduced state

## Context

`cmd_channel_state(topic, include_redacted, ...)` already exists: walks the arc and returns the reduced "current visible state" — every chat post that hasn't been redacted, with edit overlays applied. Companion to T-1524 `agent info` (metadata view): info is the topic shape, state is the rendered conversation. Thin wrapper hard-pinning topic to `agent-chat-arc` (~10 LOC). `--include-redacted` to surface redacted markers too.

## Acceptance Criteria

### Agent
- [x] New `AgentAction::State { include_redacted, hub, json }` variant in cli.rs
- [x] main.rs dispatch arm calling `cmd_channel_state("agent-chat-arc", include_redacted, hub.as_deref(), json)`
- [x] `cargo build --release --bin termlink` clean
- [x] `agent state --help` shows `--include-redacted` / `--hub` / `--json`
- [x] Live smoke text: verb renders expected rows or empty-message
- [x] Live smoke JSON: returns parseable envelope

### Human
- [ ] [REVIEW] Verify `agent state` reads naturally
  **Steps:**
  1. `target/release/termlink agent state`
  **Expected:** rendered diagnostic output makes sense.
  **If not:** report layout suggestions.

## Verification

cargo build --release --bin termlink 2>&1 | tail -5 | grep -q -E "Compiling|Finished"
target/release/termlink agent state --help 2>&1 | grep -q -- "agent"
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
**Rationale:** Closes the reduced-state read primitive. Pairs with T-1524 `agent info`. `cmd_channel_state` already does the work. Pure dispatch wrapper.
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

### 2026-05-05T09:22:38Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1540-agent-state--current-chat-arc-reduced-st.md
- **Context:** Initial task creation

### 2026-05-05T09:28:13Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
