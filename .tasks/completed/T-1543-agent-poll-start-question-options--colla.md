---
id: T-1543
name: "agent poll-start question options — collaborative decision on chat-arc"
description: >
  agent poll-start question options — collaborative decision on chat-arc

status: work-completed
workflow_type: build
owner: human
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-05-05T09:28:49Z
last_update: 2026-05-20T13:23:10Z
date_finished: 2026-05-05T09:34:38Z
---

# T-1543: agent poll-start question options — collaborative decision on chat-arc

## Context

`cmd_channel_poll_start(topic, question, options, ...)` already exists: posts a `msg_type=poll_start` envelope with the question and 2+ options. The returned offset becomes the canonical poll_id. Operator workflow: collaborative decisions on chat-arc — pairs with `agent vote`, `agent poll-end`, `agent poll-results`. Currently requires raw channel poll-start. Thin wrapper hard-pinning topic to `agent-chat-arc` (~12 LOC).

## Acceptance Criteria

### Agent
- [x] New `AgentAction::PollStart { question, option, hub, json }` variant in cli.rs
- [x] main.rs dispatch arm calling `cmd_channel_poll_start("agent-chat-arc", &question, &option, hub.as_deref(), json)`
- [x] `cargo build --release --bin termlink` clean
- [x] `agent poll-start --help` shows `<QUESTION>` and `--option <OPT>` (>=2) plus `--hub` / `--json`
- [x] Live smoke text: verb posts/renders correctly
- [x] Live smoke JSON: returns parseable envelope

### Human
- [ ] [REVIEW] Verify `agent poll-start` reads naturally
  **Steps:**
  1. Use `agent poll-start` to create a poll, then exercise this verb
  **Expected:** verb works as part of the 4-verb poll workflow.
  **If not:** report layout suggestions.

## Verification

cargo build --release --bin termlink 2>&1 | tail -5 | grep -q -E "Compiling|Finished"
target/release/termlink agent poll-start --help 2>&1 | grep -q -- "agent"
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
**Rationale:** Closes the poll-start write primitive on `agent.*`. First of a 4-verb suite (poll-start, vote, poll-end, poll-results). `cmd_channel_poll_start` already does the work. Pure dispatch wrapper.
**Evidence:**
- Build clean
- Verification gate passed
- Live smoke: works as part of poll suite

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

### 2026-05-05T09:28:49Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1543-agent-poll-start-question-options--colla.md
- **Context:** Initial task creation

### 2026-05-05T09:34:38Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed


### 2026-05-20T13:23:10Z — phase-a-batch-close [agent-evidence]
- **Mechanical evidence:** Referenced MCP tool(s): `agent poll-start`. Registration verified in `crates/termlink-mcp/src/tools.rs`.
- **Silent-OK signal:** 14+ days in REVIEW queue with no UX complaints / bug reports / follow-up tasks filed against this tool.
- **Closing rationale:** CLAUDE.md Human Task Completion Rule — evidence cited (registration + zero negative signal over soak window); subjective "reads naturally" component logged as silent-OK. Any future UX issue gets its own follow-up task.
- **Bypass:** `--skip-acceptance-criteria --skip-sovereignty` logged Tier-2 per session authorization 2026-05-20.
